# `wasi:http/types/fields` semantics

Distilled from wasi-http archived issues #117, #82, #91, #28, #131
and PRs #70, #71, #91, #121. Authoritative behaviour the reviewer
should hold guest code to.

## Equality and casing

- **Field-name comparisons are case-insensitive.** `fields.get("Foo")`
  matches `Foo`, `foo`, and `FOO`. (PR #121.)
- **Original case is preserved** in storage, in `entries()`, and on
  the wire. The fields resource remembers the casing of the first
  setter (or what arrived from the wire).
- **`wstd` inherits this** by delegating to `http::HeaderMap`, which
  is the Rust ecosystem's case-insensitive map.
- **Reviewer rule:** any guest code that uses raw byte/string
  equality on header names is a bug (`crit-8`).

## Presence vs empty

- `fields.get("absent")` returns `[]` (empty list).
- `fields.get("present-but-empty")` returns `[""]` (a single empty
  element). Empty header values are legal — `Accept-Encoding: ` for
  example has a special meaning per RFC 9110 §12.5.3 (#82 archived).
- `fields.has(name) -> bool` is the unambiguous presence check
  (PR #91 archived).
- **Reviewer rule:** if guest code "checks header presence" by
  testing emptiness of the value list, it conflates absent and
  present-but-empty. Recommend `has()`.

## Multiple values

- `fields.get(name)` returns *all* values for that name in the
  original casing and order.
- The transport may serialise them as a comma-separated list (HTTP/1)
  or as separate fields (HTTP/2+); guests don't choose, the host
  does.
- An "empty value among a multi-value list" is representable:
  `["foo", "", "bar"]`.

## Field-name range

- Field names are intended to be RFC 9110 `field-name` — visible
  US-ASCII (VCHAR) plus SP and HTAB. (Open issue wasi #786 to
  formalise this in the spec; PR not yet landed.)
- **Reviewer rule:** if guest code constructs a header name with
  bytes that aren't ASCII, that's a bug for v0.2. `http::HeaderName`
  rejects invalid names at construction time, so user code that
  goes through `HeaderName::from_str` is safe.

## Field-value range

- Field values are **`list<u8>`** in the wit (#28 archived). The
  HTTP spec allows non-UTF-8 octets (`obs-text`). A guest must not
  assume UTF-8.
- `http::HeaderValue` has `as_bytes()` for the raw view and
  `to_str()` (which fails on non-ASCII) for the convenience view.
  Use `as_bytes()` if the value might contain `obs-text`.

## Mutability

- Newly-constructed `Fields::new()` is mutable (`set`, `append`,
  `delete`). The `from-list` constructor (`fields.from-list(...)`)
  also yields a mutable fields.
- **Once a `Fields` has been attached to an outgoing-request /
  outgoing-response, it (and any other handle to it) becomes
  immutable.** Subsequent `set` / `append` / `delete` calls return
  `header-error::immutable` (PR #70 archived).
- The `headers` returned by `IncomingRequest::headers()` /
  `IncomingResponse::headers()` is immutable.
- The `headers` returned by `OutgoingResponse::headers()` (after
  construction but before sending) is mutable; the `headers` snapshot
  inside the request/response itself becomes immutable on attach.
- **Reviewer rule:** a guest that holds onto a `Fields` reference
  and mutates it after attaching it to an outgoing message will get
  `header-error::immutable` and may unwrap on it. Reorder the
  mutations to happen before attach.

## Content-Length verification

- `outgoing-body.finish` is fallible. If the outgoing-request /
  outgoing-response was constructed with a `Content-Length` header
  and the actual bytes written disagree, `finish` returns an
  `ErrorCode` (PR #71 archived).
- **Reviewer rule:** if a guest sets `Content-Length` manually, it
  must write exactly that many bytes. `wstd::http::Body` /
  `Responder::respond` set `Content-Length` automatically when the
  body is in-memory and don't set it for streaming bodies — let
  wstd handle this in normal cases.

## Trailers

- Trailers are an `option<fields>`. **`some(empty)` and `none` are
  semantically equivalent** (issue #131 archived). Don't introduce
  a special case.
- Outgoing trailers are passed to `OutgoingBody::finish(body,
  Some(trailers_or_none))`.
- Incoming trailers are read via the `FutureTrailers` returned by
  `IncomingBody::finish(body)`. The trailers can only be read after
  the body stream has been fully consumed (the input-stream child
  must be dropped before `finish`).
- In `wstd`, trailers are part of the `http_body::Frame` model;
  `Body::from_http_body` is the only constructor that lets you
  attach trailers, and `body.into_boxed_body().collect().await?`
  is the way to read them (`http_body_util::Collected::trailers`).

## What "set with empty list" means

Open question (wasi #900): `fields.set(name, [])`. Consensus from
the maintainers (`badeend`, `pchickey`):

> "Don't trap. Declare `delete` to be a semantic alias for `set`
> with an empty list."

Treat as the expected behaviour. Don't recommend a guest write
`fields.set(name, [])` to "delete" — call `fields.delete(name)`
instead, which is unambiguous.

## Cross-instance reference (wasi-http archived #38)

Multiple guest-side handles to the same `Fields` resource alias the
same underlying value (until the immutability moment). A guest that
holds two handles and mutates through one will see the mutation
through the other. This is by design.

## Reviewer crib sheet

| Question                                | Answer                          |
|-----------------------------------------|---------------------------------|
| Header lookup case-sensitive?           | No.                             |
| Original casing preserved on the wire?  | Yes.                            |
| Empty value list = absent?              | No — use `has`.                 |
| Mutation after body attach?             | Returns `header-error::immutable`. |
| `Content-Length` mismatch on finish?    | `ErrorCode`. Don't unwrap.      |
| Trailer `none` vs empty `some`?         | Semantically equivalent.        |
| Field value UTF-8?                      | Not guaranteed. Bytes.          |
| Field name UTF-8?                       | ASCII (RFC 9110 `field-name`).  |
