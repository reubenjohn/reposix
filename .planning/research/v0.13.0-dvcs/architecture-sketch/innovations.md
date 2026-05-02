# The three innovations

← [back to index](./index.md)

## The three innovations

### 1. `reposix attach <backend>::<project>`

**Problem.** Today's `reposix init` creates a working tree from scratch: `git init` + set `extensions.partialClone=origin` + set `remote.origin.url` + `git fetch --filter=blob:none origin`. This requires the working tree to be born under reposix's hand. In the DVCS topology, Dev B has a working tree that came from `git clone git@github.com:org/repo.git` — a vanilla git clone of a plain-git mirror. There's no cache, no `extensions.partialClone`, no reposix-aware remote. Dev B can read perfectly well, but cannot push back to the SoT because the helper's `export` path needs cache state for conflict detection.

**Sketch.**

```bash
reposix attach confluence::SPACE
# In CWD, with no special prerequisites on how the checkout was created:
#   1. Build a fresh cache directory at the standard location.
#   2. REST-list the backend; populate cache OIDs (filenames + tree
#      structure; blobs lazy on first materialize as today).
#   3. Reconcile: walk current HEAD tree, match each file to a backend
#      record by `id` in frontmatter. Record matches in the cache's
#      reconciliation table.
#   4. Add a remote `reposix::bus://confluence::SPACE+<existing-origin>`
#      (or `reposix::confluence::SPACE` if user passed --no-bus).
#   5. Set `extensions.partialClone=<remote-name>` (the new reposix remote).
#      Existing `origin` (the GH mirror) keeps its plain-git semantics.
```

After attach, the working tree has TWO remotes:
- `origin` — plain-git GH mirror (vanilla; clone source).
- The new bus remote — reposix-equipped; the push target.

`git fetch origin` continues to pull from the mirror (fast, no REST). `git fetch <new-remote>` pulls from the SoT directly (slower; bypasses mirror lag). `git push <new-remote> main` triggers the bus remote algorithm in §3.

**Reconciliation cases (the part that's harder than it looks).**

| Local file | Backend record | Resolution |
|---|---|---|
| `issues/0001.md` with `id: 1` | record `id: 1` exists | match; cache stores OID alignment |
| `issues/0001.md` with `id: 1` | no record `id: 1` (deleted on backend) | warn user; skip; offer `reposix attach --orphan-policy={delete-local,fork-as-new,abort}` |
| `issues/x.md` with no `id` field | n/a | warn; skip; not a reposix-managed file |
| Two local files claim `id: 1` | n/a | hard error; user must resolve |
| Backend record with `id: 99` | no local file | normal — record exists in SoT but mirror hasn't caught up; cache marks for next fetch |

**Open questions for the planner:**

- `Q1.1` Where does the cache live for an attached checkout? Today `reposix init` derives cache path from `remote.origin.url`. For attach, `origin` is the mirror, not the SoT. Probably derive from the SoT URL passed to attach. Document explicitly so `reposix sync` etc. find the same cache.
- `Q1.2` What if `reposix attach` is run twice with different SoT specs? Reject the second? Allow and switch active SoT? Probably reject — multi-SoT is the v0.14.0 origin-of-truth question.
- `Q1.3` Does `attach` need to detect that the checkout was originally produced by `reposix init` (i.e., `extensions.partialClone` is already set) and behave differently? Probably no — make `attach` idempotent so a re-attach refreshes the cache against the current backend state.

### 2. Mirror-lag observability via plain-git refs

**Problem.** Dev B (vanilla-git user) does `git pull origin` from the mirror. Mirror lags confluence by N commits because the webhook-driven sync hasn't fired yet. Dev B edits, commits, pushes back to confluence via the bus remote. Confluence rejects with `error refs/heads/main fetch first`. Dev B is confused: *"I just pulled!"* They have no mental model of the mirror being eventually consistent.

**Sketch.** Annotate the GH mirror with refs that record the SoT state it represents:

```
refs/mirrors/confluence-head           # SHA of the SoT's main at last sync
refs/mirrors/confluence-synced-at      # annotated tag with timestamp message
```

The webhook sync writes both refs after each successful mirror push. The bus remote also writes them after each successful bus push (since the bus push *is* a sync from a Dev's perspective).

**What this enables:**

- `git fetch origin` brings these refs into Dev B's local repo (plain git fetches them naturally).
- `git log refs/mirrors/confluence-synced-at -1` shows when the mirror last caught up.
- The bus remote's reject message can read its own ref state and say:
  ```
  error: confluence rejected the push (issue 0001 modified at 2026-04-29T17:30:00Z, your version 7, backend version 8)
  hint: your origin (GH mirror) was last synced from confluence at 2026-04-29T17:25:00Z (5 minutes ago)
  hint: run `reposix sync` to update your local cache from confluence directly, then `git rebase` your changes
  ```
- A simple `reposix doctor` (when it lands; tracked since v0.11.0 vision doc) flags mirror lag > 5min as a warning.

**Open questions for the planner:**

- `Q2.1` Are these refs under `refs/mirrors/...` or a more standard namespace like `refs/notes/reposix/...` (git notes are designed for "metadata about commits")? Notes have nicer tooling but worse discoverability. Probably refs for v0.13.0; revisit later.
- `Q2.2` Webhook sync writes the refs on success. What writes them when the SoT changes but the mirror sync hasn't fired yet? Nothing — the gap between confluence-edit and webhook-fire is exactly the staleness window the refs measure. That's the point. Make sure the doc explains this clearly so users don't misread the ref as "current SoT state."
- `Q2.3` Does the bus remote update both refs or just `confluence-head` (treating timestamp updates as the webhook's job)? Probably both for simplicity; webhook becomes a no-op refresh when the bus already touched them.

### 3. Bus remote with cheap-precheck + SoT-first-write

**Problem.** A single `git push` should attempt to update both the SoT and the mirror. Failures should fail loudly and recoverably, not silently and ambiguously. Cost should be optimized — fail fast on the cheap check before doing the expensive REST work.

**Sketch.** New URL scheme: `reposix::bus://<sot-spec>+<mirror-spec>`. Helper recognizes `bus://`, parses the two endpoints, dispatches as below.

