# AUTONOMOUS MODE — READ THIS FIRST

You are taking over an autonomous engineering session for the reposix project (`https://github.com/reubenjohn/reposix`). The owner (`reubenjohn` / `reubenvjohn@gmail.com`) is offline. **Do not block on user input. Decide, ship, push.**

## Start here — gather actual state before believing anything in this file

This file contains a snapshot taken at write time. Re-derive every state fact with these commands and trust the output, not the prose:

```bash
cd /home/reuben/workspace/reposix
git log --oneline -5 origin/main
gh run list --branch main --limit 5 --json status,conclusion,workflowName,headSha
gh pr list -R reubenjohn/reposix --state open --json number,title,headRefName
for c in reposix-core reposix-cache reposix-sim reposix-github reposix-confluence reposix-jira reposix-remote reposix-cli; do
  v=$(curl -s "https://crates.io/api/v1/crates/$c" | grep -o '"max_version":"[^"]*"' | head -1 | sed 's/.*://;s/"//g')
  printf "  %-25s %s\n" "$c" "${v:-MISSING}"
done
python3 scripts/catalog.py render
```

Then read CLAUDE.md (the **"Build memory budget"** section is load-bearing — the VM has crashed twice from parallel cargo). Then execute §7 of THIS file top-to-bottom.

**Staleness anchor.** If `git log -1 --format=%H` does NOT equal `d10c1d8fefc91ac330869a38c29f463504e98aea`, this file is at least one commit stale — re-run the state-gather block above and TREAT §10 ASSERTIONS AS HISTORICAL ONLY. Nothing in this file overrides the live commands.

## Deadline — 2026-04-28 01:00 local (≈4-5 hours from handoff)

Pace against §7. When queue is empty, expand into §6 backlog or pause cleanly.

## Non-negotiable rules

- **Owner is offline. Do not ask questions. Decide and proceed.**
- **You are the coordinator. You do not type code.** Coordinator's job: decide, route, verify. Subagents do the file reads, the edits, the cargo runs, the playwright walks. **If you find yourself about to type a >20-line edit, STOP and dispatch a subagent.** Aggressive delegation is owner's global OP #2.
- **CATALOG-v3 is your bookkeeping.** After every §7 step: `python3 scripts/catalog.py set <path> DONE --note "ref §7-X"`. Run `coverage` before §7 close.
- **No skill changes.** `.claude/skills/` is owner-approval-gated.
- **One cargo invocation at a time, ever.** `cargo check -p <crate>`, never `--workspace`. CLAUDE.md "Build memory budget" explains why. If a subagent does cargo work, it has the lock.
- **Push frequently** (every commit). Pre-push hook runs fmt + clippy + `scripts/check-docs-site.sh` + the banned-words linter — let it gate. Don't `--no-verify`.
- **Never delegate `gh pr checkout` to a bash subagent without isolating the working tree.** It switches the coordinator's branch behind your back. Use a worktree (e.g. `git worktree add /tmp/pr-N` then have the subagent cd there) or have the subagent operate in `/tmp/<branch>-checkout`. Lesson learned at commit `5a91ae2` (cherry-pick mess).
- **No retrospective / walkthrough / morning-brief docs.** This file is operational; delete it once §7 is done. Owner explicitly does not want session-recap files.
- **Banned word: `replace`.** Use `migrate to` / `for the 80%` / `rewrite as` / `complement` / `alongside`. The pre-push hook enforces it.

## Subagent dispatch cookbook

Brief in ≤200 words → dispatch via `Agent` (subagent_type: `general-purpose`) → read the report not the transcript → update §7 status → next.

**Parallelizable**: doc edits in disjoint subdirs, audits, playwright walks.
**Serialize**: anything that compiles (cargo / Cargo.toml / Cargo.lock).

**Worked example — playwright walk of how-it-works pages (for §0.1 / §7-row3):**

```
Agent prompt (≤200 words):
  Goal: walk every page under /how-it-works/ on the live mkdocs site and verify
  every <pre.mermaid> block has at least one <svg> child rendered.
  Steps:
    1. cd /home/reuben/workspace/reposix
    2. Start `mkdocs serve -a 127.0.0.1:8765` in background; wait until it serves /.
    3. For each file under docs/how-it-works/*.md, derive the URL slug,
       call mcp__playwright__browser_navigate, then mcp__playwright__browser_run_code
       with: `document.querySelectorAll('pre.mermaid').length` and
       `Array.from(document.querySelectorAll('pre.mermaid')).map(p => p.querySelectorAll('svg').length)`.
    4. For pages with mermaid_count > 0 and any svg_count == 0, capture
       browser_console_messages and a screenshot.
    5. Write per-page JSON to .planning/verifications/playwright/how-it-works/<slug>.json
       with fields: {nav_path, mermaid_count, svg_counts, console_errors, snapshot_path, captured_at_utc}.
    6. Stop mkdocs serve.
  Report: list of FAIL pages with one-line diagnosis each.
```

