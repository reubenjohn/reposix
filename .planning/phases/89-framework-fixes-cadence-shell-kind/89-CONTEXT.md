# Phase 89: Framework fixes — real-backend cadence + shell-subprocess kind + milestone-close litmus probe + claim-vs-assertion congruence - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning
**Discussion mode:** autonomous (per owner directive — no gray areas escalated; all six REQ-IDs were already locked by REMEDIATION-PLAN P89, implementation defaults captured below)

<domain>
## Phase Boundary

P89 builds the **framework infrastructure** that makes every other v0.13.0-extension phase's catalog rows trustworthy. Five deliverables ship in this phase:

1. A new runner cadence — `pre-release-real-backend` — that gates on `REPOSIX_ALLOWED_ORIGINS` + sanctioned-target credentials and default-skips in CI, but is **required at milestone-close** (no skip = grade RED).
2. A new verifier kind — `shell-subprocess` — that drives `reposix init/attach/sync/push` as actual subprocesses against a real backend (NOT `assert_cmd`-style cargo-test envelopes) and produces a transcript artifact.
3. A 9th probe in the milestone-close verdict template that runs the vision document's litmus test verbatim against TokenWorld; absent ⇒ verdict graded RED.
4. Two pre-push lint extensions — banned-production-error-tokens regex (`P\d+-\d+` in `crates/`) and a deferral-pointer linter that cross-references `not yet wired in P\d+` strings against the named phase's PLAN files.
5. A new required catalog-row schema field — `claim_vs_assertion_audit` — that forces every row to explicitly state how the verifier's assertion would falsify the description claim if false; runner cross-checks per Decision 3.

**What this phase does NOT ship:** the catalog-row honesty rules from F-K4 (coverage_kind, asserts cross-check, dispatch.sh wiring) — those land in **P90** as RBF-FW-06..09. The dishonest-test triage gate (F-K8) and the milestone-close adversarial pass dispatch (RBF-FW-12) also land in P90. Code/doc fixes that consume this framework (real-backend `attach`, audit log fixes, etc.) land in P91+.

This phase is the **entry point** of the v0.13.0 extension series — no upstream dependencies. Execution mode is **top-level** per CLAUDE.md ("orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`") because the work fans out across PROTOCOL.md, runners, catalog schema, verifier scripts, and the milestone-close verdict template — none of which fit `gsd-executor`'s code-write-test-commit envelope.

</domain>

<spec_lock>
## Requirements (locked via REMEDIATION-PLAN P89 + ROADMAP § Phase 89)

**6 requirements are locked.** No `89-SPEC.md` exists; the locking documents are:
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P89 — Framework fixes" + § "2 — Cross-cutting framework fixes" (F-K1 / F-K2 / F-K3 / F-K6 / F-K7)
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § "Decision 3" (claim-vs-assertion structural congruence — RBF-FW-11 + RBF-FW-12)
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 89"

Downstream agents MUST read those three documents before planning or implementing. The requirements are NOT duplicated verbatim here; the implementation-shape decisions for each REQ-ID are in `<decisions>` below.

| REQ-ID | One-liner | F-K source | Effort |
|---|---|---|---|
| **RBF-FW-01** | New `cadence: pre-release-real-backend` (env-gated; default-skip in CI; required at milestone-close) | F-K1 | M |
| **RBF-FW-02** | New `kind: shell-subprocess` verifier (real subprocess + transcript artifact; not `assert_cmd`) | F-K2 | M |
| **RBF-FW-03** | Milestone-close 9th probe — vision litmus test against real backend | F-K3 | S |
| **RBF-FW-04** | Banned-production-error-tokens — `P\d+-\d+` regex over `crates/**/*.rs` (production source only) | F-K7 | XS |
| **RBF-FW-05** | Deferral-pointer linter — pre-push grep + cross-reference against named-phase PLAN files | F-K6 | S |
| **RBF-FW-11** | `claim_vs_assertion_audit` field required on every new catalog row P89/P90 mints; runner cross-check | Decision 3 | S |

**Total effort:** 5–6 days (~18–22h) per REMEDIATION-PLAN.

**Success criteria (locked verbatim from ROADMAP § Phase 89):**

