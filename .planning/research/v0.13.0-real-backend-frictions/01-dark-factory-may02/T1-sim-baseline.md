# T1 — Sim end-to-end dark-factory test

**Subagent:** fresh-eyes (no prior reposix context)
**Date:** 2026-05-02
**Goal:** find/edit/push an issue via the simulator following user-facing docs only.

## Step-by-step log

### Step 1: read README.md

- What the doc told me to do: README "Quick start (5 min)" gives an explicit recipe:
  ```bash
  reposix sim --bind 127.0.0.1:7878 &
  reposix init sim::demo /tmp/reposix-demo
  cd /tmp/reposix-demo
  git checkout -B main refs/reposix/origin/main
  ls issues/
  sed -i 's/TODO/DONE/' issues/0001.md
  git commit -am 'mark issue 1 done'
  git push
  ```
- Doc also points to `docs/tutorials/first-run.md` for the "full walkthrough."
- What I observed (before doing anything):
  - The Quick-start tells me to run `git checkout -B main refs/reposix/origin/main`. This is unusual — most readers would expect `git checkout main` after a normal clone. The phrase "helper namespaces fetched refs" hints why, but a fresh agent won't know what that means until they try it.
  - `reposix init` is the bootstrap. The simulator is already running here, so I can skip step 1.
- Friction (so far): LOW — `git checkout -B main refs/reposix/origin/main` looks like an artifact leaking through from the helper's refspec namespace. Not broken, just jargony.

### Step 2: bootstrap with `reposix init`

- What the doc told me to do: `reposix init sim::demo /tmp/sim-darkfactory`.
- What I did: ran it (sim already up at :7878).
- What I observed:
  - Stderr WARN: `git fetch --filter=blob:none failed with status exit status: 128 — local repo is configured but not yet synced. Stderr: fatal: could not read ref refs/reposix/main`. The init still printed a "configured" success line and a "Next:" hint, so a fresh user would have no idea this WARN means "ignore me" vs "you're broken." Looking at refs after, `refs/reposix/origin/main` IS populated, so the fetch did work, just one ref look-up failed.
  - Init's own "Next:" line says: `cd /tmp/sim-darkfactory && git checkout origin/main (or git sparse-checkout set <pathspec> first)`. This contradicts the README quick-start AND `docs/tutorials/first-run.md` step 4, both of which say `git checkout -B main refs/reposix/origin/main`.
- Recovery: tried init's "Next:" hint first — `git checkout origin/main` → `error: pathspec 'origin/main' did not match any file(s) known to git`. Then tried the README form `git checkout -B main refs/reposix/origin/main` → worked.
- Friction: HIGH — the CLI tool's own success-path "Next:" message tells a fresh agent to run a command that fails. Only the docs have the correct form. Self-teaching falls apart at the first instruction.

### Step 3: discover issues

- What the doc told me to do: `ls issues/`, `cat issues/0001.md` (both README quick-start step and tutorial step 5).
- What I did: ran `ls issues/` first.
- What I observed: `ls: cannot access 'issues/': No such file or directory`. The seeded files are at the repo root: `0001.md`, `0002.md`, ..., `0006.md`.
- Recovery: ran `ls` (no path) and `cat 0001.md` directly. Got valid frontmatter + body for issue 1 ("database connection drops under load").
- Friction: HIGH — both the README quick-start AND `docs/tutorials/first-run.md` step 5 promise `issues/0001.md` as the path. Reality on the default `sim::demo` seed is repo-root `0001.md`. The tutorial even shows literal expected output: `# 0001.md  0002.md  0003.md  0004.md  0005.md` — observed reality includes `0006.md` too (extra issue) and no `issues/` prefix. A fresh agent following the tutorial verbatim hits "no such directory" on what is supposed to be a copy-pastable five-minute walkthrough.
- Bonus observation: tutorial's expected `cat issues/0001.md` body shows a "user avatar upload" issue ("Add user avatar upload"). The actual `0001.md` is "database connection drops under load." The tutorial's literal output samples are completely stale relative to the current seed.

### Step 4: edit issue body

