# Findings: `git push` through a stateless-connect + export hybrid helper

**Date:** 2026-04-24
**Question source:** `.planning/research/push-path-stateless-connect.md`
**Verdict:** **The hybrid is viable and working.** A single helper can advertise `stateless-connect` (for fetch / partial clone / lazy blobs) AND `export` (for push via fast-import stream) at the same time. Git dispatches each operation to the correct capability automatically, with no refspec or protocol trickery. Custom reject messages surface verbatim to the user. We do NOT need to solve "push through receive-pack over stateless-connect."

**Evidence:**
- Source review of `transport-helper.c` (latest master) — dispatch logic proves the hybrid works by construction.
- Working POC at `.planning/research/git-remote-poc.py` (extended from the read-path POC).
- Runner script: `.planning/research/run-poc-push.sh`.
- Full captured trace: `.planning/research/poc-push-trace.log`.

---

- [Dispatch and reject](./dispatch-and-reject.md) — Q1, Q2
- [Intercept and hybrid viability](./intercept-and-hybrid.md) — Q3, Q4
- [Conflict detection, POC bugs, recommendation, and open questions](./conflict-poc-sources.md) — Q5 onwards
