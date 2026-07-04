# Phase 89: Framework fixes — Context (consolidated background SSoT)

> **This file is the single source of truth for P89 background.** It digests the
> former `89-RESEARCH.md` (now deleted): research conclusions are encoded in the
> `89-*-PLAN.md` files; this file keeps only the load-bearing facts a developer
> needs at the keyboard plus the locked discuss-phase decisions. The plan spine
> lives in `89-PLAN-OVERVIEW.md`; owner overrides in `89-OWNER-DECISIONS.md`;
> the Nyquist contract in `89-VALIDATION.md`. Cross-reference, don't re-derive.

**Gathered:** 2026-05-08 · **Status:** PLANNED + CONVERGED (plan-check passed)
**Discussion mode:** autonomous (no gray areas escalated; six REQ-IDs locked by REMEDIATION-PLAN P89)

## Phase Boundary

P89 builds the **framework infrastructure** that makes every other v0.13.0-extension
phase's catalog rows trustworthy. Five deliverables ship:

1. New runner cadence `pre-release-real-backend` — env-gated (`REPOSIX_ALLOWED_ORIGINS` +
   sanctioned-target creds), default-skips in CI, **required at milestone-close** (no skip = RED).
2. New verifier kind `shell-subprocess` — drives `reposix init/attach/sync/push` as real
   subprocesses (NOT `assert_cmd`-style cargo-test envelopes) + produces a transcript artifact.
3. A 9th probe in the milestone-close verdict template running the vision litmus test
   verbatim against TokenWorld; absent ⇒ verdict graded RED.
4. Two pre-push lint extensions — banned-production-error-tokens regex over `crates/`,
   and a deferral-pointer linter cross-referencing `not yet wired in P\d+` strings against
   the named phase's PLAN files.
5. New required catalog-row schema field `claim_vs_assertion_audit` — forces every row to
   state how the verifier's assertion would falsify the description claim if false.

**Does NOT ship:** F-K4 honesty rules (coverage_kind, asserts cross-check, dispatch.sh) → P90
RBF-FW-06..09; dishonest-test triage (F-K8) + milestone-close adversarial dispatch (RBF-FW-12)
→ P90; consuming code/doc fixes (real-backend `attach`, audit log) → P91+.

**Entry point** of the v0.13.0 extension series — no upstream dependencies.
**Execution mode: top-level** (fans out across PROTOCOL.md, runners, catalog schema,
verifier scripts, verdict template — no `gsd-executor` code-write-test-commit envelope).

## Locked Requirements (via REMEDIATION-PLAN P89 + ROADMAP § Phase 89)

