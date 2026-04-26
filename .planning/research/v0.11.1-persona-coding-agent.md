# Persona audit: Coding agent in the dark-factory loop

> **Persona.** Claude-Code-shaped agent, no prior reposix knowledge, 200k ctx, tools = Read/Edit/Bash. Prompt: *"your harness gave you a working tree at /tmp/repo via reposix. Find issue PROJ-42 and post a comment that the bug is reproducible."*

---

## Did the loop actually work for me?

**No, and the failure is structural — not a docs gap.** The task as written is unimplementable on the current codebase, and reposix never tells me that.

Walking the loop:

1. **`ls /tmp/repo` — empty / nonexistent.** The harness said "I gave you a working tree" but the dir only exists post-`reposix init`. First 30 seconds I'd burn deciding whether to bootstrap myself, find `reposix` on `PATH`, pick the right `<backend>::<project>` — none of which is signaled in-band.

2. **`grep -r 'PROJ-42'` — finds nothing on the sim.** The sim names files `issues/0001.md` (integer keys); `PROJ-42` is JIRA-shaped. If the harness routed me to the sim, my prompt's key matches no file and there is no signal to tell me "the key shape depends on the backend."

3. **`docs/tutorials/first-run.md`.** Reads beautifully as new-install onboarding. Useless for *just tell me the verb to add a comment*. I'd copy the `cat >> issues/0001.md <<EOF\n## Comment from agent\nEOF` pattern, commit, push — and on JIRA this silently does nothing.

4. **`docs/reference/jira.md` — the punchline.** L98: `**Read-only:** create_record, update_record, and delete_or_close return "not supported". Write path lands in Phase 29.` L101: `**No comments:** JIRA comments are deferred to a future phase.` The persona task **cannot be done on the JIRA backend today**.

The dark-factory premise — "agent recovers from every reposix error by reading stderr verbatim" — only holds when the action is supported. Silently-unsupported actions give the agent no error to read.

## Friction the docs say is "self-healing" but actually isn't

### F-1. "PROJ-42" is JIRA-shaped, but JIRA is read-only

`docs/index.md` and `docs/concepts/reposix-vs-mcp-and-sdks.md` both open with `sed -i ... issues/PROJ-42.md && git push`. **Those examples don't run on the current JIRA connector** — `create_record/update_record` returns `not supported`. Either the push errors with a generic "not supported" or it silently no-ops (worse). Either way the headline example points at a backend that can't honor it.

**Fix.** Either move the JIRA example off the landing page until Phase 29, caveat it (`current JIRA connector is read-only — see reference/jira.md §Limitations`), or switch the canonical key to `issues/0001.md` (sim).

### F-2. Comments have no consistent shape across backends

Tutorial implies "comments = append a `## Comment` heading." Reality: sim round-trips it (works), JIRA drops it (deferred), Confluence has a separate inline/footer comment API, GitHub is unspecified in the docs I read. An agent that learns the pattern from the tutorial has internalized a fiction.

**Fix.** Add a one-line callout in the tutorial: *"Sim round-trips the body verbatim; comment semantics on JIRA/GitHub/Confluence are connector-specific — see `reference/<backend>.md` §Comments."*

### F-3. `git checkout origin/main` vs `git checkout -B main refs/reposix/origin/main`

`docs/index.md` quickstart says `git checkout origin/main`; `tutorials/first-run.md` step 4 says `git checkout -B main refs/reposix/origin/main`. The two are inconsistent. The tutorial form is the correct one for the write path (helper namespaces fetched refs). Pick one and update the landing page.

### F-4. `reposix doctor` is referenced before it's discoverable

The tutorial and troubleshooting page both punt to `reposix doctor`. Useful only if `reposix` is on `PATH`. There is no breadcrumb for "where the binary lives if `which reposix` fails" — the agent burns time on `find / -name reposix`. Five-second fix: add `<install-prefix>/bin/`, `$CARGO_HOME/bin`, release-tarball `bin/` to troubleshooting.md.

## Friction that genuinely is self-healing
*(Where the dark-factory pattern earned its keep)*

### S-1. Push-time conflict rejection

`! [remote rejected] main -> main (fetch first)` is upstream git's hint, not a reposix invention. Every coding agent has seen that exact text in its training data thousands of times. I'd type `git pull --rebase && git push` reflexively. **The dark-factory pattern delivers on its promise here.**

### S-2. Blob-limit error message

`error: refusing to fetch 487 blobs (limit: 200). Narrow your scope with git sparse-checkout set <pathspec> and retry.` — error names the recovery verb. Two-minute fix, not 30. **Caveat:** sparse-checkout assumes I know a path prefix; for content-search use cases the escape is `REPOSIX_BLOB_LIMIT=N`, also documented. Both work.

### S-3. Lazy materialization is invisible

`cat issues/0001.md` just works. I never had to know about partial clones or the helper RPC. ✓

### S-4. Frontmatter as schema

