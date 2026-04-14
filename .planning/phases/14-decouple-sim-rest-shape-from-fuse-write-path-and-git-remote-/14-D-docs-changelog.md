---
phase: 14
wave: D
slug: docs-changelog
serial: true
depends_on_waves: [C]
blocks_waves: []
estimated_wall_clock: 15m
executor_role: gsd-executor
---

# Wave D — docs sweep + CHANGELOG

> Read `14-CONTEXT.md` and `14-RESEARCH.md` (whole file) before executing.
> This wave runs **after** Wave C has committed `14-VERIFICATION.md` and the phase
> is green end-to-end.

## Scope in one sentence

Add a `[Unreleased]` CHANGELOG entry documenting R1 and R2 (the two accepted behavior
changes), sweep any CLAUDE.md or `docs/architecture.md` prose referencing the old v0.3
"write path still speaks sim REST" deferral, and write `14-SUMMARY.md` for the phase.

## What NOT to touch

- **No code changes.** Documentation + metadata only. If Wave D discovers a code issue,
  it returns to the responsible wave (B1/B2) for a follow-up. Wave D never ships code
  changes of its own.
- Do not create a `v0.4.1` tag here. Tagging is outside the phase; the user gates it.
- Do not touch the `BREAKING` section of CHANGELOG. R1 and R2 are `### Changed`, not
  BREAKING. The `v0.4.1` scope is explicitly non-breaking.
- Do not rewrite `docs/architecture.md` from scratch. Surgical edits to the specific
  lines that reference `fetch.rs` or the old write-path language.

## Files to touch

- `CHANGELOG.md` — add the `[Unreleased]` `### Changed` entry.
- `CLAUDE.md` (project-level, at repo root) — search for v0.3 deferral prose mentioning
  `fetch.rs::patch_issue`, `client.rs`, or "write path still speaks the sim REST
  shape". If found, update to note Phase 14 closed it.
- `docs/architecture.md` — diagram lines at 112 and 156 (per phase-plan surface check)
  reference `fetch prior tree` and sed-through-mount; verify whether they mention the
  deleted modules. Update only if there's a live reference to `fetch.rs` or `client.rs`.
- `.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-SUMMARY.md`
  — created in this wave.
- `.planning/STATE.md` — advance the cursor to reflect Phase 14 shipped. (Follow the
  project convention: don't hand-edit structure; update `last_phase_shipped`,
  `next_phase`, and any "recent work" fields the existing file has. If STATE.md has a
  `committed_this_session` list, append Phase 14 there.)

## Files to delete

None.

## Tasks

### Task D.1 — Add CHANGELOG `[Unreleased]` entry

Open `CHANGELOG.md`. The current `[Unreleased]` section reads `— Nothing yet.`. Replace
with a new `### Changed` subsection documenting R1, R2, and the refactor's grounding
note.

Draft (tune wording but keep the facts):

```markdown
## [Unreleased]

### Changed

- **Audit attribution strings now prefix `reposix-core-simbackend-<pid>-`.** The FUSE
  daemon (`reposix-fuse`) and git-remote helper (`git-remote-reposix`) now emit the
  `X-Reposix-Agent` header as `reposix-core-simbackend-<pid>-fuse` and
  `reposix-core-simbackend-<pid>-remote` respectively (was `reposix-fuse-<pid>` /
  `git-remote-reposix-<pid>`). The sim's audit table still records a row for every
  write; operators matching on the old substrings must update their queries. The role
  suffix (`-fuse` / `-remote`) preserves caller-identity fidelity.
- **FUSE PATCHes now emit explicit `"assignee": null` when frontmatter omits the
  field.** Prior behavior skipped the key entirely (leaving the server value untouched);
  the new behavior sends `null`, which the simulator interprets as "clear". This is an
  honest reflection of the mount's "file is source of truth" design: a file with no
  `assignee:` line now means "assignee cleared," consistent with how every other field
  behaves. Users who wish to preserve an assignee across edits must keep the
  `assignee:` line in the frontmatter. Only affects the FUSE write path; CLI-issued
  PATCHes are unchanged.
- **`reposix-fuse` and `git-remote-reposix` now route every write through the
  `IssueBackend` trait.** Internal refactor closing HANDOFF "Known open gaps" items 7
  and 8. The sim's REST shape is now spoken by exactly one crate
  (`reposix-core::backend::sim`); all other callers dispatch through `create_issue`,
  `update_issue`, and `delete_or_close`. Deletes
  `crates/reposix-fuse/src/fetch.rs` (596 lines) and `crates/reposix-remote/src/client.rs`
  (236 lines). No user-visible wire-shape changes beyond the two `### Changed` items
  above.
