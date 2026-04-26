# AUTONOMOUS MODE — READ THIS FIRST

You are taking over an autonomous engineering session for the reposix project (`https://github.com/reubenjohn/reposix`). The owner (`reubenjohn` / `reubenvjohn@gmail.com`) is unreachable — driving for several hours. **Do not block on user input. Decide, ship, push.**

## Start here — do these in order before anything else

1. `cd /home/reuben/workspace/reposix`
2. Read this whole file (the HANDOVER below). It is your work queue.
3. Read `CLAUDE.md` — the **"Build memory budget"** and **"Docs-site validation"** sections in particular. The RAM rule is load-bearing (the VM has crashed twice this week from parallel cargo).
4. `ls .planning/research/v0.11.1-*.md` — 7 audit reports synthesized into §4. Read them when you need depth on a row.
5. `gh run list --workflow release --limit 1` — the v0.11.0 tag's release pipeline (HEAD `9dc4311`) was firing at handoff. Verify it succeeded: binaries on `https://github.com/reubenjohn/reposix/releases/tag/v0.11.0` and crates on `https://crates.io/crates/reposix-cli`. If either failed, diagnose + fix per §3 + §7-A.

Then execute §7 of THIS file in order (steps 7-A through 7-H). Each is 30–90 min of focused work. ~5 hours total.

## Deadline — 8pm tonight (2026-04-26 20:00 local)

You have ~8 hours from this message landing. Pace yourself against the §7 list.

- **If you finish §7 early**: do NOT idle. Expand scope into §6 ("Audit reports index") items the §7 list deferred — code-quality P1s, persona-driven page rewrites, glossary polish, the v0.11.1 milestone scaffold (REQUIREMENTS + ROADMAP + STATE), the catalog generation task you'd otherwise have to do anyway. The §7 list is the floor, not the ceiling.
- **If you're going to overshoot 20:00**: stop at the next clean checkpoint, update this file's queue, and push. Don't leave half-finished work on disk past the deadline.

## Non-negotiable rules

- **Owner is driving. Do not ask questions. Decide and proceed.**
- **You are the coordinator. You do not type code.** Coordinator's job: decide, route, verify. Subagents (`general-purpose`, `Explore`, `Plan`) do the file reads, the edits, the cargo runs, the playwright walks, the doc rewrites. **If you find yourself about to type a >20-line edit, STOP and dispatch a subagent instead.** Context-fill is the failure mode that kills these sessions; aggressive delegation is how you avoid it. Owner's global OP #2 is "Aggressive subagent delegation"; CLAUDE.md repeats it ("Coordinating agents must delegate research, implementation, and validation to subagents to prevent context fill"). This is load-bearing for an 8-hour autonomous run — your context budget will not survive doing the work yourself.
- **CATALOG-v3 is your bookkeeping.** §7-C2 generates it. **After every §7 step**, dispatch a small subagent (or do it inline if trivial) to update the rows for files you touched: flip TODO → DONE, add a one-line note. Before §7-H, verify zero TODO rows lack a queued plan. This is the audit trail that prevents owner's "things slip through the cracks when I spot check" — the previous session let scope creep happen because nothing was tracking what hadn't been touched yet.
- **No skill changes.** `.claude/skills/` is owner-approval-gated.
- **One cargo invocation at a time, ever.** `cargo check -p <crate>`, never `--workspace`. CLAUDE.md "Build memory budget" explains why. If a subagent does cargo work, it has the lock — don't dispatch a second cargo subagent in parallel.
- **Push frequently** (every commit). Pre-push hook runs fmt + clippy + `scripts/check-docs-site.sh` — let it gate. Don't `--no-verify`.
- **For docs-site work**, additionally validate via playwright per CLAUDE.md "Docs-site validation". The mermaid render bug from v0.11.0 is the reason this rule exists.
- **No retrospective / walkthrough / morning-brief docs.** This file is operational; delete it once §7 is done. Owner explicitly does not want session-recap files.
- **Banned word: "replace".** Use "complement", "alongside", "for the 80%". See `.planning/research/v0.11.0-vision-and-innovations.md` §8.

