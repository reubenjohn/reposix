---
phase: 89
slug: framework-fixes-cadence-shell-kind
type: execute
execution_mode: top-level
tdd_mode: false
mvp_mode: false
nyquist_compliant: true
catalog_first: true
requirements: [RBF-FW-01, RBF-FW-02, RBF-FW-03, RBF-FW-04, RBF-FW-05, RBF-FW-11]
depends_on: []
created: 2026-05-08
---

# Phase 89 — Plan Overview

> **EXECUTION MODE: TOP-LEVEL.** Run from a top-level Claude session. **NOT** invocable via `/gsd-execute-phase` — `gsd-executor` lacks the `Task` tool and depth-2 subagent spawning is forbidden (CLAUDE.md § "Subagent delegation rules" → "Orchestration-shaped phases run at top-level"). The orchestrator routes / decides / integrates; sub-subagents (one per task wave) do all read/write work and return ≤300-word TLDRs while writing full detail to disk.

## Phase Summary

P89 ships the **5 framework deliverables** that make every other v0.13.0-extension phase's catalog rows trustworthy: (1) **`cadence: pre-release-real-backend`** (env-gated; default-skips in CI; required at milestone-close — `RBF-FW-01`); (2) **`kind: shell-subprocess`** verifier convention with transcript artifact (`RBF-FW-02`); (3) **milestone-close 9th probe SLOT** (legitimately NOT-VERIFIED until P91+P92+P93+P94+P95 land the substrate — `RBF-FW-03`); (4) two new structure-dim linters — **banned-production-tokens** (`crates/**/*.rs` regex `\bP\d{2,3}-\d+\b` — `RBF-FW-04`) + **deferral-pointer linter** (cross-references `not yet wired in P\d+` strings against named-phase PLAN existence — `RBF-FW-05`); (5) **`claim_vs_assertion_audit`** required schema field (≥50 chars; runner cross-check at catalog-load time; date-cutoff ≥ 2026-05-08 to grandfather legacy P78–P88 rows — `RBF-FW-11`). The catalog-first commit (T1) mints all 6 NOT-VERIFIED rows BEFORE any implementation lands and eats its own dogfood (each row carries the `claim_vs_assertion_audit` field that this phase introduces).

## Task Breakdown

| Task | Name | REQ-IDs | Type | Effort | Depends on | Files (key) |
|---|---|---|---|---|---|---|
| **89-01** | Catalog-first commit: mint 6 NOT-VERIFIED rows | All 6 | mechanical | XS (1–2h) | — | `quality/catalogs/agent-ux.json`, `quality/catalogs/freshness-invariants.json` |
| **89-02** | RBF-FW-04 — banned-production-tokens linter (`crates/**/*.rs`) | RBF-FW-04 | mechanical | XS (~2h) | 89-01 | `quality/gates/structure/banned-production-tokens.sh` (NEW) |
| **89-03** | RBF-FW-01 — `pre-release-real-backend` cadence + `_realbackend.py` factor | RBF-FW-01 | unit + integration | M (4–5h) | 89-01 | `quality/runners/run.py:11,45-47,153-199,297-305`, `quality/runners/_realbackend.py` (NEW), `quality/runners/test_realbackend.py` (NEW), `quality/PROTOCOL.md:140-148` |
| **89-04** | RBF-FW-02 — `kind: shell-subprocess` + transcript convention | RBF-FW-02 | smoke | M (4–5h) | 89-03 | `quality/catalogs/README.md:27`, `quality/gates/agent-ux/lib/transcript.sh` (NEW), `quality/gates/agent-ux/shell-subprocess-example.sh` (NEW), `quality/runners/run.py:259-274`, `quality/PROTOCOL.md` § "Verifier subagent prompt template", `quality/reports/transcripts/.gitkeep` (NEW dir) |
| **89-05** | RBF-FW-05 — deferral-pointer linter | RBF-FW-05 | mechanical | S (~3h) | 89-01 | `quality/gates/structure/deferral-pointer-linter.sh` (NEW) |
| **89-06** | RBF-FW-03 — milestone-close 9th probe SLOT + verdict template | RBF-FW-03 | smoke | S (~3h) | 89-01, 89-03 | `quality/dispatch/milestone-close-verdict.md` (NEW), `quality/gates/agent-ux/milestone-close-vision-litmus.sh` (NEW), `quality/PROTOCOL.md` § "Per-phase protocol" Step 6 |
| **89-07** | RBF-FW-11 — `claim_vs_assertion_audit` field + runner cross-check | RBF-FW-11 | unit | S (~3h) | 89-04 | `quality/catalogs/README.md:22-41`, `quality/runners/_audit_field.py` (NEW), `quality/runners/run.py:72-81,148-150`, `quality/runners/test_audit_field.py` (NEW) |
| **89-08** | CLAUDE.md update + push + verifier subagent dispatch | All | mechanical + manual | S (2–3h) | 89-01 .. 89-07 | `CLAUDE.md` § "Quality Gates" cadence/kind tables + § "Subagent delegation rules", `quality/reports/verdicts/p89/VERDICT.md` |