No `89-SPEC.md` exists. Locking docs (downstream agents MUST read before implementing):
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md`
  § "P89" + § "2 — Cross-cutting framework fixes" (F-K1/F-K2/F-K3/F-K6/F-K7); P89 verbatim at `:120-167`
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md`
  § "S1 — chicken-and-egg" + § "Decision 3" (`:30-44`) — RBF-FW-11 justification + RBF-FW-12 boundary
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 89" (`:135-167`) — success criteria + top-level mode
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/PATTERNS.md` — C7
  "self-licensing-deferral-loop" anti-pattern that D-03c explicitly defends against
- `.planning/research/v0.13.0-real-backend-frictions/_archive/DECISIONS-NEEDED.md:25-46` — Decision 3 + Decision 4 verbatim

| REQ-ID | One-liner | F-K | Effort |
|---|---|---|---|
| **RBF-FW-01** | `cadence: pre-release-real-backend` (env-gated; CI default-skip; milestone-close required) | F-K1 | M |
| **RBF-FW-02** | `kind: shell-subprocess` (real subprocess + transcript artifact; not `assert_cmd`) | F-K2 | M |
| **RBF-FW-03** | Milestone-close 9th probe — vision litmus against real backend | F-K3 | S |
| **RBF-FW-04** | Banned-production-error-tokens regex over `crates/**/*.rs` (production source only) | F-K7 | XS |
| **RBF-FW-05** | Deferral-pointer linter — pre-push grep + cross-ref named-phase PLAN files | F-K6 | S |
| **RBF-FW-11** | `claim_vs_assertion_audit` field required on rows minted P89/P90+; runner cross-check | Dec.3 | S |

**Total effort:** 5–6 days (~18–22h). Locked success criteria + scope split → see `89-PLAN-OVERVIEW.md`.

**Out of scope (deferred):** F-K4/F-K8/RBF-FW-10/RBF-FW-12 → P90; backfill of
`claim_vs_assertion_audit` on P78–P88 rows → P95 RBF-D-06 (runner cross-check is date-gated
so the 388 legacy rows keep validating); real-backend `attach`/`sync` → P91; audit-log fixes → P92;
tag-script (`tag-v0.13.0.sh.disabled`) re-enable + 9th-probe guard → P97 RBF-G-04.

## Implementation Decisions (autonomous-mode defaults; planner overrides noted)

> OD-1/OD-2 owner overrides live in `89-OWNER-DECISIONS.md` — that file wins on conflict.

### RBF-FW-01 — `pre-release-real-backend` cadence
- **D-01a:** Add to `VALID_CADENCES` flat tuple at `quality/runners/run.py:45-47` (one tuple-line
  edit) + bump module docstring "The 7 cadences are…" at `run.py:11`. **Confirmed:** flat
  `tuple` + `argparse choices=` at `run.py:310` (`--cadence` required). No enum class, no
  string registry — keep the flat enum (CD-02).
- **D-01b:** Env-gate at `run_row()` BEFORE the WAIVED case (`run.py:163`). Requires:
  `REPOSIX_ALLOWED_ORIGINS` set + non-127.0.0.1 (`/^https?:\/\/(?!127\.0\.0\.1)/`) AND ≥1
  complete cred set — Confluence (`ATLASSIAN_API_KEY`+`ATLASSIAN_EMAIL`+`REPOSIX_CONFLUENCE_TENANT`),
  GitHub (`GITHUB_TOKEN`), or JIRA (`JIRA_EMAIL`+`JIRA_API_TOKEN`+`REPOSIX_JIRA_INSTANCE`).
  Not set → `status: NOT-VERIFIED` + `_skipped_real_backend: true` transient flag, mirroring
  the `_stale=True` STALE-case short-circuit at `run.py:181-199` (esp. line 197). Extend the
  `_stale` label branch in `print_row_summary()` at `run.py:297-305` to also recognize the new flag.
- **D-01c:** No `compute_exit_code()` special case — skipped rows land NOT-VERIFIED which already
  flips P0/P1 exit to 1. Milestone-close ritual invokes `--cadence pre-release-real-backend`
  and requires GREEN. Document in `quality/PROTOCOL.md` latency-budgets table (`:140-148`).
- **D-01d:** CI default-skip is automatic (GH workers lack creds). P89 adds NO CI workflow for
  this cadence (would defeat env-gating); document "no CI invocation; milestone-close ritual only."
- **D-01e:** Worked example in PROTOCOL.md uses a row tagged this cadence + `kind: shell-subprocess`.
- **Module factoring (Q-RUN-1 / anti-bloat):** factor env-gate logic into NEW
  `quality/runners/_realbackend.py` (~30-50 lines) per the `_freshness.py:1-50` precedent
  (`run.py:34`). `run.py` is **375-376 lines vs the ≤350 cap** — this factoring is **in-scope
  for P89**, not deferred.

### RBF-FW-02 — `kind: shell-subprocess` verifier
- **D-02a:** Add `shell-subprocess` to the `kind` enum at `quality/catalogs/README.md:27`
  (`mechanical | container | asset-exists | subagent-graded | manual` → +`shell-subprocess`).
  **Doc/convention extension, not a runner rewrite:** the runner dispatches on script suffix
  (`.sh` vs `.py`) at `run.py:226-232`, NOT on the `kind` field.
- **D-02b:** Verifier at `quality/gates/<dim>/<row-slug>.sh`; MUST invoke `reposix init/attach/sync`
  (or `git push reposix main`) as a real subprocess — NOT a `cargo test --test agent_flow_real` envelope.
- **D-02c — transcript artifact (load-bearing):** verifier writes BOTH the JSON status artifact
  at `quality/reports/verifications/<dim>/<row-slug>.json` AND a NEW transcript at
  `quality/reports/transcripts/<row-slug>-<RFC3339>.txt` with: `argv`, `env_keys` (sorted var
  NAMES only — never values; security per threat-model "Outbound HTTP allowlist"), `cwd`,
  `exit_code`, `--- STDOUT ---`, `--- STDERR ---`. JSON artifact gets a new `transcript_path` field.
  **Genuinely new convention:** existing shell verifiers do NOT write JSON artifacts or transcripts
  — they print to stdout/stderr and the runner synthesizes the JSON from
  `result.{stdout,stderr,returncode}` at `run.py:259-274`. Evidence:
  `mirror-refs-write-on-success.sh:23-31`, `reposix-attach.sh:38-53`,
  `no-loose-top-level-planning-audits.sh:14-26`. BUT the runner WILL respect a pre-existing
  artifact JSON if the verifier wrote one (`run.py:260-264`: keeps body, annotates top-level
  metadata only) — so `shell-subprocess` verifiers must self-write both files. **Factor the
  transcript-write into a shared helper** `quality/gates/agent-ux/lib/transcript.sh` mirroring
  the `quality/gates/agent-ux/dark-factory/lib.sh` precedent.
- **D-02d:** Worked example `quality/gates/agent-ux/shell-subprocess-example.sh` runs
  `reposix --version` (minimum proof-of-kind; exercisable in CI on sim path, no real-backend env).
- **D-02e:** Runner-side: extend `run_row()` artifact synthesis at `run.py:259-274` to read
  `transcript_path` from the verifier-written JSON and copy it into the top-level artifact. No
  new dispatch table — document this so future readers don't hunt for a missing one.

### RBF-FW-03 — Milestone-close 9th probe
- **D-03a:** Both land: (1) a new entry in the verdict TEMPLATE; (2) a catalog row
  `agent-ux/milestone-close-vision-litmus-real-backend` tagged
  `cadences: ["pre-release-real-backend"], kind: shell-subprocess, blast_radius: P0`.
- **D-03b — vision litmus definition:** the dark-factory third arm against TokenWorld.
  Definitional source: `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md`; executable
  shape: `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh` (P86 invariant, dispatched via
  `dark-factory.sh:40-42`). Mirror that shape but fire against TokenWorld. Asserts 4 invariants:
  (a) vanilla-clone + `reposix attach` + edit + `git push` end-to-end, zero in-context learning;
  (b) audit rows in BOTH `audit_events_cache` AND `audit_events` (OP-3 dual-table);
  (c) `refs/mirrors/<sot>-{head,synced-at}` advanced; (d) transcript written per D-02c.
  Probe target = TokenWorld/Confluence (Q-PROBE-OWNER-1 default; planner may override with rationale
  if dark-factory third-arm against `reubenjohn/reposix` issues is operationally simpler).
- **D-03c — SLOT, not waiver:** P89 ships the SLOT. The verifier
  `quality/gates/agent-ux/milestone-close-vision-litmus.sh` legitimately returns NOT-VERIFIED
  at P89 close (substrate lands P91–P95). SC#2 is about the TEMPLATE/ROW being present, not
  green. **Explicitly NOT a C7 self-licensing-deferral-loop** because: (a) mints NOT-VERIFIED
  not WAIVED; (b) `blast_radius: P0` ⇒ any milestone-close grade while NOT-VERIFIED returns
  exit 1; (c) the `verifier.script` MUST exist + be executable (no missing-file short-circuit).
  Verifier-subagent at P89-close must check the row's `claim_vs_assertion_audit` paragraph names
  "WILL flip to PASS only when P91+P92+P93+P94+P95 ship" — making the deferral self-falsifying.
- **D-03d:** `tag-v0.13.0.sh.disabled` is disabled (Path A/Option B). P89 does NOT modify it
  but documents in PROTOCOL.md that P97's tag script MUST run
  `python3 quality/runners/run.py --cadence pre-release-real-backend` requiring exit 0.

### RBF-FW-04 — Banned-production-error-tokens
- **D-04a:** **NEW script `quality/gates/structure/banned-production-tokens.sh`, NOT an extension
  of `scripts/banned-words-lint.sh`.** Confirmed (Q-BANNED-1): `quality/gates/structure/banned-words.sh`
  is a 7-line wrapper delegating to `scripts/banned-words-lint.sh --all`; the canonical impl at
  `scripts/banned-words-lint.sh:32-75` hardcodes `DOCS_ROOT="${REPO_ROOT}/docs"` + three layered
  glob arrays (LAYER1=`index.md`, LAYER2=`concepts tutorials guides`, LAYER3=`how-it-works`).
  Its allowlist marker is `<!-- banned-words: ok -->` (`:29`, markdown-comment shape — won't work
  in Rust). **Extending to `crates/` conflates two scopes and breaks the layered model.** Mint a
  Rust-native sibling with allowlist marker `// banned-words: ok`. Mirror the `scan_files()` ERE
  alternation + allowlist-filter pattern at `scripts/banned-words-lint.sh:107-141` (the
  `grep -nHE` pattern is at `:138`).
