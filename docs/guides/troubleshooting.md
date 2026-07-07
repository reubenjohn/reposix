---
title: Troubleshooting
---

# Troubleshooting

Reposix's substrate property means an agent recovers from every error by reading stderr and following it verbatim. This page is for cases where you need a bigger hint than stderr provides — plus diagnostic queries for when nothing is broken but you want to know what reposix did.

## Quick triage with `reposix doctor`

Run `reposix doctor` from (or against) your working tree. It checks the common setup pitfalls — git layout, lazy-fetch config, remote URL scheme, helper on PATH, cache DB integrity, audit-table append-only triggers, env vars, [sparse-checkout](../reference/glossary.md#sparse-checkout) patterns ([git mode](https://git-scm.com/docs/git-sparse-checkout) that materializes only a subset of paths), git version, cache freshness — and prints a copy-pastable fix per finding.

```bash
reposix doctor                    # diagnose current dir
reposix doctor /tmp/repo          # diagnose another dir
reposix doctor --fix /tmp/repo    # also apply safe fixes
```

`--fix` only applies deterministic, non-destructive fixes (e.g. `git config extensions.partialClone origin`); it never mutates the cache, audit log, or backend. Exit 1 on any ERROR-severity finding, 0 otherwise — wire it into CI as a gate.

## `git push` rejected with "fetch first"

Symptom:

```text
$ git push
To reposix::http://127.0.0.1:7878/projects/demo
 ! [remote rejected] main -> main (fetch first)
error: failed to push some refs to 'reposix::http://127.0.0.1:7878/projects/demo'
hint: Updates were rejected because the remote contains work that you do
hint: not have locally. This is usually caused by another repository pushing
hint: to the same ref. You may want to first integrate the remote changes
hint: (e.g., 'git pull ...') before pushing again.
```

What it means: the backend version of an issue you are pushing moved since your last `git fetch`. Pushing now would silently overwrite the other writer.

Fix:

```bash
git pull --rebase
git push
```

`git pull --rebase` delta-syncs changed issues, replays your commit on top, then push again. On conflict, resolve with standard git tools (`git status`, edit, `git rebase --continue`).

Mechanism: see [git layer §push-time conflict detection](../how-it-works/git-layer.md#push-time-conflict-detection).

## `error: refusing to fetch <N> blobs (limit: <M>)`

Symptom:

```text
$ git grep TODO
error: refusing to fetch 487 blobs (limit: 200).
       Narrow your scope with `git sparse-checkout set <pathspec>` and retry.
```

What it means: a single `command=fetch` carried more `want` lines than `REPOSIX_BLOB_LIMIT` allows. The guardrail keeps a naive `git grep` over a 10 000-issue tree from racking up thousands of REST calls.

Fix:

```bash
git sparse-checkout set 'issues/PROJ-24*'
git checkout main
git grep TODO
```

`git sparse-checkout set <pathspec>` restricts blob materialization to matching paths. The next `git checkout` issues a smaller `command=fetch` that proceeds. Tighten the pathspec until you stay under the limit. (Use `main` — the local branch `reposix init` already checked out — not `origin/main`, which is never populated; see [git-layer §push round-trip](../how-it-works/git-layer.md).)

To raise the limit explicitly (one shot, your shell only):

```bash
REPOSIX_BLOB_LIMIT=1000 git checkout main
```

Mechanism: see [git layer §blob limit guardrail](../how-it-works/git-layer.md#blob-limit-guardrail).

## "I want to see what changed on the backend since my last fetch"

```bash
git fetch
git diff --name-only origin/main
```

`git fetch` runs an incremental delta-sync (only IDs whose `updated_at > last_fetched_at`). `git diff --name-only origin/main` lists changed files. No reposix-specific tooling required — the diff IS the change set.

If you want to see who/what changed it (subject to backend metadata):

```bash
git log --oneline origin/main ^HEAD
```

## Read the audit log

Every network operation reposix performs writes one append-only row to `audit_events_cache` in the helper-side cache DB. The DB lives at `<XDG_CACHE_HOME>/reposix/<backend>-<project>.git/cache.db` (or `<root>/reposix/<backend>-<project>.git/cache.db` when `REPOSIX_CACHE_DIR` is set).

Common queries:

```bash
# Last 5 push attempts (accepted or rejected) against sim::demo
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, reason FROM audit_events_cache \
     WHERE op LIKE 'helper_push_%' ORDER BY ts DESC LIMIT 5"

# Recent conflict rejections (the dark-factory teaching events)
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT * FROM audit_events_cache \
     WHERE op = 'helper_push_rejected_conflict' \
     ORDER BY ts DESC LIMIT 5"

# Blob-limit hits — agents who tried to materialise too many blobs at once
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, bytes, reason FROM audit_events_cache \
     WHERE op = 'blob_limit_exceeded' ORDER BY ts DESC LIMIT 5"

# Egress denials — origins blocked by REPOSIX_ALLOWED_ORIGINS
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, reason FROM audit_events_cache \
     WHERE op = 'egress_denied' ORDER BY ts DESC LIMIT 10"
```

Ops vocabulary you might see:

| `op` | Meaning |
|------|---------|
| `materialize` | Cache lazy-fetched a blob from the backend. |
| `egress_denied` | Outbound HTTP refused by `REPOSIX_ALLOWED_ORIGINS`. |
| `delta_sync` | `list_changed_since(last_fetched_at)` was run. |
| `helper_connect`, `helper_advertise`, `helper_fetch`, `helper_fetch_error` | Read-side helper protocol events. |
| `helper_push_started`, `helper_push_accepted`, `helper_push_rejected_conflict`, `helper_push_sanitized_field` | Write-side helper protocol events. |
| `blob_limit_exceeded` | A `command=fetch` carried more `want` lines than `REPOSIX_BLOB_LIMIT`. |
| `cache_gc` | A blob was evicted by `reposix gc`. |
| `token_cost` | One helper RPC turn — `chars_in` / `chars_out` packed in `reason` JSON. |

The full vocabulary and what each row means lives in [trust model §audit log](../how-it-works/trust-model.md#audit-log).

## Real-backend setup

If `git fetch` errors with `fatal: protocol error` or `Could not resolve hostname`, you're probably pointing at a real backend without the credential bundle. The three sanctioned test targets — Confluence TokenWorld, GitHub `reubenjohn/reposix`, JIRA `TEST` — each need a specific env-var pack.

See [Testing targets](../reference/testing-targets.md) for:

- The exact env-var names per backend.
- Rate-limit expectations (Atlassian's `Retry-After`, GitHub's 5000 req/hr).
- The owner's "go crazy, it's safe" permission statement for each target.
- The cleanup procedure (do not leave junk issues / pages lying around).

Most "real backend doesn't work" issues come down to one of two missing variables:

- `REPOSIX_ALLOWED_ORIGINS` excludes the backend host. Symptom: `egress_denied` audit rows.
- Credential env var unset (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY`, etc). Symptom: 401/403 surfaced as `helper_fetch_error`.

## "I have credentials but `git fetch` says missing-env" {#missing-env-with-creds}

Symptom: you set `GITHUB_TOKEN` (or the Atlassian variants) and `git fetch` still fails with a `git-remote-reposix: cannot instantiate ... backend — required env var(s) unset` message.

Common causes:

1. **`REPOSIX_ALLOWED_ORIGINS` excludes the backend host.** The default allowlist is loopback-only (sim). Real-backend `git fetch` against `https://api.github.com` or `https://<tenant>.atlassian.net` requires:

   ```bash
   export REPOSIX_ALLOWED_ORIGINS='https://api.github.com'                           # GitHub
   export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net'                # Confluence/JIRA
   # Or both (comma-separated):
   export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
   ```

   Note: `REPOSIX_ALLOWED_ORIGINS` is read at request time (not helper startup), so the failure surfaces as `Error::InvalidOrigin` on the first outbound call rather than a missing-env error.

2. **Helper started in a different shell than the one that set the env vars.** `git fetch` spawns `git-remote-reposix` as a subprocess inheriting the parent shell's env; if you set vars in one terminal and ran `git fetch` in another, the helper sees an empty environment. Check with:

   ```bash
   env | grep -E 'GITHUB_TOKEN|ATLASSIAN_|JIRA_|REPOSIX_'
   ```

3. **`/confluence/` or `/jira/` path marker missing in `remote.origin.url`.** The helper's URL-scheme dispatcher needs the marker to disambiguate Confluence and JIRA on the shared `*.atlassian.net` origin. Pre-Phase-36 `reposix init` emitted a marker-less URL; if your repo was init'd before that change, fix it with:

   ```bash
   # Confluence:
   git config remote.origin.url "reposix::https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/confluence/projects/<space>"
   # JIRA:
   git config remote.origin.url "reposix::https://${REPOSIX_JIRA_INSTANCE}.atlassian.net/jira/projects/<key>"
   ```

   `reposix doctor` flags the marker-less form as a `WARN`.

The full env-var matrix per backend is at [Testing targets](../reference/testing-targets.md), and the helper's missing-creds error message links there directly.

## Cache disk usage (`reposix gc`)

`reposix gc` evicts materialized blobs to keep disk usage bounded. Tree/commit objects, refs, and sync tags are NEVER touched — only loose blobs. Evicted blobs are transparently re-fetched on next read.

```bash
reposix gc                                       # LRU evict to 500 MB cap, current dir
reposix gc --strategy ttl --max-age-days 7       # evict blobs not touched in a week
reposix gc --strategy all --dry-run /tmp/repo    # plan, don't execute
```

Strategies:

- `--strategy lru` (default) — evict least-recently-accessed blobs first until total size drops below `--max-size-mb` (default 500).
- `--strategy ttl` — evict blobs older than `--max-age-days` (default 30) by file mtime.
- `--strategy all` — evict every loose blob; useful for "rebuild from scratch".

Each eviction (real or dry-run) appends an `op='cache_gc'` audit row carrying the evicted OID, bytes reclaimed, and the strategy slug. To inspect:

```bash
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, oid, bytes, reason FROM audit_events_cache \
     WHERE op = 'cache_gc' ORDER BY ts DESC LIMIT 10"
```

To wipe everything (audit log included), `rm -rf ~/.cache/reposix/<backend>-<project>.git`; next `reposix init` (or `git fetch`) re-creates it.

## Token-economy ledger (`reposix tokens`)

`reposix tokens` reads `op='token_cost'` audit rows — one per helper RPC turn — and prints a running token-spend summary plus an honest comparison against a conservative MCP-equivalent estimate (100k schema discovery + 5k per tool call):

```bash
reposix tokens /tmp/repo
```

The estimate is `chars / 4` over the WIRE bytes (incl. protocol-v2 framing); the MCP baseline is a back-of-envelope. <!-- banned-words: ok --> (protocol-v2 is the literal git wire format name)
Actual savings vary by workload — blob-heavy reads favour reposix; metadata-only calls favour MCP. The output is honest about that.

## DVCS push/pull issues

The v0.13.0 DVCS topology — SoT (Confluence/GitHub Issues/JIRA) plus a plain-git GH mirror — adds failure modes from the bus remote and `reposix attach`. Each entry below is a stderr message, what it means, and the recovery.

Read [DVCS topology — three roles](../concepts/dvcs-topology.md) first for the mental model; recoveries below assume you know what `refs/mirrors/<sot-host>-synced-at` is and what "mirror lag" means.

### Bus-remote `fetch first` rejection

Symptom (the bus push tripped on a SoT-side change you have not pulled):

```text
$ git push
error: confluence rejected the push (issue 0001 modified at 2026-04-30T17:30:00Z, your version 7, backend version 8)
hint: your origin (GH mirror) was last synced from confluence at 2026-04-30T17:25:00Z (5 minutes ago)
hint: run `reposix sync --reconcile` to refresh your cache against the SoT, then `git pull --rebase`
```

What it means: between your last `git fetch origin` (from the GH mirror) and your `git push`, the SoT moved. The mirror has not caught up — `refs/mirrors/<sot-host>-synced-at` shows the gap. Pushing now would silently overwrite the other writer's edits to issue 0001.

Fix (works when the conflict came from another git-side *push*):

```bash
reposix sync --reconcile          # full list_records walk against the SoT
git pull --rebase                 # replay your commits on top of the fresh state
git push                          # bus remote retries; precheck B passes
```

Why two commands: `git pull` from the GH mirror gives you the mirror's lagging view. `reposix sync --reconcile` walks the SoT directly via REST and updates your cache to match — the ground-truth refresh needed before rebasing. Once the cache is fresh, `git pull --rebase` becomes a local-only rebase and `git push` succeeds.

> **Known limitation (v0.13.x) — an EXTERNAL REST write (not a git push) breaks this recovery.**
> If the SoT moved because someone edited the record *directly* (web UI / REST PATCH)
> rather than via a reposix `git push`, the sequence above does **not** recover — and
> `reposix sync --reconcile` does not help. `git pull --rebase` aborts with:
> ```
> warning: Not updating refs/reposix/origin/main (new tip … does not contain …)
> fatal: error while running fast-import
> ```
> Root cause: the cache rebuilds a "Sync from REST snapshot" commit that is not a
> descendant of your current tracking tip, so git's fast-import refuses to advance the
> ref. This is the RBF-LR-03 deep-reconciliation limitation, scheduled for the v0.14.0
> reconciliation redesign.
>
> **Current workaround:** re-clone into a fresh tree —
> ```bash
> reposix init <backend>::<project> /tmp/fresh-tree
> cd /tmp/fresh-tree && git checkout -B main refs/reposix/origin/main
> ```
> The fresh tree reflects the external edit correctly. **You lose any local commits you
> had not yet pushed** — re-apply them by hand (e.g. copy your edited `.md`, re-commit).

On conflict, resolve with standard git tools (`git status`, edit, `git rebase --continue`).

Mechanism: the bus-remote `CHEAP PRECHECK B` runs `backend.list_changed_since(last_fetched_at)` on the SoT before reading stdin; the rejection comes from that step. See [DVCS topology — Two refs, and where they actually live](../concepts/dvcs-topology.md#two-refs-and-where-they-actually-live) for the staleness model.

### Bus-remote mirror-egress rejection (`egress-denied`)

Symptom (the bus push refused to contact the mirror host):

```text
$ git push
mirror push blocked: origin `ssh://github.com` is not authorised by REPOSIX_ALLOWED_ORIGINS.
The mirror push sends issue content over the network, so the mirror host must be on the egress
allowlist (the allowlist matches on HOST, so an `https://<host>` entry authorises an `ssh` mirror
on the same host).
To authorise it:
  export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net,https://github.com'
error: remote rejected main -> main (egress-denied)
```

What it means: a bus push (`reposix::<sot>?mirror=<mirror-url>`) shells out `git ls-remote` and `git push` against the mirror. That is a **second egress channel** — issue content leaves the machine to the mirror host — so it is gated by the same `REPOSIX_ALLOWED_ORIGINS` allowlist that guards REST calls to the SoT. The mirror host is not on your allowlist, so the push was refused **before any network contact** (no `git ls-remote`, no SoT write).

Fix: add the mirror host to the allowlist. The allowlist grammar is `http`/`https` only and the mirror gate matches on **host**, so use an `https://<host>` entry even when the mirror remote itself is `ssh` (`git@github.com:org/repo.git`):

```bash
export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net,https://github.com'
git push
```

Local mirrors (`file://…` or a filesystem path) are exempt — they perform no network egress and never trip this gate.

Note: this gate lives in the bus remote (`git-remote-reposix`). The webhook GH Action syncs the mirror with **plain `git push`** (not the bus remote), so it is unaffected by this check — its allowlist only needs the SoT tenant.

Mechanism: `bus_handler::handle_bus_export` runs the mirror-egress check (`mirror_egress::check_mirror_allowed`) before STEP 0 / PRECHECK A. See [DVCS topology — Why SoT-first for writes](../concepts/dvcs-topology.md#why-sot-first-for-writes-asymmetry-on-purpose).

### Attach reconciliation warnings

`reposix attach <backend>::<project>` walks the working-tree HEAD, matches each `*.md` file to a backend record by its frontmatter `id`, and records the alignment in the cache. The walk produces one of five outcomes per file:

| Case | What you see | Resolution |
|---|---|---|
| **match** | (silent — no warning) | Nothing to do; cache stores the OID alignment. |
| **no-id** | `NO_ID local_file=./README.md` | If the file IS supposed to be tracked, add `id: <number>` to the frontmatter and re-attach. If it is genuinely a local artifact (notes, drafts), leave it; the bus push will not propagate it. |
| **backend-deleted** | `BACKEND_DELETED id=1 local_file=issues/0001.md` (default `--orphan-policy=abort`); `... action=DELETED` for `--orphan-policy=delete-local`; `... action=FORK_AS_NEW (kept; next push creates it)` for `--orphan-policy=fork-as-new` | The record was deleted on the SoT side after your last fetch. Re-run with `reposix attach --orphan-policy=delete-local` to remove the local file, `--orphan-policy=fork-as-new` to file a new issue with the local content, or leave `--orphan-policy=abort` (default) for manual triage — none of the three abort the attach itself; duplicate-id is the only hard stop. |
| **duplicate-id** | `Error: duplicate id across local records: [(RecordId(1), ["issues/0001.md", "issues/duplicate.md"])]; reconciliation aborted (no rows committed)` | You have two local files claiming the same backend `id`. Pick one, rename or delete the other, then re-attach. This is hard-error because reconciliation cannot guess your intent — no cache rows are written for that attach run. |
| **mirror-lag** | (no per-file line; folded into the one-line summary every attach prints) `attach: matched=0 no_id=1 backend_deleted=0 mirror_lag=3` | Normal. The SoT has records the mirror has not synced yet (the staleness window). The cache notes them; your next `git fetch` will pull them in once the mirror catches up. |

If the walk fails entirely (cache initialization error, REST 401, missing credentials), the attach aborts before touching any local state — your working tree is unchanged.

Re-running `reposix attach` against the same SoT is **idempotent** (it refreshes the cache against the current backend state). Re-running against a **different** SoT is **rejected** with `working tree already attached to <existing-sot>; multi-SoT not supported in v0.13.0`. To switch SoT, run `reposix detach` first (or remove the `extensions.partialClone` config + cache directory by hand).

Mechanism: see [DVCS topology — Pattern C: Vanilla clone, then `reposix attach`](../concepts/dvcs-topology.md#pattern-c-vanilla-clone-then-reposix-attach-round-tripper).

### Webhook race conditions (`--force-with-lease` rejections)

Symptom (the webhook-driven mirror sync rejected its own push):

```text
$ gh run view <run-id> --log
... ! [rejected] main -> main (stale info)
error: failed to push some refs to 'github.com:org/<space>-mirror'
```

What it means: between the workflow's `git fetch mirror main` and its `git push --force-with-lease`, a developer's bus-remote push landed on the mirror. The lease check (`--force-with-lease=refs/heads/main:<sha-the-workflow-fetched>`) saw the mirror's `main` had moved off that SHA and refused to clobber. This is the **correct behavior**: the bus push already did the work the webhook would have done.

Fix: nothing. The workflow exits cleanly (push-step failure is caught and logged); the next webhook fire or cron tick sees a clean state. If this fires more than once an hour, bus pushes and webhook syncs are racing — increase the cron interval or rely on webhooks alone.

Why `--force-with-lease` and not plain `--force`: plain `--force` would clobber the bus-pushed commit Dev B already fetched. Their `git pull` would fast-forward back to the older SoT state and their next push would replay an outdated diff. `--force-with-lease` makes the race observable instead of silently destructive.

Mechanism: see the workflow template at [`dvcs-mirror-setup-template.yml`](dvcs-mirror-setup-template.yml) (the `Push to mirror with --force-with-lease` step) and [DVCS mirror setup → Step 4](dvcs-mirror-setup.md#step-4-smoke-test-with-a-manual-run).

### Cache-desync recovery via `reposix sync --reconcile`

Symptom: bus pushes pass the cheap precheck (`list_changed_since` empty) but writes land on stale records — you push a fix to issue 42, audit log shows `helper_push_accepted`, but the SoT still shows your old edit. Or: your cache claims an OID for a record the SoT no longer has.

What it means: your cache has drifted from the SoT. L1 conflict-detection trusts the cache as prior; a desync from a failed sync (network blip mid-fetch, manual cache mutation, race with a concurrent run) makes the bus precheck see nothing wrong because it compares against a stale prior. The fix is to re-walk the SoT and rebuild the cache.

Fix:

```bash
reposix sync --reconcile          # full list_records walk; rebuilds cache OID alignment
git fetch                         # bring in any records the cache missed
git push                          # bus push now sees fresh prior
```

`reposix sync --reconcile` is the explicit escape hatch for cache desync. **Safe to run any time** — it never mutates the SoT, only refreshes the local cache. Cost equals the pre-L1 per-push cost (one full `list_records` walk), which is why it's on-demand rather than automatic.

When to suspect cache desync (signals from the audit log):

```bash
sqlite3 ~/.cache/reposix/<backend>-<project>.git/cache.db \
    "SELECT ts, op, reason FROM audit_events_cache \
     WHERE op LIKE 'delta_sync%' ORDER BY ts DESC LIMIT 10"
```

If `delta_sync` rows returned empty over a stretch but the SoT actually moved during that window, the `last_fetched_at` cursor is wrong — `reposix sync --reconcile` rebuilds it from the SoT's current state.

Mechanism: see [DVCS topology — Cache coherence: L1 / L2 / L3](../concepts/dvcs-topology.md#cache-coherence-l1-l2-l3-adr-010) for the trade-off. L3 (transactional cache writes) is shipped — it restores the tree↔`oid_map` invariant at its source, which is what `reposix sync --reconcile` above is recovering from when it drifts. Only L2 (re-fetch-on-cache-miss) remains deferred to v0.14.0; see [ADR-010](../decisions/010-l2-l3-cache-coherence.md) for the full trade-off.

### Duplicate record after an interrupted create (real backend, v0.13.0 known limitation)

Symptom: you pushed a brand-new record (a create, not an edit) to a real backend — GitHub Issues, JIRA, or Confluence — the network dropped mid-push, and after `git push` retried you now see **two** copies of the same record on the backend (one with the id the backend assigned, one the retry created).

What it means: this is a **documented, owner-signed v0.13.0 known limitation**, not a bug you caused. A real backend assigns its own id to a new record. If the connection drops in the narrow window *after* the backend created the record but *before* your cache finished matching that new id back to your local file, the retry cannot yet recognise the already-created record as the same one, so it creates a second. The window is narrow (real backend + a create + a mid-push network drop) and there is **no data loss and no cache corruption** — only an extra record.

Confirm it from the audit log (an interrupted create leaves a `helper_push_started` row with no matching `helper_push_accepted` for the same push):

```bash
sqlite3 ~/.cache/reposix/<backend>-<project>.git/cache.db \
    "SELECT ts, op, reason FROM audit_events_cache \
     WHERE op LIKE 'helper_push_%' ORDER BY ts DESC LIMIT 5"
```

A `helper_push_started` row whose push never reached a `helper_push_accepted` (the accept row that a clean push writes on success) is the fingerprint of the interrupted create — the push began, the backend created the record, but the connection dropped before the accept landed and matched the new id back. The retry then shows as its own `started` → `accepted` pair against the duplicate.

Fix (once you notice the duplicate, before pushing anything further):

Do not push again until you've resolved the duplicate below — a second push on top of an unreconciled cache can create a third copy.

```bash
# 1. Look on the backend for two records with identical title/body.
#    (GitHub: the Issues list; JIRA: the project board; Confluence: the space.)
# 2. Keep the one you intend to track; hand-delete the duplicate on the backend UI.
# 3. Re-fetch so your cache matches the backend's surviving record:
reposix sync --reconcile
git fetch
```

Why hand-delete: reposix will not guess which of the two copies you meant to keep, so recovery is a deliberate one-click delete on the backend, not an automatic merge. Once the duplicate is gone and `reposix sync --reconcile` has refreshed the cache, your working tree and the backend agree again.

The clean fix is a v0.14.0 milestone: a redesign that models a create as a durable slug→id translation so an interrupted create leaves a well-defined state to continue instead of a duplicate. See [ADR-010 §3 — `SotPartialFail` recovery](../decisions/010-l2-l3-cache-coherence.md) for the full known-limitation marker and the pivot it points to.

## See also

- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — when an error message stops making sense, re-read this; the three keys are the cheat sheet.
- [DVCS topology — three roles](../concepts/dvcs-topology.md) — the mental model behind the bus remote, mirror lag, and `reposix attach`.
- [DVCS mirror setup](dvcs-mirror-setup.md) — the owner's walk-through for installing the webhook + GH Action.
- [How it works — git layer](../how-it-works/git-layer.md) — push round-trip, blob limit, conflict detection.
- [How it works — trust model](../how-it-works/trust-model.md) — the audit log and what every `op` means.
- [Testing targets](../reference/testing-targets.md) — env vars and permission statements for the three real-backend targets.
- [First-run tutorial](../tutorials/first-run.md) — the seven-step happy path; useful as a sanity check when something feels off.
