# Common substitutions

A lookup table for "the user wrote X; what's the right thing in a
`wasm32-wasip2` guest?" Distilled from the intent's "Reach for `wstd`
abstractions" priority and from `wstd`'s public surface.

When a finding cites a substitution, the message should explain *why*
in dynamic / scripting-language terms (intent: target reviewer may not
be a Rust expert) — and link to the `wstd` API.

## Time / sleep

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `std::thread::sleep(d)`                     | `wstd::task::sleep(d).await` or `wstd::time::Timer::after(d).wait().await` |
| `tokio::time::sleep(d).await`               | same                                              |
| `tokio::time::timeout(d, fut)`              | `wstd::future::FutureExt::timeout(fut, d).await`  |
| `std::time::Instant::now()`                 | `wstd::time::Instant::now()`                      |
| `std::time::SystemTime::now()`              | `wstd::time::SystemTime::now()` (or `into()` to `std::time::SystemTime`) |

`std::thread::sleep` blocks the single-threaded reactor; nothing else
makes progress while it runs (`crit-3`).

`tokio::time::sleep` doesn't link in `wasm32-wasip2`.

## Concurrency / async runtime

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `#[tokio::main]`                            | `#[wstd::main]`                                   |
| `tokio::runtime::Runtime::new()?.block_on(...)` | `wstd::runtime::block_on(...)`                |
| `tokio::spawn(fut)`                         | `wstd::runtime::spawn(fut)` (requires `'static`)  |
| `tokio::join!(a, b)`                        | `(a, b).join().await` (futures-concurrency) or `futures_lite::future::zip(a, b)` |
| `tokio::try_join!(a, b)`                    | `futures_lite::future::try_zip(a, b).await`       |
| `tokio::select!`                            | `(a, b).race().await` (futures-concurrency)       |
| `Arc<Mutex<T>>` for cross-task state        | `Rc<RefCell<T>>` (single-threaded) or just `&mut` |
| custom executor (`smol::block_on`, …)       | `wstd::runtime::block_on`                         |

`tokio` does not link in `wasm32-wasip2` (`crit-2`). Mention this
explicitly when recommending the substitution; the user has likely
seen `tokio` examples and assumes it works.

## I/O

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `std::fs::*`                                | (no async filesystem yet — wstd issue #93. Use raw `wasip2::filesystem::*` if necessary, or stay in memory.) |
| `tokio::fs::*`                              | same                                              |
| `std::net::TcpStream`                       | `wstd::net::TcpStream`                            |
| `tokio::net::TcpStream`                     | `wstd::net::TcpStream`                            |
| `std::net::TcpListener`                     | `wstd::net::TcpListener`                          |
| `tokio::io::AsyncReadExt`                   | `wstd::io::AsyncRead` (`read`, `read_to_end`)     |
| `tokio::io::AsyncWriteExt`                  | `wstd::io::AsyncWrite` (`write`, `write_all`, `flush`) |
| `tokio::io::copy(a, b)`                     | `wstd::io::copy(a, b)` (uses wasi `splice`)       |
| `std::io::Stdin/Stdout`                     | `wstd::io::stdin()/stdout()`                      |

The runner CLI flag for TCP listening is `-S inherit-network=y`, not
the deprecated `-S tcplisten` (`wstd` issue #67).

## HTTP

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `reqwest::Client`                           | `wstd::http::Client`                              |
| `hyper::Client`                             | `wstd::http::Client`                              |
| `ureq::*`                                   | `wstd::http::Client`                              |
| Hand-rolled wit `OutgoingRequest::new(...)` etc. | `wstd::http::Request::builder()...body(...)?` + `Client::send` |
| Hand-rolled `incoming_handler::Guest` impl  | `#[wstd::http_server]` macro                      |
| `Vec<(String, String)>` for headers         | `http::HeaderMap` (re-exported by `wstd::http`)   |
| `String` literals for header names          | `http::header::CONTENT_LENGTH` / `CONTENT_TYPE` / etc. |
| `serde_json::from_slice(body_bytes)?`       | `body.json::<T>().await?` (with `json` feature)    |
| `serde_json::to_vec(&t)?` then `Body::from` | `Body::from_json(&t)?`                            |

If the user really needs something `wstd` doesn't expose, depend on
the `wasip2` crate directly — not on `wstd::__internal::wasip2`,
which is private.

## Random / crypto

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `rand::thread_rng()`                        | seed from `wstd::rand` / `wasip2::random::random` or `getrandom` (which targets wasi-random under the hood) |
| `OpenSSL` / `rustls` direct                 | `wstd::http::Client` does TLS via the host; don't link rustls in a wasi-http guest |

## Logging

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `println!(...)`                             | works (routes through wasi-cli stdio)             |
| `eprintln!(...)`                            | works (routes through wasi-cli stderr)            |
| `tracing::info!(...)`                       | not yet supported via host integration; use `log` + `env_logger` (`wstd` issue #103: log-crate ecosystem just works) |
| `log::info!(...)`                           | works after `env_logger::init()`                  |

## Error handling

| User wrote                                  | Use instead                                       |
|---------------------------------------------|---------------------------------------------------|
| `result.unwrap()` on a `Client::send`       | `?` to propagate, then downcast to `ErrorCode` if reacting |
| `panic!("...")` in handler                  | return `Err(anyhow::anyhow!("..."))` so the macro calls `Responder::fail` |
| `thiserror::Error` for an opaque error type | `anyhow::Error` (matches wstd's convention)       |

## What you can *keep* using

These work in `wasm32-wasip2` guests; don't recommend changing them:

- `serde`, `serde_json` — pure Rust, no syscalls.
- `bytes`, `http`, `http_body`, `http_body_util` — already in
  `wstd`'s dep tree; ecosystem-standard types.
- `futures_lite`, `futures_concurrency`, `pin_project_lite`,
  `pin_project`, `async_task` — the async toolkit `wstd` itself
  uses.
- `regex`, `chrono`, `time`, `uuid` (with appropriate features) —
  pure Rust.
- `clap` — the `complex_http_client` example uses it.

## What does *not* link

- `tokio` (and most things that depend on it: `axum` 0.x, `tower`,
  `hyper`, `mio`, `tokio-tungstenite`).
- `async-std`, `smol`, `actix-rt`.
- `mio`, `socket2` direct usage of native sockets.
- `libsqlite3-sys` (and most native-C dependencies that don't ship
  a `wasm32-wasip2` build).
- `rustls` in many configurations (depends on the crypto backend
  selected and getrandom version).

If a `Cargo.toml` in the diff lists any of these for a wasi-http
guest target, that's an `arch` smell — even behind a feature flag.

## Footnote on `axum` / ecosystem ports

Per `wasi-rs` issue #107 (maintainer pchickey): the long-term plan is
that ecosystem crates will get `#[cfg(target_family="wasm",
target_env = "p2")]` paths that depend on `wasip2` directly. In the
meantime, an `axum_wstd` shim (`axum_wstd = { package = "axum",
version = "..." }`) is the recommended bridge if a project really
wants axum semantics. Don't recommend porting `axum` itself — that's
out of scope for guest review.