- **D-04c — REGEX OVERRIDE (load-bearing):** CONTEXT's original `\bP[0-9]+-[0-9]+\b` would BLOCK
  ~10 legitimate code-quality audit IDs on first run. **Use the tightened
  `\bP\d{2,3}-\d+\b`** (≥2 digit phase number; matches `P79-02`/`P83-01`, misses `P1-1`/`P0-2`/`P1-5`).
  Confirmed false-positive sources: `crates/reposix-core/src/error.rs:54,81` (`P1-1`),
  `crates/reposix-core/src/backend/sim.rs:610`. The planner overrides the CONTEXT D-04c default here.
- **D-04b — scope:** `crates/**/*.rs` EXCLUDING `tests/` + `**/tests/**` (phase IDs in test
  fn names are legit), `**/CHANGELOG.md` (historical record by definition), `*/target/*`, and
  lines containing `// banned-words: ok`. **Include comments in scan** (a stale
  `// TODO P79-02 wire this up` is exactly the target). `#[cfg(test)] mod tests` in-file blocks:
  defer to comment-allowlist for P89 (tree-sitter block detection is a P95 polish).
  All crates uniformly incl. `*-sim/` + `reposix-swarm/` (CD-04).
- **D-04d:** Catalog row `structure/banned-production-tokens`,
  `cadences: ["pre-commit","pre-push","pre-pr"], blast_radius: P1`. Wires through
  `.githooks/pre-push` → `run.py --cadence pre-push` (runner discovery; **no hook-script edit**).
