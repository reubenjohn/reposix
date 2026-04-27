# quality/gates/code/ — code-dimension verifiers

Verifiers backing the catalog rows in `quality/catalogs/code.json`.

## Verifiers

| Script | Catalog rows | Cadence |
|---|---|---|
| `clippy-lint-loaded.sh` | code/clippy-lint-loaded | pre-push |
| `ci-job-status.sh` | code/cargo-test-pass, code/cargo-fmt-clean | pre-pr |
| `check-fixtures.py` | code/fixtures-valid | pre-push |

## Conventions

- Bash for thin gh-CLI wrappers; Python for stdlib-rich validation.
- Exit 0 = PASS, 1 = FAIL (per `quality/runners/run.py`).
- Artifacts at `quality/reports/verifications/code/<row-slug>.json`.

## SIMPLIFY-04 absorption (P58 Wave C)

`scripts/check_clippy_lint_loaded.sh` migrated to
`quality/gates/code/clippy-lint-loaded.sh` (canonical home). Old path
**deleted** — no script-level callers (caller analysis: zero matches
in `*.sh`/`*.yml`/`*.toml`; doc references only).

## SIMPLIFY-05 absorption (P58 Wave C)

See
`.planning/phases/58-release-dimension-gates-code-absorption/58-03-SIMPLIFY-05-AUDIT.md`
for the audit decision (Option A: code-dim gate). Old path
`scripts/check_fixtures.py` **deleted**.

## POLISH-CODE P58-stub

`code/cargo-test-pass` and `code/cargo-fmt-clean` rows are
documentation-of-existing — they reference ci.yml's `test` and
`rustfmt` jobs. Full per-row enforcement (running cargo test + cargo
fmt locally from the verifier) is P63 final per POLISH-CODE
traceability.

## Cross-references

- `quality/catalogs/code.json` — rows backed by these verifiers.
- `quality/PROTOCOL.md` — runner + verdict contract.
- `.github/workflows/ci.yml` — the documentation-of-existing target.
