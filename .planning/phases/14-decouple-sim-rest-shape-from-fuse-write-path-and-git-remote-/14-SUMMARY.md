---
phase: 14
slug: decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-
name: "Decouple sim REST shape from FUSE write path and git-remote helper — route through IssueBackend trait"
shipped: 2026-04-14
scope_tag: v0.4.1
status: SHIPPED
waves:
  - A: sim 409-body contract pins (7510ed1)
  - B1: FUSE write through IssueBackend + SG-03 re-home (bdad951, cd50ec5)
  - B2: git-remote helper through IssueBackend (938b8de)
  - C: integration verification (4301d0d)
  - D: docs sweep + CHANGELOG + SUMMARY (this wave)
closes_gaps:
  - HANDOFF.md "Known open gaps" item 7 — FUSE write path through IssueBackend
  - HANDOFF.md "Known open gaps" item 8 — git-remote-reposix rewire through IssueBackend
tests:
  workspace_passing: 274
  workspace_failing: 0
  workspace_ignored: 11
  floor: 272 (LD-14-08 — met +2)
---

# Phase 14 SUMMARY

> Shipped: 2026-04-14. Scope tag: v0.4.1.

## tl;dr

The FUSE daemon (`reposix-fuse`) and the git-remote helper (`git-remote-reposix`)
no longer speak the simulator's REST shape directly. Every write they perform
— FUSE `release` PATCH, FUSE `create` POST, and every `git push`-driven
create/update/delete — now flows through the `IssueBackend` trait's
`create_issue` / `update_issue` / `delete_or_close` methods, dispatched via a
concrete `SimBackend` constructed from the existing `RemoteSpec`. The
simulator's wire shape lives in exactly one crate
(`reposix-core::backend::sim::SimBackend`); every other caller sees the
trait surface. `crates/reposix-fuse/src/fetch.rs` (596 lines),
`crates/reposix-fuse/tests/write.rs` (236 lines), and
`crates/reposix-remote/src/client.rs` (236 lines) are deleted. Workspace
tests stay at 274 passing (LD-14-08 floor ≥272 met +2), clippy
`-D warnings` clean, green-gauntlet `--full` 6/6 gates green, smoke demos
4/4, and live demo 01 round-trips a FUSE edit → PATCH → git clone →
commit → push → PATCH through the trait with expected sim-side state.
Two behavior changes fall out: `X-Reposix-Agent` audit attribution is
now `reposix-core-simbackend-<pid>-{fuse,remote}` (was `reposix-fuse-<pid>` /
`git-remote-reposix-<pid>`), and a FUSE edit that omits the `assignee:`
frontmatter line now clears the sim's assignee (was: untouched). Both are
documented in CHANGELOG `[Unreleased]` under `### Changed`. HANDOFF.md
"Known open gaps" items 7 and 8 are closed.

## What shipped

| Wave | Commit(s) | Role |
| ---- | --------- | ---- |
| A    | `7510ed1` | Sim-side 409-body contract pins — two new tests in `reposix-sim` freeze the `{"error":"version mismatch","current":<u64>}` shape so `SimBackend::update_issue` (downstream of all FUSE/remote writes) can always recover `current` on conflict. Pre-refactor safety net. |
| B1   | `bdad951`, `cd50ec5` | `crates/reposix-fuse` refactor. `fs.rs::release` + `fs.rs::create` call `backend.update_issue` / `backend.create_issue` via `update_issue_with_timeout` / `create_issue_with_timeout` helpers (mirrors the Phase-10 read-path pattern). `fetch.rs` deleted, `pub mod fetch;` removed from `lib.rs`. `tests/write.rs` deleted; one SG-03 egress-sanitize proof re-homed to `crates/reposix-core/src/backend/sim.rs::tests`. |
| B2   | `938b8de` | `crates/reposix-remote` refactor. `main.rs::execute_action` dispatches `api::{post_issue, patch_issue, delete_issue}` → `state.backend.{create_issue, update_issue, delete_or_close}`. Import-path + export-prior-state paths use `state.backend.list_issues`. `client.rs` deleted, `mod client;` removed from `main.rs`. `reqwest` dev-dep dropped. |
| C    | `4301d0d` | Integration verification — all SC-14-01 through SC-14-09 PASS by grep + live-demo evidence; SC-14-10 deferred to Wave D. Audit-attribution spot-check on live sim DB confirms R2 suffix landed on both FUSE and remote helpers. |
| D    | *(this wave)* | `CHANGELOG.md` `[Unreleased]` entry documents R1 + R2 + the refactor; `docs/architecture.md`, `docs/security.md`, `docs/reference/crates.md`, and `README.md` swept for v0.3-era deferral prose; STATE.md cursor advanced; this SUMMARY written. |