## When the queue is empty

- Update or delete this file. Don't append "I did X" entries.
- Update `.planning/STATE.md` with the new cursor.
- Commit + push.
- If CI is green, all 8 crates are at the latest workspace version on crates.io, dependabot inbox empty: stop.

**Go.**

---

# §0. CRITICAL OWNER FEEDBACK — added 2026-04-27 PM (read before §1)

## EXECUTION ORDER (non-negotiable)

1. **§0.7** — patch CLAUDE.md (the instructions layer).
2. **§0.8** — build the SESSION-END-STATE framework (the verifier layer).
3. **§0.6** — superseded; the §0.8 verifier subsumes the six tools. SKIP unless §0.8 is descoped.
4. **§0.1–§0.5** — let the §0.8 framework's verdict drive these as FAIL rows; do NOT hand-fix them ahead of the framework.
5. **§3a → §3b → §7-A..G** — the original v0.11.1/v0.11.2 finish.

If you run out of time mid-queue, the FLOOR is: keep CI green, keep the latest workspace-version crates published, keep `.planning/STATE.md` and HANDOVER.md current. Everything else is gravy. The full decision tree lives at the top of §7.

---

## 0.1 Mermaid renders are broken on some `docs/how-it-works/` pages

**Owner verbatim**: *"the mkdocs site is not rendering mermaid on a few how it works pages. Did you use playwright or set up UI tests? I've caught this issue multiple times."*

**Friction**: Previous session shipped POLISH2-22 mermaid edits with playwright validation scoped to a single hero page; `mkdocs build --strict` does not fail on broken SVGs.

**Fix path** (executed THROUGH §0.8 framework — write the FAIL rows first, then close):
1. Walk every `docs/how-it-works/*.md` page with playwright (`mcp__playwright__browser_navigate` + `browser_snapshot` + `browser_console_messages`). Identify pages whose `<pre.mermaid>` blocks have zero `<svg>` children.
2. Diagnose. Likely candidates: a `<br/>` literal that escaped, a `{id}` template var that became HTML, a `pymdownx.emoji` interaction, a `pymdownx.superfences` config drift after the rusqlite bump churned Cargo.lock.
3. Add `scripts/check-mermaid-renders.sh` that loops over EVERY mkdocs page (not just how-it-works) and asserts each `<pre.mermaid>` element has at least one `<svg>` child. Wire into pre-push.

**Verify**: `bash scripts/check-mermaid-renders.sh` exits 0 AND `.planning/verifications/playwright/how-it-works/*.json` has one file per `docs/how-it-works/*.md` with `svg_counts` showing no zeros.

## 0.2 Install instructions on home page lead with source-compile, not package manager

**Owner verbatim**: *"now that we have crates, we should continue to have Six-line quickstart instructions to compile from source but shouldn't the more prominent instructions like the ones on home page be to install using a popular package manager with links for other ways of installation?"*

**Friction**: Hero on `docs/index.md` keeps the source-compile six-line prominent even though all 8 crates are now on crates.io.

**Fix path**:
1. After all 8 crates at the latest workspace version are confirmed live (re-run the state-gather block at the top), reorder `docs/index.md` hero: package-manager install band PROMINENT (3 cards: brew / binstall / curl|sh), source-compile six-line as a `<details>` collapsible "Build from source".
2. Same edit on `README.md` if it's similarly source-first.
3. Validate via playwright per §0.1 mitigation.

**Verify**: `grep -A 30 '^# ' docs/index.md | head -40` — first install snippet uses `brew install` or `cargo binstall`, NOT `git clone`.

## 0.3 Version-pinned filename: `docs/benchmarks/v0.9.0-latency.md`

**Owner verbatim**: *"is it intentional?: The latency page shows under v0.9.0. Why name the page 0.9.0?"*

**Friction**: `repo-org-gaps.md` rec #8 flagged this; it was deferred. The page is about CURRENT latency, not v0.9.0-historical.

**Fix path**:
1. `git mv docs/benchmarks/v0.9.0-latency.md docs/benchmarks/latency.md`.
2. `git mv scripts/v0.9.0-latency.sh scripts/latency-bench.sh` (update internal `SCRIPT_NAME` refs).
3. Add `last_measured_at: <RFC3339>` frontmatter field; bench-cron writes it.
4. Update every cross-ref (`README.md`, `docs/index.md`, `mkdocs.yml` nav, `bench-latency-cron.yml`, `ci.yml`).
5. mkdocs `redirect` plugin entry from old path → new path so external links don't 404.
6. Validate playwright.

**Verify**: `find docs scripts -type f | grep -E 'v[0-9]+\.[0-9]+\.[0-9]+' | grep -v CHANGELOG` returns nothing AND `bash scripts/check-docs-site.sh` is green.

## 0.4 Token economy benchmark `benchmarks/RESULTS.md` not in mkdocs nav

