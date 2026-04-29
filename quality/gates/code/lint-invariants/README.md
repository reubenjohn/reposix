<!-- quality/gates/code/lint-invariants/README.md — sub-area README (P72). -->

# `code/lint-invariants/` — workspace-level lint + config invariants

These verifiers bind the 9 README + `docs/development/contributing.md` claims
that assert workspace-wide Rust / Cargo invariants. The walker hashes the
prose AND each verifier file body (P71 schema 2.0 `--test <path>` mode);
drift on either fires `STALE_DOCS_DRIFT` or `STALE_TEST_DRIFT`.

## Verifiers (8 files, 9 rows)

| Verifier                       | Catalog row(s)                                                                                              | Requirement     |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------- | --------------- |
| `forbid-unsafe-code.sh`        | `README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate` (shared, D-01) | LINT-CONFIG-01 |
| `rust-msrv.sh`                 | `README-md/rust-1-82-requirement`                                                                           | LINT-CONFIG-02 |
| `tests-green.sh`               | `README-md/tests-green`                                                                                     | LINT-CONFIG-03 |
| `errors-doc-section.sh`        | `docs-development-contributing-md/errors-doc-section-required`                                              | LINT-CONFIG-04 |
| `rust-stable-channel.sh`       | `docs-development-contributing-md/rust-stable-no-nightly`                                                   | LINT-CONFIG-05 |
| `cargo-check-workspace.sh`     | `docs-development-contributing-md/cargo-check-workspace-available`                                          | LINT-CONFIG-06 |
| `cargo-test-count.sh`          | `docs-development-contributing-md/cargo-test-133-tests`                                                     | LINT-CONFIG-07 |
| `demo-script-exists.sh`        | `docs-development-contributing-md/demo-script-exists`                                                       | LINT-CONFIG-08 |

(`LINT-CONFIG-09` is the CLAUDE.md H3 grounding update — no shell verifier; per-row coverage lands implicitly via QG-07.)

## Memory budget (D-04, load-bearing)

CLAUDE.md "Build memory budget" forbids parallel cargo invocations on this
VM. Three of these verifiers shell out to cargo:

- `tests-green.sh` — `cargo test --workspace --no-run`
- `cargo-check-workspace.sh` — `cargo check --workspace -q`
- `cargo-test-count.sh` — `cargo test --workspace --no-run --message-format=json`
- `errors-doc-section.sh` — `cargo clippy --workspace --message-format=json`

The runner (`quality/runners/run.py`) invokes verifiers serially by default,
so the constraint is satisfied automatically. Local debugging MUST NOT run
two of these in parallel.

## Implementation language (D-03)

Bash + standard Unix tools (`grep`, `find`, `jq`, `cargo`). Lint-config
invariants are mostly textual; Python adds a runtime dep without buying
expressiveness. All verifiers use `set -euo pipefail`, name failing files /
lines in stderr (Principle B), exit non-zero on FAIL.

## Cross-references

- `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint"
- `quality/catalogs/README.md` § "docs-alignment dimension"
- CLAUDE.md "Build memory budget"
- `.planning/phases/72-lint-config-invariants/CONTEXT.md` (D-01..D-10)
