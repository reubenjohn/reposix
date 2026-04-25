---
title: Troubleshooting
---

# Troubleshooting

When reposix's substrate property holds, an agent recovers from every error reposix can produce by reading the stderr message and following its instructions verbatim. This page is for the cases where you (or your agent) need a slightly bigger hint than the stderr alone provides — and for the diagnostic queries you'd run when nothing is broken but you want to know what reposix did.

## Quick triage with `reposix doctor`

When something feels off, run `reposix doctor` from inside (or against) your reposix working tree. It checks the most common setup pitfalls — git repo layout, lazy-fetch config, remote URL scheme, helper binary on PATH, cache DB integrity, audit-table append-only triggers, env vars, sparse-checkout patterns, git version, and cache freshness — and prints a copy-pastable fix command for each finding.

```bash
reposix doctor                    # diagnose current dir
reposix doctor /tmp/repo          # diagnose another dir
reposix doctor --fix /tmp/repo    # also apply safe fixes
```

`--fix` only applies deterministic, non-destructive fixes (e.g. `git config extensions.partialClone origin`). It will never mutate the cache, the audit log, or the backend. Exit code is 1 if any ERROR-severity finding is reported, 0 otherwise — so you can wire `reposix doctor` into CI as a gate.

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

What it means: the helper noticed that the backend version of an issue you are pushing has moved since your last `git fetch`. Pushing now would silently overwrite the other writer.

Fix:

```bash
git pull --rebase
git push
```

`git pull --rebase` runs a delta-sync of the changed issues into your working tree, replays your commit on top, and you push again. If the rebase produces a conflict, resolve it with the standard git tools (`git status`, edit, `git rebase --continue`).

Mechanism: see [git layer §push-time conflict detection](../how-it-works/git-layer.md#push-time-conflict-detection).

## `error: refusing to fetch <N> blobs (limit: <M>)`

Symptom:

```text
$ git grep TODO
error: refusing to fetch 487 blobs (limit: 200).
       Narrow your scope with `git sparse-checkout set <pathspec>` and retry.
```

What it means: the helper counted more `want` lines on a single `command=fetch` request than `REPOSIX_BLOB_LIMIT` allows. This is the guardrail that keeps a naive `git grep` over a 10 000-issue tree from racking up thousands of REST calls.

Fix:

```bash
git sparse-checkout set 'issues/PROJ-24*'
git checkout origin/main
git grep TODO
```

`git sparse-checkout set <pathspec>` tells git to only materialize blobs matching the pathspec. The next `git checkout` issues a much smaller `command=fetch` and the operation proceeds. Tighten the pathspec until you stay under the limit.

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

`git fetch` runs a delta-sync against the backend (incremental — only IDs whose `updated_at > last_fetched_at`). `git diff --name-only origin/main` lists changed files. No reposix-specific tooling required; the diff IS the change set.

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

The full vocabulary and what each row means lives in [trust model §audit log](../how-it-works/trust-model.md#audit-log).

## Real-backend setup

If `git fetch` errors with `fatal: protocol error` or `Could not resolve hostname`, you are probably pointing at a real backend without the credential bundle. The three sanctioned test targets — Confluence TokenWorld, GitHub `reubenjohn/reposix`, JIRA `TEST` — each need a specific env-var pack.

See [Testing targets](../reference/testing-targets.md) for:

- The exact env-var names per backend.
- Rate-limit expectations (Atlassian's `Retry-After`, GitHub's 5000 req/hr).
- The owner's "go crazy, it's safe" permission statement for each target.
- The cleanup procedure (do not leave junk issues / pages lying around).

Most "real backend doesn't work" issues come down to one of two missing variables:

- `REPOSIX_ALLOWED_ORIGINS` not including the backend host. Symptom: `egress_denied` audit rows.
- A credential env var unset (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY`, etc). Symptom: 401/403 from the REST call surfaced as `helper_fetch_error`.

## Cache eviction (`reposix gc`)

Coming in v0.13.0. The cache currently grows monotonically — there is no LRU, no TTL, no manual `reposix gc`. If your cache directory is using uncomfortable disk:

```bash
rm -rf ~/.cache/reposix/<backend>-<project>.git
```

The next `reposix init` (or `git fetch`) re-creates it. You lose the helper-side audit log when you do this — if that history matters, copy `cache.db` somewhere first.

## See also

- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — when an error message stops making sense, re-read this; the three keys are the cheat sheet.
- [How it works — git layer](../how-it-works/git-layer.md) — push round-trip, blob limit, conflict detection.
- [How it works — trust model](../how-it-works/trust-model.md) — the audit log and what every `op` means.
- [Testing targets](../reference/testing-targets.md) — env vars and permission statements for the three real-backend targets.
- [First-run tutorial](../tutorials/first-run.md) — the seven-step happy path; useful as a sanity check when something feels off.
