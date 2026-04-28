# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0-autonomous-execution-protocol.md` § "SURPRISES.md format": append one line per unexpected obstacle + its one-line resolution. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`.

Anti-bloat: ≤200 lines. When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface".

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

**Archive rotations** (newest first):
- **P63 Wave 6 (2026-04-28):** archived 7 P59 entries (68 lines) when active crossed 282 lines after P63 entries landed. Active retains P60 onward.
- **P62 Wave 4 (2026-04-28):** archived 10 P57+P58 entries (106 lines) when active crossed 302 lines after P59-P61 entries landed. Active retained P59 onward.
- **P59 Wave F (2026-04-27):** archived 5 P56 entries when active crossed 204 lines.

---

(P56 entries archived 2026-04-27 by P59 Wave F to `quality/SURPRISES-archive-2026-Q2.md`.)



2026-04-27 P60: Wave E pre-push hook one-liner — warm-cache profile
of `python3 quality/runners/run.py --cadence pre-push` was 7.0s on
first run + 5.3s on second (well under the 60s pivot threshold
documented in the plan). Decision: NO PIVOT. cargo fmt + clippy stay
routed through the runner via the Wave D code-dimension wrappers
(cargo's incremental cache makes warm clippy 0.23s; the wrapper is
trivial subprocess overhead on top). Hook body collapsed 229 → 40
lines total / 10 body lines. — Resolution: SIMPLIFY-10 closed in
commit f00affc; the test-pre-push.sh harness needed no edits but
required the hook to be COMMITTED first (test 6's `git reset --hard
HEAD^` reverts uncommitted working-tree changes, restoring the OLD
hook from HEAD before test 6's restore-from-string).

2026-04-27 P60: Wave F mkdocs auto-include verified — `cp
quality/reports/badge.json docs/badge.json && mkdocs build --strict`
produces `site/badge.json` with matching content. mkdocs-material
copies non-md files under `docs/` into the published site without
any `extra_files` directive. No mkdocs.yml edit needed. GH Pages
publish completed within ~90s of the Wave F push commit (verified
via `curl -sIL https://reubenjohn.github.io/reposix/badge.json`
returning HTTP 200 + Content-Type application/json). — Resolution:
QG-09 P60 closure shipped in commit 96b28ca; WAVE_F_PENDING_URLS
cleared in badges-resolve.py; verifier 8/8 PASS immediately
(shields.io endpoint URL returns image/svg+xml even when the inner
github.io URL is mid-publish, so PARTIAL window was nil).

2026-04-27 P60: Wave G zero-RED at sweep entry — the broaden-and-deepen
sweep planned for fixing RED rows surfaced by the dimension's first
production run found NOTHING TO FIX. All 5 P60-touched verifiers
(mkdocs-strict, mermaid-renders, link-resolution, cargo-fmt-check,
cargo-clippy-warnings) PASS individually; all 4 cadences exit 0;
zero P0+P1 NOT-VERIFIED. Waves A-F (catalog-first → migrations → BADGE-01
→ SIMPLIFY-09 → hook one-liner → QG-09 publish) left the dimension
pristine. — Resolution: Wave G shipped a new artifact instead —
`quality/runners/check_p60_red_rows.py` (50-line stdlib Python sentry
that reads the 3 P60-relevant catalogs and reports per-row grades
for the 8 P60-touched rows). Promoted from ad-hoc bash per CLAUDE.md
§4 self-improving infrastructure; reusable by Wave H + verifier
subagent + future regression detection. Lesson: catalog-first
discipline (write rows BEFORE implementation) means the dimension's
first runner sweep is the verification of the planned design, not
a discovery sweep. The "broaden-and-deepen" pattern remains valuable
as insurance, but a clean phase produces a clean sweep.

