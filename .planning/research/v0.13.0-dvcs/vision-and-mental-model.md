# v0.13.0 — Vision and Mental Model

> **Audience.** The next agent picking up v0.13.0 planning, after v0.12.1 closes. Read this BEFORE running `/gsd-new-milestone v0.13.0`. Sibling doc:
> - `architecture-sketch.md` (sibling in this folder) — the technical design for the three innovations + open questions a planner needs to resolve.
>
> **Supersedes:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo". That earlier plan (OTel spans, `reposix tail`, multi-project helper) is **deferred to v0.14.0**, not cancelled — the consolidated v0.14.0 scope lives at `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`. The reasoning: DVCS is a thesis-level shift; observability is operational maturity for an existing thesis. Ship the thesis shift first while the design space is still fluid.
>
> **Drafted:** 2026-04-29 by the v0.12.1 planning session, while v0.12.1's autonomous run (P72–P77) was in flight. Owner-approved via discussion transcript that day.

## The thing we are building

Today reposix is a **VCS over REST**: one developer (the SoT-holder) initializes a working tree against a single backend (confluence / GitHub Issues / JIRA), and `git push` from that tree translates to REST writes against that one backend. Other developers either install reposix and `init` against the same backend (everyone hits confluence directly), or they don't participate.

The v0.13.0 thesis is **DVCS over REST**: confluence (or any one backend) remains the source of truth, but a **plain-git mirror** on GitHub becomes the universal-read surface for everyone else. Developer B can `git clone git@github.com:org/repo.git` with **vanilla git, no reposix install**, get all the markdown, edit, commit, and push back through a reposix-equipped path that fans out to both the SoT and the mirror.

The litmus test for "we shipped v0.13.0" is the following sequence working end-to-end with no manual sync between the two halves:

```bash
# Dev A (the SoT-holder, has reposix installed)
reposix init confluence::SPACE /tmp/repoA
cd /tmp/repoA
git remote add github git@github.com:org/repo.git
git push github main                        # plain-git mirror of the markdown

# Dev B (no reposix installed yet)
git clone git@github.com:org/repo.git /tmp/repoB
cd /tmp/repoB
cat issues/0001.md && grep -ril TODO .       # works, vanilla git
$EDITOR issues/0001.md && git commit -am 'fix typo'

# Dev B installs reposix, attaches to the SoT, pushes back
cargo binstall reposix
reposix attach confluence::SPACE             # builds local cache from REST,
                                             # reconciles with current HEAD
git remote set-url --add --push origin reposix::bus://confluence::SPACE+github::org/repo
git push origin main                         # writes to confluence (SoT) and GH (mirror) atomically

# Human edits a confluence page directly in the browser.
# Webhook → GH Action → reposix init confluence + git push github = mirror catches up automatically.
# Dev B's next `git pull origin` from the GH mirror sees the change.
```

If a dev can run that sequence and the data round-trips correctly with conflict detection in both directions, v0.13.0 ships.

## Why this is the right next thesis

Three independent pressures converge on DVCS:

1. **Adoption gradient.** "Install reposix to read your team's tracker" is a hard sell for a curious developer. "Read it as a vanilla git repo on GitHub; install reposix only when you want to write back" is much easier. The DVCS topology exposes the read surface to everyone with zero install cost — perfectly aligned with the dark-factory argument that good substrates win by *not making the agent learn anything new*.
2. **The mirror question already exists in the field.** `reposix init confluence /tmp/x && git remote add github && git push github` already works today (modulo blob-limit + Dev B re-attach). Devs *will* do this once they have a working confluence backend. Not having a documented, safe, atomic story means they'll roll their own with `pushurl` + sync scripts and hit split-brain failures we could have prevented.
3. **The pull side is the missing half of the existing read path.** The cache already does `list_changed_since` for delta-sync on `git fetch`. Pulling that into a cross-backend sync story (webhook-driven mirror updates) is the natural extension — same primitive, new wiring.

## Mental model

**Three roles in a v0.13.0 deployment:**