- **D-04e — worked example:** `crates/reposix-cli/src/attach.rs:163-164`
  (`"attach: backend ... not yet wired in P79-02 scaffold (sim only)"`) MUST trigger BLOCK
  once unfixed; the FIX lands P91 (RBF-A-03), the LINT lands here. Second violation site:
  `crates/reposix-cli/src/sync.rs:42`. Post-hoc refactor markers
  (`crates/reposix-remote/src/main.rs:439` `P83-01 T02`; `bus_handler.rs:25,112,222`;
  `db.rs:97`; `fixtures/cache_schema.sql`) are NOT active deferrals — handled by the regex
  tightening above; allowlist markers only if any survive `P\d{2,3}-\d+`.

### RBF-FW-05 — Deferral-pointer linter
- **D-05a/b:** NEW `quality/gates/structure/deferral-pointer-linter.sh`. **Three regex patterns
  (F-K6 verbatim):** `not yet wired in P\d+`, `lands? (alongside|in) P\d+`, `substrate-gap-deferred`.
- **D-05c — algorithm:** `grep -rnHE` over `crates/` → parse `P\d+` → resolve
  `find .planning/phases -maxdepth 1 -type d -name "${N}-*"` → if no `*PLAN*.md` in that dir,
  BLOCK with structured error `crates/X/src/Y.rs:42 references P${N} but no
  .planning/phases/${N}-*/*PLAN*.md exists`. **Existence-only for P89** (Q-DEFLINK-1); content
  cross-reference is P90/P95 polish (adds ~10 lines if upgraded).
- **D-05d:** Row `structure/deferral-pointer-linter`, `cadences: ["pre-push"], blast_radius: P1`,
  in `quality/catalogs/freshness-invariants.json` (the structure-dim home).