2026-04-27 P61: Wave B run.py LOC overshoot — adding parse_duration +
is_stale + STALE branch + STALE label + main() suffix wiring pushed
quality/runners/run.py from 330 LOC to a transient 399, over the
390 anti-bloat cap. Pivot rule from 61-02-PLAN fired: extracted
parse_duration to a new sibling module quality/runners/_freshness.py
(50 LOC); is_stale stayed in run.py as a thin 3-line wrapper that
injects parse_rfc3339. Final run.py 388 LOC under cap. Lesson: when
a single file's gain pushes past the soft cap, the pivot is "extract
the leaf utility" not "rewrite the integration"; the integration
preserves the API and the next agent can keep doing
`from run import parse_duration`.

2026-04-27 P61: Wave G runner-subprocess overwrite — the dispatcher
(`bash .claude/skills/reposix-quality-review/dispatch.sh --rubric ...`)
when invoked from the runner subprocess does NOT have Task tool
access, so the Path A scored verdict produced from a Claude session
is overwritten by Path B stub artifacts on every subsequent runner
sweep. The waiver branch in run_row also writes a WAIVED-shape
artifact (no score field) on every cadence run for waivered rows,
clobbering any preceding scored artifact. Resolution: the catalog row
authoritatively encodes the Wave G grading via an extended waiver
(WAIVED-2026-07-26 with documented `dispatched_via=Wave-G-Path-B-in-session`
evidence in the waiver.reason); the artifact JSON is a re-derivation
target. Filed as v0.12.1 MIGRATE-03 carry-forward (e): "Subjective
dispatch-and-preserve runner invariant" -- run_row should treat a
subagent-graded row's recent Path A artifact as authoritative
(read-only on subsequent sweeps; runner sets row.status from the
artifact's score, never overwrites the artifact). Lesson: when the
runner sets the artifact AND reads the artifact, "single writer" is
ambiguous; v0.12.1 needs an explicit kind-aware read-only branch.

2026-04-27 P61: Wave F GH Actions cross-workflow chaining limitation
confirmed (this is a re-mention of P56 SURPRISES row 1 in P61
context). `.github/workflows/quality-pre-release.yml` cannot
chain via `needs:` from `.github/workflows/release.yml` because
`needs:` is same-workflow-only. v0.12.0 ships SOFT-GATE
(parallel-execution + maintainer-alert pattern); HARD-GATE
chaining (release waits for pre-release verdict) requires composite
workflow OR `workflow_run` trigger. Filed as v0.12.1 MIGRATE-03
carry-forward (g). Lesson: every new workflow that wants to gate a
release.yml in this milestone hits the same wall; the v0.12.1 fix
is a single composite workflow restructure, not a per-gate
workaround.

2026-04-27 P61: Wave G broaden-and-deepen produced ZERO P0/P1
findings — the rubric subagent (Path B in-session grading) scored
all 3 rubrics CLEAR (cold-reader 8, install-positioning 9,
headline-numbers 9). 4 P2 polish items deferred to v0.12.1 (MCP
acronym un-glossed; "promisor remote" jargon; docs/index.md
target-arch surfacing; "5-line install" approximation). Lesson:
prior phases (P56 install-path + P58 release dimension) baked the
package-manager-first install ordering and the inline benchmark
citations into the source files; the subjective rubrics now grade
GREEN on first dispatch because the underlying prose is already
clean. The broaden-and-deepen sweep is more valuable as insurance
+ future-regression detection than as a fix-it-now hammer when the
prose is already shipped clean.

2026-04-28 P62: Wave 2 pre-check revealed ~50/99 audited items were
already closed by SIMPLIFY-04..11 + P56-P61 sweeps. — Wave 2 became
"verify closures" not "plan fixes"; Wave 3's actual fix list dropped
to 2 mechanical items (audit relocations + SESSION-END-STATE archive).

2026-04-28 P62: catalog-first dominated planning. Wave 1 locked
ORG-01 + POLISH-ORG (3 structure rows + dim README) BEFORE Wave 2
rendered the audit. — No pivot; the rule worked as designed.

2026-04-28 P62: `scripts/__pycache__/` rec was already-closed by
`.gitignore:30` (covers `__pycache__/` recursively). The 2 .pyc
files were workspace-only, never tracked. — Closed-by-deletion via
workspace cleanup. Lesson: audit doc snapshots can overstate gaps;
the verifier re-classifies present-tense.

2026-04-28 P62: 2 audit "fuse residue" recs
(`docs/development/{roadmap, contributing}.md`) were false positives.
roadmap.md mentions are historical release-notes context (allowed,
like CHANGELOG); contributing.md grep matched the substring "fuse"
inside "**re**fuse". — Both re-classified to closed-by-existing-gate.
Lesson: future audits should `grep -w` for jargon-residue counts.

2026-04-28 P62: quality/gates/structure/freshness-invariants.py
grew to 402 lines after 3 verifier branches landed (over the ~300
anti-bloat hint). Branches share existing helpers; cohesion preserved.
— Deferred helper-module extraction to v0.12.1 MIGRATE-03 unless
Wave 6 flags it. P61's `_freshness.py` is the precedent.

2026-04-28 P63 Wave 1: scripts/check_quality_catalogs.py held stale
contracts (release=16 expecting reposix-swarm row that P58 Wave A
removed; code=3 missing the P58/P60 fmt-check + clippy-warnings +
fixtures-valid additions; orphan-scripts=1 expecting the
crates-io-max-version waiver row that P58 Wave E removed). — Updated
catalog contracts to match current reality (release=15, code=6 with
required-ids enforcing POLISH-CODE rows + extras allowed,
orphan-scripts=17 after Wave 2 populates from caller-scan). Lesson:
catalog-validator scripts need same incremental-update discipline as
the catalogs themselves.

2026-04-28 P63 Wave 2: 5 of 22 audited scripts had zero callers AND
canonical equivalents under quality/runners/* OR per-row
release-assets.json verifiers — DELETE landed cleanly. The other 17
survived as SHIM-WAIVED or KEEP-AS-CANONICAL because CI workflows
(`.github/workflows/{ci.yml, docs.yml, bench-latency-cron.yml}`),
CLAUDE.md command-path documentation, and OP-5 reversibility argued
against deletion. — Lesson: caller-scan that excludes only
`.planning/archive/**` + `quality/SURPRISES-archive-*.md` (the P63 Wave
2 default) gives an accurate picture; scripts with zero non-doc
callers AND a documented canonical equivalent are safe to delete. The
8 KEEP-AS-CANONICAL scripts gained `# KEEP-AS-CANONICAL (P63
SIMPLIFY-12)` header markers as the verifier's source-of-truth.

2026-04-28 P63 Wave 3: cargo-fmt-clean wiring decision — direct
`cargo fmt --all -- --check` invocation honored ONE cargo at a time
rule (read-only, ~5s, no compile). cargo-test-pass intentionally NOT
wired the same way: workspace `cargo nextest run` is 6-15 min +
violates memory-budget + exceeds pre-pr 10-min cadence cap. — CI
remains canonical enforcement venue; tracked-forward to v0.12.1
MIGRATE-03 for per-crate / sccache-warmed alternatives. Lesson:
read-only cargo subcommands (fmt --check, tree, metadata) are safe
verifier-targets; compile-or-test cargo subcommands are not.

2026-04-28 P63 Wave 4: cross-link audit found `bench-token-economy.py`
typo in quality/gates/perf/README.md (P59 SIMPLIFY-11 record had a
dash where the actual file uses underscore). Plus 11 truncated /
template paths flagged by the bare regex (`v0.X.0` placeholders,
`p<N>` template, retired script lineage references). — Typo fixed
in-line; verifier extended with KNOWN_HISTORICAL_OR_PLANNED set + a
`looks_like_doc_anchor` heuristic skipping template patterns
(`v0.X.`, `YYYY`, trailing-dash truncations). 100 paths now verified,
0 stale. Lesson: cross-link verifiers need a small whitelist for
documented-historical refs; bare regex is too aggressive.

2026-04-28 P63 Wave 5: `.planning/milestones/v0.12.1-phases/` scaffold
landed INSIDE the dimension dir per CLAUDE.md `.planning/milestones/`
convention (Option B from HANDOVER §0.5). The
freshness/no-loose-roadmap-or-requirements verifier stays GREEN
because the 2 new files are inside `*-phases/`, not at the
`.planning/milestones/` top level. — Convention is now load-bearing
across 13 milestones (v0.1.0 through v0.12.1). Lesson: when a
"convention" exists for 13 prior milestones, the next milestone scaffold
follows it BY DEFAULT; deviation needs explicit reasoning.

2026-04-28 P63 Wave 5: ad-hoc bash hook flagged a 587-char inline
catalog-tracked-in cross-check pipeline. — Promoted to
`quality/gates/structure/catalog-tracked-in-cross-link.py` per
CLAUDE.md OP-4 (self-improving infrastructure). 4/4 catalog
tracked_in REQ-IDs resolve to v0.12.1 placeholders. Lesson: if you
write a 500-char inline JSON/regex pipeline twice, the second time
the right move is `quality/gates/<dim>/<verb>-<noun>.py` first.

2026-04-28 P64: no significant pivots; the 7-doc design bundle at
`.planning/research/v0.12.0-docs-alignment-design/` left every
architectural decision pre-decided. Wave 1 (catalog-first commit
`d0d4730`) ~25min; Wave 2 (full Rust crate + 28 tests + hash binary
`98dcf11`+`86036c5`) ~15min wall-clock; Wave 3 (this commit + Path B
verifier dispatch) within plan budget. Suspicion-of-haste rule
honored: verifier scrutinized 14 success criteria with primary-source
evidence, spot-checked 3 catalog rows + 3 tests, re-ran cargo test
exit 0. — Lesson: a tight upfront design bundle (rationale +
architecture + execution-modes + overnight-protocol + p64-infra-brief
+ p65-backfill-brief + README) trades ~3h planning for ~5h execution
saved. Worth it on phases that touch >5 files and >2 abstractions.

2026-04-28 P64 Wave 3: docs-alignment/walk gate registry placement
required a design call — the doc-alignment.json catalog has its own
rigid claim-row schema (id/claim/source/source_hash/test/...) that
the binary's `Catalog` struct deserializes; mixing a runner-style gate
row (cadence/verifier/artifact) into `rows[]` would break
deserialization. — Resolved by adding the `docs-alignment/walk` row
to `quality/catalogs/freshness-invariants.json` (the structure
dimension's catalog) under dimension=`docs-alignment`. The runner is
catalog-agnostic — it discovers rows across every catalog file. New
gate row landed at P0 pre-push without schema change to either
catalog. Lesson: the "catalog" dimension boundary in the unified
schema is per-row (`row.dimension`), not per-file — gate rows can
live wherever the schema fits.

2026-04-28 P64 Wave 3: walker writes `summary.last_walked` on every
invocation, mutating `quality/catalogs/doc-alignment.json` even when
rows == [] (empty-state). This produces git churn on every pre-push
that violates the runner's `catalog_dirty()` philosophy
(per-run timestamp churn lives in artifacts, not committed catalogs).
— Accepted for v0.12.0; the walker's spec at
`.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
treats `last_walked` as a catalog-summary field, not artifact metadata.
v0.12.1 carry-forward (filed as part of MIGRATE-03): either move
`last_walked` into the artifact (`quality/reports/verifications/docs-alignment/walk.json`)
or extend `catalog_dirty()` to ignore summary.last_walked drift the
same way it ignores per-row last_verified drift. Lesson: walker
state-change semantics need to align with the runner's
status-only-persists rule from day one; retrofit is cheaper before
backfill populates rows.