**Owner verbatim**: *"shouldn't the token economy benchmark also be part of the mkdocs site?"*

**Friction**: Hero cites `https://github.com/reubenjohn/reposix/blob/main/benchmarks/RESULTS.md` as an absolute github URL, bypassing mkdocs entirely; the 89.1% headline number lives outside the site.

**Fix path**:
1. Move or symlink `benchmarks/RESULTS.md` → `docs/benchmarks/token-economy.md`.
2. Add to `mkdocs.yml` nav under the existing `Benchmarks` section, next to `latency.md` (post-§0.3 rename).
3. Update the hero (`docs/index.md`) cite from absolute github URL to relative `benchmarks/token-economy.md`.
4. Same for any other inbound links (`git grep -l 'benchmarks/RESULTS'`).
5. Validate playwright.

**Verify**: `git grep -l 'benchmarks/RESULTS' -- ':!HANDOVER.md'` returns empty AND `grep -c 'token-economy' mkdocs.yml` is ≥ 1.

## 0.5 GSD planning org: `.planning/milestones/v0.X.0-ROADMAP.md` lives at wrong level

**Owner verbatim**: *"why is .planning/milestones/v0.10.0-ROADMAP.md not in .planning/milestones/v0.10.0-phases? What is the proper gsd organization?"*

**Friction**: `repo-org-gaps.md` flagged this; loose `v0.X.0-{ROADMAP,REQUIREMENTS}.md` sit alongside the per-milestone `*-phases/` dirs instead of inside or in `.planning/archive/`.

**Fix path**: pick one option, apply uniformly to v0.8 / v0.9 / v0.10, then document the chosen GSD convention in `CLAUDE.md` § "Workspace layout".
  - **Option A** (preserve per-milestone history): each `.planning/milestones/v0.X.0-phases/ARCHIVE.md` (created in POLISH2-21) ABSORBS the corresponding `v0.X.0-ROADMAP.md` content.
  - **Option B** (clean separation): `git mv` each loose `v0.X.0-{ROADMAP,REQUIREMENTS}.md` into the matching `v0.X.0-phases/` dir, OR into `.planning/archive/milestones/`.

**Verify**: `find .planning/milestones -maxdepth 2 -name '*ROADMAP*' -o -name '*REQUIREMENTS*' | grep -v phases | grep -v archive` returns empty.

## 0.6 The meta-question: missing quality tools — SUPERSEDED by §0.8

**Owner verbatim**: *"Are you missing tools to help catch these quality issues?"*

The original answer ranked six bespoke checkers (mermaid render, install currency, version-pinned filename, GSD org, doc-clarity, playwright cron). **§0.8's verifier subagent generalises all six**: each becomes a single row in the end-state contract whose `verifier` field points at a one-line shell command or a playwright artifact path. **SKIP §0.6 implementation** unless the next agent decides to descope §0.8 — in which case ship rows 1, 4, 6 of the original table (mermaid render check, GSD org check, playwright site walk) because those are the ones automated hooks cannot otherwise catch.

Original six-tool table preserved for reference:

| Gap | Proposed tool | Where it lives |
|---|---|---|
| Mermaid render correctness on EVERY page | `scripts/check-mermaid-renders.sh` (playwright) | pre-push + CI |
| Install-instruction freshness | `scripts/check-install-currency.py` | weekly cron |
| Version-pinned filename detection | `scripts/check-no-version-pinned-paths.sh` | pre-push |
| GSD planning org conformance | `scripts/check-gsd-org.sh` | pre-push |
| Cold-reader pass on doc set | `doc-clarity-review` skill | manual gate |
| Periodic playwright walk of live site | extend `scripts/check-docs-site.sh` | pre-push |

## 0.7 Update CLAUDE.md so the misses don't recur

The misses in §0.1-§0.5 happened because CLAUDE.md was AMBIGUOUS, not because the rules were absent. Concrete patches the next agent's FIRST commit must apply to `CLAUDE.md`:

- **"Docs-site validation" section**: change to "playwright walk EVERY page in the affected nav section, not just the file you changed; for `mkdocs.yml` or any `pymdownx.*` change, walk the entire site."
- **Add "Cold-reader pass" section**: "Before declaring any user-facing surface (hero, install instructions, headline numbers, benchmarks) shipped, dispatch the `doc-clarity-review` skill on the affected pages with isolated context. Mechanical hooks miss positioning misses (install-path freshness, version-pinned filenames, missing nav entries)."
- **Add "Freshness invariants" list**: no version-pinned filenames outside `CHANGELOG`; install path leads with package manager once crates.io publish is live; benchmarks belong in mkdocs nav; loose `*ROADMAP*.md` outside `*phases/` is structural drift.
- **Extend "Subagent delegation rules"**: "Never delegate `gh pr checkout` to a bash subagent without isolation (worktree or `/tmp/<branch>`). Coordinator's local checkout is shared state."
- **Add meta-rule**: "When an owner catches a quality issue the agent missed, the FIX is two-fold: (1) fix the issue, (2) update the instructions so the next agent's session reads them. Just shipping a fix without updating CLAUDE.md guarantees recurrence."

