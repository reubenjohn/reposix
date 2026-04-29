# Phase 72: Lint-config invariants — bind 9 README/contributing.md claims (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution
**Mode:** `--auto` (sequential gsd-executor on main; depth-1)
**Milestone:** v0.12.1
**Estimated effort:** 3.5 hours wall-clock (9 verifiers × 15-25 min each)

<domain>
## Phase Boundary

Bind 9 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` that assert workspace-level lint or configuration invariants. Each row binds to a single-purpose shell verifier under `quality/gates/code/lint-invariants/`. The walker hashes both the source prose range AND the verifier file body (P71 schema 2.0 `--test <path>` mode); drift on either fires `STALE_DOCS_DRIFT` and the next agent reviews.

Concretizes the principle "lint-config rows ARE testable" — historically these rows lived in MISSING_TEST as 'we-know-it's-true-but-no-test-binds-it' and the walker had no way to detect drift if e.g. someone removed `forbid(unsafe_code)` from a single crate.

**Explicitly NOT in scope:**
- Closing connector-contract rows (P73).
- Narrative cleanup or UX bindings (P74).
- The bind-verb hash-overwrite fix (P75 — that fix lands AFTER P72-P74 because the bind-verb behavior matters for P75's regression test).
- Adding clippy lints not already in CLAUDE.md / per-crate `#![warn]`.

</domain>

<decisions>
## Implementation Decisions

### D-01: One verifier file per claim (with one shared exception)
9 rows → 8 verifier files. The two `forbid(unsafe_code)` rows (`README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate`) bind to the SAME verifier (`forbid-unsafe-code.sh`) — single source of truth. Other rows are 1:1.

### D-02: Verifier home is `quality/gates/code/lint-invariants/`
Mirrors the `quality/gates/<dimension>/<sub-area>/` pattern from P58/P60. The `lint-invariants` subdir scopes to "verifies a workspace-level Rust/Cargo invariant," distinct from `quality/gates/code/{clippy,fmt,nextest}` which already exist.

### D-03: Shell verifiers, not Python
Bash + standard Unix tools (`grep`, `find`, `cargo`). Rationale: lint-config invariants are mostly textual (regex grep over Cargo.toml / lib.rs / contributing.md). Python adds a runtime dep without buying expressiveness here. The existing `cargo-check-workspace-available` and `tests-green` verifiers DO call cargo (one invocation each, sequentially).