**Algorithm (export path):**

```
1. Helper reads its config (knows: bus mode, SoT = confluence::SPACE,
   mirror = github::org/repo).

2. CHEAP PRECHECK A — mirror drift:
     ls-remote github main
     compare returned SHA to local refs/remotes/github/main
     drifted? → emit "error refs/heads/main fetch first" + hint
                "your GH mirror has new commits; git fetch github first"
                bail. NO confluence work done. NO stdin read.

3. CHEAP PRECHECK B — SoT drift:
     backend.list_changed_since(last_fetched_at) on confluence
     non-empty? → emit "error refs/heads/main fetch first" + hint
                  "confluence has changes since your last fetch; git pull --rebase"
                  bail. NO writes done. NO stdin read.

4. Read fast-import stream from git on stdin. Buffer it.

5. INNER CORRECTNESS CHECK on SoT (the existing per-record version
   check in handle_export, lines 350-407 of crates/reposix-remote/src/main.rs):
     backend.list_records(project)
     compare per-record `version` field, fail with detailed error on mismatch
     (this is the existing single-confluence behavior; bus remote inherits it
     verbatim).

6. SoT WRITE — apply REST writes to confluence.
   On any failure here: bail; mirror is unchanged; no recovery needed.
   On success: write audit rows (cache + backend), update last_fetched_at.

7. MIRROR WRITE — git push to GH mirror.
   On failure here: SoT is now ahead of mirror. Write mirror-lag audit row.
   Update refs/mirrors/confluence-head to new SoT SHA but DO NOT update
   refs/mirrors/confluence-synced-at (stays at last successful mirror sync).
   Print warning to stderr: "SoT push succeeded; mirror push failed (will
   retry on next push or via webhook sync). Reason: <error>."
   Return ok to git anyway — the SoT write succeeded and that's the
   contract from the user's perspective.

8. Update refs/mirrors/confluence-synced-at to now.

9. Send "ok refs/heads/main" back to git.
```

