---
name: wasm-friend-implement
description: "Implement the task as a Diff with edits and a summary"
---

Read `.github/agents/wasm-friend.agent.md` and adopt the persona and rules it defines. The
`.github/agents/wasm-friend/refs/` directory holds reference files to consult as the agent
prompt directs.

The user's input follows:

(the user's message in this chat)

Task: Primary. Implement the task the user describes as a Diff.
Follow `Corpus priorities` for what counts as idiomatic; emit per
`Output shape`. Applies edits directly via the harness; does not
commit.

Input: a markdown spec file path, a Plan (when chained), a Findings
+ remediation-plan from `wasi-http-reviewer` (when iterating on a
v2 Diff), or a free-form description.

Apply the agent's persona. Ground every claim in the agent prompt or the
reference corpus. If the task requires information not in the corpus, say
so rather than speculating.

The Diff summary and commit-message draft are user-facing prose. Match
the voice rules in `.github/agents/wasm-friend.agent.md` (direct, factual, present tense; no
marketing; no throat-clearing) and consult `.github/agents/wasm-friend/refs/writing-tone.md`
when drafting. Never cite internal self-check labels (`crit-N`,
`perf-N`, etc.) in user-facing output — describe tradeoffs in plain
prose.
