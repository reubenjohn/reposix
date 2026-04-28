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

### P63 -- POLISH-CODE final wiring (added 2026-04-28)

POLISH-CODE closes at P63: the two `code/cargo-*` rows graduate from
P58-stub documentation-of-existing into final wiring.

- `code/cargo-fmt-clean` flips its verifier from the `ci-job-status.sh`
  wrapper to `quality/gates/code/cargo-fmt-clean.sh` (Wave 3 ships the
  script). The new verifier directly invokes `cargo fmt --all -- --check`
  -- read-only, ~5s, safe under the CLAUDE.md "Build memory budget"
  ONE cargo at a time rule (no compile). Waiver dropped; row enforces
  locally as well as in CI.
- `code/cargo-test-pass` keeps the `ci-job-status.sh` wrapper as
  canonical. Running `cargo nextest run --workspace` from the verifier
  consumes 6-15 minutes (memory-budget rule violation per CLAUDE.md +
  exceeds the pre-pr 10-minute cadence cap). CI is the canonical
  enforcement venue; the waiver `tracked_in` flips to `v0.12.1
  MIGRATE-03 -- per-row local cargo enforcement` so the v0.12.1
  implementer can explore per-crate alternatives, sccache-warmed
  CI-only verifier, or accepting CI as the only home.

Cross-reference: `.planning/REQUIREMENTS.md` § POLISH-CODE.

## Cross-references

- `quality/catalogs/code.json` — rows backed by these verifiers.
- `quality/PROTOCOL.md` — runner + verdict contract.
- `.github/workflows/ci.yml` — the documentation-of-existing target.
