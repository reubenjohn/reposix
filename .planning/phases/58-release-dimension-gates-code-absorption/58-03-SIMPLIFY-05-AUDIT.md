# SIMPLIFY-05 audit — scripts/check_fixtures.py

## Decision: Option A (code-dim gate)

Migrated to `quality/gates/code/check-fixtures.py`. New catalog row
`code/fixtures-valid` added to `quality/catalogs/code.json` (P2,
pre-push). Old path `scripts/check_fixtures.py`: **DELETED** (no
script-level callers; only documentation references).

## Rationale

- 185 lines of Python validating shape + size + no-secrets across
  3 fixture files. Re-implementing in Rust (Option B) would triple
  the code volume and split the test surface across two languages.
- Pre-push cadence matches the script's current dev-loop invocation
  pattern.
- The framework's intended pattern is one verifier file per
  `quality/gates/<dim>/` + one catalog row.

## Caller analysis

`grep -rln "check_fixtures" --include="*.sh" --include="*.yml" --include="*.toml"` returned zero matches.

`grep -rln "check_fixtures" --include="*.md"` returned only documentation
references (`.planning/research/*`, planning notes). No live shell hooks,
CI workflows, or build scripts invoke the script.

Action: **DELETED** the old path entirely. Doc references will be
swept by future doc-clarity passes (low priority — each is a
historical reference, not a current invocation).

## Modifications to the migrated file

- **FIXTURES path:** changed from cwd-relative `pathlib.Path("benchmarks/fixtures")`
  to `REPO_ROOT / "benchmarks" / "fixtures"` where `REPO_ROOT = Path(__file__).resolve().parents[3]`.
  The runner can now invoke from any working directory.
- **Artifact write:** added `write_artifact()` step that writes
  `quality/reports/verifications/code/fixtures-valid.json` with
  RFC3339 `ts`, `row_id`, `exit_code`, `asserts_passed`,
  `asserts_failed`. The runner reads this artifact to grade the row.
- **Stdout output preserved** for backward-compat interactive use
  (PASS / FAIL summary).
- **Per-check return type:** changed from `list[str]` (errors only)
  to `tuple[list[str], list[str]]` (passed, failed) so the artifact
  can list specific assertions met.

## Self-test (Wave C)

```
python3 quality/gates/code/check-fixtures.py
github_issues.json: 11579 bytes, 3 issues
confluence_pages.json: 7281 bytes, 3 pages
fixtures/README.md: 4311 bytes
All fixture checks passed.
exit=0
```

Artifact written to `quality/reports/verifications/code/fixtures-valid.json`
with 7 asserts_passed, 0 asserts_failed.

## v0.13.x reconsideration trigger

If the bench fixture format changes substantially (e.g., switch from
JSON to a binary format, or fixtures grow to require schema validation),
reconsider Option B (Rust integration test under
`crates/reposix-swarm/tests/fixture_shape.rs`). The validation logic
would simplify in Rust if a `serde_json::Value` or typed serde struct
can replace the manual key-set checks.