| Role | Tools | Reads from | Writes to |
|---|---|---|---|
| **SoT-holder** (Dev A) | reposix-equipped, attached via `init` | Confluence (cache-backed) | Confluence + GH mirror (atomic via bus remote) |
| **Mirror-only consumer** (Dev B before installing reposix) | Vanilla git only | GH mirror (plain repo) | Cannot write back |
| **Round-tripper** (Dev B after `reposix attach`) | reposix-equipped, attached after the fact | GH mirror for fast clones; confluence for ground truth | Confluence + GH mirror (atomic via bus remote) |

**One source of truth, one mirror, one human edit path:**

```
                       ┌─── webhook on edit ───┐
                       │                       ▼
                  ┌─────────┐             ┌─────────┐
   Dev A push ──► │Confluence│ ◄── pull ──│ GH Action│ ── push ──► ┌─────────┐
   (bus remote)   │  (SoT)  │             │(mirror)  │             │ GH repo │
                  └─────────┘             └─────────┘             │(mirror) │
                       ▲                                           └─────────┘
                       │                                                ▲
                       │          ┌─── git clone (vanilla) ─────────────┤
                       │          │                                      │
                       │          ▼                                      │
                       │     ┌────────┐                                  │
                       └─ push (bus) ─┤  Dev B │── git push (vanilla) ───┘
                                  └────────┘
```

The bus remote on Dev A and Dev B is the only writer to confluence. The GH Action is the only writer to the mirror that didn't come through the bus remote. Webhook latency (~30s typical) is the upper bound on staleness for vanilla-git readers. Reposix-equipped users always see real-time SoT state by fetching directly.

## Success gates

A v0.13.0 release ships when all of these hold:

1. **`reposix attach <backend>::<project>`** is implemented and tested. Takes an existing checkout (no requirement that it was created by `reposix init`), builds a cache by REST-listing the backend, reconciles cache OIDs against the current `HEAD` tree. End-to-end test: clone a vanilla mirror, attach, edit, push to bus remote, see writes land in both confluence and GH.
2. **Bus remote** (`reposix::bus://<sot-spec>+<mirror-spec>`) is implemented with the precheck-then-SoT-first-write algorithm (see architecture sketch §3). Conflict detection works at the cheap-precheck stage and again at the inner write stage. Mirror-write failures after SoT-write success are tracked as "mirror lag" and recoverable on next push, not data loss.
3. **Mirror-lag observability via plain-git refs.** A ref like `refs/mirrors/confluence-synced-at` is updated on each successful bus push (and on each successful webhook-driven sync). Plain-git users can `git log` it to see staleness. The bus remote's reject message points at it when conflict detection trips because of mirror lag.
4. **Webhook-driven mirror sync.** A reference GitHub Action workflow ships in `docs/guides/dvcs-mirror-setup.md` — receives a confluence webhook, runs `reposix init confluence + git push github-mirror`, updates the `refs/mirrors/...` annotation. Latency target: < 60s p95 from confluence edit to GH ref update.
5. **Cold-reader pass on the DVCS docs.** New page `docs/concepts/dvcs-topology.md` (the three roles + the diagram + when to choose each pattern) passes `doc-clarity-review` against a reader who has read only `docs/index.md` and `docs/concepts/mental-model-in-60-seconds.md`.
6. **Dark-factory regression extended.** The existing `scripts/dark-factory-test.sh` adds a third arm: a fresh subprocess agent given only "the repo at git@github.com:org/repo.git mirrors a confluence backend; install reposix, attach, fix the bug in issues/0001.md, push" must complete the full attach + edit + bus-push sequence with zero in-context learning beyond what the helper's stderr teaches.

## Out of scope (explicitly deferred)

- **OTel / `reposix tail` / multi-project helper.** Moves to v0.14.0 — see `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`. These are operational maturity for an existing thesis and don't depend on DVCS shipping; equally, DVCS doesn't depend on them. Explicit decision: ship thesis shift first.
- **Origin-of-truth frontmatter enforcement.** Moves to v0.14.0 — see `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` § "Origin-of-truth frontmatter enforcement". A guardrail (`origin_backend: confluence` rejects pushes to a non-matching backend) that only matters when the bus pattern fans out across **multiple issues backends** (e.g., GH Issues + JIRA simultaneously). The v0.13.0 bus pattern is "one issues backend (SoT) + one plain-git mirror" where this can't go wrong.
- **Sync daemon as a long-running process.** Webhook-driven CI is the v0.13.0 default. A daemon may be added later for backends that don't emit webhooks; not in v0.13.0 scope.
- **Atomic two-phase commit across backends.** The bus remote is "SoT-first, mirror-best-effort with lag tracking," not a true 2PC. Real 2PC would need a coordinator the helper doesn't have. Document the asymmetry; don't try to hide it.
- **Bus remote with N > 2 endpoints.** Algorithm generalizes (sort prechecks by latency ascending, write to SoT first, fan out mirrors), but the v0.13.0 implementation can hardcode 1+1. Generalize when a third endpoint shows up.

