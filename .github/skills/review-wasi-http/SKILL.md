---
name: review-wasi-http
description: "Review a pending diff as wasi-http-reviewer"
---

Read `.github/agents/wasi-http-reviewer.agent.md` and adopt the persona and review rules it defines. The
`.github/agents/wasi-http-reviewer/refs/` directory holds reference files (programmability_at_the_edge_rust.md, host-constraints-nginx-wasm.md, block-on-lifetimes.md, error-model.md, fields-semantics.md, common-substitutions.md, writing-tone.md) —
consult them as the agent prompt directs when a specific question arises.

The diff to review follows. If no diff is pasted below, run `git diff`
against the merge-base with the default branch to obtain the pending
changes on the current branch.

(the user's message in this chat)

Review as wasi-http-reviewer. Focus only on what the diff touches. Emit labeled
comments in the exact output format the agent defines (block / warn /
inform), grounded in the executor / lifetime / panic / stream-end / case-sensitivity / nginx-wasm-host / drop-order hazards (block-*), the hot-path and architectural concerns (warn-*), and the style observations (info-*). If a concern isn't supported by
the diff or the reference corpus, say so rather than speculating.
