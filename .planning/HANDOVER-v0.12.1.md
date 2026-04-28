# v0.12.1 HANDOVER — entry point for the next session

> **Eventually delete this file.** It's a session-bridge artifact. The
> next session reads it cold, executes the queued work, and the file
> deletes itself when the queued items all close (last phase commit
> includes `git rm .planning/HANDOVER-v0.12.1.md`).

**Created:** 2026-04-28T07:53Z by overnight orchestrator.
**Deadline:** 2026-04-28 17:00 PT (next business day close-of-day).
**Owner:** reuben.

---

## Autonomous-mode session log (2026-04-28 PT, in flight)

Owner kicked off autonomous mode mid-day. Closures since then:

| W item | Phase | Status | Commit |
|---|---|---|---|
| W2-followup | P67 redirect cleanup (mkdocs-redirects plugin) | DONE | `527c4d0` |
| W3 P67 | Extractor + grader prompt: transport-vs-feature heuristic | DONE | `5ac7c41` |
| W6 P70 | Pre-push self-test FAIL-path coverage | DONE | `f23f935` |
| W7a P71 | Row schema -- tests Vec / per-element drift / migration | DONE | `d2127c3` + `8f7762b` (fmt + verifier 2.0 fix) |
| W7b P71 | bind --test repeatable | DONE | `f4962b3` |
| W7c P71 | Catalog README v2 + CLAUDE.md pointer + HANDOVER | DONE | `1d72329` |
| W4 P68 | next_action enum + status breakdown + 388-row backfill | DONE | `948069b` |
| W5 P69 | confirm-retire --i-am-human bypass with audit trail | DONE | `b3c0102` |
| -- | Remove obsolete v0.12.1 retire-confirm one-shots | DONE | `4746674` |

**Cluster sweeps** (alignment-lift work):

| Sweep | What | Commit |
|---|---|---|
| Round-1 | 22 high-confidence rebinds (BackendConnector trait, push/conflict, write paths, audit, allowlist, real-git-tree) | `53c3f3e` |
| Drift fix + 3 FUSE retire-proposes | Heal 4 STALE_DOCS_DRIFT after round-1 + propose-retire smoking-gun confluence rows | `ada8231` |
| 10 STALE_DOCS_DRIFT refresh | Mechanical: re-bind trust-model + audit rows with current source content | `bb6b0c5` |
| Scan B | 7 connector + 1 jira phase-28 retire | `cfaacac` |
| Scan C | 10 ADR/v0.11 rebinds + 12 v0.8 FUSE retires | `aad1169` |
| Scan E | 4 index/contributing/concepts | `93392cb` |
| Scan G | 5 tutorial/roadmap/git-version → dark_factory | `1ff1d9e` |
| Schema ext | `--test` accepts shell/Python verifier paths (file sha256) | `090193d` |
| Scan H | 20 perf/clippy rows via shell verifier mode (first sweep with new schema) | `faffd33` |
| Scan I | 25 mirror/tutorial/lint rows (aliasing accepted) | `48bf274` |
| Scan J | 9 install/badge/exit/lint final easy-rebinds | `854edc1` |

**P72 doc rewrite**: `docs/reference/confluence.md` FUSE-era section rewritten for v0.9.0 git-native shape (`b6f6dd7`).

**Catalog state shift this session** (vs handover snapshot at top of file):

```
                    Start (handover)  Now (854edc1)
claims_bound        171               284   (+113)
claims_missing_test 166                52   (-114)
claims_retire_proposed  41 -> 0       16    (a) 41 -> 0 cleared via owner TTY before this session resumed; (b) 0 -> 16 from autonomous propose-retire of FUSE-era rows
claims_retired      0 -> 30           30
alignment_ratio     0.4407            0.7933   (+0.35)
```