**Verify**: `git diff HEAD~1 CLAUDE.md` shows the four bullets above present in the diff.

## 0.8 The deeper fix: machine-verifiable end-state contract

**Owner verbatim**: *"is it more robust to define a clear end state? e.g. a brand new catalogue json file, with mandatory fields populated for every file? a playwright validation JSON file for every page with mermaid?... I need the session to be exhaustive unlike previous sessions. I am still catching quality issues."*

**Owner request to the next agent**: prioritize §0.8 ABOVE §0.1-§0.5. Build the framework first, then port the §0.1-§0.5 fixes through it. That way the framework's first proof-of-life is closing the misses that motivated it.

**Why**: §0.1-§0.7 are reactive — each owner-caught miss → bespoke fix → updated instructions. The pattern repeats because the agent's "done" is *self-reported*. The structural fix is to make "done" something an **unbiased subagent grades from artifacts**, not the executing agent's word.

### A. Catalog v4 — verification-first JSON schema

`.planning/SESSION-END-STATE.json` (complements the existing `v0.11.1-catalog.json`). Mandatory fields per file:

```json
{
  "path": "docs/how-it-works/git-layer.md",
  "status": "DONE",
  "category": "doc-page-with-mermaid",
  "claims": [
    {"id": "renders-clean", "verifier": "playwright", "artifact": ".planning/verifications/playwright/how-it-works/git-layer.json"},
    {"id": "no-banned-words", "verifier": "scripts/banned-words-lint.sh docs/how-it-works/git-layer.md", "artifact": null},
    {"id": "links-resolve", "verifier": "scripts/check-doc-links.py docs/how-it-works/git-layer.md", "artifact": null}
  ],
  "last_verified_at": "2026-04-27T20:38:00Z",
  "verifier_signature": "sha256:<hex of the verifier subagent's PASS report bytes>",
  "agent_id": "<session UUID written at session start by scripts/end-state.py init>"
}
```

`verifier_signature` is the SHA-256 of the bytes the verifier subagent emitted as its PASS evidence (so re-tampering with the artifact later invalidates the signature). `agent_id` is the session UUID written at `end-state.py init` time so the verifier can reject claims signed by an earlier session. If either field is hard to implement on day one, OMIT them from v0 and add a TODO row in `SESSION-END-STATE.md`.

The schema differs by `category` — Rust source rows demand `cargo_check_log + clippy_log + test_run_id`; workflow rows demand `last_run_id + conclusion`; doc rows demand the playwright artifact path. **Every row has an artifact path or a deterministic command an unbiased verifier can re-run.**

### B. Verification artifacts directory

`.planning/verifications/` becomes the audit trail:

- `playwright/<page-slug>.json` — `{nav_path, mermaid_svg_count, console_errors[], snapshot_taken_at, agent_id}` per docs page that has any rendered content (mermaid, admonitions, tables).
- `crates-io/<crate>.json` — `{version, published_at, install_dry_run_log_path, binstall_resolves: bool}` per published crate.
- `invariants.json` — single file listing freshness invariants from §0.6 + last-checked timestamp + verdict per invariant.
- `cargo/<crate>.json` — `{check_passed, clippy_passed, test_count, test_passed_count, last_log_path}` per crate.

### C. End-state contract written FIRST, not last

**At session START** (not end), the coordinator writes `.planning/SESSION-END-STATE.md` declaring the exhaustive set of conditions the session PROMISES to satisfy. Example:

```
- Every docs/how-it-works/*.md page MUST have a playwright artifact dated this session.
- Every reposix-* crate at the workspace version MUST be published to crates.io with binstall verified.
- mkdocs.yml nav MUST include every doc page (no orphans).
- No version-pinned filenames outside CHANGELOG.md / .planning/milestones/v*-phases/*.
- No `*ROADMAP*.md` outside a `*phases/` dir or `.planning/archive/`.
- All open dependabot PRs MUST be either merged or closed-with-rationale.
```

This contract is the session's commitment, not its retrospective.

### D. Unbiased verifier subagent

At end-of-session, the coordinator dispatches an `Explore`-typed subagent with prompt: *"Read `.planning/SESSION-END-STATE.md`. For each declared condition, find the verification artifact or run the named verifier command. Output PASS/FAIL/PARTIAL per condition with cited evidence. Refuse to grade the session GREEN unless every condition is PASS. You have NO context from the session — only the end-state contract and the artifacts."*

Subagent's output goes to `.planning/SESSION-END-STATE-VERDICT.md`. If RED, session is NOT done — fix the failing rows, re-verify. The agent cannot hand off until the verdict is GREEN.

### E. Why this fixes §0.1-§0.5

