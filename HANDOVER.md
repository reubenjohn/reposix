# AUTONOMOUS MODE — READ THIS FIRST

You are taking over an autonomous engineering session for the reposix project (`https://github.com/reubenjohn/reposix`). The owner (`reubenjohn` / `reubenvjohn@gmail.com`) is offline. **Do not block on user input. Decide, ship, push.**

## Start here — do these in order before anything else

1. `cd /home/reuben/workspace/reposix`
2. Read this whole file. It is your work queue.
3. Read `CLAUDE.md` — the **"Build memory budget"** section is load-bearing (the VM has crashed twice from parallel cargo).
4. `gh run list --branch main --limit 5 --json status,conclusion,name,headSha` — **CI on HEAD `12c2ec1` is RED** (delta_sync test failures). See §3a.
5. `gh pr view 20 --json mergeStateStatus,statusCheckRollup` — release-plz auto-PR for v0.11.1 is open and needs attention before publish can complete. See §3b.
6. `python3 scripts/catalog.py render` — refresh the JSON-backed catalog after touching anything.

Then execute §7 of THIS file in order. Total estimate: ~3 hours of focused work — most v0.11.1 scope is already shipped.

## Deadline — none fixed; work to next owner check-in

This is not time-boxed. Pace against §7. When the queue is empty, expand into §6 backlog or pause cleanly.

## Non-negotiable rules

- **Owner is offline. Do not ask questions. Decide and proceed.**
- **You are the coordinator. You do not type code.** Coordinator's job: decide, route, verify. Subagents do the file reads, the edits, the cargo runs, the playwright walks. **If you find yourself about to type a >20-line edit, STOP and dispatch a subagent.** Aggressive delegation is owner's global OP #2.
- **CATALOG-v3 is your bookkeeping.** After every §7 step: `python3 scripts/catalog.py set <path> DONE --note "ref §7-X"`. Run `coverage` before §7 close.
- **No skill changes.** `.claude/skills/` is owner-approval-gated.
- **One cargo invocation at a time, ever.** `cargo check -p <crate>`, never `--workspace`. CLAUDE.md "Build memory budget" explains why. If a subagent does cargo work, it has the lock.
- **Push frequently** (every commit). Pre-push hook runs fmt + clippy + `scripts/check-docs-site.sh` + the banned-words linter — let it gate. Don't `--no-verify`.
- **Never delegate `gh pr checkout` to a bash subagent without isolating the working tree.** It switches the coordinator's branch behind your back. Use `EnterWorktree` or have the subagent cd to `/tmp/<branch>-checkout`. Lesson learned at commit `5a91ae2` (cherry-pick mess).
- **No retrospective / walkthrough / morning-brief docs.** This file is operational; delete it once §7 is done. Owner explicitly does not want session-recap files.
- **Banned word: `replace`.** Use `complement` / `alongside` / `for the 80%` / `migrate to`. The pre-push hook enforces it.

## Subagent dispatch cookbook

Same as previous handover: brief in ≤200 words → dispatch via `Agent` (subagent_type: `general-purpose`) → read the report not the transcript → update §7 status → next.

**Parallelizable**: doc edits in disjoint subdirs, audits, playwright walks.
**Serialize**: anything that compiles (cargo / Cargo.toml / Cargo.lock).

## When the queue is empty

- Update or delete this file. Don't append "I did X" entries.
- Update `.planning/STATE.md` with the new cursor.
- Commit + push.
- If CI is green, release-plz published all 9 crates, dependabot inbox empty: stop.

**Go.**

---

# HANDOVER — for the next agent picking up after the v0.11.1 sweep

**Created**: 2026-04-27 by Claude Opus 4.7 (1M context). **Operational — delete once §7 is done.**

The previous milestone (v0.11.0) is closed and tagged. v0.11.1 substantively shipped (~18-20 of 22 POLISH2-* requirements over 50+ commits): typed Error variants (sim.rs migrated; jira/confluence/github extended to `NotSupported`), reposix-doctor capability check, `exit-codes.md`, ADR-009 v1.0 stability commitment, jira+confluence lib.rs splits, `scripts/catalog.py` JSON-first tracker, mkdocs-material upstream issue #8584, dual-audit-schema endorsed in module docs. Crates.io publish chain is in flight — release-plz auto-PR #20 opened for v0.11.1 across 8 crates; first publish run failed (see §3b).