1. `quality/PROTOCOL.md` documents new cadence + kind with worked example; `quality/runners/run.py` recognizes `pre-release-real-backend` (default-skip when env not set).
2. Milestone-close verdict template carries 9th probe entry; absent ⇒ verdict graded RED.
3. Pre-push gate runs deferral-pointer linter; banned-production-error-tokens regex `P\d+-\d+` extended in banned-words.sh (or its successor for `crates/` scope — see decisions below).
4. `claim_vs_assertion_audit` field present on every new catalog row P89/P90 mints; runner cross-check passes (Decision 3).
5. Catalog-first commit mints **5+ rows** in `quality/catalogs/{agent-ux,framework}.json` with `status: NOT-VERIFIED` BEFORE implementation commits land; CLAUDE.md updated in same PR.
6. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p89/VERDICT.md`.

**In scope:**
- Runner-side cadence enum extension + env-gating logic.
- Verifier-kind dispatch table extension + transcript artifact convention.
- One worked-example `shell-subprocess` verifier (proves the kind works end-to-end against the simulator at minimum).
- Milestone-close verdict TEMPLATE update (template lives at TBD location — see open questions Q-LOC-1).
- New `quality/gates/structure/deferral-pointer-linter.sh` script + pre-push wiring.
- `crates/`-scope banned-tokens enforcement (likely a NEW script, not extension of the docs-only `banned-words.sh` — see open questions Q-SCOPE-1).
- `claim_vs_assertion_audit` schema field documentation in `quality/catalogs/README.md` + runner validation.
- 5+ catalog rows in `quality/catalogs/agent-ux.json` (or new `framework.json` — see Q-CAT-1) defining what GREEN looks like for THIS phase, minted NOT-VERIFIED in the catalog-first commit.
- CLAUDE.md updates: § "Quality Gates" cadence/kind tables, § "Build memory budget" (no-op — no cargo work), one new entry naming the deferral-pointer linter.

**Out of scope (deferred to other phases):**
- Catalog-row honesty rules — `coverage_kind: real-backend`, `expected.asserts` ↔ `asserts_passed` cross-check, `dispatch.sh` wiring requirement → **P90 RBF-FW-06..08**.
- Dishonest-test triage gate (`test-name-vs-asserts.sh`) → **P90 RBF-FW-09**.
- Honesty-spot-check meta-rule for absorption phase → **P90 RBF-FW-10**.
- Milestone-close adversarial pass dispatch (subagent reads description-only and grades) → **P90 RBF-FW-12** (P89 ships only the structural FIELD; P90 ships the dispatch).
- Backfilling `claim_vs_assertion_audit` on existing P78–P88 catalog rows → **P95 RBF-D-06** (P89's runner cross-check is gated on rows minted P89/P90 onward; legacy rows continue to validate without the field until P95 migrates them).
- Real-backend `attach` / `sync --reconcile` wiring → **P91**.
- Audit log fixes (`helper_push_*` rows; `.with_audit()` chain) → **P92**.

</spec_lock>

<decisions>
## Implementation Decisions

> All defaults below are CLAUDE-DISCRETION captures, made in autonomous mode. The planner may override if research surfaces evidence (e.g., the codebase scout finds a different runner cadence-registration pattern). Each decision cites the canonical ref it derives from.

### RBF-FW-01 — `cadence: pre-release-real-backend`

- **D-01a:** Add `pre-release-real-backend` to the `VALID_CADENCES` tuple in `quality/runners/run.py:45`. **Rationale:** the runner's existing cadence model is a flat tuple-driven enum (no class hierarchy); extending follows the established pattern. (Source: `quality/runners/run.py:45-47`.)
- **D-01b:** Gate semantics — at row-execution time in `run_row()` (line 153), if the row is tagged `cadences: ["pre-release-real-backend"]`, check `os.environ`:
  - `REPOSIX_ALLOWED_ORIGINS` MUST be set (non-empty, non-default-127.0.0.1 — value matches `/^https?:\/\/(?!127\.0\.0\.1)/`)
  - AT LEAST ONE credential set MUST be complete:
    - Confluence: `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`
    - GitHub: `GITHUB_TOKEN`
    - JIRA: `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE`
  - If env not set → `status: NOT-VERIFIED` with a `_skipped_real_backend: true` transient flag (mirrors the existing `_stale: true` pattern at line 197) + a `print_row_summary` extra string `"skipped: env not set (need REPOSIX_ALLOWED_ORIGINS + creds)"`.
  - If env IS set → invoke verifier as normal (no behavior change downstream).
  - **Rationale:** mirrors existing `is_stale()` short-circuit pattern; runner stays cross-platform stdlib-only per its anti-bloat header.
- **D-01c:** Milestone-close gate — `compute_exit_code()` does NOT need a special case: `pre-release-real-backend` rows that SKIP land as `NOT-VERIFIED`, which already flips P0+P1 exit to 1. The milestone-close ritual MUST invoke the runner with `--cadence pre-release-real-backend` AND requires GREEN (exit 0). Document this in `quality/PROTOCOL.md` § "Latency budgets" table (add row: `pre-release-real-backend | n/a (env-gated) | mandatory at milestone-close; default-skip in CI`).
- **D-01d:** **CI default-skip is automatic** — GH Actions workers don't have these credentials by default; `pre-release-real-backend` runs in CI return NOT-VERIFIED for every row, exit=1, but no CI workflow currently invokes that cadence. P89 does NOT add a CI workflow for this cadence (that would defeat the env-gating). Document explicitly: "no CI invocation; the cadence is invoked by the milestone-close ritual on a developer machine with creds set."
- **D-01e:** Worked example in `quality/PROTOCOL.md` — show a row `agent-ux/dvcs-third-arm-real-confluence` tagged `cadences: ["pre-release-real-backend"]` + the `kind: shell-subprocess` verifier from RBF-FW-02.

### RBF-FW-02 — `kind: shell-subprocess` verifier

- **D-02a:** New verifier kind name: `shell-subprocess`. Add to `quality/catalogs/README.md` § "Unified schema" `kind` enum: extend from `mechanical | container | asset-exists | subagent-graded | manual` to include `shell-subprocess`. **Rationale:** kind enum is documented in catalogs/README.md, not enforced as code in run.py (the runner dispatches on `verifier.script` extension not `kind` field today — see `run_row()` line 226-232). Adding the kind is a documentation + convention extension, not a runner-dispatch rewrite.
- **D-02b:** Verifier scripts live at `quality/gates/<dimension>/<row-slug>.sh` per existing convention (e.g., `quality/gates/agent-ux/reposix-attach.sh`). For `shell-subprocess` kind: the script MUST invoke `reposix init/attach/sync` (or `git push reposix main`) as a real subprocess — `subprocess.run(["reposix", "attach", "..."])` if Python, or `reposix attach "$@"` if bash. NOT via `cargo test --test agent_flow_real`-style envelope. The cargo-test pattern is what F-K2 explicitly targets as untrustworthy.
- **D-02c:** **Transcript artifact convention:** verifier writes to BOTH the existing `artifact` path (the JSON status artifact at `quality/reports/verifications/<dim>/<row-slug>.json`) AND a NEW transcript at `quality/reports/transcripts/<row-slug>-<RFC3339>.txt` containing:
  ```
  argv: <full argv>
  env_keys: <sorted list of env var NAMES that were set; NOT values — security>
  cwd: <working dir>
  exit_code: <int>
  --- STDOUT ---
  <stdout bytes>
  --- STDERR ---
  <stderr bytes>
  ```
  - The JSON artifact gets a NEW field `transcript_path: "quality/reports/transcripts/<row-slug>-<RFC3339>.txt"` so the verifier subagent at phase close can dereference it.
  - **Rationale:** transcript artifact is the load-bearing assertion — it proves the subprocess was invoked, not faked. Env keys (not values) keeps the security-first posture from CLAUDE.md threat-model § "Outbound HTTP allowlist" without leaking secrets.
- **D-02d:** Worked example in `quality/PROTOCOL.md` — write `quality/gates/agent-ux/shell-subprocess-example.sh` that runs `reposix --version` against the local binary as the minimum-viable proof-of-kind. Worked example exercises the kind without depending on RBF-FW-01's real-backend env (so the kind is testable in CI on the simulator path too).
- **D-02e:** Runner-side change: `run_row()` at line 226 currently dispatches on suffix (`.py` / `.sh`); shell-subprocess kind uses `.sh` and runs through the existing bash-dispatch path. NO new dispatch code needed — the kind is a convention enforced by the catalog schema + verifier subagent grading, not by the runner subprocess shape. Document this explicitly so future readers don't look for a missing dispatch table.

### RBF-FW-03 — Milestone-close 9th probe

- **D-03a:** **Both** of the following land:
  1. A new entry in the milestone-close verdict TEMPLATE (location TBD — see open questions Q-LOC-1; likely `.claude/skills/milestone-close-verdict/template.md` or in `quality/PROTOCOL.md` § "Verifier subagent prompt template" extension).
  2. A new catalog row at `quality/catalogs/agent-ux.json` — id `milestone-close/vision-litmus-real-backend` — tagged `cadences: ["pre-release-real-backend"]`, `kind: shell-subprocess`. The 9th probe is the row's verifier output; the verdict template instructs the verifier subagent to read this row's artifact + transcript before declaring GREEN.
  - **Rationale:** the verdict template alone is just prose; the catalog row makes the probe an executable, gradable thing. Decision 3 (COMPLETENESS-CHECK S2 fix) wants the litmus test as a runtime artifact, not just planning input.
- **D-03b:** **Vision litmus test definition:** the dark-factory third arm against TokenWorld (or the equivalent sanctioned reference scenario per `docs/reference/testing-targets.md`). The probe asserts:
  - Agent completes vanilla-clone + `reposix attach` + edit + `git push` end-to-end with zero in-context learning (helper teaching strings emit; no special-case prompt engineering).
  - Audit rows present in BOTH `audit_events_cache` AND `audit_events` tables (OP-3 dual-table contract).
  - `refs/mirrors/<sot>-{head,synced-at}` advanced (mirror-lag refs land per CLAUDE.md "Mirror-lag refs").
  - Verifier writes a transcript per RBF-FW-02 D-02c.
  - **Rationale:** matches the existing `quality/gates/agent-ux/dark-factory.sh dvcs-third-arm` shape (P86 invariant) but FIRES against TokenWorld instead of in-process sim.
- **D-03c:** **P89 ships the SLOT** for the 9th probe — the verifier script at `quality/gates/agent-ux/milestone-close-vision-litmus.sh` may legitimately exit `NOT-VERIFIED` at P89 close because the underlying `reposix attach <real-backend>` capability lands in P91. P89's success criterion #2 ("Milestone-close verdict template carries 9th probe entry; absent ⇒ verdict graded RED") is about the TEMPLATE/CATALOG-ROW being present, not the probe being green. The probe goes green at P97 milestone-close once P91+P92+P93+P94+P95 land their pieces. **This is explicitly NOT a "self-licensing-deferral-loop" (PATTERNS C7) failure** because: (a) the row mints `NOT-VERIFIED` not `WAIVED`, (b) it carries `blast_radius: P0`, so any milestone-close grading attempt while it's NOT-VERIFIED returns exit 1, and (c) the row's `verifier.script` MUST exist and be executable (no missing-file short-circuit) — it just legitimately returns NOT-VERIFIED until the substrate ships.
- **D-03d:** Tag-script update — `tag-v0.13.0.sh` is currently `.disabled` per `ls .planning/milestones/v0.13.0-phases/` (the v0.13.0 tag is held per Path A/Option B). The tag script that ships in P97 MUST include a guard that runs `python3 quality/runners/run.py --cadence pre-release-real-backend` and requires exit 0. P89 does NOT modify `tag-v0.13.0.sh.disabled` (out-of-phase scope) but DOES document the future requirement in `quality/PROTOCOL.md` so P97's plan-checker catches it.

### RBF-FW-04 — Banned-production-error-tokens regex

- **D-04a:** **NEW SCRIPT, not extension of existing `banned-words.sh`.** The existing `scripts/banned-words-lint.sh` (called by `quality/gates/structure/banned-words.sh`) scans **`docs/`** only — its pattern is layered around hero/below-fold/how-it-works in `docs/.banned-words.toml`. Extending it to `crates/` would conflate two different scopes and break its layered model. **Default:** create `quality/gates/structure/banned-production-tokens.sh` that scans `crates/**/*.rs` for `P\d+-\d+`. **Rationale:** keeps the docs-layered linter focused; new gate has its own catalog row + own verifier.
- **D-04b:** **Scope:** `crates/**/*.rs`, EXCLUDING:
  - `tests/` and `**/tests/**` directory contents (test names legitimately reference phase IDs in test fn names like `attach_p79_smoke_test`).
  - Lines containing `// banned-words: ok` allowlist marker (mirrors `docs/.banned-words.toml` ALLOWLIST_MARKER convention from `scripts/banned-words-lint.sh:29`).
  - `#[cfg(test)] mod tests` blocks within source files (harder to detect; defer to comment-allowlist for P89; Tree-sitter-style block detection is a P95 polish item if false-positives surface).
  - Comments? **Default: include comments in scan** — a `// TODO P79-02 wire this up` is exactly the kind of stale pointer that should fail. If a legitimate "phase-id in comment" needs to ship, the allowlist marker is the documented escape.
- **D-04c:** Pattern: `\bP[0-9]+-[0-9]+\b` (word-boundaried; matches `P79-02`, `P82-15`, doesn't match `IP4-1` or `port-79-02`). Use `grep -nHE` to mirror the existing `banned-words-lint.sh:138` pattern.
- **D-04d:** Pre-commit + pre-push wiring: catalog row `structure/banned-production-tokens` tagged `cadences: ["pre-commit", "pre-push"]`. Hook into `.githooks/pre-push` (canonical) via the runner — `python3 quality/runners/run.py --cadence pre-push` already runs all pre-push rows. NO new hook script needed.
- **D-04e:** Worked example for the planner: `crates/reposix-cli/src/attach.rs` line emitting `"P79-02 scaffold not yet wired"` (per H-A2 / vision-audit F2 / p79 F1) MUST trigger BLOCK. The fix lands in P91 (RBF-A-03), but the LINT that catches it lands here in P89.

### RBF-FW-05 — Deferral-pointer linter

- **D-05a:** New script: `quality/gates/structure/deferral-pointer-linter.sh`.
- **D-05b:** **Three regex patterns** (per F-K6 verbatim):
  - `not yet wired in P\d+`
  - `lands? (alongside\|in) P\d+`
  - `substrate-gap-deferred`
- **D-05c:** Algorithm:
  1. `grep -rnHE` the three patterns over `crates/`.
  2. For each match, parse the named phase number (`P\d+`).
  3. Resolve the phase directory: `find .planning/phases -maxdepth 1 -type d -name "${N}-*"`. If multiple matches (unlikely; phase numbers are unique), take the first.
  4. Within the resolved dir, look for `*PLAN*.md` files. If NONE exist, BLOCK with structured error: `crates/X/src/Y.rs:42 references P${N} but no .planning/phases/${N}-*/*PLAN*.md exists`.
  5. If PLAN files exist, optionally grep them for cross-reference of the deferred work (e.g., grep for the source line's keyword). **Default for P89:** require only that PLAN files EXIST in the named phase dir — content cross-reference is a P90/P95 polish (avoids over-engineering the linter on first pass).
- **D-05d:** Pre-push wiring: catalog row `structure/deferral-pointer-linter` tagged `cadences: ["pre-push"]`, `blast_radius: P1`. Lives under `quality/catalogs/structure/freshness-invariants.json` extension OR a new `quality/catalogs/structure/deferral-pointers.json` — **default: extend `freshness-invariants.json`** since it's the structure-dimension home for "no broken pointers" gates (already houses `no-loose-roadmap-or-requirements`, `no-pre-pivot-doc-stubs`, etc.).
- **D-05e:** Worked example: `crates/reposix-cache/src/foo.rs:99 // substrate-gap-deferred until P81 lands` would currently PASS (P81's PLAN files exist). After P89, if the comment changes to `// substrate-gap-deferred until P150 lands`, the linter BLOCKs (no `.planning/phases/150-*/`).

