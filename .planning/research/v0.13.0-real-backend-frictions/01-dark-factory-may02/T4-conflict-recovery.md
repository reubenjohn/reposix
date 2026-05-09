# T4 — Push-Time Conflict Detection + git pull --rebase Recovery

**Tester:** Naive dark-factory subagent. Never used reposix before.
**Backend:** Sim (sim::demo) at http://127.0.0.1:7878
**Workdirs:** /tmp/conflict-A and /tmp/conflict-B
**Date:** 2026-05-02

## What I expected (from docs)

`docs/guides/troubleshooting.md` § "git push rejected with 'fetch first'":

> Symptom: `! [remote rejected] main -> main (fetch first)` … plus the standard git hint block.
> Fix: `git pull --rebase && git push`. On conflict, resolve with standard git tools.

`docs/index.md` § "What it looks like underneath":

> push-time conflict detection rejects stale-base pushes with the standard git "fetch first" error so an agent recovers via `git pull --rebase`.

So my expectation: I edit issue 0001 in B with a stale base, push, see a recognisable git rejection, run `git pull --rebase`, edit if needed, push again, both edits land.

## Step-by-step log

### Step 1 — Read docs

Read `docs/guides/troubleshooting.md` § "git push rejected with 'fetch first'" + `docs/index.md` § "What it looks like underneath". Both promise the standard `[rejected] main -> main (fetch first)` git error and `git pull --rebase && git push` recovery.

No frictions reading the docs; the recovery contract is named in two places, consistent.

### Step 2 — Bootstrap two checkouts

Ran `reposix init sim::demo /tmp/conflict-A` and same for B.

```
WARN reposix_cli::init: git fetch --filter=blob:none failed with status exit status: 128 — local repo is configured but not yet synced. Stderr: fatal: could not read ref refs/reposix/main
reposix init: configured `/tmp/conflict-A` with remote.origin.url = reposix::http://127.0.0.1:7878/projects/demo
Next: cd /tmp/conflict-A && git checkout origin/main (or git sparse-checkout set <pathspec> first)
```

**Friction MED-1 (init WARN):** init prints a `WARN` about `fatal: could not read ref refs/reposix/main` BUT then says "configured … Next: cd … && git checkout origin/main" as if everything is fine. Two issues:

- The error references `refs/reposix/main` but the ref the helper actually creates is `refs/reposix/origin/main` (note the `/origin/`). So the warning's pathname is misleading.
- More confusingly, `git checkout origin/main` (the literal "Next" hint) does NOT work because `origin/main` is not the ref name — `refs/reposix/origin/main` is. Following the printed hint verbatim fails.

The README/docs use `git checkout -B main refs/reposix/origin/main` which is correct, so I went with that and it worked. But a naive agent reading just the init output would be misled.

`git checkout -B main refs/reposix/origin/main` worked in both A and B; both landed on the same `1918c6f Sync from REST snapshot` commit. **6 issue files (0001.md … 0006.md) materialized in each.** Step succeeded.

### Step 3 — Edit + push from A (baseline)

Edited `0001.md` in A: changed `title:` to `A-edit-conflict-test` and appended a marker line. Committed with `git commit -m "A: title change + appended marker"`. Ran `git push origin main`.

```
WARN git_remote_reposix: cache unavailable for push audit: open reposix-cache: git: git config --add transfer.hideRefs failed: fatal: not in a git directory
To reposix::http://127.0.0.1:7878/projects/demo
 * [new branch]      main -> main
```

**Friction MED-2 (cache audit WARN on every push):** every `git push` prints `WARN git_remote_reposix: cache unavailable for push audit: open reposix-cache: git: git config --add transfer.hideRefs failed: fatal: not in a git directory`. This says the audit-log writer can't open the cache — meaning OP-3 ("audit log is non-optional") is violated for this push. The push itself succeeds, but the audit row that should land in `audit_events_cache` apparently doesn't. The threat-model claim "every network operation reposix performs writes one append-only row" silently regresses for normal end-user pushes from a partial-clone working tree.

Push from A succeeded. Verified with `reposix list --project demo` — issue 1 now has `version: 3` and the appended marker line in body.

### Step 4 — Edit + commit + push from B (stale base)

