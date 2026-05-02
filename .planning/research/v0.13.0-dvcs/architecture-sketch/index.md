# v0.13.0 — DVCS Architecture Sketch

> **Audience.** The agent planning v0.13.0 phases. Read `vision-and-mental-model.md` (sibling in this folder) first for the thesis and success gates; this doc is the technical design + the open questions a planner needs to resolve before writing PLAN.md. After both, read `kickoff-recommendations.md` (sibling) for the pre-kickoff checklist + four readiness moves identified by the v0.12.1 close-out.
>
> **Status.** Pre-roadmap research. Owner-approved direction; specific algorithms below are starting points, not commitments.

## Chapters

- [The three innovations](./innovations.md) — `reposix attach`, mirror-lag refs, bus remote algorithm, and the `list_records` L1/L2/L3 performance subtlety.
- [Webhook-driven mirror sync](./webhook-sync.md) — GH Action workflow, confluence webhook setup, `--force-with-lease` race protection.
- [Phase decomposition, exclusions, and tie-back](./phase-decomposition.md) — sketched N–N+8 phase sequence, what is NOT in scope, and helper code tie-back.
