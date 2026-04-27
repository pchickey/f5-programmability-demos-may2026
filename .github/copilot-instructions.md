# Copilot Instructions

## Repository overview

Each numbered directory (`01-hello-world`, `02-read-write-headers`, …) is an independent Cargo project. They demonstrate F5 programmability: Rust code compiled to `wasm32-wasip2` and deployed as HTTP request handlers inside NGINX (via `nginx-wasmtime`) on an F5 UDF lab environment.

This repository is actively changing, so update this file often.

## Build and deploy

Each demo is its own Cargo project. Work inside the demo directory.

```sh
cd 01-hello-world
cargo build           # compile to wasm32-wasip2 (target set in .cargo/config.toml)
cargo run             # build + deploy to UDF via .cargo/runner.sh
```

The runner script (`runner.sh`) POSTs the compiled `.wasm` binary to the `platypus` service manager at `http://10.1.1.4:9000/services?name=<service-name>`. No test suite exists; validation is manual via HTTP requests to the deployed service.

## Creating a new demo

Each demo needs these files alongside `Cargo.toml` and `src/main.rs`:

`.cargo/config.toml`:
```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "./.cargo/runner.sh"
```

`.cargo/runner.sh`:
```bash
#!/bin/bash
set -ex
curl "http://10.1.1.4:9000/services?name=<service-name>" --data-binary "@$1"
```

`Cargo.toml` only needs `wstd = "0.6.6"` as a dependency. The entry point is always `#[wstd::http_server]` on `async fn main`.

## Architecture

- **Guest model**: each request starts a fresh Wasm component instance. There is no cross-request memory — no need to reset state, no point caching resources between requests.
- **Entry point macro**: `#[wstd::http_server]` generates the `incoming_handler::Guest` impl. The handler signature is always:
  ```rust
  async fn main(req: Request<Body>) -> Result<Response<Body>, Error>
  ```
- **Runtime**: single-threaded `wstd` reactor. `tokio`/`smol`/`async-std` do not link. Never use `std::thread::sleep` or blocking I/O inside `async`.
- **Deployment target**: `wasm32-wasip2` (set in `.cargo/config.toml` in each demo). Never target `wasm32-wasip1`.
- **`platypus`** orchestrates NGINX + wasm services on the UDF host. Systemd services in `.udf/` manage startup.

## Key conventions

- **Use `wstd::http` for all HTTP types** (`Request`, `Response`, `Body`, `HeaderName`, `HeaderValue`, `Error`). Drop to raw `wasip2` bindings only when `wstd` does not expose the surface.
- **Propagate errors with `?`**, never `.unwrap()`. A guest panic aborts the request with no signal upstream; always return `Err(anyhow::anyhow!("…"))` instead.
- **No cross-request resource caching**: don't put `Client` or handles in `static` / `OnceLock` / thread-locals. Per-request construction is free; the component model recycles.
- **Header lookup is case-insensitive** via `wstd::http::HeaderMap` — no need to normalise keys.
- **Stream, don't copy**: forward bodies with `Response::new(req.into_body())` or `wstd::io::copy` to keep `splice`. Buffering with `.to_vec()` copies through wasm linear memory on every request.
- **Set response headers on every exit path**, not just the "did work" branch.
- **Decide on request-phase signals** (URI, request headers) when targeting nginx-wasm. Response headers from the upstream are not delivered to the guest.
- **Don't set `Content-Length` manually** for in-memory bodies — wstd sets it automatically.
- **A 4xx/5xx HTTP response is `Ok(response)`**, not an error. Network failures (timeout, connection refused) arrive as `Err(ErrorCode::*)`.

## Crate compatibility

**Works:** `serde`, `serde_json`, `bytes`, `http`, `http_body_util`, `futures_lite`, `futures_concurrency`, `regex`, `chrono`, `uuid`, `anyhow`, `log` + `env_logger`.

**Does not link:** `tokio` and anything that depends on it (`axum`, `hyper`, `tower`, `mio`), `async-std`, `smol`.

Common substitutions: `std::thread::sleep` → `wstd::task::sleep(d).await`; `reqwest::Client` → `wstd::http::Client`; `tokio::spawn` → `wstd::runtime::spawn`.

`println!`/`eprintln!` route through wasi-cli stdio and work as-is. Use `log` + `env_logger` for structured logging; `tracing` has no host integration.

<!-- BEGIN agentbonk-scope:wasi-http-reviewer -->
The `wasi-http-reviewer` Copilot custom agent (`.github/agents/wasi-http-reviewer.agent.md`),
its reference files under `.github/agents/wasi-http-reviewer/refs/`, and the
companion skills under `.github/skills/` are loaded on demand when the
user invokes `/agent wasi-http-reviewer` or one of the bundled `/<cmd>` skills. Do
not summarize or inline their contents into this file — that duplicates
content that updates with the agent.
<!-- END agentbonk-scope:wasi-http-reviewer -->

## Code review

Use the bundled `wasi-http-reviewer` agent or the prompts in `.github/prompts/` to review changes:

```
/review-wasi-http
/critique-wasi-http
/remediation-plan-wasi-http
```

Reference material for the reviewer (canonical shapes, hazard list, host constraints) lives in `.github/agents/wasi-http-reviewer/refs/`.
