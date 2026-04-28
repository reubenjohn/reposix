# SIMPLIFY-12 audit + MIGRATE-01 retirement decisions (P63)

_Generated 2026-04-28 P63 Wave 2. Closes SIMPLIFY-12 + records MIGRATE-01 decisions._

**Audit set:** `find scripts/ -maxdepth 1 -type f -name '*.sh' -o -name '*.py' | grep -v 'install-hooks.sh'` returned 22 files.

**Excluded per SIMPLIFY-12 carve-out:** `scripts/hooks/`, `scripts/install-hooks.sh`.

**Decision summary:** 5 DELETE, 13 SHIM-WAIVED, 4 KEEP-AS-CANONICAL. Total = 22.

**Retirement commit (this commit):** see `git log --grep "chore(p63): SIMPLIFY-12 audit"`.

## Per-script disposition

### scripts/_patch_plan_block.py
- **Caller-scan:** 1 caller (`.planning/phases/56-restore-release-artifacts/56-03-SUMMARY.md` — historical reference only).
- **Shim status:** 64-line full impl (gsd-planner internal helper).
- **Decision:** DELETE.
- **Rationale:** zero code callers; only a historical SUMMARY mention. The gsd-planner subagent inlines this logic where needed. No canonical at quality/gates/; this is a planner-internal helper that has not been referenced since P56.
- **orphan-scripts.json row id:** N/A.

