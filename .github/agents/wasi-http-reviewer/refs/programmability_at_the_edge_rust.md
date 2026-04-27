# wasm programmability in the proxy context Development Guide


---

## Architecture: Content handler

The guest code is handling the request and response completely. If there is a need to "proxy" to an origin server, this must be expressed as a http request to that origin server. This "in guest" proxy pattern will be a divergence from how people generally think of programmability for a proxy context.

---

## Principle: Do the Least Work Possible

Proxy programmability sits in the hot path of every request. Optimise for the common
case where the code has nothing to do, and make every more-expensive
step conditional on the cheaper step being inconclusive.

### Cost hierarchy

1. **Free** — reading values already on `parts` (headers, URI, status)
2. **Cheap** — small string operations, header parsing, magic-byte sniffing
3. **Moderate** — reading/buffering the full body (`body.contents().await`)
4. **Expensive** — decoding, transforming, re-encoding body content

### Early-return cascade

Structure the response handler as a series of early returns, ordered by
cost. Each step is only reached if the cheaper check above it could not
resolve the decision:

```
// 1. Check if any work was requested at all        (free)
// 2. Seed state from response headers              (free)
// 3. Compare against desired state → bail if done  (trivial)
// 4. Read body bytes                               (moderate — only if needed)
// 5. Cheap metadata extraction from body bytes     (cheap)
// 6. Re-compare → bail if now satisfied            (trivial)
// 7. Expensive operation                           (single pass)
```

Never do an expensive operation just to check whether that operation is
needed, when a cheaper signal (headers, magic bytes) can answer the
question first.

### Avoid unnecessary allocations

- Do not `.to_vec()` or `.clone()` body bytes just to unify return
  types. Use early returns instead — a few repeated lines of header
  setting are cheaper than copying megabytes.
- Prefer `&mut self` methods over consuming-self methods on state types
  to avoid forcing callers to `.clone()` before a call that might fail.
- If a function signature forces the caller to copy data, fix the
  signature.

### Avoid redundant work across phases

If the response handler will eventually do an expensive operation on the
body (decode, parse, transform), do not also perform that same expensive
operation in a preceding "check" step. Split detection into a cheap
probe and an expensive operation. Run only the cheap probe before
deciding; the expensive operation, if needed, handles its own pass.

---

## Request Phase: Parsing Inputs

Parse parameters from headers, URI query strings, or other request
metadata during the **request** phase. Build the plugin's state
incrementally:

- Start from `None`.
- Each recognised input promotes it to `Some(State::new().field(v))`.
- Use `unwrap_or_else(State::new)` so the first input creates the state
  and subsequent inputs fold into it.
- Silently ignore unrecognised keys and unparseable values. The proxy
  should be lenient with input it does not own.

Extract input parsing into a standalone helper function when it involves
string splitting or iteration. It operates at a different abstraction
level than the SDK filter plumbing and is easier to extend and test in
isolation.

---

## Response Headers

### Correcting or setting headers

If the guest code has better knowledge of a response header value than the
origin provided (or the origin omitted it), set the header on **every**
exit path — not just the path where the guest code modifies the body. Early
returns that pass the body through unmodified should still carry
corrected headers.

---

### nginx-wasm response header limitations

The WASM guest does **not** receive response headers from the upstream.
Calling `parts.headers.get("content-type")` on an upstream response
returns `None` even if the upstream sent that header. This is an
nginx-wasm limitation tracked at
<https://github.com/nginx/nginx-wasm/issues/63>.

**Implications for guest code design:**
- Do **not** use response `Content-Type` (or other response headers)
  to decide whether to transform the body — the value will always be
  `None`. Use request-phase signals (Accept header, URI path, custom
  header) instead.

### Separate operations by cost

If a functions are created to to operations like extracting metadata from raw bytes, provide separate
entry points for cheap extraction (e.g. sniffing a header/magic bytes)
and expensive extraction (e.g. full decode). Let the caller choose the
level of work. Do not bundle them into a single method.

### Prefer `&mut self` for fallible operations

Builder-style consuming methods (`fn foo(self) -> Self`) are fine for
initial construction. For methods that might fail or that the caller
may want to retry, take `&mut self` to avoid forcing a `.clone()`.

---

## Instance Lifetime

WASM components are instantiated **fresh per request** — never reused.
`Default::default()` is always valid initial state. There is no need
to reset fields between requests, implement cleanup logic, or worry
about stale state from previous requests.

This also means expensive initialization (like compiling regexes)
happens on every request. If that cost is significant, consider
whether the work can be avoided via early returns.