- §0.1 → contract requires playwright artifact dated this session for every how-it-works page → verifier subagent finds missing artifact → FAIL → agent must run playwright before shipping.
- §0.3 → invariant "no `v\d+\.\d+` in `docs/` or `scripts/` filenames" → verifier subagent greps → FAIL → forces rename.
- §0.4 → "every benchmark MUST appear in mkdocs.yml nav" → verifier subagent diffs nav vs `find docs/benchmarks/` → FAIL.
- §0.5 → "no `*ROADMAP*.md` outside `*phases/` dir" → verifier subagent greps → FAIL.

The verifier subagent **can't be talked out of a verdict** by the executing agent's narrative. That's the structural break in the recurrence pattern.

### F. Sizing

This is real work — likely 2-4 hours including the schema design, the script, the bootstrap contract for the current session's outstanding items, and the first verifier dispatch. It is also the single highest-leverage thing the next agent can ship: it migrates every future session from "agent's word for it" to "verifier's verdict on artifacts." The §0.6 tools are individual checks; §0.8 is the framework that hosts them.

**Verify** (when §0.8 itself ships): `python3 scripts/end-state.py verdict` exits 0 AND `.planning/SESSION-END-STATE-VERDICT.md` exists with every row PASS.

---

# HANDOVER — for the next agent picking up after the v0.11.1 sweep

**Created**: 2026-04-27 by Claude Opus 4.7 (1M context). **Operational — delete once §7 is done.**