### D-04: Cargo invocations are compile-only and serialized
`cargo check --workspace -q` and `cargo test --workspace --no-run` only — no full test runs. One verifier at a time per CLAUDE.md `Build memory budget`. The walker invokes verifiers serially by default (`quality/runners/run.py` doesn't parallelize), so this is satisfied automatically.

### D-05: `tests-green` verifier compiles, doesn't run
Running the full test suite would inflate pre-push from ~5s to ~3min. The claim "tests are green" is verified by the existing pre-push `cargo nextest` row at `quality/gates/code/`; the docs-alignment binding is a CHEAP compile-only signal that "the workspace test target compiles." If the workspace doesn't compile, every other lint-config check will fail too.

### D-06: `cargo-test-133-tests` re-measures and updates prose
The "133 tests" number in `docs/development/contributing.md:20` is stale (current count is unknown without measuring; v0.11.x added tests). Verifier: count test binaries via `cargo test --workspace --no-run --message-format=json`, parse `compiler-artifact` events with `target.test == true`. PROSE: update contributing.md to current count BEFORE binding (don't bind to a stale number; otherwise STALE_DOCS_DRIFT fires immediately). Use a `>= N` floor in the verifier so test additions don't break it; only test deletions trigger BLOCK.

### D-07: `errors-doc-section-required` uses clippy lint, not grep
Clippy already has `missing_errors_doc` lint (in `clippy::pedantic`). Verifier: `cargo clippy --workspace -- -W clippy::missing_errors_doc -D warnings 2>&1 | grep -c missing_errors_doc` — assert 0. Heavier than grep but more correct (handles `Result<_, _>` vs `Result<T>` aliases vs trait methods).

### D-08: Verifier subagent dispatch — Path A preferred
P72 is one of the first phases of this autonomous run; the orchestrator IS the top-level coordinator and HAS `Task`. Dispatch `gsd-verifier` via `Task` with the standard QG-06 prompt template. Verdict at `quality/reports/verdicts/p72/VERDICT.md`. No Path B fallback.

### D-09: Eager-resolution of in-flight surprises
If a verifier reveals an actual lint-config violation in the workspace (e.g. a crate missing `forbid(unsafe_code)`), fix it inside this phase IF the fix is < 1 hour and < 5 files touched. Otherwise append to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` per CLAUDE.md OP-8 (+2 reservation practice) and let P76 resolve.

### D-10: CLAUDE.md update is mandatory at phase close
Per QG-07 — every phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. Add a P72 H3 subsection ≤30 lines under "v0.12.1 — in flight" naming the verifier home + the 9 rows it binds + the prose-update note for cargo-test count. Banned-words check (`scripts/check-banned-words.sh` or via the docs-build dimension) MUST pass.

### Cargo memory budget (load-bearing)
- One cargo invocation at a time. Each verifier is a separate process invocation; `quality/runners/run.py` runs serially.
- Per-crate cargo NOT applicable here (these verifiers test the workspace as a whole).
- The `tests-green` verifier (`cargo test --workspace --no-run`) is the heaviest; 30-60s on this VM. Schedule it last so other verifiers' faster failures surface first.

</decisions>

<canonical_refs>
## Canonical References

- `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint" + "Tools fail loud, structured, agent-resolvable" — every verifier follows these two principles.
- `quality/catalogs/README.md` § "docs-alignment dimension" — schema for bind invocations.
- `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` — the binary that mints BOUND state.
- `quality/gates/docs-alignment/hash_test_fn` — Rust function-body hasher (NOT used here; we use file-body sha256 for shell verifiers).
- `quality/gates/code/{clippy.sh, fmt.sh, nextest.sh}` — existing dimension siblings; pattern-match their structure.
- `CLAUDE.md` § "Build memory budget" — the load-bearing reason cargo invocations are sequential.
- `CLAUDE.md` § "v0.12.1 — in flight" — where the P72 H3 subsection appends.

</canonical_refs>

<specifics>
## Specific Ideas

- The 9 catalog rows are listed verbatim in `.planning/milestones/v0.12.1-phases/ROADMAP.md` § Phase 72.
- For `forbid(unsafe_code)` walker: simplest impl is `find crates -path '*/src/lib.rs' -o -path '*/src/main.rs' | xargs grep -L 'forbid(unsafe_code)'`. Empty output = PASS. Wrap in a script that prints which file(s) lack the attribute on FAIL (agent-resolvable per Principle B).
- For `cargo-test-count`: the count to measure is "test binaries" not "individual `#[test]` fns". Count is much smaller and stabler. Use `cargo metadata` or parse cargo's JSON output.
- For `errors-doc-section-required`: `cargo clippy --workspace --message-format=json -- -W clippy::missing_errors_doc 2>/dev/null | jq 'select(.reason=="compiler-message" and .message.code.code=="clippy::missing_errors_doc") | .message.spans[0]'` — count results; assert 0.
- After binding, run `target/release/reposix-quality doc-alignment refresh README.md docs/development/contributing.md` to flip rows MISSING_TEST→BOUND.
- Phase verdict measurement: capture `target/release/reposix-quality doc-alignment status` BEFORE and AFTER; report alignment_ratio delta in the verdict.

</specifics>

<deferred>
## Deferred Ideas

- Workspace-wide lint enforcement via a single `cargo lint-everything` script: nice ergonomic, but each row still needs its OWN verifier so the walker can isolate drift signal. Don't compress.
- Pre-commit hook that runs the lint-invariants verifiers locally: P77 candidate.
- Bumping MSRV from 1.82 → 1.85 (closes the `block-buffer` cargo install issue): that's MSRV-01 in P70, not P72.

</deferred>

---

*Phase: 72-lint-config-invariants*
*Context gathered: 2026-04-29*
*Source: HANDOVER-v0.12.1.md § 3a + autonomous-run prep.*
