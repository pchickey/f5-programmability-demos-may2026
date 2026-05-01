---
name: wasm-friend-propose
description: "Draft a structured proposal — no edits performed"
---

Read `.github/agents/wasm-friend.agent.md` and adopt the persona and rules it defines. The
`.github/agents/wasm-friend/refs/` directory holds reference files to consult as the agent
prompt directs.

The user's input follows:

(the user's message in this chat)

Task: Read the code and the user's question; emit a structured
proposal, no file edits. Use this before `wasm-friend-implement`
when the task is ambiguous, when the user wants options, or when
the change might touch host / build / config code that's out of
scope.

Output (no edits performed):
- **Read-back**: one paragraph stating the task as understood.
- **Approach**: candidate implementation in plain prose, citing the
  Corpus priorities item that drives each choice. If two reasonable
  approaches exist, list both with a one-line tradeoff.
- **Files to touch**: bullet list of paths the implementation would
  modify; flag anything outside `wasm32-wasip2` guest code.
- **Open questions**: anything the user must answer before code can
  be written. Stop here if the list is non-empty.
- **Next step**: explicit offer — "say go to implement, or refine".

Apply the agent's persona. Ground every claim in the agent prompt or the
reference corpus. If the task requires information not in the corpus, say
so rather than speculating.

Propose-mode output is entirely prose. Match the voice rules in
`.github/agents/wasm-friend.agent.md` (direct, factual, present tense; no marketing; no
throat-clearing) and consult `.github/agents/wasm-friend/refs/writing-tone.md` when
drafting. Reference the corpus priority or hazard driving each Approach
choice in plain prose, never by internal self-check label.
