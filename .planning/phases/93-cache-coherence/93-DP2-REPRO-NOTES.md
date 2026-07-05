# P93 DP-2 prove-before-fix — D-P92-03 delta-sync coherence (REPRODUCED)

**Lane:** P93 investigation (prove-before-fix, DP-2 discipline). **Author:** P93 Exec1.
**Date:** 2026-07-05. **Verdict: REPRODUCED — real, deterministic cache-coherence bug.
NOT environmental.** No production fix applied (coordinator-gated). Nothing pushed.

## Verdict in one line

Writer B's `git pull --rebase` after a two-writer conflict dies with
`fatal: git upload-pack: not our ref <oid>` / `could not fetch <oid> from promisor
remote` **whenever the conflicting write lands in the same wall-clock second as B's cache
cursor** — reproduced 4/4 in that window (1 deterministic pin + 3 natural same-second
runs), and cleanly ABSENT in the 2-second-gap negative control. This settles D-P92-03: P92
Exec2's non-repro was a timing straddle (its run crossed a second boundary, like the
`gap2s` control), not evidence the bug is false.

## Executed evidence (git 2.54.0 container, two independent caches)

Host box is git 2.25.1 (below the `>=2.34` helper floor), so ALL helper/push/pull litmus
ran inside a git-2.54 container (`ubuntu:24.04` + `git-core` PPA, `--network host` to reach
the host sim on `127.0.0.1:7878`, release binaries built once on the host and bind-mounted
read-only) — the same pattern as `92-T4-REPRO-NOTES.md`. Two independent working trees A/B,
each with its OWN `REPOSIX_CACHE_DIR` (two bare caches = realistic two-agent topology).

| Run | `list_changed_since` | rebase blob fetch | exit | transcript |
|-----|----------------------|-------------------|------|------------|
| `pin-cursor` (cursor pinned into A's write-second, deterministic) | `0 changed (of 6)` | `fatal: not our ref <oid>` | **128** | `transcripts/transcript.pin-cursor.txt` |
| `same-second` ×3 (natural tight init+push race, NO manipulation) | `0 changed (of 6)` | `fatal: not our ref <oid>` (all 3) | **128** | `transcripts/transcript.same-second-x3.txt` |
| `gap2s` (2s sleep before A's push → later second) — NEGATIVE CONTROL | `1 changed (of 6)` | blob materialized → `CONFLICT (content)` | 1 | `transcripts/transcript.gap2s.txt` |

The ONLY variable between REPRODUCED and CLEAN is whether A's write shares a truncated
second with B's cursor. That isolates causation to the timestamp boundary below. The
`gap2s` ordinary content conflict PROVES the blob WAS fetched when the change is detected —
so the failure is specifically the un-detected-change path, not a transport defect.

Re-run any row (rebuilds binaries, one cargo invocation, then containerized litmus):
```
bash .planning/phases/93-cache-coherence/repro/run-repro.sh <pin-cursor|same-second|gap2s>
```

## Root cause (two coupled defects — a trigger and a latent amplifier)

**1. TRIGGER — sim `list_changed_since` drops same-second writes.**
`crates/reposix-sim/src/routes/issues.rs`: `now_rfc3339()` (L138-139) stamps `updated_at` at
SECONDS precision (`to_rfc3339_opts(SecondsFormat::Secs, true)`), and the `since` filter
(L180-183) truncates the cursor to seconds too and compares with a STRICT `updated_at > ?2`.
So any write whose truncated second is `<=` the cursor's truncated second is invisible to
`list_changed_since`, even though `list_records` (unfiltered) still returns its new content.
Inline predicate from the transcript:
`updated_at('…T10:08:10Z') > trunc(cursor)='…T10:08:10Z' => FALSE => dropped`.

**2. LATENT AMPLIFIER — `Cache::sync` builds a tree entry it cannot serve** (the actual
`not our ref` defect). `crates/reposix-cache/src/builder.rs::sync()`:
- Step 4 (L293-328) sources the git TREE from `list_records` — the FULL current backend
  state — so issue 1's tree entry is ALWAYS its current-content OID (`X_new`), reflecting
  A's write regardless of the delta query.
- Steps 3 & 5 (L265-291, L381-392) only `write_blob` + `INSERT oid_map` for the
  `list_changed_since` delta.

When (1) makes those two sources disagree, the tree references `X_new` but the object store
has no such blob and `oid_map` has no `X_new → issue_id` row — a **dangling tree entry**.
On B's `pull --rebase`, the partial-clone lazy fetch of `X_new` reaches
`Cache::read_blob` (builder.rs L450-457) → `oid_map` miss → `Error::UnknownOid`. The helper
(`stateless_connect.rs` L369-385) treats `UnknownOid` as non-fatal and leaves the `want` for
`git upload-pack`, which rejects it: `not our ref X_new`. Exactly the observed stderr.

**Invariant violated:** *every blob OID the HEAD tree references must be resolvable by
`read_blob`.* The tree is derived from a DIFFERENT, broader source (`list_records`, always
current) than blob-materialization + `oid_map` (the `list_changed_since` delta); any
under-report of the delta relative to the tree produces an unservable OID. The seconds
boundary is today's trigger, but the coupling is the durable fragility — a fix that only
tightens the sim's timestamp precision would paper over a cache-layer invariant break.

## Regression test (FAILING / RED — no fix applied)

`crates/reposix-cache/tests/delta_sync.rs::delta_sync_tree_references_only_resolvable_oids`
— a deterministic, container-free RED test against the REAL in-process sim. It pins the
cache cursor into a backend write's second (same trigger), runs `sync()`, and asserts the
coherence invariant. Current-`main` output:

```
COHERENCE VIOLATION (D-P92-03): HEAD tree references blob 7afff493… that read_blob cannot
resolve — a partial-clone lazy fetch of this OID dies `git upload-pack: not our ref 7afff…`.
Got: Err(UnknownOid("7afff493…"))
```

It is `#[ignore]`d so it does NOT break the default CI gate (the fix is coordinator-gated);
prove it RED with `cargo test -p reposix-cache --test delta_sync -- --ignored`, and drop the
`#[ignore]` in the same commit that lands the coherence fix. The other 3 `delta_sync` tests
stay green (`3 passed; 1 ignored`).

## For the coordinator (D-P92-03 disposition)

- CONFIRMED real + deterministically triggerable. Recommend adjudicating a fix at the
  **cache layer** (restore the tree-vs-`oid_map` invariant), not (or not only) the sim
  timestamp precision — the sim boundary is one way to trip a general coherence break the
  helper cannot survive. A pure sim-precision fix leaves the latent amplifier live for any
  future backend whose `updated_at` resolution or clock skew re-creates the disagreement
  (real Confluence/JIRA/GitHub `updated_at` are second-resolution too).
- Candidate fix directions (for the fix plan, NOT applied here): (a) in `sync()` Step 4,
  never emit a tree entry for an OID that is neither freshly-written nor already in
  `oid_map` — reconcile the `list_records` set against the materialized/known set and
  materialize (or `oid_map`-record) the difference; or (b) make `read_blob` resolve an
  unknown OID by content-addressed re-fetch of the record whose CURRENT render hashes to the
  requested OID. Either restores the invariant.
