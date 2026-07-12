# v0.13.0 Surprises Intake (P96 source-of-truth)

> **CARRY-FORWARD BANNER — 2026-07-06 pre-v0.13.0-tag sweep.** v0.13.0 is **CLOSED-GREEN, tag imminent.** Every entry still marked **OPEN** below is a **live carry-forward** — it survives to the post-tag **v0.14.0 / v0.13.2 scoping session** for re-triage, NOT a v0.13.0 action item. A STATUS line that cites a now-closed **P9x** phase (P95, P97, …) means "deferred past that closed phase, now pending v0.14.0 re-triage" — the phase ref is historical, not a live target. Terminal (resolved / verified-clean) entries were DELETED this sweep — git is the archive (bound-to-live-state). Do NOT spin up a `v0.14.0-phases/` dir to hold these; they stay here until that scoping session ingests them.

> **Append-only intake for surprises discovered during P78-P96 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. **P96 (OP-8 Slot 1) drains this file** (was P87 in the original P78–P88 plan; renumbered when the milestone extended to P78–P97).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P97, OP-8 Slot 2).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P87 should resolve.

**STATUS:** OPEN  (← P96 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## Split index (OP-8 file-size drain)

This ledger exceeded the *.md 20k budget and was split into 7 per-part child files under `surprises-intake/`. Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here.

- [`surprises-intake/part-01.md`](surprises-intake/part-01.md) — 12 entries:
  - 2026-07-07 | discovered-by: v0.13.1 CHECKOUT-BREAK lane | severity: MEDIUM
  - 2026-07-03 11:10 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM
  - 2026-07-03 11:15 | discovered-by: resumption audit (8-week idle gap) | severity: HIGH
  - 2026-07-03 11:20 | discovered-by: resumption audit (8-week idle gap) | severity: LOW
  - 2026-07-03 11:25 | discovered-by: resumption audit (8-week idle gap) | severity: LOW
  - 2026-07-03 11:30 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM
  - 2026-07-03 11:45 | discovered-by: resumption audit (8-week idle gap) | severity: LOW
  - 2026-07-03 21:00 | discovered-by: P89 orchestrator (CI triage) | severity: LOW
  - 2026-07-03 21:00 | discovered-by: P89-01 | severity: LOW
  - 2026-07-03 21:35 | discovered-by: 89-07 | severity: HIGH
  - 2026-07-04 05:30 | discovered-by: steward-window (post-P89, PR #58 triage) | severity: LOW
  - 2026-07-04 18:10 | discovered-by: quality-convergence connector re-audit | severity: HIGH
- [`surprises-intake/part-02.md`](surprises-intake/part-02.md) — 8 entries:
  - 2026-07-04 | discovered-by: P90 90-03 (confirmed live by 90-05) | severity: MEDIUM
  - 2026-07-04 | discovered-by: P91 91-03 (D91-09 token-scrub) | severity: LOW
  - 2026-07-04 21:00 | discovered-by: P91 91-05 (vision-litmus real-run) | severity: LOW
  - 2026-07-04 22:15 | discovered-by: P91 litmus-REOPEN (repro setup + peer session) | severity: LOW
  - 2026-07-04 | discovered-by: P91 91-02 (deferred-items.md), reconciled during 91-06 docs wave | severity: MEDIUM
  - 2026-07-05 | TokenWorld two-writer conflict verifier does not exist — SC1 real-backend arm cannot be verified until built | discovered-by: P92 SC1 adjudication (D-P92-03) | severity: HIGH
  - 2026-07-05 debt-drain triage
  - 2026-07-05 debt-drain: branch hygiene + PR triage (staged for owner)
- [`surprises-intake/part-03.md`](surprises-intake/part-03.md) — 8 entries:
  - 2026-07-05 | `quality/gates/docs-build/mkdocs-strict.sh` under-reports broken internal anchors (swallowed at INFO log level) | discovered-by: P93 Wave 2a executor | severity: MEDIUM
  - 2026-07-05 | Pagination-truncation safety of sync's `prune_oid_map` — a truncated `list_records()` can DELETE oid_map rows for LIVE records beyond the cap | discovered-by: P93 DP-2 REOPEN re-review (relayed via coordinator, independently re-verified) | severity: HIGH
  - 2026-07-05 | ROADMAP.md § "Phase 94"–"Phase 97" prose is STALE/orphaned vs the LIVE STATE.md cursor | discovered-by: P94 catalog-first planning lane | severity: MEDIUM
  - 2026-07-05 | STATE.md frontmatter has no strict-YAML parseability guard — a bare `: ` regresses it silently | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW
  - 2026-07-05 | Committed catalog `status` lags the live grade between explicit `--persist` mints (by P96 design) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW
  - 2026-07-05 | `--persist` mint path should refuse to write a row it would reject at load (write-path load-refusal hardening) | discovered-by: P96 Wave 3a (residual split out of the D-P96-01 self-mutation fix) | severity: MEDIUM
  - 2026-07-05 | `--persist` mint re-flips `subjective/*` rows off a STALE rubric artifact on every mint (pre-release cadence collateral churn) | discovered-by: P96 phase-close (post-close drain / verdict NOTICED review) | severity: MEDIUM
  - S-260706-rbf-01 — ADR-010 Consequences item-4 note is itself stale (LOW / cosmetic)
- [`surprises-intake/part-04.md`](surprises-intake/part-04.md) — 2 entries:
  - S-260706-rbf-02 — CONSULT-DECISIONS T1 entry has an empty Commit field (LOW / cosmetic)
  - S-260707-rbf-01 — `crlf_blob_body_round_trips_byte_for_byte` intermittent red on PR #61's `quality-pre-pr` job (HIGH / unresolved, non-timeout assertion failure — upgraded from MEDIUM 2026-07-07)
- [`surprises-intake/part-05.md`](surprises-intake/part-05.md) — 9 entries:
  - (continued) 
  - S-260707-pr-01 — `reposix-sim` ships in NO prebuilt distribution; documented 3-crate binstall installs ZERO binaries (BLOCKER)
  - S-260707-pr-02 — `reposix init` hides a fatal fetch failure behind exit 0, then hands out a broken "Next:" hint (HIGH)
  - S-260707-pr-03 — `reposix sim` fallback shells out to `cargo run -p reposix-sim` from a release binary (HIGH)
  - S-260707-pr-04 — reposix-swarm integration tests share one on-disk cache + sqlite path across parallel threads/binaries (MEDIUM)
  - S-260707-pr-05 — `reposix init`/`doctor` write cache state to `~/.cache/reposix/` unconditionally, with no documented override (MEDIUM)
  - S-260707-pr-06 — `git-remote-reposix` has no `--version`; installer.sh has no arg validation; `REPOSIX_INSTALL_DIR` undocumented (LOW)
  - S-260707-pr-07 — `docs-build/p94-badges-real-vs-transient` catalog row reported stale-NOT-VERIFIED while the underlying gate needs live re-verification (LOW)
  - S-260707-pr-08 — agent "worktrees" are NOT isolated; a sim-seed leaf corrupted the shared repo (`t <t@t>` flipped `core.bare=true`) (HIGH)
- [`surprises-intake/part-06.md`](surprises-intake/part-06.md) — 9 entries:
  - S-260707-rbf-lr03-external-write-crash | 2026-07-07 | discovered-by: v0.13.1 B5 TRIAGE | severity: HIGH | tag: v0.14.0 RBF-LR-03 pivot
  - 2026-07-07 | discovered-by: v0.13.1 Lane E2 (agent-ux/zero-shot-onboarding) | severity: HIGH
  - 2026-07-07 | git-floor drift in planning artifacts — `.planning/PROJECT.md` and `docs_reproducible_catalog.json` still assert a HARD `git >= 2.34` floor | discovered-by: v0.13.1 mechanical filing lane | severity: LOW
  - 2026-07-07 | Cache keyed by project name, not target dir — repeat-use friction on 2nd tutorial attempt | discovered-by: v0.13.1 zero-shot re-gate | severity: MEDIUM
  - 2026-07-07 | S-260707-gh404 — GitHub real-backend helper path 404s on owner/repo: cache feeds filesystem-sanitized project into backend REST call | discovered-by: p94/protocol.rs CRLF-flake fix executor (CI run 28909417360) | severity: HIGH
  - 2026-07-07 | S-260707-desync — cache-desync on push produces confusing PATCH-404 instead of a diagnostic | discovered-by: p94/protocol.rs CRLF-flake fix executor | severity: MEDIUM
  - 2026-07-07 | S-260707-relplz-tmp-collision — `/tmp` isolated git worktree not immune to concurrent leaf seed-commit corruption | discovered-by: v0.13.1 release Step-2 version-force executor | severity: MEDIUM
  - 2026-07-07 | S-260707-relplz-tagcut-doc-gap — release-plz creates crates.io publish + per-package tags but NOT the plain vX.Y.Z aggregate tag release.yml needs — mandatory owner-gated manual tag-cut undocumented in executor-facing runbook | discovered-by: v0.13.1 release Step-3 publish executor | severity: MEDIUM
  - 2026-07-11 | S-260711-prerelease-swallow — quality-pre-release.yml runs `verdict.py --cadence pre-release || true`, swallowing a genuine RED | discovered-by: post-release verdict mint (cargo-binstall row) | severity: MEDIUM
- [`surprises-intake/part-07.md`](surprises-intake/part-07.md) — 3 entries:
  - 2026-07-11 | S-260711-waiverclear-promotion — cleared-waiver container rows never auto-promote; post-release verdict reds on stale committed status (SECOND occurrence) | discovered-by: post-release verdict mint (cargo-binstall row) | severity: MEDIUM
  - 2026-07-11 | S-260711-stale-binstall-artifact — local `cargo-binstall-resolves.json` records a FAIL "cargo-binstall not installed" beside a PASS row | discovered-by: post-release verdict mint (cargo-binstall row) | severity: LOW
  - 2026-07-11 | S-260711-docalign-walk-mutation — doc-alignment walker self-mutates its own catalog at walk-time | discovered-by: post-release verdict mint (cargo-binstall row) | severity: LOW
