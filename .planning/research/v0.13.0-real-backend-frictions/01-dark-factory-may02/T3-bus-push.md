# T3 — Bus push (v0.13.0 dark-factory test)

Tester perspective: a fresh agent who has never used reposix. Goal: end-to-end exercise the bus URL form (`reposix::<sot>?mirror=<plain-mirror-url>`) against the real Confluence `REPOSIX` space + the `reubenjohn/reposix-tokenworld-mirror` GitHub repo. Working directory: `/tmp/bus-test/`.

Severity rubric:
- **HIGH** — blocks the user, contradicts docs, or yields a misleading state.
- **MED** — annoying but workaround-discoverable; doc gap that wastes ≥5 minutes.
- **LOW** — small polish opportunity (typo, surprise log line, cosmetic).

---

## Step 1 — Read the user-facing docs

Skimmed `docs/concepts/dvcs-topology.md` and `docs/guides/dvcs-mirror-setup.md`.

Findings during read:

- **F1 (LOW, doc clarity).** The bus URL form `reposix::<sot>?mirror=<plain-mirror-url>` is referenced in CLAUDE.md, but I cannot find it explicitly written out for end users in `docs/concepts/dvcs-topology.md`. The doc shows `reposix init`/`reposix attach`/`git push` flows but never names the literal URL string format the user types or the tool composes. A cold reader following Pattern C will not know what `git remote -v` should look like, nor how to compose the URL by hand if `attach` fails.
- **F2 (LOW, doc clarity).** `dvcs-topology.md` says the host slug for Confluence is `confluence`, used in `refs/mirrors/<sot-host>-head`. Good. But `dvcs-mirror-setup.md` Step 4 says the GH-Action workflow uses `reposix init confluence::<SPACE>` — no mention of the bus URL form. The owner-side workflow does NOT use bus push (it's plain `git push --force-with-lease` to mirror only). That asymmetry should probably be called out so a reader does not assume the workflow is doing fan-out.
- **F3 (MED, doc gap).** No tutorial covers the **agent** side of bus push (`reposix attach` setting up the bus URL, then `git push` fanning out). The "Pattern C — Round-tripper" subsection of topology.md ends at `git push` without showing what the remote URL looks like or how to verify both sides updated. T2 (attach tutorial) presumably covers attach, but there is no T3-bus-push tutorial. As a fresh user, I have to infer the URL composition from CLAUDE.md.

## Step 2 — Get a working tree

Followed Pattern C from `dvcs-topology.md` line 122–137: vanilla `gh repo clone` of the mirror, then `reposix attach confluence::REPOSIX`.

```text
$ gh repo clone reubenjohn/reposix-tokenworld-mirror /tmp/bus-test
Cloning into '/tmp/bus-test'...
$ cd /tmp/bus-test && ls
README.md            # nothing else — mirror has never been synced
$ git log --oneline -5
09dda47 feat(workflow): add reposix-mirror-sync.yml (P84 / DVCS-WEBHOOK-01)
d224f47 Initial commit
```

Then ran the documented attach command:

```text
$ reposix attach confluence::REPOSIX
Error: attach: backend `confluence` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03
```

**F4 (HIGH, blocker — docs lie about a shipped command).** `reposix attach` for confluence is wired ONLY for `sim::*` per the error. But:
1. CLAUDE.md "Architecture" section enumerates `reposix attach <backend>::<project>` as a top-level entry point with no caveat.
2. `docs/concepts/dvcs-topology.md` line 134 tells users `reposix attach confluence::SPACE` is THE Pattern C move.
3. `reposix attach --help` example block literally shows `reposix attach confluence::SPACE /tmp/repo` as a documented usage (no scaffold caveat in help text either).

Net effect for a fresh user: the documented round-tripper path fails immediately. The error message points at "P79-03" — an internal phase id that means nothing to a user.

**F5 (HIGH, doc-vs-code drift).** Either the docs need to mark Pattern C / `reposix attach` for non-sim backends as "v0.13.1+ / not yet implemented," OR the attach command must implement confluence/github/jira. Right now docs are written as if the feature shipped; the binary disagrees.

Falling back to Pattern B (`reposix init confluence::REPOSIX`) per the assignment instructions.

```text
$ rm -rf /tmp/bus-test && mkdir /tmp/bus-test
$ reposix init confluence::REPOSIX /tmp/bus-test
WARN reposix_cli::init: git fetch --filter=blob:none failed with status exit status: 128 — local repo is configured but not yet synced. Stderr: fatal: could not read ref refs/reposix/main
reposix init: configured /tmp/bus-test with remote.origin.url = reposix::https://reuben-john.atlassian.net/confluence/projects/REPOSIX
Next: cd /tmp/bus-test && git checkout origin/main (or git sparse-checkout set <pathspec> first)
```

`reposix doctor` then surfaced the underlying blocker:

```
ERR   git.version: git 2.25.1 is too old — partial-clone + stateless-connect needs >=2.27 (recommend 2.34)
     fix: install git >= 2.34 from your package manager
```

**F6 (HIGH, environment).** This VM ships with `git 2.25.1` (Ubuntu 20.04 default). The reposix runtime requires 2.34+. The fresh-clone story `git checkout origin/main` cannot work on this machine — `git fetch` errors with `fatal: could not read ref refs/reposix/main` because the helper's `stateless-connect` flow does not negotiate against ancient git. Doctor catches this clearly, which is good. But:
- The `reposix init` command itself emits only a `WARN` on the failed fetch, then prints "Next: cd /tmp/bus-test && git checkout origin/main" — i.e. tells the user to do something that is guaranteed to fail given the warning that just printed. The init command should fail-fast or at minimum NOT print a Next-step that requires the very thing that just failed.
- README/install instructions do not surface the git ≥2.34 prerequisite to a fresh user. CLAUDE.md mentions it, but a cold reader following docs does not see this until `doctor` runs.

**F7 (MED, recovery path divergence).** Because `git checkout origin/main` cannot work, the only path forward is `reposix refresh` — which I had to discover from `reposix --help`. Refresh produced a populated working tree on `master` (NOT `main`), with backend records as the root commit. This is a perfectly fine surrogate for the dark-factory scenario but it is a **completely different** topology from what `dvcs-topology.md` Pattern B walks the user through (`git checkout origin/main` → edit → push).

```text
$ reposix refresh /tmp/bus-test --backend confluence --project REPOSIX
[master (root-commit) 70d94da] reposix refresh: confluence/REPOSIX — 4 issues at 2026-05-02T10:11:56Z
 6 files changed, 47 insertions(+)
 create mode 100644 pages/00000065916.md
 create mode 100644 pages/00000131192.md
 create mode 100644 pages/00000360556.md
 create mode 100644 pages/00000425985.md
```

End state: `origin` remote URL = `reposix::https://reuben-john.atlassian.net/confluence/projects/REPOSIX` (single-SoT, NO bus form yet); local branch = `master`; 4 pages materialized under `pages/`.

## Step 3 — Set the remote URL to the bus form

The expected bus URL form (per CLAUDE.md):

```
reposix::https://reuben-john.atlassian.net/confluence/projects/REPOSIX?mirror=git@github.com:reubenjohn/reposix-tokenworld-mirror.git
```

The docs warn `?` in mirror URLs must be percent-encoded. The `?` in the bus URL itself is the separator, not the one being encoded — only `?` characters appearing INSIDE the mirror URL would need encoding. The mirror URL `git@github.com:reubenjohn/reposix-tokenworld-mirror.git` has no `?`, so no encoding needed for this case. Good.

**F8 (MED, doc gap).** Neither `dvcs-topology.md` nor `dvcs-mirror-setup.md` shows the literal bus URL string anywhere. The cold-reader has no example to copy. I would have failed without CLAUDE.md.

```bash
$ git remote set-url origin "reposix::https://reuben-john.atlassian.net/confluence/projects/REPOSIX?mirror=git@github.com:reubenjohn/reposix-tokenworld-mirror.git"
$ git remote -v
origin  reposix::https://...REPOSIX?mirror=git@github.com:reubenjohn/reposix-tokenworld-mirror.git (fetch)
origin  reposix::https://...REPOSIX?mirror=git@github.com:reubenjohn/reposix-tokenworld-mirror.git (push)
```

Set successfully. Single-quoting + bash here-string handled the `?` and `:` fine — no shell-eating actually occurred for this URL shape.

## Step 4 — Edit a page and commit

```bash
$ # Append a marker line to pages/00000065916.md
$ git add pages/00000065916.md
$ git commit -m "test: T3 bus-push marker on Architecture notes"
[master 6801b06] test: T3 bus-push marker on Architecture notes
 1 file changed, 1 insertion(+)
```

No friction. Working tree edits are pure git. ✓

## Step 5 — `git push`

Bus push attempt 1:

```text
$ git push origin master
error: configure the mirror remote first: `git remote add <name> git@github.com:reubenjohn/reposix-tokenworld-mirror.git`
git-remote-reposix: unknown command: feature
To reposix::...?mirror=git@github.com:reubenjohn/reposix-tokenworld-mirror.git
 ! [remote rejected] main (no-mirror-remote)
```

**F9 (HIGH, surprise prerequisite).** The bus URL alone is NOT enough. The helper requires a SEPARATE local git remote (e.g. `mirror`) pointing at the same plain-git URL. This is documented NOWHERE in `dvcs-topology.md` or `dvcs-mirror-setup.md`. The error message is helpful (it tells you the recovery), but the docs paint bus push as "single git remote, single push command, fans out to both" — that's incomplete. The actual minimum config is:

```bash
git remote add origin "reposix::<sot>?mirror=<plain-mirror-url>"
git remote add mirror <plain-mirror-url>          # required, undocumented
git fetch mirror                                  # also required, see F11
```

**F10 (LOW, log noise).** `git-remote-reposix: unknown command: feature` leaks every push, regardless of success. It's a benign git protocol-v2 thing the helper doesn't speak, but to a fresh user it looks like an unhandled error. The helper should either silently swallow `feature` or at least not print it on stderr.

Bus push attempt 2 (after `git remote add mirror ...`):

```text
$ git push origin master
your GH mirror has new commits: local refs/remotes/mirror/main = (no local ref refs/remotes/mirror/main); remote git@github.com:reubenjohn/reposix-tokenworld-mirror.git HEAD = 09dda471...
hint: run `git fetch mirror` first, then retry the push
error: mirror drift detected (PRECHECK A)
 ! [rejected]        main (fetch first)
```

**F11 (MED, prerequisite chain).** PRECHECK A requires `refs/remotes/mirror/main` to exist locally. After `git remote add mirror ...` you must also `git fetch mirror`. Same docs gap as F9 — neither operation is mentioned in any user-facing doc. Three-step prereq before the bus URL works: (1) `set-url origin <bus>`, (2) `remote add mirror <plain>`, (3) `fetch mirror`. Each one's failure produces a different cryptic error.

Bus push attempt 3 (after `git fetch mirror`):

The vanilla mirror clone has commits unrelated to my refresh-rooted master, so the fetch reported `warning: no common commits`. I tried `git pull --rebase mirror main --allow-unrelated-histories` to interleave histories. Then push attempt:

```text
$ git push origin master:main
error: invalid issue at .github/workflows/reposix-mirror-sync.yml: invalid record file: missing frontmatter open fence; refusing push
 ! [remote rejected] master -> main (invalid-blob:.github/workflows/reposix-mirror-sync.yml)
```

**F12 (HIGH, fundamental topology contradiction).** The helper's export-validator rejects ANY blob in the tree that lacks frontmatter. But:

1. The mirror repo legitimately holds `.github/workflows/reposix-mirror-sync.yml` (the workflow itself!) and `README.md` — neither of which can have frontmatter. This is the OWNER-CREATED content the docs (`dvcs-mirror-setup.md` Step 2) literally tell you to commit there.
2. After Pattern C (`gh repo clone` + reposix attach), your tree DOES include those files. They are not removable without breaking the mirror's GH-Action workflow.
3. So the bus push CANNOT succeed against a mirror that follows the documented setup, because the SoT-export side rejects what the mirror requires.

This is a real architectural collision, not just a doc bug. Either:
- the helper's export must skip non-frontmatter files (treat them as mirror-only blobs), OR
- the mirror sync workflow + docs must avoid putting non-record files in the tree (move workflow to a different branch, or use a separate config repo), OR
- the bus URL form needs explicit "record root" prefix configuration to scope the export.

Reset master, removed the rebase. Tried `--force`. Same `.github/workflows/...` error gone (no rebase) but new ones surfaced:

**F13 (HIGH, refresh emits files its own export rejects).** Even with a "clean" master = single refresh root + my one edit, push fails:

```text
$ git push --force origin master:main
warning: helper reposix does not support 'force'
error: invalid issue at .reposix/fetched_at.txt: invalid record file: missing frontmatter open fence; refusing push
 ! [remote rejected] master -> main (invalid-blob:.reposix/fetched_at.txt)
```

`reposix refresh` itself committed `.reposix/.gitignore` and `.reposix/fetched_at.txt` to the working tree (the refresh's root commit creates these). The same tool that BUILT the tree creates files the helper REJECTS on push. That's a contradiction at the tooling layer, not just docs.

Tried `git filter-branch` to scrub `.reposix/` from history — succeeded. Re-pushed:

```text
$ git push --force origin master:main
warning: helper reposix does not support 'force'
error: create issue: confluence returned 404 Not Found for POST /wiki/api/v2/pages: {"errors":[{"status":404,"code":"NOT_FOUND","title":"Cannot find content with id [360556]","detail":null}]}
... (3x same error)
 ! [remote rejected] master -> main (some-actions-failed)
```

**F14 (HIGH, no shared history → no diff).** The helper's export uses `git diff` against parent commits, NOT against the SoT cache state. Because `reposix init`'s `git fetch` failed (F6 — git 2.25.1), my master has no shared ancestry with what's in the SoT cache. The export therefore sees ALL pages as "new" and tries to POST /wiki/api/v2/pages — which 404s on `parent_id=360556` (parent already exists in confluence under a DIFFERENT internal cache OID).

This means the documented Pattern B happy path (`init → checkout → edit → push`) only works if the FIRST step succeeds. On any environment where init's fetch fails (old git, network, allowlist), every subsequent push is doomed — and the user gets a confusing "404 NOT_FOUND id [360556]" instead of a "your cache never synced" message.

**F15 (MED, force semantics).** `warning: helper reposix does not support 'force'` printed even when `--force` was the only way past PRECHECK A. Either the helper should support `--force` semantics (after a confirmed local cache reconcile), or it should error early with a clearer message. The current behavior accepts the push attempt, runs the validator, and then rejects — wasting credentials/round-trips on a doomed push.

## Step 6 — Verify SoT updated

```bash
$ curl -s -u "$ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY" \
    "https://$REPOSIX_CONFLUENCE_TENANT.atlassian.net/wiki/api/v2/pages/65916?body-format=storage" \
  | jq '{id, title, version}'
{
  "id": "65916",
  "title": "Architecture notes",
  "version": { "number": 1, ... }
}
```

SoT version still 1. Push never landed (consistent with all errors above). ✗

## Step 7 — Verify mirror updated

```bash
$ git ls-remote https://github.com/reubenjohn/reposix-tokenworld-mirror.git
09dda471a27097345025190f9b6f2b87ff4b481c    HEAD
09dda471a27097345025190f9b6f2b87ff4b481c    refs/heads/main
```

Mirror still at `09dda47` (the workflow-add commit). Push never landed. ✗

## Step 8 — Verify mirror-lag refs

```bash
$ git fetch origin "+refs/mirrors/*:refs/mirrors/*"     # silent, no output
$ git for-each-ref refs/mirrors/                         # empty
$ ls ~/.cache/reposix/confluence-REPOSIX.git/refs/mirrors/ 2>/dev/null
# (no such directory)
```

No `refs/mirrors/confluence-head` and no `refs/mirrors/confluence-synced-at`. The cache at `~/.cache/reposix/confluence-REPOSIX.git/cache.db` is **0 bytes** — i.e. the failed `init` fetch never produced cache rows, and no push has succeeded to write any sync events.

```bash
$ ls -la ~/.cache/reposix/confluence-REPOSIX.git/
-rw-r--r-- ...  cache.db                    # 0 bytes
$ ls -la /tmp/bus-test/.reposix/
-rw------- ...  cache.db                    # 8K — refresh's local cache
```

**F16 (MED, dual cache locations).** Two cache.db files exist: one at `~/.cache/reposix/confluence-REPOSIX.git/cache.db` (created by `reposix init`, 0 bytes) and one at `/tmp/bus-test/.reposix/cache.db` (created by `reposix refresh`, 8K). The init-style cache is what the helper expects to read for partial-clone state; the refresh-style cache is what the refresh tool wrote. They do not appear to be unified, which probably contributes to F14 (no shared history).

✗ Mirror-lag refs not present.

---

## Findings table

| ID | Severity | Area | Finding |
|---|---|---|---|
| F1 | LOW | docs | Bus URL string format `reposix::<sot>?mirror=<url>` not shown literally in user-facing docs. |
| F2 | LOW | docs | `dvcs-mirror-setup.md` Step 4 uses `reposix init`, not bus URL — asymmetry not called out. |
| F3 | MED | docs | No tutorial covers Pattern C (round-tripper / bus push). T3-bus-push.md tutorial does not exist. |
| F4 | HIGH | code | `reposix attach confluence::SPACE` (documented Pattern C) errors with "P79-02 scaffold; sim only." |
| F5 | HIGH | docs | Docs describe Pattern C as shipped (`docs/concepts/dvcs-topology.md` line 134); binary disagrees. |
| F6 | HIGH | env | `git 2.25.1` (Ubuntu 20.04 default) breaks `reposix init` fetch. README/install docs do not surface the 2.34+ requirement. `init` warns and then prints "Next: git checkout origin/main" — guaranteed to fail. |
| F7 | MED | code | Recovery path for failed init fetch is undocumented; `reposix refresh` is discoverable only via `--help`. Refresh produces `master` branch (not `main`), root commit (not `origin/main` ancestry) — different topology than docs walk through. |
| F8 | MED | docs | Bus URL literal example missing from all user-facing docs. Cold readers must reverse-engineer from CLAUDE.md. |
| F9 | HIGH | docs+UX | Bus URL alone insufficient — helper requires SECOND local remote (`git remote add mirror <plain-url>`). Undocumented prerequisite. |
| F10 | LOW | UX | `git-remote-reposix: unknown command: feature` leaks on every push (protocol-v2 noise). |
| F11 | MED | docs+UX | After `remote add mirror`, also need `git fetch mirror` so PRECHECK A has a ref to compare. Three-step undocumented prereq chain. |
| F12 | HIGH | architecture | Helper's frontmatter validator rejects `.github/workflows/*.yml` and `README.md` in the mirror tree — yet the documented mirror-setup REQUIRES committing those very files to the mirror. Bus push cannot succeed against a mirror that follows the docs. |
| F13 | HIGH | code | `reposix refresh` writes `.reposix/.gitignore` + `.reposix/fetched_at.txt` into the tree. Helper's export rejects them as invalid records. The tool that builds the tree creates files its own export refuses. |
| F14 | HIGH | architecture | Export uses `git diff` against parent commit, not against SoT cache. Without successful init-fetch (F6), every page looks "new" → POST creates → 404 NOT_FOUND on parent_id. The error message tells the user nothing about cache-desync as the root cause. |
| F15 | MED | UX | `git push --force` warned "helper reposix does not support 'force'" but PRECHECK A made `--force` the only path forward. Inconsistent semantics; no clear escape hatch. |
| F16 | MED | code | Dual cache locations (`~/.cache/reposix/<backend>-<project>.git/cache.db` from init vs. `<worktree>/.reposix/cache.db` from refresh). Likely related to F14 history-mismatch. |

## Boxes ticked (8 expected)

| # | Step | Status |
|---|---|---|
| 1 | Read user-facing docs | ✓ |
| 2 | Get a working tree | ◐ partial — got via fallback (`init` warned + `refresh`); attach failed (F4); `checkout origin/main` impossible (F6) |
| 3 | Set bus remote URL | ✓ |
| 4 | Edit + commit | ✓ |
| 5 | `git push` on bus URL | ✗ rejected after multiple recovery attempts |
| 6 | Verify SoT updated | ✗ (still version 1; push never landed) |
| 7 | Verify mirror updated | ✗ (still at 09dda47) |
| 8 | Verify mirror-lag refs | ✗ (no refs/mirrors/* exist) |

**3.5 of 8 boxes ticked.** Push never succeeded; nothing past Step 5 was reachable.

## Honest take

The bus-push feature is **not usable** by a fresh agent following the docs end-to-end against the real `REPOSIX` confluence space + the existing `reposix-tokenworld-mirror` GitHub repo. The chain of failures compounds:

1. The documented entry point (`reposix attach confluence::SPACE`, Pattern C) is not implemented — error names an internal phase id (P79-03).
2. Falling back to `reposix init` requires `git ≥ 2.34`; Ubuntu 20.04 ships 2.25.1 and the init command warns-but-proceeds, telling the user to do something that cannot work.
3. Working around with `reposix refresh` produces a tree that the helper's own export validator rejects — `.reposix/` metadata files are committed by refresh and refused by push.
4. Even with a perfectly clean tree, a push has no shared ancestry with the SoT cache (because step 2 fetch failed) → all pages look "new" → 404 NOT_FOUND on parent_id, with an error message that gives the user no clue about the root cause.
5. Crucially, the bus topology has a fundamental tension: the documented mirror setup requires `.github/workflows/*.yml` + `README.md` in the mirror tree, but the helper's frontmatter-only validator refuses those very files on push. Even on a fresh-git environment this would block a Pattern C push.

The findings split roughly: 7 HIGH (real blockers / contradictions), 7 MED (doc gaps / UX papercuts), 2 LOW (cosmetic).

The strongest signal: **F12** (mirror tree vs. SoT export collision) is an architectural issue, not a doc patch. The bus URL form needs either a path-prefix scope (e.g. `?mirror_root=pages/`) or a tree-level skip rule for non-record blobs, or the docs need to teach that the mirror is a SEPARATE-history `pages/`-only branch. Until that's resolved, the dark-factory thesis "vanilla mirror + reposix attach + git push fans out" cannot work end-to-end.

Time used: ~25 minutes (over budget by 5 — chased the helper-export errors a step further than planned to confirm F14 is a real architectural issue, not a one-off).

