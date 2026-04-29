# v0.12.1 HANDOVER — what to do when you return

> Lean replacement for the previous session-log handover (now archived
> at `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md`).
> This file has ONE job: tell you what to do next. The detailed
> autonomous-mode session log is in the archive if you need it.

**Created:** 2026-04-29 PT (autonomous-mode session close).
**Owner:** reuben.
**Eventually delete this file.** When the queued items below close,
the last commit removes it.

---

## Catalog state right now

```
claims_total           388
claims_bound           300   (was 171 at session start; +129)
claims_missing_test    22    (was 166; -144)
claims_retire_proposed 36    pending owner TTY confirm
claims_retired         30    confirmed earlier in the day
alignment_ratio        0.8380   (was 0.4407; +0.40)
                       0.9317   PROJECTED after confirming 36 retires
v0.12.1 target         0.85
```

**Push status:** local ahead of origin by ~38 commits. Pre-push
consistently `21 PASS / 1 FAIL / 3 WAIVED` — only `docs-alignment/walk`
fails (per-row blocker design from HANDOVER §3 of the original
handover). Every other gate green throughout the session.

---

## What you should do (in order)

### 1. Confirm 36 retire proposals (one TTY run, ~20 seconds)

Start from a fresh terminal so `$CLAUDE_AGENT_CONTEXT` is unset:

```bash
cd /home/reuben/workspace/reposix
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
```

Walkthrough doc with full per-row context:
**`quality/reports/doc-alignment/RETIRE-PROPOSED-WALKTHROUGH.md`**

Read it first if you want to spot-check. Groups 1, 2, 5, 7, 8 are uncontroversial. Group 3 (historical milestones), 4 (one-time structural events), 6 (deferral status) you previously asked me to review — see the doc's "Did I review them myself" reflection at the bottom of this handover.

After confirming, commit:
```bash
git add quality/catalogs/doc-alignment.json
git commit -m "docs(p67): bulk-confirm 36 retirements"
git push
```

Expected pre-push state after this: `alignment_ratio = 0.9317`, walker still BLOCKs on residual MISSING_TEST rows but the ratio gate clears decisively.

### 2. Decide on the v0.12.0 G1 issue (workspace version vs CHANGELOG)

`quality/gates/structure/active-milestone-matches-workspace-version.sh` (shipped this session) catches the drift:

```
CHANGELOG.md most recent shipped: v0.12.0
Cargo.toml [workspace.package] version: 0.11.3
```

This is the same v0.12.0 G1 issue from the original handover — held at the tag boundary because workspace `Cargo.toml` is 0.11.3 but the v0.12.0 tag-gate Guard 3 expects 0.12.0. You decide:

- **Path A**: bump Cargo.toml workspace version to 0.12.0 (or 0.12.1) and tag the release. Verifier passes.
- **Path B**: roll back the v0.12.0 CHANGELOG entry to `[Unreleased]` and re-tag whenever Cargo gets bumped. Verifier passes via the symmetric direction.

The verifier surfaces this drift on every push attempt as a useful reminder. It is NOT yet wired into the pre-push runner — that's a v0.13.0 task (add a row to `quality/catalogs/freshness-invariants.json`).

### 3. Residual 22 MISSING_TEST rows — backlog for v0.13.0+

These are the genuinely-hard-to-bind rows that survived all sweeps:

| Cluster | Count | Why hard |
|---|---:|---|
| Lint-config invariants (`forbid-unsafe-code`, `rust-1-82-requirement`, `tests-green`, `cargo-test-133-tests`, `errors-doc-section-required`, `forbid-unsafe-per-crate`, `rust-stable-no-nightly`, `cargo-check-workspace-available`, `demo-script-exists`) | 9 | Need bespoke workspace-walker scripts (e.g. assert `#![forbid(unsafe_code)]` per `lib.rs`) |
| Connector contract gaps (`auth-header-exact-test`, `real-backend-smoke-fixture`, `attachments-comments-excluded`, `jira-real-adapter-not-implemented`) | 4 | Need new wiremock test fns or `cargo test --ignored` coverage |
| Narrative numbers (`use-case-{20,80}-percent-*`, `mcp-fixture-synthesized`, `mcp-schema-discovery-100k-tokens`) | 4 | Qualitative design framing; no test possible |
| docs/index UX claims (`5-line-install`, `audit-trail-git-log`, `tested-three-backends`, `spaces-01`) | 4 | Need install-position rubric / audit-shape grep / real-backend smoke harness |
| LinkedIn FUSE-era prose drift | 1 | `docs/social/linkedin.md:21` still says "FUSE filesystem" — same v0.9.0 drift class as the 92%->89.1% I fixed; one-line prose update OR new structure-freshness check for Layer-3 social copy |

### 4. Known issue captured for v0.13.0

The bind verb appends source citations (`Source::Multi`) AND overwrites `source_hash` with the new range's hash, but the walker reads the FIRST source citation. This caused false `STALE_DOCS_DRIFT` after every cluster sweep this session (mitigated inline by re-binding with the wider existing range). Real fix: either preserve first-source semantics in `bind`, OR have walker hash all sources. ~30 min Rust work in `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (around line 295).

---

## What's NEW in this session (one-line each)

- **W3 P67** — extractor + grader prompts learn the transport-vs-feature heuristic.
- **W4 P68** — `next_action` enum on every row; status breakdown.
- **W5 P69** — `confirm-retire --i-am-human` flag (audit-trailed).
- **W6 P70** — pre-push self-test FAIL-path coverage.
- **W7 P71** — `tests` Vec / per-element drift / 388-row schema 2.0 migration / `bind --test` repeatable.
- **Schema extension** — `--test` accepts shell/Python verifier paths (whole-file sha256). Unlocked perf/badge/install-asset/lint clusters.
- **9 cluster sweeps** — round-1, scan-B/C/E/G/H/I/J + STALE_DOCS_DRIFT mass refresh + historical-retire pass + final batch.
- **P72 doc rewrite** — `docs/reference/confluence.md` FUSE-era section rewritten for v0.9.0 git-native shape.
- **`active-milestone-matches-workspace-version.sh`** — new structure verifier (this session, post-pushback).

Per-commit detail in the archived session log: `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md`.

---

## Cleanup criterion

This file deletes itself when:
- Step 1 (37 retire-confirms) ships.
- Step 2 (G1 decision) lands — Cargo bumped or CHANGELOG rolled back.
- A new milestone HANDOVER (or none — `.planning/STATE.md` alone) replaces it.

The phase that ships the last item above includes
`git rm .planning/HANDOVER-v0.12.1.md` in its closing commit.

---

## Optional reading (not blocking)

- `quality/reports/doc-alignment/RETIRE-PROPOSED-WALKTHROUGH.md` — full per-row reasoning for the 36 retires queued in step 1.
- `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md` — full autonomous-mode session log (commits, scan results, all rationale).
- `quality/PROTOCOL.md` — runtime contract for the quality gates (unchanged this session).
- CLAUDE.md § "v0.12.1 — in flight" — high-level mental model for the dimension.
