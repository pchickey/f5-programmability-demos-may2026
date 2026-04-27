# wstd / wasi-http error model

Distilled from `wstd/src/http/error.rs`,
`wstd/tests/http_handle_error_code.rs`, `wstd` PRs #117 and #121,
wasi-http archived #49. Use this when reviewing how a guest reports
or handles failures.

## Layered picture

```
┌────────────────────────────────────────────────────────────────────┐
│ User code                                                          │
│   `wstd::http::Result<T> = std::result::Result<T, anyhow::Error>`  │
│                                                                    │
│   Returns from #[wstd::http_server]: Result<Response<Body>, Error> │
│   Returns from Client::send       : Result<Response<Body>, Error>  │
│   Returns from Body::contents/json: Result<&[u8] / &str / T, Error> │
└────────────────────────────────────────────────────────────────────┘
                                │
                                │ anyhow::Error wraps:
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│  wasip2::http::types::ErrorCode  ←  outgoing_handler::handle       │
│  std::io::Error                  ←  body / stream I/O              │
│  http::header::Invalid*          ←  header / method / scheme parse │
│  serde_json::Error               ←  json features                  │
│  custom strings via anyhow!()                                       │
└────────────────────────────────────────────────────────────────────┘
```

`wstd::http::Error` is `anyhow::Error`, so all of the above flow up
the same channel. Downcast at the leaf to react.

## Server side: what happens when `async fn main` returns `Err`

The `#[wstd::http_server]` macro wraps the user function in:

```rust
match __run(request).await {
    Ok(response) => responder.respond(response).await.unwrap(),
    Err(err) => responder.fail(err),
}
```

`Responder::fail(err)` looks for an `ErrorCode` in the chain:

```rust
let e = match err.downcast_ref::<ErrorCode>() {
    Some(e) => e.clone(),
    None    => ErrorCode::InternalError(Some(format!("{err:?}"))),
};
ResponseOutparam::set(self.outparam, Err(e));
```

PR #117 removed the post-fail panic — there is no longer an `unwrap`
after `responder.fail`. **Per intent, do not reintroduce one.** The
host is responsible for surfacing the error to its caller (e.g.
wasmtime serve responds with HTTP 500); the guest's job is to set
the outparam to `Err(...)` and exit cleanly.

Reviewer rule: a guest that panics rather than returning `Err(_)`
loses the `ErrorCode` channel — the host sees a wasm trap, which is
opaque to upstream. `crit-6`.

## Client side: `Client::send`

```rust
let res = wasip2::http::outgoing_handler::handle(wasi_req, opts)?;
//                                                              ^ propagates wasi-http ErrorCode
```

The `?` here is in `wstd::http::Client::send` (`wstd/src/http/client.rs`).
Before PR #121 this was `.unwrap()` and a malformed scheme would
trap; after #121, the `ErrorCode` is propagated as
`anyhow::Error`-wrapped.

Test pattern (`wstd/tests/http_handle_error_code.rs`):

```rust
let request = Request::get("ftp://example.com/").body(Body::empty())?;
let result = Client::new().send(request).await;
assert!(result.is_err());
let error = result.unwrap_err();
assert!(error.downcast_ref::<ErrorCode>().is_some());
```

Reviewer rule: client-side, downcast to `ErrorCode` to react;
re-propagate (`?`) if the caller can handle generic errors.

## What is *not* an `ErrorCode`

A 4xx or 5xx response from the remote arrives as
`Ok(IncomingResponse)` with the appropriate status code. **It is not
an error.** Per wasi-http archived #49 and the RFC 9209 alignment:

> Local errors shouldn't be mimicked as fake responses with status
> codes, otherwise you cannot distinguish WASI errors from real
> errors received from the remote.

Conversely, a real-remote-down (DNS failure, connection refused,
TLS error, timeout, etc.) arrives as `Err(ErrorCode::*)`.

The reviewer cue: when a guest pattern-matches on `result.status()
== StatusCode::INTERNAL_SERVER_ERROR` and assumes "the network is
down," that's only sometimes right. A real connection failure
arrives as `Err`, and matching on a 500 misses it.

## `anyhow::Context` discipline

`wstd` consistently attaches context strings on every conversion
boundary:

```rust
// wstd/src/http/request.rs::try_from_incoming
let headers: HeaderMap = header_map_from_wasi(incoming.headers())
    .context("headers provided by wasi rejected by http::HeaderMap")?;
```

Apply the same in guest code:

```rust
let body = response.into_body().contents().await
    .context("collecting upstream response body")?;
```

Reviewer rule: a `?` on a wasi or http-crate error that loses the
where-it-failed signal is `style-7`. Especially valuable when the
guest's caller (the host) only sees the final error string.

## Common ErrorCode handling shapes

```rust
match Client::new().send(req).await {
    Ok(resp) => /* ... */,
    Err(err) => match err.downcast_ref::<ErrorCode>() {
        Some(ErrorCode::ConnectionRefused) => /* upstream is down */,
        Some(ErrorCode::ConnectionReadTimeout) | Some(ErrorCode::HttpResponseTimeout) =>
            /* upstream is slow — return 504 */,
        Some(ErrorCode::HttpRequestUriInvalid) | Some(ErrorCode::HttpRequestMethodInvalid) =>
            /* configuration bug — log + return 500 */,
        _ => /* fall through */,
    }
}
```

A guest that surfaces every `ErrorCode` as a 500 to the upstream is
acceptable (and is the default if you just `?`-propagate). A guest
that wants 502/504 distinction must downcast.

## What about `std::io::Error`?

Body / stream operations expose `std::io::Error`. Common kinds:

- `ErrorKind::ConnectionReset` — the wasi stream was closed mid-write
  (`wstd/src/io/streams.rs::AsyncOutputStream::write` translates
  `StreamError::Closed`).
- `ErrorKind::Interrupted` — not used by `wstd` for ordinary stream
  end (which is `Ok(0)`); reserved for spurious cases.
- `ErrorKind::TimedOut` — emitted by `wstd::future::FutureExt::timeout`
  when the wrapped future doesn't complete in time
  (`wstd/tests/http_timeout.rs`).

Reviewer rule: a body write that returns `ConnectionReset` means the
upstream tore down the connection. The guest can't recover — log
and return.

## Don't recommend p3 errors

The `wasip3` crates have their own error-shape evolution
(`http_compat`, `IncomingMessage`, `body_writer::*`, etc.). All of
that is out of scope per intent. Don't recommend p3 patterns when
discussing error handling.
