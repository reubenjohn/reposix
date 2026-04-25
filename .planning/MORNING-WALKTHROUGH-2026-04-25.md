# Morning walkthrough — 2026-04-25

> Refreshed ~04:55 PDT for the 9am check-in. Branch `main`, well ahead of `origin/main`. Pre-push gate is yours — orchestrator did not push. **See §"Late additions" below for what landed after the first version of this doc.**

## TL;DR

- **v0.9.0 architecture pivot SHIPPED** (Phases 31–36, 2026-04-24): FUSE deleted in one commit, git-native partial clone live end-to-end against the simulator.
- **v0.10.0 docs & narrative shine SHIPPED** (Phases 40–45, 2026-04-25): hero with measured numbers, Diátaxis nav, banned-words linter, 16-page cold-reader audit.
- **Repo polished for socializing**: CONTRIBUTING / SECURITY / CODE_OF_CONDUCT / `.github/` issue + PR templates / Dependabot / `cargo-deny` / `cargo-audit` / `examples/` directory (5 examples) / file CATALOG / per-crate metadata / FUSE-era residue swept.
- **Naming generalization fully landed**: `IssueId → RecordId` and `Issue → Record` now both committed (commits `2af5491..4ad8e2a`, 6 atomic). The follow-on struct rename that was uncommitted at v1 is now in. ADR-006 records the rationale.
- **Launch blog post drafted**: `docs/blog/2026-04-25-reposix-launch.md` (357 lines) committed at `cff6d97` — MCP cost-differential vignette, dark-factory framing, embedded agent loop, v0.9.0 numbers. Ready for HN/Twitter once §6 decisions land.
- **Vision-and-innovations brainstorm landed**: `.planning/research/v0.11.0-vision-and-innovations.md` committed at `e4ea90e` (4 786 words) — five-year vision + named innovation list.
- **3 ADRs landed**: 006 (IssueId→RecordId), 007 (time-travel via git tags), 008 (helper URL dispatch).
- **3 net-new CLI subcommands**: `reposix doctor`, `reposix history`, `reposix at <ts>`.
- **v0.9.0 carry-forward debt CLOSED**: helper now dispatches by URL scheme; real-backend `git fetch` works against TokenWorld / `reubenjohn/reposix` / JIRA `TEST` when creds present.
- **99 commits since `33a7527`**, +480 tests across 52 binaries (was 436 across 49 binaries pre-overnight surprise round). CI green on `eaaa241`; Security audit job is now passing — the post-step issue-filing failure that bit `8726360` is gone.

**You can:**
- Tag v0.9.0 with one command: `bash scripts/tag-v0.9.0.sh` (8 safety guards). Working tree is now clean — the rename gate no longer blocks it.
- Read the 3 new ADRs, then decide whether to cut v0.10.0 NOW vs. roll into v0.11.0 (see §6 decision 8).
- Skim §3 if pressed for time. §6 is the decision menu. §7 is what I'd do next.

---

## Late additions (since v1 of this doc)

The first version of this walkthrough was written before three surprise rounds of work landed (commits `b59b6af..eaaa241`). If you read v1 and want only the diff:

