# v0.15.0 Good-to-haves / carried-forward hardening — Part 9 of 9

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## From P121 CLOSE (bookkeeping + push, 2026-07-17)

### GTH-V15-74 — ADR-009 does not explicitly lock the clap `--version` / `--help` global flag surface
- **Source:** NOTICED during the P121 close-hotfix while normalizing the ADR-009 enumeration to subcommand form. · **Severity: LOW** · STATUS: OPEN — tag docs / structure.
- **What:** ADR-009 enumerates the 16 stability-committed subcommands but treats `--version` (a clap global FLAG) vs `version` (a subcommand) inconsistently — the flag surface is neither explicitly included in nor excluded from the stability commitment. A reader cannot tell whether `reposix --version` / `reposix --help` are part of the frozen CLI contract.
- **Fix-sketch:** add a one-paragraph flag-surface decision to ADR-009 (are the clap-provided global flags `--version` / `--help` part of the stability commitment, or explicitly out of scope as clap-owned?) and rebind the affected doc-alignment row(s) in the same commit.
- **Effort:** small — one ADR paragraph + a doc-alignment rebind; no code change.

### GTH-V15-75 — doc-alignment row `reposix-init-url-shape` over-captures ADR-009 lines 35-40 (spurious future drift)
- **Source:** NOTICED during the P121 close-hotfix doc-alignment rebinding. · **Severity: LOW** · STATUS: OPEN — tag quality / doc-alignment.
- **What:** the `reposix-init-url-shape` binding hashes ADR-009 lines 35-40, which includes line 40 — the OPENING line of point 2. Because point-1's row line-span reaches into point-2's territory, a future reword of point 2's first line would spuriously trip `STALE_DOCS_DRIFT` on the point-1 row even though point 1's cited content is untouched. Line-anchored-citation sharp edge (sibling of GTH-V15-28).
- **Fix-sketch:** narrow the `reposix-init-url-shape` row's cited line span to exclude line 40 (end at line 39), so the point-1 binding no longer overlaps point-2's opening line. Small catalog rebind + a `walk` to confirm no drift.
- **Effort:** trivial (<15min) — one catalog row line-span narrowing.

### GTH-V15-76 — `exit-codes.md` does not document clap's pre-dispatch exit-code-2 usage-error layer
- **Source:** NOTICED during the P121 close-hotfix (the `exit-codes-locked` 2026-07-04 rationale already flags this); deferred from the minimal close push to avoid re-drifting that binding mid-push. · **Severity: LOW-MED** · STATUS: OPEN — tag docs / structure.
- **What:** `docs/reference/exit-codes.md`'s TL;DR documents only the CLI's own `0` (success) / `1` (error) codes. But clap emits exit code `2` for usage errors (unknown flag, missing required arg, bad subcommand) BEFORE the CLI's own dispatch runs — an undocumented third code a scripting user will hit and cannot look up. The `exit-codes-locked` binding rationale already notes this gap.
- **Fix-sketch:** add a documented code-2 (clap usage-error) section to `exit-codes.md` and rebind the `exit-codes-locked` doc-alignment row in the SAME commit (fix-twice — a later separate reword would re-drift the just-rebound row).
- **Effort:** small — one doc section + a same-commit rebind; no code change.

### GTH-V15-77 — pre-push cadence hit ~92s vs the documented ~55-60s budget (kcov shell-coverage dominated)
- **Source:** NOTICED during the P121 phase-close push (first full pre-push run on the close commits). · **Severity: LOW (health / maintenance — non-blocking, but drifting toward the documented tripwire)** · STATUS: OPEN — tag quality / cadences.
- **What:** the pre-push hook took ~92s on the P121-close push, well over the ~55-60s budget documented in root `CLAUDE.md` § Push cadence / `quality/CLAUDE.md` § Cadences. The dominant cost is `code/shell-coverage` (kcov, ~80s) — a fixed whole-repo cost (not diff-scaled) that worsens as the shell-script corpus grows (see GTH-V15-02 corpus-growth note). The aggregate is now drifting toward the non-blocking timing-budget tripwire (`260712-bgv`).
- **Fix-sketch:** profile the kcov shell-coverage run; options — (a) cache/scope kcov to changed scripts where feasible, (b) move the full kcov aggregate to a CI-only job (already present) and keep a lighter/faster coverage probe pre-push, or (c) parallelize the kcov harness within the one-cargo-invocation constraint (kcov is not cargo, so it can run alongside without tripping the mutex). Re-measure against the documented budget after.
- **Effort:** medium — profiling + a cadence-scoping decision; touches `quality/gates/code/shell-coverage.sh` and possibly `.github/workflows/ci.yml`.

