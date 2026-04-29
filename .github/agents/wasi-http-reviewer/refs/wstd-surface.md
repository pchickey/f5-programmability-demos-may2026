# wstd public surface (review-time map)

Distilled from `wstd/src/lib.rs` and the module heads. Use this as a
mental map when judging whether a diff is reaching for the right
abstraction.

## Top-level modules

| Module                  | Purpose                                                    |
|-------------------------|------------------------------------------------------------|
| `wstd::future`          | `FutureExt`: `.timeout`, etc.                              |
| `wstd::http`            | HTTP types + body + client + server.                       |
| `wstd::io`              | `AsyncRead`, `AsyncWrite`, `copy`, stdio, streams, cursor. |
| `wstd::iter`            | `AsyncIterator` trait.                                     |
| `wstd::net`             | `TcpListener`, `TcpStream`. (No UDP yet — issue #95.)      |
| `wstd::rand`            | `random_bytes` / wasi random.                              |
| `wstd::runtime`         | `block_on`, `Reactor`, `AsyncPollable`, `spawn`, `Task`.   |
| `wstd::task`            | `task::sleep` ergonomic alias.                             |
| `wstd::time`            | `Timer`, `Duration`, `Instant`, `SystemTime`, `interval`.  |

## Macros

- `#[wstd::main]` — wrap `async fn main()` for a CLI guest. Calls
  `block_on` under the hood.
- `#[wstd::http_server]` — wrap `async fn main(req: Request<Body>) ->
  Result<Response<Body>, Error>` for an HTTP guest. Generates the
  `wasi:http/incoming-handler.Guest` impl and the
  `wasip2::http::proxy::export!` invocation.
- `#[wstd::test]` — wrap `async fn` test bodies in `block_on`.
  *Footgun:* `#[ignore]` is consumed by the macro, doesn't reach the
  generated `#[test]` (issue #111). Apply `#[ignore]` *after*
  manually expanding if you need it.

## `wstd::http`

- Re-exports from the `http` crate: `Method`, `StatusCode`, `Uri`,
  `Authority`, `PathAndQuery`, `HeaderMap`, `HeaderName`,
  `HeaderValue`, `Request<B>`, `Response<B>`.
- `Body` — `wstd::http::Body`; constructed from `()`, `&[u8]`,
  `Vec<u8>`, `Bytes`, `&str`, `String`, `AsyncInputStream`, an
  `http_body::Body`, or a `Stream`. See `body.rs` distillation.
- `Client` — `Client::new()`, `client.send(req).await`,
  `client.set_connect_timeout(d)`,
  `client.set_first_byte_timeout(d)`,
  `client.set_between_bytes_timeout(d)`.
- `Error = anyhow::Error`. `Result<T> = std::result::Result<T,
  Error>`. `error::ErrorCode` is the wasi-http
  `wasip2::http::types::ErrorCode` (downcast for outbound client
  errors).
- `server::Responder` — used by the `#[wstd::http_server]` macro.
  Internal user code rarely instantiates this directly.
- `body::BodyExt` — re-export of `http_body_util::*` (`.collect()`,
  `.boxed_unsync()`, `.with_trailers()`, etc.).

Constants worth using: `http::header::{CONTENT_LENGTH, CONTENT_TYPE,
LOCATION, …}` for canonical header names.

## `wstd::io`

- `AsyncRead`, `AsyncWrite` traits with `read_to_end` /
  `write_all` defaulted in.
- `AsyncInputStream` / `AsyncOutputStream` — wrappers over wasi
  `input-stream` / `output-stream` with optimised `splice`-based
  `copy_to`.
- `copy(reader, writer).await` — uses `splice` when both ends are
  stream-typed (PR #50).
- `stdin()`, `stdout()`, `stderr()` — `AsyncRead`/`AsyncWrite`-bearing
  wrappers.
- `Cursor`, `empty()` — re-exports / wrappers similar to std.

## `wstd::runtime`

- `block_on(fut)` — drive `fut` to completion. Cannot be nested
  (issue #113). `fut` does *not* need to be `'static` (PR #108);
  internally the future is spawned `unsafe`-ly and dataflow-bound to
  the call.
- `Reactor::current()` — handle to the running reactor. Panics
  outside `block_on`. Use `.schedule(pollable)` to register a
  `Pollable` for waiting and `.spawn(fut)` to spawn an additional
  `'static` task.
- `AsyncPollable::wait_for()` — future that resolves when the
  pollable is ready.
- `spawn(fut: 'static)` — `Task<T>` you can `.await`. Tasks are
  cooperatively scheduled (PRs #71, #78, #86).

## `wstd::time`

- `Duration` and `Instant`: opaque wrappers over wasi
  `monotonic-clock` types (PR #53 dropped the Deref leak).
- `Timer::after(d)`, `Timer::at(deadline)`, `Timer::never()`. Get a
  future via `.wait().await`.
- `interval(d)` — `AsyncIterator<Item = Instant>`.
- `wstd::task::sleep(d).await` — short alias for
  `Timer::after(d).wait().await`.
- `SystemTime::now()` from `wall-clock`, convertible to
  `std::time::SystemTime` (PR #120).

## `wstd::__internal::wasip2`

`wstd` re-exports the `wasip2` crate under
`wstd::__internal::wasip2` for use only by the `wstd-macro`
generated code (PR #91). It is `#[doc(hidden)]` and not part of the
semver-public surface. **Don't reach into it from user code.** If
the user needs raw bindings, depend on the `wasip2` crate directly.

## What `wstd` does *not* expose (drop to `wasip2`)

- Imperative reads/writes on `wasi:http/types::*` resource
  methods that aren't surfaced through `Body` / `Client` /
  `Responder`.
- The `RequestOptions` resource's full mutable surface (only the
  three timeout setters are wrapped on `Client`).
- `wasi:cli/*` aside from stdio (env, args, exit, terminal,
  preopens).
- `wasi:filesystem/*` (open issue #93).
- UDP (open issue #95).

When the user must drop to `wasip2`, the canonical example is
`crates/wasip2/examples/http-proxy.rs` from `wasi-rs`.
