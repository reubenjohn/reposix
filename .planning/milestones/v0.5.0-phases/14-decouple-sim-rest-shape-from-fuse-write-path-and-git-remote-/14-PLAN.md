---
phase: 14
slug: decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-
goal: >
  Route every FUSE write callback and every git-remote-reposix execute-action call
  through the `IssueBackend` trait. Delete the two hardcoded sim-REST modules
  (`crates/reposix-fuse/src/fetch.rs`, `crates/reposix-remote/src/client.rs`) in the
  same sweep. No wire-shape changes exposed to users; the remote-helper URL syntax,
  mount semantics, and public CLI are unchanged. Threat model and audit log intact.
scope_tag: v0.4.1
parallelism: 2               # B1 and B2 run in parallel (disjoint filesets)
estimated_wall_clock: 2h15m  # A ~15m + max(B1, B2) ~45m + C ~30m + D ~15m
depends_on_phases:
  - 10   # read-path through IssueBackend — Phase 14 mirrors that pattern for writes
  - 13   # Wave C bucket-dir + tree overlay — unchanged, just must stay green
autonomous: true             # no human-gate inside the phase; tag-push gate is outside
requirements:
  - SC-14-01
  - SC-14-02
  - SC-14-03
  - SC-14-04
  - SC-14-05
  - SC-14-06
  - SC-14-07
  - SC-14-08
  - SC-14-09
  - SC-14-10
must_haves:
  truths:
    - "`fs.rs::release` writes through `backend.update_issue`; `fs.rs::create` writes through `backend.create_issue`."
    - "`reposix-remote/src/main.rs::execute_action` dispatches to `IssueBackend::{create_issue, update_issue, delete_or_close}`; the two `api::list_issues` sites call `backend.list_issues`."
    - "The sim's REST shape is spoken by exactly one crate (`reposix-core::backend::sim`). Every other crate is trait-only."
    - "Version-mismatch still surfaces as `libc::EIO` in the FUSE callback with a `current=N` warn log, and as `some-actions-failed` on the git-remote wire."
    - "Audit log keeps receiving a row for every write, with an `X-Reposix-Agent` that identifies the caller role (`...-fuse` vs `...-remote`)."
  artifacts:
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "write callbacks + new `update_issue_with_timeout` / `create_issue_with_timeout` helpers + version-mismatch-aware `backend_err_to_fetch`"
    - path: "crates/reposix-remote/src/main.rs"
      provides: "trait-driven `execute_action`; single `Arc<SimBackend>` on `State`; no `mod client` and no `use crate::client as api`"
    - path: "CHANGELOG.md"
      provides: "`[Unreleased]` entry under `### Changed` documenting R1 (assignee clear) and R2 (agent attribution)."
  deletions:
    - "crates/reposix-fuse/src/fetch.rs"
    - "crates/reposix-fuse/tests/write.rs"
    - "crates/reposix-remote/src/client.rs"
  key_links:
    - from: "crates/reposix-fuse/src/fs.rs::release"
      to: "IssueBackend::update_issue"
      via: "update_issue_with_timeout"
      pattern: "backend\\.update_issue\\("
    - from: "crates/reposix-fuse/src/fs.rs::create"
      to: "IssueBackend::create_issue"
      via: "create_issue_with_timeout"
      pattern: "backend\\.create_issue\\("
    - from: "crates/reposix-remote/src/main.rs::execute_action"
      to: "IssueBackend::{create,update,delete_or_close}_issue"
      via: "state.backend: Arc<SimBackend>"
      pattern: "state\\.backend\\."
---

<objective>
Phase 14 removes the last two hardcoded references to the simulator's REST shape that
live outside `crates/reposix-core::backend::sim`. After this phase the `IssueBackend`
trait is the seam every caller of write operations goes through. No user-visible
feature changes; scope is v0.4.1 (bugfix/refactor).

The phase is mechanical and disjoint — the FUSE refactor (B1) and the git-remote-helper
refactor (B2) touch different crates with no shared files, so they parallelize cleanly.
The CONTEXT and RESEARCH already locked every semantic decision (LD-14-01..08, R1..R10).
This plan just sequences the waves, names the files, and pins the verification bar.
</objective>

<context>
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-CONTEXT.md
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-RESEARCH.md
@.planning/SESSION-5-RATIONALE.md
@CLAUDE.md

# Wave plans — each executor reads only its own wave plan plus CONTEXT + RESEARCH.
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-A-core-prep.md
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-B1-fuse-write-through-backend.md
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-B2-remote-helper-through-backend.md
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-C-integration-verify.md
@.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-D-docs-changelog.md
</context>