**Effort total:** ~18–25h (within the 5–6 day envelope locked in REMEDIATION-PLAN).

## Wave Decomposition

| Wave | Tasks | Rationale |
|---|---|---|
| **Wave 1** | 89-01 | Catalog-first contract — MUST be the phase's first commit per CLAUDE.md "Quality Gates" + RESEARCH § Recommended Task Decomposition. All subsequent commits cite a row id. |
| **Wave 2** | 89-02, 89-03, 89-05 (parallel) | All three touch disjoint file surfaces. 89-02 = `quality/gates/structure/banned-production-tokens.sh` (new). 89-05 = `quality/gates/structure/deferral-pointer-linter.sh` (new). 89-03 = `run.py` + new `_realbackend.py`. No file overlap. CLAUDE.md "Build memory budget" is N/A (no cargo work in P89). Sub-subagents may run in parallel. |
| **Wave 3** | 89-04 → 89-06 → 89-07 (sequential — share `run.py`) | 89-04 extends `run_row()` artifact synthesis (`run.py:259-274`). 89-06 needs the `pre-release-real-backend` cadence from 89-03 to validate the new catalog row. 89-07 hooks into `load_catalog()` (`run.py:72-81`) and must happen AFTER 89-04's `run.py` edit so the diff base is clean. Sequencing avoids cross-task merge conflicts on the runner. |
| **Wave 4** | 89-08 | Wrap-up: CLAUDE.md table extensions (8 cadences / 6 kinds / new structure-dim linter entries) + `git push origin main` + verifier subagent dispatch + verdict at `quality/reports/verdicts/p89/VERDICT.md`. |

> **Wave 2 ordering note:** RESEARCH § "Recommended Task Decomposition" suggested T2/T3 in Wave 2 then T4 (cadence) sequentially. After re-reading the file-surface dependencies, **89-03 (cadence) belongs in Wave 2 alongside 89-02 + 89-05** because it touches `run.py` + a NEW sibling module (`_realbackend.py`); 89-02 and 89-05 do not touch `run.py` at all. The Wave-3 sequential block is 89-04 → 89-06 → 89-07. This is one ordering refinement vs. the RESEARCH suggestion, made for clarity.

## Threat Model

| Threat ID | Severity | Mitigation (REQ-ID) | Verifier |
|---|---|---|---|
| **T-89-01** — Catalog rows ship with bypassed validation; phase declares GREEN against rows the runner never loaded | high | 89-01 mints rows BEFORE implementation; 89-07 runner cross-check refuses load if `claim_vs_assertion_audit` missing on rows minted ≥ 2026-05-08 (RBF-FW-11) | `python3 quality/runners/run.py --cadence pre-push --dimension agent-ux,structure --dry-run` (89-01); `python3 -m unittest quality.runners.test_audit_field` (89-07) |
| **T-89-02** — Transport tests claim coverage but only invoke wiremock; verifier subagent grades GREEN against simulated bytes | high | 89-04 ships `kind: shell-subprocess` convention with transcript artifact (real argv + env_keys + exit_code); future P92 verifiers MUST adopt the kind for transport claims; verifier subagent dereferences `transcript_path` (RBF-FW-02) | `bash quality/gates/agent-ux/shell-subprocess-example.sh && test -f quality/reports/transcripts/*.txt` (89-04) |
| **T-89-03** — Milestone-close graded GREEN without real-backend probe; v0.13.0 ships a tag against simulator-only evidence | high | 89-06 ships SLOT verifier + verdict TEMPLATE with 9th probe row; the row's `blast_radius: P0` makes any milestone-close grading attempt with the row NOT-VERIFIED return exit 1 (RBF-FW-03) | `test -x quality/gates/agent-ux/milestone-close-vision-litmus.sh && test -f quality/dispatch/milestone-close-verdict.md` (89-06) |
| **T-89-04** — Phase IDs (`P79-02 scaffold`-style strings) leak from production source into user-facing stderr; agents see internal phase IDs they cannot interpret | med | 89-02 banned-production-tokens linter blocks at pre-commit/pre-push; allowlist marker `// banned-words: ok` for justified exceptions (RBF-FW-04); P91 RBF-A-03 will scrub the existing 2 production hits | `bash quality/gates/structure/banned-production-tokens.sh` (89-02) |
| **T-89-05** — Deferral pointers in `crates/` (`not yet wired in P\d+`) rot when downstream PLAN files vanish or phase numbers get renumbered | med | 89-05 deferral-pointer linter cross-references named phase against `.planning/phases/N-*/PLAN*.md` existence at pre-push (RBF-FW-05) | `bash quality/gates/structure/deferral-pointer-linter.sh` (89-05) |
| **T-89-06** — Catalog rows ship without claim-vs-assertion accountability; row description claims something the verifier asserts cannot falsify | high | 89-07 runner cross-check at catalog-load time (`load_catalog()`); rows minted ≥ 2026-05-08 lacking `claim_vs_assertion_audit` ≥50 chars cause `SystemExit` BEFORE any verifier runs (RBF-FW-11); P90 RBF-FW-12 ships the adversarial dispatch that grades the audit text itself | `python3 -m unittest quality.runners.test_audit_field` (89-07) |

