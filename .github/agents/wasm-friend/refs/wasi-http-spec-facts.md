# wasi-http spec facts the agent must obey

The operational subset of the wasi-http spec a guest author needs.
Sourced from the active `WebAssembly/WASI` repository (proposals/http)
and the merged-and-archived `WebAssembly/wasi-http` repository, with
the historical reasoning compressed out. When this ref disagrees with
something the agent reads in `sources/wasi-http/`, the active
`WebAssembly/WASI` repo wins; `sources/wasi-http/` is archived and
out of date.

## World

- `wasi:http/proxy` is the world a guest exports. A guest implementing
  it provides exactly one function: `incoming-handler.handle(request,
  response-outparam)`.
- The host owns the accept loop, instantiates a fresh component per
  request, and calls `handle` once per request.
- `wasi:cli/command` is a separate world for non-HTTP CLI components;
  a guest cannot implement both worlds in the same component.

## Header (`fields`) semantics

- **Names are case-insensitive on lookup**, but the case used at
  insertion is preserved on the wire (RFC 9110 alignment, wasi-http
  issue #117). `http::HeaderMap::get("content-type")` and
  `.get("Content-Type")` return the same value.
- **Values are bytes**, not strings. The wit type is `list<u8>` and
  values may legitimately contain non-ASCII (wasi-http issue #28).
  `HeaderValue::as_bytes()` is the correct accessor; `.to_str()` is
  fallible and may fail on non-UTF-8.
- **`fields.get(name)` returns the empty list if the header is
  absent**, not `None` (wasi-http issue #56 / #82). A header may
  legitimately have an empty value (`Accept-Encoding: ` per RFC 9110
  §12.5.3); to distinguish "absent" from "present, empty", use
  `headers.contains_key(name)` separately.
- **Multiple values for the same name are list-shaped**:
  `headers.get_all(name)` returns the iterator. The host preserves
  the order they came in over the wire.
- **`field-key` is the deprecated name; `field-name` is canonical**
  as of WASI 0.2.2 (wasi-http issue #107). Use `HeaderName` and
  `field-name` in any user-facing wording. Don't introduce
  `field-key` even if old C bindings still expose it.
- **Forbidden headers** (e.g. `Set-Cookie` controls, certain hop-by-
  hop) — the host may reject these via `HeaderError::Forbidden` from
  `Fields::set` / `append`. `wstd::http::header_map_to_wasi` surfaces
  the error with context — let it propagate via `?`.

## Trailer semantics

- A response (or request) body may have an `option<trailers>`. There
  is **no semantic difference between `Some(empty fields)` and
  `None`** on the wire (wasi-http issue #131). Pick whichever is
  ergonomic. `wstd`'s `Body::from_*` constructors mostly produce
  `None`; `Body::from_http_body` with `BodyExt::with_trailers` is the
  way to attach trailers explicitly.
- `IncomingBody::finish(body)` returns a `FutureTrailers`. The
  trailers can only be read **after** the body stream has been fully
  consumed. `wstd::http::Body::into_boxed_body().collect().await?`
  takes care of this; the trailers are then available via
  `.trailers().cloned()` on the collected result.
- `OutgoingBody::finish(body, trailers)` sends the trailers; calling
  it without finishing the body stream first is an error (the host
  will trap or return `HttpResponseTrailerSectionSizeReason`).
  `wstd::http::Body::send` and `Responder::respond` order this
  correctly.

## Status codes

- `set-response-outparam` takes `result<outgoing-response,
  error-code>`. An `Ok(response)` with a 4xx/5xx status is a normal
  response; an `Err(error-code)` is "the guest failed before
  producing a response" (wasi-http issue #14, #44, #49). The two
  paths are distinct on the wire.
- The `#[wstd::http_server]` macro maps a user `Ok(response)` to the
  first; a user `Err(_)` to the second via `Responder::fail`.
- A status code outside `100..=599` is rejected by the wasi-http host
  (wasi PR #849). `http::StatusCode` validates this client-side.

## Methods and schemes

- Standard methods are enumerated as `Method::Get`, `Post`, …; non-
  standard methods come through as `Method::Other(String)`.
- Standard schemes are `Http`, `Https`; non-standard schemes come
  through as `Scheme::Other(String)`. `wstd` defaults the outgoing
  scheme to `Https` if the URI omits it.
- `outgoing_handler::handle` returns `ErrorCode::HttpRequestUriInvalid`
  for an unsupported scheme (e.g. `ftp://`) — see PR #121 in `wstd`
  and the test `tests/http_handle_error_code.rs`. The agent should
  match and surface this rather than panic.

## Request options

- `request-options.set-connect-timeout(duration)`,
  `set-first-byte-timeout(duration)`, and
  `set-between-bytes-timeout(duration)` accept a `wasi:clocks/types/duration`
  (nanoseconds). `wstd::http::Client::set_*_timeout` take a
  `Duration` and convert.
- The host may reject any of these as unsupported (`return Err(())`).
  `wstd::http::Client::wasi_options` surfaces this as
  `"wasi-http implementation does not support … option"`.
- The historical `*-ms` naming was a refactoring leftover; current
  wit names omit the `-ms` suffix (wasi-http issue #78).

## Future-incoming-response and future-trailers

Both follow the pattern `get -> Option<Result<Result<T, ErrorCode>,
()>>` where:

- The outer `Option` is "is it ready yet?". `None` means not ready;
  the caller must wait on the subscribed `Pollable`.
- The middle `Result<..., ()>` is "have you already called `get`
  successfully?". `Err(())` means the value was already consumed.
- The inner `Result<T, ErrorCode>` is "did the operation succeed?".

`wstd::http::Client::send` and `wstd::http::Body::send` already wrap
both layers; user code never inspects this directly.

## Body limits

- The host may set an upper bound on total body size; if exceeded,
  `outgoing_handler::handle` returns
  `ErrorCode::HttpResponseBodySize(Some(bytes))` or one of the size-
  exceeded variants. The agent does not need to enforce a limit
  client-side — let the host's error surface.

## What the spec leaves to the implementation

- Connection pooling between requests is host-controlled; the guest
  cannot rely on or detect connection reuse.
- The wire-level HTTP version (1.1, 2, 3) is host-controlled; guest
  code is portable across all of them. Don't write code that depends
  on a specific version.
- Header ordering on outgoing responses is preserved by the host in
  insertion order (best-effort). Don't rely on a specific order for
  multi-valued headers.
