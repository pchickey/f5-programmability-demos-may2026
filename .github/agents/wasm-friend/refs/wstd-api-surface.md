# wstd API surface — quick reference

A module-by-module map of the parts of `wstd` an implementer reaches
for. Anything not listed below is either re-exported from the `http`
crate (`HeaderMap`, `Method`, `StatusCode`, `Uri` …), an internal-
only type, or covered by a separate ref. Versions: this is accurate
for `wstd 0.6.x`.

## `wstd::http`

Top-level re-exports:

- `Body`, `BodyExt` — body type and ergonomics. Construct via
  `Body::empty()`, `Body::from_json(&v)`, `Body::from_stream(s)`,
  `Body::from_try_stream(s)`, `Body::from_http_body(b)`, or
  `From<&[u8] | Vec<u8> | Bytes | &str | String | AsyncInputStream>`.
  Consume via `body.contents().await?` (bytes), `body.str_contents().await?`
  (utf-8), `body.json::<T>().await?` (json), `body.into_boxed_body()`
  (interop with hyper/axum), `body.content_length()` (Option<u64>).
- `Client` — outbound HTTP. `Client::new()`, then optional
  `set_connect_timeout` / `set_first_byte_timeout` /
  `set_between_bytes_timeout`. `client.send(req).await ->
  Result<Response<Body>, Error>`. `Client` is `Clone` but cheap —
  construct fresh per request.
- `Error`, `Result` — `Error = anyhow::Error`. Use `.context(...)` for
  layering. Downcast WASI errors via
  `error.downcast_ref::<wstd::http::error::ErrorCode>()`.
- `ErrorCode` — re-export of `wasip2::http::types::ErrorCode`. Variants
  the agent will see in real responses: `ConnectionRefused`,
  `ConnectionTimeout`, `ConnectionTerminated`, `DnsError(...)`,
  `HttpRequestUriInvalid`, `HttpRequestMethodInvalid`,
  `HttpResponseTimeout`, `InternalError(Option<String>)`.
- `HeaderMap`, `HeaderName`, `HeaderValue` — re-exports from `http`.
  Case-insensitive lookups; values are bytes via `as_bytes()`.
- `Method`, `StatusCode`, `Uri`, `Authority`, `PathAndQuery`,
  `Scheme`, `InvalidUri` — re-exports from `http`.
- `Request`, `Response` — re-exports from `http`. Use
  `Request::get(uri).body(b)?` / `Request::post(uri).body(b)?` /
  `Request::builder().uri(...).method(...).header(k,v).body(b)?`.

Submodules:

- `wstd::http::body` — `Body`, `BodyHint`, `IncomingBody`,
  `InvalidContentLength`, plus `util::*` re-exports of
  `http_body_util::*` (`Full`, `StreamBody`, `BodyExt`, `Empty`,
  `combinators::*`).