> **Trust boundary:** The catalog file → runner load path is the boundary. Bytes flow from JSON → Python dict → row execution. T-89-01 + T-89-06 mitigate at load-time (fail-loud BEFORE any verifier runs). T-89-04 + T-89-05 mitigate at write-time (linters block commits/pushes that would land bad strings). T-89-02 + T-89-03 mitigate the GREEN-without-real-evidence failure mode that motivated the entire v0.13.0-extension series.

## Catalog-First Reminder

The phase's **FIRST commit** (89-01) mints **6 catalog rows** — `status: NOT-VERIFIED` on all six — BEFORE any implementation commit lands:

- `quality/catalogs/agent-ux.json` (3 new rows): `agent-ux/cadence-pre-release-real-backend`, `agent-ux/kind-shell-subprocess-worked-example`, `agent-ux/milestone-close-vision-litmus-real-backend`
- `quality/catalogs/freshness-invariants.json` (3 new rows): `structure/banned-production-tokens`, `structure/deferral-pointer-linter`, `structure/claim-vs-assertion-audit-required`

**3 + 3 split** (NOT a new `framework.json`) per RESEARCH § Q-CATALOG-DIM-1 — there is no `framework` dimension in CLAUDE.md's "9 dimensions" table; existing `agent-ux` + `structure` (catalog file `freshness-invariants.json` per its `"dimension": "structure"` wrapper) absorb the rows cleanly. The ROADMAP P89 SC #5 wording `quality/catalogs/{agent-ux,framework}.json` is conventional shorthand; literal compliance would require a schema migration that bloats the phase. The override is documented in 89-01-PLAN.md.

Each row carries a **`claim_vs_assertion_audit` paragraph (≥50 chars)** in the catalog-first commit — eating its own dogfood, since RBF-FW-11 is the row that introduces the field. CD-01 from CONTEXT delegates the exact phrasing to the planner; templates are inlined in 89-01-PLAN.md.

## Push Cadence Reminder

Per CLAUDE.md "Push cadence — per-phase":

> Every phase closes with `git push origin main` BEFORE the verifier-subagent dispatch. Pre-push gate-passing is part of phase close, not an end-of-session sweep — verifier grades RED if the phase shipped without the push landing.

Task **89-08** owns the close ritual: (1) run `python3 quality/runners/run.py --cadence pre-push` locally and confirm exit 0; (2) `git push origin main`; (3) dispatch the verifier subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt template"; (4) await GREEN verdict at `quality/reports/verdicts/p89/VERDICT.md`. Trivial in-phase chores (typo, comment cleanup) ride to origin with this terminal push, not their own round-trip.

## No-Cargo Note (CLAUDE.md "Build memory budget")

P89 does **NOT** touch `crates/` except for the banned-tokens linter (89-02), which only **READS** `*.rs` files via `grep -nHE`. No `cargo check` / `cargo test` / `cargo build` invocations are required by any P89 task. The CLAUDE.md "Build memory budget" section is N/A for this phase. This means Wave 2's three parallel sub-subagents face zero RAM contention; the only constraint is wall-clock orchestration latency.

