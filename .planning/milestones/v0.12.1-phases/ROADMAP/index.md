# v0.12.1 Roadmap (PLANNING)

_Carry-forward milestone closing v0.12.0 stubs + v0.11.x debts + the docs-alignment punch list surfaced by P65. Detail lands via `/gsd-plan-phase` per phase._

> **Phase numbering update (2026-04-28).** v0.12.0 grew P64 (docs-alignment infrastructure) and P65 (docs-alignment backfill) before tagging. v0.12.1 grew P66 (coverage_ratio metric — the second axis alongside `alignment_ratio`) at the front of the milestone. Existing v0.12.1 phases originally scoped at P64–P68 were renumbered to P66–P70 in the v0.12.0/P64+P65 insertion; the next renumber moved them to P67–P71 to make room for the new P66 (coverage).

> **2026-04-29 prep update.** P72–P77 added as the autonomous-run cluster: P72 (lint-config), P73 (connector contract gaps), P74 (narrative + UX cleanup), P75 (bind-verb hash fix), **P76 (surprises absorption — +2 reservation slot 1)**, **P77 (good-to-haves polish — +2 reservation slot 2)**. The +2 reservation is a project-wide operating principle (see CLAUDE.md OP-8): every milestone's last two phases absorb surprises and good-to-haves discovered during planned-phase execution, so plans no longer have to be perfect at design time and signal doesn't get dropped. P67–P71 carry-forwards are deferred to a follow-up session.

## Scope

v0.12.1 closes:

1. **v0.12.0 carry-forwards** — perf-dimension full implementation, security-dimension stubs → real verifiers, cross-platform rehearsals, MSRV / binstall / latest-pointer / release-PAT carry-forwards, the v0.11.1 `Error::Other` migration completion, subjective-dimension runner invariants from P61 Wave G.
2. **Docs-alignment gap closure** — the v0.9.0 pivot silently dropped behaviors prior milestones promised; P65's backfill surfaces them as `MISSING_TEST` rows clustered by user-facing surface (Confluence backend parity, JIRA shape, ease-of-setup happy path, outbound HTTP allowlist behavior, and others discovered by the backfill). Each cluster gets its own phase. Each phase's success criterion is "every catalog row in the cluster transitions BOUND or RETIRE_CONFIRMED." The catalog becomes the milestone-completion contract.

## Depends on

- v0.12.0 GREEN verdict at `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` (re-graded after P64 + P65 ship).
- `quality/catalogs/doc-alignment.json` populated with extracted claims (P65 deliverable).
- `quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md` reviewed by the human; cluster scopes confirmed.
- `quality/PROTOCOL.md` v0.12.0 runtime contract intact (the two project-wide principles from P64 inform every gap-closure phase).

## Chapters

- [phases.md](./phases.md) — All phase descriptions: P66 (coverage_ratio), P67–P71 (carry-forwards), P72–P77 (autonomous-run cluster + +2 reservation), and the deferred P67–P71 milestone-completion contract.
