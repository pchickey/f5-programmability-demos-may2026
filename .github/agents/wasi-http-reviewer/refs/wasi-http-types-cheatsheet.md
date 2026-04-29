# `wasip2::http::types` cheatsheet

Distilled from `wasi-rs/crates/wasip2/src/lib.rs` + the canonical raw
proxy example. Use this when judging a diff that drops to raw
`wasip2` bindings (because the diff genuinely needs something `wstd`
doesn't expose, or because the diff *shouldn't* have dropped down).

The full `wasip2` is procedurally generated and ~500KB; this is just
the names and the contracts a reviewer needs.

## Crate layout

```
wasip2::http::types::*           // resource types and free functions
wasip2::http::outgoing_handler   // outbound: handle(req, options)
wasip2::http::proxy::export!     // generate proxy-world export glue
wasip2::exports::http::incoming_handler::Guest  // server trait
wasip2::cli::*                   // stdio, env, args, exit, terminal
wasip2::clocks::*                // monotonic, wall, timezone
wasip2::io::{streams, poll, error}  // foundational io, pollables, errors
```

## HTTP resource types

| Resource                | Role                                                  |
|-------------------------|-------------------------------------------------------|
| `Fields`                | The `headers` / `trailers` map. Mutable until attached to a body. |
| `IncomingRequest`       | Server side: handed to the `Guest::handle` impl.      |
| `IncomingResponse`      | Client side: result of `outgoing_handler::handle`.    |
| `OutgoingRequest`       | Client side: built by guest, passed to `outgoing_handler::handle`. |
| `OutgoingResponse`      | Server side: built by guest, passed to `ResponseOutparam::set`. |
| `IncomingBody`          | Body associated with an `IncomingRequest` / `IncomingResponse`. Single-shot stream. |
| `OutgoingBody`          | Body associated with an outgoing message. Write-once. |
| `RequestOptions`        | Knobs: `connect-timeout`, `first-byte-timeout`, `between-bytes-timeout`. Resource so it can be extended. |
| `ResponseOutparam`      | Server side: the slot the host gave you for setting the response. Set exactly once. |
| `FutureIncomingResponse`| Client side: poll for the response. |
| `FutureTrailers`        | Read trailers after the body stream is consumed. |
| `Pollable`              | (`wasi:io/poll`) Subscribe to readiness.    |

## Resource lifetime ordering (load-bearing)

Children must drop **before** parents:

- `OutgoingResponse → OutgoingBody → OutputStream`. Drop the
  `OutputStream` (`drop(out)`) **before** `OutgoingBody::finish(body,
  trailers)`. Calling `finish` while a stream is still live traps.
- `IncomingResponse → IncomingBody → InputStream`. Drop the
  `InputStream` before `IncomingBody::finish(body)` (which yields
  the `FutureTrailers`).
- `IncomingBody → InputStream` likewise (drop input-stream before
  consuming the body to get trailers).
- `Stream / Body → Pollable`. The `Pollable` returned by
  `subscribe()` is a child of the resource it was taken from. Drop
  the pollable first.

`wstd` orders these for the user via field ordering inside its
internal types (`AsyncInputStream` keeps `subscription:
OnceLock<AsyncPollable>` *before* `stream` to ensure the subscription
drops first). When a diff hand-rolls these structs, look for
inversions.

## Canonical raw-bindings server (from `wasi-rs/crates/wasip2/examples/http-proxy.rs`)

```rust
use std::io::Write as _;
use wasip2::http::types::{Fields, IncomingRequest, OutgoingBody,
                          OutgoingResponse, ResponseOutparam};

wasip2::http::proxy::export!(Example);

struct Example;

impl wasip2::exports::http::incoming_handler::Guest for Example {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
        let resp = OutgoingResponse::new(Fields::new());
        let body = resp.body().unwrap();         // first call only — second traps

        ResponseOutparam::set(response_out, Ok(resp));   // *** response starts streaming now ***

        let mut out = body.write().unwrap();     // first call only — second traps
        out.write_all(b"Hello, WASI!").unwrap();
        out.flush().unwrap();
        drop(out);                               // *** drop output-stream before finish ***

        OutgoingBody::finish(body, None).unwrap();
    }
}
```

Reviewer cues:

- `OutgoingResponse::body()` returns `Result<OutgoingBody, ()>`
  whose `Err` means "body was already taken." Same for
  `OutgoingBody::write()`.