## Two-Channel Rule (for sub-subagents during execution)

Each sub-subagent dispatched by the top-level orchestrator MUST:
1. Write **full detail to disk** — code/docs created, full test output, file:line citations, decision rationale.
2. Return a **≤300-word TLDR** to the orchestrator naming: files touched, verifier command + exit code, deviations from RESEARCH (if any), and any items appended to `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` per OP-8.

This rule mirrors CLAUDE.md OP-2 ("aggressive subagent delegation") and OP-3 ("audit log is non-optional"). Each PLAN.md step block calls it out explicitly.

## Goal-Backward Verification (ROADMAP § Phase 89 Success Criteria)

| ROADMAP SC (verbatim, lines 144-149) | Delivered by | Verifier command |
|---|---|---|
| **SC1.** `quality/PROTOCOL.md` documents new cadence + kind with worked example; `quality/runners/run.py` recognizes `pre-release-real-backend` (default-skip when env not set) | 89-03 (cadence) + 89-04 (kind) + 89-03's PROTOCOL.md edit | `python3 -m unittest quality.runners.test_realbackend` + `grep -F 'pre-release-real-backend' quality/PROTOCOL.md` + `grep -F 'shell-subprocess' quality/PROTOCOL.md` |
| **SC2.** Milestone-close verdict template carries 9th probe entry; absent ⇒ verdict graded RED | 89-06 (template + SLOT verifier) | `test -f quality/dispatch/milestone-close-verdict.md && grep -cE '^\| ?[1-9]' quality/dispatch/milestone-close-verdict.md \| awk '$1 >= 9 {exit 0} {exit 1}'` |
| **SC3.** Pre-push gate runs deferral-pointer linter; banned-production-error-tokens regex `P\d+-\d+` extended | 89-02 (banned-tokens) + 89-05 (deferral-pointer) | `bash quality/gates/structure/banned-production-tokens.sh && bash quality/gates/structure/deferral-pointer-linter.sh` |
| **SC4.** `claim_vs_assertion_audit` field present on every new catalog row P89/P90 mints; runner cross-check passes | 89-01 (mint with field) + 89-07 (runner cross-check) | `python3 -m unittest quality.runners.test_audit_field && python3 quality/runners/run.py --cadence pre-push --dry-run` |
| **SC5.** Catalog-first commit mints 5+ rows in `quality/catalogs/{agent-ux,framework}.json` with `status: NOT-VERIFIED` BEFORE implementation commits land; CLAUDE.md updated in same PR | 89-01 (catalog-first; 3+3 split per RESEARCH override) + 89-08 (CLAUDE.md update) | `git log --oneline --first-parent -- quality/catalogs/agent-ux.json quality/catalogs/freshness-invariants.json | head -1 \| grep -F 'catalog'` (verifies catalog-first) + `grep -F '8 cadences' CLAUDE.md` + `grep -F 'shell-subprocess' CLAUDE.md` |
| **SC6.** Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p89/VERDICT.md` | 89-08 | `git status --short` (clean) + `git log origin/main..HEAD` (empty) + `test -f quality/reports/verdicts/p89/VERDICT.md` + `grep -E '(GREEN|PASS)' quality/reports/verdicts/p89/VERDICT.md` |

> **SC5 partial-override note:** RESEARCH § Q-CATALOG-DIM-1 establishes that `framework.json` is conventional shorthand in the ROADMAP — there is no `framework` dimension. The 3+3 split across `agent-ux.json` + `freshness-invariants.json` honors the locked SC's intent (≥5 rows minted NOT-VERIFIED in catalog-first commit) without forcing a schema migration. Document the override in 89-01-PLAN.md and 89-08 (CLAUDE.md update).

## Auto-Resolution Preference (CLAUDE.md OP-8)

Surprises < 1 hour incremental work AND introducing no new dependency get fixed in the discovering task. Bigger surprises append to `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` with severity + what + why-out-of-scope + sketched resolution. Each PLAN.md step block reminds the sub-subagent of this rule.

## Plan-Check Materials

`gsd-plan-checker` should validate against:
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § P89 + § "Cross-cutting framework fixes"
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` § Phase 89 (locked SCs lines 144-149)
- `89-CONTEXT.md` § `<decisions>` (D-01a..D-CLM-04)
- `89-RESEARCH.md` § "Recommended Task Decomposition" + § "Risks and Watchouts"
- `89-VALIDATION.md` § "Per-Task Verification Map"