The previous milestone (v0.11.0) is closed and tagged. v0.11.1 is FULLY published (all 8 crates on crates.io, verified at write time — re-confirm via the state-gather block). v0.11.2 release-plz auto-PR is OPEN at handoff (PR #21).

---

## 1. Read-this-first checklist

1. Run the state-gather block at the top of this file. Do NOT skip.
2. `cat .planning/STATE.md` — current cursor.
3. `git log --oneline -25` — what landed since this handoff.
4. `cat CHANGELOG.md` (`[Unreleased]` block).
5. Audit reports under `.planning/research/v0.11.1-*.md` — see §6.

---

## 2. In-flight at handoff

- **crates.io v0.11.1**: ALL 8 crates published (`reposix-core`, `cache`, `sim`, `github`, `confluence`, `jira`, `remote`, `cli`). Verified via crates.io API at write time.
- **PR #21**: release-plz auto-PR `chore: release v0.11.2`. All 8 crates bump to 0.11.2 (`reposix-core`, `cache`, `confluence`, `github`, `jira` as patch; `sim`, `cli` as API-compatible patch; `remote` as Cargo.lock-only). MERGEABLE at write time.
- **Catalog**: `.planning/v0.11.1-catalog.json` source-of-truth, auto-rendered to `.planning/research/v0.11.1-CATALOG-v3.md`. Re-render after touching anything.
- **Dependabot inbox at write time**: empty for application deps. PRs #15-#20 all MERGED. PR #21 is the only open PR.

---

## 3. Critical open items

### 3a. CI status — check live before assuming RED

At the previous handover write the CI was RED on commit `d1628a0` with `delta_sync` test failures. Since then:

- PR #18 (rusqlite 0.32→0.39) MERGED at `5108dbc`.
- Six more commits landed on top.
- Most recent observed runs on HEAD: Security audit GREEN, release-plz GREEN, CI in progress (re-check via `gh run list --branch main --limit 5`).

**Action**: re-run `gh run list --branch main --limit 5 --json status,conclusion,workflowName,headSha`. If CI is GREEN on HEAD, this row is closed — proceed to §3b. If CI is RED, the failing test signature is likely still `delta_sync_*` panicking inside `seed_demo_issues`.

**Fix path (only if CI is RED)**:
1. Reproduce locally: `cargo test -p reposix-cache --test delta_sync` (single-crate, RAM-safe).
2. Bisect the rusqlite breakage: `git log --oneline 5108dbc..HEAD -- crates/reposix-cache` — start from `5108dbc` and check whether the panic signature still matches.
3. Read the panic site at `crates/reposix-cache/tests/delta_sync.rs` (line numbers shifted post-bumps; grep for `seed_demo_issues`).
4. The rusqlite 0.32 → 0.39 BREAKING surface to scrutinize first: `Connection::execute_batch` semantics, `Row::get_unwrap` behaviour on NULL columns, and the `ToSql`/`FromSql` impls for `chrono::DateTime<Utc>` (the chrono feature flag changed name between minor releases). If you didn't bisect, dispatch a 5-min subagent: *"Extract the actual panic message from `cargo test -p reposix-cache --test delta_sync`, then list the rusqlite 0.32→0.39 BREAKING items relevant to that signature."*
5. One commit, push. Watch `gh run watch`.

**Verify**: `gh run list --branch main --workflow CI --limit 1 --json conclusion --jq '.[0].conclusion'` returns `success`.

### 3b. crates.io publish chain — likely already complete; verify and shrink

At write time, all 8 crates are at 0.11.1 on crates.io. The previous handover claimed `reposix-cli` was missing; that is no longer true. PR #21 is open to bump everything to 0.11.2.

**Action**:
1. Re-run the per-crate `curl crates.io` block at the top of this file.
2. If the workspace `Cargo.toml` version equals every published `max_version`: this row is CLOSED. Delete it.
3. If a version mismatch exists: merge PR #21 (or whatever release-plz auto-PR is open), then watch release-plz publish, then verify via the same curl loop.
4. Confirm `cargo binstall reposix-cli` resolves and `brew install reubenjohn/reposix/reposix` resolves at the latest version.

**Why §3b is unlikely to be substantial**: release-plz handled the v0.11.1 publish chain successfully; the previous handover's "publish prep error on cli" was either spurious or fixed by a subsequent commit. **Do NOT manually `cargo publish`** — release-plz manages the version-tag-publish chain; manual publishes break the bookkeeping.

**Verify**: `for c in reposix-core reposix-cache reposix-sim reposix-github reposix-confluence reposix-jira reposix-remote reposix-cli; do curl -s https://crates.io/api/v1/crates/$c | grep -o '"max_version":"[^"]*"'; done` shows the same version for all 8 AND that version equals the workspace `Cargo.toml` `version`.

### 3c. Dependabot inbox

- PR #16 (axum 0.7→0.8) — MERGED at commit `2a06ac2`.
- PR #17 (rand 0.9→0.10) — MERGED at commit `d10c1d8`.
- PR #18 (rusqlite 0.32→0.39) — MERGED at `5108dbc`. Suspected source of any lingering §3a delta_sync flake.
- PR #19 (bench-cron) + PR #20 (release-plz v0.11.1) — MERGED.
- PR #21 (release-plz v0.11.2) — OPEN; see §3b.

If new dependabot PRs appeared after write time, dispatch a fixer subagent to a worktree (`git worktree add /tmp/pr-N pr-N-branch`) — do NOT `gh pr checkout` from a bash subagent.

**Verify**: `gh pr list --state open --json number,title` returns at most the active release-plz auto-PR.

---

## 4. Friction matrix — net new this session (most v0.11.0/v0.11.1 rows RESOLVED)

| # | Friction | P | Mitigation |
|---|---|---|---|
| N1 | **Bash subagents that run `gh pr checkout` switch the coordinator's local branch.** Caused the cherry-pick mess at commit `5a91ae2` — coordinator made a commit thinking it was on `main` but was on `dependabot/cargo/axum-0.8.9`. | P0 | Never delegate `gh pr checkout` to a bash subagent without isolation. Use a worktree (`git worktree add /tmp/pr-N`) or `/tmp/<branch>-checkout`. Documented in the AUTONOMOUS rules above. |
| N2 | **Edit tool can fail silently in batched commands.** "File has been modified since read" blocks the edit but the surrounding bash commit still runs, capturing whatever's on disk. | P0 | Always verify the diff after a commit, especially when chaining edit+commit in one bash invocation. `git show --stat HEAD` after each commit. |
| N3 | **release-plz `release-pr` step can fail with HTTP 403** even when `permissions: pull-requests: write` is in the workflow file. | P1 | Repo's `default_workflow_permissions` setting is a CEILING. Fix once: `gh api -X PUT repos/<owner>/<repo>/actions/permissions/workflow -f default_workflow_permissions=write -F can_approve_pull_request_reviews=true`. Already applied for `reubenjohn/reposix`. |
| N4 | **Bash hook denies inline `<crate-loop>` longer than 300 chars.** | P2 | Promote to a script in `scripts/` or split into multiple smaller calls. Don't try to squeeze under the threshold with trivial rewrites. |

Rows from the previous handover that are now RESOLVED: rows 1, 3, 4, 6, 9 fully verified; rows 2, 5, 7, 8, 10-22 closed via POLISH2-* commits or scope-deferred to v0.12.0 with catalog refs. Row 11 (`Error::Other` 156→144 occurrences) is partial: sim.rs migrated, ~150 backend sites remain — explicitly deferred to v0.12.0 per STATE.md.

---

## 5. Spec sections — none open

Hero, capability matrix, exit-codes, and ADR-009 stability commitment all shipped. The only "spec" left is "complete the v0.11.x publish cycle": all crates at workspace version on crates.io, latest git tag pushed, GH release page populated, homebrew formula bumped. release-plz handles the mechanics.

---

## 6. Audit reports index

All in `.planning/research/`:

| File | Purpose |
|---|---|
| `v0.11.1-persona-mcp-user.md` | external evaluation friction |
| `v0.11.1-persona-harness-author.md` | integration friction |
| `v0.11.1-persona-security-lead.md` | risk-assessment friction |
| `v0.11.1-persona-skeptical-oss-maintainer.md` | critical-review friction |
| `v0.11.1-persona-coding-agent.md` | end-user dark-factory friction |
| `v0.11.1-code-quality-gaps.md` | Rust idiom + API surface gaps |
| `v0.11.1-repo-organization-gaps.md` | structural / archive cleanup |
| `v0.11.1-CATALOG-v3.md` | per-file living tracker (auto-rendered from JSON) |
| `v0.11.0-CATALOG-v1.md` | renamed from v0.11.0-CATALOG (lineage v1→v2→v3) |
| `v0.11.0-CATALOG-v2.md` | partial — most rows shipped, rest scope for v0.11.1 organization audit |

Source-of-truth JSON: `.planning/v0.11.1-catalog.json`. Driver: `scripts/catalog.py`.

---

## 7. Task list — execute top to bottom

### If you run out of time — decision tree

- **FLOOR (must hold no matter what)**: CI green on `main`; all workspace-version crates published; HANDOVER.md and `.planning/STATE.md` reflect reality; pre-push hook passes.
- **NEXT-MOST-VALUABLE if floor secured**: §7 row 2 (the §0.8 framework) — it converts every future session from self-graded to verifier-graded. One half-built framework is worth more than five hand-fixed §0 rows because it changes the slope of all future sessions.
- **ABOVE-FLOOR but compressible**: §7 rows 1, 3 are fast; do them first if time-boxed. Rows 4-8 (the §0 fixes through the framework) compress by deferring to v0.12.0 with catalog notes.

### Row 1 — Patch CLAUDE.md per §0.7 (15-30 min)

> **Verbatim row 1**: Apply the four bullets in §0.7 to `CLAUDE.md`: tighten "Docs-site validation" to whole-nav-section playwright walks; add "Cold-reader pass" section; add "Freshness invariants" list; extend "Subagent delegation rules" to forbid bash-subagent `gh pr checkout` without worktree isolation; append the "fix-the-issue + fix-the-instructions" meta-rule.

`Blocked-by:` none. `Unblocks:` rows 2-8 (every subsequent fix reads tightened CLAUDE.md). Verify: `git diff HEAD~1 CLAUDE.md` shows the four sections updated.

### Row 2 — Build the §0.8 SESSION-END-STATE framework (2-4 hr)

Per §0.8 work order: design `.planning/SESSION-END-STATE.json` schema; build `scripts/end-state.py` (`init`, `record-claim`, `verify`, `verdict` subcommands); bootstrap `.planning/SESSION-END-STATE.md` for THIS session's outstanding items (§0.1, §0.3, §0.4, §0.5, §3a, §3b); wire `end-state.py verdict` into pre-push as a non-fatal warning. `Blocked-by:` row 1. `Unblocks:` rows 3-8 (those are graded by the framework, not hand-checked). Verify: `python3 scripts/end-state.py verdict` runs and emits a verdict file with at least the §0 conditions enumerated.

### Row 3 — Run the verifier subagent against the bootstrapped contract (15 min)

Dispatch the `Explore`-typed unbiased verifier per §0.8.D. Output goes to `.planning/SESSION-END-STATE-VERDICT.md`. `Blocked-by:` row 2. `Unblocks:` rows 4-8 (the verdict's FAIL list IS the work for those rows). Verify: verdict file exists; if all rows GREEN, jump to row 9.

### Row 4 — Close the §0.1 mermaid-render FAIL (60-90 min)

Walk every `docs/how-it-works/*.md` with playwright per §0.1 fix path. Write per-page artifacts under `.planning/verifications/playwright/how-it-works/`. Diagnose any FAILs and fix. `Blocked-by:` row 3 (verdict identifies the failing pages). `Unblocks:` row 8 final verifier re-run. Verify: §0.1 verify command.

### Row 5 — Close the §0.2 install-instruction freshness FAIL (30-45 min)

Reorder `docs/index.md` and `README.md` per §0.2 fix path. `Blocked-by:` row 3. Verify: §0.2 verify command.

### Row 6 — Close the §0.3 + §0.4 doc reorganisation FAIL (45-60 min)

Per §0.3 (rename version-pinned latency files) and §0.4 (move RESULTS.md into mkdocs nav). Single mkdocs touch — batch them. `Blocked-by:` row 3. Verify: §0.3 + §0.4 verify commands.

### Row 7 — Close the §0.5 GSD-org FAIL (30 min)

Pick option A or B per §0.5; apply uniformly across v0.8/v0.9/v0.10; document the chosen convention in `CLAUDE.md` § "Workspace layout". `Blocked-by:` row 3. Verify: §0.5 verify command.

### Row 8 — Re-run the verifier subagent — must be GREEN (15 min)

Same dispatch as row 3. `Blocked-by:` rows 4-7. `Unblocks:` row 9. Verify: `.planning/SESSION-END-STATE-VERDICT.md` shows every row PASS.

### Row 9 — Original v0.11.x finish: §3a CI + §3b publish (15-60 min)

If §3a is already GREEN (likely at handoff) and §3b shows all 8 crates at workspace version (likely): merge open release-plz PR #21 → watch publish → verify the latest version on crates.io + brew + binstall. The release-plz workflow auto-tags after publish succeeds, so a separate "tag" step is folded in here. `Blocked-by:` row 8. Verify: §3a + §3b verify commands AND `git tag --sort=-creatordate | head -1` is the latest workspace version.

### Row 10 — Trigger fresh bench-latency-cron (5 min)

`gh workflow run bench-latency-cron.yml -R reubenjohn/reposix`. Confirms the latency table on the latest release reflects current numbers. `Blocked-by:` row 9.

### Row 11 — Catalog + STATE.md final pass (15 min)

`python3 scripts/catalog.py coverage` (must exit 0) → `python3 scripts/catalog.py render` → update `.planning/STATE.md` cursor → commit + push. `Blocked-by:` row 10.

### Row 12 — Delete this HANDOVER.md (1 min)

If everything above is done and main is green: `git rm HANDOVER.md && git commit -m "docs: remove HANDOVER.md (v0.11.x cycle done)" && git push`. Otherwise shrink HANDOVER.md to ONLY the open items.

---

## 8. What to NOT touch

- `.claude/skills/` — owner approval required.
- `mkdocs-material` JS / `extra_javascript` / `mkdocs.yml` — three stacked bugs were fixed in commits `66836f7`, `e119006`, `100ae00`. Don't change `fence_div_format`, don't re-enable `minify_html: true`, don't remove the mermaid CDN load.
- The `refs/reposix/origin/main` checkout step in `docs/tutorials/first-run.md` step 4 — that non-standard refspec is load-bearing.
- v0.11.0 phase dirs in `.planning/phases/` — there should be none. Don't accidentally re-create them.
- Banned word `replace` (per `scripts/banned-words-lint.sh` and `.banned-words.toml`). Use `migrate to` / `for the 80%` / `rewrite as` / `complement` / `alongside`.
- **`gh pr checkout` from a bash subagent.** Per N1. Use a worktree.

## 9. Owner preferences (durable)

- **No walkthrough / morning-brief / session-recap docs.** This HANDOVER.md is operational; delete after use.
- **Subagent delegation aggressive.** Coordinator should not type code a subagent could type.
- **Push frequently.** Pre-push hook gates fmt + clippy + check-docs-site + banned-words.
- **One cargo invocation at a time.** RAM budget per CLAUDE.md.
- **No skills changes without explicit owner approval.**
- **Owner is reubenjohn (`reubenvjohn@gmail.com`).** gh CLI is authenticated. Repo secrets settable via `gh secret set`.

---

## 10. Status snapshot at handoff (re-derive via the state-gather block)

> Snapshot captured 2026-04-27 PM. Re-derive everything below before acting; if `git log -1 --format=%H` ≠ `d10c1d8fefc91ac330869a38c29f463504e98aea`, this section is stale.

**HEAD on `origin/main`**:
```
d10c1d8 chore(deps): bump rand from 0.9.3 to 0.10.1 (#17)
2a06ac2 chore(deps): bump axum from 0.7.9 to 0.8.9 (#16)
cb0f190 docs(handover): §0.8 — machine-verifiable end-state contract (highest-leverage P0)
```

**CI on HEAD `d10c1d8`**: Security audit GREEN; release-plz GREEN; CI in progress at write time. Re-check before acting.

**Open PRs**:
- #21 — `chore: release v0.11.2` (release-plz auto-PR; MERGEABLE; bumps all 8 crates to 0.11.2 — `core/cache/confluence/github/jira` patch, `sim/cli` API-compatible patch, `remote` Cargo.lock-only).

**crates.io v0.11.1 status (verified at write time)**:
```
  reposix-core              0.11.1
  reposix-cache             0.11.1
  reposix-sim               0.11.1
  reposix-github            0.11.1
  reposix-confluence        0.11.1
  reposix-jira              0.11.1
  reposix-remote            0.11.1
  reposix-cli               0.11.1
```
ALL 8 crates published. The previous handover's "7 of 8" / "reposix-cli MISSING" is HISTORICAL.

**Tags**: `v0.11.0` shipped + binaries on GH Releases. `v0.11.1` should already exist (release-plz auto-tags after publish); verify via `git tag --sort=-creatordate | head -3`.

**Working tree**: should be clean. If confused, `git reset --hard origin/main` — origin is source of truth.

**Catalog**: re-run `python3 scripts/catalog.py coverage` to confirm 0 TODO rows.

**Owner-action items**: NONE outstanding for this session.

### Session pace observation (for the next agent's calibration)

Previous session shipped ~50 commits over 8 hours but missed the §0 quality issues (mermaid render gaps, install-instruction freshness, version-pinned filenames, RESULTS.md not in nav, GSD planning org drift). The pace was high; the *cold-reader* checks were absent. Slow down for §0 work — playwright every page you touch, owner-walk-through-as-cold-reader before declaring shipped. The §0.8 framework is the structural mitigation; ship it before the §0 row-fixes.

---

*End of handover. Good luck.*
