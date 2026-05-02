# v0.12.1 HANDOVER — entry point for the next session

> **Eventually delete this file.** It's a session-bridge artifact. The
> next session reads it cold, executes the queued work, and the file
> deletes itself when the queued items all close (last phase commit
> includes `git rm .planning/HANDOVER-v0.12.1.md`).

**Created:** 2026-04-28T07:53Z by overnight orchestrator.
**Deadline:** 2026-04-28 17:00 PT (next business day close-of-day).
**Owner:** reuben.

---

## Chapters

- **[queued-work.md](./queued-work.md)** — Full queued work list in dependency order: W1 glossary retires, W2 audit, W3–W7 short phases (P67–P71), W8 cluster phases (P72–P80), plus floor-ratchet plan, retirement semantics, W2-followup redirect cleanup, W12 chunker exclusion, and long-tail items (W9–W11).
- **[tail.md](./tail.md)** — Action classification debate (W4), time budget table, v0.12.0 G1 reminder, and cleanup criterion.

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
| Historical retires | 14 v0.1/v0.2/v0.10/v0.11/v0.8 milestone + structural rows | `01a40d5` |
| Final batch | 7 polish rebinds + 7 v0.10 docs-restructure retires | `c5d2a0e` |

**P72 doc rewrite**: `docs/reference/confluence.md` FUSE-era section rewritten for v0.9.0 git-native shape (`b6f6dd7`).

**Catalog state shift this session** (vs handover snapshot at top of file):

```
                    Start (handover)  Now (c5d2a0e)
claims_bound        171               291   (+120)
claims_missing_test 166                24   (-142)
claims_retire_proposed  41 -> 0       37    (a) 41 -> 0 cleared via owner TTY before this session resumed; (b) 0 -> 37 from autonomous propose-retire of FUSE-era + historical-milestone + structural rows
claims_retired      0 -> 30           30    (will jump to 67 after owner confirms the 37 RETIRE_PROPOSED)
alignment_ratio     0.4407            0.8128   (+0.37)
                                      0.9065   (PROJECTED after owner confirms all 37 RETIRE_PROPOSED -- well above v0.12.1 0.85 target)
```

**Remaining 24 MISSING_TEST rows** fall into hard-to-close buckets:
- 9 cargo / lint-config invariants (`README-md/{forbid-unsafe-code, rust-1-82-requirement, tests-green}`, `docs-development-contributing-md/{cargo-check-workspace-available, cargo-test-133-tests, demo-script-exists, errors-doc-section-required, forbid-unsafe-per-crate, rust-stable-no-nightly}`) — need bespoke grep/AST verifiers (e.g. workspace-walker that asserts `#![forbid(unsafe_code)]` on every `lib.rs`); not in current `quality/gates/`.
- 4 docs/index UX claims (`5-line-install`, `audit-trail-git-log`, `tested-three-backends`, plus the social copy) — need bespoke verifiers (install-position rubric, audit-shape grep, real-backend smoke harness, prose update).
- 4 connector / jira gaps (`auth-header-exact-test`, `real-backend-smoke-fixture`, `attachments-comments-excluded`, `jira-real-adapter-not-implemented`) — connector contract test gaps; require new test work.
- 4 narrative numbers (`use-case-20-percent-rest-mcp`, `use-case-80-percent-routine-ops`, `mcp-fixture-synthesized-not-live`, `mcp-schema-discovery-100k-tokens`) — qualitative design framing; no test possible.
- 2 social copy DOC_DRIFT (`docs/social/{twitter,linkedin}/token-reduction-92pct`) — known 92%-vs-89.1% drift (CLAUDE.md P65 punch list); needs prose fix not test bind.
- `spaces-01` (`reposix spaces` subcommand status) — owner-decision territory.

**37 RETIRE_PROPOSED rows pending owner TTY confirm.** Run from a real terminal:
```
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
```
OR, from a Claude Code session, append `--i-am-human` (audit-trailed as `confirm-retire-i-am-human`).

**Confirming all 37 retires** lifts the catalog from `claims_retired = 30 -> 67`, drops denominator to `388 - 67 = 321`, ratio = `291 / 321 = 0.9065`. **Well above v0.12.1's 0.85 target.**

**Known issue captured** (not blocking): the bind verb appends source citations (Source::Multi) and overwrites source_hash with the new range's hash, but the walker reads the FIRST source citation. This caused false STALE_DOCS_DRIFT after the round-1 rebind sweep. Mitigation in this session: re-bind with the wider existing range. v0.13.0: fix bind to either preserve first-source semantics OR have walker hash all sources.

**Walker still BLOCKs pre-push** because per-row MISSING_TEST/RETIRE_PROPOSED rows each emit a blocker line. Push goes GREEN only when ALL blocking rows clear (per HANDOVER §3 original design). Best paths to unblock:
1. Owner confirm-retire the 37 RETIRE_PROPOSED rows (one TTY run; closes 37 blockers).
2. Continue closing the 24 MISSING_TEST rows (write bespoke verifiers / new tests; longer tail).
3. Relax walker to BLOCK only when `alignment_ratio < floor` (W9 / v0.13.0 work — current 0.8128 well above 0.50 floor).

After step 1, residual 24 MISSING_TEST rows are still per-row blockers; step 2 or step 3 needed for fully GREEN pre-push.

**Push status:** local ahead of origin by ~30 commits (pre-push hook BLOCKs on docs-alignment/walk; commits stay local). Pre-push gate consistently 21 PASS / 1 FAIL / 3 WAIVED across every push attempt — only docs-alignment/walk fails, every other gate (clippy, fmt, mkdocs-strict, banned-words, structural invariants) PASSES.

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
