# nginx-wasm and BIG-IP TMM host quirks

The deployment target for these guests is NGINX (via nginx-wasm) or
BIG-IP TMM. Both run guests as a programmability layer on every
request. The following behaviours are specific to those hosts and do
not appear in the upstream `wstd` / `wasi-http` documentation. The
agent must respect them.

## Upstream response headers are not delivered (nginx-wasm)

When an nginx-wasm guest receives a `Response` it has not produced —
i.e., when nginx forwarded the request to an origin and is now passing
the response through the guest — the guest **does not see upstream
response headers**.

```rust
// On nginx-wasm: this is always None even if the upstream sent
// Content-Type. Tracked at nginx/nginx-wasm#63.
let ct = upstream_response.headers().get("content-type");
```

Implications:

- Code that branches on upstream response headers is silently always
  taking the missing-header branch.
- `Content-Type`-driven decisions must come from the **request**:
  `Accept` header, URI path, custom request header. Build that
  decision in the request phase, then carry it across to the response
  phase as part of the handler's local state.
- Response-body sniffing (magic bytes) is the fallback when request-
  phase signals are insufficient. Sniff before any expensive
  transformation; never re-decode the body twice (programmability-
  edge-rust, "Separate operations by cost").
- This rule is host-specific. On `wasmtime serve`, on BIG-IP TMM, and
  on other hosts, upstream response headers may be delivered. When a
  diff applies the rule, name the host in a code comment or in the
  diff summary so a reader on a different host knows it doesn't apply
  to them.

## Component instance is fresh per request (both hosts)

Each request gets a new wasm component instance. There is no
persistent in-guest memory across requests. Restated for emphasis:

- `static` / `OnceLock` / `LazyLock` populated on first request is
  *not* a cache — it is reset on the next request. Treat it as
  request-scoped.
- `Default::default()` is always valid initial state.
- Connection pools, prepared statements, decoders that warmup, regex
  caches — all of them die at the request boundary. If the cost
  matters, push it into early returns or out of the guest entirely.
- Ramifications for testing: tests that cover "behaviour on second
  request" must drive a second request through the host harness, not
  call the handler twice in-process.

## Single-threaded reactor (both hosts)

WASI 0.2 has no threads. The single reactor that `wstd::runtime::block_on`
sets up is the only execution context.

- Anything that blocks the OS thread (e.g. `std::thread::sleep`) is a
  full freeze of the request handler. The `wstd 0.6.x` reactor will
  fall back to a non-blocking pollable check if a CPU-heavy task
  yields (issue #73 / PR #78), but it cannot un-stick a non-yielding
  blocking call.
- `Send`/`Sync` bounds are noise on this target. Don't add them and
  don't remove them from `wstd` types that don't have them.

## "Proxy to origin" is a guest-issued HTTP call (both hosts)

The host does not provide a "let nginx forward this request" escape.
The guest issues an outbound `Client::send` to the origin and forwards
the response. This is the model `wstd::http::Client` is built for and
the shape `examples/http_server_proxy.rs` demonstrates.

- The "in-guest proxy" pattern is the supported way to express a
  forward-to-origin step.
- Splicing the request body straight from `server_req.into_body()` to
  `client_req.body(...)` is necessary for performance — the data does
  not transit guest memory (PR #50, PR #66).

## Headers corrected on every exit path (both hosts)

If the guest knows a corrected value for a response header (e.g. a
recomputed `Content-Length` after a body rewrite, a `Cache-Control`
that the guest is responsible for), set the header on **every** exit
path of the handler — including pass-through paths that don't modify
the body. This is a deployment-context rule that
`programmability-edge-rust` makes load-bearing; it is restated here so
the agent doesn't need to chase it across two refs.

## What the host owns vs what the guest owns

- Host: TLS termination, the accept loop, request parsing, log
  routing, deciding which route invokes the guest.
- Guest: the body of the request handler, including any "proxy to
  origin" step.
- Build glue (Cargo.toml flags, target features, wasmtime invocation
  flags, nginx config, TMM iRule glue) is host territory. If a guest
  change requires a host-side adjustment, name it in the diff
  summary; do not modify host config in a guest diff.