---

## 1. Read-this-first checklist

1. `cat .planning/STATE.md` — current cursor.
2. `git log --oneline -25` — what landed since this handoff.
3. `gh run list --branch main --limit 5` — CI is RED on HEAD `12c2ec1`. See §3a before anything else.
4. `gh pr list --state open` — three PRs open: #20 (release-plz v0.11.1 chore), #16 (axum 0.7→0.8 dependabot), #17 (rand 0.9→0.10 dependabot).
5. `cat CHANGELOG.md` (`[Unreleased]` block).
6. Audit reports under `.planning/research/v0.11.1-*.md` — see §6.

---

## 2. In-flight at handoff

- **release-plz** (run `24976688322`) — failed at the publish step. PR #20 is OPEN with v0.11.1 bumps for 8 crates. `reposix-core` v0.11.0 was already published successfully. The next 7 (cache, sim, github, confluence, jira, remote, cli) need PR #20 merged + a re-trigger after the prep failure is investigated. See §3b.
- **Dependabot fixer subagent** (id `a825d495b91ff734b`) was dispatched for PR #16 (axum) + PR #17 (rand) before this handoff; status TBD. Verify with `gh pr view 16` / `gh pr view 17`. PR #18 (rusqlite 0.32→0.39) and #19 (bench-cron auto-PR) are already MERGED.
- **Catalog**: `.planning/v0.11.1-catalog.json` is current (~84 files, 0 TODO post-condensation per STATE.md). Auto-rendered to `.planning/research/v0.11.1-CATALOG-v3.md`. Re-render after touching anything: `python3 scripts/catalog.py render`.

---

## 3. Critical open items

### 3a. CI RED on main (HEAD `12c2ec1`) — `delta_sync` test failures

**Symptom**: `cargo test --workspace --locked` fails in `crates/reposix-cache/tests/delta_sync.rs`:

```
delta_sync_empty_delta_still_writes_audit_and_bumps_cursor (line 309 / 336)
delta_sync_updates_only_changed_issue (line 218)
```

Both panic at `delta_sync.rs:124` inside the shared `seed_demo_issues` helper. PR #18 (rusqlite 0.32→0.39) is the most likely culprit — it landed at commit `5108dbc`, just before HEAD. The `bundled` feature semantics or row-iteration API may have shifted.

**Fix path**:
1. Reproduce locally: `cargo test -p reposix-cache --test delta_sync` (single-crate, RAM-safe).
2. Read the panic site at `crates/reposix-cache/tests/delta_sync.rs:124`.
3. If rusqlite 0.39's API is the cause, fix `seed_demo_issues` and the two callers.
4. One commit, push. Watch `gh run watch`.

### 3b. release-plz publish failed at `reposix-sim`

**Symptom**: After `reposix-core` 0.11.0 published cleanly (last session), the v0.11.1 publish chain failed for `reposix-sim` with:

```
failed to open file `/home/runner/work/reposix/reposix/crates/reposix-sim/CHANGELOG.md`: No such file or directory
failed to publish reposix-sim: failed to prepare local package for uploading
```

The "failed to prepare local package" is the real error; the missing per-crate CHANGELOG is just a warning. Most likely cause: `reposix-sim/Cargo.toml` is missing `description`, `license`, or `repository` (cargo refuses to publish without all three), OR a path-dep version field is still wrong even after commits `e8aebfa` + `12c2ec1`.

**Fix path**:
1. `cargo publish -p reposix-sim --dry-run` locally to surface the exact prep error.
2. Fix the missing field(s) in the relevant `Cargo.toml`. Same audit for `reposix-cache`, `reposix-github`, `reposix-confluence`, `reposix-jira`, `reposix-remote`, `reposix-cli` — likely all 7 hit the same wall.
3. Once green, merge PR #20. release-plz will pick up the merge and finish publishing all 8.
4. After all 9 crates are on crates.io, tag `v0.11.1` (release-plz handles this via the `release` step after publish succeeds).

