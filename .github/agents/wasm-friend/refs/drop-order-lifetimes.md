# Component-boundary drop order and lifetimes

The wasi component model imposes constraints that don't surface as
ordinary Rust borrow-checker errors. The borrow checker only sees the
compile-time lifetimes Rust gives it; the component model adds
runtime constraints (parent resource must outlive children) that show
up as `wasmtime` traps if violated. This ref is the operational
summary; the underlying tickets are `wstd` issue #31, #69, #107, and
PRs #35, #78, #108.

## Rule 1: child WASI resources drop before their parents

Concretely, this means:

- A `wasip2::io::streams::InputStream` taken out of a
  `wasip2::http::types::IncomingBody` (via `body.stream()`) must be
  dropped before the `IncomingBody`.
- A `wasip2::http::types::FutureTrailers` returned by
  `IncomingBody::finish(body)` is independent of the body, so it can
  outlive the body's stream — but its own subscribed `Pollable` must
  drop before it.
- A `wasip2::http::types::FutureIncomingResponse` (the value
  `outgoing_handler::handle` returns) is the parent of the
  subscription pollable used to await it.

`wstd` enforces these orderings internally:

- `AsyncInputStream` / `AsyncOutputStream` use `OnceLock<AsyncPollable>`
  with `subscription` declared **before** `stream` in the struct, so
  `stream` outlives the pollable on drop (`src/io/streams.rs`, "Field
  ordering matters" comment).
- `IncomingBody::poll_frame` explicitly transitions through
  `IncomingBodyState::{Body, Trailers}` and drops the body's stream
  state before constructing the `TrailersState`.
- `Body::send`'s `Incoming` branch drops `in_stream` and `out_stream`
  before calling `WasiIncomingBody::finish`.

User code should not reach into `wasip2::http::types::*` to take apart
these resources. As long as the guest sticks to `wstd::http::Body` and
its consumers (`contents`, `into_boxed_body`, `into_body`,
`from_http_body`), drop order is correct by construction.

If a user *must* drop down to `wasip2`, the discipline is:

```rust
// ok: stream dropped before body
let stream = body.stream()?;
do_work(&stream).await?;
drop(stream);
let trailers = wasip2::http::types::IncomingBody::finish(body);

// wrong: body dropped while stream still owned
let stream = body.stream()?;
drop(body);              // <-- traps later when stream is used
```

## Rule 2: futures crossing `block_on` must outlive their use

`wstd::runtime::block_on` polls futures on the call stack of
`block_on`. A future whose value depends on a borrow that was scoped
to the *caller* of `block_on` must not move into the future.

The classic case (issue #107) is `wit-bindgen`'s generated borrow
type:

```rust
// The borrow has a non-static lifetime tied to the export call.
fn use_test(test: TestBorrow<'_>) -> String {
    block_on(async move {
        // Was rejected by the type checker on wstd ≤ 0.5.4 because
        // block_on required `F: Future<Output: 'static>`. Compiles
        // on wstd ≥ 0.6.0 because PR #108 lifted that bound.
        test.get::<Test>().hello()
    })
}
```

What this means today:

- `wstd ≥ 0.6.0`: borrows are fine to move into `block_on`. Field
  ordering / drop order rules still apply.
- `wstd < 0.6.0`: don't move `Borrow<'_>` (or any non-`'static` value)
  into `block_on`. Either pin the wstd version to `0.6.0+` in
  `Cargo.toml`, do that work synchronously, or restructure so the
  resource has a longer-lived owner.

If the agent doesn't know which version the user is on, ask. Do not
silently assume `'static` is required.

## Rule 3: only one `block_on` at a time

```rust
// Panics: "cannot wstd::runtime::block_on inside an existing block_on!"
fn outer() {
    block_on(async {
        // ... some sync code that calls a function that calls block_on
        inner();
    });
}
```

Implications:

- `#[wstd::main]`, `#[wstd::http_server]`, and `#[wstd::test]` each
  start a `block_on`. None of them can be nested inside another one
  (issue #113).
- A guest cannot drive an async export from inside the synchronous
  trait method generated for a non-async wit signature unless that
  driver is the *only* `block_on` in scope. If the export method's
  outer macro already opened a `block_on`, there is no way to open
  another one without the panic.
- Tests that need to bracket a sync export call with async setup and
  teardown: write the test as a regular `#[test]`, call `block_on`
  twice — once for setup, once for teardown — with the sync call
  between them.

## Rule 4: futures dropped without their pollables waking

Dropping a future before completion is fine; that is how
`FutureExt::timeout` works. But the future being dropped must have
correctly-ordered fields so its pollable subscription drops before
its parent resource. `wstd` types do this; user code that builds its
own `Future` impl over `wasip2` types must follow the same discipline
(issue #31 and the PR #35 rewrite are the historical record).

If the agent finds itself authoring an `impl Future` directly over
`wasip2` types — stop. Use `AsyncPollable::wait_for` or
`Reactor::current().schedule(pollable).wait_for()` and let `wstd` own
the `pin_project_lite` projection.

## Rule 5: the `#[wstd::http_server]` macro owns the export

The macro generates `impl wasip2::exports::http::incoming_handler::Guest`,
calls `try_from_incoming(request)`, runs `block_on`, and routes the
result through `Responder::respond` or `Responder::fail`. User code
must not export `incoming_handler::Guest` again — the symbol is
already taken — and must not call `Responder::*` directly (those are
`#[doc(hidden)]` and the macro uses them).

## Quick checklist before emitting code

- [ ] If the diff calls `wasip2::http::types::*` directly, the
      function it lives in maintains the parent-after-children drop
      order on every return path.
- [ ] If the diff uses `block_on`, it is the outermost `block_on` in
      the call stack.
- [ ] If the diff moves a non-`'static` value into a future, the
      project is on `wstd ≥ 0.6.0`. (When unsure, ask.)
- [ ] If the diff implements `Future` over a wasi resource by hand,
      it uses `pin_project_lite` and orders fields with the
      subscription pollable declared before its parent resource.
- [ ] If the diff exports `incoming_handler::Guest` (e.g. without
      `#[wstd::http_server]`), it does so exactly once and orders
      `ResponseOutparam::set` calls correctly.