I read `status: open`, run `sed -i 's/^status: .*/status: in_progress/'`, see the diff, commit. Simplest possible UX. Only wrinkle: server-controlled fields (`id`, `version`, `created_at`) are silently stripped on push — `reposix doctor` could surface a warning for diffs that try to mutate them.

## What error messages saved me / cost me time

### Saved time
- **`fetch first`** (S-1) — three words, complete recovery loop.
- **`refusing to fetch N blobs (limit: M). Narrow your scope with git sparse-checkout set <pathspec>`** (S-2) — error names the verb.
- **`reposix doctor`'s copy-pastable Fix lines** (per `troubleshooting.md`) — exemplary; no parsing required.

### Cost time (or would, if encountered)
- **JIRA "not supported"** — too generic. Error needs to name (a) the unsupported op, (b) the phase that will land it, (c) the side-channel workaround: *"jira does not yet support comments (Phase 29). Workaround: curl the JIRA REST API directly with `$ATLASSIAN_API_KEY`."* That last sentence is the dark-factory move — the error tells the agent which escape hatch to use when the substrate can't help.
- **Missing-env on wrong shell** — the docs catch this in §missing-env-with-creds, but the error itself doesn't say "check the parent shell of `git fetch`." It should.
- **No file matching `issues/PROJ-42.md`** — `grep` returns nothing, no error. Agent has to *infer* a naming-convention mismatch. A `reposix doctor` check that prints the configured backend's filename pattern would close this in seconds.

## What the docs would have to add for me to never get confused

1. **A "Step 0: discover what you have"** before tutorial step 1: `cat .git/config | grep remote.origin.url` (backend), `ls issues/ pages/ 2>/dev/null` (naming convention), `git log --oneline -1 origin/main` (cache populated?). That's the actual first move in an unknown tree.
2. **Connector-capability matrix on the landing page.** One table:

   | backend | read | create | update | comments | delete | versioning |
   |---|---|---|---|---|---|---|
   | sim | ✓ | ✓ | ✓ | ✓ (round-tripped) | ✓ | strong |
   | github | ✓ | ✓ | ✓ | ✓ | ✓ (close) | ETag |
   | jira | ✓ | — | — | — | — | timestamp |
   | confluence | ✓ | ✓ | ✓ | — (separate api) | ✓ | strong |

3. **`reposix doctor` should print the configured backend's matrix row.**
4. **`reposix help comment`** (or equivalent) that maps the abstract verb onto the backend's actual mechanism.

## "Agent UX is pure git" — true or marketing?

**Truthy, two big asterisks.**

True for: read (`cat`, `grep`, `git log`), navigate (`cd`, `ls`), local edit (`sed`, editor), diff (`git diff`, `git status`), conflict-rebase recovery. I would not need to learn one reposix concept to do those things — the dark-factory premise delivers.

Asterisks:

- **`init` is not pure git.** `reposix::<backend>::<project>` URL syntax, binary on PATH, `extensions.partialClone=origin`, namespaced fetch refs (`refs/reposix/origin/*`). Invisible if the harness ran it; an entire concept stack if I'm bootstrapping.
- **Connector capabilities are not pure git.** "I can write a comment" on JIRA is currently false; "change the assignee" is false. The agent only learns this by hitting the error (or by reading the reference doc — which is exactly what "pure git UX" promises to avoid).

Honest framing: **reposix delivers pure-git UX on the read path and the conflict path. The write path is pure-git-shaped but capability-gated by the connector, and the agent has no in-band way to discover the gates.** That gap is the marketing/ground-truth delta.

## Bottom line

Pattern works where the substrate ships: read, grep, conflict-rebase, blob-limit-narrow. Headline persona task fails because:

1. JIRA writes/comments aren't implemented (Phase 29 deferred).
2. Landing-page example uses a JIRA-shaped key against the not-yet-implemented backend.
3. No in-band way (error, doctor output, file in tree) for the agent to discover supported verbs per backend.

Fixing #2 and #3 is cheap (capability matrix + doctor extension). Fixing #1 is Phase 29. Until #1 lands, **the docs do not tell the agent that the task is impossible** — and that is the single failure mode the dark-factory pattern is supposed to make impossible.

---

## Files referenced

- `/home/reuben/workspace/reposix/docs/index.md` — landing page, contains the JIRA-shaped headline example.
- `/home/reuben/workspace/reposix/docs/tutorials/first-run.md` — the canonical happy-path; sim-shaped paths.
- `/home/reuben/workspace/reposix/docs/guides/troubleshooting.md` — exemplary stderr-driven recovery doc.
- `/home/reuben/workspace/reposix/docs/concepts/mental-model-in-60-seconds.md` — the "three keys"; clear and accurate for the read path.
- `/home/reuben/workspace/reposix/docs/reference/jira.md` — lines 96–103 list the JIRA connector's deferred capabilities (read-only, no comments, no attachments).
- `/home/reuben/workspace/reposix/docs/concepts/reposix-vs-mcp-and-sdks.md` — repeats the JIRA-shaped headline example.
- `/home/reuben/workspace/reposix/docs/how-it-works/git-layer.md` — push-time conflict and blob-limit mechanics; matches the recovery story.
