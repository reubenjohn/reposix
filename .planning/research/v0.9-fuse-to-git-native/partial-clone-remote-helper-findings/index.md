# Findings: git remote helpers CAN serve as promisor remotes for partial clone

**Date:** 2026-04-24
**Question source:** `.planning/research/partial-clone-remote-helper.md`
**Verdict:** **YES** — a remote helper that advertises `stateless-connect` and proxies protocol-v2 traffic to `git upload-pack` can act as a fully functional promisor remote. We can delete `crates/reposix-fuse` if we accept the architectural shift.
**Evidence:** working POC at `.planning/research/git-remote-poc.py`, run script `.planning/research/run-poc.sh`, captured helper trace `.planning/research/poc-helper-trace.log`. The POC clones with `--filter=blob:none --no-checkout`, then proves lazy fetches happen through the helper by reading individual blobs and observing the helper being re-spawned with single-want `command=fetch` requests.

---

## Chapters

- **[Q1–Q5: Research questions](./questions.md)** — Five research questions with empirical evidence: whether helpers can be promisor remotes (Q1), what capabilities they need (Q2), alternatives if they couldn't (Q3), per-request inspection and refusal (Q4), and sparse-checkout interaction (Q5).

- **[Recommendation, Feasibility, Sources, Open questions](./recommendation.md)** — The recommended path B (`stateless-connect` tunnel), architecture diagram, what gets deleted/added, risks, POC feasibility confirmation, sources cited, and open questions for next phase planning.
