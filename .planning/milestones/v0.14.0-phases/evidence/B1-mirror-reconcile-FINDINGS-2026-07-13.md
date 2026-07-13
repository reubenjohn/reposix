# B1 mirror-reconcile — FINDINGS (2026-07-13)

**Lane:** v0.14.0 tag-remediation, STEP 1 (B1). **Status: BLOCKED** — the
documented operational fix does NOT resolve the litmus. Root-caused; needs a
manager/owner decision on the mirror-refresh path (do NOT improvise per charter).

Raw evidence in this dir:
- `B1-reconcile-probe-2026-07-13.txt` — before/after mirror state around `reposix sync --reconcile`.
- `B1-litmus-rerun-transcript-2026-07-13.txt` — post-reconcile litmus run (exit 1).

## What was tried (charter Task 2 + Task 3)

Ran the documented catch-up (root CLAUDE.md § "Mirror-head refresh promise"):
`reposix sync --reconcile` against `confluence::REPOSIX` (== TokenWorld, space
360450 — the exact remote/mirror wiring the litmus uses), then re-ran the
milestone-close vision litmus.

## Result — reconcile did NOT refresh the GitHub mirror repo

| Probe point | GitHub mirror HEAD | mirror `pages/2818063.md` |
|---|---|---|
| BEFORE `reposix sync --reconcile` | `3be8390` | `version: 1` |
| AFTER (`reconcile_exit=0`) | `3be8390` (**unchanged**) | `version: 1` (**unchanged**) |
| Backend (authoritative, v2 API) | — | `status: current, version: 7` |

Reconcile rebuilt the **local cache** (synthesis-commit `32cd15a`,
`meta.last_fetched_at` advanced) and pushed **nothing** to the GitHub mirror repo
`git@github.com:reubenjohn/reposix-tokenworld-mirror.git`. The cache's
`refs/mirrors/*` namespace was even empty afterward.

Litmus re-run (post-reconcile) → **exit 1**, identical rejection:

```
lost-update guard: record 2818063 conflicts (local base v1 vs backend v7, divergent writable content)
issue 2818063 modified on backend at 2026-07-06T06:28:06.103+00:00 (local base version: 1, backend version: 7). Run: git pull --rebase
 ! [rejected]        main -> main (fetch first)
```

## Root cause

`reposix sync --reconcile` calls `Cache::build_from`
(`crates/reposix-cli/src/sync.rs:112`, `crates/reposix-cache/src/builder.rs:68`),
which rebuilds the **local bare-repo cache** (`oid_map` + a synthesis commit on
`refs/heads/main` + the `last_fetched_at` cursor). It does **not** `git push` to
the GitHub mirror repo. The GitHub mirror's `pages/*.md` content is refreshed
**exclusively** by a bus push (`bus_handler::push_mirror`) — the very path
currently deadlocked by the stale record.

The litmus `git clone`s the GitHub mirror repo **fresh** every run
(`quality/gates/agent-ux/lib/litmus-flow.sh:27`) and edits the first
non-protected page — which is `pages/2818063.md`. Its working-tree base therefore
comes from the mirror at `version: 1`; the backend is at `version: 7` with
divergent writable content, so the push-time lost-update guard
(`crates/reposix-remote/src/precheck.rs:281`) correctly rejects. This is **not a
reposix bug** — the guard is working as designed. The stale data lives in the
**GitHub mirror repo content**, which reconcile does not touch.

**The deadlock:** refreshing the mirror's `2818063.md` to v7 requires a
successful bus push; the bus push is rejected because the mirror's `2818063.md`
is stale — chicken-and-egg. The out-of-band API restore (manager-authorized,
2026-07-13) bumped the backend without going through a reposix push, so the
mirror-head refresh promise never fired.

## Doc/reality gap noticed (fix-twice candidate)

Root CLAUDE.md § "Mirror-head refresh promise" implies `reposix sync --reconcile`
is the catch-up for a stale mirror. Empirically it only heals the **local cache**
(`oid_map`/cursor) and the cache's `refs/mirrors/<sot>-head` observability ref —
NOT the **GitHub mirror repo content** that a fresh clone reads. The prose
conflates two distinct "mirrors": (a) the cache's `refs/mirrors/<sot>-head`
observability ref, and (b) the external GitHub mirror repo. `--reconcile` cannot
refresh (b). This is the exact class GTH-V15-09 (self-healing fixture) targets.

**The helper's own teaching string is also suspect here:** it prints
`Run: git pull --rebase`, but attach sets `fetch`/`pull` to read from the
`origin` (stale mirror) — so a naive `git pull --rebase` would re-pull `version:
1` from the same stale mirror and not resolve the conflict. Un-sticking requires
fetching backend-current explicitly through the reposix bus remote, not the
mirror. Flagged for the manager's mirror-refresh decision.

## Recommended next move (for the manager — NOT executed here)

The mirror needs its `pages/2818063.md` refreshed to backend-current v7. Options
(each an external mutation to the sanctioned mirror via a non-standard path —
above the self-decide bar, hence reported not improvised):
1. Fetch backend-current through the reposix bus remote into a persistent attach
   clone, rebase the marker edit, and push (bus push refreshes the mirror). Needs
   verification that fetch reads the backend, not the stale mirror.
2. Directly commit the v7-content `pages/2818063.md` to the GitHub mirror repo and
   push (force the mirror to backend-current).
3. Accept the litmus NOT-VERIFIED for this tag with a documented reason and land
   GTH-V15-09 (litmus self-reconciles the mirror before push) as the durable fix.