- **Confirmed live matches (Q-DEFERRAL-1):** exactly two production-source strings today —
  `crates/reposix-cli/src/attach.rs:163` (`not yet wired in P79-02 scaffold`) and
  `crates/reposix-cli/src/sync.rs:42` (real-backend wiring lands in P82+). Both resolve to
  existing phase dirs ⇒ PASS today. False-positive risk LOW. Watchout: phase renumbering
  during milestone re-org could break references en masse — document
  "phase-number-renaming requires updating deferral-pointer refs in crates/" in PROTOCOL.md/CLAUDE.md.

### RBF-FW-11 — `claim_vs_assertion_audit` schema field
- **D-11a/b:** REQUIRED string (≥50 chars) on every row minted P89/P90+; explains how the
  verifier's `expected.asserts` would falsify the row's `description`/`comment` if the claim
  were false. Schema doc → `quality/catalogs/README.md:22-41` table, new row noting
  "yes (for rows minted ≥ 2026-05-08); pre-existing rows validate without it until P95 RBF-D-06."
- **D-11c — runner cross-check (Q-AUDIT-1 / Q-RUNNER-CROSS-CHECK):** the runner today validates
  NO row schema beyond wrapper structure (`run.py:72-81` checks `dimension`+`rows` keys + JSON
  parse only; no JSON-Schema validator — rows are duck-typed; the `$schema` field is decorative).
  Fires **at catalog-load time, NOT row-grade time** (matches "fail loud, structured" Principle B
  at `PROTOCOL.md:47`; avoids partial-success runs). Add NEW `quality/runners/_audit_field.py`
  (`_freshness.py` factoring precedent) with `validate_row(row, path, parse_rfc3339)`; hook into
  `load_catalog()` after the wrapper-key check (`run.py:81`). Cutoff
  `2026-05-08T00:00:00+00:00`; rows with `last_verified is None` OR `>= cutoff` are subject;
  pre-cutoff legacy rows PASS unconditionally (P95 RBF-D-06 backfills). Reuse existing
  `parse_rfc3339()` at `run.py:55-59`. Raise
  `SystemExit(f"FAIL: {path}: row {id} missing claim_vs_assertion_audit (≥50 chars required for rows minted on/after 2026-05-08)")`.
- **D-11d:** Ship a `claim_vs_assertion_audit_hash` (sha256) artifact field at grade time
  (`write_artifact` at `run.py:148-150`) so a verifier subagent can detect mint→grade drift.
- **D-11e:** P89 ships ONLY the structural FIELD + runner cross-check; P90 RBF-FW-12 ships the
  adversarial DISPATCH at `quality/dispatch/milestone-adversarial.md`. Document this boundary
  in CLAUDE.md so the planner doesn't fold the dispatch into P89.

### Catalog-first commit, CLAUDE.md, Claude's discretion
- **D-CAT-01/03:** Mint **6 NOT-VERIFIED rows** in the first commit BEFORE any implementation:
  3 → `quality/catalogs/agent-ux.json` (cadence, kind-worked-example, 9th-probe SLOT);
  3 → `quality/catalogs/freshness-invariants.json` (banned-tokens, deferral-pointer,
  claim-vs-assertion-required). **Do NOT create `framework.json`** (Q-CAT-1/Q-CATALOG-DIM-1):
  CLAUDE.md "9 dimensions" has no `framework` dimension; `freshness-invariants.json` IS the
  structure-dim catalog (wrapper `"dimension": "structure"`; there is no `structure.json`).
  The ROADMAP SC#5 `{agent-ux,framework}.json` mention is conventional shorthand — literal
  compliance is a schema migration that bloats scope; the dimension-respecting layout overrides it.
- **D-CAT-02:** All 6 rows carry `claim_vs_assertion_audit` paragraphs in the catalog-first
  commit (eating own dogfood). Paragraph wording is planner discretion (CD-01) — constraint is
  content (≥50 chars + falsification-shape), not phrasing.