```

Rationale for each bullet:

- **Bullet 1 (R2):** Required by R2 resolution; operators must know. Phrased as
  "substring changes" so monitoring dashboards adjust.
- **Bullet 2 (R1):** Required by R1 resolution; semantically significant; flagged as
  FUSE-only so CLI users aren't confused.
- **Bullet 3 (grounding):** The refactor itself deserves a changelog note because it
  permanently sets expectations about where the sim REST lives. Future readers scanning
  CHANGELOG understand why `fetch.rs` is gone.

### Task D.2 — Sweep CLAUDE.md (project-level, at repo root)

```bash
grep -nE "fetch\.rs|fetch::patch_issue|fetch::post_issue|client\.rs|still speaks the sim REST|write path still hardcodes" CLAUDE.md
```

If hits exist, update the prose to note Phase 14 closed the deferral. Suggested
replacement phrasing (adapt to surrounding style):

> Writes go through `IssueBackend::{create_issue, update_issue, delete_or_close}`
> (closed in Phase 14, 2026-04-14). The simulator's REST shape lives exclusively in
> `crates/reposix-core/src/backend/sim.rs`.

Do NOT rewrite other sections. CLAUDE.md carries operating rules the agent reads every
session; keep the diff surgical.

### Task D.3 — Sweep `docs/architecture.md`

```bash
grep -nE "fetch\.rs|fetch::|patch_issue|post_issue|client\.rs|EgressPayload" docs/architecture.md
```

If hits exist, rewrite only the relevant sentences to reference the trait path instead.
The sequence diagram at line ~156 (`H->>S: GET /projects/demo/issues (fetch prior tree)`)
is describing wire behavior — if it labels the client-side helper as
`fetch::fetch_issues`, update to `backend.list_issues`. Otherwise leave the diagram
alone — it's about wire lines, not code structure.

Line 112 (`Write path: sed -i 's/status: open/status: done/' /mnt/reposix/issues/00000000001.md`)
is a user-facing example; verify it still makes sense (it should — the sed command
works identically post-refactor because the FUSE callback is the seam).

### Task D.4 — Write `14-SUMMARY.md`

Create
`.planning/phases/14-decouple-sim-rest-shape-from-fuse-write-path-and-git-remote-/14-SUMMARY.md`
as the phase's durable artifact.

Template:

```markdown
# Phase 14 SUMMARY

> Shipped: <date>. Scope tag: v0.4.1.

## What shipped

- `reposix-fuse` and `git-remote-reposix` now route every write through the
  `IssueBackend` trait. Read `14-PLAN.md` for the wave-by-wave breakdown.

## Decisions locked during the phase

- **R1 (assignee clear on untouched PATCH):** accepted as-is. The sim's three-value
  `FieldUpdate<>` semantics now flow honestly through the FUSE mount. CHANGELOG entry
  written.
- **R2 (audit attribution suffix):** accepted as-is. New strings are
  `reposix-core-simbackend-<pid>-{fuse,remote}`. CHANGELOG entry written.
- **R5 (wire kind on git-remote version-mismatch):** clarified SC-14-09 prose. The
  existing `some-actions-failed` kind is preserved; no new `version-mismatch` wire kind
  introduced.

## Files removed

- `crates/reposix-fuse/src/fetch.rs` (596 lines)
- `crates/reposix-fuse/tests/write.rs`
- `crates/reposix-remote/src/client.rs` (236 lines)

Total: ~830 lines removed from the write path. Net test count: <record from 14-VERIFICATION.md>.

