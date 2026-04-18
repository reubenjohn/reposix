---
plan: 27-03
phase: 27
title: "Add Issue.extensions field + ADR-004 + v0.8.0 bump + CHANGELOG"
status: complete
completed_at: "2026-04-16"
---

# 27-03 Summary — Issue.extensions + ADR-004 + v0.8.0

## What was done

### Task 1: Issue.extensions field

Added `extensions: BTreeMap<String, serde_yaml::Value>` to the `Issue` struct in
`crates/reposix-core/src/issue.rs` with `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`.

Mirrored the field in the private `frontmatter::Frontmatter` struct so render/parse
round-trips correctly. Updated `render()` to copy and `parse()` to restore the field.

Propagated the new required field to all Issue initializers across the workspace:
- `crates/reposix-core/src/backend/sim.rs` (4 sites)
- `crates/reposix-core/src/taint.rs` (sanitize destructure + struct init + test helper)
- `crates/reposix-confluence/src/lib.rs` (translate() + make_untainted test helper)
- `crates/reposix-confluence/tests/roundtrip.rs`
- `crates/reposix-github/src/lib.rs` (gh_to_issue + 2 test helpers)
- `crates/reposix-fuse/src/fs.rs` (placeholder + sample_untainted + mk_issue + 3 inline)
- `crates/reposix-fuse/src/labels.rs` (make_issue test helper)
- `crates/reposix-fuse/src/tree.rs` (mk_issue test helper)
- `crates/reposix-swarm/src/sim_direct.rs`
- `crates/reposix-remote/src/diff.rs` (sample test helper)
- `crates/reposix-remote/tests/bulk_delete_cap.rs`
- `crates/reposix-cli/tests/refresh_integration.rs`
- `crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs`
- `crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs`

Updated trybuild `.stderr` snapshots via `TRYBUILD=overwrite`.

Three new unit tests in `crates/reposix-core/src/issue.rs`:
- `extensions_empty_omitted_from_yaml` — empty map not present in YAML
- `extensions_roundtrip` — non-empty map survives render→parse cycle
- `extensions_defaults_to_empty_on_parse` — legacy frontmatter without key parses fine

### Task 2: ADR-004 + version bump + CHANGELOG + STATE + ROADMAP

- `docs/decisions/004-backend-connector-rename.md` — ADR-004 written documenting the
  IssueBackend → BackendConnector rename rationale with alternatives table.
- `Cargo.toml` workspace version bumped `0.7.0 → 0.8.0`; `cargo check --workspace`
  regenerated Cargo.lock.
- `CHANGELOG.md` — `[v0.8.0] — 2026-04-16` section added with Breaking + Added entries;
  footer comparison links updated.
- `STATE.md` — cursor updated to Phase 27 SHIPPED, status set to complete.
- `ROADMAP.md` — Phase 27 plans list updated: 27-03 marked `[x]`, count to 3/3.

## Verification results

- `cargo test --workspace` — all test results ok (pre-existing reqwest crate-not-found
  in 3 doc-test bins is unrelated to this change, confirmed by baseline check)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `grep -rn "IssueBackend" crates/ --include="*.rs"` — 0 matches
- `grep "extensions: BTreeMap" crates/reposix-core/src/issue.rs | wc -l` — 2 (Issue + Frontmatter)
- `grep "skip_serializing_if.*BTreeMap::is_empty" crates/reposix-core/src/issue.rs | wc -l` — 2
- `test -f docs/decisions/004-backend-connector-rename.md` — EXISTS
- `grep 'version = "0.8.0"' Cargo.toml` — match
- `grep '\[v0.8.0\]' CHANGELOG.md` — match
