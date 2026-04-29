# `wstd::http::Body` review reference

Distilled from `wstd/src/http/body.rs`. The body type is the highest-
churn surface in `wstd::http` (PR #92 was a major rewrite landing in
0.6) and the most likely to be misused. Use this when judging body
construction or consumption in a diff.

## What `Body` is

`wstd::http::Body` is a concrete type with three internal variants
(not exposed to the user):

- `Complete { data: Bytes, trailers: Option<HeaderMap> }` — bytes in
  memory.
- `Boxed(UnsyncBoxBody<Bytes, Error>)` — a generic `http_body::Body`.
- `Incoming(Incoming)` — backed by a wasi `incoming-body` resource.
  This variant has a fast path: forwarding it to an outgoing body
  uses wasi `splice` and never copies through wasm linear memory.

## Constructors

```rust
Body::empty()                                           // empty
Body::from(())                                          // empty (alias)
Body::from(b"…")            // &[u8]   — clones into Bytes
Body::from(vec![…])         // Vec<u8> — moves into Bytes
Body::from(bytes)           // Bytes
Body::from("…")             // &str    — clones
Body::from(string)          // String
Body::from_json(&t)?        // Serialize → JSON bytes (feature `json`)
Body::from_stream(s)        // S: Stream<Item: Into<Bytes>>
Body::from_try_stream(s)    // S: Stream<Item = Result<D, E>>
Body::from_http_body(b)     // any http_body::Body  (the only path supporting trailers)
AsyncInputStream::into()    // From<AsyncInputStream> for Body
```

Reviewer cues:

- For a one-shot owned response, `Body::from(bytes_or_str)` is the
  obvious choice.
- For trailers, `Body::from_http_body` is the *only* constructor;
  use `http_body_util::StreamBody::new(once_future(async move
  Frame::trailers(map)))` (see
  `wstd/examples/http_server.rs::http_echo_trailers`).
- To forward an `IncomingBody` to a `Client` or back as a
  `Response`, *do not* go through `into_boxed_body()` —
  `client_req.body(server_req.into_body())` keeps the splice path
  (PR #66).

## Consumers

```rust
body.contents().await?      // -> &[u8]    (collects everything)
body.str_contents().await?  // -> &str
body.json().await?          // -> T: Deserialize  (feature `json`)
body.into_boxed_body()      // -> UnsyncBoxBody<Bytes, Error>  (axum interop)
body.content_length()       // -> Option<u64>  from headers OR known size
```

Reviewer cues:

- `contents()` / `str_contents()` / `json()` take `&mut self` and
  cache the result internally — repeated calls are cheap, but they
  buffer the entire body. Don't recommend them for a hot-path
  pass-through.
- `content_length()` is `Some(_)` if the incoming side had
  `Content-Length`, or if the body was constructed from in-memory
  bytes. It's `None` for stream-shaped bodies.
- `into_boxed_body()` is the right interop point for ecosystem code
  that takes `impl http_body::Body`.

## Outbound (server) body — what `Responder::respond` does

When the user returns `Ok(Response<B>)` from `#[wstd::http_server]`,
`Responder::respond`:

1. Computes `wasi_headers` from `response.headers()`.
2. Converts `response.into_body().into() : Body`.
3. If `Body::content_length()` is `Some(len)`, **adds** a
   `Content-Length` header automatically (`server.rs`).
4. Constructs `OutgoingResponse`, sets the status code, opens the
   `OutgoingBody`, calls `ResponseOutparam::set` (this is when the
   host starts streaming the response back), then drives
   `body.send(wasi_body)` to write all body bytes and finish with
   trailers if any.

Reviewer cues:

- Don't manually set `Content-Length` for the in-memory
  variants — wstd does it.
- For streaming variants (`from_stream`, `from_http_body` over a
  stream), wstd cannot know the length and won't set it. If the
  user knows it, they can set it explicitly — but if they're wrong,
  `OutgoingBody::finish` returns an `ErrorCode` (wasi-http archived
  PR #71).
- Headers attached to the response are immutable as soon as the
  body is attached and the `OutgoingResponse` is built (wasi-http
  archived PR #70). Mutate via `response.headers_mut()` *before*
  the response leaves the user's hand.

## Incoming body — life-cycle

```text
Request<Body> from #[wstd::http_server]
   │  body is `Body::Incoming(Incoming { body, size_hint })`
   │
   ├─ `body.contents().await?` ─────────► reads to EOF, caches Bytes
   ├─ `body.str_contents().await?`  ───► same, then UTF-8 decode
   ├─ `body.json().await?` ────────────► same, then serde_json
   └─ `body.into_boxed_body()` ────────► UnsyncBoxBody (use http_body)
```

Once consumed, the incoming body cannot be re-read — it is a
forward-only stream.

Stream-end is *not* an error (wstd issue #12, PR #42). When the
underlying wasi `input-stream` returns `StreamError::Closed`,
`AsyncRead::read` returns `Ok(0)`, `Stream::next` returns `None`,
and `Body::contents` returns the data accumulated so far.

## Trailers

The trailers come *after* the body bytes. To read them you must drive
the body to completion:

```rust
let collected = body.into_boxed_body().collect().await?;
let bytes = collected.to_bytes();
let trailers = collected.trailers().cloned();
```

(`wstd/examples/http_client.rs`,
`wstd/examples/complex_http_client.rs`,
`wstd/examples/http_server.rs::http_echo_trailers`.)

To *write* trailers, only `Body::from_http_body` works:

```rust
let body = http_body_util::StreamBody::new(once_future(async move {
    anyhow::Ok(Frame::<Bytes>::trailers(trailers))
}));
Body::from_http_body(body)
```

Reviewer cues:

- A user who wants trailers but constructs `Body::from(bytes)` will
  silently drop them (no compile error). Flag.
- An empty `Some(HeaderMap::new())` and `None` for trailers are
  semantically equivalent (wasi-http archived #131). Don't
  introduce a special case.