## Subagent dispatch cookbook (use this; don't reinvent)

For each §7 step, the default pattern is:

1. **Coordinator** (you) reads HANDOVER §7-X enough to write a self-contained subagent brief. Cite file paths, line numbers, audit-report rows. Don't hand off vague intent.
2. **Dispatch** via `Agent` tool, `subagent_type: general-purpose`, with the brief. For pure-doc work, set `run_in_background: true` if you can run the next step in parallel. For cargo-touching work, run foreground and serialize.
3. **Subagent** does the work, runs validation, commits, pushes — return a 200-word report.
4. **Coordinator** reads the report (NOT the full transcript), updates HANDOVER §7-X status, checks CI green via `gh run list`, moves to the next step.

If a subagent's brief would be more than ~200 words, the step is too big — split it.

**Parallelizable** (no cargo, no file overlap):
- doc-only edits in disjoint subdirs (e.g. `docs/reference/` vs `docs/concepts/`)
- repo-cleanup deletes in disjoint dirs (`scripts/demos/` vs `.planning/milestones/`)
- audit / read-only investigations
- playwright site walks

**Serialize** (anything cargo, anything touching `Cargo.toml` or `Cargo.lock`):
- code refactors
- new CLI subcommands
- dependency bumps

## When you finish §7-H

- Update this file: either delete it (if §7 is genuinely done) or shrink it to ONLY the items still pending. Don't append a "I did X" log — just update the queue.
- Update `.planning/STATE.md` with the new cursor.
- Commit + push.
- If time remains before 20:00 and queue is empty: see "If you finish §7 early" above. Expand scope.
- At 20:00: stop. The owner will check back when they're back online.

## If something goes wrong

- **VM crash**: see CLAUDE.md "If the VM crashes again" — suspect parallel cargo, check `ps aux` for orphans, fewer subagents in flight.
- **CI red**: `gh run view <id> --log-failed | head -30`. Don't bypass; fix.
- **Release pipeline red**: most likely a Windows or cross-compile issue beyond what's already gated. See §3b.
- **crates.io 403**: token-scope issue. The owner regenerated the token with `publish-new` + `publish-update` + `yank` scopes (restricted to `reposix-*`) and we re-provisioned the gh secret at handoff. If `release-plz` STILL 403's, the scopes weren't right — flag in this file, skip §7-A's crates.io step.
- **Owner returns asking "what did you do?"**: point at `git log origin/main..HEAD` and this file's queue updates.

Branch: `main`. No feature branches. The pre-push hook is your safety net. Trust it. The previous agent (Claude Opus 4.7 1M, this same session-flavor) shipped 50+ commits in the last 24h without breaking main. You can do the same.

**Go.**

---

# HANDOVER — for the next agent picking up after v0.11.0

**Created**: 2026-04-26 by Claude Opus 4.7 (1M context). Updated 2026-04-26 (later that day): owner returned briefly mid-handover, paired down the friction list, then handed back to autonomous mode for the rest of a multi-hour drive. **Operational — delete once §7 is done. Do not append "what I did" entries; just update the queue.**

The previous milestone (v0.11.0 — Polish & Reproducibility) shipped 50+ commits and closed all 17 POLISH-* requirements (see `.planning/REQUIREMENTS.md`). Release-pipeline retag (round 3) is in flight at HEAD `9dc4311`; release.yml binaries + release-plz crates.io publish + homebrew formula push all expected to succeed this round. Several persona / code-quality audits surfaced gaps the milestone didn't cover — synthesized into §4 and queued in §7.