### RBF-FW-11 — `claim_vs_assertion_audit` schema field

- **D-11a:** New REQUIRED field on every catalog row minted P89/P90 onward: `claim_vs_assertion_audit` (string, ≥50 chars). Paragraph explains how the verifier's `expected.asserts` would FALSIFY the row's `description`/`comment` claim if the claim were false.
- **D-11b:** **Schema doc:** add to `quality/catalogs/README.md` § "Unified schema" table as a new row: `claim_vs_assertion_audit | yes (for rows minted ≥ 2026-05-08) | string ≥50 chars; explains how verifier asserts would falsify the description claim if false. Pre-existing rows continue to validate without this field until P95 RBF-D-06 migrates them.`
- **D-11c:** **Runner cross-check:** `quality/runners/run.py` `load_catalog()` (line 72) extension — for each row whose `last_verified` is null OR `last_verified >= "2026-05-08T00:00:00Z"` (the field's introduction date), require `claim_vs_assertion_audit` to be present + non-empty + ≥50 chars. If missing, the runner exits with `SystemExit(f"FAIL: {path}: row {row.id} missing claim_vs_assertion_audit")` BEFORE running the verifier. **Rationale:** "fail loud, structured, agent-resolvable" per `quality/PROTOCOL.md` Principle B. The date-cutoff gate prevents the runner from breaking on the 388 existing P78–P88 rows; P95's RBF-D-06 backfills those.
- **D-11d:** **Content-hash logging:** at grade time, the runner SHOULD log `sha256(claim_vs_assertion_audit)` to the artifact JSON so a verifier subagent can detect if the audit text was edited between the row mint and the grading run (drift signal). **Default:** ship this in P89 as a new artifact field `claim_vs_assertion_audit_hash`. Cheap, future-proof.
- **D-11e:** **Boundary with RBF-FW-12 (P90):** P89 ships ONLY the structural FIELD + runner cross-check. P90's RBF-FW-12 ships the milestone-close ADVERSARIAL DISPATCH — a fresh subagent reads the field + the artifact and grades whether the assertion would actually falsify the description. The dispatch lives at `quality/dispatch/milestone-adversarial.md` (per ROADMAP P90 SC #6). **Document this boundary in CLAUDE.md so the planner doesn't accidentally fold the dispatch into P89.**

### Catalog-first commit (per success criterion #5)

- **D-CAT-01:** Mint **at least 5 rows** in the catalog-first commit, BEFORE any implementation lands:
  1. `agent-ux/cadence-pre-release-real-backend` — proves RBF-FW-01 ships (asserts `VALID_CADENCES` contains the new value).
  2. `agent-ux/kind-shell-subprocess-worked-example` — proves RBF-FW-02 ships (runs the worked-example verifier).
  3. `agent-ux/milestone-close-vision-litmus-real-backend` — proves RBF-FW-03 ships SLOT (verifier exists + executable + returns NOT-VERIFIED until P91+ land — see D-03c).
  4. `structure/banned-production-tokens` — proves RBF-FW-04 ships.
  5. `structure/deferral-pointer-linter` — proves RBF-FW-05 ships.
  6. `structure/claim-vs-assertion-audit-required` — proves RBF-FW-11 runner cross-check ships (asserts the runner rejects rows lacking the field when minted after the cutoff date).
- **D-CAT-02:** All 6 rows carry `claim_vs_assertion_audit` paragraphs in the catalog-first commit (eating their own dogfood — the rows that introduce the field carry the field).
- **D-CAT-03:** Catalog file location — **default: extend `quality/catalogs/agent-ux.json`** for rows 1+2+3 and **extend `quality/catalogs/freshness-invariants.json`** for rows 4+5+6 (structure-dim invariants). **Do NOT create a new `framework.json`** unless the planner has strong evidence — the existing dimension layout (8 dimensions per CLAUDE.md "Quality Gates" § "9 dimensions") doesn't have a `framework` dimension and adding one is a schema migration that bloats P89's scope. The ROADMAP success criterion mentions `framework.json` but the catalog file naming is conventional, not contractual — every dimension-name in CLAUDE.md is satisfiable via existing files. (See open question Q-CAT-1.)

### CLAUDE.md update (per success criterion #5)

- **D-CLM-01:** Update CLAUDE.md § "Quality Gates" tables in-place per the existing pattern:
  - **9 dimensions** table — no change (no new dimension).
  - **7 cadences** table — extend to **8 cadences** by adding `pre-release-real-backend` row with `(local + milestone-close, env-gated, mandatory at tag-time, blocking)`.
  - **5 kinds** table — extend to **6 kinds** by adding `shell-subprocess` row with `(real subprocess invocation + transcript artifact)`.
- **D-CLM-02:** Add a one-paragraph entry to CLAUDE.md § "Quality Gates" pointing at the deferral-pointer linter + banned-production-tokens linter as new structure-dimension gates.
- **D-CLM-03:** Update CLAUDE.md § "Subagent delegation rules" with one new bullet: "**The milestone-close 9th probe (RBF-FW-03) is non-skippable.** Any milestone-close ritual that does not include `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades the milestone RED. The probe runs the vision litmus test against the sanctioned real backend (TokenWorld for v0.13.0); the catalog row's `verifier.script` is `quality/gates/agent-ux/milestone-close-vision-litmus.sh`."
- **D-CLM-04:** Per CLAUDE.md anti-bloat rules: prefer revising existing tables over appending new sections. The cadence table extension is the natural revision shape.

### Claude's Discretion

- **CD-01:** The exact wording of the 6 catalog-first rows' `claim_vs_assertion_audit` paragraphs is delegated to the planner — the constraint is content (≥50 chars + falsification-shape), not phrasing.
- **CD-02:** Whether to use `argparse subparsers` or extend the flat enum in `run.py` for the `--cadence` arg — keep flat enum per `run.py:45-47` precedent. (Already captured as D-01a but called out explicitly: no architectural rewrites of the runner; minimal-extension principle.)
- **CD-03:** Worked-example transcript file naming — `<row-slug>-<RFC3339>.txt` vs `<row-slug>.<RFC3339>.txt` vs ISO-without-colons. Planner picks; consistency within P89's 6 rows is the only requirement.
- **CD-04:** Whether `crates/` scope for RBF-FW-04 includes `crates/*-sim/` and `crates/reposix-swarm/` — **default: include all crates uniformly**; if a sim/swarm-specific exemption surfaces during planning, file as `// banned-words: ok` allowlist marker rather than per-crate scope rules.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope + remediation logic
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P89" + § "2 — Cross-cutting framework fixes" F-K1 / F-K2 / F-K3 / F-K6 / F-K7 — the locked REQ-IDs + their F-K source patches.
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § "S1 — chicken-and-egg" + § "Decision 3" — the structural justification for RBF-FW-11 (claim-vs-assertion congruence) and the boundary against RBF-FW-12 (P90 dispatch).
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 89" — locked success criteria + execution mode (top-level).
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/PATTERNS.md` (referenced indirectly via COMPLETENESS-CHECK) — read for C7 "self-licensing-deferral-loop" anti-pattern that D-03c explicitly defends against.
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/STRATEGIC-REFRAME.md` § Q3 + Q6 — the "redesign-with-patch-as-concrete-first-commits" + "vision-litmus-test must become a runtime artifact" framings.

### Quality framework runtime contract
- `quality/PROTOCOL.md` — runtime contract for catalog-first commits, verifier subagent dispatch, latency budgets per cadence, waiver protocol. Read § "Per-phase protocol" + § "Latency budgets" before extending the cadence table.
- `quality/catalogs/README.md` — catalog row schema spec; the place where the `claim_vs_assertion_audit` field doc lands.
- `quality/runners/run.py` — runner implementation (376 lines, stdlib-only, cross-platform). The `VALID_CADENCES` tuple at line 45 is the extension point for RBF-FW-01.
- `quality/runners/_freshness.py` — freshness helper module (extracted per Wave B pivot rule). Mirror this pattern if RBF-FW-01's env-gate logic grows beyond ~30 lines.
- `quality/gates/structure/banned-words.sh` (and its canonical impl `scripts/banned-words-lint.sh`) — pattern to MIRROR (allowlist marker convention, ERE alternation pattern), NOT to extend (scope is `docs/` only; `crates/` scope warrants a sibling script per D-04a).
- `quality/gates/agent-ux/dark-factory.sh` + `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh` — the existing dispatcher pattern + the third-arm shape that the milestone-close 9th probe MUST mirror against TokenWorld.
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` — the existing 8-probe milestone-close verdict format that gets a 9th-probe row in P97; read to understand the table shape.
- `quality/reports/verdicts/p87/` (most recent phase verdict) — pattern for per-phase verdict file shape.

### Project-level grounding
- `CLAUDE.md` § "Quality Gates — dimension/cadence/kind taxonomy" — the 9 dimensions, 7 cadences, 5 kinds tables that get extended in this phase.
- `CLAUDE.md` § "Subagent delegation rules" — meta-rule for top-level execution mode (P89 IS top-level per CLAUDE.md "Orchestration-shaped phases run at top-level").
- `CLAUDE.md` § "Push cadence — per-phase" — the `git push origin main` BEFORE verifier dispatch contract.
- `CLAUDE.md` § "Build memory budget" — no-op for P89 (no cargo work expected) but read to confirm.
- `CLAUDE.md` § "Threat model" — `claim_vs_assertion_audit_hash` per D-11d serves the same forensic shape as the audit-log dual-table principle (OP-3); read § "Audit log append-only" for the spirit.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld Confluence space, `reubenjohn/reposix` issues, JIRA `TEST` project) — RBF-FW-03's 9th probe targets one of these.
- `.githooks/pre-push` (likely; confirm during scout) — the canonical pre-push hook that invokes `quality/runners/run.py --cadence pre-push`. RBF-FW-04 + RBF-FW-05 wire through this without modifying it.

### Plan-checker upstream materials (for the planner's plan-check)
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" — the verbatim prompt the verifier subagent at phase close uses to grade P89.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (mirror these patterns; do not rewrite)

- **Runner cadence enum:** `quality/runners/run.py:45-47` — `VALID_CADENCES` flat tuple. Extension is a single-tuple-edit + a docstring update at line 11.
- **Runner short-circuit pattern:** `quality/runners/run.py:163-176` (WAIVED case) + `:181-199` (STALE case) — the established shape for "row applies to cadence but doesn't run verifier this time." RBF-FW-01's env-gate skip MUST mirror this exact shape.
- **Runner artifact-write pattern:** `quality/runners/run.py:148-150` — `write_artifact()` is the artifact write helper. RBF-FW-02's transcript artifact piggybacks on this.
- **Runner subprocess dispatch:** `quality/runners/run.py:226-247` — picks `cmd` based on script suffix; `shell-subprocess` kind per D-02e uses the `.sh` branch unchanged.
- **Catalog wrapper validation:** `quality/runners/run.py:72-81` `load_catalog()` — fails loud with `SystemExit` on schema violation. RBF-FW-11 D-11c extends this for the new required field.
- **Banned-tokens scanning convention:** `scripts/banned-words-lint.sh:107-141` `scan_files()` — the ERE alternation pattern + allowlist marker shape. `quality/gates/structure/banned-production-tokens.sh` (NEW per D-04a) mirrors this approach.
- **Existing structure-dim verifier shape:** `quality/gates/structure/no-pre-pivot-doc-stubs.sh`, `no-loose-top-level-planning-audits.sh`, `freshness-invariants.py` — RBF-FW-04 + RBF-FW-05 land as siblings.
- **Existing agent-ux dim verifier shape:** `quality/gates/agent-ux/reposix-attach.sh`, `mirror-refs-write-on-success.sh`, `dark-factory.sh` (and its dispatcher children) — RBF-FW-02's worked example + RBF-FW-03's milestone-close probe land as siblings.
- **Catalog row hand-edit precedent:** `quality/catalogs/agent-ux.json` rows already carry `_provenance_note` fields explaining hand-edits (e.g., row `agent-ux/reposix-attach-against-vanilla-clone`). New P89 rows MAY use this convention if needed.

### Established Patterns

- **Catalog-first contract** (`quality/PROTOCOL.md` § "Step 3"): the FIRST commit of P89 mints the 6 catalog rows from D-CAT-01 with `status: NOT-VERIFIED`; subsequent implementation commits cite the row id (e.g., `Implements catalog row: structure/banned-production-tokens`).
- **Stdlib-only runner** (`quality/runners/run.py:1-30` header): no third-party deps in the runner. RBF-FW-11's runner cross-check MUST stay stdlib-only.
- **Anti-bloat caps** (`quality/PROTOCOL.md` § "Anti-bloat rules per surface"):
  - `run.py` ≤350 lines (currently 376 — already over by 26; flag as a polish item if RBF-FW-01 + RBF-FW-11 push it further. Default: factor env-gate + claim-vs-assertion-audit logic into a sibling module like `_realbackend.py` + `_audit_field.py` per the `_freshness.py` precedent at line 34).
  - `quality/PROTOCOL.md` ≤500 lines (current size unverified; check during planning).
  - `CLAUDE.md` enforced via `scripts/banned-words-lint.sh` — no length cap but new sections cross-reference rather than duplicate.
- **Verifier "fail loud, structured, agent-resolvable"** (`quality/PROTOCOL.md` § "Principle B"): every new gate emits stderr with a slash-command hint pointing to the recovery path. RBF-FW-04 stderr should hint at `// banned-words: ok` allowlist; RBF-FW-05 stderr should hint at the missing PLAN file.
- **ERE alternation + allowlist marker** (`scripts/banned-words-lint.sh:117-141`): the canonical pattern for production-source token scanning.
- **Top-level execution mode** (`CLAUDE.md` § "Subagent delegation rules"): orchestrator IS the executor; `gsd-executor` cannot run this phase (no `Task` tool, no depth-2 spawning). Each catalog row's verifier may be authored by a sub-subagent dispatched from the orchestrator.

### Integration Points

- **Pre-push hook:** `.githooks/pre-push` (canonical) → `python3 quality/runners/run.py --cadence pre-push` → discovers + runs all rows tagged `cadences: ["pre-push"]`. RBF-FW-04 + RBF-FW-05 catalog rows wire through this WITHOUT modifying the hook script.
- **Tag-time guard (P97 future-coupling):** `tag-v0.13.0.sh.disabled` is currently disabled per Path A/Option B. P97 will re-enable + extend it; P89 documents the future contract in `quality/PROTOCOL.md` so P97's plan-checker catches the missing 9th-probe guard.
- **Verifier subagent at phase close:** `quality/PROTOCOL.md` § "Verifier subagent prompt template" — the prompt P89's verifier subagent uses. P89 does NOT modify this template (no scope creep); P90's RBF-FW-10 is the absorption-phase honesty-rule update.
- **CLAUDE.md update in same PR:** mandatory per `quality/PROTOCOL.md` Step 5 (QG-07). RBF-FW-04 + RBF-FW-05 + RBF-FW-01 + RBF-FW-02 + RBF-FW-11 all introduce new gates/conventions, so CLAUDE.md table updates per D-CLM-01..04 are in-scope for the same PR.
- **CHANGELOG:** P89 is internal infrastructure; CHANGELOG entries for the v0.13.0 extension series go through P97. P89 does NOT touch CHANGELOG.

</code_context>

<specifics>
## Specific Ideas

### Catalog-first commit shape (worked example for the planner)

The first commit of P89 should look like:

```
catalog(P89): mint 6 framework-fix rows NOT-VERIFIED

Adds:
- agent-ux/cadence-pre-release-real-backend       (RBF-FW-01)
- agent-ux/kind-shell-subprocess-worked-example   (RBF-FW-02)
- agent-ux/milestone-close-vision-litmus-real-backend (RBF-FW-03; SLOT)
- structure/banned-production-tokens              (RBF-FW-04)
- structure/deferral-pointer-linter               (RBF-FW-05)
- structure/claim-vs-assertion-audit-required     (RBF-FW-11)

Each row carries claim_vs_assertion_audit (eating own dogfood).
status: NOT-VERIFIED on all six. Verifier scripts land in
subsequent commits per the F-K2/F-K3 conventions documented in
this commit's catalog rows + the still-pending PROTOCOL.md edits.

Implements: REMEDIATION-PLAN P89 SC #5 (catalog-first contract).
```

### `claim_vs_assertion_audit` paragraph templates (worked examples for D-CAT-02)

For row `agent-ux/cadence-pre-release-real-backend`:
> "The verifier asserts `quality/runners/run.py` `VALID_CADENCES` tuple contains the literal string `pre-release-real-backend` AND that a probe row tagged with that cadence runs only when REPOSIX_ALLOWED_ORIGINS + a credential set are present. If the description's claim ('the new cadence is env-gated and default-skips in CI') were false — e.g., the runner ran the verifier with no env vars set — the assertion would fail because the env-gate short-circuit would not fire and the verifier would attempt a real-backend HTTP call against a missing allowlist."

For row `structure/banned-production-tokens`:
> "The verifier asserts `grep -nHE '\bP[0-9]+-[0-9]+\b' crates/**/*.rs` (excluding tests/ and allowlist-marker lines) returns zero matches. If the description's claim ('phase IDs are banned in production source') were false — e.g., a `P79-02 scaffold` literal stayed in `crates/reposix-cli/src/attach.rs` — the assertion would fail because grep would report the match line and the verifier would exit 1."

### Worked example for the milestone-close 9th probe (D-03b illustrating the SLOT shape)

The verifier `quality/gates/agent-ux/milestone-close-vision-litmus.sh` at P89 close looks like:

```bash
#!/usr/bin/env bash
# milestone-close-vision-litmus — RBF-FW-03 SLOT.
# Substrate dependency: P91 (real-backend attach), P92 (audit log), P93 (cache-coherence),
# P94 (bus tree), P95 (claim qualifier). Until those land, this verifier
# legitimately returns NOT-VERIFIED — the row's blast_radius=P0 means any
# milestone-close grading attempt while NOT-VERIFIED returns exit 1.

set -euo pipefail

if [[ -z "${REPOSIX_ALLOWED_ORIGINS:-}" || -z "${ATLASSIAN_API_KEY:-}" ]]; then
    echo "SKIP: real-backend env not set" >&2
    exit 75  # EX_TEMPFAIL — runner treats as NOT-VERIFIED
fi

# TODO P91-P95: invoke the real-backend dark-factory third arm against TokenWorld
# and assert the 4 invariants from RBF-FW-03 D-03b.
echo "NOT-VERIFIED: substrate not landed (depends on P91+P92+P93+P94+P95)" >&2
exit 75
```

The exit-75 → NOT-VERIFIED mapping needs runner support; if the runner doesn't already map specific exit codes to NOT-VERIFIED, the verifier writes the artifact directly with the NOT-VERIFIED status and exits 0. **Planner: verify which path the runner supports during scout.**

</specifics>

<deferred>
## Deferred Ideas

Items surfaced during scout that belong in OTHER phases:

### Out of P89 scope, into P90:
- **F-K4 catalog-row honesty rules** (coverage_kind, asserts cross-check, dispatch.sh wiring) → RBF-FW-06..08.
- **F-K8 dishonest-test triage** (`test-name-vs-asserts.sh`) → RBF-FW-09.
- **Honesty-spot-check meta-rule** for absorption phase → RBF-FW-10.
- **Milestone-close adversarial pass dispatch** (subagent reads description-only and grades whether assertion would falsify) → RBF-FW-12. P89 ships the FIELD; P90 ships the DISPATCH.

### Out of P89 scope, into P95:
- **Backfill `claim_vs_assertion_audit` on existing P78–P88 catalog rows** → RBF-D-06. P89's runner cross-check is gated on rows minted after 2026-05-08T00:00:00Z so the 388 legacy rows continue to validate.
- **Dead `pre-pr` cadence cleanup** (per W2 in COMPLETENESS-CHECK — `cadences: ["pre-pr"]` rows have no CI invocation today) → P95 polish if it surfaces, otherwise v0.14.0.

### Out of P89 scope, into P97 (milestone-close):
- **Tag-script (`tag-v0.13.0.sh.disabled` re-enable + 9th-probe guard wiring)** → P97 RBF-G-04. P89 documents the future contract in PROTOCOL.md.

### Surfaced during scout, defer to v0.14.0 unless owner overrides:
- **`quality/PROTOCOL.md` line cap (currently ≤500)** — adding the worked examples for cadence + kind + 9th probe will likely push it over. If so, factor § "Latency budgets" + § "Verifier subagent prompt template" into separate `quality/gates/<dim>/README.md` files per the PROTOCOL.md anti-bloat rule. Defer the factoring decision to the planner.
- **Runner over-budget (`run.py` is 376 lines vs ≤350 cap)** — RBF-FW-01 + RBF-FW-11 push it further. Factor into `_realbackend.py` + `_audit_field.py` siblings per `_freshness.py` precedent. **This factoring is in-scope for P89** (mentioned here as a planning hint, not a deferral).
- **Cross-AI peer review of P89/P90 (M1 in COMPLETENESS-CHECK)** — `gsd-review` to Codex/Cursor/Gemini before P89-01 ships. Owner directive: deferred (autonomous mode); planner may revisit if confidence in P89's framework changes is LOW.

### Reviewed Todos (not folded)
None — no `.planning/todos/` cross-reference performed in autonomous mode. If todos exist, the planner can reconsider during plan-check.

</deferred>

<open_questions>
## Open Questions for the Planner

These are questions where the autonomous discussion captured a default but the planner SHOULD validate during research/planning. Each carries `VALIDATE during research` and a proposed default.

### Q-LOC-1 — Where does the milestone-close verdict TEMPLATE live?
- **Default:** `.claude/skills/milestone-close-verdict/template.md` does NOT obviously exist (no skill scaffolding seen during scout). Most likely the "template" is implicit in `quality/PROTOCOL.md` § "Verifier subagent prompt template" or in the per-phase verdict shape at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (the latter is a verdict ARTIFACT, not a template).
- **Action for planner:** grep for `milestone-close` and `9th probe` in `.claude/skills/`, `quality/`, and `.planning/`. If no template file exists, P89 CREATES one at `quality/dispatch/milestone-close-verdict.md` (paralleling the P90 RBF-FW-12 dispatch path).
- **VALIDATE during research.**

### Q-SCOPE-1 — Should `crates/` banned-tokens be a NEW script or extension of `banned-words.sh`?
- **Default:** NEW script per D-04a (different scope, different layered model). Existing `scripts/banned-words-lint.sh` is `docs/`-only.
- **Action for planner:** confirm by reading the full `scripts/banned-words-lint.sh` (verify the layered-paths model is hardcoded to `docs/`). If confirmed, mint the new sibling.
- **VALIDATE during research.**

### Q-CAT-1 — Does `framework.json` need to exist as a new catalog file?
- **Default:** NO per D-CAT-03 — split rows across existing `agent-ux.json` + `freshness-invariants.json`. CLAUDE.md "9 dimensions" doesn't list a `framework` dimension.
- **Action for planner:** the ROADMAP P89 SC #5 mentions `quality/catalogs/{agent-ux,framework}.json` literally. If the planner's reading of ROADMAP requires literal compliance, mint `framework.json` + add `framework` to the dimension list in `quality/catalogs/README.md`. But this is a schema migration; default avoids it unless owner pushes back.
- **VALIDATE during research.**

### Q-EXIT-1 — Does the runner support exit-75 (or any specific exit code) → NOT-VERIFIED mapping for verifier scripts?
- **Default:** unknown from current scout. Per `quality/runners/run.py:276-283`, the runner maps:
  - `timed_out` → FAIL
  - `exit_code == 0` → PASS
  - `exit_code == 2` → PARTIAL
  - else → FAIL
  - There is NO mapping for "verifier wants to declare NOT-VERIFIED via exit code." The verifier MUST write the artifact directly with status NOT-VERIFIED to express skip-with-context (e.g., "real-backend env not set, defer grading").
- **Action for planner:** the worked-example `milestone-close-vision-litmus.sh` (per D-03c + Specifics) needs to write the artifact with `status: NOT-VERIFIED` directly rather than relying on an exit-code → NOT-VERIFIED mapping. Confirm by reading `run_row()` line 256-285 carefully.
- **VALIDATE during research.**

### Q-RUN-1 — Should the env-gate logic in RBF-FW-01 live in `run_row()` or a new `_realbackend.py` module?
- **Default:** new `_realbackend.py` module per the `_freshness.py` precedent (`run.py:34`) since `run.py` is already over its 350-line cap.
- **Action for planner:** if the env-gate logic fits in <20 lines, inline in `run_row()` is acceptable. If it grows (validation, error messages, `assert_env_complete()` helpers), factor.
- **VALIDATE during research.**

### Q-DEFLINK-1 — Should the deferral-pointer linter cross-reference PLAN file CONTENT or just EXISTENCE?
- **Default:** EXISTENCE only for P89 (D-05c step 5 step "**Default for P89**"). Content cross-reference is a P90/P95 polish.
- **Action for planner:** if the planner's research shows existence-only check is too lax (e.g., a phase dir exists but its PLAN files don't actually mention the deferred work), upgrade to grep-for-keyword. Adds ~10 lines to the linter.
- **VALIDATE during research.**