## From P122 W4 (stateless-connect read-path verification, 2026-07-18)

### GTH-V15-78 — `rebase-recovery-reconciles.sh` is ~42k chars (4x the 10k `.sh` file-size ceiling)
- **Source:** NOTICED during P122 W4 while adding the stateless-connect legs — the pre-commit file-size early-warning fired (`41684 chars, limit 10000`). · **Severity: LOW (health / readability — the over-budget tier is WAIVED until 2026-08-08; non-blocking today)** · STATUS: OPEN — tag structure / quality.
- **What:** `quality/gates/agent-ux/rebase-recovery-reconciles.sh` was already well over the 10k `.sh` ceiling before this wave (three negative guards + three import scenarios + long teaching comments); W4's stateless-connect legs grew it further to ~42k. It is a single cohesive shell-subprocess verifier, so the size is legitimate, but it is a readability outlier and drags the structure/file-size drain list.
- **Fix-sketch:** extract the reusable pieces into a sourced lib (mirror the existing `quality/gates/agent-ux/lib/litmus-flow.sh` pattern) — e.g. `lib/rebase-recovery-helpers.sh` holding `init_clone`/`issue_version`/the two negative-guard blocks and a `run_drift_scenario`/`run_stateless_scenario` function the import and stateless legs both call. Keeps the driver thin and lets the shared drift/recovery pattern be reused instead of duplicated between the import and stateless legs. Re-run the gate + `bash quality/gates/structure/file-size-limits.selftest.sh` after.
- **Effort:** medium (~1-2h) — a careful refactor of a currently-green load-bearing pre-push gate; must preserve leaf-isolation (`cd`-in-same-invocation) + transcript logging exactly, so it is out of this verification lane's charter.

## From P122 close (deterministic close, 2026-07-18)

### GTH-V15-79 — cadence decision for the two P122 on-demand rows (import-parent-resolve-fails-loud, init-refuses-nested-in-shared-tree): kept on-demand, not promoted
- **Source:** NOTICED during the P122 close — both new rows sat NOT-VERIFIED at tip because `on-demand` never fires in a CI cadence; the close asked whether to promote them into pre-push/post-push (continuously verified) or keep on-demand + green them now with a filed rationale. · **Severity: LOW (health/documentation — the decision itself is the deliverable, not a defect)** · STATUS: DONE (decision made + rows greened this close) — tag quality / cadences.
- **What:** Both rows wrap a scoped `cargo test -p <crate>` invocation against real regression tests (`agent-ux/import-parent-resolve-fails-loud` → `cargo test -p reposix-remote`, ~9s warm-cache; `agent-ux/init-refuses-nested-in-shared-tree` → `cargo test -p reposix-cli -- <5 named tests>`, ~0.5s warm-cache). Rather than promote to pre-push, this close kept both at `on-demand`, matching the EXACT precedent already established by this dimension's P120 sibling rows (`agent-ux/cli-errors-teach-recovery`, `agent-ux/helper-errors-teach-recovery`, `agent-ux/rpx-codes-registry` — all also wrap scoped `cargo test -p <crate>` + real-binary CLI assertions, all on-demand) and the code-dimension precedent (`code/cargo-test-pass`'s D-CONV-1 ruling: even a `cargo test` invocation, once it competes for the ONE-cargo-at-a-time build-memory-budget mutex and the fixed ~55s pre-push wall-clock budget, belongs at on-demand/CI-only, not pre-push). Both rows were greened this close via a DIRECT scoped verifier invocation (`bash quality/gates/agent-ux/<row>.sh`, both exited 0 against reality) rather than a blanket `run.py --cadence on-demand --persist` sweep — the full on-demand cadence has 41 rows total, 39 of them unrelated to P122 and several already FAIL/NOT-VERIFIED at rest pending real backends or other phases' fixtures; a blanket re-grade at P122 close risked out-of-scope catalog churn on rows this close has no mandate to touch.
- **Fix-sketch:** none required — this is a documented decision, not a defect. If a future phase wants these two rows to self-verify continuously (e.g. because `resolve_import_parent` or the init-refusal latches become a frequent regression source), promote them to `pre-push` at that time and re-measure the pre-push wall-clock budget (`CLAUDE.md` § Push cadence tripwire) before committing to the promotion.
- **Effort:** none (decision already executed this close).