**Latest in-session updates** (post-original-handoff, before the autonomous handback):
- v0.11.0 tag force-updated to `9dc4311` (windows compile fix round-2 + JIRA cache routing fix).
- `CARGO_REGISTRY_TOKEN` re-provisioned 2026-04-26 — owner regenerated with `publish-new` + `publish-update` + `yank` scopes, restricted to `reposix-*`. release-plz should now ship.
- `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT` + `HOMEBREW_TAP_TOKEN` all provisioned. Confluence latency cell + homebrew formula push both unblocked.
- 89.1% / 92.3% inconsistency scrubbed across 7 surfaces. GitHub repo description updated. `git checkout origin/main` → `refs/reposix/origin/main` corrected in README + index. JIRA cache slug bug fixed. GitHub token Debug-leak fixed.

---

## 1. Read-this-first checklist

Before touching anything else:

1. `cat .planning/STATE.md` — current cursor + frontmatter `status:`.
2. `git log --oneline -25` — what landed since the previous handoff.
3. `gh run list --workflow release --limit 3 --json status,conclusion,headSha` — has the v0.11.0 retag's release pipeline finished? If not, watch it before doing anything else.
4. `cat CHANGELOG.md` (`[Unreleased]` block) — what's pending docs/notes.
5. `cat CLAUDE.md` — esp. the **Build memory budget** section. **Do not run parallel cargo workspace builds.** Run one cargo invocation at a time, prefer `-p <crate>` over `--workspace`.
6. Audit reports under `.planning/research/v0.11.1-*.md` — see §6 below for the index.

---

## 2. In-flight at handoff

The v0.11.0 tag was force-updated from `8d3bbd0` (broken release) to `9d585bb` (with cross-compile fixes). The release pipeline was firing as this doc was written. **First action: verify it succeeded.**

```bash
gh run list --workflow release --limit 1 --json status,conclusion,headSha
gh run view <id> --json jobs -q '.jobs[] | "\(.status)\t\(.conclusion // "—")\t\(.name)"'
```

Expected: `plan` ✓, 4 build matrix jobs ✓ (linux-x86_64-musl, macOS x86 + arm64, windows-msvc), `host` ✓ (creates the GH Release page), `upload-homebrew-formula` ✓ (HOMEBREW_TAP_TOKEN is set).

If any build fails, the host job skips, and there's no Release page. Pre-1.0; you can move the tag again if needed.

---

## 3. Critical open items (pre-condition for "v0.11.0 is actually shipped")

### 3a. release-plz crates.io publish — TOKEN SCOPE
**Symptom**: `release-plz` workflow at `8d3ac34` (and previously) fails with `403 Forbidden — "this token does not have the required permissions to perform this action"`.

**Cause**: `CARGO_REGISTRY_TOKEN` was generated without `publish-new` scope. The first publish needs to register the crate name; subsequent versions only need `publish-update`.

**Fix**:
1. https://crates.io/me → "API Tokens" → "New Token"
2. Scopes: ✅ `publish-new` ✅ `publish-update`. Optional: ✅ `yank`.
3. Crate-pattern: leave unrestricted, OR restrict to `reposix-*` (the safer choice).
4. Copy → `gh secret set CARGO_REGISTRY_TOKEN -R reubenjohn/reposix < token.txt`
5. `gh workflow run release-plz.yml -R reubenjohn/reposix`
6. Watch: `gh run watch <id>`. The first run publishes 9 crates in topological order: reposix-core → reposix-cache + reposix-sim → reposix-github + reposix-confluence + reposix-jira → reposix-remote → reposix-cli (`reposix-swarm` is `publish=false`).

### 3b. linux-aarch64-musl held back
**Status**: target dropped from `[workspace.metadata.dist].targets` in commit `9d585bb` to unblock v0.11.0. The link step failed because GitHub-hosted runners default to `aarch64-linux-gnu-gcc` — linking a musl arm64 binary needs `aarch64-linux-musl-gcc` configured explicitly.