**Remaining ~52 MISSING_TEST rows** are mostly:
- Historical roadmap entries (`v0-1-0-shipped`, `v0-2-0-alpha-shipped`, etc.) — should retire as documented historical record (cluster phase work).
- Lint/build invariants without 1:1 verifier (`forbid-unsafe-per-crate`, `rust-1-82-requirement`, `errors-doc-section-required`) — need bespoke grep/AST verifiers.
- Subjective rubrics (`cold-reader-16page-audit`, `playwright-screenshots-deferred`) — handled by `reposix-quality-review` skill, not docs-alignment.
- Narrative numbers (`use-case-20-percent-rest-mcp`, `mcp-fixture-synthesized`) — qualitative claims; no test possible.
- Doc-existence claims (`docs01-value-prop-10sec`, `docs02-three-page-howitworks`) — structure freshness covers loosely; need section-presence verifiers.
- Twitter/LinkedIn social `92%` token-reduction copy — known DOC_DRIFT vs measured 89.1% (see CLAUDE.md P65 punch list); needs prose fix not test bind.

**16 RETIRE_PROPOSED rows pending owner TTY confirm** (FUSE smoking gun + ADR-003 + v0.8 mount-shape rows + jira phase-28 + ADR-008 audit etc.). Run from a real terminal:
```
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
```
OR, from a Claude Code session, append `--i-am-human` (audit-trailed as `confirm-retire-i-am-human`).

**Confirming all 16 retires drops the denominator to 342, lifting ratio to 284/342 = 0.8304** — within striking distance of the v0.12.1 0.85 target.

**Known issue captured** (not blocking): the bind verb appends source citations (Source::Multi) and overwrites source_hash with the new range's hash, but the walker reads the FIRST source citation. This caused false STALE_DOCS_DRIFT after the round-1 rebind sweep. Mitigation in this session: re-bind with the wider existing range. v0.13.0: fix bind to either preserve first-source semantics OR have walker hash all sources.

**Walker still BLOCKs pre-push** because per-row MISSING_TEST/RETIRE_PROPOSED rows each emit a blocker line. Push goes GREEN only when ALL blocking rows clear (per HANDOVER §3 original design). Best path to unblock: continue cluster phases for the residual 52 MISSING_TEST + retire-confirm the 16 RETIRE_PROPOSED, OR relax walker to BLOCK only when alignment_ratio < floor (W9 / v0.13.0 work).

**Push status:** local ahead of origin by ~25 commits (pre-push hook BLOCKs on docs-alignment/walk; commits stay local). Pre-push gate consistently 21 PASS / 1 FAIL / 3 WAIVED across every push attempt — only docs-alignment/walk fails, every other gate (clippy, fmt, mkdocs-strict, banned-words, structural invariants) PASSES.

---

## TL;DR for the next session

1. **v0.12.0 is held at the tag boundary.** Owner decides G1 (workspace `Cargo.toml` 0.11.3 vs tag-gate Guard 3 expects 0.12.0). See `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` § Gap-block G1.
2. **v0.12.1 is in flight.** P66 (coverage_ratio metric) shipped this session. P67-P71 placeholders renumbered. P72+ cluster phases TBD per scoping below.
3. **Pre-push correctly BLOCKs** on misaligned rows (since hook fix `fdb4d24`). You can't push until cluster-closure phases lift `alignment_ratio` ≥ 0.50 AND clear blocking row states.
4. **Local is ahead of origin by 9 commits.** Owner should `git pull --rebase` BEFORE pushing — the previous attempted push failed with stale-base ref-lock (origin moved during this session via release-plz or similar).

## Live state snapshot

```
HEAD = db7366d  (P66 SHIPPED)
local ahead of origin by 9 commits

quality/catalogs/doc-alignment.json:
  claims_total          388
  claims_bound          171
  claims_missing_test   166  ← BLOCKING walker
  claims_retire_proposed 41  ← BLOCKING walker; 24 glossary + 17 audit candidates + 1 corrected
  claims_retired          0  ← becomes 24 after glossary bulk-confirm
  alignment_ratio       0.4407  ← BLOCKING (floor 0.5000)
  coverage_ratio        0.2055  ← PASS  (floor 0.1000)

pre-push:    21 PASS, 1 FAIL, 3 WAIVED  →  exit=1  (push blocks)
```

---

