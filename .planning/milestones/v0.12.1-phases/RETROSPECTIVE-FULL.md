# v0.12.1 — Carry-forwards + docs-alignment cleanup (full retrospective)

Distilled lessons live in `.planning/RETROSPECTIVE.md`. This file holds the
full narrative (What Was Built, Cost Observations, verbose Carry-forward
prose) that was trimmed from RETROSPECTIVE.md per the file-size budget.

**Shipped:** 2026-04-30 (autonomous run 2026-04-29; owner-TTY close-out 2026-04-30)
**Phases:** 6 autonomous-run (P72–P77) + owner-TTY follow-ups | **Carry-forward:** P67–P71 deferred to a follow-up session

## What Was Built
- **P72** Lint-config invariants — 8 shell verifiers under `quality/gates/code/lint-invariants/` binding 9 `MISSING_TEST` rows for README + `docs/development/contributing.md` workspace-level invariants
- **P73** Connector contract gaps — 4 `MISSING_TEST` rows closed with byte-exact wiremock auth-header assertions for GitHub (Bearer) + Confluence (Basic), and a JIRA `list_records` rendering-boundary test asserting `Record.body` excludes attachments/comments per ADR-005:79–87
- **P74** Narrative + UX cluster — 4 propose-retires for qualitative/design rows + 5 hash-shape binds for UX claims on `docs/index.md` + REQUIREMENTS rows + `docs/social/linkedin.md` prose fix dropping FUSE-era framing; promoted bind sweep to `scripts/p74-bind-ux-rows.sh` (OP #4: ad-hoc bash is a missing-tool signal)
- **P75** `bind` verb hash-overwrite fix — preserved `source_hash` on `Source::Multi` rebinds; refresh only on `Source::Single`; 3 walker regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`
- **P76** Surprises absorption — drained `SURPRISES-INTAKE.md` (3 LOW entries discovered during P72 + P74); each entry RESOLVED/WONTFIX with commit SHA or rationale; +2 phase practice now operational
- **P77** Good-to-haves polish — drained `GOOD-TO-HAVES.md` (1 XS entry from P74); heading rename `What each backend can do → Connector capability matrix` + verifier regex narrowed back to literal `[Cc]onnector`
- **Owner-TTY close-out (2026-04-30)** — SSH config drift fixed (`~/.ssh/config` IdentityFile rename); 27 RETIRE_PROPOSED rows confirmed via `--i-am-human` bypass; jira.md "Phase 28 read-only" prose dropped (Phase 29 had shipped write path); cargo fmt drift from P73/P75 cleaned; pre-commit fmt hook installed; v0.12.0 tag pushed; 5 backlog items filed (999.2–999.6); milestone-close verdict ratified by unbiased subagent

## Cost Observations
- Model: claude-opus-4-7[1m] (1M context, owner-TTY close-out session)
- Autonomous-run model: claude-sonnet-4-6 (P72-P77)
- Sessions: 1 autonomous (2026-04-29) + 1 owner-TTY close-out (2026-04-30)
- Notable: the close-out session caught fmt drift, retire-backlog, doc-prose drift, and SSH config drift in one ~3-hour pass; would have surfaced incrementally as v0.13.0 mid-session surprises if not addressed

## Carry-forward to v0.13.0 (verbose)
- **P67–P71 deferred** — original v0.12.1 carry-forward bundle (separate from the autonomous-run cluster). Re-evaluate scope at v0.13.0 kickoff
- **Backlog 999.2** — `confirm-retire --all-proposed` batch flag (OP #4 missing-tool)
- **Backlog 999.3** — pre-push runner timeout-vs-asserts_failed conflation
- **Backlog 999.4** — autonomous-run push-cadence decision (CLAUDE.md scope)
- **Backlog 999.5** — `docs/reference/crates.md` zero claim-coverage
- **Backlog 999.6** — docs-alignment coverage_ratio climb from 0.20
- **3 WAIVED structure rows** expire 2026-05-15 — `no-loose-top-level-planning-audits`, `no-pre-pivot-doc-stubs`, `repo-org-audit-artifact-present`. Verifier scripts must land before the TTL or the waiver renews defeat the catalog-first rule
- **RETROSPECTIVE.md backfill** for v0.9.0 → v0.12.0 — distill from each milestone's `*-phases/` artifacts into the OP-9 template (multi-hour synthesis; 5+ milestones)
