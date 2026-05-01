# Copilot Instructions

## Repository overview

Each numbered directory is an independent Cargo project demonstrating F5 programmability: Rust compiled to `wasm32-wasip2` and deployed as an HTTP request handler inside NGINX (via `nginx-wasmtime`) or BIG-IP TMM on an F5 UDF lab environment.

| Directory | Demo |
|-----------|------|
| `00-introduction` | Lab orientation |
| `01-hello-world` | Minimal HTTP handler |
| `02-read-write-headers` | Request/response header manipulation |
| `03-redact-response-body` | Body rewrite (redaction) |
| `04-more-body-mechanisms` | Body read, transform, forward |
| `05-weather-demo` | Outbound HTTP + JSON parsing |
| `06-image-transcode` | Image processing with embedded assets |
| `07-llm-metrics-aggregator` | Outbound HTTP fan-out + aggregation |
| `08-reverse-response-body` | Streaming body reversal |
| `98-kv-store` | Host KV-store integration |
| `99-example-origin` | Mock upstream (not a wasm guest) |

This repository is actively changing, so update this file often.

## Build and deploy

Each demo is its own Cargo project. Work inside the demo directory.

```sh
cd 01-hello-world
../common/build.sh                  # cargo build --target wasm32-wasip2
../common/run.sh --nginx            # upload to NGINX via platypus
../common/run.sh --bigip            # upload to BIG-IP TMM via platypus
```

The VS Code **Build**, **Run in NGINX**, and **Run in BIG-IP** tasks in each demo's `.vscode/tasks.json` invoke these scripts automatically.

`platypus` POSTs the `.wasm` binary to the appropriate wasmtime endpoint:
- NGINX: `http://10.1.1.4:9000/services?name=<binary-name>` (env: `PLATYPUS_NGINX`)
- BIG-IP: `http://10.1.1.4:9001/services?name=<binary-name>` (env: `PLATYPUS_BIGIP`)

No test suite exists; validation is manual via HTTP requests to the deployed service.

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
      "command": "../common/build.sh",
      "group": { "kind": "build", "isDefault": true }
    },
    {
      "label": "Run in NGINX",
      "type": "shell",
      "command": "../common/run.sh --nginx",
      "group": "test",
      "dependsOn": ["Build"]
    },
    {
      "label": "Run in BIG-IP",
      "type": "shell",
      "command": "../common/run.sh --bigip",
      "group": "test",
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
- **`platypus`** orchestrates NGINX + wasm services on the UDF host. Systemd services in `.udf/` manage startup. BIG-IP TMM is also supported; see `common/run.sh`.
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

<!-- BEGIN agentbonk-scope:wasm-friend -->
The `wasm-friend` Copilot custom agent (`.github/agents/wasm-friend.agent.md`),
its reference files under `.github/agents/wasm-friend/refs/`, and the
companion skills under `.github/skills/` are loaded on demand when the
user invokes `/agent wasm-friend` or one of the bundled `/<cmd>` skills. Do
not summarize or inline their contents into this file — that duplicates
content that updates with the agent.
<!-- END agentbonk-scope:wasm-friend -->

## Code review and implementation

Use the bundled `wasi-http-reviewer` agent to review changes:

```
/review-wasi-http          # standard diff review
/critique-wasi-http        # hard-look pass (every corpus-licensed concern)
/remediation-plan-wasi-http  # sequenced fix plan from findings
```

Use the bundled `wasm-friend` agent to implement or propose changes:

```
/wasm-friend-implement     # implement the task as a diff with edits and summary
/wasm-friend-propose       # draft a structured proposal — no edits performed
```

Reference material for both agents (canonical shapes, hazard list, host constraints) lives in `.github/agents/wasi-http-reviewer/refs/` and `.github/agents/wasm-friend/refs/`.
