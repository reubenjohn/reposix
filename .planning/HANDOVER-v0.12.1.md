# v0.12.1 HANDOVER — what to do when you return

> Lean handover. Detailed session log archived at
> `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md`.

**Created:** 2026-04-29 PT (autonomous-mode session close).
**Owner:** reuben.
**Eventually delete this file** — last commit closing the queue removes it.

---

## Catalog state right now

```
claims_total           388
claims_bound           313   (was 171 at session start; +142)
claims_missing_test    22    (was 166; -144)
claims_retire_proposed 23    pending owner TTY confirm
claims_retired         30
alignment_ratio        0.8743   ← CLEARS v0.12.1 0.85 target
                       0.9343   PROJECTED after confirming 23 retires
```

**Push status:** local ahead of origin by ~42 commits. Pre-push consistently `21 PASS / 1 FAIL / 3 WAIVED` — only `docs-alignment/walk` fails (per-row blocker design from original handover §3). Every other gate green.

**G1 (v0.12.0 tag boundary): RESOLVED.** Workspace `Cargo.toml` bumped 0.11.3 → 0.12.0 to match shipped CHANGELOG. The new `active-milestone-matches-workspace-version.sh` verifier now PASSES. You can run `bash .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` whenever ready.

---

## What you should do (in order)

### 1. Confirm 23 retire proposals (one TTY run, ~15 seconds)

Start from a fresh terminal so `$CLAUDE_AGENT_CONTEXT` is unset:

```bash
cd /home/reuben/workspace/reposix
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
git add quality/catalogs/doc-alignment.json
git commit -m "docs(p67): bulk-confirm 23 retirements"
git push
```

Per-row context: **`quality/reports/doc-alignment/RETIRE-PROPOSED-WALKTHROUGH.md`** (note: that doc was written when 37 rows were proposed; 14 have since flipped back to BOUND — Group 3 (3 rows) and Group 4 (10 rows) and Group 9 (1 row) — see "What's NEW" below).

The 23 remaining are Groups 1, 2, 5, 6, 7, 8 — all uncontroversial:
- 14 FUSE-pivot supersession (Group 1)
- 4 v0.9.0-superseded benchmarks (Group 2)
- 2 subjective rubrics → handled by `reposix-quality-review` (Group 5)
- 1 deferral status (`playwright-screenshots-deferred`, Group 6)
- 1 Phase-28→29 supersession (jira phase-28 read-only, Group 7)
- 1 outdated forward-looking ("v0.4 will add write path", Group 8)

### 2. Tag v0.12.0 (G1 unblocked)

```bash
bash .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh
```

(After this, bump Cargo.toml to 0.12.1 in a follow-up commit if you want active dev to track v0.12.1 next-release. Optional — current 0.12.0 is fine for "shipping mode".)

### 3. Residual 22 MISSING_TEST rows — what I need from you

I can't auto-close these without your judgment on each. Decisions I need:

#### 3a. Lint-config invariants (9 rows) — write bespoke verifiers? `[Y/N]`
- `forbid-unsafe-code`, `forbid-unsafe-per-crate`, `rust-1-82-requirement`, `rust-stable-no-nightly`, `errors-doc-section-required`, `tests-green`, `cargo-check-workspace-available`, `cargo-test-133-tests`, `demo-script-exists`
- Verifiers needed: workspace-walker that asserts `#![forbid(unsafe_code)]` on every `lib.rs`, grep for `rust-version = "1.82"` in workspace Cargo.toml, etc. Each verifier is ~15-30 min.
- **If yes**: I write them; rows go BOUND. ~3-4 hours total work.
- **If no**: leave MISSING_TEST as backlog; pre-push walker keeps blocking on them.

#### 3b. Connector contract gaps (4 rows) — backlog or address now? `[backlog/now]`
- `auth-header-exact-test`, `real-backend-smoke-fixture`, `attachments-comments-excluded`, `jira-real-adapter-not-implemented`
- These need NEW Rust tests (wiremock fns or `cargo test --ignored` coverage). Each is ~30-60 min real test-writing work.
- **If now**: I write them; rows go BOUND.
- **If backlog**: surface as v0.13.0 phase per HANDOVER P73 (JIRA shape) / P75 (connector authoring guide).

