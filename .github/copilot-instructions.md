# Copilot Instructions

## Repository overview

Each numbered directory (`01-hello-world`, `02-read-write-headers`, …) is an independent Cargo project. They demonstrate F5 programmability: Rust code compiled to `wasm32-wasip2` and deployed as HTTP request handlers inside NGINX (via `nginx-wasmtime`) on an F5 UDF lab environment.

This repository is actively changing, so update this file often.

## Build and deploy

Each demo is its own Cargo project. Work inside the demo directory.

```sh
cd 01-hello-world
cargo build --target wasm32-wasip2
# Deploy the compiled binary to platypus:
curl "http://10.1.1.4:9000/services?name=hello-world" \
  --data-binary "@target/wasm32-wasip2/debug/hello-world.wasm"
```

The VS Code "Run (in NGINX)" task in `.vscode/tasks.json` builds and deploys automatically via direct curl.

`platypus` POSTs the `.wasm` binary to NGINX's wasmtime module at `http://10.1.1.4:9000/services?name=<service-name>`. No test suite exists; validation is manual via HTTP requests to the deployed service.

## Creating a new demo

Each demo needs these files alongside `Cargo.toml` and `src/main.rs`:

`.vscode/tasks.json`:
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build",
      "type": "shell",
      "command": "cargo build --target wasm32-wasip2",
      "group": { "kind": "build", "isDefault": true }
    },
    {
      "label": "Run (in NGINX)",
      "type": "shell",
      "command": "curl \"http://10.1.1.4:9000/services?name=<service-name>\" --data-binary \"@target/wasm32-wasip2/debug/<service-name>.wasm\"",
      "group": "test",
      "presentation": { "reveal": "always", "panel": "new" },
      "dependsOn": ["Build"]
    }
  ]
}
```

`Cargo.toml` only needs `wstd = "0.6.6"` as a dependency. The entry point is always `#[wstd::http_server]` on `async fn main`.

## Architecture

- **Guest model**: each request starts a fresh Wasm component instance. There is no cross-request memory — no need to reset state, no point caching resources between requests.
- **Entry point macro**: `#[wstd::http_server]` generates the `incoming_handler::Guest` impl. The handler signature is always:
  ```rust
  async fn main(req: Request<Body>) -> Result<Response<Body>, Error>
  ```
- **Runtime**: single-threaded `wstd` reactor. `tokio`/`smol`/`async-std` do not link. Never use `std::thread::sleep` or blocking I/O inside `async`.
- **Deployment target**: `wasm32-wasip2` (passed via `--target` flag or set in `.vscode/tasks.json`). Never target `wasm32-wasip1`.
- **`platypus`** orchestrates NGINX + wasm services on the UDF host. Systemd services in `.udf/` manage startup.
- **`99-example-origin`** is a mock upstream server (not an nginx-wasm guest demo) that exposes `/people.json`, `/show_headers`, `/image.png`, `/metrics.json`, and `/reader`. Demos that call outbound HTTP target it at `http://10.1.1.4:8001/`.

## Key conventions

- **Use `wstd::http` for all HTTP types** (`Request`, `Response`, `Body`, `HeaderName`, `HeaderValue`, `Error`). Drop to raw `wasip2` bindings only when `wstd` does not expose the surface.
- **Two response construction patterns**: `Response::new(body)` for simple cases; `Response::builder().status(StatusCode::NOT_FOUND).header("key", "val").body(body)?` for responses with custom status/headers. `wstd::http` re-exports `StatusCode` from the `http` crate.
- **Reading a body as text**: `body.str_contents().await` (or `body.bytes_contents().await` for raw bytes). These consume the body and must be awaited.
- **Propagate errors with `?`**, never `.unwrap()`. A guest panic aborts the request with no signal upstream; always return `Err(anyhow::anyhow!("…"))` instead.
- **No cross-request resource caching**: don't put `Client` or handles in `static` / `OnceLock` / thread-locals. Per-request construction is free; the component model recycles.
- **Single-threaded**: use `Rc<RefCell<T>>` (not `Arc<Mutex<T>>`) for any within-request shared mutable state.
- **Header lookup is case-insensitive** via `wstd::http::HeaderMap` — no need to normalise keys.
- **Stream, don't copy**: forward bodies with `Response::new(req.into_body())` or `wstd::io::copy` to keep `splice`. Buffering with `.to_vec()` copies through wasm linear memory on every request.
- **Set response headers on every exit path**, not just the "did work" branch.
- **Decide on request-phase signals** (URI, request headers) when targeting nginx-wasm. Response headers from the upstream are not delivered to the guest.
- **Don't set `Content-Length` manually** for in-memory bodies — wstd sets it automatically.
- **A 4xx/5xx HTTP response is `Ok(response)`**, not an error. Network failures (timeout, connection refused) arrive as `Err(ErrorCode::*)`.
- **Static assets**: `include_bytes!` and `include_str!` work fine for embedding files at compile time.

## Crate compatibility

**Works:** `serde`, `serde_json`, `bytes`, `http`, `http_body_util`, `futures_lite`, `futures_concurrency`, `regex`, `chrono`, `uuid`, `anyhow`, `log` + `env_logger`.

**Does not link:** `tokio` and anything that depends on it (`axum`, `hyper`, `tower`, `mio`), `async-std`, `smol`.

Common substitutions: `std::thread::sleep` → `wstd::task::sleep(d).await`; `reqwest::Client` → `wstd::http::Client`; `tokio::spawn` → `wstd::runtime::spawn`; `tokio::join!` → `(a, b).join().await` (futures-concurrency).

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