## Queued work, in dependency order

### Immediate (≤30 min, no fresh subagent dispatch needed)

#### W1 — Bulk-confirm 24 glossary retirements

Owner runs from a real TTY:

```bash
bash scripts/v0.12.1-confirm-glossary-retires.sh
```

Script lives at that path; reads the 24 row IDs via `jq` from the live catalog. Loops `confirm-retire`. Uses `--row-id` per call. After: commit the catalog mutation. Expected effect: `claims_retire_proposed` 41 → 17, `claims_retired` 0 → 24, `alignment_ratio` slightly bumps (denominator drops by 24).

#### W2 — Apply 17-row audit recommendations from RETIRE-AUDIT.md

A subagent (Opus, Path A) is reviewing the 17 non-glossary RETIRE_PROPOSED rows in background as of 2026-04-28T07:50Z. Output lands at:

```
quality/reports/doc-alignment/backfill-20260428T085523Z/RETIRE-AUDIT.md
```

The doc will contain per-row recommendations (CONFIRM_RETIRE / FLIP_TO_MISSING_TEST_IMPL_GAP / etc.) PLUS two ready-to-run scripts:
- `confirm-retire` script for legitimate retirements (owner runs from TTY).
- `mark-missing-test` script for over-retired rows (orchestrator runs from this session; not env-guarded).

Estimated 3-5 confirm-retires + 12-14 flips. Apply both scripts. Commit.

### Short (1-2h each; subagent-dispatchable)

#### W3 — P67: RETIRE audit + extractor prompt update

Currently a renumbered placeholder. Repurpose: this phase OWNS W2 (apply audit findings) plus updates `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` to teach the transport-vs-feature distinction:

> Retirement requires the FEATURE to be intentionally dropped with a documented decision. Transport / implementation-strategy changes do NOT retire claims about user-facing surface — those remain MISSING_TEST and become gap-closure work for the next implementation strategy.

Proposed phase scope: catalog correction (W2 above) + extractor prompt update + 1 regression test (re-run `plan-refresh` on a doc with FUSE-era prose; assert no new RETIRE_PROPOSED proposals).

**Status (2026-04-28):** extractor prompt + grader prompt updated with the new "Retirement vs implementation-gap" section + canonical examples drawn from commit `24b2b62` (audit flips). Smoke-test at `scripts/check-docs-extractor-prompt.sh` asserts the section header + `IMPL_GAP:` / `DOC_DRIFT:` rationale-prefix conventions stay present (cheap revert-guard). **TODO (deferred):** the proper regression test — re-run `plan-refresh` on a doc with FUSE-era prose and assert no new `RETIRE_PROPOSED` proposals — requires subagent dispatch (Task tool); `plan-refresh` itself is a read-only manifest emitter and does not invoke an extractor in-process. Defer until W4 (`next_action`) lands; structured field makes the assertion mechanical (count rows where `next_action == RETIRE_FEATURE` introduced by the run, expect 0).

#### W4 — P68: `next_action` field schema extension

Add `next_action: enum { WRITE_TEST, FIX_IMPL_THEN_BIND, UPDATE_DOC, RETIRE_FEATURE, BIND_GREEN }` to `Row` struct. Default `WRITE_TEST` (back-compat for existing populated rows). Update extractor prompt to set the field appropriately. Update `status` and `--json` to display. One-time backfill script walks existing 388 rows + reassigns `next_action` heuristically (RETIRE_PROPOSED → RETIRE_FEATURE; rationale prefix `IMPL_GAP:` → FIX_IMPL_THEN_BIND; default → WRITE_TEST).

#### W5 — P69: `confirm-retire --i-am-human` flag

