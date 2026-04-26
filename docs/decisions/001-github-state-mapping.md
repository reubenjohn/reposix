# ADR-001: GitHub state mapping

- **Status:** Accepted
- **Date:** 2026-04-13
- **Deciders:** reposix core team
- **Supersedes:** none
- **Superseded by:** none
- **Scope:** `crates/reposix-github` (v0.2, `GithubReadOnlyBackend`) and any future
  backend that consumes GitHub's 2-valued `state` + `state_reason` model.

## Context

reposix models issue workflow with a 5-valued `IssueStatus` (see
`crates/reposix-core/src/issue.rs`): `Open`, `InProgress`, `InReview`, `Done`,
`WontFix`. This is Jira-flavored — it captures both "in flight" and "terminal"
states with enough granularity for FUSE-mounted views to be useful.

GitHub's Issues API is 2-valued: `state` is `open` or `closed`. GitHub v3 then
layered `state_reason` on top (`completed`, `not_planned`, `reopened`, `duplicate`,
plus `null` for legacy rows). That is still not enough to encode the distinction
between "being worked on" and "awaiting review" — GitHub surfaces that via
labels, projects, and milestones.

The `GithubReadOnlyBackend` (Phase 8-C) needs a **deterministic, documented,
round-trippable mapping** so:
- The list/get read path translates GitHub rows into the same
  `IssueStatus` enum the FUSE layer already understands.
- The dark-factory regression (`scripts/dark-factory-test.sh github`) can drive
  end-to-end clone+grep+edit+push against the sim and GitHub backends and get
  structural agreement on normalized output.[^parity]
- The future v0.2 write path can reverse the mapping (set a state + state_reason
  + label combination from an `IssueStatus`).

## Decision

Use this mapping, with GitHub labels `status/in-progress` and `status/in-review`
as the distinguishing signal for the two "open-but-active" variants.

| `IssueStatus` | GitHub `state` | `state_reason`  | Labels                  |
| ------------- | -------------- | --------------- | ----------------------- |
| `Open`        | `open`         | (any / null)    | no `status/*` label     |
| `InProgress`  | `open`         | (any / null)    | `status/in-progress`    |
| `InReview`    | `open`         | (any / null)    | `status/in-review`      |
| `Done`        | `closed`       | `completed`     | (label ignored)         |
| `WontFix`     | `closed`       | `not_planned`   | (label ignored)         |

### Read-path rules (GitHub → reposix)

1. If `state == open`:
   - If the issue carries label `status/in-review` → `InReview`.
   - Else if it carries label `status/in-progress` → `InProgress`.
   - Else → `Open`.

**Label precedence when BOTH `status/in-review` AND `status/in-progress` are present:** `InReview` wins (review is downstream of in-progress in a typical workflow; treating a PR with both labels as "in review" errs toward the more-recent state). The write path, when called with `IssueStatus::InReview`, always removes `status/in-progress` to keep the two labels mutually exclusive on round-trip. `GithubReadOnlyBackend::translate` encodes this precedence in its `else if` chain at `crates/reposix-github/src/lib.rs` (review check precedes progress check).
2. If `state == closed`:
   - If `state_reason == "not_planned"` → `WontFix`.
   - If `state_reason == "completed"` → `Done`.
   - Unknown / missing `state_reason` (e.g. `"reopened"`, `"duplicate"`, `null`
     on pre-2022 rows) → fall through to `Done`. **Pessimistic fallback**:
     "we know it's closed; we don't know if it was successful, but we don't
     want to lie and say the issue is still `Open`."

### Write-path rules (reposix → GitHub, v0.2)

1. `Open` → `PATCH { state: "open" }` + remove `status/in-progress`,
   `status/in-review` labels.
2. `InProgress` → `PATCH { state: "open" }` + add `status/in-progress` label
   (and remove `status/in-review` if present).
3. `InReview` → `PATCH { state: "open" }` + add `status/in-review` label
   (and remove `status/in-progress` if present).
4. `Done` → `PATCH { state: "closed", state_reason: "completed" }`.
5. `WontFix` → `PATCH { state: "closed", state_reason: "not_planned" }`.

### Unknown label handling

A `status/unknown-flavor` label on an open issue is **ignored** — the mapping
table falls back to `Open`. This is deliberate: third-party workflow tooling
often sprays labels, and we don't want a typo to leak an unexpected enum
variant into the FUSE mount.

## Consequences

- **Label naming convention is load-bearing.** Teams that use
  `status: in progress` (with a space, or a colon instead of a slash) will
  not round-trip. ADR-001 picks `status/in-progress` because
  [GitHub's own documentation](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels)
  recommends `scope/value` naming for scoped labels.
- **The mapping is lossy on the closed side.** GitHub has `state_reason` values
  beyond `completed` / `not_planned` (`reopened`, `duplicate`); reposix folds
  them into `Done`. The write path only ever emits `completed` or `not_planned`.
- **Parity demos must pass labels through.** The `list` subcommand's JSON
  output includes the `labels` field untouched, so humans can verify the
  `status/*` convention is being honored end to end.
- **v0.2 will add a `Duplicate` enum variant** if user demand is high enough.
  Until then, `Duplicate` maps to `Done` on read and `Done` -> `completed` on
  write. Documented here so the future migration path is obvious.

## References

- [GitHub REST API: state_reason values](https://docs.github.com/en/rest/issues/issues#get-an-issue)
- [GitHub REST API: update an issue](https://docs.github.com/en/rest/issues/issues#update-an-issue)
- `crates/reposix-core/src/issue.rs` — `IssueStatus` definition.
- `crates/reposix-core/src/backend.rs` — the `IssueBackend` seam this adapter implements.
- `.planning/phases/08-demos-and-real-backend/08-CONTEXT.md` — phase spec that
  introduced the seam + demanded this ADR.

[^parity]: The historical `scripts/demos/parity.sh` demo script was deleted in
    v0.11.1 (§7-F2) alongside the FUSE-era `scripts/demos/` directory. The
    v0.9.0 architecture's regression equivalent is `scripts/dark-factory-test.sh
    github`.