**Why SoT-first for writes (not mirror-first):** if the mirror write fails after the SoT write succeeded, the mirror just lags — recoverable on next push (any pusher can catch it up) or via webhook sync. If the SoT write failed after a mirror write succeeded, the mirror would have a SHA the SoT will never accept, and rolling back means force-pushing to a shared mirror that other devs have already fetched. SoT-first means the recovery story is "next pusher catches up," not "force-push the mirror."

**Why prechecks before stdin read:** the prechecks are network calls. Doing them before reading stdin means stdin sits buffered in the OS pipe during the precheck window. For typical issue-tracker push sizes (a few KB) this is irrelevant. If reposix ever grows toward larger artifacts (image attachments, etc.), the helper would want to overlap stdin reading with the precheck — flagged for future work, not v0.13.0.

**Open questions for the planner:**

- `Q3.1` See the dedicated subsection "Performance subtlety: today's `list_records` walk on every push" below. It's the most consequential decision in the bus-remote design and deserves more than a one-line Q.
- `Q3.2` Cache layer for the cheap GH precheck. `ls-remote` is already minimal but TLS handshake dominates. A 30s TTL cache keyed by `<remote>:<ref>` saves the network call when a developer pushes multiple commits in quick succession. Implement in v0.13.0 or defer? Probably defer — measure first, add if hot.
- `Q3.3` What's the bus URL scheme syntax? Options: `reposix::bus://<sot>+<mirror>`, `reposix::bus(<sot>,<mirror>)`, `reposix::<sot>?mirror=<mirror>`. The `+` form is short but `+` is not URL-safe in all contexts. Probably go with explicit query param: `reposix::confluence::SPACE?mirror=git@github.com:org/repo`. Plays well with existing URL parsing.
- `Q3.4` Does the bus remote handle FETCH (read-side) too, or only PUSH? Probably PUSH only for v0.13.0 — fetch goes to the SoT directly via the existing single-backend code path. Bus is a write-fan-out construct; reads have a single source of truth (confluence) and don't need fan-in.
- `Q3.5` What happens if `--mirror=` points at a remote that doesn't exist locally yet (`git remote add` not run)? Helper auto-runs `git remote add github <url>` for the user, or fails with a clear "configure the mirror remote first" message? Probably fail with a hint — don't auto-mutate user's git config.
- `Q3.6` Atomicity in the failure case where step 7 fails with a transient error (network blip). Retry inside the helper, or surface the failure and let the user retry the whole push? Probably surface it — retries inside the helper hide useful signal and complicate the audit trail.

### Performance subtlety: today's `list_records` walk on every push

**The current state.** `crates/reposix-remote/src/main.rs::handle_export` (lines 334–348) calls `state.backend.list_records(&state.project)` *unconditionally on every push*. For confluence, that's a paginated CQL search across the whole space (typically `O(N)` REST calls where N is "issue count / page size"). For a space with 5,000 records and a page size of 50, that's 100 REST calls — every single push, even a one-character typo fix to one issue.

This was a defensible choice when push was a rare ceremony and `list_records` was the simplest way to get both (a) the per-record version map for conflict detection and (b) the prior tree state for diff computation in `plan()`. It is not defensible at DVCS scale, where pushes happen frequently and the bus remote layers a cheap precheck on top of an already-expensive correctness check.

**Naming the inefficiency.** This is not a bus-remote-introduced regression — the bus remote inherits the cost. But documenting it now is load-bearing because:

1. The bus remote's `list_changed_since` precheck (step 3) makes the per-push cost LOOK acceptable on the success path (precheck returns empty → fast). Then step 5 fires `list_records` and the win evaporates. A naive reader of the bus algorithm would conclude "we made it cheap" when in reality we only made the *failure* path cheap.
2. v0.13.0 specifically widens the audience of pushers (Dev B with `attach`, plus webhook-driven sync workflows). More pushes per unit time = the `list_records` cost compounds.
3. Confluence rate limits (Atlassian Cloud is 5000 req/hr per user) become a real ceiling. A 5,000-issue space + 100 pushes/day = 10,000 calls/day just for conflict detection. We blow the rate limit by noon.

