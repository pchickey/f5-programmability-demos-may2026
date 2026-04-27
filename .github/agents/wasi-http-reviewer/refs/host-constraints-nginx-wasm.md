# Host-specific constraints (NGINX / BIG-IP)

Distilled from `programmability_at_the_edge_rust.md` and the linked
upstream issues. Cite this when reviewing diffs whose target host is
NGINX (via nginx-wasm) or BIG-IP, especially when a finding hinges
on a host-specific behaviour the upstream `wstd` / `wasi-http`
sources don't cover.

## Fresh-per-request instance model

Both NGINX and BIG-IP instantiate the wasm component **fresh per
request**. There is no instance reuse, no pooling, no warm path that
preserves state between requests.

Consequences:

- `Default::default()` is always valid initial state.
- No need for cleanup logic, drain caches, or reset fields between
  requests.
- Expensive initialization (compiling regexes, parsing config) runs
  per request. If the cost is significant, push the work into an
  early-return guard so it only runs when needed.
- `static` caches do not survive requests. (Reaffirmed by intent's
  "don't worry about cross-request memory state.")
- The p3 instance-reuse discussion (wasi #898) is a host-side
  *optimisation*; per intent it is out of scope, and even when it
  lands a guest must not rely on it for correctness.

Cite this when finding a `static`-cached resource (rubric `crit-1`).

## nginx-wasm: upstream response headers are not delivered

**The single most important nginx-wasm-specific limitation.** When
running on nginx-wasm, the wasm guest does **not** receive response
headers from the upstream. Calling `parts.headers.get("content-type")`
on an upstream response returns `None` even if the upstream sent
that header.

Tracked at https://github.com/nginx/nginx-wasm/issues/63.

Implications:

- **Don't decide body transformations based on response
  `Content-Type` (or any response header).** The value will always
  be `None`. Use request-phase signals instead:
  - `Accept`
  - URI path
  - Custom request header set by upstream config
  - Method
  - Query parameters
- A guest that branches on response headers will silently take the
  default branch on every request — not a runtime error, just a
  silent "transformation never fires."

Cite this when:

- The host is named (CLAUDE.md, intent, diff comment) as nginx-wasm
  → `crit-9` (block).
- The host is BIG-IP or unspecified → `warn` ("non-portable to
  nginx-wasm").

## Content-handler architecture (both hosts)

Both NGINX and BIG-IP expect the guest to **handle the request and
response completely**. There is no host-managed "forward to origin"
hook; "proxy to origin" must be expressed as a guest-issued
outgoing HTTP call (`wstd::http::Client::send`).

This is a divergence from how proxy programmability is normally
modelled — in proxy-wasm (the predecessor), the host provides
`proxy_http_call` for side calls and the request flows through the
filter chain on its own. In wasi-http on these hosts, there is
**one** outgoing channel: the guest's `outgoing-handler::handle`.

Implications:

- A "proxy"-shaped guest is a `Client::send` consumer; see
  `wstd/examples/http_server_proxy.rs`.
- Don't recommend host-specific bypass APIs. They don't exist in
  wasi-http v0.2.
- The p3 `service`/`middleware`/`origin` proposal (wasi #793) would
  someday offer two outgoing channels, but is out of scope per intent.

## Hot-path discipline (both hosts)

Both NGINX and BIG-IP run the wasm guest on **every** request through
the proxy. Cumulative cost matters. The expert source defines the
canonical structure:

### Cost hierarchy

1. **Free** — values already on `parts` (headers, URI, method, status).
2. **Cheap** — small string ops, header parsing, magic-byte sniff.
3. **Moderate** — buffering the full body (`body.contents().await`).
4. **Expensive** — decoding, transforming, re-encoding body content.

### Early-return cascade

```text
1. Check if any work was requested at all          (free)
2. Seed state from request headers                  (free)
3. Compare against desired state → bail if done    (trivial)
4. Read body bytes                                  (moderate — only if needed)
5. Cheap metadata extraction from body bytes       (cheap)
6. Re-compare → bail if now satisfied               (trivial)
7. Expensive operation                              (single pass)
```

Reviewer cues:

- Don't run an expensive operation just to *check* whether the
  expensive operation is needed.
- Don't `.clone()` or `.to_vec()` body bytes to unify return types.
  Early returns are cheaper than copying megabytes (`perf-1`).
- Helper functions that probe metadata should split *cheap*
  (header / magic-byte sniff) and *expensive* (full decode) entry
  points; let the caller choose (`perf-4`, pattern P18).

## Response-header rule (both hosts)

If the guest knows a response header value better than the origin
(or the origin omitted it), set the header on **every** exit path
of the handler — including the pass-through path that doesn't
modify the body.

Reviewer rule: a `Content-Disposition`, `X-Content-Type-Options`,
or `Cache-Control` correction added only to the "we transformed
this" branch leaks the origin's wrong/missing value out the
pass-through branch. `arch-4`.

## Request-phase parsing

Parse parameters from headers, URI query, or other request metadata
during the request phase. Build state incrementally:

- Start from `None`.
- Each recognised input promotes it to `Some(State::new().with_x(v))`.
- Use `unwrap_or_else(State::new)` to fold subsequent inputs in.
- **Silently ignore** unrecognised keys and unparseable values. The
  proxy is lenient with input it does not own.

Reviewer rule: a guest that 4xx-rejects unknown `x-` headers, an
extra query parameter, or one malformed cookie pair is `arch-3`.

## What to *not* worry about

- Cross-request state. Fresh per request; no leftover.
- Multi-threading. Single-threaded reactor; no `Send`/`Sync`
  bounds, no `Mutex` lock contention.
- `tokio` / `smol` / `async-std`. Don't link in this target.
- `wasi-preview-3`. Out of scope per intent.

## Naming the host in findings

- If the diff is unambiguously targeted at nginx-wasm (Cargo metadata,
  comments, deployment script), say "nginx-wasm" in the finding.
- If targeted at BIG-IP, say "BIG-IP."
- If unspecified, name the assumed host and qualify ("if this is
  intended for nginx-wasm, …"); don't escalate a host-specific
  finding to `block` without confirming the host.

This matters because the same finding can be `crit` on one host and
`warn` on another (e.g. P12 / `crit-9` is `crit` on nginx-wasm and
`warn` elsewhere because of upstream header delivery).