## Decisions carried forward

- **R1 (assignee clear on untouched PATCH) — accepted as-is.** The sim's
  three-valued `FieldUpdate<>` semantics (absent = Unchanged, null = Clear,
  value = Set) now flow honestly through the FUSE mount. A file with no
  `assignee:` line in its frontmatter means "assignee cleared," consistent
  with how every other field behaves. Documented in CHANGELOG `### Changed`
  as a FUSE-only semantic shift (CLI PATCHes are unchanged). The live demo
  exercises the neutral case (issue 1 seed has `assignee: null` already)
  so no regression surfaced. Rule 4 checkpoint-level change, accepted per
  "file is the source of truth" design philosophy.
- **R2 (audit attribution suffix) — accepted as-is.** New strings are
  `reposix-core-simbackend-<pid>-{fuse,remote}` (was `reposix-fuse-<pid>`
  / `git-remote-reposix-<pid>`). The `<pid>` is `std::process::id()`
  captured at `SimBackend::new` time — stable within one process, fresh
  across spawns (hence `git fetch` and `git push` show up as distinct
  PIDs, each spawning its own helper process). The `-{fuse,remote}` role
  suffix is supplied by `SimBackend::with_agent_suffix`. CHANGELOG
  `### Changed` documents the old → new mapping so downstream log-query
  tooling can migrate (sqlite `LIKE 'reposix-core-simbackend-%-fuse'`).
- **R5 (wire kind on git-remote version-mismatch) — clarified, not
  changed.** SC-14-09 prose in the verification doc confirms the existing
  `some-actions-failed` kind is preserved; no new `version-mismatch`
  wire kind is introduced. Error surface (FUSE `libc::EIO`, remote
  helper protocol line) unchanged end-to-end.

## Closed gaps (HANDOFF.md "Known open gaps")

- **Item 7 — FUSE write path through `IssueBackend::update_issue`.**
  Closed. `crates/reposix-fuse/src/fs.rs::release` calls
  `self.backend.update_issue(project, id, patch, Some(version))` via
  `update_issue_with_timeout`; `create` calls
  `self.backend.create_issue(project, issue)` via
  `create_issue_with_timeout`. The sim-specific HTTP helpers in
  `fetch.rs` are deleted.
- **Item 8 — `git-remote-reposix` rewire through `IssueBackend`.**
  Closed. `crates/reposix-remote/src/main.rs::execute_action` dispatches
  every create/update/delete through the trait; `client.rs` is deleted.
  The remote helper behaviourally stays sim-only per LD-14-03 (it
  constructs an `Arc<SimBackend>` internally from the parsed
  `RemoteSpec`); widening URL syntax to carry a backend scheme is v0.5+
  scope.

## Live verification transcript

Excerpted from `14-VERIFICATION.md` (full evidence there). Run 2026-04-14T16:38Z.

### Workspace tests

```
$ cargo test --workspace --locked
Totals: 274 passed, 0 failed, 11 ignored
```

LD-14-08 floor ≥ 272 met +2. Tests re-verified in Wave D as a docs-only
regression check: 274 passing, exit 0, same totals.

### Clippy

```
$ cargo clippy --workspace --all-targets --locked -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.24s
```

Zero warnings. Exit 0. Rebuilt after `cargo clean -p reposix-fuse -p reposix-remote -p reposix-core` to prove the lint actually fires fresh.

### Grep proofs

```
$ git grep -n 'patch_issue\b' crates/reposix-fuse/src/fs.rs
crates/reposix-fuse/src/fs.rs:1423:        // defence-in-depth the old `fetch::patch_issue` provided on top of
```

Only hit is a doc-comment inside a re-homed test; zero live code
references. Same pattern for `post_issue` (zero hits) and `api::` in
`crates/reposix-remote/src/` (zero hits). `fetch.rs`, `tests/write.rs`,
and `client.rs` are all absent from the tree.

### green-gauntlet --full