- `ResponseOutparam::set` is the line where the response *begins
  streaming back to the host* (wasi-http archived #18 / PR #19).
  Anything you want in the response head must be set *before* this
  call. Body bytes follow.
- `OutgoingBody::finish` finalises the body and (with `Some(trailers)`)
  attaches trailers. It returns an `ErrorCode` if the
  `Content-Length` header was set and bytes-written disagrees
  (wasi-http archived PR #71).

## Canonical raw outbound

```rust
let req = OutgoingRequest::new(Fields::new());
req.set_method(&Method::Get).unwrap();
req.set_scheme(Some(&Scheme::Https)).unwrap();
req.set_authority(Some("example.com")).unwrap();
req.set_path_with_query(Some("/")).unwrap();

let body = req.body().unwrap();                    // OutgoingBody
let stream = body.write().unwrap();                // OutputStream
// ... (write bytes if any) ...
drop(stream);
OutgoingBody::finish(body, None).unwrap();

let fut = wasip2::http::outgoing_handler::handle(req, None /* options */).unwrap();
// poll fut.subscribe() until ready, then fut.get() returns the IncomingResponse
```

(`wstd::http::Client::send` is the wrapper around this whole shape.)

## Fields semantics (load-bearing for any header finding)

- **Equality is case-insensitive** (`fields.get("foo")` matches a
  field appended as `Foo`). PR #121 archived.
- **Original case is preserved** through `entries()` and on the
  wire. Don't lowercase in user code.
- **`fields.get("missing")` returns `[]`** (empty list); a
  present-but-empty field returns `[""]`. Use `fields.has(name)`
  for unambiguous presence (PR #91 archived).
- **`fields.set(name, [])` is equivalent to `fields.delete(name)`**
  (current consensus on wasi #900; not yet specified in
  v0.2 wit but is reflected in the implementation behaviour).
- **Field values are bytes, not UTF-8.** `field-value: list<u8>`
  (#28 archived); HTTP allows obs-text. `wstd` uses
  `http::HeaderValue` which has bytes accessors.
- **`headers` attached to an `outgoing-{request,response}` are
  immutable** once the body has been opened (PR #70 archived).
  Mutate before opening the body / before `ResponseOutparam::set`.
- **Field-name range** is RFC-9110 `field-name` (visible US-ASCII
  with HTAB / SP) — non-ASCII names should be rejected (wasi #786,
  pending spec text).

## Error model

`outgoing_handler::handle` returns `Result<FutureIncomingResponse,
ErrorCode>` where `ErrorCode` is the wasi-http error type. The
guidance from wasi-http archived #49 (RFC 9209-aligned):

- A 4xx / 5xx from a real remote arrives as `Ok(IncomingResponse)`
  with the appropriate status code. **It is not an `ErrorCode`.**
- An `ErrorCode::*` is a transport / proxy-internal failure
  *between* the client and the remote — DNS failure, connection
  refused, TLS error, etc. ("Proxy-Status errors are emitted only
  by proxies and other intermediaries.")

Common `ErrorCode` variants the reviewer will see:

- `DnsError(...)`, `DnsTimeout`
- `DestinationNotFound`, `DestinationUnavailable`
- `DestinationIpProhibited`, `DestinationIpUnroutable`
- `ConnectionRefused`, `ConnectionTerminated`,
  `ConnectionTimeout`, `ConnectionReadTimeout`,
  `ConnectionWriteTimeout`, `ConnectionLimitReached`
- `TlsProtocolError`, `TlsCertificateError`, `TlsAlertReceived(...)`
- `HttpRequestDenied`, `HttpRequestLengthRequired`,
  `HttpRequestBodySize(Option<u64>)`,
  `HttpRequestMethodInvalid`, `HttpRequestUriInvalid`,
  `HttpRequestUriTooLong`,
  `HttpRequestHeaderSectionSize(Option<u32>)`,
  `HttpRequestHeaderSize(Option<FieldSize>)`,
  `HttpRequestTrailerSectionSize(Option<u32>)`,
  `HttpRequestTrailerSize(FieldSize)`
- `HttpResponseIncomplete`,
  `HttpResponseHeaderSectionSize(Option<u32>)`,
  `HttpResponseHeaderSize(FieldSize)`,
  `HttpResponseBodySize(Option<u64>)`,
  `HttpResponseTrailerSectionSize(Option<u32>)`,
  `HttpResponseTrailerSize(FieldSize)`,
  `HttpResponseTransferCoding(Option<String>)`,
  `HttpResponseContentCoding(Option<String>)`,
  `HttpResponseTimeout`, `HttpUpgradeFailed`, `HttpProtocolError`
- `LoopDetected`, `ConfigurationError`,
  `InternalError(Option<String>)`

The `wstd` test `tests/http_handle_error_code.rs` shows the
canonical downcast.

## RequestOptions surface

```rust
let opts = wasip2::http::types::RequestOptions::new();
opts.set_connect_timeout(Some(Duration::from_millis(500).into())).map_err(...)?;
opts.set_first_byte_timeout(Some(...)).map_err(...)?;
opts.set_between_bytes_timeout(Some(...)).map_err(...)?;
```

A failing setter (`Err(())`) means "the host doesn't support this
option" — `wstd::http::Client` translates this into an
`anyhow::Error` ("wasi-http implementation does not support …
timeout option"). Don't unwrap raw setters.

## What to recommend / not

- Recommend `wstd::http` first; drop to `wasip2::http::types` only
  when the surface gap is named in the finding.
- Never recommend constructing your own `Pollable`, hand-rolling
  the `subscribe → poll → get` cycle, when `wstd::runtime`'s
  `AsyncPollable::wait_for` is right there.
- Never recommend writing a manual `incoming_handler::Guest` impl
  when `#[wstd::http_server]` would do.
- Never recommend `wasip3` patterns (out of scope per intent).