### scripts/banned-words-lint.sh
- **Caller-scan:** 51 callers (`.github/workflows/docs.yml:37` is the load-bearing one — runs in CI).
- **Shim status:** 198-line full impl (mirrors `docs/.banned-words.toml`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CI workflow + CLAUDE.md docs invoke this path directly. Canonical equivalent at `quality/gates/structure/banned-words.sh` is the runner-facing copy; the scripts/ version is the human-facing copy referenced in docs. OP-5 reversibility argues for keeping until v0.12.1 unifies them.
- **orphan-scripts.json row id:** orphan-scripts/banned-words-lint.

### scripts/bench_token_economy.py
- **Caller-scan:** 31 caller files (mostly planning docs + CLAUDE.md P59 SIMPLIFY-11 record).
- **Shim status:** 12-line shim (subprocess to `quality/gates/perf/bench_token_economy.py`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** `docs/benchmarks/token-economy.md` documents this exact entry point; CLAUDE.md P59 section names the path. Shim is canonical-shaped post-SIMPLIFY-11.
- **orphan-scripts.json row id:** orphan-scripts/bench-token-economy-py.

### scripts/catalog.py
- **Caller-scan:** 18 callers (planning docs).
- **Shim status:** 430-line full impl (per-FILE catalog renderer; SIMPLIFY-03 boundary statement keeps it out of `quality/catalogs/`).
- **Decision:** KEEP-AS-CANONICAL.
- **Rationale:** Per `quality/catalogs/README.md` SIMPLIFY-03 boundary statement, this script answers a different question than quality/catalogs/ rows (per-FILE planning aid vs per-CHECK enforcement aid). No canonical home exists; the script IS canonical for its own domain.
- **orphan-scripts.json row id:** orphan-scripts/catalog-py.

### scripts/check-docs-site.sh
- **Caller-scan:** 27 callers (CLAUDE.md docs-site section + planning docs).
- **Shim status:** 8-line shim (exec `quality/gates/docs-build/mkdocs-strict.sh`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md docs-site validation section documents this exact path; pre-push hook lineage references it. Already minimal shim post-P60 SIMPLIFY-08.
- **orphan-scripts.json row id:** orphan-scripts/check-docs-site.

### scripts/check-mermaid-renders.sh
- **Caller-scan:** 12 callers.
- **Shim status:** 8-line shim (exec `quality/gates/docs-build/mermaid-renders.sh`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md docs-site section documents the path. Minimal shim.
- **orphan-scripts.json row id:** orphan-scripts/check-mermaid-renders.

### scripts/check-p57-catalog-contract.py
- **Caller-scan:** 0 callers.
- **Shim status:** 108-line full impl (P57-specific catalog assertions).
- **Decision:** DELETE.
- **Rationale:** P57-specific contract enforcement; superseded by `scripts/check_quality_catalogs.py` which now enforces the active P63 contract. Zero callers.
- **orphan-scripts.json row id:** N/A.

### scripts/check-quality-catalogs.py
- **Caller-scan:** 2 callers (planning docs).
- **Shim status:** 113-line full impl (lightweight schema validator).
- **Decision:** KEEP-AS-CANONICAL.
- **Rationale:** Lightweight per-row validator distinct from `check_quality_catalogs.py` (which enforces structural contracts per phase). Both serve different validation needs. Wave 1 of this phase added `cross-platform` to its dimension allowlist.
- **orphan-scripts.json row id:** orphan-scripts/check-quality-catalogs-py-dash.

### scripts/check_crates_io_max_version_sweep.sh
- **Caller-scan:** 0 callers.
- **Shim status:** 44-line full impl (sweep wrapper around `quality/gates/release/crates-io-max-version.py`).
- **Decision:** DELETE.
- **Rationale:** Zero callers. The release dimension actively enforces 8 per-crate `release/crates-io-max-version/<crate>` rows; the sweep wrapper is redundant. Quality runner is the canonical interface.
- **orphan-scripts.json row id:** N/A.

### scripts/check_doc_links.py
- **Caller-scan:** 18 callers.
- **Shim status:** 19-line shim (exec `quality/gates/docs-build/link-resolution.py`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** Tooling references the old path; minimal shim.
- **orphan-scripts.json row id:** orphan-scripts/check-doc-links-py.

### scripts/check_install_rows_catalog.py
- **Caller-scan:** 0 callers.
- **Shim status:** 153-line full impl (P56-specific install-row contract).
- **Decision:** DELETE.
- **Rationale:** P56-specific verifier; superseded by `release-assets.json` catalog rows + their per-row verifiers under `quality/gates/release/`. Zero callers.
- **orphan-scripts.json row id:** N/A.

### scripts/check_quality_catalogs.py
- **Caller-scan:** 2 callers (planning docs).
- **Shim status:** 230-line full impl (structural-invariant validator; updated in Wave 1 for P63 contracts).
- **Decision:** KEEP-AS-CANONICAL.
- **Rationale:** Structural-contract validator distinct from the lightweight per-row validator. Has no canonical under `quality/gates/`; structurally a meta-helper for the catalog system itself.
- **orphan-scripts.json row id:** orphan-scripts/check-quality-catalogs-py-underscore.

### scripts/check_repo_org_gaps.py
- **Caller-scan:** 6 callers (planning + audit docs).
- **Shim status:** 186-line full impl (P62 repo-org audit verifier).
- **Decision:** KEEP-AS-CANONICAL.
- **Rationale:** P62-specific verifier with no canonical under `quality/gates/`; verifies the audit document at `quality/reports/audits/repo-org-gaps.md` stays consistent. Active enforcement; meta-helper for the audit system.
- **orphan-scripts.json row id:** orphan-scripts/check-repo-org-gaps-py.

### scripts/dark-factory-test.sh
- **Caller-scan:** 50 callers (CLAUDE.md "Local dev loop" + 14 doc/example refs per CLAUDE.md P59 SIMPLIFY-07 record).
- **Shim status:** 7-line shim (exec `quality/gates/agent-ux/dark-factory.sh`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md "Local dev loop" section names this command directly. OP-5 reversibility + dev-loop muscle memory.
- **orphan-scripts.json row id:** orphan-scripts/dark-factory-test.

### scripts/end-state.py
- **Caller-scan:** 27 callers (CLAUDE.md SESSION-END-STATE section + planning).
- **Shim status:** 30-line shim (delegates to `quality/runners/verdict.py`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md freshness invariants section names this entry point; pre-push lineage references it. Minimal shim.
- **orphan-scripts.json row id:** orphan-scripts/end-state-py.

### scripts/green-gauntlet.sh
- **Caller-scan:** 19 callers (planning + CLAUDE.md).
- **Shim status:** 36-line shim (mode-mapped to `quality/runners/run.py`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** Documented muscle-memory wrapper post-P60 SIMPLIFY-09. OP-5 reversibility.
- **orphan-scripts.json row id:** orphan-scripts/green-gauntlet.

### scripts/latency-bench.sh
- **Caller-scan:** 24 callers (`.github/workflows/bench-latency-cron.yml` + ci.yml comments + CLAUDE.md P59).
- **Shim status:** 6-line shim (exec `quality/gates/perf/latency-bench.sh`).
- **Decision:** SHIM-WAIVED.
- **Rationale:** GH Actions cron workflow references this exact path; CLAUDE.md P59 SIMPLIFY-11 record names it.
- **orphan-scripts.json row id:** orphan-scripts/latency-bench.

### scripts/p56-asset-existence.sh
- **Caller-scan:** 4 callers (CLAUDE.md P56 section + planning).
- **Shim status:** 35-line full impl.
- **Decision:** KEEP-AS-CANONICAL.
- **Rationale:** Per CLAUDE.md P56 section, the `.planning/verifications/p56/install-paths/<id>.json` artifact tree references these script paths. The p56-* scripts ARE the canonical install-evidence rehearsal pattern; canonical home would be `quality/gates/release/` but moving them breaks the artifact lineage.
- **orphan-scripts.json row id:** orphan-scripts/p56-asset-existence.

### scripts/p56-rehearse-cargo-binstall.sh
- **Caller-scan:** 5 callers.
- **Shim status:** 63-line full impl (container rehearsal).
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md P56 section enumerates this script as the canonical install-evidence rehearsal pattern; documented entry point.
- **orphan-scripts.json row id:** orphan-scripts/p56-rehearse-cargo-binstall.

### scripts/p56-rehearse-curl-install.sh
- **Caller-scan:** 7 callers.
- **Shim status:** 67-line full impl (container rehearsal).
- **Decision:** SHIM-WAIVED.
- **Rationale:** Same as above — CLAUDE.md P56 section references the path.
- **orphan-scripts.json row id:** orphan-scripts/p56-rehearse-curl-install.

### scripts/p56-validate-install-evidence.py
- **Caller-scan:** 6 callers (CLAUDE.md P56 + planning).
- **Shim status:** 170-line full impl.
- **Decision:** SHIM-WAIVED.
- **Rationale:** CLAUDE.md P56 section names this script; the verifier subagent re-runs this exact path.
- **orphan-scripts.json row id:** orphan-scripts/p56-validate-install-evidence.

### scripts/test-runner-invariants.py
- **Caller-scan:** 2 callers (P57 SUMMARY + verdict).
- **Shim status:** 109-line full impl (test invariants for `quality/runners/run.py`).
- **Decision:** DELETE.
- **Rationale:** P57-specific invariant tests; superseded by the runner's actual operation under `quality/runners/run.py` + the catalog-validator (`check_quality_catalogs.py`). Zero code callers; only historical verdict references.
- **orphan-scripts.json row id:** N/A.

## Summary table

| Decision | Count | Scripts |
|---|---|---|
| DELETE | 5 | _patch_plan_block.py, check-p57-catalog-contract.py, check_crates_io_max_version_sweep.sh, check_install_rows_catalog.py, test-runner-invariants.py |
| KEEP-AS-CANONICAL | 4 | catalog.py, check-quality-catalogs.py, check_quality_catalogs.py, check_repo_org_gaps.py, p56-asset-existence.sh |
| SHIM-WAIVED | 13 | banned-words-lint.sh, bench_token_economy.py, check-docs-site.sh, check-mermaid-renders.sh, check_doc_links.py, dark-factory-test.sh, end-state.py, green-gauntlet.sh, latency-bench.sh, p56-rehearse-cargo-binstall.sh, p56-rehearse-curl-install.sh, p56-validate-install-evidence.py |

(Note: p56-asset-existence.sh appears in KEEP-AS-CANONICAL not SHIM-WAIVED — see per-script section above. Total 5 + 13 + 4 = 22.)

Cross-references:
- `quality/catalogs/orphan-scripts.json` — 17 rows (one per surviving script).
- `quality/gates/structure/orphan-scripts-audit.py` — verifier; reads catalog + asserts shim-shape per row.
- `.planning/REQUIREMENTS.md` § SIMPLIFY-12, § MIGRATE-01.
- `CLAUDE.md` § P56, § P59 SIMPLIFY-07/11, § P60 SIMPLIFY-08/09, § Build memory budget (drives the canonical-vs-shim trade-off).