In B (still at `1918c6f`, doesn't know A pushed):

- Changed `title:` to `B-edit-conflict-test`, appended marker line.
- `git commit -m "B: title change + appended marker"` → success.
- `git push origin main`:

```
WARN git_remote_reposix: cache unavailable for push audit: ...
issue 1 modified on backend at 2026-05-02T10:19:25+00:00 since last fetch (local base version: 2, backend version: 3). Run: git pull --rebase
To reposix::http://127.0.0.1:7878/projects/demo
 ! [rejected]        main -> main (fetch first)
error: failed to push some refs to 'reposix::http://127.0.0.1:7878/projects/demo'
hint: Updates were rejected because the remote contains work that you do
hint: not have locally. ...
```

Exit code: **1** (correct).

**This is the contract: a recognisable git rejection plus a custom helper line that names the issue, version delta, and the recovery command.** The helper's diagnostic "Run: git pull --rebase" is exactly the agent-friendly stderr promise the project makes. WIN.

### Step 5 — Observe rejection (analysis)

Format matches `docs/guides/troubleshooting.md` § "fetch first" almost exactly. The custom helper line above the standard `[rejected]` line is helpful (gives version numbers, points to recovery command). The standard git hint block ("Updates were rejected because…") follows. An agent grep-ing for `[rejected]` or `fetch first` finds them.

**No friction here — this is the win the architecture promises.**

### Step 6 — Recovery via `git pull --rebase`

```
$ git pull --rebase origin main
warning: Not updating refs/reposix/origin/main (new tip 97b078edc8d4760a897fd3553a34534ff618b9c1 does not contain 1918c6f1219710054c297e1fce50d64501ec05c6)
fatal: error while running fast-import
```

**Friction HIGH-1 (documented recovery path FAILS):** `git pull --rebase` — the COMMAND THE HELPER ITSELF JUST TOLD ME TO RUN — fails with `fatal: error while running fast-import`. This is the failure mode of a non-fast-forward fetch: the helper rebuilds the bare-repo cache from REST and produces a NEW root commit `97b078e` that has no ancestry relationship to B's current `refs/reposix/origin/main` at `1918c6f`. `git fetch`'s fast-forward check refuses, and even if it accepted via `--force`, the rebase would have nothing to rebase onto.

I tried recovery moves the message did not name (because the message is from fast-import, not the helper):

- `git fetch origin` alone → same `fatal: error while running fast-import`. Plain fetch breaks too — not a rebase-specific issue.
- `git fetch --force origin` → same error. The error is from fast-import internals, not from ref-update.
- `git fetch` from checkout A (which JUST successfully pushed) → same error. So this is not B-specific; ANY second fetch against the helper after a push has happened breaks.

This means: **after the very first `git fetch` post-init, any subsequent `git fetch` is broken on the sim backend.** The helper appears to mint commits per-fetch from scratch (snapshot-style) rather than building on the previous ref tip.

Recovery moves I'd consider but the agent UX promise rules out (agent shouldn't need reposix-specific knowledge):

- `reposix refresh` — works as a heavyweight "rebuild + commit" but is a reposix-specific command, not standard git. Also it commits the rebuild on top of HEAD which would obliterate B's local edit, not rebase it.
- `reposix sync --reconcile` — listed in troubleshooting but only for the DVCS bus-remote case (v0.13.0 P82+); shouldn't be needed for a single-SoT push conflict in v0.12.0.
- Manually deleting `refs/reposix/origin/main` and re-fetching — this is reposix-internal git surgery, not the documented contract.

**Severity HIGH because:** the entire load-bearing claim of v0.9.0 — "agent recovers via `git pull --rebase`" — does not work on the sim backend in the simplest possible two-writer scenario. The error stack ends with `fatal: error while running fast-import` which (a) doesn't name a recovery move, (b) is generated by upstream git, not the reposix helper, so the substrate-property promise "stderr names the next command" breaks here.

### Step 7 — Verify both pushes landed