**Fix path** (for v0.11.1):
- Use `cross` (https://github.com/cross-rs/cross) for cross-compilation — `dist` supports `--cross-compiler cross` for aarch64 targets.
- OR pin a self-hosted runner with the musl toolchain.
- OR use the `cargo-zigbuild` action that ships a working musl-linker.

Track in v0.11.1 milestone (§7).

### 3c. Cargo.lock duplicate-version warnings (latent)
Running `cargo tree --duplicates` will likely surface 5–10 duplicates from the gix chain (gix transitively pins different `gix-*` minor versions). Not blocking — but `release-plz` will warn on every run. Track for v0.12.0; not v0.11.0 work.

---

## 4. Persona audits — synthesized friction matrix

**Five persona audits ran in parallel during the handoff.** Each was instructed to walk the live site as a specific reader:

| File | Persona | Stance |
|---|---|---|
| `.planning/research/v0.11.1-persona-mcp-user.md` | First-time MCP user evaluating reposix as alternative | cautiously interested, skeptical |
| `.planning/research/v0.11.1-persona-harness-author.md` | OSS coding-agent harness maintainer | considering integration |
| `.planning/research/v0.11.1-persona-security-lead.md` | Series B fintech security lead | writing risk assessment |
| `.planning/research/v0.11.1-persona-skeptical-oss-maintainer.md` | dtolnay-tier OSS maintainer | 15-minute critical review |
| `.planning/research/v0.11.1-persona-coding-agent.md` | LLM agent in dark-factory loop | actual end-user |

**Read all 5 first.** Synthesized friction matrix (P0 = block-the-pilot, P1 = block-the-recommend, P2 = polish):

| # | Friction | Personas | P | Fix |
|---|---|---|---|---|
| 1 | **Headline `92.3%` was wrong; corrected to `89.1%`** across README, docs/index.md, demos/index.md, social/_build_*.py, social/benchmark.svg in this handoff (commit `f6345dd`). MCP comparison fixture is synthesized — not measured against a real MCP server. Add a clearer methodology callout to the vs-MCP page. | mcp-user, skeptical-oss | P0 | mostly done, methodology callout pending |
| 2 | **JIRA cache routing bug**: `backend_slug_from_origin(spec.origin)` returned "confluence" for any atlassian.net host. JIRA worktrees pointed at the confluence cache. Fixed in commit (this handoff): now takes the full URL and reads `/jira/` vs `/confluence/` markers. Doctor.rs:271/697 still pass `spec.origin` (display-only impact, not data-corrupting). | code-quality | P0 | partially done; doctor.rs needs the same fix |
| 3 | **GitHub repo description still says FUSE** (until commit `f6345dd` of this handoff which `gh repo edit`'d it). Verify on hover. | harness-author, skeptical-oss | P0 | done, verify |
| 4 | **`/benchmarks/RESULTS/` 404** on the live site. README + index.md hero now point at the absolute github URL. Verify on next deploy. | mcp-user | P0 | done, verify |
| 5 | **Latency table real-backend cells empty**. Atlassian secrets provisioned in this handoff (commit `ce2f577` time). Re-trigger `bench-latency-v09` to populate confluence column. JIRA still has no creds. | mcp-user, harness-author, skeptical-oss | P0 | secrets in; bench rerun pending |
| 6 | **Site footer chip says `v0.8.0`** while page text references v0.9.0/v0.11.0. Probably hardcoded in `mkdocs.yml extra` or a theme override. Find and bump. | mcp-user, harness-author | P0 | not started |
| 7 | **Hero example uses JIRA-style `PROJ-42`** but JIRA backend is read-only; tutorial says `sed + git push` — the example is DOA on JIRA. Either change the headline ID to a generic `0001` (sim-style) or add a connector capability matrix to make capabilities visible. | coding-agent | P0 | not started |
| 8 | **`git checkout origin/main` vs `git checkout -B main refs/reposix/origin/main` inconsistency**. Fixed in README + docs/index.md in this handoff. Mental-model + concept pages may still have stale form — grep and fix. | coding-agent, harness-author | P0 | partially done |
| 9 | **GITHUB_TOKEN auto-derived `Debug` leak**. Fixed in commit `ce2f577`. | security-lead | P0 | done |
| 10 | **Phase-51 worktree_helpers re-duplicated** by Phase 55 work — `cache_path_from_worktree` thin-wrappers in gc.rs:166, tokens.rs:69, cost.rs:282 each delegate to canonical but defeat the de-dup intent. Consider inlining or accepting them as thin existence-check wrappers. | code-quality | P1 | not started |
| 11 | **`Error::Other(String)` 153 occurrences** dominates internal error vocabulary; round-trips JSON through error messages in sim.rs. Replace with typed variants. | code-quality | P1 | not started |
| 12 | **Two parallel audit-log schemas** (`audit_events_cache` in cache crate, `audit_events` in sim/confluence/jira) means `reposix doctor` only checks the cache schema. Unify or document. | code-quality, security-lead | P1 | not started |
| 13 | **No v1.0 stability commitment** anywhere on `/decisions/`. ADR-008 itself documents a breaking URL-shape change in v0.10.0 — bad signal. | harness-author | P1 | doc-only ADR write |
| 14 | **No documented exit codes / `--json`/`--format=json` output / canonical env-var page / concurrency contract / already-init'd-directory behavior**. Six concrete machine-readability gaps the harness persona surfaced. | harness-author | P1 | doc + small CLI flags |
| 15 | **Internal ADRs (002, 003 — deprecated FUSE) still in nav**. `docs/decisions/002-confluence-page-mapping.md` is current (kept ADR); `003-nested-mount-layout.md` references the FUSE-era nested-mount that's gone — mark superseded or delete. | mcp-user, skeptical-oss | P1 | nav cleanup |
| 16 | **`scripts/demos/` (11 files) + `docs/demos/recordings/` (12 typescripts) entirely FUSE-era**. Repo-org audit recommends deletion. ~280 files. | repo-org | P1 | bulk delete |
| 17 | **`.planning/milestones/v0.{1..8}.0-phases/` is 273 files** (74% of `.planning/` markdown). Condense each into a single `ARCHIVE.md`. | repo-org | P2 | bulk condense |
| 18 | **`scripts/__pycache__/*.pyc` is committed**. .gitignore miss. | repo-org | P2 | gitignore + git rm |
| 19 | **Supply chain: no signing, no SBOM, no SLSA, no cosign, no Apple notarization, no Authenticode**. v0.11.0 ships `curl | sh`. | security-lead | P2 | release.yml extension |
| 20 | **`research/threat-model-and-critique.md` referenced by CLAUDE.md but missing or unpublished**. | security-lead | P2 | write or delete the ref |

---

## 5. Hero rewrite spec — homepage doesn't sell hard enough

**Owner feedback verbatim**: *"I still feel like the home page doesn't sell this project enough. e.g. how many tokens are being saved, maybe a high level timing diagram."*

**Current home page** (`docs/index.md`): leads with mental-model framing + `cargo run reposix init` example. It DOES include 4 ms numbers (8/24/9/5 for cache-read, init, list, capabilities) but no token-cost numbers, no MCP comparison, no visual.

**v0.11.1 hero block (proposed structure):**

1. **One-liner**: existing.
2. **Hard-numbers strip** (3 cells, one row):
   - 92.3% token reduction vs MCP for the same task suite (cite `benchmarks/RESULTS.md`)
   - 8 ms cached read, 24 ms cold init (cite `docs/benchmarks/v0.9.0-latency.md`)
   - 5-line install (cite `docs/tutorials/first-run.md` step 1)
3. **High-level timing diagram** (mermaid sequenceDiagram) comparing reposix-loop vs MCP-loop for "agent reads issue, edits, posts comment":
   - reposix path: cat → grep → edit → git push → done. ~3 REST calls.
   - MCP path: list_tools → call_tool(get_issue) → call_tool(post_comment) → ~6 round trips with schema-rendered JSON.
   - Numbers from the `benchmarks/RESULTS.md` token-economy comparison.
4. **"Install in 30 seconds"** band: the curl/PowerShell/brew/binstall paths from `tutorials/first-run.md` step 1.
5. **Existing content** (concept links, etc.) — moves below the hero.

**Caveat**: the mermaid render workaround (`docs/javascripts/mermaid-render.js`) MUST be preserved. Test the new diagram against `bash scripts/check-docs-site.sh` AND a playwright pre-flight (per CLAUDE.md "Docs-site validation").

---

## 6. Audit reports index — load before deep work

All in `.planning/research/v0.11.1-*.md`:

| File | Purpose |
|---|---|
| `v0.11.1-persona-mcp-user.md` | external evaluation friction |
| `v0.11.1-persona-harness-author.md` | integration friction |
| `v0.11.1-persona-security-lead.md` | risk-assessment friction |
| `v0.11.1-persona-skeptical-oss-maintainer.md` | critical-review friction |
| `v0.11.1-persona-coding-agent.md` | end-user dark-factory friction |
| `v0.11.1-code-quality-gaps.md` | Rust idiom + API surface gaps |
| `v0.11.1-repo-organization-gaps.md` | structural / archive cleanup |

**Plus existing v0.11.0 research that's still relevant** (don't re-do these):

| File | Still useful? |
|---|---|
| `.planning/research/v0.11.0-vision-and-innovations.md` | YES — strategy direction + §8 owner decisions |
| `.planning/research/v0.11.0-CATALOG-v2.md` | partial — ~17/38 refactors shipped; the rest is scope for the v0.11.1 organization audit |
| `.planning/research/v0.11.0-mkdocs-site-audit.md` | mostly done; mermaid actually fixed via the JS workaround |
| `.planning/research/v0.11.0-jargon-inventory.md` | done; absorbed into the glossary |
| `.planning/research/v0.11.0-latency-benchmark-plan.md` | partial — confluence + jira cells still need a CI run with the just-provisioned secrets (§7-A) |
| `.planning/research/v0.11.0-release-binaries-plan.md` | mostly shipped; arm64-musl follow-up |
| `.planning/research/v0.11.0-cache-location-study.md` | resolved (kept XDG, added `gc --orphans`) |

---

## 7. The 6-hour task list (in order — do not skip ahead)

### 7-A. Verify v0.11.0 release shipped end-to-end (15 min)
- `gh run list --workflow release --limit 1 --json status,conclusion`
- Open https://github.com/reubenjohn/reposix/releases/tag/v0.11.0 — does the release page exist with binaries attached?
- Try `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh` in a clean docker container.
- Do `release-plz` retry per §3a once the owner provides the new token.

### 7-B. Confluence latency cell — re-run bench with the just-provisioned secrets (10 min)
The Atlassian secrets (`ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`) were set on `2026-04-26T16:48`. The `bench-latency-v09` CI job hadn't run yet at handoff. Trigger it:
- `gh workflow run ci.yml -R reubenjohn/reposix` (or push a trivial commit)
- After completion: download the `latency-table` artifact, inspect the confluence column.
- If populated: commit the regenerated `docs/benchmarks/v0.9.0-latency.md`. (Or wait for the weekly cron PR.)

### 7-C. Synthesize the persona-friction matrix (already done — verify and extend)
The previous coordinator already populated §4 of THIS doc with a 20-row P0/P1/P2 matrix synthesizing all 5 persona files plus the code-quality and repo-org audits. **Read §4 first.** If a row's status reads "done, verify" — verify it on the live site / in the codebase. If a row reads "not started" — that's part of your §7 queue (already mapped to specific subsequent steps).

If the audits surface NEW frictions you think the matrix missed, append rows. Don't rewrite existing rows.

### 7-C2. Generate `CATALOG-v3.md` — the LIVING per-file tracker (90 min initial; ongoing thereafter)

**Owner verbatim ask** (with critical clarification): *"The catalogue is to ensure the agent can keep track of every file and what needs to be done so it doesn't miss anything. Because that's what I've noticed — many things slip through the cracks when I spot check."*

This is **not a one-shot audit** — it's a working dashboard the autonomous agent uses to verify EVERY file got considered. Critical for an 8-hour run where small omissions slip past.

**Format** — write to `.planning/research/v0.11.1-CATALOG-v3.md`:

```markdown
# CATALOG-v3 — living per-file tracker

Updated: <datetime>  ·  Files tracked: N  ·  TODO: N  ·  DONE-this-session: N  ·  KEEP: N

## How to use this file

After every §7 step, find the rows for files you touched and flip status to DONE
with a one-line note. Before stopping the session, verify:
1. No row has status TODO that the §7 queue claimed was done.
2. Every file in DELETE has been git-rm'd.
3. Every file in REFACTOR has a follow-up task either complete or surfaced
   in the §4 friction matrix or §7 queue.

## Per-directory dashboard

| Dir | Files | KEEP | TODO | DONE | REVIEW | DELETE | Notes |
|---|---|---|---|---|---|---|---|

## Tracker (every file with status)

| Path | Status | Owner-prior-audit | Action | Notes |
|---|---|---|---|---|
| crates/reposix-core/src/lib.rs | KEEP | catalog-v2:KEEP | — | well-maintained pub API |
| crates/reposix-cli/src/cache_db.rs | TODO | catalog-v2:DELETE | DELETE-DEFERRED | Phase 51-C kept as option-b; revisit in v0.12.0 |
| ... | ... | ... | ... | ... |
```

**Generation strategy**:
1. Dispatch a `general-purpose` subagent (read-only, no cargo) with a ≤200-word brief that:
   - runs `git ls-files` for the canonical list (~600 files)
   - reads each file's head + cross-refs prior audits (`v0.11.0-CATALOG-v2.md`, `v0.11.1-repo-organization-gaps.md`, `v0.11.1-code-quality-gaps.md`)
   - assigns initial status: KEEP / TODO / DONE / REVIEW / DELETE
   - structures the output as above
2. After the subagent returns, scan the TODO rows. Many will map to existing §7 steps; map them explicitly in the row's Notes column.
3. **From this point onward, every §7 step ends with "update CATALOG-v3 rows for the files I touched"**. This is the audit trail.

**Coverage discipline**: at §7-H (final state), the catalog table must have ZERO rows with `Status: TODO` that aren't also in the open queue or §4 friction matrix. If a TODO row has no plan, surface it in HANDOVER §7 or delete the file.

### 7-D. Hero rewrite — implement §5 (60 min)
Write the new hero in `docs/index.md`. The mermaid timing diagram must:
- Use clean `sequenceDiagram` syntax (no `<br/>`, no `<id>` literals — see commit `b685dfb` for the lesson).
- Validate via `bash scripts/check-docs-site.sh`.
- Validate via playwright (CLAUDE.md "Docs-site validation"): load page, confirm `<svg>` rendered.
- Token-cost numbers cite `benchmarks/RESULTS.md`'s 92.3% reduction.

### 7-E. Apply code-quality P0 fixes (90 min)
Read `.planning/research/v0.11.1-code-quality-gaps.md`. Pick the P0 list. Apply each as one commit:
- Per-crate cargo check after each (CLAUDE.md RAM rule).
- Push after each commit.

### 7-F. Repo cleanup pass (60 min)
Read `.planning/research/v0.11.1-repo-organization-gaps.md`. Apply the "archive/delete" verdicts:
- `.planning/milestones/v0.X.0-phases/` — condense or archive per the audit's recommendation.
- `scripts/` deletions — only delete after confirming no CI / docs reference each script.
- Top-level orphans (CONTRIBUTING.md staleness, etc.).
- One commit per logical group.

### 7-G. Open the v0.11.1 milestone scaffold (30 min)
After 7-A through 7-F, open v0.11.1 in `.planning/REQUIREMENTS.md` + `ROADMAP.md`. Carry-forward items:
- linux-aarch64-musl cross-link (§3b)
- crates.io publish completion (§3a if not done by then)
- Hero shipped or partially-shipped depending on owner review
- Whatever P1 items the audits surface
- Upstream issue against `squidfunk/mkdocs-material` for the `<pre.mermaid>` content-strip bug — file it; link from `.planning/research/v0.11.0-mkdocs-site-audit.md`.

### 7-H. State + push final (15 min)
- Update `.planning/STATE.md` with the new cursor.
- Commit + push.
- Run `bash scripts/check-docs-site.sh` once more.
- Verify CI green.
- Stop.

**Total: ~5 hours active + 1 hour buffer.**

---

## 8. What to NOT touch

- `.claude/skills/` — owner approval required. There are 2 skills (`reposix-banned-words`, `reposix-agent-flow`); leave them.
- `mkdocs-material` JS / `extra_javascript`/`mkdocs.yml` — three stacked bugs were fixed in commits `66836f7`, `e119006`, `100ae00`. Don't change `fence_div_format`, don't re-enable `minify_html: true`, don't remove the mermaid CDN load. Read those commit messages before touching anything mermaid-adjacent.
- The `refs/reposix/origin/main` checkout step in `docs/tutorials/first-run.md` step 4 — that's the actual root cause of the broken-quickstart bug owner reported. The non-standard refspec is load-bearing.
- v0.11.0 phase dirs in `.planning/phases/` — there should be none (Phase 50 archived Phase 30 to `.planning/milestones/v0.9.0-phases/`). Don't accidentally re-create them.
- Banned words: `replace` (per the banned-words linter `scripts/banned-words-lint.sh` and `.banned-words.toml`). Use `complement`, `alongside`, `for the 80%`. Owner explicitly ratified "complement for the 80%, replace nothing." in `.planning/research/v0.11.0-vision-and-innovations.md` §8.

## 9. Owner preferences (durable)

- **No walkthrough / morning-brief / session-recap docs.** This HANDOVER.md is operational; delete after use. Don't write retrospectives.
- **Subagent delegation aggressive.** Coordinator should not type code a subagent could type.
- **Push frequently.** Pre-push hook runs fmt + clippy + check-docs-site.sh — let it gate.
- **One cargo invocation at a time.** RAM budget. See CLAUDE.md.
- **No skills changes without explicit owner approval.**
- **Owner is reubenjohn (`reubenvjohn@gmail.com`).** gh CLI is authenticated. Repo secrets you can set via `gh secret set`.

---

## 10. Status snapshot at handoff

- Branch: `main`
- HEAD: `9d585bb` (fix(release): unblock v0.11.0 — windows compile + drop arm64-musl)
- Tag: `v0.11.0` → `9d585bb` (force-updated from `8d3bbd0`)
- Working tree: clean (verify with `git status --short`)
- CI on HEAD: green (`gh run list --branch main --limit 1`)
- Docs site: green (mermaid diagrams render, glossary live, 24 jargon terms)
- 49 commits since Phase 50 close (8158e2d..9d585bb), all 17 POLISH-* requirements satisfied
- crates.io publish: BLOCKED on token scope (§3a)
- Homebrew tap: `reubenjohn/homebrew-reposix` exists, scaffolded; first formula push pending the next release run

---

*End of handover. Good luck. The previous agent (Claude Opus 4.7 1M) is on standby for direct hand-off if you have questions.*