- `wstd::http::server` — `Responder`. Used by the macro; user code
  rarely touches it. `Responder::respond(response)` auto-adds
  `Content-Length` if the body has a known length;
  `Responder::fail(err)` reports an error to the host without
  panicking afterward (PR #117).
- `wstd::http::request` — `Builder`, `Request` re-exports. Internal
  `try_into_outgoing` / `try_from_incoming` for the macro.
- `wstd::http::response` — `Builder`, `Response` re-exports.

## `wstd::io`

- `AsyncRead`, `AsyncWrite` — traits with `read(&mut self, &mut [u8])
  -> Result<usize>`, `read_to_end(&mut self, &mut Vec<u8>) -> Result<usize>`,
  `write(&mut self, &[u8]) -> Result<usize>`, `write_all`, `flush`.
  `read` returning `0` is EOF (Rust convention).
- `AsyncInputStream`, `AsyncOutputStream` — wrappers around WASI
  streams that own a single subscription pollable for the stream's
  lifetime. Both implement `AsyncRead`/`AsyncWrite`. `AsyncInputStream::copy_to(&AsyncOutputStream)`
  uses the `splice` fast path.
- `copy(reader, writer)` — generic copy that short-circuits to
  `splice` when both sides expose the underlying streams.
- `empty()` -> `Empty` — readable: `Ok(0)` immediately; writable:
  swallows bytes.
- `stdin()`, `stdout()`, `stderr()` — async stdio handles.
- `Cursor`, `BufReader`-style helpers — none yet (issue #23 tracks
  buffered streams). Use `Body::contents()` for bulk reads.
- `Error`, `Result` — re-exports of `std::io::Error` / `Result`.

## `wstd::runtime`

- `block_on(fut)` — drives a future to completion using the singleton
  reactor. Cannot be nested. Since `wstd 0.6.0`, the future does not
  need to be `'static` (PR #108).
- `Reactor::current()` — returns a clone of the active reactor;
  panics if called outside `block_on`. User code rarely needs this;
  it's how internal types schedule pollables.
- `AsyncPollable`, `WaitFor`, `Task` — the async-task surface.
  `spawn(fut)` is available but treat it as the abstraction of last
  resort (issue #17).

## `wstd::time`

- `Duration` — wraps a wasi `monotonic_clock::Duration` (nanoseconds).
  `Duration::from_secs(n)`, `from_millis(n)`, `from_micros(n)`,
  `from_nanos(n)`, `from_secs_f32(f)`. Methods `as_secs`,
  `as_millis`, `as_micros`, `as_nanos`. Convertible to/from
  `std::time::Duration`.
- `Instant` — monotonic instant. `Instant::now()`,
  `instant.duration_since(other) -> Duration`, `instant.elapsed()`.
- `SystemTime` — wall clock; `SystemTime::now()` and
  `Into<std::time::SystemTime>`.
- `Timer::after(d)`, `Timer::at(i)`, `Timer::never()`,
  `timer.wait().await -> Instant`. `Timer::set_after(&mut self, d)` for
  reuse.
- `interval(d) -> Interval` — async iterator firing every `d`.

## `wstd::task`

- `sleep(d).await` — non-blocking sleep, wraps `Timer::after(d).wait()`.
- `sleep_until(i).await` — deadline form, wraps `Timer::at(i).wait()`.

## `wstd::future`

- `FutureExt::timeout(deadline)` — wraps any future; returns
  `io::Result<F::Output>`. Deadline is anything that
  `IntoFuture<Output = ()>` (e.g. `Duration`, `Instant`).
- `FutureExt::delay(deadline)` — delays the future's start until
  the deadline.

## `wstd::iter`

- `AsyncIterator` — minimal trait for async iteration; powers
  `TcpListener::incoming()`, `Interval`.

## `wstd::net`

- `TcpListener::bind(addr).await -> Result<TcpListener>`.
- `TcpListener::incoming() -> impl AsyncIterator<Item =
  Result<TcpStream>>`.
- `TcpStream::connect(addr).await -> Result<TcpStream>` (added in
  `wstd 0.6.0`, PR #96).
- `TcpStream` implements `AsyncRead`/`AsyncWrite` and exposes
  `peer_addr`, `local_addr`.

## `wstd::rand`

- `wstd::rand::*` — wrappers around `wasi:random`. Available but
  rarely needed in proxy guests.

## `wstd::prelude`

- Re-exports `FutureExt`, `AsyncRead`, `AsyncWrite`. Use sparingly;
  prefer named imports.

## Macros

- `#[wstd::main]` — async CLI entry. Function must be `async fn main()
  -> T`; takes no arguments. Generates a sync `main` that calls
  `block_on`. Does *not* set a non-zero exit code on `Err` (issue
  #109).
- `#[wstd::http_server]` — async HTTP-server entry. Function must be
  `async fn main(req: Request<Body>) -> Result<Response<Body>, Error>`.
  Generates the `wasi:http/proxy` export and runs `block_on` per
  request.
- `#[wstd::test]` — async test attribute. Function must be `async fn`,
  no arguments. Generates a sync `#[test]` that calls `block_on`.
  Cannot be nested inside another `block_on` (issue #113).

## What `wstd` does *not* expose (yet)

- Async filesystem (issue #93). Use `std::fs` from outside hot
  paths if needed; in a request-phase guest, prefer no filesystem at
  all.
- Buffered I/O (issue #23). Use `Body::contents()` for bulk reads.
- Query-parameter builder (issue #68). Build the query string by
  hand or via the `serde_qs` crate.
- Multi-task `spawn_blocking`-style escape (issue #113). The reactor
  is single-threaded.

When a `wstd` surface is missing for the task at hand, drop down to
`wasip2::*` and leave a one-line comment naming the gap.