- What I did: appended a `## Comment from dark-factory T1` block to `0001.md` via plain shell redirection (no `sed`, since the tutorial's `sed -i 's/^status: .*/status: in_progress/'` operates on a frontmatter line and I just need to append).
- What I observed: `git diff` showed clean three-line addition. Standard git, no surprises.
- Friction: none.

### Step 5: commit and push

- What the doc told me to do: `git add issues/0001.md && git commit && git push` (tutorial step 7).
- What I did: `git add 0001.md && git commit -m … && git push`.
- What I observed: bare `git push` failed with `fatal: The current branch main has no upstream branch. To push the current branch and set the remote as upstream, use git push --set-upstream origin main`. Recovery: ran the suggested `git push --set-upstream origin main`. Push then succeeded with `* [new branch] main -> main`.
  - Stderr also emitted: `WARN git_remote_reposix: cache unavailable for push audit: open reposix-cache: git: git config --add transfer.hideRefs failed: fatal: not in a git directory`. The push succeeded server-side regardless, but the cache-side audit row promised by tutorial step 8 was not written.
- Friction: HIGH — README quick-start tells a fresh agent: `git commit -am 'mark issue 1 done' && git push` with no `--set-upstream` step. With the README's `git checkout -B main refs/reposix/origin/main` form, `main` is created as a fresh branch with no upstream; bare `git push` cannot work the first time. The tutorial's step 7 has the same gap. Either the docs need to add `--set-upstream origin main` (or `-u origin main`) on the first push, or `reposix init` / the helper's checkout-out instruction needs to set the upstream automatically.
- Friction: MED — the `cache unavailable for push audit … not in a git directory` WARN is concerning. The helper running inside `/tmp/sim-darkfactory/.git` should certainly be "in a git directory." A push that the docs say also writes an audit row silently swallowed the audit-log half. See Step 7 below.

### Step 6: verify change persisted via REST

- What the doc told me to do: `docs/reference/simulator.md` lists routes; relevant: `GET /projects/<slug>/issues/<id>`.
- What I did: `curl -s http://127.0.0.1:7878/projects/demo/issues/1` and `curl -s http://127.0.0.1:7878/projects/demo/issues`.
- What I observed: `version: 2`, `updated_at: 2026-05-02T10:00:16Z` (was `2026-04-13T00:00:00Z`), and the appended `## Comment from dark-factory T1` block is in the `body` field of the JSON. SoT-side write confirmed.
- First attempt I made was the wrong endpoint `/projects/demo/records/1` (404). I went to it because the project elevator pitch and architecture docs talk about "records" extensively (`reposix-core::Record`, `list_records`, etc.) — there's a vocabulary mismatch between architecture docs ("records") and the public REST surface ("issues"). For a fresh user this is minor but still confusing.
- Friction: LOW — vocabulary mismatch between architecture-level "record" and REST-surface "issue." Once `simulator.md` is consulted it's unambiguous.

### Step 7: verify audit row (tutorial step 8 — bonus check)

- What the doc told me to do: `sqlite3 ~/.cache/reposix/sim-demo.git/cache.db "SELECT ts, op, decision FROM audit_events_cache WHERE op LIKE 'helper_push_%' ORDER BY ts DESC LIMIT 3"`.
- What I did: ran `ls ~/.cache/reposix/sim-demo.git/` first.
- What I observed: cache directory exists (HEAD, config, hooks/, info/, objects/, refs/) but no `cache.db` file. The tutorial's verification step would error with "no such file." This is consistent with the push-time WARN `cache unavailable for push audit: open reposix-cache: … fatal: not in a git directory` — the helper failed to open the cache and never created/wrote the audit row.
- Friction: HIGH — last documented step of the canonical 5-minute tutorial fails. The system did the right thing functionally (push round-tripped to the SoT) but the promised audit-log "system outcome" half of the story is silently absent. The CLAUDE.md OP-3 rule says "Audit log is non-optional"; for a fresh user following the tutorial, this is the difference between believing the security story and not.

## Findings summary

| #  | Severity | What                                                                                                                       | Evidence                                                                                                                                                                            |
|----|----------|----------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| F1 | HIGH     | `reposix init` "Next:" hint contradicts README/tutorial — tells user to run `git checkout origin/main` which fails.        | `reposix init sim::demo /tmp/sim-darkfactory` prints `Next: cd … && git checkout origin/main`; running it: `error: pathspec 'origin/main' did not match any file(s) known to git`. |
| F2 | HIGH     | `reposix init` emits a noisy WARN on the success path: `git fetch --filter=blob:none failed … fatal: could not read ref refs/reposix/main`, but the fetch did populate `refs/reposix/origin/main` and init claims success. | stderr from `reposix init sim::demo /tmp/sim-darkfactory`. Init's exit code is 0 and the docs treat init as succeeded; the WARN scares a fresh user with no recovery action. |
| F3 | HIGH     | README quick-start AND tutorial step 5 promise issues live at `issues/0001.md`; the default `sim::demo` seed puts them at the repo root (`0001.md` … `0006.md`). Verbatim copy-paste of `ls issues/` and `sed -i … issues/0001.md` fails. | `ls issues/` → `No such file or directory`. `ls /tmp/sim-darkfactory` lists `0001.md`…`0006.md` at root. Tutorial step 5 expected output: `# 0001.md  0002.md  0003.md  0004.md  0005.md`. |
| F4 | HIGH     | Tutorial step 5 expected `cat issues/0001.md` output ("Add user avatar upload" / `assignee: alice@acme.com`) is stale — actual seed issue 1 is "database connection drops under load." | `cat 0001.md` returned the database/load issue; tutorial-shown output never appears in any seeded issue. |
| F5 | HIGH     | First-time `git push` fails: `fatal: The current branch main has no upstream branch.` Both README quick-start and tutorial step 7 use bare `git push`. Recovery is `git push --set-upstream origin main` but docs do not mention it. | `git push` failure verbatim. After `git checkout -B main refs/reposix/origin/main`, `main` has no tracking branch — bare `git push` can never work. |
| F6 | HIGH     | Push-time WARN `cache unavailable for push audit … fatal: not in a git directory`; tutorial step 8's audit-row verification (`sqlite3 ~/.cache/reposix/sim-demo.git/cache.db …`) fails because `cache.db` is never created. SoT push succeeds; helper-side audit log silently missing. | Stderr WARN on push. `ls ~/.cache/reposix/sim-demo.git/` shows no `cache.db` (only HEAD/config/hooks/info/objects/refs). |
| F7 | LOW      | Vocabulary mismatch: architecture/CLAUDE.md docs talk about "records" / `list_records` / `Record`; user-facing REST surface is `/projects/<slug>/issues`. A fresh user copying from the elevator pitch tries `/records/1` and 404s. | `curl -s -w "%{http_code}" /projects/demo/records/1` → 404. `simulator.md` clarifies the route is `/issues/<id>`. |
| F8 | LOW      | `git checkout -B main refs/reposix/origin/main` is jargony; the linked `git-layer` page explains why but a fresh agent doesn't know to look there until they hit step 4. | Tutorial step 4 + README quick-start both punt to the `git-layer` doc with "one line of friction at clone time." |

## Goal outcome

- [x] Working tree bootstrapped (with friction — wrong "Next:" hint had to be ignored)
- [x] Issue found via `cat`/`grep`/`ls` (NOT under `issues/` as docs claim — at repo root)
- [x] Issue body edited
- [x] `git push` succeeded (after recovery via `--set-upstream origin main`)
- [x] Server-side change confirmed via REST (`version: 2`, body contains appended comment)
- [ ] Audit row visible per tutorial step 8 (FAILED — `cache.db` never created)

## Honest assessment

A fresh agent following the docs verbatim gets stuck in the first 30 seconds: `reposix init`'s own success-path "Next:" line tells you to run a command that fails, and once you ignore it and try the README/tutorial form, the very next instruction (`ls issues/`) targets a path that does not exist on the seeded sim. The tutorial's literal expected outputs are stale (the seed has changed under the docs at least once — different titles, different IDs, more issues than promised). The first `git push` requires a `--set-upstream` step the docs omit. End-to-end the agent CAN complete the task by ignoring the docs' specifics and trusting general git muscle memory, which is on-brand for the dark-factory thesis but quietly invalidates "five-minute copy-pastable tutorial." The audit-log verification at the end fails outright — `cache.db` is not created on push because the helper's cache open hits "fatal: not in a git directory" — so the tutorial's grand finale ("`git log` is intent, `audit_events_cache` is outcome") cannot be demonstrated by a new user. Three of six tutorial steps need correction; the audit-log gap is a systemic bug, not just a doc nit.



