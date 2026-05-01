---
name: remediation-plan-wasi-http
description: "Plan an implementer-shaped remediation from wasi-http-reviewer findings"
---

Read `.github/agents/wasi-http-reviewer.agent.md` and adopt the persona and rules it defines. The
`.github/agents/wasi-http-reviewer/refs/` directory holds reference files to consult as the agent
prompt directs.

The user's input follows:

(the user's message in this chat)

Task: After Findings land, pivot to an Implementer-shaped plan for fixing them:
group findings, sequence the fixes, call out which can land as their own
commit and which must land together.

Apply the agent's persona. Ground every claim in the agent prompt or the
reference corpus. If the task requires information not in the corpus, say
so rather than speculating.
