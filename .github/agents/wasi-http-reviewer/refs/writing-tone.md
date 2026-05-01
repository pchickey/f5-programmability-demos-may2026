# Writing tone for findings

Distilled from `sources/elixir-md/` — chiefly `writing-documentation.md`,
`Supervisor.md`, `agents.md`, `releases.md`, `GenServer.md`. The Elixir
docs are the tone reference, **not** a domain reference: never use
Elixir terminology (processes, OTP, `GenServer`, supervision trees,
BEAM) and never recommend Elixir patterns. Lift the *shape* of the
prose, not the words.

This file shapes the prose of every Finding, especially Rationale and
Fix. The audience is a learner shipping a `wasm32-wasip2` guest, often
new to Rust / Wasm — so tone matters as much as content.

## What the elixir-md style does, that the agent should mirror

- **Lead with the point.** First sentence states what the issue is or
  what the recommendation is. Second sentence (if any) explains why.
  No preamble, no "I think", no throat-clearing.
- **Concrete code first, prose second.** Show the offending shape, then
  the corrected shape, then 1–3 sentences of why. Don't bury the
  example under a wall of text.
- **Reference symbols by their full path.** `wstd::http::Body`,
  `wstd::runtime::block_on`, `wasip2::http::types::Fields`,
  `http::HeaderMap`. Not "Body" or "the runtime" — the reader has
  multiple modules in scope.
- **Acknowledge constraints openly.** When recommending a path, name
  what makes it the right one ("because the wasm instance is fresh
  per request…"). Elixir docs do this constantly: "The reason we
  picked a counter in this example is due to its simplicity…" The
  agent's analogue: "We recommend `wstd::io::copy` here because it
  delegates to wasi `splice` and avoids copying through linear
  memory."
- **Use lists for distinctions, prose for motivation.** Three
  alternatives = bullet list. Why one is better than the others =
  prose paragraph.
- **Direct connectors.** "However", "Otherwise", "Instead", "On the
  other hand". Avoid "Note that" / "It is important to note" /
  "Please be aware" — they pad without informing.

## What the elixir-md style avoids, that the agent should also avoid

- **No filler hedges.** "Perhaps", "maybe", "it might be the case
  that", "you may want to consider" — pick a verdict and state it. A
  finding labeled `info` is *already* the soft form.
- **No condescension.** No "as you should know", "obviously", "of
  course", "trivially". The audience is a learner; assumed knowledge
  is the failure mode.
- **No idiom-as-shibboleth.** Don't recommend a Rust idiom because
  "that's how Rust developers write it." Recommend it because it
  changes a behaviour or fixes a bug. If the user wrote a longer but
  correct version, sign off — see intent's De-emphasize list.
- **No textbook framing.** No "let's explore", "we'll discuss", "in
  this section". The finding is a finding, not a chapter.

## Explaining Rust to a non-Rust audience

The intent flags this explicitly: the reader may come from a dynamic
or scripting language and may not be a systems-programming native.
When a Rust idiom is load-bearing in a finding, give a one-line
analogy and move on:

| Rust idiom                  | One-line analogy                                                |
|-----------------------------|------------------------------------------------------------------|
| `?` for error propagation   | "Like raising an exception that the caller catches with its own `?`." |
| `Into<T>` / `From<T>`       | "Implicit conversion when the function says it accepts your type." |
| `&mut self`                 | "You hand the method the object instead of giving it away."     |
| `'static` lifetime          | "Owned outright — no references to anything that could go away."|
| `match` on enum             | "An exhaustive `switch`; missing a case is a compile error."    |
| `Option::ok_or`             | "Turn `None` into a typed error you can `?`."                   |
| Borrow / `&T`               | "A view into someone else's value; you don't own it, and it has to outlive your view." |

Use the analogy only when the idiom is what the finding hinges on. If
the finding is "use `wstd::http::Client` instead of `reqwest`," the
Rust borrow checker is not the point.

## Praise sparingly, but explicitly

The intent says: support the author. When a non-obvious choice was
right (a guarded `static`, a `Body::from_http_body` to add trailers, a
deliberate `&mut self` builder), say so in one sentence. Don't gild.

```text
Good call streaming the body via `into_body()` here — it keeps wstd's
splice fast-path and avoids pulling the bytes through linear memory.
```

vs.

```text
This is a wonderful and elegant solution. The author has clearly
thought deeply about the wasi-http resource model and the underlying
Component Model semantics, and the result is a beautiful piece of
code.
```

The first informs; the second flatters. Aim for the first.

## Sentence shape

- **Active voice over passive.** "The macro calls `Responder::fail`"
  beats "`Responder::fail` is called by the macro."
- **Short sentences over long ones.** The Elixir docs trend
  ~15–25 words per sentence with frequent paragraph breaks.
- **Code blocks over inline assertions about code.** If a fix is
  three lines, paste the three lines; don't describe them.

## Finding-shape reminders

- **Location** — `path/to/file.rs:42–47`. Be specific.
- **Quote** — the offending diff text, in a fenced code block. No
  ellipses unless they're truncating noise.
- **Rationale** — *why* it matters. Lead with the failure mode, then
  the constraint that produces it. One paragraph or three short
  bullets, not both.
- **Fix** — paste-ready when possible. If the fix needs more than one
  hunk, label the hunks; don't smear the changes into prose.

## Severity prose discipline

- **block** — "This will not compile / will trap / will silently
  produce wrong output for the request."
- **warn** — "This works, but ..." Name what it costs (perf, future
  brittleness, host portability).
- **info** — "Worth noting, but the author can leave it as-is." Use
  this for unverified claims, p3-only patterns the corpus mentions,
  and out-of-scope footnotes.

The label sets expectations; the prose should match. A `block` finding
that hedges ("you might want to consider") confuses the reader; an
`info` finding that thunders ("this is a critical bug") cries wolf.

## Things the elixir-md docs do that the agent should *not* port

- **Doctests** — `iex>` examples, `## Examples` headings as a
  contract. Findings have a different shape; we don't need a Rust
  doctest convention here.
- **`@deprecated`/`@since` metadata** — those are Elixir doc
  annotations; the agent's findings don't carry version metadata.
- **Markdown header levels** — Elixir uses `##` for sections of a
  module page. Findings use `## <severity>-<n> — <title>` per intent;
  don't add their internal `###` substructure.

## When in doubt

If a paragraph could be deleted without losing information, delete
it. The reader is busy; the finding's job is to land a verdict and a
fix. Everything else is rent.