## Grounding impact

- The sim's REST shape lives in exactly one place (`reposix-core::backend::sim`).
- Future real-backend write support (Cluster A — Confluence writes, GitHub writes) now
  composes automatically through the FUSE mount and remote helper — no additional
  plumbing required.
- HANDOFF "Known open gaps" items 7 and 8 closed.

## Risks realized vs projected

- <list any risks from R1-R13 that materialized unexpectedly; document resolutions>.
- If none materialized: "All projected risks held their resolutions. No new surprises."

## Cost

- Wave A: ~<X> min.
- Wave B1: ~<X> min (parallel with B2).
- Wave B2: ~<X> min (parallel with B1).
- Wave C: ~<X> min.
- Wave D: ~<X> min.
- Total wall-clock: ~<sum — expect close to 2h15m per estimate>.

## Next steps

- User gates `v0.4.1` tag via `scripts/tag-v0.4.1.sh` (or equivalent — see tag-time
  runbook referenced in recent git history: commit 886215b).
- Next session candidates (session-5 stretch goals or next cluster):
  - Cluster C (swarm `--mode confluence-direct`).
  - OP-2 partial (`pages/INDEX.md`).
  - OP-7 SSRF + contention probes.
  - Cluster A (Confluence writes) — now unblocked by Phase 14.
```

### Task D.5 — Update STATE.md

Read `.planning/STATE.md`. Follow its existing structure to:

- Mark Phase 14 as shipped with today's date.
- Record the scope tag: `v0.4.1`.
- List the two files deleted (`crates/reposix-fuse/src/fetch.rs`,
  `crates/reposix-remote/src/client.rs`).
- Update `next_phase` or equivalent cursor to whatever session 5's stretch goal
  picked (or `unscoped` if no stretch goal landed).

Do not restructure STATE.md. Minimal, additive edits.

### Task D.6 — Commit

Single commit:

```
docs(14-D): close phase 14 — CHANGELOG + SUMMARY + STATE

- CHANGELOG.md: [Unreleased] ### Changed documents the audit
  attribution suffix shift (R2) and the assignee-clear-on-PATCH
  semantic (R1), plus the write-path refactor itself.
- CLAUDE.md: sweep v0.3 write-path-deferral prose (if any).
- docs/architecture.md: surgical edits to any line referencing
  fetch.rs or client.rs.
- .planning/phases/14-.../14-SUMMARY.md: phase artifact.
- .planning/STATE.md: cursor advanced to post-phase-14.

Phase 14 complete. v0.4.1 ready to tag at user gate.
```

## Tests to pass before commit

- `cargo check --workspace --locked` — still green (documentation changes should not
  affect it, but smoke-test).
- No markdown linter is configured in this project, so no lint pass required.
- Spot-check the CHANGELOG renders reasonably:
  `head -40 CHANGELOG.md` — verify the new `[Unreleased]` block appears at the top,
  above the `[v0.4.0]` block.

## Acceptance criteria

- [ ] `CHANGELOG.md` `[Unreleased]` section contains the three `### Changed` bullets.
- [ ] CLAUDE.md no longer references the old `fetch.rs` / `client.rs` write-path
      deferral (if it did before).
- [ ] `docs/architecture.md` has no stale references to the deleted modules.
- [ ] `14-SUMMARY.md` exists and is non-empty.
- [ ] `.planning/STATE.md` reflects Phase 14 shipped.
- [ ] Commit lands cleanly. `git status` clean.

## Non-scope

- Tagging `v0.4.1` — user-gated, outside the phase.
- Triggering CI / pushing tags.
- Starting the next phase or cluster.
- Any code change (Wave D is docs-only).

## References

- `14-CONTEXT.md` SC-14-10 (docs sweep).
- `14-RESEARCH.md` Q4 (R2 phrasing), Q9 (R1 phrasing), R5 (wire-kind clarification).
- `14-PLAN.md` risk log — R1, R2, R5.
- `CHANGELOG.md` existing `### Changed` examples under `[v0.4.0]` for style reference.
- Recent commit `886215b` (tag-time runbook reference for the next session).