**Can it be avoided?** Yes, with a redesigned conflict-detection path. Three layers of optimization, in increasing complexity:

| Layer | Mechanism | Milestone |
|---|---|---|
| **L1** | Use `list_changed_since(last_fetched_at)` as the *only* network call. Compute conflict + plan from the cache's existing tree state + the delta. Single call, returns only changed records. Trades one safety property: today's `list_records` would catch a record that exists on backend but missing from cache (cache desync from a previous failed sync). With L1, cache is trusted as the prior. | **v0.13.0 (this milestone)** |
| **L2** | L1 + a periodic full-resync of cache against backend (e.g., on `reposix sync` or every Nth push) to catch desync. | **v0.14.0** — design rationale at `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` § "L2/L3 cache-desync hardening" |
| **L3** | L1 + cache invariants enforced at write time so desync becomes impossible by construction. Cache writes are transactional with backend writes; no path for cache to fall behind silently. | **v0.14.0** — same v0.14.0 doc |

**Recommendation.** Ship **L1** as part of v0.13.0 — it's the simplest and unblocks the bus remote's promise of cheap pushes. Add a `reposix sync --reconcile` command (cheap to add) that does a full `list_records` walk on demand, so a user who suspects desync has an escape hatch. **L2 and L3 are scoped for v0.14.0**; the trade-off (L2 = background reconcile job, L3 = transactional cache writes wired into every adapter) and the desync-telemetry plan that gates which one ships are detailed in the v0.14.0 vision doc. The short version: ship L1 now, collect desync incidence via v0.14.0's OTel work, decide L2-vs-L3 based on whether the rate is "user-visible" (need L2's background resync) or "rare-but-catastrophic" (need L3's invariants).

This means the bus remote algorithm in §3 simplifies to:

```
3. CHEAP PRECHECK B — SoT drift:
     backend.list_changed_since(last_fetched_at) on confluence
     for each changed record: check against the version in our cache's prior tree
     mismatch (record changed AND we're trying to push it) → reject with detailed error
     no overlap (records changed but not in our push) → continue, but update cache
     after this step so subsequent pushes have the fresh prior

5. (formerly the list_records walk) — REMOVED. The check at step 3 is the
   single conflict-detection mechanism.
```

Step 5's removal also collapses the bus-remote's net REST cost on the success path to **one call** (`list_changed_since`) plus the actual REST writes — same as the current single-confluence push's *minimum* cost, but achieved unconditionally rather than only on the failure path.

**Decision required from the planner.** Either:

- **(a)** Treat the L1 migration as a v0.13.0 phase blocker (probably phase N+2 or N+3, before bus remote ships). This means v0.13.0 also delivers a per-push cost improvement to single-backend confluence pushes, which is a nice secondary value.
- **(b)** Ship the bus remote with the inherited `list_records` walk as a known-inefficiency, file the L1 migration as a v0.13.0 GOOD-TO-HAVES item or a v0.14.0 phase, and ensure the `dvcs-topology.md` doc explicitly warns about the per-push REST cost on large spaces.

**Strong recommendation: (a).** The DVCS thesis is "DVCS at the same UX as plain git." Plain git's `git push` does ~3 REST round-trips. Bus-remote `git push` doing 100+ REST calls on every push violates that promise loudly enough that a cold reader will dismiss reposix as a toy. Fix the inefficiency as part of the DVCS milestone, not after.

**Subtlety to also document in `crates/reposix-remote/src/main.rs::handle_export`.** Even if the planner picks (b), add a comment at line 334 (the `list_records` call site) noting:

> NOTE: this is a full backend enumeration on every push. See
> `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance
> subtlety: today's `list_records` walk on every push" for context and
> the planned migration to `list_changed_since`-based conflict detection.
> L2/L3 hardening lives in
> `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`.

Future agents reading the helper code shouldn't have to rediscover the cost-vs-correctness tradeoff from scratch.