### Q-PROBE-OWNER-1 — Which BACKEND does the milestone-close 9th probe target?
- **Default:** TokenWorld (Confluence) per D-03b — the v0.13.0 architecture-sketch posits Confluence-as-canonical-SoT and TokenWorld is the sanctioned target per `docs/reference/testing-targets.md`.
- **Action for planner:** if the planner's research surfaces a STRONGER candidate (e.g., the dark-factory third-arm against `reubenjohn/reposix` issues is operationally simpler), document the rationale + override.
- **VALIDATE during research.**

### Q-DOC-LINT-1 — Where in `CLAUDE.md` does the new `pre-release-real-backend` cadence get cross-referenced?
- **Default:** § "Quality Gates — dimension/cadence/kind taxonomy" 7 cadences table → 8 cadences (per D-CLM-01).
- **Action for planner:** also cross-reference in § "Threat model" (since the cadence is the gate that ensures real-backend HTTP allowlist enforcement is testable end-to-end) and § "Push cadence — per-phase" (since the milestone-close ritual now requires the cadence's exit 0).
- **VALIDATE during research.**

</open_questions>

<task_breakdown_hint>
## Suggested Task Breakdown for the Planner

Per CLAUDE.md catalog-first rule + minimum-six-task expectation. Order matters: catalog rows MUST land BEFORE implementation per `quality/PROTOCOL.md` Step 3.

| Order | Task | REQ-IDs | Effort | Notes |
|---|---|---|---|---|
| **T1** | Mint 6 NOT-VERIFIED catalog rows in `agent-ux.json` + `freshness-invariants.json` | All 6 | XS | The catalog-first commit per success criterion #5. ALL subsequent commits cite a row id. |
| **T2** | RBF-FW-04 — Banned-production-tokens linter | RBF-FW-04 | XS | Smallest implementation; quick-win to validate the catalog-first contract works end-to-end. New `quality/gates/structure/banned-production-tokens.sh` + pre-push wiring via runner cadence. |
| **T3** | RBF-FW-05 — Deferral-pointer linter | RBF-FW-05 | S | New `quality/gates/structure/deferral-pointer-linter.sh`; resolves named phase numbers against `.planning/phases/N-*/PLAN*.md` existence. |
| **T4** | RBF-FW-01 — `pre-release-real-backend` cadence + env-gate | RBF-FW-01 | M | Extends `VALID_CADENCES` + factors env-gate logic into `_realbackend.py` per `_freshness.py` pattern. PROTOCOL.md table extension. CLAUDE.md cadence table extension. |
| **T5** | RBF-FW-02 — `kind: shell-subprocess` + transcript artifact convention | RBF-FW-02 | M | Schema doc in `catalogs/README.md`; worked-example verifier `quality/gates/agent-ux/shell-subprocess-example.sh`; transcript path convention in run.py write_artifact extension. |
| **T6** | RBF-FW-11 — `claim_vs_assertion_audit` field + runner cross-check | RBF-FW-11 | S | Schema doc in `catalogs/README.md`; runner cross-check (date-cutoff-gated) in `_audit_field.py` (or `load_catalog()` extension); `_hash` artifact field. |
| **T7** | RBF-FW-03 — Milestone-close 9th probe SLOT | RBF-FW-03 | S | Verifier script at `quality/gates/agent-ux/milestone-close-vision-litmus.sh` (legitimately NOT-VERIFIED until P91+); verdict template at `quality/dispatch/milestone-close-verdict.md` (per Q-LOC-1) OR PROTOCOL.md extension. |
| **T8** | CLAUDE.md update + verifier-subagent dispatch | (all) | S | Per D-CLM-01..04 — extend cadence + kind tables; new entry for the structure-dim linters. Phase-close: `git push origin main` THEN dispatch unbiased verifier subagent per PROTOCOL.md Step 7. |

**Effort total:** ~5 days (XS+XS+S+M+M+S+S+S = ~18–22h per REMEDIATION-PLAN). Within the locked envelope.

**Parallelism note:** T2 + T3 can run in parallel (disjoint files). T4 + T5 share `run.py` extension surface — run sequentially. T6's runner extension also touches run.py — sequence after T4 + T5. T7's verifier script is independent of the run.py work. T8 is the wrap-up.

**Plan-check materials for `gsd-plan-checker`:** REMEDIATION-PLAN P89 § success criteria + ROADMAP § Phase 89 success criteria + this CONTEXT.md `<decisions>` section. The plan-checker validates that every locked SC has a corresponding task + verifier output.

</task_breakdown_hint>

---

*Phase: 89-framework-fixes-cadence-shell-kind*
*Context gathered: 2026-05-08*
*Discussion mode: autonomous (per owner directive)*
