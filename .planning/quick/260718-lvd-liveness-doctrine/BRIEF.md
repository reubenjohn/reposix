---
quick_id: 260718-lvd
slug: liveness-doctrine
type: execute
autonomous: true
files_modified: [.planning/ORCHESTRATION.md, .claude/skills/coordinator-dispatch/SKILL.md, CLAUDE.md]
---

# Brief — fold corrected liveness doctrine into orchestration doctrine

**Goal:** Fold the corrected LIVENESS DOCTRINE (L0 ruling, 2026-07-17) into
`.planning/ORCHESTRATION.md` and `coordinator-dispatch/SKILL.md` so the next coordinator
session reads the tightened rule, instead of it living only in a milestone handover file.
A GSD-quick doctrine-update was explicitly owed for this (commit `95bc7c5f`).

**Incident this fixes:** During the P122 close, the C1 phase-coordinator pushed, then
backgrounded its OWN `gh run watch` to wait for CI and ended its turn assuming it would
re-wake on completion. It did not — background-task re-invocation is reliable ONLY at
L0 (top-level); a subagent's self-owned background watcher goes dormant and stalls the
close. The C2 coordinator-of-coordinators had to be poked by L0 and take deterministic
control (dispatch `gsd-verifier` + `gsd-executor` directly) rather than wait on the C1's
self-resume (2026-07-18).

**Doctrine folded (tight paraphrase, not verbatim):** at the push→CI-in-flight boundary
a coordinator STOPS and RETURNS to its dispatching parent (pushed SHA + in-flight run
id); the parent relays up to L0, which holds the durable CI watch and SendMessages the
coordinator to resume the close on green. Direct child-agent completion notifications DO
reliably re-invoke a parent; bare background-bash watchers do not. Everything before the
push (plan → execute → code-review) runs straight through — only this one boundary is a
stop-and-return point.

**Files touched:**
- `.planning/ORCHESTRATION.md` §3 (new liveness paragraph, relief/liveness area) + §11
  (one-line pointer inside the five-tier recursion paragraph)
- `.claude/skills/coordinator-dispatch/SKILL.md` (new liveness subsection near §6
  Relief trigger)
- `CLAUDE.md` (optional in-charter eager-fix: one pointer bullet under "Orchestration
  doctrine")

**Constraints:** doctrine-only; no P123-scope files touched. Commit locally, no push
(rides the P123 phase-close push).
