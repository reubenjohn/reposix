---
phase: 89
reviewers: [claude, codex, gemini]
reviewed_at: 2026-05-08T21:25:00Z
plans_reviewed: [89-PLAN-OVERVIEW.md, 89-01-PLAN.md, 89-02-PLAN.md, 89-03-PLAN.md, 89-04-PLAN.md, 89-05-PLAN.md, 89-06-PLAN.md, 89-07-PLAN.md, 89-08-PLAN.md]
in_house_plan_check_verdict: PASS (89-PLAN-CHECK.md, 3 LOW recommendations)
---

# Cross-AI Plan Review — Phase 89 (P89 framework fixes)

Three independent AI lineages reviewed the P89 plan set against the same artifact bundle (CONTEXT, RESEARCH, VALIDATION, PLAN-OVERVIEW, 8 per-task PLANs, in-house PLAN-CHECK). Each reviewer was asked to be hostile and to interrogate implicit assumptions, unstated dependencies, boundary conditions, backward-compat, and failure modes — particularly because P89 is the framework-fix phase and cannot honestly self-grade (Strategic Reframe Q1 + Decision 3).

**Verdict: REVISE (then proceed).** Two distinct HIGH-severity concerns surfaced across the lineages — neither flagged by the in-house plan-checker. A targeted replan addresses both without touching the wave decomposition.

## Claude Review

(Claude Opus 4.7 1M, separate session — independent of in-house plan-checker.)

### Summary

The plan WILL achieve P89's stated goal but with three execution-time hazards the in-house plan-checker missed. The catalog-first contract is genuinely honored; the C7 self-licensing-deferral defense in 89-06 is structurally sound; the runner cross-check in 89-07 is the strongest possible enforcement (load-time `SystemExit`). The two planner overrides (regex tightening + 3+3 catalog split vs. literal `framework.json`) are well-grounded and documented at every relevant surface. **However**, three concerns (one MEDIUM, two LOW) warrant resolution before or during execution.

### Strengths

- **Genuine catalog-first integrity.** 89-01 mints all 6 rows; 89-02..89-08 `files_modified` lists exclude `quality/catalogs/*.json`. Spot-checked. Rows reference verifier scripts that don't yet exist — the runner correctly handles missing scripts as NOT-VERIFIED (`run.py:204-221`), so dry-runs don't crash.
- **Dogfooding RBF-FW-11.** Row 6 (`structure/claim-vs-assertion-audit-required`) is the row that introduces the field, AND carries the field. The chicken-and-egg risk is genuinely closed.
- **Dual-path SLOT semantics in 89-06.** Without env, runner short-circuits via `_realbackend.is_skipped` → NOT-VERIFIED. With env, script writes NOT-VERIFIED artifact + exits 1 (FAIL). Either way, `blast_radius: P0` blocks milestone-close GREEN.
- **Wave 2 disjointness holds.** Independently verified: 89-02, 89-03, 89-05 have no file overlap. Critically, NEITHER 89-02 NOR 89-05 modifies `.githooks/pre-push` (both wire via catalog-row discovery).
- **RBF-FW-04 regex tightening empirically grounded.** The override from `\bP\d+-\d+\b` → `\bP\d{2,3}-\d+\b` is justified by enumerated false positives in `crates/reposix-core/src/error.rs:54,81` and `crates/reposix-remote/src/{main.rs:439, bus_handler.rs:25,112,222}`. The miss-class (`P0-X`, `P1-X`, `P9-X`) is intentional and explicit.

### Concerns

#### MEDIUM — Worked-example claim/assertion mismatch in 89-04

The catalog row `agent-ux/kind-shell-subprocess-worked-example` (minted in 89-01) lists as its first `expected.asserts` entry:

> `"worked-example verifier exits 0 against local reposix binary"`