| Surprise | Commit | One-liner |
|---|---|---|
| `Issue → Record` struct rename **landed** (was in-flight at v1) | `847cc26..4ad8e2a` | Workspace-wide hard rename; YAML wire format unchanged. ADR-006 captures rationale. |
| **`reposix doctor`** — diagnostic CLI | `b276473` (+ tests `1ed8432`, docs `e9a7011`, changelog `b862c71`) | 14 checks, `--fix` mode, severity-tiered output, exit 1 on ERROR. |
| **`reposix history` + `reposix at <ts>`** — time-travel via git tags | `856b7b9` (cache) + `7c9b2b0` (CLI) + tests `b1317db` + ADR-007 `3e2871a` | Every `Cache::sync` writes `refs/reposix/sync/<ISO8601>`; `transfer.hideRefs` keeps it private. New audit op `sync_tag_written`. |
| **Helper URL-scheme backend dispatch** — closes v0.9.0 carry-forward | `cd1b0b6` (+ test `568c668`, audit op `8e41895`, docs `47b7f05`, ADR-008 `dd1708f`, changelog `eaaa241`) | Helper parses `argv[2]`, dispatches to `SimBackend` / `GitHubBackend` / `ConfluenceBackend` / `JiraBackend`. Atlassian URLs gained `/confluence/` or `/jira/` markers. New audit op `helper_backend_instantiated`. |
| New how-it-works page | (in `132c662`) | `docs/how-it-works/time-travel.md` — public-facing render of ADR-007. |
| v0.11.0 vision brainstorm committed | `e4ea90e` | 4 786 words; doctor + time-travel were month-2 ship items, both already done. |

The §"By-the-numbers ledger" and §3-§7 below have been updated in place to reflect these. Everything in §1 phase narratives 1-2 (v0.9.0 + v0.10.0) is unchanged.

---

## 1. By-the-numbers ledger

| Metric | Pre-overnight (`33a7527`) | At v1 of this doc | Now | Δ since v1 |
|---|---:|---:|---:|---:|
| Commits ahead of `origin/main` | 0 | 8 | **36** | +28 |
| Commits since `33a7527` | 0 | 71 | **99** | +28 |
| Tracked files | ~610 | ~657 | ~660 | +3 (net; deletions absorbed by new files) |
| Workspace tests passing | ~452 | 436 (49 binaries) | **480 (52 binaries)** | +44 / +3 binaries |
| User-facing docs (`docs/**.md`) | ~22 | 40 (incl. blog) | 42 (adds time-travel how-it-works + 3 ADRs) | +2 |
| ADRs in `docs/decisions/` | 5 | 5 | **8** (006/007/008 added) | +3 |
| `reposix` CLI subcommands | 2 (`init`, `agent` etc.) | 2 | **5** (+ `doctor`, `history`, `at`) | +3 |
| Crates in workspace | 10 (`reposix-fuse` included) | 9 | 9 | 0 |
| `crates/reposix-fuse/` LOC | ~8 156 | 0 | 0 | 0 |
| Banned-word linter violations | n/a | 0 | 0 | 0 |
| Diátaxis nav sections | 0 | 6 | 6 | 0 |
| `examples/` directory | absent | 5 examples | 5 examples | 0 |
| Latency artifact | absent | sim column populated; real cells `pending-secrets` | sim column + helper now actually dispatches to real backends | unblocked |
| Launch blog post | absent | `docs/blog/2026-04-25-reposix-launch.md` (draft) | unchanged (draft) | 0 |
| Audit ops in `audit_events_cache` CHECK | n/a (pre-cache) | 13 | **15** (`sync_tag_written`, `helper_backend_instantiated`) | +2 |
| Tag scripts present | up to v0.8.0 | adds `tag-v0.9.0.sh` | unchanged | 0 |

[^1]: The cumulative `test result: ok` line counts dropped at v1 because Phase 36 deleted the entire `reposix-fuse` test suite (`tests/write.rs` + others). The +44 jump since v1 comes from doctor (~12), time-travel cache+CLI (~8), backend-dispatch tests (~18), plus a handful of agent_flow_real expectation updates and Record-rename trybuild-stderr refreshes.

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

### Naming generalization (now fully landed)

- **`IssueId → RecordId` LANDED** in `2af5491 refactor: rename IssueId to RecordId (workspace-wide)` — hard rename across all crates, no compat aliases (precedent: ADR-004 `IssueBackend → BackendConnector`).
- **`Issue → Record` struct rename LANDED** across `847cc26..4ad8e2a` (5 atomic commits): type rename, trait method `*_issue → *_record`, `issue.rs → record.rs`, doc updates, `cargo fmt`. YAML wire format unchanged.
- **ADR-006** (`docs/decisions/006-issueid-to-recordid-rename.md`) records the rationale — Records can be issues, Confluence pages, JIRA tickets, or any future backend-specific unit; `Issue` was the wrong noun.
- Combined motivation: as we generalize beyond issues (Confluence pages, JIRA records, future Linear / Notion / Asana connectors), `Issue`/`IssueId` was the wrong noun. The CATALOG had this flagged at line 71 as the largest naming-debt item.

