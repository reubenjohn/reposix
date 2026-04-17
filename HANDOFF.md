# HANDOFF — v0.7.0 (post-ship) + open problems for the next agent

> **Current state (2026-04-16, session 6+):** v0.7.0 is tagged and pushed. 317+ workspace tests pass.
> **The next agent is YOU.** See [§Open problems for the next agent](#open-problems-for-the-next-agent) — OP-1 through OP-11 are all CLOSED (Phases 13–25). Phase 27+ (JIRA Cloud) is the current direction.
>
> ---
>
> *Historical context:* This doc was started after session 3 (v0.3.0) and augmented through session 6+ (v0.7.0). Previous v0.1/v0.2-era briefs are archived at [`docs/archive/MORNING-BRIEF.md`](docs/archive/MORNING-BRIEF.md) and [`docs/archive/PROJECT-STATUS.md`](docs/archive/PROJECT-STATUS.md). This doc subsumes the old session-2 `HANDOFF.md` (deleted) and the session-3 `MORNING-BRIEF-v0.3.md` (renamed into this file).

## v0.7.0 current state (2026-04-16)

**317+ workspace tests pass.** `cargo clippy --workspace --all-targets -- -D warnings` clean. `mkdocs build --strict` green. `bash scripts/demos/smoke.sh` 4/4.

Backends live: `sim` (default), `github` (read-only), `confluence` (read + write). FUSE mount exposes `issues/` or `pages/` bucket + `tree/` hierarchy overlay (Confluence) + `labels/` read-only overlay + `whiteboards/` and `.attachments/` directories. `reposix refresh` snapshots backend state to git-tracked `.md` files.

**Next direction (Phase 27+):** JIRA Cloud read-only adapter, `BackendConnector` rename, `Issue.extensions` field. See [HANDOFF §Open problems](#open-problems-for-the-next-agent) and `.planning/ROADMAP.md`.

---

## Historical tl;dr (v0.3.0 — Phase 11)

> *The sections below are the original session-by-session augmentation records, preserved for audit/history. Start with [§v0.7.0 current state](#v070-current-state-2026-04-16) above for the actionable summary.*

Phase 11 shipped read-only **Atlassian Confluence Cloud** support end-to-end: adapter crate, CLI dispatch, contract test (parameterized over sim + wiremock + live), Tier 3B + Tier 5 demos, ADR-002, reference docs, a "build-your-own-connector" guide, and a CHANGELOG v0.3.0 block. Workspace is **193/193 passing**, clippy clean, fmt clean, `scripts/demos/smoke.sh` 4/4, `mkdocs build --strict` green. Live-wire verification **ran successfully** tonight against `reuben-john.atlassian.net` space `REPOSIX` (4 seeded pages round-tripped through CLI `list`, and through the **Tier 5 FUSE mount with full `cat` body output** — see §"Live proof captured" below). 2 MEDIUM code-review findings + 3 LOW all fixed. **One late-stage FUSE cache bug** found during live Tier 5 verification and fixed in commit `6cd6e43` — the fix is in this release (see CHANGELOG `[v0.3.0] — Fixed` section).

~~**The one thing left for you to do:** run `bash scripts/tag-v0.3.0.sh` to cut + push the `v0.3.0` annotated tag.~~ **DONE — v0.3.0, v0.4.0, v0.4.1, and v0.5.0 are all tagged and pushed.** See §Session 5 augmentation for current state.

## Live proof captured tonight

Not a plan, not a promise — actual output, captured from the dev host:

```
$ reposix list --backend confluence --project REPOSIX --format table
ID         STATUS       TITLE
---------- ------------ ----------------------------------------
65916      open         Architecture notes
131192     open         Welcome to reposix
360556     open         reposix demo space Home
425985     open         Demo plan

$ reposix mount /tmp/reposix-conf-mnt --backend confluence --project REPOSIX &
$ ls /tmp/reposix-conf-mnt
131192.md  360556.md  425985.md  65916.md
$ cat /tmp/reposix-conf-mnt/131192.md
---
id: 131192
title: Welcome to reposix
status: open
assignee: 557058:dd5e2f19-5bf6-4c0a-be0b-258ab69f6976
created_at: 2026-04-14T04:16:31.091Z
updated_at: 2026-04-14T04:16:31.091Z
version: 1
---
<p>This Confluence space is mounted as a POSIX directory tree by <strong>reposix</strong>.
Each page is a file; <code>cat</code> prints this HTML body; <code>ls</code> lists every page.</p>
<p>This page was seeded during Phase 11 of reposix v0.3.</p>
$ fusermount3 -u /tmp/reposix-conf-mnt   # clean
$ bash scripts/demos/06-mount-real-confluence.sh
… == DEMO COMPLETE ==
```

That's the HANDOFF §9 proof command finishing green against a real Atlassian tenant, using the REPOSIX space I created + seeded for you during the session (plus the space-homepage that Confluence auto-provisions on space creation). Your personal `~TokenWorld` space is untouched and still fetchable via `--project ~TokenWorld` if you want to try it.

## What shipped

| Crate / file | What it does |
|---|---|
| `crates/reposix-confluence/` | Read-only Confluence Cloud REST v2 adapter. 17 wiremock tests; `Tainted<T>` everywhere; redacted `Debug`. Commit `fafec8f`. |
| `reposix list --backend confluence --project <SPACE_KEY>` | Live-verified against `reuben-john.atlassian.net` tonight. Commit `5182b72`. |
| `reposix mount --backend confluence --project <SPACE_KEY>` | FUSE-mounts a Confluence space as `<padded-id>.md` files. Same `Mount::open(Arc<dyn IssueBackend>)` path as `github` and `sim`. |
| `crates/reposix-confluence/tests/contract.rs` | The same contract assertions run against `SimBackend` (always), wiremock (always), and live Confluence (`--ignored`). Commit `f1ec6c1`. |
| `scripts/demos/parity-confluence.sh` + `scripts/demos/06-mount-real-confluence.sh` | Tier 3B + Tier 5 demos; skip cleanly with `SKIP:` if Atlassian env vars absent. Commit `a45b332`. |
| `docs/decisions/002-confluence-page-mapping.md` | ADR for the Option-A flatten decision. |
| `docs/reference/confluence.md` + `docs/connectors/guide.md` | User-facing backend reference + "build-your-own-connector" guide (462 lines). The connector guide previews the **Phase 12 subprocess/JSON-RPC ABI** as the scalable-without-forking successor to the current "fork + add dispatch" model. Commits `234beef`, `4dd73fa`, `eeb8baf`. |
| `CHANGELOG.md` | [Unreleased] block promoted to `[v0.3.0] — 2026-04-14` (this plan, Task 2). |

See [CHANGELOG.md](CHANGELOG.md) `[v0.3.0]` section for the full release notes.

## Connector scalability — the story in one paragraph

The current v0.3 connector story is "fork reposix, add a Cargo dep on `reposix-adapter-<name>`, wire three lines of CLI dispatch." That's fine for the first three adapters (sim, github, confluence) shipped by this repo — it's not fine for the long tail of internal-tracker integrations a real user wants. **Phase 12 lifts the adapter boundary across a subprocess/JSON-RPC ABI** so third parties can ship connectors as standalone binaries: no fork, no Rust-ABI coupling, no recompile of reposix. See [`docs/connectors/guide.md`](docs/connectors/guide.md) — the guide already documents the v0.3 short-term model AND sketches the Phase 12 migration path, so anyone starting an adapter today won't have to throw it away when Phase 12 lands.

## 30-second fix: credentials (if you're starting from a clean `.env`)

> Skip this section if your `.env` already has `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, `REPOSIX_CONFLUENCE_SPACE`. Tonight's session verified those are populated.

1. Visit <https://id.atlassian.com/manage-profile/security/api-tokens>. Create a token if you don't have one.
2. **Note the email at the top-right of that page.** That exact email is `ATLASSIAN_EMAIL` — Atlassian user API tokens are account-scoped, not email-scoped, so the email must match the account the token was issued under. (The session 3 probe originally guessed wrong and burned 4 minutes on it; see [`.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md`](.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md).)
3. Decide a tenant subdomain. Confirm with:
   ```bash
   curl -s -u "$ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY" \
     "https://YOUR_TENANT.atlassian.net/wiki/api/v2/spaces?limit=1" | head -c 200
   ```
   A non-empty JSON response ⇒ you have the right tenant.
4. Pick a space key — the `<KEY>` segment of any page URL `https://<tenant>.atlassian.net/wiki/spaces/<KEY>/...`.

## Prove it works (commands to copy-paste)

```bash
# From the repo root (adjust the path).
cd /home/reuben/workspace/reposix

# (If starting fresh) update .env with the four Atlassian values:
cat > .env <<'EOF'
ATLASSIAN_API_KEY=<your token from step 1>
ATLASSIAN_EMAIL=<the email from step 2>
REPOSIX_CONFLUENCE_TENANT=<your tenant subdomain>
REPOSIX_CONFLUENCE_SPACE=<any space key you can read>
EOF

# Export them into the shell.
set -a; source .env; set +a
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

# Build release binaries (skip if you already have target/release/reposix from last night).
cargo build --release --workspace --bins --locked
export PATH="$PWD/target/release:$PATH"

# (A) List a real Confluence space from the CLI.
reposix list --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" --format table

# (B) FUSE-mount a real Confluence space — this is the money shot from HANDOFF §9 step B.
mkdir -p /tmp/reposix-conf-mnt
reposix mount /tmp/reposix-conf-mnt --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" &
MOUNT_PID=$!
sleep 3
ls /tmp/reposix-conf-mnt | head -20
cat /tmp/reposix-conf-mnt/*.md | head -50
fusermount3 -u /tmp/reposix-conf-mnt
wait $MOUNT_PID 2>/dev/null || true

# (C) The two new demos — both skip cleanly with SKIP: when env is unset.
bash scripts/demos/parity-confluence.sh
bash scripts/demos/06-mount-real-confluence.sh

# (D) Full workspace tests + live contract half + smoke regression.
cargo test --workspace --locked                    # expect 191/191
cargo test -p reposix-confluence --locked -- --ignored  # live half (contract_confluence_live)
bash scripts/demos/smoke.sh                        # expect 4 passed, 0 failed
```

## CI secrets (one-shot)

The `integration-contract-confluence` CI job is already wired into `.github/workflows/`. It activates automatically once these four repo secrets are set:

```bash
gh secret set ATLASSIAN_API_KEY        --body "$ATLASSIAN_API_KEY"
gh secret set ATLASSIAN_EMAIL          --body "$ATLASSIAN_EMAIL"
gh secret set REPOSIX_CONFLUENCE_TENANT --body "$REPOSIX_CONFLUENCE_TENANT"
gh secret set REPOSIX_CONFLUENCE_SPACE --body "$REPOSIX_CONFLUENCE_SPACE"
```

Without them the `if:` clause in the workflow skips the job cleanly (no failure, no proof). With them, every push runs the live contract against your tenant.

## Cutting the tag — DONE

> **All tags pushed.** v0.3.0, v0.4.0, v0.4.1, and v0.5.0 are all tagged and on the remote. CI `release.yml` ran green for each; prebuilt x86_64 + aarch64 Linux binaries are attached to every release. Nothing to do here.
>
> *The original session-3 instructions (tag-v0.3.0.sh, working-tree cleanup steps) are preserved in git history at the commit that first added this section.*

## Known open gaps

Per `~/.claude/CLAUDE.md` OP #6 ("ground truth obsession"), being loud about what's NOT shipped:

- **`atlas_doc_format` → Markdown rendering** is deferred to v0.4. Page bodies in v0.3 are raw Confluence storage XHTML. Human-readable but not as tidy as GitHub's plain markdown. ADR-002 documents the decision.
- **Write path** (`create_issue` / `update_issue` / `delete_or_close`) on `ConfluenceReadOnlyBackend` returns `NotSupported`. v0.4.
- **`PageBackend` trait** (ADR-002 Option-B) is deferred. v0.3 flattens Confluence's page hierarchy into `parent_id` frontmatter metadata; there is no `cd` into subpages. v0.4 will add a sibling trait if user feedback warrants.
- **Phase 12 subprocess/JSON-RPC connector ABI** — the scalable-without-forking successor to today's "fork + add dispatch" model. Documented in `docs/connectors/guide.md` and `ROADMAP.md §Phase 12`. Not started.
- **Labels, attachments, comments** on Confluence pages — not exposed by the adapter. v0.4+.
- **Swarm harness against Confluence** (`--mode confluence-direct`) — Phase 11 stretch goal; deferred because rate limits make a 50-client 30s run expensive.
- **FUSE write path through `IssueBackend::update_issue`** — still routes through the sim-specific REST shape in `crates/reposix-fuse/src/fetch.rs`. Same v0.3-era deferral noted in v0.2.0-alpha notes; no new work tonight.
- **`git-remote-reposix` rewire through `IssueBackend`** — still hardcodes the simulator. Mechanical but not done.

## Stats

| Metric | v0.2.0-alpha | v0.3.0 |
|---|---|---|
| Workspace tests | 168 | **191** (+23 from `reposix-confluence` unit + contract tests) |
| Commits since prior tag | — | 24 atomic commits across 11-A..11-F |
| `cargo clippy --all-targets -- -D warnings` | clean | clean |
| `cargo fmt --all --check` | clean | clean |
| `mkdocs build --strict` | green | green |
| `scripts/demos/smoke.sh` | 4/4 | **4/4** |
| Backends | `sim`, `github` | `sim`, `github`, **`confluence`** |

## Open problems for the next agent

> **All OP-1 through OP-11 items are CLOSED as of v0.7.0.** The next mission is Phase 27+ (JIRA Cloud adapter, `BackendConnector` rename, `Issue.extensions` field). See `.planning/ROADMAP.md` for the current plan.

| OP | Description | Status |
|----|-------------|--------|
| OP-1 | labels + spaces directory views | **CLOSED** — Phase 19 (v0.6) |
| OP-2 | tree-recursive + mount-root `_INDEX.md` | **CLOSED** — Phase 18 (v0.6) |
| OP-3 | `reposix refresh` + git-diff cache | **CLOSED** — Phase 20 (v0.6) |
| OP-4 | prebuilt release binaries | **CLOSED** — Phase 13 (v0.4) |
| OP-5 | `social/` → `docs/social/` | **CLOSED** — Phase 13 (v0.4) |
| OP-6 | full-repo code sweep findings | **CLOSED** — Phases 13–14 |
| OP-7 | hardening bundle (contention, truncation, chaos) | **CLOSED** — Phase 21 (v0.7) |
| OP-8 | honest-tokenizer benchmarks | **CLOSED** — Phase 22 (v0.7) |
| OP-9 | Confluence beyond pages (comments/attachments/whiteboards) | **CLOSED** — Phases 23–24 (v0.7) |
| OP-10 | eject 3rd-party adapter crates | **DEFERRED** — user-gated; not started |
| OP-11 | docs reorg (`InitialReport.md` → `docs/research/`) | **CLOSED** — Phase 25 (v0.7) |

---

## Handoff

If there's a next overnight agent: your starting point is `.planning/STATE.md` (current cursor), then `.planning/ROADMAP.md` for Phase 27+ scope (JIRA Cloud adapter, `BackendConnector` rename, `Issue.extensions` field). The `[v0.7.0]` block in `CHANGELOG.md` has the full release notes. Historical phase artifacts are under `.planning/phases/`.

---

## OP-6 Backlog from full-repo sweep

OP-6 sweep items HIGH-1 through HIGH-5 and MEDIUM-6 through MEDIUM-17 resolved in sessions 3–4 (see session-4 drive-by block below). Remaining LOW items (obsolete scripts, marketing doc staleness) are low-priority; no blocking work remains.

## OP-7 (hardening bundle) — CLOSED Phase 21 (v0.7)
Contention/truncation/chaos hardening shipped. See `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-SUMMARY.md`.

## OP-8 (honest-tokenizer benchmarks) — CLOSED Phase 22 (v0.7)
Real `count_tokens` API replaces `len/4` heuristic. 89.1% reduction confirmed. See `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md`.

## OP-9 (Confluence beyond pages) — CLOSED Phases 23–24 (v0.7)
- OP-9a (comments): Phase 23 — `pages/<id>.comments/` FUSE directory with per-comment `.md` files.
- OP-9b (whiteboards/attachments/folders): Phase 24 — `whiteboards/`, `.attachments/` per-page, folder-parented pages in `tree/`.

## OP-10 — Eject 3rd-party adapter crates — DEFERRED

User-gated hard stop — do not start without explicit check-in.

## OP-11 (docs reorg) — CLOSED Phase 25 (v0.7)
`InitialReport.md` and `AgenticEngineeringReference.md` moved to `docs/research/`. Root stubs deleted in Phase 26-01. See `.planning/phases/25-op-11-docs-reorg-initialreport-md-and-agenticengineeringrefe/25-SUMMARY.md`.

---

## Sign-off

— Claude Opus 4.6 1M context, 2026-04-13 / 2026-04-14 (overnight session 3).

---

## Session 4 augmentation — 2026-04-14 overnight (v0.4.0 ready-to-tag)

> Author: Claude Opus 4.6 1M context, overnight session 4. Built on top of session 3's v0.3.0 ship.

### tl;dr

Phase 13 shipped: **OP-1 from session 3's open-problems list — the "hero.png" folder-structure promise.** The FUSE mount now exposes Confluence pages as both (a) a flat `pages/<padded-id>.md` bucket for writes and git-tracking, and (b) a synthesized read-only `tree/` overlay of FUSE-emitted symlinks rendering Confluence's native parentId hierarchy at human-readable slug paths. Symlinks (not duplicate files) dissolve the concurrent-edit merge-conflict problem — the canonical file stays on a single stable path regardless of title/reparent churn.

**Workspace is 261/261 tests passing** (up from 193 at v0.3.0, +68), clippy clean, fmt clean, `scripts/demos/smoke.sh` 4/4, `scripts/demos/full.sh` end-to-end green, `mkdocs build --strict` green. **Live verify against `reuben-john.atlassian.net` space `REPOSIX` captured tonight** — `ls mount/` shows `pages/` + `tree/`, `readlink` returns correct `../../pages/00000131192.md`-style relative targets, `diff` between `cat tree/…/welcome.md` and `cat pages/00000131192.md` confirms kernel symlink resolution routes the read through FUSE correctly. Full transcript in `.planning/phases/13-.../13-SUMMARY.md §"Live-verify transcript"`.

Workspace `Cargo.toml` bumped to `0.4.0`. Code review PASSED (0 HIGH, 2 WARNING polish, 4 INFO — see `13-REVIEW.md`).

**The one thing left for you to do:** run `bash scripts/tag-v0.4.0.sh` to cut + push the v0.4.0 annotated tag. The autonomous session deliberately stopped short of pushing the tag (session 3 set the precedent; the plan's `autonomous: false` frontmatter on Wave E is a human-gate).

### What shipped in Phase 13 (one paragraph summary)

Nine atomic plans across five waves (A serial → B1/B2/B3 parallel → C serial → D1/D2/D3 parallel → E serial). Wave A added the foundational primitives to `reposix-core`: `Issue::parent_id: Option<IssueId>`, `BackendFeature::Hierarchy`, `IssueBackend::root_collection_name()` default, and the `path::{slugify_title, slug_or_fallback, dedupe_siblings}` pure-function helpers with 26 new unit tests covering T-13-01/02 adversarial slug inputs. Wave B1 extended `reposix-confluence` to deserialize `parentId`+`parentType`, filter to `parentType == "page"`, override `root_collection_name` to `"pages"`, and return `true` for `supports(Hierarchy)`; added 10 unit tests + a live-contract test against REPOSIX. Wave B2 added a new pure `reposix-fuse::tree` module — `TreeSnapshot` with cycle-safe DFS builder (T-13-03), sibling-slug dedupe (T-13-04), and depth-aware relative symlink target construction that provably cannot escape the mount (T-13-05) — 21 unit tests. Wave B3 filled the frontmatter-parent-id test gap (5 new tests). Wave C did the heavy integration: `ReposixFs` now dispatches `lookup`/`getattr`/`readdir`/`readlink` on disjoint inode ranges (ROOT / BUCKET_DIR / GITIGNORE / real issues / tree dirs / tree symlinks), synthesizes the mount-root `.gitignore` containing exactly `/tree/\n`, normalizes the real-file padding to 11 digits to match B2's symlink-target format, and landed `tests/nested_layout.rs` — 5 wiremock-Confluence + FUSE-mount integration tests proving the end-to-end flow (all passing under real `fusermount3 3.9.0`). Wave D1 swept 19 files for the BREAKING path change (`mount/<id>.md` → `mount/<bucket>/<id>.md` + 4-digit → 11-digit padding). Wave D2 wrote ADR-003 (supersedes ADR-002's layout decision), appended CHANGELOG `[v0.4.0]`, corrected CLAUDE.md fuser version (0.15 → 0.17), refreshed the operating-principle #1 wording (the v0.1 sim-only gate is no longer accurate at v0.4), added README "Folder structure" and prebuilt-binaries Quickstart sections (OP-12 fold-in from session 3's handoff). Wave D3 cloned `scripts/tag-v0.3.0.sh` → `scripts/tag-v0.4.0.sh` (6 safety guards intact) and wrote `scripts/demos/07-mount-real-confluence-tree.sh` demonstrating the hero `cd`-through-wiki flow. Wave E ran the full gauntlet + live verify. 32 atomic commits on `main` since baseline `d43f1d9`.

### Live proof captured tonight

```
$ reposix mount /tmp/reposix-v04-verify --backend confluence --project REPOSIX &
$ sleep 5
$ ls /tmp/reposix-v04-verify
pages  tree
$ cat /tmp/reposix-v04-verify/.gitignore
/tree/
$ ls /tmp/reposix-v04-verify/pages
00000065916.md  00000131192.md  00000360556.md  00000425985.md
$ ls /tmp/reposix-v04-verify/tree
reposix-demo-space-home
$ ls /tmp/reposix-v04-verify/tree/reposix-demo-space-home
_self.md  architecture-notes.md  demo-plan.md  welcome-to-reposix.md
$ readlink /tmp/reposix-v04-verify/tree/reposix-demo-space-home/welcome-to-reposix.md
../../pages/00000131192.md
$ diff <(cat .../tree/.../welcome-to-reposix.md) <(cat .../pages/00000131192.md) && echo OK
OK
$ fusermount3 -u /tmp/reposix-v04-verify    # clean
```

Full transcript is in `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md`.

### Drive-by fixes landed tonight (outside Phase 13 scope)

While Wave B agents were running in parallel (and, later, after Wave E closed the phase), the orchestrator shipped additional OP-6, OP-7, and review-polish items on disjoint files to make the morning review cleaner and v0.4.0 release-ready. All are independent `chore(drive-by)` / `docs(drive-by)` / `fix(13-review)` / `feat(hooks)` commits — not tagged as Phase 13 commits:

**OP-6 doc sweeps**
- **HIGH-1 (`8931a75`)** — `MORNING-BRIEF.md` and `PROJECT-STATUS.md` redirect pointers from the now-deleted `MORNING-BRIEF-v0.3.md` to `HANDOFF.md`.
- **HIGH-2 (`c5ea596`)** — `docs/security.md` rewritten for v0.4 truth: three real backends behind one allowlist; `tree/` overlay security properties; swarm harness shipped; `X-Reposix-Agent` spoofing framed honestly.
- **HIGH-3 (`de1735e`)** — `docs/demo.md` "No real backend. Simulator-only." callout replaced with an accurate v0.4 framing + pointer at the Tier-5 real-backend demos.
- **MEDIUM-6/7/8 (`45febb9`)** — `docs/development/roadmap.md` rewritten as v0.4+ (the old priority-1-through-17 list is replaced with a "what shipped" table and a pointer at HANDOFF for current open problems); `docs/index.md` homepage admonition now says "v0.4 — four autonomous overnight sessions"; `docs/why.md` line 127 no longer claims swarm harness is v0.2 work.
- **MEDIUM-10/11 (`c88e3f4`)** — `crates/reposix-sim/src/{routes/issues.rs, middleware/audit.rs}` stale `TODO(phase-3)` → `TODO(v0.4+)`; `routes/transitions.rs` "deferred to v0.2" → version-neutral.
- **MEDIUM-12/17 (`7792418`)** — `crates/reposix-confluence/src/lib.rs` write-path `Err` messages no longer say "in v0.3"; `docs/social/linkedin.md` stale v0.2 marketing line rewritten for v0.4.

**OP-7 hardening probes**
- **SSRF regression test (`ea5e548`)** — three new tests in `crates/reposix-confluence/tests/contract.rs` prove the adapter ignores attacker-controlled `_links.base`, `webui_link`, `_links.webui`, `_links.tinyui`, `_links.self`, and `_links.edit` fields. Uses wiremock `.expect(0)` + drop-panic enforcement plus inline `received_requests()` assertions for pinpoint regression diagnosis. Insurance against a future "follow the webui_link for screenshots" feature accidentally enabling SSRF.
- **Credential-hygiene pre-push hook (`f357c92` + `5361fd5`)** — `scripts/hooks/pre-push` rejects pushes containing literal `ATATT3…` (Atlassian API tokens), `Bearer ATATT3…` headers, `ghp_…` (classic GitHub PAT), or `github_pat_…` (fine-grained PAT) in outgoing ref ranges. Installed via `bash scripts/install-hooks.sh` (symlinks so hook updates land on next pull without re-install). Commit `5361fd5` fixed a self-match bug (the hook was flagging its own PATTERNS literal) and a `${}`-bad-substitution in the help text.
- **Pre-push hook unit test (`1df7bab` + `976540c`)** — `scripts/hooks/test-pre-push.sh` exercises 6 cases (clean pass, ATATT3 rejected, Bearer ATATT3 rejected, ghp_ rejected, github_pat_ rejected, self-scan exclusion honored). Run via `bash scripts/hooks/test-pre-push.sh` — 6/6 green. Cleanup restores the original branch (was leaving detached HEAD in the first iteration).

**Phase 13 code-review polish (bfdb846)**
Applied 5 of 6 items from `13-REVIEW.md`:
- `WR-01` — `tree/` readdir first-touch refresh (was returning empty listing silently).
- `WR-02` — `PATH_MAX` guard on symlink target (debug_assert + release warn).
- `IN-01` — cap attacker-controlled tracing strings in Confluence adapter (`parentId` / `parentType` truncated to 64 bytes).
- `IN-02` — `slugify_title` pre-caps the intermediate `to_lowercase()` allocation at ~240 chars so pathological 10MB titles don't balloon memory.
- `IN-03` — stable `mount_time` timestamp on symlink `FileAttr` (was drifting on every `getattr`, confusing rsync/make/backups).
- Skipped: `IN-04` — `TreeDir` `..` entry always uses `TREE_ROOT_INO`. Cosmetic only; kernel doesn't trust `..` inodes from FUSE. Adds a struct field for no runtime benefit.

**CHANGELOG extension (`f09aba5`)** — `[v0.4.0]` block extended with `### Hardening` and `### Security` subsections documenting the review polish, SSRF regression tests, pre-push hook, and security.md refresh. Also fixed a stale anchor (`security.md#whats-deferred-to-v02`) in `docs/reference/http-api.md` to point at the renamed heading.

**Gitignore housekeeping (`667cde0`)** — `.claude/` session-state dir now gitignored so the tag-v0.4.0.sh clean-tree guard doesn't trip on it in the morning.

**Script-dir reorg (`3b22365`)** — OP-6 LOW-18/19/20 done: `scripts/phase2_goal_backward.sh`, `scripts/phase2_smoke.sh`, `scripts/phase3_exit_check.sh` → `.planning/archive/scripts/`; `scripts/probe-confluence.sh` → `scripts/dev/`; `scripts/mermaid_divs_to_fences.py`, `scripts/fix_demos_index_links.py` → `scripts/migrations/`. No external references broken.

**reposix-github wiremock test coverage (`1769062`)** — OP-6 MEDIUM-13 done: 7 new always-run wiremock tests at `crates/reposix-github/tests/contract.rs` (+438 lines). Coverage: `contract_github_wiremock` (full `assert_contract` sequence), `pagination_follows_link_header`, `rate_limit_429_surfaces_clean_error` (regression guard — documents current NO-retry behavior), `state_reason_maps_to_status` (6-row matrix pinning ADR-001), `adversarial_html_url_does_not_trigger_outbound_call` (SSRF tripwire), `malformed_assignee_object_degrades_to_none`, `user_agent_header_is_set`. Mirrors the Confluence contract pattern.

**reposix-swarm mini E2E (`8b6f889`)** — OP-6 MEDIUM-14 done: `crates/reposix-swarm/tests/mini_e2e.rs::swarm_mini_e2e_sim_5_clients_1_5s` — spins `reposix-sim` on an ephemeral port via `run_with_listener`, runs `SimDirectWorkload` × 5 clients for 1.5s, asserts summary metrics + audit-log ≥5 rows + clean exit. 1.52s run-time. Closes the "zero integration tests" gap in reposix-swarm.

**Test count delta** — 261 → **272** (+11 since Wave E — 3 SSRF, 7 github wiremock, 1 swarm E2E).
**Commit count** — 50 total on `main` since v0.3.0 baseline `d43f1d9` (32 Phase-13 + 18 post-Phase-13 bonus).

### What I deliberately did NOT do (explicit non-scope)

Per the user's pre-sleep instructions, I did NOT:
- Start OP-10 (eject 3rd-party adapter crates) — user said "not tonight."
- Start OP-11 (repo-root reorg — `InitialReport.md` / `AgenticEngineeringReference.md` → `docs/research/`) — user said "not tonight."
- Push the v0.4.0 tag — autonomous session deliberately stops at the tag-script gate.

### Open problems rollup for next session (still outstanding)

Unchanged from session 3's handoff unless noted:

- **OP-1 (this session — SHIPPED).** Confluence parentId tree is live. The remaining pieces of OP-1's original scope (labels/, recent/, spaces/, multi-space mount, symlink into `labels/bug/0001.md` from `issues/0001.md`) are explicitly out of scope for v0.4 and deferred to v0.5+.
- **OP-2 — dynamically generated `INDEX.md` per directory.** Not started. Highest ROI next-session item IMO — tree/ now exists, so `tree/INDEX.md` producing a one-shot sitemap would be a killer agent-UX multiplier.
- **OP-3 — cache refresh via `git pull` semantics.** Not started. Tree/ is still live-on-every-mount; no snapshotting to git yet.
- **OP-4 — prebuilt binaries.** DONE (session 3; OP-12 install-docs fold-in landed in D2 tonight).
- **OP-5 — `social/` → `docs/social/`.** DONE (session 3).
- **OP-6 — sweep findings.** HIGH-1 + MEDIUM-10/11 done this session. Remaining: HIGH-2 (`docs/security.md` v0.3 rewrite — D2 noted this as scope-boundary deferred; ready for a `/gsd-quick`), HIGH-3 (`docs/demo.md` "Simulator-only" callout — same), HIGH-4 (CLAUDE.md OP-1 wording — DONE tonight in D2), plus all MEDIUM and LOW items.
- **OP-7 — hardening probes.** Not started. Specifically: concurrent-write contention swarm, FUSE under real-backend load, 500-page limit probe, credential-hygiene grep, SSRF on `webui_link`/`_links.base`, chaos audit-log restart test.
- **OP-8 — honest-tokenizer benchmarks.** Not started.
- **OP-9 — Confluence beyond pages (whiteboards, comments, attachments, live docs, folders, multi-space).** Not started. Comments (`pages/<id>.comments/<cid>.md`) is the next-most-compelling use case after the tree/ overlay that just shipped — they compose naturally now that folder structure exists in the mount.
- **OP-10 — eject 3rd-party adapter crates.** Not started (user-gated).
- **OP-11 — repo-root reorg.** Not started (user-gated).
- **OP-12 — install docs for prebuilt binaries.** DONE (folded into Phase 13-D2 tonight).

### Post-review cleanup candidates (from `13-REVIEW.md`, NONE blocking v0.4.0)

- **WARN-01:** `tree/` readdir doesn't refresh on cold first-touch (small UX paper cut — user has to re-mount to see new Confluence pages). Queue for OP-3 cache-refresh phase.
- **WARN-02:** Symlink target byte-length uncapped vs `PATH_MAX` (4096 on Linux). With 60-byte slug cap and max realistic depth ~6, the worst-case target is ~400 bytes — theoretically safe but no explicit assertion. Queue for OP-7 hardening.
- **INFO-01:** Unbounded `tracing::warn!` of attacker-controlled `parentId` / `parentType` in the Confluence adapter. Byte caps would be trivial to add. Queue for OP-7.
- **INFO-02:** `slugify_title` allocates the full title String before capping at 60 bytes. Pathological 10MB titles already proven not to panic but briefly balloon memory. Easy fix: pre-cap at input byte-slice.
- **INFO-03:** Symlink `mtime` drifts on each getattr (currently returns `SystemTime::now()` for synthesized nodes).
- **INFO-04:** `TreeDir` `..` entry always uses `TREE_ROOT_INO` regardless of depth — benign (kernel doesn't trust `..` inodes from FUSE anyway) but technically inaccurate.

— Claude Opus 4.6 1M context, 2026-04-14 (overnight session 4).

---

## Session 5 augmentation — 2026-04-14 daytime (v0.4.1 + v0.5.0 ready-to-ship)

> Author: Claude Opus 4.6 1M context, session 5 (wall-clock ~08:00-11:30 PDT).
> Built on top of session 4's v0.4.0 ship. Two tags pushed this session:
> **v0.4.1** (Phase 14 — Cluster B refactor) and **v0.5.0** (Phase 15 — OP-2
> partial bucket-level `_INDEX.md`). Both had CI release workflows fire on
> tag push; prebuilt Linux binaries (x86_64 + aarch64) attached to both
> GitHub releases.

### tl;dr

Two phases shipped back-to-back.

**Phase 14 (v0.4.1, bugfix/refactor scope).** Closes v0.3-era HANDOFF "Known
open gaps" items 7 and 8 — FUSE write path and `git-remote-reposix` now both
route through `IssueBackend::{create_issue, update_issue, delete_or_close}`
instead of hardcoded sim-REST `fetch.rs` / `client.rs`. Net deletion:
`crates/reposix-fuse/src/fetch.rs` (596 LoC), `crates/reposix-fuse/tests/write.rs`
(236 LoC), `crates/reposix-remote/src/client.rs` (236 LoC) — ~1,068 LoC
removed, ~400 LoC net added between `fs.rs` rewire + sim.rs re-homed tests.
Also swept v0.3-era deferral prose out of `docs/security.md`,
`docs/reference/crates.md`, `README.md`, and `docs/architecture.md`. Wave
structure A(serial) → B1‖B2(parallel) → C(serial, verify) → D(serial, docs).

**Phase 15 (v0.5.0, feature scope).** OP-2 partial — `mount/<bucket>/_INDEX.md`
now renders a YAML-frontmatter + markdown-table sitemap on read. Visible in
`ls <bucket>/`, hidden from `*.md` globs (leading underscore), read-only
(EROFS/EACCES on write/create/unlink), rendered from the same issue-list
cache that backs readdir, sorted ascending by id. Not in `tree/`, not at
mount root — those are deferred to follow-up phases. Ships with
`scripts/dev/test-bucket-index.sh` as the live proof.

**Workspace is 277/277 tests passing** (up from 272 at v0.4.0, +5 from Phase
14's re-homing + Phase 15's render tests; no net regression from Phase 14
deletions). Clippy clean, fmt clean, `bash scripts/green-gauntlet.sh --full`
6/6 gates green (fmt/clippy/test/smoke/mkdocs-strict/fuse-ignored). Live
demos `01-edit-and-push.sh` + `06-mount-real-confluence.sh` + the new
`scripts/dev/test-bucket-index.sh` all exit 0 on the dev host.

19 atomic commits since `3b3f867` (session-4 CI fix, session-5 baseline).
Tag `v0.4.1` → commit `9ee8a1a`. Tag `v0.5.0` → commit `82f73d1`. Both
pushed; both release workflows ran green; prebuilt binaries on GitHub.

### Live proof captured tonight

**Phase 14 — audit-attribution spot-check (proof of R2 behavior-change):**

```
$ sqlite3 /tmp/reposix-demo-01-sim.db \
    "SELECT agent_id, COUNT(*) FROM audit_events GROUP BY agent_id ORDER BY agent_id;"
reposix-core-simbackend-<pid>-fuse|12
reposix-core-simbackend-<pid>-remote|8
```

Zero rows tagged `reposix-fuse-<pid>` or `git-remote-reposix-<pid>` in a
fresh DB. The refactor's new suffix-based attribution is live-confirmed.

**Phase 15 — bucket index live proof (`scripts/dev/test-bucket-index.sh`):**

```
$ ls /tmp/reposix-15-bucket-index-mnt/issues/
00000000001.md  00000000002.md  00000000003.md  00000000004.md
00000000005.md  00000000006.md  _INDEX.md

$ cat /tmp/reposix-15-bucket-index-mnt/issues/_INDEX.md
---
backend: simulator
project: demo
issue_count: 6
generated_at: 2026-04-14T18:21:44Z
---

# Index of issues/ — demo (6 issues)

| id | status | title | updated |
| --- | --- | --- | --- |
| 1 | open | database connection drops under load | 2026-04-13 |
| 2 | in_progress | add `--no-color` flag to CLI | 2026-04-13 |
...

$ touch /tmp/reposix-15-bucket-index-mnt/issues/_INDEX.md
touch: cannot touch '...': Permission denied

== BUCKET INDEX PROOF OK ==
```

### What shipped (Phase 14 — v0.4.1)

| Commit | Role |
|---|---|
| `7510ed1` | test(14-A): pin sim 409 body shape (R13 mitigation) |
| `cd50ec5` | test(14-B1): re-home SG-03 egress-sanitize proof onto `SimBackend` |
| `bdad951` | refactor(14-B1): `fs.rs` write path through `IssueBackend` |
| `938b8de` | refactor(14-B2): `reposix-remote` through `IssueBackend` |
| `4301d0d` | docs(14-C): verification doc — all SCs PASS |
| `547d9e0` | docs(14-D): CHANGELOG `[Unreleased]` + v0.3-era deferral-prose sweep |
| `142f761` | docs(14-D): 14-SUMMARY.md + STATE.md cursor |
| `2393d85` | docs(14-review): code review of Phase 14 commits |
| `1ffe47b` | docs(14-review-fix): LOW-01/02 doc-comment refresh |
| `9ee8a1a` | chore(release): version bump to 0.4.1 + CHANGELOG promotion + `tag-v0.4.1.sh` |

### What shipped (Phase 15 — v0.5.0)

| Commit | Role |
|---|---|
| `7eec57d` | chore(15-planning): CONTEXT + Wave A plan |
| `6a2e256` | feat(15-A): reserve `BUCKET_INDEX_INO = 5` inode |
| `a94e970` | feat(15-A): synthesize `_INDEX.md` in FUSE bucket dir |
| `3309d4c` | chore(15-A): `scripts/dev/test-bucket-index.sh` live proof |
| `c3d2901` | docs(15-B): CHANGELOG `[v0.5.0]` + version bump to 0.5.0 |
| `f43f0e5` | docs(15-B): 15-SUMMARY.md + STATE.md cursor |
| `ceec233` | chore(15-B): `scripts/tag-v0.5.0.sh` |
| `82f73d1` | docs(15-B): backfill Wave B commit hashes in STATE.md |

### Accepted behavior changes (documented in CHANGELOG)

- **R1 — Assignee-clear on untouched PATCH.** The old `fetch::patch_issue`
  skipped the `assignee` field when `None`; the new
  `SimBackend::update_issue` path emits `"assignee": null`, which the sim
  treats as *clear*. FUSE mount semantics: the file is the source of truth
  — if the user removes `assignee:` from the frontmatter, the assignee is
  cleared on next release. Consistent with how every other field behaves.
- **R2 — Audit attribution suffix-normalized.**
  - FUSE writes: `reposix-fuse-<pid>` → `reposix-core-simbackend-<pid>-fuse`.
  - `git-remote-reposix`: `git-remote-reposix-<pid>` → `reposix-core-simbackend-<pid>-remote`.
  - Downstream log/audit-query tooling grouping on the old prefixes needs
    to widen the match to `reposix-core-simbackend-%-{fuse,remote}` or
    query the new full-string forms.

### Non-behavioral sweeps (in the same session, independent commits)

None. The session was narrowly scoped to Phase 14 + Phase 15.

### Stats

| Metric | v0.4.0 | v0.5.0 |
|---|---|---|
| Workspace tests | 272 | **277** (+5; +2 Phase-14 `sim.rs` re-home + 3 Phase-15 render-function unit tests; -4 redundant tests from the write.rs re-home) |
| Commits since prior tag (`v0.4.0`) | — | 19 atomic commits |
| LoC deleted (Phase 14) | — | ~1,068 (`fetch.rs` + `write.rs` + `client.rs`) |
| LoC added | — | ~400 net (fs.rs rewire + sim.rs tests + `_INDEX.md` renderer) |
| `cargo clippy --all-targets -- -D warnings` | clean | clean |
| `cargo fmt --all --check` | clean | clean |
| `mkdocs build --strict` | green | green |
| `scripts/demos/smoke.sh` | 4/4 | 4/4 |
| `green-gauntlet.sh --full` | (not yet shipped in session 4) | **6/6** |
| Backends | sim, github, confluence | sim, github, confluence (unchanged) |

### What I deliberately did NOT do (explicit non-scope)

Per the session-5 brief:
- Did NOT start OP-10 (eject 3rd-party adapter crates) — user-gated.
- Did NOT start OP-11 (repo-root reorg — `InitialReport.md` / `AgenticEngineeringReference.md` → `docs/research/`) — user-gated.
- Did NOT start Phase 12 (subprocess/JSON-RPC connector ABI) — user-gated, design question open.
- Did NOT start Cluster A (Confluence writes) — deliberate punt; Phase 14 unblocks it, but the atlas_doc_format round-trip is multi-session scope.

### Session-5 open problems rollup (what's still outstanding)

Open problems queued as Phases 16–25 (milestones v0.6.0 and v0.7.0). See ROADMAP.md.

### New discoveries / known infra gaps (from this session)

- **C-1 — `scripts/green-gauntlet.sh` does not rebuild release binaries.**
  Phase 14 Wave C caught this during audit-attribution spot-check: smoke
  demos will silently run against stale `target/release/*` binaries if
  they exist, masking whatever's in the current working tree. The gauntlet
  passes visually but isn't actually testing the latest code. Fix: either
  build-first (add a `cargo build --release --workspace --bins --locked`
  gate before smoke) or assert binary mtime is post-HEAD. Queued as a
  `/gsd-quick` candidate.
- **C-2 — `audit_events` schema.** The column is `agent_id` (not `agent`).
  Verification-doc snippets floating around mention `SELECT agent FROM
  audit`; those are wrong. Correct column name captured in this session's
  14-VERIFICATION.md and CHANGELOG.
- **`LICENSE-APACHE` exists** per session-4's OP-6-HIGH-5 fix; no change
  this session.

### Post-review cleanup candidates (all LOW, none blocking)

From `14-REVIEW.md`:
- **INFO-01..04** — near-duplicate R13 pin tests (defensible), `_reason`
  param discarded in `delete_or_close` (pre-existing, correct for sim),
  version-mismatch prefix-match string contract codified in backend.rs
  doc (note for future typed-variant refactor), `FetchError` visibility
  tightened from pub to private (positive side-effect).

From `15-SUMMARY.md` follow-ups:
- Tree-level recursive `_INDEX.md` (biggest remaining OP-2 piece).
- Mount-root `_INDEX.md` (smallest remaining OP-2 piece).
- User-configurable column set in the index.
- `_INDEX.md`-in-`git diff` round-trip semantics (ties into OP-3).

### Mission recommendation for session 6

**Pick one of (ordered by ROI):**

1. **Cluster A (Confluence writes)** — Phase 14 unblocked this. `create_issue`,
   `update_issue`, `delete_or_close` on `ConfluenceBackend` + `atlas_doc_format`
   ↔ Markdown converter. Highest user-visible ROI left. Realistically multi-session;
   session-6 could scope-tight to just `update_issue` (the most common op) + a
   minimal storage-format↔markdown renderer. Ships v0.6.0.

2. **Cluster C (swarm `--mode confluence-direct`)** — Small warm-up (~300 LoC).
   Exercises Phase 14's refactor against Confluence + proves the trait truly
   generalizes. Even cheaper now that rate-limiting is well-understood from
   Phase 9. Ships v0.5.1 (bugfix-size but feature-flavored) or folds into a
   bigger release.

3. **OP-2 tree-recursive `_INDEX.md`** — Phase 15's follow-up. Pattern proven
   this session. Cycle-safe recursive synthesis is ~200 LoC extension of
   `TreeSnapshot::dfs`. Ships v0.5.1 or v0.6.0.

4. **OP-7 hardening bundle** — Concurrent-write contention swarm, 500-page
   truncation probe, chaos audit-log restart, macFUSE parity CI matrix. All
   are additive tests + small flag additions; low blast-radius. Ships v0.5.1.

5. **OP-3 `reposix refresh` + git-diff cache** — Mount-as-time-machine.
   Biggest conceptual win but biggest scope. Needs a new `reposix-cache`
   crate with sqlite WAL. Multi-session. Ships v0.6.0+.

**Do NOT** start OP-10 / OP-11 / Phase 12 without explicit user check-in.

### The norms still apply

Simulator before real backend · tainted by default · audit log non-optional ·
no hidden state · mount = git repo · `REPOSIX_ALLOWED_ORIGINS` guards every
egress · Untainted<Issue> discipline holds through the trait boundary.

This session touched none of these rails. Phase 14's refactor concentrated
the sim-REST shape into exactly one crate (`reposix-core::backend::sim`);
Phase 15 added one synthesized file behind a new reserved inode slot with
EROFS-by-default on writes. The trust model is narrower after this session,
not wider.

### Install the pre-push hook first thing next session

Per session-4's instruction pattern:

```bash
bash scripts/install-hooks.sh
```

This session ran it at start and caught zero violations on both tag pushes.

### Cutting future tags

`scripts/tag-v0.5.0.sh` is now the template. Clone and version-bump for the
next release. The seven safety guards (branch/clean/no-local-tag/no-remote-
tag/CHANGELOG/Cargo.toml/tests+smoke) are battle-tested.

**Pre-tag PATH setup:** the tag script's guard 7 runs `smoke.sh`, which
requires `reposix-sim`/`reposix-fuse`/`git-remote-reposix` on PATH. Add
`export PATH="$PWD/target/debug:$PATH"` before running the tag script, OR
the smoke check will fail with "required command not found." (Green-gauntlet
handles this automatically; the tag script does not.) See C-1 above.

### Sign-off

— Claude Opus 4.6 1M context, 2026-04-14 (session 5 daytime).