But the verifier in `89-04-PLAN.md` step 4 explicitly falls back to `bash --version` if the `reposix` binary is unavailable. When the fallback fires, the verifier exits 0, transcript machinery passes all 4 asserts, AND the catalog row flips to PASS — but the first `expected.asserts` claim ("against local reposix binary") is **literally not satisfied**. This is exactly the failure mode RBF-FW-11 is designed to surface.

If the verifier subagent at phase close reads `claim_vs_assertion_audit` carefully, it should grade RED on this row. **This is the highest-leverage finding** because it tests whether P89's own framework catches a real claim/assertion mismatch in P89's own catalog.

**Resolution options (pick one before 89-04 ships):**
- **(a)** Tighten `expected.asserts` first entry to `"exits 0 against a real binary subprocess (reposix preferred; bash --version as CI fallback when cargo target absent)"` AND update the row's `claim_vs_assertion_audit` to match.
- **(b)** Remove the bash fallback. Require `cargo build -p reposix-cli` as a precondition. (Violates P89's no-cargo guarantee.)
- **(c)** Add a `pre-push` task to ensure `target/debug/reposix` exists before the verifier runs.

**Recommend (a).**

#### LOW — `parse_rfc3339` parser-divergence risk in 89-07

`_audit_field.validate_row(row, path, parse_rfc3339)` accepts the parser as a parameter — the test file uses a stub. If the runner's actual `parse_rfc3339` (`run.py:55-59`) is stricter, legacy `last_verified` strings using `Z` suffix would raise inside `validate_row`, **breaking catalog loading entirely**.

**Recommendation:** Insert a smoke step before 89-07 step 5 that walks every existing catalog row's `last_verified` string through the production parser to surface format incompatibilities independently of the cross-check.

#### LOW — Allowlist-marker dead-code coupling between P89 and P91

89-02 step 4 instructs adding `// banned-words: ok — P91 RBF-A-03 will remove this string` markers. After P91 RBF-A-03 ships and removes those strings, the allowlist comments become dead — but P91 has no explicit instruction to clean them up.

**Recommendation:** Add to 89-02 step 4 a final sub-bullet: *"File a pointer in `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` noting that when P91 RBF-A-03 scrubs the deferral strings, it MUST also remove the corresponding `// banned-words: ok` allowlist comments."*

### Suggestions (additional)

- **89-08 step 9:** verifier dispatch prompt should `grep -n` the section header in `quality/PROTOCOL.md` and quote the prompt body explicitly into the Task invocation rather than rely on a relative reference.
- **89-04 step 6:** RESEARCH § Q-SHELL-1 confirms `run.py:260-264` already preserves verifier-written body bytes. The proposed insertion may be redundant.
- **89-04 step 4:** the worked-example script's `REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." …)"` resolves to `quality/`, not the repo root — should be `../../..`. Compare with `lib/transcript.sh` in the same task which is correct. Off-by-one (matches plan-checker REC-3).
- **89-08 step 5:** the runner writes artifacts to `quality/reports/verifications/` and `quality/reports/transcripts/`. Sub-subagent should `grep -i 'quality/reports' .gitignore` before pushing.
- **9th-probe `with-env` diagnostic UX (89-06):** when env IS set but substrate not landed, runner reports row as FAIL. Consider extending `_realbackend.py` (89-03) to recognize a `_substrate_not_landed: true` transient surfacing as `(skipped: substrate not landed [P91-P95])` in runner output.

### Risk Assessment

**LOW–MEDIUM.** The MEDIUM finding (worked-example claim/assertion mismatch in 89-04) is the only one that could cause P89 itself to ship with an internal hypocrisy detectable by its own framework. Resolving it via option (a) is a 30-minute edit during 89-01 + 89-04. The two LOW findings are execution-time defenses-in-depth that the existing acceptance criteria will likely catch on first run. The plan-checker's PASS verdict stands; this review adds the three items above as concrete fixes the executing orchestrator should apply.

---

## Codex Review

(GPT-5 / Codex CLI — independent lineage.)

### Summary

The P89 plan is broadly sound and should achieve the phase goal: it lands the framework primitives subsequent phases depend on, and it treats the chicken-and-egg risk seriously by making catalog rows first-class artifacts before implementation. The plan has good goal-backward traceability, explicit top-level execution mode, and concrete verifier commands for each success criterion. Pass for execution with a few fixes, mostly around exact runner semantics, no-cargo discipline, and the banned-token regex trade-off.

### Strengths

- **Catalog-first is explicit and credible.** `89-01` is isolated as Wave 1, modifies only catalog files, and mints all six rows before verifier scripts land.
- **Good decomposition by requirement.** Each RBF-FW requirement has one owning task and a named verifier command.
- **Top-level execution is prominent.** `89-PLAN-OVERVIEW.md` makes the "NOT `/gsd-execute-phase`" rule hard to miss.
- **The 9th-probe SLOT is honest.** The `P0` blast radius, never-WAIVED rule, and explicit dependency on P91–P95 are the right guardrails against a self-licensing deferral.
- **The `claim_vs_assertion_audit` design is pragmatic.** Load-time validation plus date cutoff avoids breaking legacy rows while making new rows accountable.
- **The plan correctly avoids a new `framework` dimension.**

### Concerns

#### HIGH — The no-cargo guarantee is overstated

`89-04` says the worked example invokes `reposix --version` or falls back to `bash --version` if no binary exists. That weakens the claim that `kind: shell-subprocess` proves reposix subprocess behavior. If `reposix` is absent, the row can PASS while never invoking reposix. That is "met in name only" risk for RBF-FW-02.

#### MEDIUM — `shell-subprocess` is only conventionally enforced

The plan acknowledges this, but P89's goal is framework trustworthiness. A catalog row can declare `kind: shell-subprocess` and point to a non-transcript `.sh` script unless the verifier subagent catches it. P89 should at least add a lightweight mechanical check that `kind == shell-subprocess` rows have artifacts with `transcript_path`.

#### MEDIUM — Banned-token regex tightening may miss real leaks

Tightening to `\bP\d{2,3}-\d+\b` avoids legitimate `P1-1` audit IDs, but it also permits user-facing production strings like `P1-1 not implemented` or `P9-2 scaffold`. If the invariant is "no internal phase IDs in production strings," the tightened regex encodes current historical phase numbering rather than the actual class of leak.

#### MEDIUM — `89-02` plans to edit `crates/` despite the no-cargo note

The phase says it does not touch crates except grep, but `89-02` explicitly adds `// banned-words: ok` markers to Rust source files. That is not just read-only scanning. It likely does not require cargo, but the plan should stop claiming crates are untouched.

#### MEDIUM — SLOT verifier exit semantics are muddy

`89-06` says "FAIL-with-NOT-VERIFIED-artifact" when env is set. If the runner computes status from exit code and overwrites or annotates artifact status as FAIL, downstream grading could see RED rather than NOT-VERIFIED. Needs a targeted test.

#### LOW — Wave 2 is file-disjoint but commit-order fragile through catalog rows

`89-02` and `89-05` depend on catalog rows from `89-01`, but if parallel workers run verifier commands before `89-01` is merged into their branch/workspace, they may fail discovery. The orchestrator must merge/land `89-01` before spawning Wave 2.

#### LOW — `quality/PROTOCOL.md` may become over-budget

The plan notes this as possible but does not define a stop condition. Adding worked examples could push past the anti-bloat cap.

### Suggestions

- In `89-04-PLAN.md`, remove the `bash --version` fallback or make it a clearly separate transcript-helper smoke test. The `kind-shell-subprocess-worked-example` row should invoke a real reposix binary, even if that means using `cargo run -p reposix-cli -- --version` with an explicit note that this is the only allowed cargo-adjacent exception, or prebuilding is required.
- Add a small structure verifier in P89 or strengthen `89-04`'s verifier: scan all catalog rows with `kind: shell-subprocess` and assert their artifact JSON contains `transcript_path` after execution.
- In `89-02-PLAN.md`, clarify the invariant: either "ban v0.13+ phase tokens only" and keep `P\d{2,3}-\d+`, or "ban all phase-like IDs in production user-facing strings" and use a smarter allowlist/classifier.
- Update the no-cargo/no-crates statements in `89-PLAN-OVERVIEW.md` and `89-08-PLAN.md`: P89 may edit Rust comments for allowlist markers, but should not run cargo unless `89-04` explicitly chooses the reposix binary route.
- Add an explicit test for the 9th-probe runner path: run the SLOT row with synthetic env and assert the final observable state is intentionally non-green with the expected `substrate_not_landed` artifact.
- In `89-08`, require the verifier dispatch prompt to include the six exact row IDs and artifact paths.

### Risk Assessment

**MEDIUM.** The plan is well-structured and likely executable, but P89 is foundational, so small semantic gaps matter more than usual. The biggest risks are not missing tasks; they are "proof shape" issues: a shell-subprocess example that can pass without reposix, a new kind that remains mostly conventional, and SLOT status semantics that may depend on how the runner preserves verifier-written artifacts. Fix those before execution and the plan drops to LOW risk.

---

## Gemini Review

(Gemini 2.0 Flash — independent lineage. First attempt failed on stdin overflow; second attempt with file-read tools succeeded.)

### Summary

The plan is exceptionally thorough, structurally sound, and will successfully deliver the P89 framework infrastructure. It carefully navigates the "chicken-and-egg" bootstrap phase by ensuring catalog rows are minted first (and eat their own dogfood), and accurately maps all 6 ROADMAP requirements to concrete, verifiable tasks. The overrides made by the planner (e.g., regex tightening, `framework.json` rejection) demonstrate strong domain reasoning. The plan is ready for execution, provided a critical false-negative in the deferral linter is patched.

### Strengths

- **Catalog-First Integrity:** 89-01 mints all 6 rows with `status: NOT-VERIFIED` and includes the newly introduced `claim_vs_assertion_audit` field on itself, perfectly satisfying the bootstrap requirement.
- **Planner Overrides:** Tightening the banned-tokens regex is an excellent, proactive save. Splitting rows across existing dimensions instead of migrating to a new `framework` dimension prevents unnecessary scope bloat.
- **Anti-C7 Defense:** The 9th-probe SLOT verifier (89-06) brilliantly handles the lack of substrate by intentionally yielding `NOT-VERIFIED` coupled with `blast_radius: P0`, successfully blocking milestone-close without falsely claiming a RED failure.
- **Fail-Loud Load Verification:** Putting the `claim_vs_assertion_audit` cross-check in `load_catalog()` ensures immediate, structured feedback before any verifiers waste time running.

### Concerns

#### HIGH — Deferral-Pointer Linter False Negative (89-05)

The script parses phase numbers using `grep -oE 'P[0-9]+'`. If a developer writes `// substrate-gap-deferred` with NO phase number attached, the `grep` command returns empty, the inner `while read N` loop never executes, and the linter **silently passes**. This completely defeats the cross-reference goal. The script must assert that at least one phase number was extracted if a pattern matches.

#### MEDIUM — Python Import Context in `run.py` (89-07 & 89-03)

The plan suggests using relative imports like `from . import _audit_field`. Because `run.py` is invoked directly as a script (`python3 quality/runners/run.py`), relative imports will crash with `ImportError: attempted relative import with no known parent package`. You must use absolute imports (e.g., `import _audit_field`) assuming the runner directory is in `sys.path` during execution.

#### MEDIUM — SLOT Verifier Status Overwrite (89-06)

The 9th probe SLOT verifier writes an artifact with `"status": "NOT-VERIFIED"` and then executes `exit 1`. If the runner synthesizes the final artifact and blindly overwrites `status` with `"FAIL"` due to the non-zero exit code, the honest `NOT-VERIFIED` signal will be destroyed on disk. This could cause the verifier subagent to grade the outcome as a catastrophic RED rather than a legitimate deferral. Ensure `run.py` respects a pre-existing `NOT-VERIFIED` status, or adjust the exit code mapping.

#### LOW — Worked-Example CWD Resolution (89-04)

`REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." &> /dev/null && pwd)"` in `shell-subprocess-example.sh` resolves to `quality/`, not the workspace root. It needs to be `../../..`.

#### LOW — Date Cutoff Format Parsing (89-07)

The cutoff date is defined as `2026-05-08T00:00:00+00:00`. Depending on the exact implementation of `parse_rfc3339`, it might expect a literal `Z` instead of the `+00:00` offset.

### Suggestions

- **Fix 89-05 (Deferral Linter):** Modify the loop logic to track if extraction succeeded; emit an explicit error when a deferral pattern matches but no phase number is present.
- **Fix 89-07 & 89-03 (Imports):** Instruct sub-subagents to strictly use `import _realbackend` and `import _audit_field` rather than `from . import …` in `run.py`.
- **Fix 89-07 (Date format):** Change `CUTOFF_ISO` in `_audit_field.py` to use `Z`: `2026-05-08T00:00:00Z`.
- **Confirm 89-04 (Artifact Merge):** Instruct the 89-04 subagent to verify whether `run.py` already preserves the `transcript_path` during its dictionary merge.

### Risk Assessment

**LOW.** Despite the high-severity logic gap found in the deferral linter, the overall orchestration, execution environment, and constraint handling of the plan are robust. Incorporating the suggested fixes into the executing subagents' prompts will guarantee a smooth implementation.

---

## Consensus Summary

### Agreed Strengths (mentioned by 2+ reviewers)

- **Catalog-first integrity (3/3).** All three reviewers verified 89-01 mints all 6 rows BEFORE any verifier scripts land.
- **`claim_vs_assertion_audit` dogfooding (3/3).** The row that introduces RBF-FW-11 carries the field itself.
- **9th-probe SLOT semantics (3/3).** All three confirm the dual-path NOT-VERIFIED design (env-skip vs. exit-1) is the right anti-C7 defense.
- **RBF-FW-04 regex tightening (3/3).** All three accept the `\bP\d{2,3}-\d+\b` override + `CHANGELOG.md` exclusion as well-grounded.
- **3+3 catalog split vs. literal `framework.json` (3/3).** All three accept the planner's rejection of a new dimension.
- **Wave 2 disjointness (3/3 indirectly).** Independent verification: no `.githooks/pre-push` overlap; both linters wire via catalog-row discovery.

### Agreed Concerns (raised by 2+ reviewers — highest priority)

#### HIGH — bash-fallback / no-cargo overstatement in 89-04 + 89-PLAN-OVERVIEW (Codex HIGH, Claude MEDIUM)

The `shell-subprocess-example.sh` falls back to `bash --version` when `reposix` binary is absent. The catalog row's first `expected.asserts` claims "exits 0 against local reposix binary" — the bash-fallback doesn't satisfy this, but the verifier still exits 0 and the row flips to PASS. P89's own RBF-FW-11 framework should catch this as an internal hypocrisy at phase close. **Plus:** 89-PLAN-OVERVIEW + 89-08-PLAN claim "no cargo work" but 89-02 step 4 actually edits `.rs` files for allowlist markers. Two sub-issues with one root cause: the no-cargo guarantee is wishful thinking masking a worked-example that doesn't prove the kind it's meant to demonstrate.

**Resolution direction (the planner's three options synthesized):**
- The bash-fallback must NOT silently masquerade as "real reposix subprocess." Either (a) tighten the catalog row text to honestly describe the fallback as a CI-portability behavior AND update the `claim_vs_assertion_audit` to match, OR (b) require `cargo run -p reposix-cli -- --version` as the worked-example invocation and redefine "no-cargo" as "no full-workspace cargo, single-crate worked-example permitted." Option (a) preserves the no-cargo discipline; option (b) preserves the kind:shell-subprocess proof shape. **Pick one explicitly in the replan.**
- 89-PLAN-OVERVIEW + 89-08-PLAN must drop the literal "no cargo, no crates" statement — replace with "no full-workspace cargo; targeted `crates/` edits permitted (allowlist comments in 89-02 + worked-example invocation if option (b))."
- 89-02's allowlist-marker addition must be explicitly named in PLAN-OVERVIEW's "Files touched" so the no-crates claim doesn't surface elsewhere.

#### HIGH — Deferral-pointer linter silent-pass when phase number absent (Gemini HIGH; Claude/Codex didn't surface)

89-05's `quality/gates/structure/deferral-pointer-linter.sh` greps for the pattern then extracts phase numbers via a separate `grep -oE 'P[0-9]+'`. If a developer writes `// substrate-gap-deferred` with NO phase number, the inner extraction returns empty, the cross-reference loop never executes, and the linter exits 0 — silently. This defeats the gate's purpose. **Fix:** assert that ≥1 phase number was extracted whenever the deferral pattern matches; otherwise emit an explicit error and BLOCK with stderr citing the missing PNN suffix. (Three of the three deferral patterns in scope DO require a `P\d+` suffix, but the script must enforce it rather than silently skip.)

#### MEDIUM — SLOT verifier exit-code → status overwrite risk in 89-06 (Codex + Gemini converge)

When env is set and 89-06's SLOT verifier writes `"status": "NOT-VERIFIED"` and exits 1, the runner may overwrite the artifact status to FAIL based on the non-zero exit code. This destroys the honest NOT-VERIFIED signal. Two mitigation paths:
- (a) Runner respects pre-existing `NOT-VERIFIED` status in verifier-written artifacts (modify `run.py:259-283` synthesis logic — but this is in 89-04's scope).
- (b) SLOT verifier exits 75 (NOT-VERIFIED convention) instead of 1; runner already has a reserved exit code mapping for this in `run.py:276-283`.

Per RESEARCH § Q-RUNNER-CROSS-CHECK, the runner currently maps only `0/2/timeout/else` — so (a) requires runner change, (b) requires adding exit-75 → NOT-VERIFIED to `_realbackend.py` semantics. **Recommend (b)** because it's narrower and the SLOT pattern is a P89-specific need; runner-wide artifact-status preservation is a P90 concern (RBF-FW-07 / F-K4b territory).

#### MEDIUM — Python import context in run.py (Gemini-only HIGH-correctness; Codex/Claude missed)

The plan implies relative imports `from . import _audit_field` / `from . import _realbackend` but `run.py` is invoked as a script (`python3 quality/runners/run.py`), not as a module. Relative imports will crash with `ImportError: attempted relative import with no known parent package`. This is a one-character class of fix (`from . import X` → `import X`) but if missed, the new cadence + audit-field rows will fail to load entirely, taking the entire pre-push runner with them.

#### MEDIUM — Banned-token regex semantic gap (Codex-only)

The tightened `\bP\d{2,3}-\d+\b` excludes legitimate `P1-1` audit IDs but ALSO would miss future legitimate user-facing leaks like `"P1-1 not implemented"` if such a phrase appeared in stderr. The invariant the regex is meant to enforce is "no internal phase IDs in production user-facing strings" — the fix is either (a) keep the tighter regex and accept the trade-off as documented (P89's regex catches v0.13+ phase numbers), or (b) maintain an explicit allowlist of legitimate `P\d-\d` codes. **Recommend (a) plus document the trade-off in 89-02 step 1.5 explicitly:** "v0.8/v0.9-era P1-X audit IDs are NOT scope for this gate; future audit-ID conventions should adopt `P\d{2,3}-` numbering or use a different prefix to avoid the framework banning them."

### Divergent Views (worth investigating)

- **Risk overall:** Gemini LOW, Claude LOW–MEDIUM, Codex MEDIUM. Convergent on "execution-ready with revisions"; divergent on the magnitude. The HIGH-from-Gemini (deferral-linter false-negative) was missed by the other two; the HIGH-from-Codex (no-cargo overstatement) was downgraded to MEDIUM by Claude. Both are real and replan-worthy.
- **`shell-subprocess` enforcement strength:** Codex wants a P89-side mechanical check that `kind:shell-subprocess` rows have `transcript_path`; Claude/Gemini are content to defer to P90's RBF-FW-07 (catalog-row honesty rules). **Recommend folding Codex's enforcement into 89-07's `_audit_field.py` validator** — it's load-time anyway and the cost is one extra dict lookup. Documented as RBF-FW-11 scope expansion (still RBF-FW-11; the field-presence check is the spec of "claim_vs_assertion_audit" applied to the kind=shell-subprocess sub-class).
- **SLOT exit-code handling:** Codex says "needs a targeted test"; Gemini suggests `run.py` change OR exit-code change; Claude is silent. Resolved above (recommend exit-75 in `_realbackend.py`).
- **`parse_rfc3339` format:** Claude + Gemini both flag potential `Z` vs. `+00:00` mismatch but neither verified the actual parser. Recommend the simple fix: change `CUTOFF_ISO` to `2026-05-08T00:00:00Z` regardless (more compatible with strict parsers).

### Plan-checker recap

The in-house plan-checker graded PASS with three LOW recommendations (one of which — REC-3 cwd typo — was independently confirmed by Gemini and Claude as the worked-example `../..` → `../../..` bug). The plan-checker missed the two HIGH-severity findings above. This is the chicken-and-egg risk operationalized: the in-house checker shares the planner's blind spots.

### Recommendation

**Re-invoke `gsd-plan-phase 89 --reviews`** to fold the convergent findings into a revised plan set. Specifically address:

1. **HIGH (89-04 + 89-PLAN-OVERVIEW + 89-08):** Pick option (a) or (b) for the worked-example bash-fallback; update the no-cargo claim across all three documents accordingly; if (a), tighten the catalog row's `expected.asserts` and `claim_vs_assertion_audit` to honestly describe the fallback.
2. **HIGH (89-05):** Fix the deferral-pointer linter to BLOCK when a deferral pattern matches but no phase number is extracted. Add a regression test (existing-match smoke + a synthetic no-PNN case).
3. **MEDIUM (89-06):** SLOT verifier exits 75 (not 1); add exit-75 → NOT-VERIFIED mapping to `_realbackend.py`. Add a targeted runner test that 89-06's SLOT row produces NOT-VERIFIED status when env is set + substrate absent.
4. **MEDIUM (89-03 + 89-07):** Use absolute imports (`import _realbackend`, `import _audit_field`) in run.py. Add a smoke step that imports the new modules under the actual `python3 quality/runners/run.py --help` invocation as a regression test.
5. **MEDIUM (89-02):** Document the `\bP\d{2,3}-\d+\b` trade-off explicitly as step 1.5 — enumerate what the gate catches (v0.13+ tokens) and what it intentionally misses (v0.8/v0.9 audit IDs).
6. **MEDIUM (89-07 expansion of RBF-FW-11):** `_audit_field.py` validator additionally checks that `kind: shell-subprocess` rows have non-empty `expected.artifact.transcript_path` (or equivalent). This is the spec applied — same load-time SystemExit shape.
7. **LOW (89-04):** Fix `../..` → `../../..` cwd resolution in `shell-subprocess-example.sh`.
8. **LOW (89-07):** Change `CUTOFF_ISO` to use `Z` suffix.
9. **LOW (89-02 + 89-08 cross-link):** Append SURPRISES-INTAKE pointer that P91 RBF-A-03 must scrub the `// banned-words: ok` allowlist comments when it removes the deferral strings.

After replan, the in-house plan-checker re-runs against the revised PLAN files. If PASS again, proceed to execution (top-level orchestration of P89). The cross-AI loop does NOT need to re-run unless the replan introduces new architectural changes (it shouldn't — these are surgical fixes inside existing tasks).

**Convergence rule:** No HIGH concerns remaining → ready for execution. Both HIGH findings above must be addressed before the next milestone in this loop.