<waves>

## Dependency graph

```
         Wave A (~15m, serial)
         core-crate prep; likely no-op but mandatory gate
              │
              ▼
   ┌──────────────────────────┐
   │                          │
Wave B1 (~45m)        Wave B2 (~45m)
reposix-fuse          reposix-remote
   │                          │
   └────────────┬─────────────┘
                ▼
         Wave C (~30m, serial)
         green-gauntlet --full + smoke + live-write-demo
                │
                ▼
         Wave D (~15m, serial)
         CHANGELOG + docs sweep
```

- **A blocks B1 and B2.** A confirms reposix-core needs no changes (or, if any shared
  helper must land, lands it first). Waiting 15 minutes to fan out is cheaper than a
  mid-B1 "oh we need a core-crate change" pivot.
- **B1 and B2 are parallel.** Disjoint filesets (`crates/reposix-fuse/**` vs
  `crates/reposix-remote/**`). They share only `Cargo.lock`, which is auto-merged —
  both executors commit the lockfile as part of their own wave commit and the second
  commit rebases cleanly because the resolver output is the same set of crate versions.
- **C depends on both B waves.** Integration verification only makes sense once both
  crates have been rewired.
- **D depends on C.** Docs get updated after the phase is actually green.

## Wave summaries

### Wave A — core-crate prep
*File:* `14-A-core-prep.md`
*Executor time:* ~15 min (mostly read + sanity-grep; almost certainly produces no code change)
*Scope:* Confirm `reposix-core` needs no trait-surface changes. RESEARCH.md Q1/Q7 already
concluded no new `Error::VersionMismatch` variant is needed (string-match in `fs.rs`
instead) and `DeleteReason::Abandoned` already exists. If the executor discovers a
genuine blocker (e.g. a helper the remote crate would need that has to be `pub`), it
ships a narrow core-crate change here before B1/B2 start. Otherwise Wave A closes with a
status commit documenting "no core changes required; A is a no-op gate."

### Wave B1 — reposix-fuse write path through IssueBackend
*File:* `14-B1-fuse-write-through-backend.md`
*Executor time:* ~45 min
*Scope:* All of `crates/reposix-fuse/**` change. Rewrite `fs.rs::release` and
`fs.rs::create` to call the backend trait via two new timeout wrappers. Fold the surviving
bits of `fetch.rs` (`FetchError` enum minus `BadRequest`, `ConflictBody` helper, the
version-mismatch JSON re-parse in `backend_err_to_fetch`) into `fs.rs`. Delete `fetch.rs`
and `tests/write.rs`. Re-home the write-path tests onto `SimBackend` (in
`crates/reposix-core/src/backend/sim.rs`) and two new `fs.rs` tests for the new helpers.

### Wave B2 — reposix-remote helper through IssueBackend
*File:* `14-B2-remote-helper-through-backend.md`
*Executor time:* ~45 min
*Scope:* All of `crates/reposix-remote/**` change. Construct an `Arc<SimBackend>` from
the `RemoteSpec` once, store on `State`. Rewrite `execute_action` to dispatch to
`IssueBackend::{create_issue, update_issue, delete_or_close}` and `handle_import` /
`handle_export` to call `state.backend.list_issues`. Delete `src/client.rs`. Prune
`thiserror` and `reqwest` from `Cargo.toml` if unreferenced. Use
`SimBackend::with_agent_suffix(origin, Some("remote"))` per RESEARCH Q4.

### Wave C — integration verify
*File:* `14-C-integration-verify.md`
*Executor time:* ~30 min (mostly waiting on the gauntlet)
*Scope:* Run `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets
--locked -- -D warnings`, `bash scripts/green-gauntlet.sh --full` (includes
`--ignored` FUSE integration tests), `bash scripts/demos/smoke.sh`, and the
`scripts/demos/01-edit-and-push.sh` live write-demo. Grep-prove SC-14-01..05. Fix
anything red (expected: zero — but the executor must actually see green).