## Risks and how we'll know early

- **`reposix attach` reconciliation is harder than it looks.** Building a cache from REST against an arbitrary checkout (not necessarily one ever produced by `reposix init`) means matching backend records to local files by `id` in frontmatter. If frontmatter has been edited offline (renamed `id`, missing `id`, manual record creation by hand), reconciliation is ambiguous. **Early signal:** prototype `attach` against a deliberately-mangled checkout in the first phase; if reconciliation rules need >5 distinct cases, the design needs revisiting.
- **Webhook latency variance.** GitHub Actions cold-start can be 30–60s; some confluence webhooks batch with their own delay. p95 < 60s may be aspirational. **Early signal:** measure end-to-end latency in a sandbox during the webhook-sync phase; if p95 is >120s, document the constraint and tune `refs/mirrors/...` semantics so users have a clear picture of expected staleness.
- **Race between bus remote's GH push and a concurrent webhook-driven sync.** Both write to the GH mirror. Git's atomic ref-update prevents corruption, but interleaved pushes could cause one to fail with non-fast-forward. **Mitigation:** webhook sync uses `--force-with-lease` against the last known mirror ref; bus remote uses ordinary push and retries on transient non-fast-forward.
- **Bus remote complexity attracts bug surface.** New URL scheme + new conflict-detection sequencing + new error messages. **Mitigation:** dedicate one phase purely to fault injection — kill the GH push between confluence-write and ack, kill the confluence-write mid-stream, simulate confluence 409 after precheck passed. Each of these is a row in the new `bus-remote` test suite.

## Tie-back to project-level invariants

- **OP-1 (simulator-first):** all v0.13.0 phases run end-to-end against the simulator. Two simulators in one process serve as "confluence-shaped SoT" + "GitHub-shaped mirror" for tests. Real-backend tests (TokenWorld + reubenjohn/reposix) gate the milestone close, not individual phase closes.
- **OP-2 (tainted by default):** mirror writes carry tainted bytes from the SoT. The GH mirror's frontmatter must preserve `Tainted<T>` semantics — a downstream agent reading from the GH mirror gets the same trifecta protection as one reading from the SoT directly. The `attach` cache also must mark all materialized blobs as tainted.
- **OP-3 (audit log):** every bus-remote push writes audit rows to **both** tables — the cache audit (helper RPC turn) and the backend audit (the SoT REST mutation). The mirror push doesn't write to backend audit (no REST mutation), but does write a cache-audit row noting "mirror lag now zero" or "mirror lag now N."
- **OP-7 (verifier subagent grades GREEN):** every v0.13.0 phase close dispatches the verifier per `quality/PROTOCOL.md`. The DVCS round-trip test is a catalog row in dimension `agent-ux`, kind `subagent-graded`, cadence `pre-pr`.
- **OP-8 (+2 phase practice):** v0.13.0 reserves its last two phases for surprises absorption + good-to-haves polish. The DVCS scope is large enough that something will surface; do not omit the +2 reservation.

## Where to start when you pick this up

1. Read this doc.
2. Read `architecture-sketch.md` (sibling).
3. Skim `crates/reposix-remote/src/main.rs:300-407` (`handle_export`) — that's the existing single-backend conflict detection the bus remote extends.
4. Skim `crates/reposix-cache/src/lib.rs` lazy-materialization path — that's what `attach` builds against an existing checkout.
5. Read `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` — the next milestone's pre-roadmap scope (consolidated observability + origin-of-truth + L2/L3 cache hardening), so the v0.13.0 ROADMAP knows what NOT to absorb when surprises surface. Original source for the observability portion is `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo" (renamed and renumbered when DVCS jumped ahead).
6. Run `/gsd-new-milestone v0.13.0`. Hand the planner this doc + the architecture sketch as inputs.