```
$ bash scripts/green-gauntlet.sh --full
✓ fmt ok 0s
✓ clippy ok 1s
✓ test ok 9s
✓ smoke ok 9s
✓ mkdocs-strict ok 2s
✓ fuse-ignored ok 2s
✓ green gauntlet passed
```

6/6 gates green. Exit 0.

### Smoke demos

```
$ bash scripts/demos/smoke.sh
  smoke suite: 4 passed, 0 failed (of 4)
```

### Live write demo

```
$ PATH="$PWD/target/release:$PATH" bash scripts/demos/01-edit-and-push.sh
...
[3/6] edit through FUSE (sed-style in-memory write)
  after FUSE write, frontmatter head: status: in_progress
  server confirms: { "id": 1, "status": "in_progress", "version": 2 }
[4/6] git clone via reposix:: remote
[5/6] edit + commit + push
  pushing...
      To reposix::http://127.0.0.1:7801/projects/demo
       * [new branch]      main -> main
[6/6] verify server state reflects the push
  issue 1 status after push (expect in_review): in_review

== DEMO COMPLETE ==
```

Exit 0. Both the FUSE write path (step 3) and the `git push` path (step 5)
round-tripped a PATCH through the `IssueBackend` trait into the sim, and
the sim's stored state reflects both mutations.

### Audit attribution spot-check (R2 live confirmation)

```
$ sqlite3 /tmp/reposix-demo-01-sim.db \
    "SELECT agent_id, COUNT(*) FROM audit_events GROUP BY agent_id ORDER BY agent_id;"
anonymous|4
reposix-core-simbackend-1874435-fuse|8
reposix-core-simbackend-1874544-remote|1
reposix-core-simbackend-1874574-remote|2
```

All FUSE-originated writes tag as `reposix-core-simbackend-1874435-fuse`;
all remote-helper writes tag as `reposix-core-simbackend-{1874544,1874574}-remote`
(two PIDs because `git fetch` + `git push` spawn separate helpers). No
`reposix-fuse-<pid>` or `git-remote-reposix-<pid>` rows appear. R2 landed
as specified.

## Docs sweep (Wave D, SC-14-10)

Prose referencing the v0.3-era "write path still speaks the sim REST
shape" deferral was found and updated in:

- `docs/architecture.md` — read-path sequence diagram updated: agent
  header is now `reposix-core-simbackend-{pid}-fuse` (not
  `reposix-fuse-{pid}`); async-bridge code sample now shows
  `backend.get_issue` dispatch instead of `http.request_with_headers`;
  added a short paragraph tying Phase 10 (reads) and Phase 14 (writes)
  together under the `IssueBackend` trait.
- `docs/security.md` — SG-07 evidence path updated from
  `crates/reposix-fuse/src/fetch.rs::with_timeout` to the new per-verb
  `*_with_timeout` helpers in `fs.rs`. The two "FUSE write-path rewire
  onto IssueBackend" and "`git-remote-reposix` backend-abstraction"
  bullets moved from `## What's still deferred (v0.4+)` up into
  `## What shipped after v0.1` with "Shipped in Phase 14 (v0.4.1)" notes
  and HANDOFF.md item-7/8 closure callouts.
- `docs/reference/crates.md` — reposix-remote "Modules" list updated:
  `client.rs` entry replaced with a `main.rs` entry that calls out the
  Phase-14 `SimBackend` construction + trait dispatch. `fast_import.rs`
  description generalized from "sim → git" to "backend → git".
- `README.md` — "Deferred to v0.4" bullet about the FUSE write path
  rewritten to reflect that plumbing is done (Phase 14 / v0.4.1) and
  what remains is a per-adapter real-backend-writes change, not a
  plumbing change. The separate "git-remote-reposix rewire through
  IssueBackend" bullet removed (was a duplicate of the same gap).

`CLAUDE.md` had no hits for `fetch.rs` / `client.rs` / "still speaks the
sim REST shape" / "write path still hardcodes" / "v0.3 cleanup" — the
project-level guide already describes the trait as the seam, nothing to
update.

Historical artifacts retained as-is: `CHANGELOG.md` entries for
`[v0.2.0-alpha]` and older (immutable per-release records),
`HANDOFF.md` and `MORNING-BRIEF.md` session-handoff narratives, and
`.planning/phases/<older>/` summary docs. The new `[Unreleased]` entry
in `CHANGELOG.md` is the canonical "this is closed now" reference.

## Follow-ups