### Wave D — docs + CHANGELOG
*File:* `14-D-docs-changelog.md`
*Executor time:* ~15 min
*Scope:* CHANGELOG `[Unreleased]` entry under `### Changed` documenting R1 (assignee
clear on untouched PATCH is now the sim's three-value-clear behavior) and R2 (agent
attribution changes). Sweep CLAUDE.md v0.3 deferral prose. Update `docs/architecture.md`
diagram lines if they mention `fetch.rs`. Set the stage for the `v0.4.1` tag (tagging
itself happens post-phase at the user's gate).

</waves>

<risk_log>

Each risk is carried from `14-RESEARCH.md`; resolutions are locked by the orchestrator
and must not be re-litigated inside the phase.

| ID | Description | Resolution | Owning wave |
|----|-------------|------------|-------------|
| R1 | `render_patch_body` emits `"assignee": null` when the parsed frontmatter lacks an assignee; old `EgressPayload` skipped the key. The sim interprets `null` as Clear, absent as Unchanged — so untouched PATCHes now clear the assignee. | **Accept.** Document in CHANGELOG `### Changed`. The FUSE mount design treats the file as source of truth; clearing on an empty field is the honest interpretation. Do not modify `SimBackend::render_patch_body`. | B1 (tests adjust expectations); D (CHANGELOG entry) |
| R2 | `X-Reposix-Agent` header value changes from `reposix-fuse-<pid>` to `reposix-core-simbackend-<pid>-fuse` (and similarly for the remote helper). | **Accept.** Use `SimBackend::with_agent_suffix(origin, Some("fuse"))` / `Some("remote")`. Document in CHANGELOG `### Changed`. Sim's audit log still carries attribution; operators running `SELECT agent ...` must update substring matches. | B1 + B2 (call sites); D (CHANGELOG entry) |
| R3 | `If-Match` quoting changes from `"3"` unquoted to `"\"3\""` quoted on the wire. | Sim accepts both; tests update to quoted form when re-homed. No action beyond test assertion-string updates. | B1 (test re-home) |
| R4 | `FetchError::BadRequest` variant drops; POST 4xx becomes `FetchError::Core`. Both already mapped to `EIO`. | **Accept.** Not user-visible; marginal loss of log-line specificity. Add a low-value regression test in `sim.rs` for POST 400 body preservation. | B1 (test) |
| R5 | SC-14-09's prose mentions `version-mismatch` as a wire kind on the git-remote's `error refs/heads/...` line. The actual existing wire kind is `some-actions-failed`. | **Resolved as documentation clarification.** The correct wire kind is `some-actions-failed` (existing). Do NOT invent a `"version-mismatch"` wire kind. Wave B2 and Wave C phrase their assertions against `some-actions-failed`; Wave D updates any prose in PLAN/CONTEXT/CHANGELOG that alluded to a new kind. | B2 (assertions); D (docs wording) |
| R6 | Deleting `fetch_issue_origin_rejected` may lose SG-01 regression coverage if `http.rs` allowlist tests don't cover the `SimBackend` path. | **Mitigation:** Wave B1 adds a one-line test in `sim.rs` that constructs `SimBackend::new("http://evil.example".into())` and asserts `list_issues` returns `Err(Error::InvalidOrigin(_))`. | B1 (test) |
| R7 | Did not exhaustively verify every read-path call site in `fs.rs` routes through `_with_timeout` helpers. | **Mitigation:** Wave B1 runs `git grep -n "fetch::" crates/reposix-fuse/src/` after refactor; build-break at any surviving import is deterministic and catches it. Wave C also re-runs this grep in its acceptance checklist. | B1 (grep); C (grep) |
| R8 | Did not read `scripts/green-gauntlet.sh`; unverified that `--full` actually exercises the write-path end-to-end. | **Mitigation:** Wave C's checklist includes a **hand-run** of `scripts/demos/01-edit-and-push.sh` (the canonical write demo — verified present at phase-plan time) AND the gauntlet. If the gauntlet alone misses the write path, the demo run catches it. | C (live-demo step) |
| R9 | `SimBackend::delete_or_close` 404 prose differs from the old `ClientError::Status` prose; no test asserts on either. | **Accept.** Low risk. If an operator-facing stderr format matters, future phase adds it. | — (no-op) |
| R10 | `wiremock::Match` import path varies across versions. | **Mitigation:** Workspace pins `wiremock = "0.6"`; current code already uses this shape. If CI surfaces a version skew, the executor fixes the import in the same commit. | B1 (test re-home) |
| **R11 (new)** | Parallel waves B1 and B2 both touch `Cargo.lock` implicitly (via Cargo.toml edits or workspace-lock regeneration). Simultaneous commits may race. | **Mitigation:** Both wave plans instruct the executor to include `Cargo.lock` in the same commit as their crate change. The later-merging commit rebases and re-runs `cargo check --workspace`; if the resolver produces identical versions (expected — neither wave changes direct deps in a way that affects resolution), the rebase is trivial. Wave C's first step is `cargo check --workspace` to catch any real lockfile drift. | B1 + B2 (commit policy); C (sanity check) |
| **R12 (new)** | If the executor of B1 renames `FetchError` to `FsError` as a readability nicety (mentioned as optional in RESEARCH Q8), it ripples through `fs.rs::fetch_errno`, log messages, and any `#[doc]` that cites `FetchError`. | **Mitigation:** Wave B1 plan explicitly says **do NOT rename**. Keep the enum named `FetchError` to minimize diff. Rename is explicitly out-of-scope; flag for a future cleanup phase if anyone cares. | B1 (explicit scope guard) |
| **R13 (new)** | The `fs.rs::release` callback today uses `FetchError::Conflict { current }` to `warn!(ino = ino_u, current, "release: 409 conflict ...")`. After refactor, if the sim's 409 body ever drops the `current` field (or the prefix string `"version mismatch:"` changes), `backend_err_to_fetch` silently falls back to `FetchError::Conflict { current: 0 }` and the log shows `current=0` — misleading. | **Mitigation:** Wave B1 includes a pin test in `sim.rs` that the 409 body contains `"current":\d+` and that `Error::Other` starts with `"version mismatch:"`. If either contract changes, that test fails loudly, not the FUSE log. | B1 (pin test) |

</risk_log>

<verification_plan>

This is Wave C's executable checklist. It runs at the end of the phase and gates the
Wave D docs sweep.

## Required passes

1. `cargo test --workspace --locked` — expect **≥ 272** passing (LD-14-08). Failures are
   blocking.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` — clean. Any pedantic
   warning the refactor introduces must be addressed in the wave that produced it, not
   silenced here.
3. `bash scripts/green-gauntlet.sh --full` — includes `cargo test --workspace --release -- --ignored`
   (the FUSE integration tests that mount a real FUSE filesystem). Green.
4. `bash scripts/demos/smoke.sh` — **4 of 4** passing.
5. **Live write-demo:** `bash scripts/demos/01-edit-and-push.sh` exits 0. This is the
   authoritative end-to-end proof that `echo "..." > /tmp/mnt/issues/00000000001.md`
   lands on the sim, the audit row is written, and the read-back shows the new body.

## Grep proofs (executed verbatim and copy-pasted into Wave C's summary)

- **SC-14-01 (FUSE PATCH through trait):**
  `git grep -n 'patch_issue\b' crates/reposix-fuse/src/fs.rs` → **zero hits**.
  `git grep -n 'backend\.update_issue\|update_issue_with_timeout' crates/reposix-fuse/src/fs.rs` → at least one hit in `release`.
- **SC-14-02 (FUSE POST through trait):**
  `git grep -n 'post_issue\b' crates/reposix-fuse/src/fs.rs` → **zero hits**.
  `git grep -n 'backend\.create_issue\|create_issue_with_timeout' crates/reposix-fuse/src/fs.rs` → at least one hit in `create`.
- **SC-14-03 (fetch.rs deleted):**
  `test ! -e crates/reposix-fuse/src/fetch.rs` exits 0.
  `git grep -n 'pub mod fetch' crates/reposix-fuse/src/lib.rs` → **zero hits**.
  `git grep -n 'EgressPayload\|ConflictBody' crates/reposix-fuse/` → only test re-homes in `fs.rs` if any, or zero hits.
- **SC-14-04 (remote helper through trait):**
  `git grep -n 'api::\(list_issues\|patch_issue\|post_issue\|delete_issue\)' crates/reposix-remote/src/main.rs` → **zero hits**.
  `git grep -n 'state\.backend\.\(list_issues\|create_issue\|update_issue\|delete_or_close\)' crates/reposix-remote/src/main.rs` → at least four hits (list ×2, create, update, delete).
- **SC-14-05 (client.rs deleted):**
  `test ! -e crates/reposix-remote/src/client.rs` exits 0.
  `git grep -n 'mod client\|use crate::client' crates/reposix-remote/src/main.rs` → **zero hits**.

## Additional live-environment checks (R8 mitigation)

Wave C runs a manual smoke of the FUSE write path:

```bash
cargo run -p reposix-sim &                          # start sim on :7777
SIM_PID=$!
mkdir -p /tmp/reposix-mnt
cargo run -p reposix-fuse -- /tmp/reposix-mnt &     # mount
MNT_PID=$!
sleep 2

# Seed an issue (via sim's HTTP; the mount is read-through).
curl -sS -X POST http://127.0.0.1:7777/projects/demo/issues \
  -H 'X-Reposix-Agent: orchestrator-14C-0' \
  -H 'Content-Type: application/json' \
  -d '{"title":"phase-14-smoke","body":"before","labels":[]}'

# Read-back via mount to obtain the padded filename.
ls /tmp/reposix-mnt/issues/

# Write via the mount.
cat > /tmp/reposix-mnt/issues/00000000001.md <<EOF
---
id: 1
title: phase-14-smoke
version: 1
labels: []
---
edited via FUSE mount in phase 14 verification
EOF

# Verify the sim received the PATCH.
curl -sS http://127.0.0.1:7777/projects/demo/issues/1 | grep 'edited via FUSE mount'

# Verify audit log took a row.
sqlite3 runtime/sim.db 'SELECT agent, action, path FROM audit ORDER BY id DESC LIMIT 5;'
# Expect: most recent row's agent matches 'reposix-core-simbackend-%-fuse'.

# Cleanup.
fusermount3 -u /tmp/reposix-mnt || fusermount -u /tmp/reposix-mnt
kill $MNT_PID $SIM_PID
```

If any step fails, Wave C does NOT close; the phase returns to the responsible wave
(B1 for read-back mismatch, audit-log mismatch; B2 only if the git-remote-push variant
of the demo fails).

## Non-regression checks

- `git log --oneline -20` shows atomic commits with `(14-A)`, `(14-B1)`, `(14-B2)`,
  `(14-C)`, `(14-D)` phase prefixes.
- Phase 13 Wave C bucket-dir + tree overlay stays green (the `nested_layout.rs` and
  `tree.rs` integration tests are in the `--ignored` set; the gauntlet covers them).

</verification_plan>

<rollback_procedure>

If morning review rejects Phase 14 (either the agent-attribution change or the
assignee-clear semantic is deemed a wire-shape change we shouldn't ship under a
v0.4.1 tag), rollback is clean because every wave commits atomically with a distinct
phase prefix.

```bash
# From the repo root, with the phase's commits identified by the (14-*) prefix:
git log --oneline | grep -E '\(14-[A-D][12]?\)' | awk '{print $1}' > /tmp/phase14-shas.txt

# Revert in reverse chronological order (newest first).
tac /tmp/phase14-shas.txt | while read -r sha; do
    git revert --no-edit "$sha"
done

# Or, if the revert chain itself is hairy (unlikely — disjoint files mostly):
git reset --hard v0.4.0
# ^ DESTRUCTIVE. Only if a clean reset to the prior tag is desired.
# (User authorization required; CLAUDE.md OP-5 reversibility rules apply.)

cargo check --workspace
cargo test --workspace --locked
```

The `reset --hard v0.4.0` flavor requires user approval per CLAUDE.md OP-5. The revert
chain is the default. Either way, Phase 14's refactor is isolated enough that rollback
cost is ~5 minutes plus a `cargo build`.

Post-rollback, the phase directory stays in `.planning/phases/14-...` as an artifact
of the attempt. Next session either re-runs it with the rejected decision changed
(e.g. option B for R1 instead of A) or tombstones the phase and picks another cluster.

</rollback_procedure>

<success_criteria>

- [ ] SC-14-01..10 all satisfied (see `14-CONTEXT.md`).
- [ ] Every risk in `14-RESEARCH.md` §Risks has a logged resolution (see risk_log above).
- [ ] Every wave produced exactly one atomic commit with phase-prefixed message.
- [ ] `cargo test --workspace --locked` ≥ 272 passing.
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- [ ] `bash scripts/green-gauntlet.sh --full` green.
- [ ] `bash scripts/demos/smoke.sh` 4/4.
- [ ] `bash scripts/demos/01-edit-and-push.sh` exits 0.
- [ ] CHANGELOG `[Unreleased]` under `### Changed` documents R1 + R2.
- [ ] `git grep -n 'fetch::' crates/reposix-fuse/src/` returns zero hits.
- [ ] `git grep -n 'mod client' crates/reposix-remote/src/` returns zero hits.

</success_criteria>

<output>
After the phase completes, Wave C writes `.planning/phases/14-.../14-VERIFICATION.md`
(the verification run output). Wave D writes
`.planning/phases/14-.../14-SUMMARY.md` summarizing what shipped and the two `### Changed`
items (R1, R2) that become user-facing notes. Next session tags `v0.4.1`.
</output>
</content>
</invoke>