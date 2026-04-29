# v0.12.0 — phases archive

**Shipped:** 2026-04-27 — Quality Gates framework + 8 dimensions homed.

**Release status:** NOT TAGGED. No `crates/` source changed across P56–P63 (framework + docs + scripts + CI only), so a binary release would be bit-for-bit identical to v0.11.3. Workspace `Cargo.toml` stays at `0.11.3`. Next binary release is v0.13.0, which will tag the cumulative worktree. See `CHANGELOG.md` `[v0.12.0]` for the user-facing version of this note.

**Phases:** 56, 57, 58, 59, 60, 61, 62, 63.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.12.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` § v0.12.0 — milestone summary.
- `.planning/milestones/v0.12.0-phases/{ROADMAP.md, REQUIREMENTS.md}` — milestone-level scoping.
- `quality/reports/verdicts/p{57,58,59,60,61,62,63}/VERDICT.md` + `.planning/verifications/p56/VERDICT.md` — per-phase verifier verdicts (all GREEN; P56 pre-dates the `quality/reports/` tree per QG-06 transitional note).
- `git log --oneline v0.11.x..v0.12.0` — implementation commits.

**Per-phase contributions (formerly in CLAUDE.md; archived 2026-04-27 as part of post-milestone cohesion pass):**

## v0.12.0 Quality Gates — phase log

The v0.12.0 milestone migrates ad-hoc `scripts/check-*.sh` and the
conflated `scripts/end-state.py` to a dimension-tagged Quality
Gates system at `quality/{gates,catalogs,reports,runners}/`. **The
framework itself ships in P57** — until then, the catalog format
lives in `.planning/docs_reproducible_catalog.json` (ACTIVE-V0; P57
migrates it to the unified schema). Read
`.planning/research/v0.12.0/{vision-and-mental-model,
naming-and-architecture, roadmap-and-rationale,
autonomous-execution-protocol}.md` before working on any v0.12.0
phase. Read `quality/SURPRISES.md` (append-only pivot journal — seeded
in P56) at the start of every phase to avoid repeating dead ends.
Future phases follow `quality/PROTOCOL.md` (lands P57 — autonomous-mode
runtime contract).

## Per-phase files

- [p56.md](p56.md) — P56 contribution — release pipeline + install-evidence pattern
- [p58.md](p58.md) — P58 — Release dimension live
- [p59.md](p59.md) — P59 — Docs-repro + agent-ux + perf-relocate dimensions live
- [p60.md](p60.md) — P60 — Docs-build dimension live + composite cutover
- [p61.md](p61.md) — P61 — Subjective gates skill + freshness-TTL
- [p62.md](p62.md) — P62 — Repo-org-gaps cleanup + audit closure
- [p63.md](p63.md) — P63 — MIGRATE-02 cohesion pass + SIMPLIFY-12 + POLISH-CODE final + v0.12.1 carry-forward
- [p64.md](p64.md) — P64 — Docs-alignment dimension framework + skill + runner integration
- [p65.md](p65.md) — P65 — Docs-alignment backfill
