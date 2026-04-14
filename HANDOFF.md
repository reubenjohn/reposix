# HANDOFF — overnight mission brief

> You are the next coding agent. This repo shipped v0.1 and v0.2 in two sessions today (see [`MORNING-BRIEF.md`](MORNING-BRIEF.md)). **Your mission tonight: add a Confluence adapter** using the same pattern the GitHub one follows. You have until **08:00 tomorrow** and the user will be asleep. Go crazy.

---

## 0. Before you do anything

```bash
git pull origin main
cat MORNING-BRIEF.md         # 5-minute orientation
cat PROJECT-STATUS.md        # timeline + invariants
cat CHANGELOG.md             # what shipped + what's deferred
cat CLAUDE.md                # the operating rules the repo expects
cat .planning/PROJECT.md     # core value + requirements
```

**Then invoke GSD.** Paste this block into your first turn verbatim; it's the prompt the human would paste if they were typing it themselves:

---

### :rocket: Your overnight invocation

> **I want to continue the reposix project.** Read [`HANDOFF.md`](HANDOFF.md) end-to-end, then [`MORNING-BRIEF.md`](MORNING-BRIEF.md), then [`PROJECT-STATUS.md`](PROJECT-STATUS.md), then [`CHANGELOG.md`](CHANGELOG.md), then the existing [`GithubReadOnlyBackend`](crates/reposix-github/src/lib.rs) and [`IssueBackend` trait](crates/reposix-core/src/backend.rs) since that's the pattern you'll extend. After that read [`AgenticEngineeringReference.md`](AgenticEngineeringReference.md) for the dark-factory ethos and [`InitialReport.md`](InitialReport.md) for the POSIX-over-REST thesis the user originally handed me.
>
> Your mission: **add a Confluence adapter** that implements `IssueBackend` (or a new `PageBackend` trait — your call, see §3 below). Use the Atlassian Teamwork Graph API (an API token is already in `.env` as `TEAMWORK_GRAPH_API`; it's gitignored — do not echo the value anywhere persistent). Get to parity with the GitHub adapter: read-only `list_issues` + `get_issue` + a wiremock unit test suite + a parameterized contract test against real Atlassian + a Tier 3 parity demo + a `reposix mount --backend teamwork` FUSE path.
>
> **Use GSD** for everything: `/gsd-add-phase` then `/gsd-plan-phase` then `/gsd-execute-phase` then `/gsd-code-review` then `/gsd-verify-work`. Skip only the discuss step (`workflow.skip_discuss: true` in `.planning/config.json`). Aggressively leverage subagents for parallelism — today's build hit ~7× wall-clock speedup from parallel executors on disjoint crates. The user's global [`~/.claude/CLAUDE.md`](~/.claude/CLAUDE.md) Operating Principles are bible.
>
> **Go crazy. You have until 08:00. Demo ready for me in the morning.**

---

## 1. What's already shipped (so you don't rebuild it)

- [x] `IssueBackend` trait in `reposix-core::backend` with `list_issues` / `get_issue` / `create_issue` / `update_issue` / `delete_or_close` + `supports(BackendFeature)` + `DeleteReason`. [`backend.rs`](crates/reposix-core/src/backend.rs)
- [x] `SimBackend` — the simulator-as-first-class-backend. [`backend/sim.rs`](crates/reposix-core/src/backend/sim.rs)
- [x] `GithubReadOnlyBackend` — read-only real-GitHub adapter with Link-header pagination, `x-ratelimit-reset` backoff, 14 wiremock tests. [`reposix-github/src/lib.rs`](crates/reposix-github/src/lib.rs)
- [x] `reposix list --backend {sim,github}` and `reposix mount --backend {sim,github}` — end-to-end CLI commands that work against real GitHub. [`cli/src/list.rs`](crates/reposix-cli/src/list.rs), [`cli/src/mount.rs`](crates/reposix-cli/src/mount.rs)
- [x] Contract test parameterized over both backends. [`reposix-github/tests/contract.rs`](crates/reposix-github/tests/contract.rs) — **your new adapter MUST be added to this test.**
- [x] Tier 1-5 demos in [`scripts/demos/`](scripts/demos/). Tier 1 runs in CI via `smoke.sh`.
- [x] Swarm harness that drove 132k ops / 0% errors through SimBackend (Phase 9). [`reposix-swarm`](crates/reposix-swarm)
- [x] ADR-001 for GitHub state mapping. [`docs/decisions/001-github-state-mapping.md`](docs/decisions/001-github-state-mapping.md)
- [x] 8 security guardrails (SG-01..08), 169 tests, clippy clean, CI green. `#![forbid(unsafe_code)]` throughout.

## 2. What you're building tonight — the Confluence adapter

**Goal:** `reposix mount --backend teamwork --project SPACE_KEY` lands a Confluence space as a POSIX directory tree. A user can `cat /mnt/reposix/HOME.md` and read the real page.

**Canonical proof command by 08:00:**

```bash
export TEAMWORK_GRAPH_API="$(grep '^TEAMWORK_GRAPH_API=' .env | cut -d= -f2-)"
REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://api.atlassian.com' \
    reposix mount /tmp/reposix-conf-mnt \
        --backend teamwork --project <their-space-key>
ls /tmp/reposix-conf-mnt     # pages as markdown files
cat /tmp/reposix-conf-mnt/HOME.md   # real page body
fusermount3 -u /tmp/reposix-conf-mnt
```

**Suggested phase breakdown (feel free to adjust):**

- **Phase 11-A** — New crate `reposix-teamwork` with `TeamworkReadOnlyBackend` implementing `IssueBackend` (or `PageBackend` — see §3). Wiremock unit tests ≥5. Contract test added to `tests/contract.rs` and `#[ignore]`-gated for the live Atlassian half.
- **Phase 11-B** — `reposix list --backend teamwork` + `reposix mount --backend teamwork` CLI dispatch. Update `list.rs` and `mount.rs` to route to the new backend.
- **Phase 11-C** — ADR-002 for how Confluence pages map onto reposix's issue model (hierarchy handling is the interesting question — see §3).
- **Phase 11-D** — Tier 3B demo `scripts/demos/parity-confluence.sh` + a new Tier 5 demo `06-mount-real-confluence.sh`. Record both via `script(1)`.
- **Phase 11-E** — Docs update (README Tier 3 / Tier 5 tables, `docs/architecture.md` adds Confluence to the crate topology diagram, `docs/reference/` gets a Teamwork Graph API page).
- **Phase 11-F (stretch)** — Swarm harness `--mode teamwork-direct` run against the real Atlassian API (use small N because of rate limits).

## 3. The open design decision — you must pick one

Confluence pages form a **tree** (parent/child hierarchy) unlike GitHub issues (flat). Three choices; each has a clear reason. Pick one, document it in ADR-002:

**Option A — keep `IssueBackend`, flatten the hierarchy into a label.** Every page becomes a file; parent relationship encoded as `parent_id` in frontmatter. Simple; reuses all existing FUSE machinery. Loses the "browse pages via `cd`" UX.

**Option B — add a `PageBackend` trait** with `list_pages(space, parent_id: Option<PageId>) -> Vec<Page>` + `get_page`. Have `reposix-fuse` accept either trait. Cleanest model; more code; biggest impact.

**Option C — extend `IssueBackend` with an optional `parent` field** on `Issue` and let the FUSE layer render subdirectories when parent chains are detected. Middle ground; risks breaking SimBackend + GithubReadOnlyBackend invariants subtly.

**Recommendation:** Option B. The trait boundary is already the project's most important design — adding a sibling trait is on-pattern. [`docs/decisions/001-github-state-mapping.md`](docs/decisions/001-github-state-mapping.md) is the template for ADR-002.

## 4. Credentials handling — this is load-bearing

- **The token** lives in `.env` at the repo root. That file IS gitignored (verified). Do not move it. Do not commit it. Do not echo its value into any committed file, log, or commit message.
- **For every shell command** that touches Atlassian, source the env file: `set -a; source .env; set +a` OR inline via `TEAMWORK_GRAPH_API=... command`.
- **For the CI `integration-contract-teamwork` job** you'll need a repo secret. The user can add `TEAMWORK_GRAPH_API` via `gh secret set TEAMWORK_GRAPH_API` — mention this in the CI YAML as a required-secret comment; DO NOT attempt to set it yourself. Gate the job on `if: ${{ secrets.TEAMWORK_GRAPH_API != '' }}` so other contributors' forks don't fail.
- **Allowlist** — add `https://api.atlassian.com` to the required `REPOSIX_ALLOWED_ORIGINS` doc examples. The compile-time default stays loopback-only; SG-01 invariant holds.

## 5. Invariants you MUST NOT break

1. `cargo test --workspace --locked` green before every push.
2. `cargo clippy --workspace --all-targets -- -D warnings` clean.
3. `cargo fmt --all --check` clean (the `rustfmt` CI job is strict).
4. `bash scripts/demos/smoke.sh` still 4/4 green — Tier 1 demos are load-bearing.
5. `integration-contract` CI job stays strict (`continue-on-error: false`).
6. `#![forbid(unsafe_code)]` on every new crate root.
7. All HTTP via `reposix_core::http::HttpClient` — the clippy `disallowed-methods` lint catches violations.
8. `Tainted::new(...)` on every ingress from the new backend; `sanitize()` on every egress.
9. Never downgrade an existing test to `#[ignore]` to make something pass. Fix the thing.
10. Each commit atomic with a `feat(11-X-N):` / `test(...):` / `docs(...):` / `fix(...):` prefix.

## 6. Patterns that worked today (use them again)

- **Aggressive parallel executors** — Wave A launched demo-suite + trait-extraction in parallel on disjoint crates; 45 min wall-clock for 2h of work. Today you can probably parallelize: (adapter + unit tests) + (CLI dispatch) + (demo scripts) on disjoint files.
- **Close the feedback loop** — before declaring any phase done, run the demo on the dev host. `ls /tmp/reposix-conf-mnt` printing real page files is the test that matters; unit tests alone shipped two silently-broken things today (H-01, H-02 in Phase 8 review) that empirical run would have caught.
- **Plan-check subagent** — catches real blockers before executor starts. Today it caught a merge-collision in Wave 1 that would've cost 15 min of CI cycles.
- **Playwright self-review** — after deploying docs, screenshot them. Confirmed mkdocs + mermaid render correctly; would have missed the `<div class="mermaid">` → fenced-code migration without it.
- **Normalization-of-deviance discipline** — flip `continue-on-error: true` → false the moment a CI job is stable. Phase 8 review caught one; Phase 10 caught another. Keep the ratchet.
- **Empirical benchmarks with conservative fixtures** — `benchmarks/RESULTS.md` measured 92.3% vs the paper's 98.7% claim by using a smaller MCP fixture. Honest beats optimistic.

## 7. References — bookmark these

- **Operating rules:** [`CLAUDE.md`](CLAUDE.md) (repo-local) + [`~/.claude/CLAUDE.md`](~/.claude/CLAUDE.md) (user's global Operating Principles).
- **Project spec:** [`.planning/PROJECT.md`](.planning/PROJECT.md) — core value + 17 active requirements.
- **Roadmap:** [`.planning/ROADMAP.md`](.planning/ROADMAP.md) — phase history; append Phase 11 at the end.
- **Morning brief (today's outcome):** [`MORNING-BRIEF.md`](MORNING-BRIEF.md).
- **Timeline:** [`PROJECT-STATUS.md`](PROJECT-STATUS.md).
- **Architecture ethos:** [`InitialReport.md`](InitialReport.md) (~6000 words on FUSE + git-remote-helper for agents) + [`AgenticEngineeringReference.md`](AgenticEngineeringReference.md) (dark-factory pattern + lethal trifecta).
- **Research consumed today:** [`.planning/research/`](.planning/research/) — FUSE patterns, git-remote-helper protocol, simulator design, threat model.
- **Existing ADR template:** [`docs/decisions/001-github-state-mapping.md`](docs/decisions/001-github-state-mapping.md). Your ADR-002 for Confluence hierarchy goes next to it.
- **Demo suite docs:** [`docs/demos/index.md`](docs/demos/index.md) — Tier structure; add rows for the new Confluence demos.
- **Social assets:** [`social/assets/`](social/assets/) — `hero.png`, `demo.gif`, `architecture.png`, `benchmark.svg` etc. are checked in; add a Confluence-flavored equivalent if you have time.
- **Atlassian docs** (start here):
  - Teamwork Graph API: <https://developer.atlassian.com/cloud/teamwork-graph/> (GraphQL; token auth)
  - Confluence Cloud REST (fallback if Graph is too much in one night): <https://developer.atlassian.com/cloud/confluence/rest/v2/>
  - Rate limits: <https://developer.atlassian.com/cloud/confluence/rate-limiting/>

## 8. Watchouts from today's review findings (don't repeat them)

- **Cargo.lock drift** will tank CI if you add a dep and forget to commit the lockfile. `cargo test --locked` in CI is strict.
- **Axum `http::request::Parts` is not `Clone`** in 0.7 — don't try, use a move-based pattern.
- **Wiremock strict matchers** — when you want to prove "header X is absent", write a custom `Match` impl with `request.headers.contains_key`. A permissive matcher always passes and tests nothing.
- **`mkdocs build --strict`** refuses relative links outside the `docs/` tree. Use absolute GitHub URLs for anything in `scripts/` or `.planning/`.
- **Continue-on-error rot** — check every new CI job. If you set `continue-on-error: true` as a temporary measure, document the trigger to flip it off and do it the moment it's stable.
- **500-issue/page soft cap** in pagination — keep the cap but log a WARN when you hit it. GitHub adapter has the same TODO; fix both if time.

## 9. How to know you've nailed it

By 08:00 tomorrow the user should be able to do, from a fresh clone:

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix
source .env                  # contains TEAMWORK_GRAPH_API
export REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://api.atlassian.com'
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"

# (A) List a real Confluence space from CLI
reposix list --backend teamwork --project THEIR_SPACE_KEY --format table

# (B) FUSE-mount a real Confluence space
mkdir -p /tmp/reposix-conf-mnt
reposix mount /tmp/reposix-conf-mnt --backend teamwork --project THEIR_SPACE_KEY &
sleep 3
ls /tmp/reposix-conf-mnt
cat /tmp/reposix-conf-mnt/*.md | head -50
fusermount3 -u /tmp/reposix-conf-mnt

# (C) The tests
cargo test --workspace --locked   # >=180 passing
bash scripts/demos/smoke.sh        # 4/4 (Tier 1 still green)
bash scripts/demos/06-mount-real-confluence.sh  # new, exits 0
```

That's a successful handoff. Same structure as v0.2 — new adapter, new FUSE path, new demo, new contract, new docs, updated CHANGELOG, new tag `v0.3.0`.

## 10. Final note

This is the third agent to work on this codebase today. The first built v0.1 overnight. The second (me) built v0.2. You're extending to v0.3.

The project deliberately treats each agent as a first-class contributor: every commit is atomic, every phase has CONTEXT + PLAN + DONE + REVIEW + VERIFICATION files, every decision goes into an ADR. That's so the *next* agent — you — can pick it up without asking questions the previous agent already answered.

Return the favor when you hand off to whoever comes next.

— Claude Opus 4.6 1M context, signing off at ~11:50 PDT, 2026-04-13.
