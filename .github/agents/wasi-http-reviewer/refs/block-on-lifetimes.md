# `block_on` and resource lifetimes

Distilled from `wstd/src/runtime/{block_on.rs, reactor.rs, mod.rs}`,
the `wstd-macro` expansion, issue #107, PR #108, issue #113. Use this
when reviewing diffs that touch `block_on`, `Reactor::spawn`, or the
borrow / `'static` story around `wit-bindgen` resources.

## The `block_on` execution model

```text
fn block_on<F: Future>(fut: F) -> F::Output {
    let reactor = Reactor::new();
    if REACTOR.replace(Some(reactor.clone())).is_some() {
        panic!("cannot wstd::runtime::block_on inside an existing block_on!")
    }
    let root_task = unsafe { reactor.spawn_unchecked(fut) };
    loop { ... pop_ready_list / wasi::io::poll::poll ... }
    REACTOR.replace(None);
    // pull Ready out of root_task
}
```

Three load-bearing properties:

1. **There is at most one `block_on` running at a time, globally.**
   Issue #113: nested `block_on` panics. This is a fundamental
   property of the single-threaded reactor — to support nesting, we
   would need stack-switching (not available in wasm32-wasip2 yet).
2. **The top-level future does not need to be `'static`.** PR #108
   removed the constraint by using an internal `spawn_unchecked`
   that the surrounding loop machinery makes safe. The top-level
   future may borrow.
3. **`Reactor::spawn` (for additional tasks) still requires
   `'static`.** Spawned tasks may outlive the calling scope as far
   as the type system can see, so the bound is real.

## `wit-bindgen` `Borrow<'_>` resources

`wit-bindgen` generates two flavours of resource handle for an
exported resource type:

- `Resource<'a>` (owned)
- `Borrow<'a>` (borrowed, with a lifetime)

Borrowed forms appear in exported function signatures where the wit
declared `borrow<r>`. A function exported by the guest that takes
`borrow<test>` will show up in Rust as:

```rust
fn use_test(test: TestBorrow<'_>) -> String { ... }
```

`TestBorrow<'_>` is **not** `'static`. The lifetime is tied to the
duration of the host call.

## The collision (issue #107)

```rust
fn use_test(test: TestBorrow<'_>) -> String {
    block_on(async move { test.get::<Test>().hello() })
    //                    ^^^^ captured by the future
}
```

Pre-PR #108: `block_on` required `F: 'static`, the future captures a
non-`'static` borrow → compile error.

Post-PR #108: top-level `block_on` accepts the non-`'static` future
because `spawn_unchecked` says "the Task does not outlive the future
or its output" by construction inside the `block_on` loop. So:

- A `block_on(async move { my_borrow.foo() })` at the *outermost*
  level (the `use_test` body above) compiles.
- A `Reactor::current().spawn(async move { my_borrow.foo() })`
  inside a `block_on` does *not* compile — `spawn` still requires
  `'static`.
- A `tokio::spawn`-style "spawn this and forget" with a borrowed
  resource will not work in `wstd` — the task may outlive the
  borrow.

## Reviewer cues

- **Diff has `block_on` in a synchronous exported guest function and
  the body uses a `Borrow<'_>` resource:** with current `wstd`
  (>= 0.6.5, post-#108) this is fine; cite #108 in the finding so
  the author knows why the older version's compile error went away.
  If the diff pins `wstd <= 0.5.4`, it will not compile — recommend
  upgrading.
- **Diff calls `Reactor::current().spawn(async move { borrow.foo()
  })` or `wstd::runtime::spawn(async move { borrow.foo() })`:** that
  will not compile because `spawn` requires `'static`. Restructure
  to drive the borrowed work synchronously inside the `block_on`.
- **Diff has `block_on` *inside* an async function that's already
  driven by a `block_on`:** panic at runtime (issue #113). The
  fix is to make the inner function `async` and `.await` it.
- **Diff `static`-caches a value derived from a wit-bindgen
  resource (e.g. an `http::Client` whose backing handles are wasi
  resources):** crit. Wasm instance is fresh per request; the
  cached value's underlying handles do not survive.

## The single-threaded reactor consequence

Because the wasm32-wasip2 environment is single-threaded:

- `wstd` does not require `Send`/`Sync` bounds on futures or tasks.
- `Mutex`/`RwLock` work but are unnecessary; a `RefCell` or plain
  `&mut` is enough.
- `async-task` (used for `wstd::runtime::spawn`) is invoked with a
  schedule function that is not `Send` — the one `unsafe` block in
  `wstd` is the `async_task::spawn_unchecked` to allow a non-Send
  schedule (PR #86).

Reviewer cue: any diff that adds `Send + Sync + 'static` bounds to
internal types is almost always a port from a tokio example. Ask
why; in 90%+ of cases, drop the bounds.

## When to spawn vs structured concurrency

`wstd` deliberately surfaces `futures-concurrency` patterns ahead of
spawning:

- For "race two futures": `(a, b).race().await` or
  `futures_lite::future::race(a, b).await`.
- For "wait for both": `(a, b).join().await` /
  `futures_lite::future::try_zip(a, b).await` (used by
  `wstd::http::Client::send` to drive request body and response
  receipt concurrently).
- For dynamic sets: `futures_concurrency::future::FutureGroup`,
  `futures_concurrency::stream::Merge`.
- Spawn (`Reactor::current().spawn` / `wstd::runtime::spawn`) is a
  last resort, e.g. accepting on a TCP listener while servicing
  in-flight connections. The wstd `tcp_echo_server` example uses
  it for that exact case (PR #86).

Reviewer rule: a diff that introduces `spawn` for what could be
structured `race`/`join` is `arch-W` — it's a more complex shape and
ties up a `'static` constraint.

## Cooperative concurrency footgun (#73)

Pre-PR #78, a CPU-heavy task that yields with
`futures_lite::future::yield_now()` inside a `FutureGroup` could
prevent timer-driven futures from making progress because the
reactor only checked pollables when the ready list was empty. The
fix (PR #78) added a non-blocking pollable check when the ready
list refills.

Reviewer rule: this is fixed in current `wstd`. If a diff carries a
workaround for it (manual `yield_now`s in a hot loop, or a custom
busy-poll), recommend removing the workaround and pinning a
post-#78 `wstd`.