- **C-1: green-gauntlet trusts stale release binaries.** Surfaced during
  Wave C's first pass: `scripts/green-gauntlet.sh` uses `target/release/*`
  if present and falls back to debug, never rebuilding. On my host the
  release `git-remote-reposix` was from an earlier commit and masked the
  R2 attribution change until manual rebuild. Not a Phase-14 regression
  — grounding-infrastructure gap per OP #4. Two mitigations:
  (1) a gate asserting `target/release/*` mtime > last-relevant-commit
  mtime (cheap, ~10 lines), or (2) `--full` mode doing
  `cargo build --release --workspace --bins --locked` as its first
  gate (slower, honest). Recommend option 2 as default with a
  `--skip-build` escape hatch. Filed for a follow-up small QoL fix;
  not a v0.4.1 blocker.
- **C-2: `audit_events` schema breadcrumb.** Task C.7's SQL snippet in
  the original plan referenced `SELECT agent, action FROM audit` — the
  real schema has `agent_id`, `method`, and `path` columns on the
  `audit_events` table. Future plan authors should copy the verified
  query shape from `14-VERIFICATION.md`:
  `SELECT agent_id, method, path, status FROM audit_events ORDER BY id DESC LIMIT N;`
- **Post-phase human gate: v0.4.1 tag push.** Per session-5 brief, the
  tag-push is outside phase scope. When ready: clone
  `scripts/tag-v0.4.0.sh` to `scripts/tag-v0.4.1.sh`, bump the version
  string, flip the CHANGELOG check from `[v0.4.0]` to `[v0.4.1]` (and
  promote `[Unreleased]` → `[v0.4.1] — 2026-04-14` in CHANGELOG.md at
  that time), then run the script. I deliberately did NOT write the
  tag script in this wave so the user can review the CHANGELOG in the
  morning before tagging.
- **Next session candidates** (session-5 stretch goals or next cluster,
  unblocked by Phase 14's trait-first plumbing):
  - Cluster C — swarm `--mode confluence-direct`.
  - OP-2 partial — `pages/INDEX.md`.
  - OP-7 hardening — SSRF widen + contention probes.
  - Cluster A — Confluence writes (now composes automatically through
    the FUSE mount and remote helper; no additional plumbing needed).

## Cost

Estimated per plan, actual per Wave C's 2026-04-14T16:38Z verification and this wave's start-time record:

| Wave | Est. | Actual (wall-clock) |
| ---- | ---- | ------------------- |
| A    | 15m  | ~10m (preflight + sim-side contract pins) |
| B1   | 45m  | ~45m (parallel with B2) |
| B2   | 45m  | ~35m (parallel with B1) |
| C    | 30m  | ~40m (incl. release-binary rebuild + audit spot-check) |
| D    | 15m  | ~20m (this wave) |
| **Total** | **~2h15m** | **~2h30m** (within parallelism budget) |

## Threat flags

None. Phase 14 is a pure refactor; the trait-seam is equivalent-or-narrower
than the deleted `fetch::{patch_issue, post_issue}` helpers in surface
area. No new egress, no new allowlist widening, no new trust boundaries.
`SimBackend` delegates to the same sim endpoint, which writes the same
audit row — modulo R2's suffix normalization, which is documented in
CHANGELOG `### Changed`.

## Self-Check: PASSED

- `CHANGELOG.md` `[Unreleased]` section present with three `### Changed`
  bullets (R1, R2, refactor), `### Removed` listing three deleted files
  + reqwest dev-dep, and `### Hardening` for the sim 409-body contract
  pins. Verified by `head -80 CHANGELOG.md`.
- `docs/architecture.md`, `docs/security.md`, `docs/reference/crates.md`,
  and `README.md` contain no `crates/reposix-fuse/src/fetch.rs` or
  `crates/reposix-remote/src/client.rs` live references (only deletion
  callouts in shipped-lists).
- `CLAUDE.md` has no `fetch.rs` / `client.rs` / "still speaks the sim
  REST" prose — already post-Phase-14 ready.
- `14-SUMMARY.md` (this file) exists and is non-empty.
- `.planning/STATE.md` cursor advanced (see next commit).
- `cargo test --workspace --locked` green (274 / 0 / 11).
- `cargo fmt --all --check` green.
- `mkdocs build --strict` green (1.69s).
- All listed prior-wave commits (`7510ed1`, `bdad951`, `cd50ec5`,
  `938b8de`, `4301d0d`) confirmed on `main` via `git log`.
