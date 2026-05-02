# scripts/ — verdict per file

[← index](./index.md)

| File | Verdict | Rationale |
|---|---|---|
| `scripts/banned-words-lint.sh` | **KEEP** | CI + pre-commit gate; canonical DOCS-09 enforcement. |
| `scripts/check-docs-site.sh` | **KEEP** | POLISH-17 mkdocs+mermaid HTML guardrail; ran by pre-push hook. |
| `scripts/check_clippy_lint_loaded.sh` | **KEEP** | Behavioural proof that `clippy.toml` loads; OP-6 ground-truth check. |
| `scripts/check_doc_links.py` | **KEEP** | Phase 44 audit gate; useful pre-commit. |
| `scripts/check_fixtures.py` | **KEEP** | benchmark fixture validator; harmless. |
| `scripts/dark-factory-test.sh` | **KEEP** | Central regression for the v0.9.0 architecture; CI-bound. |
| `scripts/green-gauntlet.sh` | **KEEP** | Pre-tag mega-script; recommended by CONTRIBUTING.md. |
| `scripts/install-hooks.sh` | **KEEP** | Recommended by CONTRIBUTING.md. |
| `scripts/repro-quickstart.sh` | **KEEP** | POLISH-05 reproducibility regression; CI-bound. |
| `scripts/v0.9.0-latency.sh` | **KEEP, RENAME** | Should drop the version pin (rec #8). |
| `scripts/bench_token_economy.py` + `test_bench_token_economy.py` | **KEEP** | drives `benchmarks/RESULTS.md`. |
| `scripts/take-screenshots.sh` | **DELETE** | NOT IMPLEMENTED stub (rec #9). |
| `scripts/tag-v0.3.0.sh` … `tag-v0.6.0.sh` (5 files, pre-pivot) | **ARCHIVE** | These shipped tags exist in git history; the scripts themselves are not re-runnable. Move to `.planning/archive/scripts/tag-v0.X.0.sh`. |
| `scripts/tag-v0.8.0.sh` | **ARCHIVE** | Same. |
| `scripts/tag-v0.9.0.sh` + `tag-v0.10.0.sh` | **KEEP** | Recent + the audit checklists in their headers are still relevant when authoring `tag-v0.11.0.sh`. |
| `scripts/demos/*` (11 files) | **DELETE** | Rec #1. FUSE-era. `scripts/demo.sh` shim already removed per CHANGELOG. |
| `scripts/dev/{list-confluence-spaces,probe-confluence}.sh` | **DELETE** | Rec #2. Replaced by `reposix doctor`/`reposix list`. |
| `scripts/migrations/*.py` | **ARCHIVE** | Rec #4. Move to `.planning/archive/scripts/`. |
| `scripts/hooks/{pre-push,test-pre-push.sh}` | **KEEP** | OP-7 credential-leak hook; CI-tested. |
| `scripts/__pycache__/*` | **DELETE + .gitignore-test** | Rec #3. |
