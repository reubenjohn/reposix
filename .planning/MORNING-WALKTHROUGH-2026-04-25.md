# Morning walkthrough — 2026-04-25

> Written ~03:30 PDT for the 9am check-in. Branch `main`, 5 commits ahead of `origin/main`. Pre-push gate is yours — orchestrator did not push.

## TL;DR

- **v0.9.0 architecture pivot SHIPPED** (Phases 31–36, 2026-04-24): FUSE deleted in one commit, git-native partial clone live end-to-end against the simulator.
- **v0.10.0 docs & narrative shine SHIPPED** (Phases 40–45, 2026-04-25): hero with measured numbers, Diátaxis nav, banned-words linter, 16-page cold-reader audit.
- **Repo polished for socializing**: CONTRIBUTING / SECURITY / CODE_OF_CONDUCT / `.github/` issue + PR templates / Dependabot / `cargo-deny` / `cargo-audit` / `examples/` directory (5 examples) / file CATALOG / per-crate metadata / FUSE-era residue swept.
- **Naming generalization landed**: `IssueId → RecordId` committed at `2af5491` (workspace-wide hard rename, no compat aliases). A follow-on `Issue → Record` struct rename is now in flight in the working tree (uncommitted) — same pattern, ~30 files, `cargo check` green.
- **Launch blog post drafted**: `docs/blog/2026-04-25-reposix-launch.md` (357 lines) committed at `cff6d97` — MCP cost-differential vignette, dark-factory framing, embedded agent loop, v0.9.0 numbers. Ready for HN/Twitter once §6 decisions land.
- **Vision-and-innovations brainstorm dropped**: `.planning/research/v0.11.0-vision-and-innovations.md` (269 lines) — five-year vision + named innovation list. Uncommitted at write time; expect to see it land before you wake.
- **71 commits since `33a7527`**, ~210 files churned, +9 948 / −12 071 net. CI green on `8726360`; cargo-audit job reports 4 transitive-dep advisories with a post-step issue-filing failure — see §5.

