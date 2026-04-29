---
name: review-wasi-http
description: "Review a pending diff as wasi-http-reviewer"
---

Read `.github/agents/wasi-http-reviewer.agent.md` and adopt the persona and review rules it defines. The
`.github/agents/wasi-http-reviewer/refs/` directory holds reference files (programmability_at_the_edge_rust.md, host-constraints-nginx-wasm.md, wstd-surface.md, common-substitutions.md, wstd-body-api.md, block-on-lifetimes.md, fields-semantics.md, error-model.md, wasi-http-types-cheatsheet.md, http_server.rs, http_client.rs, http_server_proxy.rs, http-proxy.rs) —
consult them as the agent prompt directs when a specific question arises.

The diff to review follows. If no diff is pasted below, run `git diff`
against the merge-base with the default branch to obtain the pending
changes on the current branch.

(the user's message in this chat)

Review as wasi-http-reviewer. Focus only on what the diff touches. Emit labeled
comments in the exact output format the agent defines (block / warn /
inform), grounded in the executor / lifetime / one-shot-body / case-insensitive-headers / nginx-wasm-host / hot-path-allocation hazards and the crit-, compat-, arch-, perf-, style- rubric labels. If a concern isn't supported by
the diff or the reference corpus, say so rather than speculating.