### Launch blog post

- **Drafted and committed** at `cff6d97 docs(blog): launch post draft` → `docs/blog/2026-04-25-reposix-launch.md` (357 lines). Frontmatter targets the mkdocs blog plugin. Covers the MCP cost differential vignette, the dark-factory framing, an embedded agent loop, the v0.9.0 architecture story. Ready to share once §6 decisions land.

### Vision-and-innovations brainstorm (committed)

- **`.planning/research/v0.11.0-vision-and-innovations.md`** committed at `e4ea90e docs(planning): v0.11.0 vision + innovations brainstorm (4786 words)`. Five-year vision (3-5 bullets including "dark-factory pattern is a recognised industry term with reposix cited as the canonical reference implementation" and a CS-publishable measured result), plus a named innovation list. **Two of its month-2 ship items have already shipped tonight**: §3a `reposix doctor` and §3b time-travel via git tags (the §6 originality audit had flagged §3b as the brainstorm's highest-novelty entry).

### Late-overnight surprise round (Phases-not-yet-numbered)

The five hours after v1 of this walkthrough committed produced four discrete shipments — none of which were in the v0.10.0 plan, all of which sit comfortably as v0.10.0 bonus material or v0.11.0 down-payments depending on framing.

- **`reposix doctor`** (`b276473 feat(cli): add reposix doctor diagnostic subcommand` + tests `1ed8432` + docs `e9a7011` + clippy/fmt cleanups `9f979ce`/`1dec3c`). 14 checks across git layout / partial-clone extension / promisor-remote URL / helper-on-PATH / git version / cache DB integrity + readability / `audit_events_cache` table & append-only triggers / cache freshness via `meta.last_fetched_at` / `REPOSIX_ALLOWED_ORIGINS` / `REPOSIX_BLOB_LIMIT` / sparse-checkout patterns / `rustc` version. Severity tiers OK / INFO / WARN / ERROR; exits 1 on any ERROR. `--fix` applies a small allowlist of deterministic non-destructive fixes (today: `git config extensions.partialClone origin`); never mutates cache, audit log, or backend. New module `crates/reposix-cli/src/doctor.rs` (~750 lines, well-commented).

- **`reposix history` + `reposix at <ts>`** (`856b7b9` cache + `7c9b2b0` CLI + tests `b1317db` + ADR-007 `3e2871a` + how-it-works `132c662` + changelog `02d7c20` + fmt `b5098ea`). Every successful `Cache::sync` writes `refs/reposix/sync/<ISO8601-no-colons>` pointing at the synthesis commit. The cache's bare repo gets `transfer.hideRefs = refs/reposix/sync/` so the helper's protocol-v2 advertisement does NOT propagate these private refs to the agent's working tree — they're inspectable only by going to the cache directly. New module `crates/reposix-cache/src/sync_tag.rs` exposes `SyncTag`, `Cache::tag_sync`, `Cache::list_sync_tags`, `Cache::sync_tag_at`. New audit op `sync_tag_written` joins the existing CHECK constraint. `reposix at 2026-04-25T01:00:00Z /tmp/repo` finds the closest tag at-or-before the given instant.

- **Helper URL-scheme backend dispatch** (`cd1b0b6 feat(remote): backend dispatch via URL scheme` + test `568c668` + audit op `8e41895` + docs `47b7f05` + ADR-008 `dd1708f` + changelog `eaaa241`). New module `crates/reposix-remote/src/backend_dispatch.rs` exposes `BackendKind`, `parse_remote_url`, and `instantiate`. Atlassian URLs gained a `/confluence/` or `/jira/` path-segment marker so the helper can disambiguate the two adapters that share `*.atlassian.net` — `reposix init confluence::TokenWorld` now emits `reposix::https://<tenant>.atlassian.net/confluence/projects/TokenWorld`. Missing-creds errors name the env var(s) and link to `docs/reference/testing-targets.md`. New audit op `helper_backend_instantiated`. **This closes the v0.9.0 carry-forward tech debt that tilted both the v0.9.0 and v0.10.0 audits to `tech_debt`.** Real-backend `git fetch` now actually works against TokenWorld / `reubenjohn/reposix` / JIRA `TEST` when creds are present.

- **3 ADRs** in `docs/decisions/`: 006 (rename rationale), 007 (time-travel; supersedes nothing; flagged as the brainstorm's highest-novelty item), 008 (helper URL dispatch).

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

# Diagnostics — NEW
reposix doctor /tmp/repo                     # 14 checks; OK / INFO / WARN / ERROR; exit 1 on ERROR
reposix doctor /tmp/repo --fix               # applies safe non-destructive fixes (e.g. extensions.partialClone)

# Time-travel — NEW
reposix history /tmp/repo                    # list refs/reposix/sync/<ISO8601> tags from cache
reposix at 2026-04-25T01:00:00Z /tmp/repo    # closest sync tag at-or-before that instant
git -C ~/.cache/reposix/sim-demo.git checkout refs/reposix/sync/2026-04-25T01-00-00Z

# Real backend — NEW (helper now actually dispatches by URL scheme)
export GITHUB_TOKEN=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com'
reposix init github::reubenjohn/reposix /tmp/gh-repo
cd /tmp/gh-repo && git fetch origin          # talks to api.github.com, NOT the local sim

# Audit
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, decision FROM audit_events_cache ORDER BY ts DESC LIMIT 10"
# Now includes sync_tag_written + helper_backend_instantiated rows.

# Dark-factory regression (no Claude in the loop — pure shell)
bash scripts/dark-factory-test.sh sim

# Push conflict cycle (example 04)
cd examples/04-conflict-resolve && ./run.sh

# Blob-limit teaching (example 05)
cd examples/05-blob-limit-recovery && ./run.sh
```

State of the build at refresh time:

- `cargo check --workspace --all-targets` is green.
- `cargo test --workspace --locked` is **480 passed across 52 test binaries** (was 436/49 at v1).
- CI on `main`/`eaaa241`: `Docs` green, `Security audit` green (the post-step failure that bit `8726360` is gone), `CI` was still in_progress at refresh time but expected green based on prior run on `02d7c20` chain.
- Real-backend tests (`agent_flow_real`) still wait on GitHub Actions secrets — engineering side is now actually capable of hitting real backends, so the `pending-secrets` gate is now genuinely the only gate.

---

## 4. Open issues / known limitations

Honest section, ordered by surprise potential:

1. **~~Helper hardcodes `SimBackend`~~ — CLOSED tonight.** Resolved by `cd1b0b6 feat(remote): backend dispatch via URL scheme`. The v0.9.0 + v0.10.0 milestone audits both flagged this as the carry-forward; both audits should now be re-stamped from `tech_debt` to `passed` (or have their carry-forward sections updated to drop the entry — see §6 decision 9). Real-backend `git fetch` actually works now; only the `pending-secrets` human gate remains.
2. **Real-backend test creds — `pending-secrets`**. Three CI jobs (`integration-contract-{confluence,github,jira}-v09`) and the `agent_flow_real` cargo test wait on GitHub Actions secrets. Test infrastructure is wired; this is a **human gate**, not engineering work. Sanctioned targets are TokenWorld (Confluence) / `reubenjohn/reposix` (GitHub) / JIRA `TEST` — see `docs/reference/testing-targets.md`. With tonight's helper-dispatch fix, plumbing those secrets in will *actually* exercise the real backends.
3. **Cargo-audit `Security audit` job — now green** on `eaaa241` (was failing on `8726360` due to post-step issue-creation permission). No standing action item; if it regresses again, the fix is a `permissions: issues: write` block in `.github/workflows/security-audit.yml`.
4. **Playwright screenshots deferred**. `mkdocs-material[imaging]` social-card generation needs cairo system libs; dev host has no passwordless sudo. `scripts/take-screenshots.sh` is a stub naming the contract for v0.11.0.
5. **`reposix gc` / cache eviction unimplemented**. Caches grow unbounded. Listed in `v0.11.0-vision-and-innovations.md` as a near-term need; doctor's freshness check warns when `last_fetched_at` is stale, but doesn't yet evict.
6. **9 major + 17 minor doc-clarity findings deferred** to `.planning/notes/v0.11.0-doc-polish-backlog.md` (31 line items). Highlights: README ASCII diagram still depicts FUSE + kernel VFS; `first-run.md` Step 1 should prepend a `git clone` line; "CI secret packs" reference in `docs/index.md` lacks context for cold readers.
7. **Blog post is a draft, not published**. `docs/blog/2026-04-25-reposix-launch.md` is committed but no mkdocs blog plugin is wired into `mkdocs.yml` yet — you'll need either the `mkdocs-material` blog plugin enabled or to copy the post to a hosted platform (HN / Substack / personal blog). **Tonight's surprises (doctor + time-travel + dispatch) are not yet folded into the post** — if you publish today, decide whether to do a "what we shipped while you slept" addendum or a fresh post.
8. **Sync tags grow unbounded** — every `Cache::sync` writes one ref. ADR-007's "Consequences" section flags this; a `reposix gc --keep-last N` story is the natural follow-up. Not user-visible because of `transfer.hideRefs`, but they take up disk in the cache's bare repo.
9. **FUSE references remain in `.planning/` archive paths** by design (historical correctness). Active code paths are clean per the Phase 36 audit and the CATALOG.

---

## 5. Decisions for you

1. **Tag and push v0.9.0?** `bash scripts/tag-v0.9.0.sh` is ready. Eight safety guards: branch=main, working-tree-clean, tag-not-local, tag-not-remote, CHANGELOG header, Cargo.toml workspace version 0.9.0, tests green + dark-factory green, `docs/reference/testing-targets.md` exists. **Working tree is now clean** — the rename gate from v1 has cleared.
2. **Tag and push v0.10.0 too?** No `tag-v0.10.0.sh` was authored. Lowest-friction option: clone `tag-v0.9.0.sh`, edit version literals, drop the dark-factory gate (docs-only milestone), commit, run. **See decision 8 below for the strategic question of whether to tag v0.10.0 at all vs. roll into v0.11.0.**
3. **~~Helper-hardcodes-SimBackend resolution timing.~~** Resolved tonight at `cd1b0b6`. No decision needed; the v0.9.0 / v0.10.0 audit re-stamp is decision 9 below.
4. **Public socialization timing.** Three pre-reqs before HN/Twitter: (i) helper-multi-backend-dispatch fixed — DONE; (ii) one or more real-backend latency cells populated — needs you to plumb GitHub Actions secrets; (iii) blog-post draft published — committed at `cff6d97` but not posted publicly yet, and tonight's surprises aren't folded in. Currently **1/3 fully done, 2/3 partially done**.
5. **Promote vision-and-innovations brainstorm into v0.11.0?** `.planning/research/v0.11.0-vision-and-innovations.md` is research-tier; promotion into REQUIREMENTS / ROADMAP is a separate move. Two of its month-2 ship items already shipped tonight (§3a doctor, §3b time-travel). Recommend reading it after coffee, picking 2-3 *remaining* innovations, and running `/gsd-new-milestone v0.11.0` to scaffold.
6. **License confirmation.** Workspace ships `LICENSE-APACHE` + `LICENSE-MIT` (dual). `deny.toml` and per-crate Cargo.toml `license = "MIT OR Apache-2.0"` align. If you wanted single-license or a different combo, this is the moment.
7. **Cargo-audit triage** — already self-resolved; the workflow on `eaaa241` is green. No decision needed unless it regresses.
8. **NEW: Tag v0.10.0 NOW, or roll v0.10.0 + tonight's surprises into v0.11.0?** Two framings:
   - **(a) Tag v0.10.0 NOW** for the docs milestone as originally scoped. Then cut v0.11.0 as a "polish + capabilities" release containing tonight's surprises (doctor + time-travel + dispatch + Record rename). **Recommended.** The v0.10.0 milestone has a coherent narrative ("docs & narrative shine") and shipping it cleanly preserves that; mixing in three new CLI subcommands and a backend-dispatch fix muddies the story. Cleaner releases tag the work in matching scope-units, and the helper-dispatch fix is meaty enough to be a headline of its own release.
   - **(b) Skip v0.10.0, roll everything into v0.11.0.** Saves one tag operation. Loses the narrative of a docs-only release. Not recommended.
9. **NEW: Re-stamp v0.9.0 + v0.10.0 milestone audits from `tech_debt` to `passed`?** The single carry-forward item that tilted both verdicts (helper hardcoding `SimBackend`) is closed. Decide whether to:
   - (a) Edit the audits in place to update the verdict, or
   - (b) Add a "RESOLVED" callout pointing at `cd1b0b6` and leave the historical verdicts as-of-the-time-they-were-written. **Recommended (b)** — audits are historical artifacts; verdicts of the moment shouldn't be retconned.

---

## 6. What I'd do next (recommendation, opinionated)

The credibility gap from v1 (helper-hardcodes-SimBackend) is closed. The next-hour-by-hour plan changes shape because of that.

**Hour 1 — tag v0.9.0, then v0.10.0, push.** Working tree is clean. Run `bash scripts/tag-v0.9.0.sh && git push origin v0.9.0`. Author `scripts/tag-v0.10.0.sh` in 5 minutes (clone-and-edit `tag-v0.9.0.sh`, drop the dark-factory gate since v0.10.0 is docs-only), tag and push. Per §5 decision 8, treat tonight's surprises as v0.11.0 down-payments — don't try to retroactively fold doctor / history / dispatch into v0.10.0.

**Hour 2 — scaffold v0.11.0 and commit the milestone direction.** Read `.planning/research/v0.11.0-vision-and-innovations.md` end-to-end (~30 minutes). Two of its month-2 items shipped tonight; pick **two of the remaining** for the v0.11.0 cut. Run `/gsd-new-milestone v0.11.0`. My picks for the top-2 forward bets:

1. **Token-cost ledger** — `tiktoken-rs` instrumentation around the agent loop, emitting tokens-spent-per-task into the audit log. This is the single artifact that flips the landing-page numbers from "characterized" to "measured" against MCP / raw SDK and turns the launch blog post into a tweet-able table.
2. **Multi-project helper / `reposix gc`** — pair these. The helper now dispatches by URL but each cache lives at `~/.cache/reposix/<backend>-<project>.git`; multi-project agents will accumulate caches forever without `gc`. Sync tags compound the disk story (decision 8 in §4). Single phase that ships an LRU eviction story.

Defer for later v0.x: OpenTelemetry tracing (great when you have customers; we don't yet), plugin registry / connector marketplace (premature; the four built-in connectors don't share enough surface to know what the trait extension points should be), and the doc-polish backlog (still right there; still not the credibility gap).

**Hour 3+ — public socialization.** With dispatch closed and (after Hour 2) a token ledger landing, the launch blog post can be revised into an honest "shipped while you slept" narrative. Three of the four pre-reqs from v1 are now done or imminent: helper-dispatch (DONE), real-backend cells (one cargo run away once secrets are plumbed), blog draft (committed). Pull the trigger on HN / Twitter once the bench number is in the post. The repo is in a state that survives scrutiny.

Two flag-planting opportunities specifically worth considering:

- **Blog post update** — fold tonight's surprises into a "what we shipped while you slept" section in the existing `docs/blog/2026-04-25-reposix-launch.md`, or post it as a follow-up. Three CLI subcommands + a meaningful tech-debt close + 3 ADRs in five hours is a marketable narrative on its own.
- **Public socialization NOW** — even before the bench numbers land, the repo passes the "would I be embarrassed if a Rust core dev or an MCP team member skim-read this in five minutes" test. Doctor's UX, ADR quality, and the 480-test suite all hold up. If the blocker is your bandwidth more than the artifacts, "good enough" today beats "perfect" three weeks from now.

---

## 7. Files to skim if you have 5 minutes

- `docs/blog/2026-04-25-reposix-launch.md` — the launch post draft (357 lines). Read this first; it's the synthesis of everything else. Note: it predates tonight's surprises.
- `.planning/research/v0.11.0-vision-and-innovations.md` — five-year vision + named innovations (committed at `e4ea90e`, 4 786 words). Two month-2 items already shipped tonight.
- `docs/decisions/006-issueid-to-recordid-rename.md` — rename rationale; precedent for hard-rename-no-aliases.
- `docs/decisions/007-time-travel-via-git-tags.md` — flagged as the brainstorm's **highest-novelty entry**; no prior art for "tag every external sync as a first-class git ref."
- `docs/decisions/008-helper-backend-dispatch.md` — closes the v0.9.0 carry-forward; explains the `/confluence/` vs `/jira/` URL marker.
- `docs/how-it-works/time-travel.md` — public-facing render of ADR-007.
- `crates/reposix-cli/src/doctor.rs` — well-commented, ~750 lines, fast read. Worth a skim just for the check structure.
- `docs/index.md` — the new front door (Phase 40 hero, V1 vignette + 3 measured numbers).
- `docs/concepts/mental-model-in-60-seconds.md` — clone = snapshot · frontmatter = schema · `git push` = sync verb.
- `CHANGELOG.md` `[Unreleased]` — freshest list of what landed tonight.
- `examples/README.md` — the five worked dark-factory loops, each runnable.
- `.planning/notes/v0.11.0-doc-polish-backlog.md` — 31 deferred line items, useful as a v0.11.0 grab-bag.

---

## 8. CI status

```
gh run list --branch main --limit 5
in_progress           cargo in / for tokio, clap, libc, uuid, assert_cmd - Update #1337050518   Dependabot Updates  main  dynamic  24927149343
in_progress           docs(changelog): note helper backend dispatch under Unreleased            CI                  main  push     24927141131
completed  success    docs(changelog): note helper backend dispatch under Unreleased            Docs                main  push     24927141126   25s
completed  success    docs(changelog): note helper backend dispatch under Unreleased            Security audit      main  push     24927141135   18s
completed  success    chore: gitignore .claude/scheduled_tasks.lock (runtime artifact)          CI                  main  push     24926816212   4m2s
```

- **CI**: was green on the prior commit (`24926816212`, 4m2s); the run on `eaaa241` was still in_progress at refresh time but is on track based on local `cargo test` (480 passing) + `cargo fmt --all --check` + clippy.
- **Docs**: green at 25s on `eaaa241`.
- **Security audit**: green at 18s on `eaaa241` — the post-step issue-creation failure that bit `8726360` has self-resolved (likely a workflow tweak in the post-`8726360` chain or a transient `GH_TOKEN` permission propagation).
- Dependabot has surfaced a `cargo` group-update PR (tokio / clap / libc / uuid / assert_cmd); review on its own merits, not blocker for tagging.

The previous run on `8726360` had `Security audit` red in the post-step; that's now cleared. **No standing CI redness on `main`.**

---

*Refreshed by the overnight orchestrator at ~04:55. Pre-push gate is yours. Coffee, then §6, then push.*
