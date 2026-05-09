# T2 — reposix attach (real Confluence)

**Subagent:** fresh-eyes (no prior reposix context)
**Date:** 2026-05-02
**Goal:** clone vanilla mirror, `reposix attach confluence::REPOSIX`, edit, push, verify.

## Step-by-step log

### Step 1: read user-facing docs about `attach`

- **README.md**: Mentions `reposix init` and the simulator quickstart. **Does NOT mention `reposix attach` at all.** The "Quick start (5 min)" only walks through `init`. As a fresh user landing on the README, I would have no idea `attach` is even a thing.
- **`docs/concepts/dvcs-topology.md`**: This is where I found `attach`. Pattern C (round-tripper) gives me exactly the recipe I want:
  ```bash
  cd /tmp/issues
  $EDITOR issues/0001.md && git commit -am 'fix typo'
  cargo binstall reposix-cli
  reposix attach confluence::SPACE
  git push
  ```
  Clear, brief. Good — but assumes I knew to land on this concept page. The README doesn't link there from any "I have a vanilla clone, what now?" entry point.
- **`docs/guides/troubleshooting.md`** § "Attach reconciliation outcomes": Lists the 5 reconciliation cases (match, no-id, backend-deleted, duplicate-id, mirror-lag) clearly. Good reference, but it's in the troubleshooting section — a successful happy-path user would not naturally read this until something breaks.
- **`docs/guides/dvcs-mirror-setup.md`**: Owner-side walk-through for setting up the GH mirror. Mentions `attach` in passing but is not a user-facing onboarding doc.
- **`docs/tutorials/first-run.md`**: Did NOT check (tutorial is for `init`, not `attach`).
- **`reposix attach --help`**: Decent. Examples show three forms. Lists all flags with defaults. Good.

#### Findings from Step 1

- **F1 (MED)**: README has zero mention of `attach`. The whole "I already have a vanilla clone of the GH mirror, can I write back?" use case is absent from the front-door surface. README → `docs/concepts/dvcs-topology.md` is the only path, and I only got there because I `grep`d for "attach". A new user reading top-down would never click "DVCS topology — three roles" looking for the answer to "how do I push from my vanilla clone?".
- **F2 (LOW)**: There's no `docs/tutorials/attach.md` parallel to `first-run.md`. Pattern C in the concept page IS a tutorial-shaped block, but tutorials live in tutorials/.
- **F3 (LOW)**: `--help` short-form (`-h`) for `reposix attach` says "see a summary with '-h'" inside the long form — circular wording (the long form is shown by `--help`; the short by `-h`). Minor.

### Step 2: clone vanilla mirror

```
$ git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git /tmp/attach-test
Cloning into '/tmp/attach-test'...
$ ls /tmp/attach-test
README.md
$ find /tmp/attach-test -type f -not -path '*/.git/*'
/tmp/attach-test/README.md
/tmp/attach-test/.github/workflows/reposix-mirror-sync.yml
```

The vanilla mirror clone has only `README.md` and `.github/workflows/reposix-mirror-sync.yml` — no `pages/` directory, no markdown content for any Confluence pages. The mirror has never been synced (or was synced when the SoT was empty).

The mirror's README says: "Plain-git mirror of Confluence TokenWorld space — used by reposix v0.13.0 DVCS tests (dark-factory third arm + webhook-driven sync). Auto-managed by reposix; do not commit directly."

#### Findings from Step 2

- **F4 (MED)**: The vanilla mirror has no content. Pattern C in `dvcs-topology.md` describes the use case as *"Already have a vanilla clone: cd /tmp/issues; $EDITOR issues/0001.md && git commit -am 'fix typo'"* — the example assumes the mirror has actual `issues/0001.md` content to edit. With this real mirror, there's nothing to edit before attach. A fresh user would be confused: "is the mirror broken? do I need to wait for the cron? do I attach first and let attach populate the working tree?". The docs do not say what to do when the mirror is empty.
- **F5 (LOW)**: Mirror's README says "do not commit directly." Pattern C in the concept page tells me to commit *before* attach. The two messages collide for a user who reads the mirror README first. Consider clarifying on the mirror README: "do not push directly via vanilla git; bus pushes via `reposix attach` are fine."

### Step 3: attach to confluence::REPOSIX — BLOCKED

```
$ cd /tmp/attach-test
$ reposix attach confluence::REPOSIX
Error: attach: backend `confluence` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03
$ echo $?
1
```

Sanity-checked the other backends:

```
$ reposix attach github::REPOSIX
Error: attach: backend `github` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03

$ reposix attach jira::REPOSIX
Error: REPOSIX_JIRA_INSTANCE must be set for `jira::<key>` (subdomain of your Atlassian Cloud tenant)
# (likely DOES check for env vars — i.e. jira IS wired but I haven't set the instance var; not pursued further)

$ reposix attach sim::demo
Error: build cache from backend
Caused by: egress denied: blocked origin: http://127.0.0.1:7878/projects/demo/issues
# (sim works in principle but the allowlist I set for confluence blocks it; expected)
```

Sanity-checked that the confluence backend itself works for read paths:

```
$ reposix list --backend confluence --project REPOSIX | head
[ { "id": 65916, "title": "Architecture notes", ... }, ... ]
```

So **the backend works**, but **`reposix attach` against confluence is not wired** in this build. The `--help` output for `attach` advertises `reposix attach confluence::SPACE` as a valid invocation, and `dvcs-topology.md` Pattern C ("the v0.13.0 thesis path") tells me to do exactly this. The error message names internal phase IDs (`P79-02`, `P79-03`) which mean nothing to a user.