#### 3c. Narrative numbers (4 rows) — retire as untestable? `[Y/N]`
- `use-case-20-percent-rest-mcp`, `use-case-80-percent-routine-ops`, `mcp-fixture-synthesized-not-live`, `mcp-schema-discovery-100k-tokens`
- These are qualitative design framing (e.g. "20% of agent ops use REST/MCP"). No test could falsify them.
- **If yes**: propose-retire with rationale "qualitative design framing; no behavioral assertion possible" (you'd confirm in step 1's pattern).
- **If no**: tell me what you want them to bind to; otherwise I leave MISSING_TEST.

#### 3d. docs/index UX claims (4 rows) — handle now? `[per-row guidance]`
- `5-line-install` — "the install path fits in 5 lines on the landing page". Could bind to subjective-rubrics catalog (install-positioning rubric) instead of docs-alignment.
- `audit-trail-git-log` — "git log IS the audit trail" claim. Could bind to a verifier that asserts `git log` shows audit-relevant commits, OR mark qualitative.
- `tested-three-backends` — "three real backends are tested". Could bind to `quality/gates/agent-ux/` if a multi-backend smoke test exists, or write one.
- `spaces-01` — `reposix spaces` subcommand. Owner-decision territory: does the subcommand still exist? If yes, bind to a CLI smoke test. If no, retire.

#### 3e. LinkedIn FUSE drift (1 row): fix prose now? `[Y/N]`
- `docs/social/linkedin.md:21` still says "FUSE filesystem" framing — v0.9.0-era doc-drift class.
- **If yes**: I update prose to drop "FUSE" reference (one-line edit), rebind to bench script.
- **If no**: leave MISSING_TEST as backlog.

### 4. Known issue captured for v0.13.0

The bind verb appends source citations (`Source::Multi`) AND overwrites `source_hash` with the new range's hash, but the walker reads the FIRST source citation. Caused false `STALE_DOCS_DRIFT` after every cluster sweep this session (mitigated inline by re-binding with the wider existing range). Fix: either preserve first-source semantics in `bind`, OR have walker hash all sources. ~30 min Rust work in `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (line 295 area).

---

## What's NEW in this session (one-line each)

- **W3 P67** — extractor + grader prompts learn the transport-vs-feature heuristic.
- **W4 P68** — `next_action` enum on every row; status breakdown.
- **W5 P69** — `confirm-retire --i-am-human` flag (audit-trailed).
- **W6 P70** — pre-push self-test FAIL-path coverage.
- **W7 P71** — `tests` Vec / per-element drift / 388-row schema 2.0 migration / `bind --test` repeatable.
- **Schema extension** — `--test` accepts shell/Python verifier paths (whole-file sha256).
- **9 cluster sweeps** — alignment lift via rebinds + retire-proposes.
- **P72 doc rewrite** — `docs/reference/confluence.md` FUSE-era section rewritten for v0.9.0 git-native shape.
- **3 new structure verifiers**:
  - `active-milestone-matches-workspace-version.sh` — catches CHANGELOG/Cargo version drift.
  - `required-doc-surfaces.sh` — asserts v0.10 Diataxis docs structure (14 paths + nav + theme).
  - `release-tags-present.sh` — asserts shipped milestones have git tags. **Currently flags v0.10.0 tag missing despite CHANGELOG entry — real release-tooling drift to close in v0.13.0.**
- **G1 closed** — Cargo workspace bumped 0.11.3 → 0.12.0.
- **Social 92% → 89.1% prose fix** + 2 social rows BOUND.

Per-commit detail in `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md`.

---

## Cleanup criterion

This file deletes itself when:
- Step 1 (23 retire-confirms) ships.
- Step 2 (v0.12.0 tag) ships.
- Step 3 decisions made (whichever path you choose).

The commit closing the last item includes `git rm .planning/HANDOVER-v0.12.1.md`.

---

## Optional reading

- `quality/reports/doc-alignment/RETIRE-PROPOSED-WALKTHROUGH.md` — per-row retire reasoning (note: 14 of the original 37 have been un-retired; doc still describes them as RETIRE_PROPOSED but they're now BOUND).
- `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md` — full session log.
- `quality/PROTOCOL.md` — runtime contract for the quality gates.
- CLAUDE.md § "v0.12.1 — in flight" — high-level mental model.
