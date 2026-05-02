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
git checkout origin/main
git grep TODO
```

`git sparse-checkout set <pathspec>` restricts blob materialization to matching paths. The next `git checkout` issues a smaller `command=fetch` that proceeds. Tighten the pathspec until you stay under the limit.

To raise the limit explicitly (one shot, your shell only):

```bash
REPOSIX_BLOB_LIMIT=1000 git checkout origin/main
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
    "SELECT ts, op, decision, reason FROM audit_events_cache \
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

Fix:

```bash
reposix sync --reconcile          # full list_records walk against the SoT
git pull --rebase                 # replay your commits on top of the fresh state
git push                          # bus remote retries; precheck B passes
```

Why two commands: `git pull` from the GH mirror gives you the mirror's lagging view. `reposix sync --reconcile` walks the SoT directly via REST and updates your cache to match — the ground-truth refresh needed before rebasing. Once the cache is fresh, `git pull --rebase` becomes a local-only rebase and `git push` succeeds.

On conflict, resolve with standard git tools (`git status`, edit, `git rebase --continue`).

Mechanism: the bus-remote `CHEAP PRECHECK B` runs `backend.list_changed_since(last_fetched_at)` on the SoT before reading stdin; the rejection comes from that step. See [DVCS topology — Two refs you can `git log`](../concepts/dvcs-topology.md#two-refs-you-can-git-log) for the staleness model.

### Attach reconciliation warnings

`reposix attach <backend>::<project>` walks the working-tree HEAD, matches each `*.md` file to a backend record by its frontmatter `id`, and records the alignment in the cache. The walk produces one of five outcomes per file:

| Case | What you see | Resolution |
|---|---|---|
| **match** | (silent — no warning) | Nothing to do; cache stores the OID alignment. |
| **no-id** | `WARN: issues/x.md has no 'id' field — skipping (not a reposix-managed file)` | If the file IS supposed to be tracked, add `id: <number>` to the frontmatter and re-attach. If it is genuinely a local artifact (notes, drafts), leave it; the bus push will not propagate it. |
| **backend-deleted** | `WARN: issues/0001.md claims id: 1 but no backend record exists — skipping` | The record was deleted on the SoT side after your last fetch. Re-run with `reposix attach --orphan-policy=delete-local` to remove the local file, `--orphan-policy=fork-as-new` to file a new issue with the local content, or `--orphan-policy=abort` (default) to leave it for manual triage. |
| **duplicate-id** | `ERROR: id: 1 claimed by both issues/0001.md and issues/duplicate.md — refusing to attach` | You have two local files claiming the same backend `id`. Pick one, rename or delete the other, then re-attach. This is hard-error because reconciliation cannot guess your intent. |
| **mirror-lag** | (no warning per file; one summary line) `INFO: backend has 3 records not yet in the mirror; cache marks for next fetch` | Normal. The SoT has records the mirror has not synced yet (the staleness window). The cache notes them; your next `git fetch` will pull them in once the mirror catches up. |

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
    "SELECT ts, op, decision, reason FROM audit_events_cache \
     WHERE op LIKE 'delta_sync%' ORDER BY ts DESC LIMIT 10"
```

If `delta_sync` rows returned empty over a stretch but the SoT actually moved during that window, the `last_fetched_at` cursor is wrong — `reposix sync --reconcile` rebuilds it from the SoT's current state.

Mechanism: see [DVCS topology — Out of scope](../concepts/dvcs-topology.md#out-of-scope-intentionally) for the L1/L2/L3 trade-off; L2 hardening (background reconcile job) and L3 (transactional cache writes) defer to v0.14.0.

## See also

- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — when an error message stops making sense, re-read this; the three keys are the cheat sheet.
- [DVCS topology — three roles](../concepts/dvcs-topology.md) — the mental model behind the bus remote, mirror lag, and `reposix attach`.
- [DVCS mirror setup](dvcs-mirror-setup.md) — the owner's walk-through for installing the webhook + GH Action.
- [How it works — git layer](../how-it-works/git-layer.md) — push round-trip, blob limit, conflict detection.
- [How it works — trust model](../how-it-works/trust-model.md) — the audit log and what every `op` means.
- [Testing targets](../reference/testing-targets.md) — env vars and permission statements for the three real-backend targets.
- [First-run tutorial](../tutorials/first-run.md) — the seven-step happy path; useful as a sanity check when something feels off.