### 3c. Dependabot inbox

- PR #16 — `axum 0.7→0.8`. Major bump; touches `reposix-sim`. Verify the fixer subagent's commits compile per-crate before merging.
- PR #17 — `rand 0.9→0.10`. Smaller surface; check `reposix-sim` (seed-data RNG) and any swarm-test usage.
- PR #18 — `rusqlite 0.32→0.39` ✓ MERGED but suspected to have introduced the §3a failure.

---

## 4. Friction matrix — net new this session (most v0.11.0/v0.11.1 rows RESOLVED)

The 23-row matrix from the previous handover is fully verified-resolved or shipped via POLISH2-*. Net new frictions surfaced this session:

| # | Friction | P | Mitigation |
|---|---|---|---|
| N1 | **Bash subagents that run `gh pr checkout` switch the coordinator's local branch.** Caused the cherry-pick mess at commit `5a91ae2` — coordinator made a commit thinking it was on `main` but was on `dependabot/cargo/axum-0.8.9`. | P0 | Never delegate `gh pr checkout` to a bash subagent without isolation. Use `EnterWorktree` or have the subagent operate in `/tmp/<branch>-checkout`. Documented in the AUTONOMOUS rules above. |
| N2 | **Edit tool can fail silently in batched commands.** "File has been modified since read" blocks the edit but the surrounding bash commit still runs, capturing whatever's on disk. | P0 | Always verify the diff after a commit, especially when chaining edit+commit in one bash invocation. `git show --stat HEAD` after each commit. |
| N3 | **release-plz `release-pr` step can fail with HTTP 403** even when `permissions: pull-requests: write` is in the workflow file. | P1 | Repo's `default_workflow_permissions` setting is a CEILING. Fix once: `gh api -X PUT repos/<owner>/<repo>/actions/permissions/workflow -f default_workflow_permissions=write -F can_approve_pull_request_reviews=true`. Already applied for `reubenjohn/reposix`. |
| N4 | **Bash hook denies inline `<crate-loop>` longer than 300 chars.** | P2 | Promote to a script in `scripts/` or split into multiple smaller calls. Don't try to squeeze under the threshold with trivial rewrites. |

Rows from the previous handover that are now RESOLVED: rows 1, 3, 4, 6, 9 fully verified; rows 2, 5, 7, 8, 10-22 closed via POLISH2-* commits or scope-deferred to v0.12.0 with catalog refs. Row 11 (`Error::Other` 156→144 occurrences) is partial: sim.rs migrated, ~150 backend sites remain — explicitly deferred to v0.12.0 per STATE.md.

---

## 5. Spec sections — none open

Hero, capability matrix, exit-codes, and ADR-009 stability commitment all shipped. If §3b drags out, the only "spec" left is "complete v0.11.1 publish": 9 crates on crates.io, `v0.11.1` git tag pushed, GH release page populated, homebrew formula bumped. release-plz handles the mechanics; the human (you) handles the unblocking.

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

## 7. Task list (in order — do not skip ahead)

### 7-A. Unblock CI on main (30-60 min)
Fix the `delta_sync` test failures per §3a. One commit, push, verify `gh run list --branch main --limit 1` flips to green. **Nothing else proceeds while CI is red.**

### 7-B. Drive release-plz to all-9-crates published (60-90 min)
Per §3b: `cargo publish --dry-run -p <crate>` for each of the 7 still-unpublished crates to surface prep errors; fix the underlying `Cargo.toml` metadata gaps; merge PR #20; watch release-plz pick it up; verify `https://crates.io/crates/reposix-cli` resolves.

### 7-C. Resolve dependabot PRs #16 + #17 (30 min)
Per §3c. The fixer subagent (id `a825d495b91ff734b`) may already have done the work — check `gh pr view 16` / `gh pr view 17` for new commits and CI status. If green, merge. If red, dispatch a fresh fixer subagent to a worktree (per N1) — do NOT `gh pr checkout` from a bash subagent.

