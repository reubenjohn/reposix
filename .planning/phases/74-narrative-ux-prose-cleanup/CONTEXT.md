# Phase 74: Narrative cleanup + UX bindings + linkedin prose fix (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution
**Mode:** `--auto` (sequential gsd-executor on main; depth-1)
**Milestone:** v0.12.1
**Estimated effort:** 1.5-2 hours wall-clock (4 propose-retire + 5 verifier scripts + 1 prose edit)

<domain>
## Phase Boundary

Close the remaining 9 `MISSING_TEST` rows + 1 free-standing prose fix. Four narrative rows propose-retire (qualitative design framing — no behavioral assertion possible). Five rows bind to hash-shape verifier scripts (P71 `--test <path>` pattern); one of those is a CLI smoke test for a known-live subcommand. Plus a prose edit to drop "FUSE filesystem" framing from `docs/social/linkedin.md:21` (the existing BOUND row at that line auto-rebinds via walker after refresh).

After P72-P74 ship, the docs-alignment dimension's `claims_missing_test` count drops from 22 to 0 (within rounding — assuming the 4 narrative retires close cleanly via owner TTY confirm-retire after this autonomous run).

**Explicitly NOT in scope:**
- The 4 propose-retires here CAN be done by an agent; the `confirm-retire` step is owner-TTY only and lives in HANDOVER step 1, not in this phase.
- New end-to-end install testing (P74's `5-line-install` verifier shape-checks the snippet, not the install actually running).

</domain>

<decisions>
## Implementation Decisions

### D-01: Verifier home is `quality/gates/docs-alignment/verifiers/`
Mirrors the dimension's existing `quality/gates/docs-alignment/{walk.sh, hash_test_fn}` pattern. The `verifiers/` subdir is for shape-check verifiers minted to bind specific rows; canonical home for P74's 5 new shell verifiers + future docs-alignment shape checks.

### D-02: Hash-shape verifiers are TINY (10-30 lines)
Each verifier is a one-purpose check: grep, line count, regex match. No deep workflow logic. The point is: when prose changes, verifier file body OR source range hash drifts → walker fires STALE_DOCS_DRIFT → an agent reviews. The verifier itself doesn't need to be fancy.

### D-03: 5-line-install verifier shape-checks docs/index.md:19 line, not the actual install
`docs/index.md:19` claims "**`5-line install`** — `curl`, `brew`, `cargo binstall`, or `irm`". Verifier shape-check: assert that line contains all four channel names AND matches one specific line. The "5-line" claim is shape-checked separately in `tutorials/first-run.md` (the tutorial body should have a code block ≤5 lines for the install workflow). Two-row binding via parallel-array in P71 schema 2.0 IF we want to bind both — the simpler bind is just `docs/index.md:19` for the docs-alignment row; the tutorials/first-run.md shape stays as a P77 good-to-have.

### D-04: audit-trail-git-log verifier asserts the claim's premise via shell
The claim "The audit trail is `git log`" (docs/index.md:78) is a design-narrative claim, not a testable behavioral assertion. The verifier asserts the premise: `git log --oneline` returns ≥1 line in the repo. If git log breaks for some reason (e.g. shallow clone in CI without history), the verifier fails — that's the right behavior. NOT a deep claim; the row's value is "this prose stays referring to git log and not e.g. SDK telemetry."

### D-05: tested-three-backends verifier counts test fns
`grep -c 'fn dark_factory_real_' crates/reposix-cli/tests/agent_flow_real.rs` — assert ≥ 3. Cheap; rebind on test-fn rename.

### D-06: connector-matrix-on-landing verifier
`grep -E '^## .*[Cc]onnector' docs/index.md` AND `grep -E '^\| .* \| .* \|$' docs/index.md` — assert both have ≥1 match (heading + table row). Catches "connector matrix accidentally deleted from landing" failure mode.

### D-07: cli-spaces-smoke verifier
`target/release/reposix spaces --help 2>&1 | grep -q "List all readable Confluence spaces"` — assert exit 0. Subcommand exists at `crates/reposix-cli/src/spaces.rs:1` (verified during prep).

### D-08: Linkedin prose edit is one-line
`docs/social/linkedin.md:21` — replace the FIRST sentence "🚀 reposix — a working FUSE filesystem + git-remote-helper for issue trackers." with "🚀 reposix — a working git-native partial clone + git-remote-helper for REST issue trackers." The existing BOUND row `docs/social/linkedin/token-reduction-92pct` at line 21 will detect source_hash drift on next walk → rebind via the binary on next pass.

### D-09: Propose-retire rationales are short and identical-format
`{narrative_id} — qualitative design framing; no behavioral assertion possible. Decided 2026-04-29.` Use the same one-liner for all 4 narrative retires for consistency. Owner confirms in HANDOVER step 1.

### D-10: NO new test files in `crates/`
This phase is shell-script-and-prose only. If a binding requires a Rust test, it belongs in P73 not P74.

### D-11: CLAUDE.md update
P74 H3 subsection ≤30 lines under "v0.12.1 — in flight". List the 5 new verifier scripts + the linkedin prose change. Note that the 4 narrative rows are PROPOSE_RETIRED awaiting owner confirm.

### D-12: Eager-resolution of in-flight surprises
If `cli-spaces-smoke` reveals the `reposix spaces` subcommand has actually broken (it printed help fine in prep, but maybe `--backend confluence` is wedged), append to SURPRISES-INTAKE.md. If `connector-matrix-on-landing` reveals the matrix actually disappeared from landing, that's a P77 good-to-have to restore (or could be eagerly fixed if < 30 min — judge ROI).

</decisions>

<canonical_refs>
## Canonical References

- `quality/gates/docs-alignment/walk.sh` — pattern for sibling verifier scripts.
- `crates/reposix-cli/src/spaces.rs` — confirmed-live subcommand source.
- `crates/reposix-cli/tests/agent_flow_real.rs` — `dark_factory_real_*` test fns counted by tested-three-backends.
- `docs/index.md` — landing page; the source for 5-line-install, audit-trail-git-log, tested-three-backends, polish2-06-landing.
- `docs/social/linkedin.md:21` — the prose-fix target.
- `quality/PROTOCOL.md` — Principles A + B.

</canonical_refs>

<specifics>
## Specific Ideas

- The 10 catalog actions are listed verbatim in `.planning/milestones/v0.12.1-phases/ROADMAP.md` § Phase 74.
- After binding the 5 new verifiers, run `target/release/reposix-quality doc-alignment refresh docs/index.md .planning/milestones/v0.11.0-phases/REQUIREMENTS.md .planning/milestones/v0.8.0-phases/REQUIREMENTS.md` to flip rows.
- After the linkedin edit, run `refresh docs/social/linkedin.md` to re-hash the existing BOUND row's source_hash.
- Phase verdict measurement: capture `status` BEFORE and AFTER; expect alignment_ratio rises by ~5/(total non-retired) and claims_missing_test drops by 5 (or 9 — depends on whether the 4 narrative rows count as MISSING_TEST or RETIRE_PROPOSED at the moment of measurement).

</specifics>

<deferred>
## Deferred Ideas

- A real "5-line install snippet renders successfully" end-to-end test (would require curl + brew + cargo binstall in a container): P67 cross-platform-rehearsals or v0.13.0.
- Restoring the connector capability matrix to landing IF connector-matrix-on-landing verifier fails: P77 GOOD-TO-HAVE.
- Refactoring docs/social/linkedin.md as part of a v0.12.0 architecture sweep (the file has multiple v0.4-era references): P77 candidate or v0.13.0 docs-cleanup phase.

</deferred>

---

*Phase: 74-narrative-ux-prose-cleanup*
*Context gathered: 2026-04-29*
*Source: HANDOVER-v0.12.1.md § 3c + 3d + 3e + autonomous-run prep.*