#### Findings from Step 3

- **F6 (HIGH)**: **`reposix attach confluence::*` errors out in this build** with `backend "confluence" not yet wired in P79-02 scaffold (sim only)`. The docs say it works (`docs/concepts/dvcs-topology.md` Pattern C; `reposix attach --help` examples list `confluence::SPACE`). Reality says no. This is the canonical doc-vs-reality HIGH bug — docs promise a feature the binary cannot deliver. Same applies to `github::*`. Only `sim::*` and (presumably, given the different error class) `jira::*` are wired. A new user reading Pattern C would be stopped cold here with no path forward — the error names internal planning phase IDs (`P79-02`, `P79-03`) which leak implementation details and offer zero recovery hint.
- **F7 (HIGH)**: Error message leaks internal jargon. "P79-02 scaffold" / "P79-03" are GSD planning phase identifiers. A user has no way to know what those mean, when P79-03 ships, or whether they should wait, file a bug, or use a workaround. Compare to a useful version: `attach: backend "confluence" is not yet supported in this release. Tracking: <github issue url>. Workaround: use \`reposix init confluence::<SPACE> <path>\` to create a fresh partial-clone working tree (does not reuse your vanilla mirror).`
- **F8 (MED)**: `reposix attach --help` advertises confluence/jira/github via examples, but the binary only supports sim. Either the help text should warn that only sim is wired in this release, OR the binary should support what the help advertises. Mismatch between `--help` and behavior is a docs bug.
- **F9 (LOW)**: The error returned via anyhow `Error: ...` does not include a "Caused by" chain like the sim error does. For a "feature not implemented" assertion that's reasonable, but consistency would help.

### Steps 4–6: SKIPPED — blocked on F6

Cannot edit / push / verify against Confluence because `reposix attach confluence::REPOSIX` cannot complete. Recovery moves considered but not pursued (per the "one recovery move" budget):

- **`reposix init confluence::REPOSIX /tmp/init-test`** — would create a fresh partial-clone tree, NOT reuse the vanilla mirror. This is Pattern B from `dvcs-topology.md`, not Pattern C. Different code path, different test surface; T1 / T3 may already cover it.
- **Switch to `jira::TEST`** — out of scope (T2 is explicitly the Confluence attach run).
- **Switch to `sim::demo`** — out of scope (T2 is explicitly real-backend, and would also need a sim simulator running, which I'd have to start, etc.).

## Findings summary

| # | Severity | What | Evidence |
|---|---|---|---|
| F6 | HIGH | `reposix attach confluence::*` errors out (also `github::*`) — docs say it works, binary says "not yet wired in P79-02 scaffold (sim only)". The thesis path of the v0.13.0 release does not function against the documented backends. | `reposix attach confluence::REPOSIX` exit=1 with `Error: attach: backend "confluence" not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03` |
| F7 | HIGH | Error message leaks internal planning-phase IDs (`P79-02`, `P79-03`) and offers no recovery hint. User has no way to know what to do next. | Same stderr line as F6. |
| F1 | MED | README has zero mention of `reposix attach`. The "I have a vanilla clone, can I write back?" use-case has no front-door entry point in the README. | `grep -i attach /home/reuben/workspace/reposix/README.md` returns nothing. |
| F4 | MED | The vanilla mirror clone has no `pages/` content (only README.md + workflow file). Pattern C in `dvcs-topology.md` describes editing `issues/0001.md` before attach, but a real fresh clone has nothing to edit. Docs do not address the empty-mirror state. | `find /tmp/attach-test -type f -not -path '*/.git/*'` returns 2 files only. |
| F8 | MED | `reposix attach --help` examples advertise `confluence::SPACE` as a valid invocation; binary refuses with F6. Help and behavior disagree. | `reposix attach --help` and the F6 error. |
| F2 | LOW | No `docs/tutorials/attach.md`. Pattern C is documented in a *concept* page, not a *tutorial*. Diátaxis-shape mismatch. | `ls docs/tutorials/` shows only `first-run.md`. |
| F3 | LOW | `reposix attach -h` long-form ends with "see a summary with '-h'" — circular. | `reposix attach --help` last line. |
| F5 | LOW | Mirror README says "do not commit directly" — collides with Pattern C's "$EDITOR; git commit -am" before attach. Needs a clarifying clause. | `/tmp/attach-test/README.md` line 2. |
| F9 | LOW | Anyhow error from attach lacks a "Caused by" chain that other reposix errors have. | F6 error vs sim attach error format. |

## Goal outcome

- [x] Vanilla clone obtained
- [ ] `reposix attach` ran and modified git config (BLOCKED — F6: confluence backend not wired)
- [ ] Edit + commit succeeded (BLOCKED — depends on attach)
- [ ] `git push` succeeded (BLOCKED — depends on attach)
- [ ] Server-side change confirmed via REST (BLOCKED — depends on push)

**1 of 5 ticked.**

## Honest assessment

A fresh user could not do this from the docs alone, because the binary in this build literally errors out on the documented Pattern C invocation (`reposix attach confluence::SPACE`). The docs are written for a feature that is not yet in the shipped binary — a textbook docs-vs-reality HIGH bug, made worse by an error message that leaks internal phase IDs and gives no workaround. Even if attach *did* work, the README has no entry point for the "I have a vanilla clone, can I write back?" question, and the actual mirror is empty (no markdown content), which Pattern C's example does not address. The conceptual writing in `dvcs-topology.md` is good — but it describes a v0.13.0 that the user-facing binary at `target/release/reposix` cannot yet deliver against real backends.