## From Phase 123 close (P114 OQ1 residual carry-forward)

### GTH-V15-80 — Confluence list-path lacks the storage-format fallback get_record has (pre-ADF pages would still oid-drift)
- **Source:** P114 `114-VERIFICATION.md` OQ1 residual (carried forward, NOT re-diagnosed by this phase) · **Severity: MEDIUM** · STATUS: OPEN.
- **What:** the LIST path requests `body-format=atlas_doc_format` with NO storage fallback, while `get_record` DOES fall back to `body-format=storage` — a pre-ADF (storage-only) Confluence page would therefore still trip the oid-drift check on checkout.
- **Fix-sketch:** add the same storage-format fallback to the list path that `get_record` already has, in the Confluence connector under `crates/reposix-core`.

## From Phase 123 close (REQUIREMENTS.md coverage-table staleness noticing)

### GTH-V15-81 — `.planning/REQUIREMENTS.md` coverage table shows `Pending` for several already-closed phases
- **Source:** NOTICED during 123-07 close-wave Task 2 (REQUIREMENTS.md DRAIN checkbox flip) · **Severity: LOW (hygiene)** · STATUS: OPEN.
- **What:** the coverage table (~L303-330) has not been kept current except for Phase 122's own DRAIN-07/08/09 flip (and now this close's DRAIN-01/03/04/05/06/10 flip) — several rows for phases that closed GREEN long ago (e.g. FIX-01/FIX-02 for the CLOSED Phase 114, DOCS-01..09 for the CLOSED Phases 117/118/119, UX-01/UX-02 for the CLOSED Phases 120/121) still read `Pending`.
- **Fix-sketch:** a dedicated drain pass reconciling every `REQ-ID` row's `Status` column against its phase's actual close state (cross-ref `.planning/ROADMAP.md`'s Progress table), not a piecemeal per-phase flip. Do not expand any single phase's close task to fix every stale row — this is its own bounded hygiene sweep.

## From Phase 123 close (Lane 1 code-review noticing, file-size early-warning)

### GTH-V15-82 — `quality/gates/structure/verifier-script-exists.selftest.sh` is 9994/10000 chars (99.9% of the `.sh` ceiling) after the SC4 tightening
- **Source:** NOTICED by 123-07 close-wave Lane 1 (code review) while closing the SC4 null-script-PASS hole (`857e3c3a`) · **Severity: LOW** · STATUS: OPEN — tag structure / readability.
- **What:** the SC4 truth-table tightening (PASS+null-script → violation; FAIL/PARTIAL+null-script → exempt) grew the selftest to 9994 of its 10000-char `.sh` ceiling — 99.9%, inside the EARLY-WARNING band and one assertion away from breaching. The `structure/file-size-limits` over-budget tier is WAIVED until 2026-08-08, so this is non-blocking today, but the next test case added to this selftest will trip the hard ceiling.
- **Fix-sketch:** split into a fixture-builder module (the throwaway `/tmp` catalog-repo construction helpers) + a slimmer assertion-only driver script, mirroring the `lib/`-extraction pattern already used elsewhere in `quality/gates/agent-ux/lib/`. Small (~30min), no new dependency.
- **Effort:** small.

### GTH-V15-83 — `quality/runners/test_run.py` is 37720 chars (2.5x the 15000 `.py` ceiling)
- **Source:** NOTICED by 123-07 close-wave Lane 2 (PLANNING-CLOSE + AUDIT), corroborating the same-file noticing already logged in `123-05-SUMMARY.md` § Noticed (never filed to this ledger until now) · **Severity: LOW** · STATUS: OPEN — tag structure / readability.
- **What:** across the 123-02/04/05/06 waves, `test_run.py` accumulated four TestCase classes (`TestPersistGate`, `TestPersistDowngradeGuard`, `TestPersistCatalogLock`, `TestEnvSelfSourcing`) in one file, now 37720 of its 15000-char `.py` ceiling (2.5x over). `quality/runners/run.py` itself is also over budget (29915 chars) and pre-existing. Both sit under the same WAIVED-until-2026-08-08 over-budget tier, so non-blocking today.
- **Fix-sketch:** split `test_run.py` along its natural TestCase seams into per-feature modules — `test_persist_gate.py` / `test_persist_downgrade.py` / `test_persist_lock.py` / `test_env_load.py` — mirroring how `_persist_guard.py`/`_env_load.py` are already split as sibling modules on the implementation side. Re-run the full suite after to confirm no cross-module fixture coupling.
- **Effort:** medium (~1h) — mechanical split, but touches every test invocation path (`python3 -m unittest quality.runners.test_run...` references across gates/CI) so needs a careful grep-and-update of all callers in the same commit.

## From Phase 124 close (container-rehearse harness hardening, 2026-07-18)

### GTH-V15-84 — `quality/gates/docs-repro/container-rehearse.sh` is 17623 chars (1.76x the 10000 `.sh` ceiling)
- **Source:** NOTICED at P124 phase-close hygiene lane — the `structure/file-size-limits` early-warning band covers it (over the 10000-char `.sh` ceiling). · **Severity: MEDIUM (health/readability — but a hard deadline, see below)** · STATUS: OPEN — tag structure / quality.
- **What:** the container-rehearse harness accreted its docker-drive path, the doctor/attach output harvest, the artifact-writer, and the `--selftest-*` docker-free modes into one 17623-char script across P124's waves. The `structure/file-size-limits` over-budget tier is WAIVED only until **2026-08-08T00:00:00Z**; after that date this file **BLOCKS pushes** (the waiver is a single global row in `quality/catalogs/freshness-invariants.json`). So unlike most GTH readability items this one carries a real hard deadline, not just drift.
- **Fix-sketch:** extract the harvest (doctor/attach output-capture) helpers and the artifact-writer into `quality/gates/docs-repro/lib/` — W2 already established this exact precedent by factoring `lib/sim-lifecycle.sh` (6634 chars) out of this same harness, so the convention and the `source`-from-`lib/` wiring already exist. Keep the driver thin (docker orchestration + assertion sequencing) and move the reusable output-parsing/artifact-JSON pieces to `lib/rehearse-harvest.sh` + `lib/rehearse-artifact.sh`. Re-run `bash quality/gates/docs-repro/container-rehearse-sigkill-safe.sh` + `bash quality/gates/structure/file-size-limits.selftest.sh` after; must preserve leaf-isolation (`cd`-in-same-invocation) + the `--selftest-*` docker-free modes exactly.
- **Effort:** medium (~1-2h) — careful refactor of a green load-bearing pre-push gate; do it BEFORE 2026-08-08 or the file starts blocking pushes.

### GTH-V15-85 — container-rehearse `--selftest-port-gate` writes a bogus-row-id verification artifact into the shared reports dir when 7878 is occupied
- **Source:** NOTICED at P124 phase-close while removing the stray `quality/reports/verifications/docs-repro/--selftest-port-gate.json` cruft (W2 sigkill-safe selftest residue). · **Severity: LOW (hygiene — the artifact is gitignored via `.gitignore:83 quality/reports/verifications/*/*.json`, never enters the repo)** · STATUS: OPEN — tag quality / cleanup.
- **What:** `container-rehearse-sigkill-safe.sh` drives `bash "$HARNESS" --selftest-port-gate` (via `lib/sim-lifecycle.sh`) to prove the harness fails loud against a planted 7878 listener. When the port IS occupied, the harness's artifact-writer emits a verification JSON to a default path derived from the literal flag as a row-id, producing a file literally named `--selftest-port-gate.json` in `quality/reports/verifications/docs-repro/`. A `--selftest-*` mode should not write a shared "verification" report at all, and certainly not one named after a CLI flag; harmless today only because the path is gitignored.
- **Fix-sketch:** in the harness, make the `--selftest-*` docker-free modes either (a) suppress the artifact-writer entirely (they are self-tests, not graded rows), or (b) redirect their artifact to a `/tmp` throwaway path. Trivial once the artifact-writer is factored into `lib/` (see GTH-V15-84) — pass the selftest an explicit `--artifact /dev/null`-equivalent or skip the write when the invocation row-id starts with `--`.
- **Effort:** trivial (<30min); best folded into the GTH-V15-84 `lib/` extraction.