Did NOT reach this step: B is stuck at HIGH-1 (cannot fetch / pull --rebase against the helper after A's push). Sim REST shows only A's edit landed (version 3, A's marker line). B's edit is still local-only on commit `af67f67` and unpushable until the fetch path is unblocked.

A second curl confirms the simulator's view:

```
$ reposix list --project demo
... id: 1, version: 3, title: "A-edit-conflict-test", body ends with "Edit from A at 2026-05-02T03:19:22-07:00"
```

B's edit (`Edit from B at …`) is absent.

**Severity HIGH for this step too:** the claim "both writers' edits land via rebase" is not demonstrable end-to-end against the sim with the documented commands.

## Friction tally

| ID | Severity | What |
|---|---|---|
| MED-1 | MED | `reposix init` prints a confusing `WARN` about `fatal: could not read ref refs/reposix/main` then says "Next: cd … && git checkout origin/main" — the warning's ref name is wrong (it's `refs/reposix/origin/main`) AND the "Next" hint command (`git checkout origin/main`) doesn't work; the README's `git checkout -B main refs/reposix/origin/main` does. Init's success message and its hint contradict each other. |
| MED-2 | MED→HIGH on review | Every `git push` prints `WARN cache unavailable for push audit: ... fatal: not in a git directory`. Audit row is silently NOT written for end-user pushes from a partial-clone working tree. OP-3 ("audit log non-optional") regresses. **Verified after the fact**: `sqlite3 ~/.cache/reposix/sim-demo.git/cache.db 'SELECT op, ts FROM audit_events_cache ORDER BY ts DESC LIMIT 15'` returned exactly ONE row, an `egress_denied` from 14 minutes BEFORE our two pushes — zero `helper_push_started/accepted/rejected_conflict` rows for THIS test's pushes. The audit log is not just patchy, it's dark for the entire push-side flow. The CLAUDE.md threat model treats this as a release blocker; arguably HIGH not MED. The `git config --add transfer.hideRefs` command itself works fine when I run it manually in either the working tree or the cache bare repo, so this is a helper code-path bug (wrong cwd? wrong gitdir resolution?), not a missing tool. |
| WIN | — | Push rejection on stale base IS the documented `[rejected] main -> main (fetch first)` plus a helpful custom line naming issue id, version delta, and recovery command. Exit 1. This part of the contract works exactly as advertised. |
| HIGH-1 | HIGH | `git pull --rebase` (the recovery command the helper itself names) fails with `fatal: error while running fast-import` because the helper produces a new root commit per fetch with no ancestry to the prior ref tip. `git fetch --force` doesn't help (error is from fast-import internals, not ref-update). Affects ALL subsequent fetches after any push, in any checkout — not B-specific. The load-bearing v0.9.0 architecture claim "agent recovers via `git pull --rebase`" does not work end-to-end on the sim. The rebase recovery for two-writer conflict is unreachable; step 7 (both edits landed) is unreachable. |

## Boxes ticked

| Box | Status |
|---|---|
| 1. Read docs | DONE |
| 2. Bootstrap two checkouts | DONE (with MED-1 caveat) |
| 3. A edits + pushes (baseline) | DONE (push lands; MED-2 audit silently lost) |
| 4. B edits with stale base + push | DONE (push correctly rejected) |
| 5. Observe rejection | DONE — matches docs |
| 6. Recover via `git pull --rebase` | **FAILED** — fast-import error |
| 7. Verify both pushes landed | **NOT REACHED** — blocked on step 6 |

**5 of 7.**

## Honest take

The rejection-detection half of the contract works perfectly: B's stale-base push gets the textbook git `[rejected] main -> main (fetch first)` plus a custom helper line that names the conflict (issue 1, version 2 → 3) and the exact recovery command. That's a wonderful agent-UX moment.

The recovery half is broken on the sim. Running the helper's own named recovery (`git pull --rebase`) hits `fatal: error while running fast-import` because every helper fetch produces a fresh root commit with no ancestry to the prior `refs/reposix/origin/main`. So `git fetch` alone fails with non-fast-forward; `git pull --rebase` cascades the same fast-import error. `--force` doesn't help (the error is internal to fast-import). The architectural cornerstone — "two writers race, loser sees a familiar error and recovers with familiar commands" — recovers from the rejection on the sim by being unable to recover at all.

If this is a sim-only bug (the cache-rebuild path producing per-fetch root commits is idiomatic of an early sim implementation), the dark-factory regression test should be exercising exactly this flow on every CI run. If the dark-factory regression is currently passing while this scenario fails locally with prebuilt binaries, the regression isn't covering what it claims to cover.

The MED-2 audit-log silence on push is a separate, smaller concern but worth noting because OP-3 forbids "feature isn't done" outcomes from missing audit rows. Two consecutive pushes in this test produced zero `helper_push_*` audit events that I could verify (cache wasn't even openable due to the `transfer.hideRefs` config error).