Owner explicitly authorizes retirement via flag (audit-trailed in catalog row's `last_extracted_by` field). Lets human authorize from a Claude Code session without leaving for a fresh terminal. Small Rust change + test asserting the flag's audit trail is preserved.

#### W6 — P70: hook self-test extension

Extend `scripts/test-pre-push.sh` (currently only verifies PASS-path) to also force a runner FAIL and assert the hook propagates exit non-zero. The recent `fdb4d24` hook fix was invisible to the existing test because the test asserts behavior on the PASSING side only. Companion: audit `~/.git-hooks/pre-push` (personal global) for the same `if ! cmd; exit $?` pattern.

#### W7 — P71: Schema cross-cut consolidation + many-to-many bindings (MIGRATE-03 (i)) — **PRIORITY: land BEFORE P72 cluster phases**

**Why elevated:** cluster-closure phases (P72-P80) will produce rows that bind a single claim to multiple tests (e.g. a JIRA-writes claim binding to create + update + delete + conflict-recovery tests). Forcing one row per test multiplies the catalog without semantic gain. The schema generalization must land before cluster work begins.

Cross-cuts to fix in this phase:

1. `Row.test: String` → `Row.test: Vec<String>`. Empty vec = no test (currently `Option<String>` semantics). Single-element vec is the common case. Multi-element supports many-tests-per-claim.

2. `Row.test_body_hash: Option<String>` → `Row.test_body_hashes: Vec<String>`. **Parallel to `Row.test`.** This is the per-function-hash improvement: when the walker detects drift, it isolates which specific test fn changed rather than re-grading every binding.
   - Walker drift detection becomes per-element comparison.
   - `STALE_TEST_DRIFT` carries an index or list of which test(s) drifted in the row's diagnostic.
   - On `bind`, the binary computes hashes for each test in the input list.

3. `bind` writes `Row.source` as `SourceCite` object; `merge-shards` reads `Source` enum (Single|Multi). Reconciled mid-flight via jq during P65; should be unified at the type level. (Source is already multi-capable conceptually; just needs writer-side consistency.)

4. (NEW from this session) `Row.rationale` is `Option<String>` in the writer side but the walker's deserialize was failing on missing rationale. Verify whether this is a serde back-compat issue or a writer bug. Test required: walker round-trips a catalog with rows lacking `rationale`.

5. (NEW) `FloorWaiver` struct expects fields `{until, rationale}` but the design brief had `{until, reason, dimension_owner}`. Pick one consistent shape; update either the brief, the schema spec, or the struct. (Pre-existing fix: the floor_waiver block in `doc-alignment.json` was reconciled to the struct shape during this session and is now removed entirely along with the walker waiver per owner directive.)

6. **Migration script** for the existing 388-row catalog. `Row.test: String` → `Row.test: vec![string]`. `Row.test_body_hash: Option<String>` → `Row.test_body_hashes: vec![hash]` (or empty vec if hash absent). One-shot Python script that reads the catalog, transforms in place, validates against the new schema. Commit the migrated catalog as part of W7.

CLI updates:
- `bind` accepts `--test <file::fn>` repeatably, OR `--tests <file::fn,file::fn,...>` (pick one ergonomic; recommend repeatable for shell-quoting safety).
- `verify --row-id X` displays per-test hashes + drift status per binding.
- `status` shows count of multi-test rows for visibility.

**Status (W7 closed):**

- W7a SHIPPED at `d2127c3` (Row schema vectors + walker per-element drift + 388-row catalog migration) + `8f7762b` (cargo fmt + structural verifier accepts schema_version `"2.0"`). Catalog `schema_version` is now `"2.0"`; parallel-array invariant `tests.len() == test_body_hashes.len()` enforced via `Row::set_tests`.
- W7b in flight (CLI surface — repeatable `--test` on `bind`).
- W7c shipping this commit (docs: `quality/catalogs/README.md` row spec v2 + CLAUDE.md P64 pointer to schema bump).
- Cross-cut §3 (SourceCite vs Source enum unification) verdict: **no-op**. Walker round-trips today's catalog cleanly; the v0.12.0 P65 jq-transform reconciliation was a one-shot during the backfill merge, not a recurring drift. If a regression surfaces in P72+ shard merges, file under W11 (subagent default-catalog refusal) territory.
- Cross-cut §4 (rationale `Option` round-trip): **real bug, fixed in W7a** via `serde(default)` on `tests`, `test_body_hashes`, and `rationale`. Catalogs that omit any of the three deserialize cleanly.
- Cross-cut §5 (FloorWaiver shape): **no-op**. The `floor_waiver` block was removed from `doc-alignment.json` per owner directive earlier this session; the struct/brief mismatch is moot.

#### W8 — P72+: Cluster-closure phases per PUNCH-LIST.md

The 14 clusters identified in `quality/reports/doc-alignment/backfill-20260428T085523Z/PUNCH-LIST.md` need closure. After W2 (audit corrections), some MISSING_TEST counts shift. Re-read PUNCH-LIST.md (or regenerate via `python3 scripts/gen_punch_list.py quality/reports/doc-alignment/backfill-20260428T085523Z/`) before scoping P72+ phases.

Likely cluster ordering by leverage:

- **P72 — Confluence backend parity (smoking gun).** ~15 rows including `docs/reference/confluence.md` FUSE-era stale section + ADR-002/003 nested-shape promises (after W2 flip). Two paths per row: fix impl OR update doc. Cluster phase scope is the resolution.
- **P73 — JIRA shape.** ~10 rows. ADR-005, `docs/reference/jira.md` Phase 28 read-only stale (W2 confirms retire), `parent` symlink claim if any.
- **P74 — Benchmark numbers.** 20 MISSING_TEST. Either Rust-port `quality/gates/perf/{latency-bench.sh, bench_token_economy.py}` so each row binds to `<file>::<fn>`, OR extend the binary to accept Python verifier paths. Includes drift fixes: `~92%` (social) → measured 89.1%; `24 ms cold init` (mental-model) → measured 27 ms.
- **P75 — Connector authoring guide.** 24 MISSING_TEST. Trait method contracts asserted in code without named test fns. Add `#[test] fn backendconnector_supports_required_methods()` style harnesses.
- **P76 — Tutorial first-run.** 6 MISSING_TEST steps 4-8 (checkout, edit, push, audit). Integration test extending `dark_factory_sim_happy_path`.
- **P77 — Developer workflow + invariants.** 17 rows. Policy invariants (`#![forbid(unsafe_code)]`, MSRV, cargo test count) needing bespoke verifiers. Some collapse into a single shell-grep verifier under `quality/gates/code/`.
- **P78 — Concepts (mental model).** 13 MISSING_TEST. Includes the 24ms vs 27ms drift.
- **P79 — Internals + research notes.** Mostly already BOUND; small cleanup.
- **P80 — Coverage chunker redirect-following.** New finding from P66: `docs/connectors/guide.md`, `docs/security.md`, `docs/why.md`, `docs/reference/crates.md` show 0 rows in coverage even though shards extracted from their redirect targets. Either the chunker should follow redirects, OR rows should track the prose-source file rather than the canonical-redirect file. Pick one.

After P72-P80 land, alignment_ratio + coverage_ratio both lift. Re-dispatch the milestone-close verifier; v0.12.1 ships.

### Floor-ratchet plan (owner directive, captured 2026-04-28)

Two floors evolve differently because they measure different things:

| Floor | v0.12.0 | v0.12.1 target | v0.13.0 target | Asymptote |
|---|---:|---:|---:|---:|
| `alignment_floor` | 0.50 | 0.85 | 0.95 | 0.99 |
| `coverage_floor`  | 0.10 | 0.25 | 0.40 | 0.40-0.60 |

**alignment_floor → 100% is the right target.** Every behavioral claim should have a test. The 1% asymptote covers genuinely-unbinder claims (subjective rubrics that grade rather than bind, manual gates with TTL freshness). Each cluster phase commits a floor ratchet in its closing commit: `summary.floor: 0.5000 → 0.5800` after P72 closes ~30 rows, etc.

**coverage_floor → 100% is wrong.** Not every doc line is a behavioral claim — preambles, narrative, examples, attribution don't bind to tests. Realistic asymptote 0.40-0.60. Anything above suggests over-mining (the v0.12.1 P65 backfill already showed the 24-glossary over-extraction failure mode). The chunker should learn to skip (W12 below) before the floor ratchets aggressively.

Rule: only a deliberate human commit ratchets either floor up. The walker NEVER auto-ratchets. A regression below floor BLOCKs pre-push.

### What "retire" means + per-row cleanup actions

Retirement removes a row from the `claims_total - claims_retired` denominator. Whether the SOURCE FILE/LINE also gets cleaned up depends on what it is — owner clarified 2026-04-28:

| Source kind | After retire | Why |
|---|---|---|
| Redirect-only stub doc (`docs/architecture.md`, `docs/demo.md`) | **DELETE the stub file** + add to `mkdocs.yml` `plugins.redirects.redirect_maps` | The stub IS the redirect mechanism today; the proper mechanism is mkdocs `redirects` plugin (HTTP-level redirects). Stubs are technical debt — clean them up. |
| Archived `REQUIREMENTS.md` line (e.g. `HARD-04`, `SWARM-01`, `helper-sim-backend-tech-debt-closed`) | **KEEP line** | Historical record of what the closed milestone promised. Deletion rewrites history. |
| Catalog row | **KEEP** (`last_verdict: RETIRE_CONFIRMED`) | Audit trail of the retirement decision; inert in alignment math. |

For the 6 audit-recommended retires (W2 closure):

1. `docs/architecture/redirect` → `confirm-retire` THEN delete `docs/architecture.md` + update `mkdocs.yml`.
2. `docs/demo/redirect` → `confirm-retire` THEN delete `docs/demo.md` + update `mkdocs.yml`.
3. `helper-sim-backend-tech-debt-closed` → `confirm-retire` only (line stays).
4. `hard-04` → `confirm-retire` only.
5. `swarm-01` → `confirm-retire` only.
6. `swarm-02` → `confirm-retire` only.

For the 24 glossary RETIRE_CONFIRMED rows (already done): the `docs/reference/glossary.md` file STAYS — glossary is a useful reference page; just file-level-excluded from chunker via W12 (`.docalignignore` or frontmatter directive).

`confirm-retire` is the only verb that flips a row to RETIRE_CONFIRMED. It is env-guarded (`$CLAUDE_AGENT_CONTEXT`) AND tty-guarded (`isatty(stdin)`), so only humans from a real terminal can confirm. The path of least resistance for an agent CANNOT be "delete the claim to make CI green."

### W2-followup — redirect cleanup (NEW, owner clarified)

After `bash scripts/v0.12.1-confirm-audit-retires.sh` succeeds, the redirects need physical cleanup:

```bash
# 1. Add the redirects plugin to mkdocs.yml under `plugins:`:
#    - redirects:
#        redirect_maps:
#          architecture.md: how-it-works/git-layer.md
#          demo.md: tutorials/first-run.md
#
# 2. Delete the stub files:
git rm docs/architecture.md docs/demo.md
#
# 3. Verify the redirect plugin is installed:
pip show mkdocs-redirects || pip install mkdocs-redirects
#
# 4. Build + verify:
mkdocs build --strict
#
# 5. Spot-check that the previous URLs still redirect (open the site
#    locally with `mkdocs serve` and visit /architecture/ + /demo/).
#
# 6. Commit:
git add mkdocs.yml docs/architecture.md docs/demo.md
git commit -m "docs(p67): replace stub redirects with mkdocs-redirects plugin"
```

Note: `mkdocs-redirects` may already be in the project's `requirements*.txt` for docs builds. Check before installing.

### W12 — File-level chunker exclusion (NEW)

`plan-backfill`'s input set should support file-level exclusion via either:
- `.docalignignore` at repo root (gitignore-style globs)
- frontmatter directive `--- docalignment: skip ---`

Today the chunker mines `docs/reference/glossary.md` and produces 24 definitional rows that all proposed retirement on first pass. File-level exclusion prevents recurrence on next backfill. Exclude glossary, redirect stubs, social copy (or include with explicit human curation).

### Long-tail

#### W9 — Walker BLOCK granularity (per-priority gating)

Currently the walker BLOCKs on ANY MISSING_TEST/RETIRE_PROPOSED row. Once `next_action` lands (W4), the walker could be smarter: e.g. exit non-zero ONLY on rows where the `next_action` is overdue (e.g. `FIX_IMPL_THEN_BIND` aged > 90 days), and treat fresh rows as warnings (P2). This prevents v0.12.x+ milestones from inheriting all 166 rows as P0 blockers.

Scope as v0.13.0 work — too aggressive for v0.12.1 unless the cluster phases finish faster than expected.

#### W10 — Walker `last_walked` artifact promotion (MIGRATE-03 (h))

Walker writes `summary.last_walked` on every invocation, mutating the catalog file. This produces git churn on every pre-push. Either move to the artifact at `quality/reports/verifications/docs-alignment/walk.json` OR extend `catalog_dirty()` to ignore `summary.last_walked` drift.

#### W11 — Subagent default-catalog refusal (MIGRATE-03 (j))

Shard 016 in P65 wrote 17 rows to the LIVE catalog instead of its shard catalog because the agent forgot the `--catalog <shard-path>` flag. Binary should refuse to mutate the default-path catalog when invoked under a known subagent context (env-guard or required-flag pattern).

---

## Action classification debate (W4 above)

The owner asked: *"does adding `next_action` add too much burden on the agent classifying or does it encourage more reasoning and accuracy?"*

**My recommendation: include it.** Reasoning:

- **Default-WRITE_TEST means inattentive extractors produce correct output by accident.** A careful extractor produces structured signal. Net positive.
- **Consumer-side win is large.** Punch list + cluster-phase scoping become filterable. v0.12.1 can scope phases as "close all FIX_IMPL_THEN_BIND" rather than reading 166 rationale fields by hand.
- **Existing failure mode the field would fix:** The `IMPL_GAP:` rationale-prefix convention this session is informal and grep-only. Structured field formalizes it.
- **Cost is one prompt instruction + one catalog field.** Trivial schema change.

The field is on the *Row*, not the verdict. Verdict (`last_verdict`) stays orthogonal — same row can be MISSING_TEST + WRITE_TEST (test missing, just write it) OR MISSING_TEST + FIX_IMPL_THEN_BIND (test missing because impl regressed; fix both). Both axes are needed.

---

## Time budget

| Block | Estimate | Cumulative |
|---|---|---|
| W1 (glossary bulk-confirm) | 5 min | 5 min |
| W2 (apply audit) | 15 min | 20 min |
| W3 P67 (audit + extractor prompt) | 1h | 1h20min |
| W4 P68 (next_action field) | 1.5h | 2h50min |
| W5 P69 (--i-am-human flag) | 30 min | 3h20min |
| W6 P70 (hook self-test) | 30 min | 3h50min |
| W7 P71 (schema cross-cut) | 1h | 4h50min |
| W8 P72-P80 (cluster phases) | 6-8h | 10h50min - 12h50min |
| Milestone-close verifier | 30 min | + |

5pm deadline = ~9h from session start. Realistic: W1-W7 + 2-3 cluster phases ship by 5pm. Remaining clusters become v0.12.1.x or are rolled into a follow-on session.

**Recommended phase order for next session:** W1, W2 (consume the audit), W3, W6 (small + unblocks future hook regressions). Then W4 (structured field) BEFORE the cluster phases, because each cluster phase benefits from the structured `next_action`.

---

## v0.12.0 G1 reminder

**The orchestrator does NOT push the v0.12.0 tag.** Status remains
`ready-to-tag-pending-owner-decision` until owner picks Path A or B
(see `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` § Gap-block
G1). v0.12.1 work proceeds in parallel; the tag-cut waits.

---

## Cleanup criterion

This file deletes itself when:
- W1 and W2 closed (catalog at expected ratios for v0.12.1).
- P67-P71 (W3-W7) shipped GREEN at per-phase verifier.
- A v0.12.1-specific HANDOVER (or none — STATE.md alone) replaces it.

The phase that ships W7 (or whichever closes the last item above) includes `git rm .planning/HANDOVER-v0.12.1.md` in its closing commit.