- **D-CLM-01..04:** CLAUDE.md § "Quality Gates" — 7→**8 cadences** (+`pre-release-real-backend`),
  5→**6 kinds** (+`shell-subprocess`), no new dimension; one paragraph naming the two new
  structure-dim linters; § "Subagent delegation rules" gets a bullet: the 9th probe (RBF-FW-03)
  is non-skippable — any milestone-close without
  `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades RED. Prefer
  revising existing tables over appending sections. Also cross-reference the new cadence in
  § "Threat model" + § "Push cadence — per-phase" (Q-DOC-LINT-1).
- **Q-LOC-1 resolved:** the milestone-close verdict TEMPLATE does NOT exist today
  (`find` for `milestone-close*`/`milestone-verdict*` returns nothing; the closest artifact
  `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:1-67` is a free-form-prose VERDICT with
  two markdown tables — probe results `:12-23` (8 probes), per-phase verdicts `:27-39` — NOT a
  template, NOT structured). **P89 CREATES NEW `quality/dispatch/milestone-close-verdict.md`**
  (parallels the future `quality/dispatch/milestone-adversarial.md` P90 lands per
  `REMEDIATION-PLAN.md:191`): a 9-probe-row table skeleton. P89 does NOT touch the existing
  VERDICT.md; P97 RBF-G-04 uses the template to overwrite it (Decision 4 option (b),
  `_archive/DECISIONS-NEEDED.md:42-44`).

## Key code anchors (reusable patterns — mirror, don't rewrite)

- `quality/runners/run.py:45-47` `VALID_CADENCES` flat tuple (+ `:11` docstring, `:310` argparse choices).
- `quality/runners/run.py:181-199` STALE-case + `:163-176` WAIVED-case short-circuit — the
  exact shape RBF-FW-01's env-gate skip mirrors (`_stale=True` transient flag at `:197`).
- `quality/runners/run.py:148-150` `write_artifact()` helper (RBF-FW-02 transcript + RBF-FW-11 hash).
- `quality/runners/run.py:226-232/259-274` subprocess dispatch + artifact synthesis (suffix-based;
  respects verifier-written JSON at `:260-264`).
- `quality/runners/run.py:72-81` `load_catalog()` — fails loud `SystemExit` on wrapper violation
  (RBF-FW-11 cross-check hook). `parse_rfc3339()` at `:55-59`.
- `quality/runners/_freshness.py:1-50` — sibling-module factoring precedent (`run.py:34`).
- `scripts/banned-words-lint.sh:32-75` (DOCS_ROOT hardcoded), `:107-141` `scan_files()` ERE
  alternation, `:29` allowlist marker, `:138` `grep -nHE`.
- `quality/gates/structure/{no-pre-pivot-doc-stubs.sh,no-loose-top-level-planning-audits.sh:14-26,
  freshness-invariants.py}` — structure-dim verifier siblings for RBF-FW-04/05.
- `quality/gates/agent-ux/{reposix-attach.sh:38-53,mirror-refs-write-on-success.sh:23-31,
  dark-factory.sh:40-42,dark-factory/{lib.sh,dvcs-third-arm.sh}}` — agent-ux siblings + the
  third-arm shape RBF-FW-03 mirrors; `lib.sh` is the shared-helper factoring precedent.
- `.githooks/pre-push:1-71` — invokes `run.py --cadence pre-push`; RBF-FW-04/05 rows wire
  through it via runner discovery, **no hook-script edit**.
- `quality/catalogs/agent-ux.json` rows carry `_provenance_note` for hand-edits (e.g.
  `agent-ux/reposix-attach-against-vanilla-clone`) — new P89 rows MAY use this.

## Anti-bloat caps (PROTOCOL.md § "Anti-bloat rules per surface")

- **`run.py` ≤350 lines — currently 375-376, already over by ~26.** RBF-FW-01 + RBF-FW-11
  push it further; factoring env-gate → `_realbackend.py` and audit-check → `_audit_field.py`
  is **in-scope for P89** (not a deferral), per the `_freshness.py` precedent.
- `quality/PROTOCOL.md` ≤500 lines (currently ~307). The cadence/kind/9th-probe worked
  examples add bulk; if it crosses 500, factor § "Latency budgets" + § "Verifier subagent
  prompt template" into per-dimension `quality/gates/<dim>/README.md` files — **defer that
  factoring decision; don't pre-emptively split.**
- `CLAUDE.md` — no length cap but new sections cross-reference rather than duplicate; prefer
  revising the existing cadence/kind tables (D-CLM-04).

## Runtime constraints / gotchas confirmed during research

- **No exit-code → NOT-VERIFIED mapping (Q-EXIT-1).** `run.py:276-283` maps: `timed_out`→FAIL,
  `exit==0`→PASS, `exit==2`→PARTIAL, else→FAIL. There is NO exit-75 (or any) "verifier wants
  NOT-VERIFIED" mapping. So the RBF-FW-03 SLOT verifier must **write its own artifact JSON with
  `status: NOT-VERIFIED` directly** (the worked-example exit-75 sketch in older notes does NOT
  work). The combined SLOT shape: row tagged `cadences: ["pre-release-real-backend"]` → without
  env, RBF-FW-01's new `_realbackend.is_skipped` short-circuit returns NOT-VERIFIED before the
  script runs; with env, the script runs and (until P91+) writes its own NOT-VERIFIED artifact.
  Either way milestone-close grading produces "not green" — the desired SLOT semantics.
- **Runner stays stdlib-only** (`run.py:1-30` header) — `_realbackend.py` + `_audit_field.py`
  must use Python 3 stdlib only.
- **Verifier "fail loud, structured, agent-resolvable"** (Principle B) — every new gate emits
  stderr with a recovery hint: RBF-FW-04 → `// banned-words: ok` allowlist; RBF-FW-05 → the
  missing PLAN file path.