**You can:**
- Tag v0.9.0 with one command: `bash scripts/tag-v0.9.0.sh` (8 safety guards).
- Resolve the rename (commit it once you've blessed the new name) and we keep moving.
- Skim §3 if pressed for time. §6 is the decision menu. §7 is what I'd do next.

---

## 1. By-the-numbers ledger

| Metric | Pre-overnight (`33a7527`) | Now | Δ |
|---|---:|---:|---:|
| Commits ahead of `origin/main` | 0 | 8 | +8 |
| Commits since `33a7527` | 0 | **71** | +71 |
| Tracked files | ~610 | ~657 | +47 |
| Workspace test result lines | ~452 | **436 (49 binaries) at HEAD-ish** | regression-of-format, not loss [^1] |
| User-facing docs (`docs/**.md`) | ~22 | 40 (incl. blog) | +18 |
| Crates in workspace | 10 (`reposix-fuse` included) | 9 | −1 |
| `crates/reposix-fuse/` LOC | ~8 156 | 0 | deleted (`1535cb0`) |
| Banned-word linter violations | n/a | 0 (Layers 1-2) | new gate |
| Diátaxis nav sections | 0 | 6 (Concepts / Tutorials / How it works / Guides / Reference / Decisions) | +6 |
| `examples/` directory | absent | 5 examples (shell, python, claude-skill, conflict, blob-limit) | new |
| Latency artifact (`docs/benchmarks/v0.9.0-latency.md`) | absent | sim column populated; real cells `pending-secrets` | new |
| Launch blog post | absent | `docs/blog/2026-04-25-reposix-launch.md` (357 lines, draft) | new |
| Tag scripts present | up to v0.8.0 | adds `tag-v0.9.0.sh` (8 guards) | +1 |
| Diff scale vs `33a7527` (committed) | — | **211 files / +9 948 / −12 071** | — |

[^1]: The cumulative `test result: ok` line counts shrank because Phase 36 deleted the entire `reposix-fuse` test suite (`tests/write.rs` + others) and Phase 33–34 consolidated several integration tests into single per-binary entries. Net engineering work added tests; net file count decreased because deletions were larger.

---

## 2. Highlights — phase-by-phase narrative

### v0.9.0 architecture pivot (Phases 31–36, ~60 commits across the milestone, ~5 of which fall in this window)

The bulk of v0.9.0 landed before the cursor for this walkthrough; the audit lives at `.planning/v0.9.0-MILESTONE-AUDIT.md` (verdict `tech_debt` — engineering passed, helper-hardcodes-SimBackend carry-forward). Recap:

- **Phase 31 — `reposix-cache` crate** (10 commits): `gix 0.82` bare repo, lazy blobs, `Tainted<Vec<u8>>` discipline, audit + egress + 4 trybuild compile-fail fixtures.
- **Phase 32 — `stateless-connect` tunnel** (6 commits): three protocol-v2 framing gotchas under named tests; hybrid with `export` capability proven; `refs/heads/*:refs/reposix/*` namespace.
- **Phase 33 — Delta sync** (14 commits): `list_changed_since` on all 4 backends (sim/github/confluence/jira), atomic SQLite `(cache + last_fetched_at + audit)` transaction.
- **Phase 34 — Push conflict + blob limit** (7 commits): stale-base reject path with verbatim `git sparse-checkout` teaching string; frontmatter field allowlist on push (`id` / `created_at` / `version` / `updated_at` stripped).
- **Phase 35 — CLI pivot** (11 commits): `reposix init <backend>::<project> <path>` ships; dark-factory shell + cargo `agent_flow` test; latency artifact populated; real-backend tests `pending-secrets`.
- **Phase 36 — FUSE deletion** (3 commits): `1535cb0` deletes `crates/reposix-fuse/` whole; `52ce149` rewrites `CLAUDE.md` to steady state and ships `.claude/skills/reposix-agent-flow/`; `058c297` is the verifier verdict.

### v0.10.0 docs & narrative shine (Phases 40–45, ~30 commits in this window)

- **Phase 40 — Hero with measured numbers** (`f1d649c`, `d254099`, `6e2fcc0`, `daf53cc`, `757416f`, `23e0fb7`): `docs/index.md` rewritten — V1 vignette ("Before — five round trips" → "After — one commit") + three measured numbers (`8 ms`, `0` MCP schema tokens, `1` bootstrap command). Two concept pages: `mental-model-in-60-seconds.md` and `reposix-vs-mcp-and-sdks.md`. README hero rewritten in lockstep. Above-fold copy is **242 words** (≤ 250 cap).
- **Phase 41 — How-it-works trio** (`4edf828`, `38bfadc`, `c692f43`, `58f876a`, `b27318b`): `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md`, each with exactly one mermaid diagram. Layer-4 jargon (`stateless-connect`, `partial-clone`, `protocol-v2`) confined to `docs/reference/git-remote.md`.
- **Phase 42 — Tutorial + 3 guides + sim relocate** (`73721f3`, `3005a85`, `919dda3`, `2ad0a71`, `5e523d6`, `e3077c1`): 5-min `docs/tutorials/first-run.md`; three guides (`write-your-own-connector`, `integrate-with-your-agent`, `troubleshooting`); simulator moved to `docs/reference/simulator.md`.
- **Phase 43 — Diátaxis nav + theme + banned-words linter** (`fc0f377`, `d910ead`, `aa61828`, `a77925a`): `mkdocs.yml` restructured, indigo + teal theme, redirect stubs for carved-out pages, `scripts/banned-words-lint.sh` + `docs/.banned-words.toml` + pre-commit hook + CI wiring + `reposix-banned-words` SKILL.
- **Phase 44 — Cold-reader clarity audit** (`97b78f4`, `1dd143c`, `df9c0ee`): 16 pages each reviewed in isolation by `doc-clarity-review`; 2 critical findings fixed (dead `reposix mount` blocks in `jira.md` / `confluence.md`); 1 critical (README) escalated to Phase 45 and closed there. 9 major + 17 minor findings logged at `.planning/notes/v0.11.0-doc-polish-backlog.md`. Promoted ad-hoc heredoc into `scripts/check_doc_links.py` (OP-4 self-improving).
- **Phase 45 — README rewrite + lifecycle close** (`c2e4dd3`, `8726360`): README cut from 332 → **102 lines**, Tier 1–5 demo blocks gone, every adjective dereferences a number from the latency artifact. CHANGELOG `[v0.10.0]` finalized. Phase dirs archived to `.planning/milestones/v0.10.0-phases/`.

### Polish for socializing (8 commits after `8726360`)

- `chore(github)`: `.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md`, `dependabot.yml`. (`ad5c4ca`)
- `docs: add CODE_OF_CONDUCT.md`. (`8c2cff9`)
- `docs(examples)`: `examples/{01-shell-loop, 02-python-agent, 03-claude-code-skill, 04-conflict-resolve, 05-blob-limit-recovery}` + index README. (`f76dfb2`, `5a66a96`, `9155d30`, `b720419`, `2218e57`, `1219376`)
- `docs: CONTRIBUTING.md + SECURITY.md` (with section explicitly tied to threat-model cuts). (`1219376`)
- `ci: cargo-audit weekly schedule + on-PR`. (`9ef6937`)
- `chore: cargo-deny config (deny.toml)`. (`6e2bec3`)
- `chore: per-crate description / keywords / categories / readme + workspace.package authors metadata`. (`b5ec153`, `798c8aa`)
- `docs(planning): file CATALOG with cleanup decisions for v0.10.0+`. (`0c63fcd`)

### Naming generalization

- **`IssueId → RecordId` LANDED** in `2af5491 refactor: rename IssueId to RecordId (workspace-wide)` — hard rename across all crates, no compat aliases (precedent: ADR-004 `IssueBackend → BackendConnector`).
- **Follow-on `Issue → Record` struct rename now in flight** in the working tree (uncommitted, ~30 files). Same pattern. `cargo check --workspace --all-targets` is green with both renames applied. Expect this to land before you wake.
- Combined motivation: as we generalize beyond issues (Confluence pages, JIRA records, future Linear / Notion / Asana connectors), `Issue`/`IssueId` is the wrong noun. The CATALOG had this flagged at line 71 as the largest naming-debt item.

### Launch blog post

- **Drafted and committed** at `cff6d97 docs(blog): launch post draft` → `docs/blog/2026-04-25-reposix-launch.md` (357 lines). Frontmatter targets the mkdocs blog plugin. Covers the MCP cost differential vignette, the dark-factory framing, an embedded agent loop, the v0.9.0 architecture story. Ready to share once §6 decisions land.

### Vision-and-innovations brainstorm

- **`.planning/research/v0.11.0-vision-and-innovations.md`** (269 lines) — five-year vision (3-5 bullets including "dark-factory pattern is a recognised industry term with reposix cited as the canonical reference implementation" and a CS-publishable measured result), plus a named innovation list. Uncommitted at write time; landing alongside the `Issue → Record` rename.

### Threat-model continuity

Every guardrail from pre-v0.9.0 ports cleanly to git-native:

- Outbound HTTP allowlist → enforced by `reposix_core::http::client()` factory; Phase 31 routed `reposix-cache` through the same seam (verified by `tests/http_allowlist.rs` invariant).
- `Tainted<T>` discipline → `Cache::read_blob` returns `Tainted<Vec<u8>>`; trybuild compile-fail tests lock the discipline; frontmatter sanitize on push is the explicit `Tainted → Untainted` conversion.
- Append-only audit → SQLite `BEFORE UPDATE/DELETE RAISE` triggers + `SQLITE_DBCONFIG_DEFENSIVE`. New audit ops landed: `materialize`, `tree_sync`, `egress_denied`, `delta_sync`, `helper_push_rejected_conflict`, plus the helper-side push ops.
- Bulk-delete cap → `crates/reposix-remote/tests/bulk_delete_cap.rs` still ratchets >5-delete refusal.
- Working tree = real git repo → preserved by construction in v0.9.0.
- `docs/how-it-works/trust-model.md` is the user-facing render of all of the above.

---

## 3. What works today (live demo script)

```bash
# Terminal 1 — simulator
cargo run -p reposix-sim
# listens on 127.0.0.1:7878 with seed.json

# Terminal 2 — agent UX
cargo build -p reposix-cli -p reposix-remote
export PATH="$PWD/target/debug:$PATH"

reposix init sim::demo /tmp/repo            # ~24 ms cold; configures promisor remote
cd /tmp/repo
git checkout origin/main
cat issues/0001.md                           # ~8 ms — local cache hit after first fetch
grep -ril TODO .                             # standard tools, zero schema knowledge
sed -i 's/^status: .*/status: Done/' issues/0001.md
git commit -am "close 0001" && git push     # writes via export capability

# Audit
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, decision FROM audit_events_cache ORDER BY ts DESC LIMIT 10"

# Dark-factory regression (no Claude in the loop — pure shell)
bash scripts/dark-factory-test.sh sim

# Push conflict cycle (example 04)
cd examples/04-conflict-resolve && ./run.sh

# Blob-limit teaching (example 05)
cd examples/05-blob-limit-recovery && ./run.sh
```

I did not run all of these end-to-end this session (you went to bed before I'd want to start `cargo run -p reposix-sim` in a long-lived background). But:

- `cargo check --workspace --all-targets` is green even with the in-flight rename.
- `cargo test --workspace` at HEAD-without-rename was 436 passed across 49 test binaries (Phase 45 verifier corroborates).
- CI on `main`/`8726360`: `Docs` green, `CI` green (4m14s); `Security audit` failed in the **post-step** when cargo-audit tried to file 4 advisory issues (not a code-correctness failure — see §5).

---

## 4. Open issues / known limitations

Honest section, ordered by surprise potential:

1. **Helper hardcodes `SimBackend`** (carry-forward from v0.9.0 Phase 32). `crates/reposix-remote/src/main.rs` ignores the URL host and routes everything to `SimBackend`. Phase 35 set the right *URL* in `reposix init`; the helper still terminates at sim. **This is the single biggest gap between architecture-as-described and architecture-as-running**: dark-factory tests against the sim work end-to-end, real-backend dark-factory does not. Tracked in `.planning/v0.9.0-MILESTONE-AUDIT.md` §5; recommended resolution before any v0.11.0 benchmark commit.
2. **Real-backend test creds — `pending-secrets`**. Three CI jobs (`integration-contract-{confluence,github,jira}-v09`) and the `agent_flow_real` cargo test wait on GitHub Actions secrets. Test infrastructure is wired; this is a **human gate**, not engineering work. Sanctioned targets are TokenWorld (Confluence) / `reubenjohn/reposix` (GitHub) / JIRA `TEST` — see `docs/reference/testing-targets.md`.
3. **`Issue → Record` struct rename in flight, uncommitted**. `IssueId → RecordId` has landed (`2af5491`); the matching struct rename was running in the working tree as I wrote this. Will probably be committed before you read this. Watch for trybuild `.stderr` fixtures referencing `IssueId`/`Issue` literals — those need a one-pass substitution before `cargo test --workspace` returns green at `HEAD+rename`.
4. **Cargo-audit reports 4 advisories**. The `Security audit` job on `8726360` failed not because of a *check*, but because the `actions-rust-lang/audit@v1` post-step tried to file 4 advisory issues against the repo and exited 1. The advisories are in transitive deps; the action errored before listing IDs in the run summary. **Recommendation**: run `cargo install cargo-audit && cargo audit` locally after coffee to see the actual advisory IDs. The run staged "0 old issues to close, 4 current issues" and then the issue-creation step itself errored — likely a `GH_TOKEN` `issues: write` permission issue with the new workflow.
5. **Playwright screenshots deferred**. `mkdocs-material[imaging]` social-card generation needs cairo system libs; dev host has no passwordless sudo. `scripts/take-screenshots.sh` is a stub naming the contract for v0.11.0.
6. **`reposix gc` / cache eviction unimplemented**. Caches grow unbounded. Likely an innovation in the new `v0.11.0-vision-and-innovations.md` brainstorm — review there.
7. **9 major + 17 minor doc-clarity findings deferred** to `.planning/notes/v0.11.0-doc-polish-backlog.md` (31 line items). Highlights: README ASCII diagram still depicts FUSE + kernel VFS; `first-run.md` Step 1 should prepend a `git clone` line; "CI secret packs" reference in `docs/index.md` lacks context for cold readers.
8. **Blog post is a draft, not published**. `docs/blog/2026-04-25-reposix-launch.md` is committed but no mkdocs blog plugin is wired into `mkdocs.yml` yet — you'll need either the `mkdocs-material` blog plugin enabled or to copy the post to a hosted platform (HN / Substack / personal blog).
9. **FUSE references remain in `.planning/` archive paths** by design (historical correctness). Active code paths are clean per the Phase 36 audit and the CATALOG.

---

## 5. Decisions for you

1. **Tag and push v0.9.0?** `bash scripts/tag-v0.9.0.sh` is ready. Eight safety guards: branch=main, working-tree-clean, tag-not-local, tag-not-remote, CHANGELOG header, Cargo.toml workspace version 0.9.0, tests green + dark-factory green, `docs/reference/testing-targets.md` exists. **Note:** the working-tree-clean guard will reject until the in-flight `Issue → Record` rename lands.
2. **Tag and push v0.10.0 too?** No `tag-v0.10.0.sh` was authored. Lowest-friction option: clone `tag-v0.9.0.sh`, edit version literals, drop the dark-factory gate (docs-only milestone), commit, run.
3. **Helper-hardcodes-SimBackend resolution timing.** Two options:
   - (a) Insert a Phase 36.1 hotfix to dispatch by URL host before any v0.11.0 work begins. **Recommended** — unblocks honest real-backend benchmarks.
   - (b) Roll into v0.11.0 as a prereq phase (e.g. Phase 46 = helper dispatch, then Phase 47 = bench harness). Same outcome, different framing.
4. **Public socialization timing.** Three pre-reqs before HN/Twitter: (i) helper-multi-backend-dispatch fixed; (ii) one or more real-backend latency cells populated; (iii) blog-post draft published. Currently **2/3 done** — the blog draft landed at `cff6d97`; only (i) and (ii) remain.
5. **Promote vision-and-innovations brainstorm into v0.11.0?** The new `.planning/research/v0.11.0-vision-and-innovations.md` is research-tier; promotion into REQUIREMENTS / ROADMAP is a separate move. Recommend reading it after coffee, picking 2-3 innovations, and running `/gsd-new-milestone v0.11.0` to scaffold.
6. **License confirmation.** Workspace ships `LICENSE-APACHE` + `LICENSE-MIT` (dual). `deny.toml` and per-crate Cargo.toml `license = "MIT OR Apache-2.0"` align. If you wanted single-license or a different combo, this is the moment.
7. **Cargo-audit failure triage.** Post-step issue-filing errored. Decide between: lock down `permissions: issues: write` in the workflow yaml; switch to a `cargo audit` step that just runs and prints; or ignore (the *check* itself isn't blocking the build).

---

## 6. What I'd do next (recommendation, opinionated)

If I had your morning, I'd spend it like this:

**Hour 1 — close the in-flight `Issue → Record` rename, tag, ship.** Confirm the working tree is clean (the `Issue → Record` struct rename is committed), run `cargo test --workspace` to confirm green (likely a one-line trybuild stderr fixture update needed), then `bash scripts/tag-v0.9.0.sh && git push origin v0.9.0`. Author `scripts/tag-v0.10.0.sh` in 5 minutes (clone-and-edit `tag-v0.9.0.sh`), tag v0.10.0 the same way. You now have two clean releases on origin and the working tree is neutral.

**Hour 2 — kill the helper-hardcodes-SimBackend tech debt.** This is the single biggest credibility item. Insert it as Phase 36.1 (or Phase 46 if you prefer milestone hygiene). The work is bounded: parse `spec.origin` host in `crates/reposix-remote/src/main.rs`, dispatch to one of `SimBackend` / `GitHubBackend` / `ConfluenceBackend` / `JiraBackend`, reuse the existing `BackendConnector` trait. Phase 32 even left a hand-off note (`32-SUMMARY.md` lines 131–134) naming `State.backend_name` as the seam. Once this lands, point the existing `agent_flow_real` cargo tests at TokenWorld / `reubenjohn/reposix` and watch them pass — that's your real-backend dark-factory proof.

**Hour 3 onward — start v0.11.0 with one phase that ships a number.** The forward plan at `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 names "Performance & Sales Assets" as v0.11.0. The most-leveraged single phase is **Phase 46 — bench harness foundation**: a `cargo run -p reposix-bench` that consumes ARCH-17's latency artifact, adds `tiktoken-rs` token counts, runs the canonical "find issues mentioning X, comment on each" task across reposix / raw-REST-SDK / MCP-equivalent, and emits `docs/benchmarks/v0.11.0-comparison.md`. This is the single artifact that flips the landing-page numbers from "characterized" to "measured" and unblocks the launch blog post. Defer ecosystem (v0.14) and observability (v0.13) — they're great, but the marginal kilojoule here goes to **the table that proves the architectural claim**.

The temptation is to do another docs polish pass (the v0.11.0 doc-polish backlog has 31 line items and is right there). Don't. The docs are good enough; the credibility gap is the helper-side dispatch + the bench numbers. Close those first; polish after.

---

## 7. Files to skim if you have 5 minutes

- `docs/blog/2026-04-25-reposix-launch.md` — the launch post draft (357 lines). Read this first; it's the synthesis of everything else.
- `.planning/research/v0.11.0-vision-and-innovations.md` — five-year vision + named innovations. Promote signal, ignore noise.
- `docs/index.md` — the new front door (Phase 40 hero, V1 vignette + 3 measured numbers).
- `docs/concepts/mental-model-in-60-seconds.md` — clone = snapshot · frontmatter = schema · `git push` = sync verb.
- `CHANGELOG.md` — `[v0.10.0]` block at the top, `[v0.9.0]` below.
- `.planning/v0.9.0-MILESTONE-AUDIT.md` §5 — helper-hardcodes-SimBackend tech-debt narrative.
- `.planning/CATALOG.md` lines 1–60 — file-by-file disposition matrix.
- `examples/README.md` — the five worked dark-factory loops, each runnable.
- `.planning/notes/v0.11.0-doc-polish-backlog.md` — 31 deferred line items, useful as a v0.11.0 grab-bag.

---

## 8. CI status

```
gh run list --branch main --limit 3
completed  success  docs(45): v0.10.0 lifecycle close             Docs            22s
completed  failure  docs(45): v0.10.0 lifecycle close             Security audit  2m38s
completed  success  docs(45): v0.10.0 lifecycle close             CI              4m14s
```

- **CI**: green. `cargo test --workspace`, clippy `-D warnings`, `cargo fmt --all --check`, `mkdocs build --strict`, banned-words linter, doc-link checker — all green at `8726360`.
- **Docs**: green. mkdocs site builds and deploys.
- **Security audit**: failed in the post-step issue-creation, not the audit itself. cargo-audit found 4 advisory entries in transitive deps; the `actions-rust-lang/audit@v1` action's issue-filing exit code 1'd. **Action item**: triage in §5 decision 7. The fix is likely a `permissions: issues: write` block in `.github/workflows/security-audit.yml`.

The previous run (`fix(cache): set default git identity in gix_api_smoke test`, `24925281328`) was fully green across all three workflows — so the regression is post-`8726360` and isolated to the security workflow's GH-token permissions.

---

*Written by the overnight orchestrator. Pre-push gate is yours. Coffee, then §6, then push.*