### 7-D. Tag v0.11.1 + verify release artifacts (15 min)
Once §7-A through §7-C are done: release-plz creates the tag automatically on PR #20 merge. Verify:
- `https://github.com/reubenjohn/reposix/releases/tag/v0.11.1` exists with binaries for all 4 platforms (linux-x86_64-musl, linux-aarch64-musl, macOS x86 + arm64, windows-msvc).
- `brew install reubenjohn/reposix/reposix` resolves to v0.11.1.
- `cargo binstall reposix-cli` resolves to v0.11.1.

### 7-E. Trigger fresh bench-latency-cron run (10 min)
JIRA cells went LIVE last session via PR #19. Trigger one more run after v0.11.1 tag to confirm the latency table on the v0.11.1 release reflects current numbers: `gh workflow run bench-latency-cron.yml -R reubenjohn/reposix`.

### 7-F. Catalog + STATE.md final pass (15 min)
- `python3 scripts/catalog.py coverage` — must exit 0.
- `python3 scripts/catalog.py render` — refresh the MD view.
- Update `.planning/STATE.md` cursor to reflect v0.11.1 fully shipped.
- Commit, push, watch CI.

### 7-G. Delete this HANDOVER.md (1 min)
If §7-A..F are all done and main is green: `git rm HANDOVER.md && git commit -m "docs: remove HANDOVER.md (v0.11.1 fully shipped)" && git push`.

If anything is still pending: shrink HANDOVER.md to ONLY the open items. Don't append "I did X".

---

## 8. What to NOT touch

- `.claude/skills/` — owner approval required.
- `mkdocs-material` JS / `extra_javascript` / `mkdocs.yml` — three stacked bugs were fixed in commits `66836f7`, `e119006`, `100ae00`. Don't change `fence_div_format`, don't re-enable `minify_html: true`, don't remove the mermaid CDN load.
- The `refs/reposix/origin/main` checkout step in `docs/tutorials/first-run.md` step 4 — that non-standard refspec is load-bearing.
- v0.11.0 phase dirs in `.planning/phases/` — there should be none. Don't accidentally re-create them.
- Banned word `replace` (per `scripts/banned-words-lint.sh` and `.banned-words.toml`). Use `complement` / `alongside` / `for the 80%` / `migrate to`.
- **`gh pr checkout` from a bash subagent.** Per N1. Use a worktree.

## 9. Owner preferences (durable)

- **No walkthrough / morning-brief / session-recap docs.** This HANDOVER.md is operational; delete after use.
- **Subagent delegation aggressive.** Coordinator should not type code a subagent could type.
- **Push frequently.** Pre-push hook gates fmt + clippy + check-docs-site + banned-words.
- **One cargo invocation at a time.** RAM budget per CLAUDE.md.
- **No skills changes without explicit owner approval.**
- **Owner is reubenjohn (`reubenvjohn@gmail.com`).** gh CLI is authenticated. Repo secrets settable via `gh secret set`.

---

## 10. Status snapshot at handoff

- Branch: `main`
- HEAD: `12c2ec1` (`fix(cargo): really add version on workspace.dependencies reposix-core`)
- Tag latest: `v0.11.0` → `9d585bb` (force-updated). `v0.11.1` not yet cut (pending §7-B).
- Working tree: clean.
- CI on HEAD: **RED** — `delta_sync` test failures (run `24976684353`). See §3a.
- release-plz: **FAILED** on `reposix-sim` publish. PR #20 OPEN. See §3b.
- Open PRs: #20 (release-plz v0.11.1), #16 (axum), #17 (rand).
- Recently merged: #18 (rusqlite — likely cause of §3a), #19 (bench-cron auto-PR).
- Catalog: 0 TODO rows (coverage clean per last `set` invocation).
- Owner-action items: NONE (all 3 from previous handover resolved — crates.io email verified, JIRA secrets provisioned, skill ref path repointed).

---

*End of handover. Good luck.*