- **Test framework:** Python 3 stdlib `unittest` (runner-side) + bash (verifiers). Existing
  tests `quality/runners/test_freshness{,_synth}.py`. New: `test_realbackend.py`,
  `test_audit_field.py`. Run: `python3 -m unittest discover -s quality/runners -p "test_*.py"`.

## Risks / watchouts

- **RBF-FW-04 false positives — HIGH if regex unfixed.** Mitigated by the `P\d{2,3}-\d+`
  tightening + `**/CHANGELOG.md` + `**/tests/**` exclusion (above). Residual: `attach.rs:163`
  / `sync.rs:42` strings remain — intended catch (P91 fixes). `bus_handler.rs` P83-01 refactor
  markers are archaeology — survive only if they match `P\d{2,3}-\d+`; allowlist if so.
- **RBF-FW-05 false positives — LOW.** Only 2 live matches, both resolve. Renumbering-during-
  reorg is the real watchout (document the constraint).
- **RBF-FW-01 cache-warming — MEDIUM.** First `--cadence pre-release-real-backend` against
  TokenWorld pays full `list_records` + blob-materialization cost; per CLAUDE.md "Build memory
  budget" run it in a no-cargo session window. Document "first-run heavy" in the PROTOCOL latency row.
- **RBF-FW-11 backward-compat — LOW** (date-cutoff design exempts the 388 legacy rows). Watchout:
  a P89 row hand-set to a pre-cutoff `last_verified` to dodge the check = process violation;
  the phase-close verifier subagent spot-checks this.
- **RBF-FW-03 C7 risk — MEDIUM** (mitigated: NOT-VERIFIED not WAIVED, `blast_radius: P0`,
  never a `waiver` block, self-falsifying `claim_vs_assertion_audit`).
- **RBF-FW-02 scope-creep — MEDIUM.** `kind` is doc-convention not runtime-enforced; adding
  `shell-subprocess` to the enum doesn't prevent a row declaring it while pointing at a `.py`.
  Real structural enforcement is a P90 concern (RBF-FW-08). P89's worked example proves the
  kind end-to-end; document the boundary.

## Deferred / not folded

- Cross-AI peer review of P89/P90 (M1 in COMPLETENESS-CHECK) — deferred (autonomous mode);
  planner may revisit if framework-change confidence is LOW.
- Dead `pre-pr` cadence cleanup (W2) → P95 polish or v0.14.0.
- No `.planning/todos/` cross-reference performed (autonomous mode).

---

**See also (these own the live contract — defer to them on conflict):**
- `89-PLAN-OVERVIEW.md` — the plan spine (task waves T1–T8, success-criteria mapping).
- `89-OWNER-DECISIONS.md` — OD-1/OD-2 owner overrides (authoritative over any default here).
- `89-VALIDATION.md` — the Nyquist sampling contract + req→test map.
- `89-PLAN-CHECK.md` — plan-check verdict (convergence record).

*Phase: 89-framework-fixes-cadence-shell-kind · Context gathered 2026-05-08 · autonomous mode*
*Consolidated 2026-05-16: CONTEXT now digests former 89-RESEARCH.md (deleted).*
