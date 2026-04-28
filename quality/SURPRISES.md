# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0-autonomous-execution-protocol.md` § "SURPRISES.md format": append one line per unexpected obstacle + its one-line resolution. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`.

Anti-bloat: ≤200 lines. When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface".

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

**Archive rotations** (newest first):
- **P62 Wave 4 (2026-04-28):** archived 10 P57+P58 entries (106 lines) when active crossed 302 lines after P59-P61 entries landed. Active retains P59 onward.
- **P59 Wave F (2026-04-27):** archived 5 P56 entries when active crossed 204 lines.

---

(P56 entries archived 2026-04-27 by P59 Wave F to `quality/SURPRISES-archive-2026-Q2.md`.)


2026-04-27 P59: Wave B fenced-block survey returned 32 blocks across 6
files (under PIVOT_THRESHOLD=50 → per-block tracking applies). Of
those, 11 are covered by existing release-assets + docs-repro example
rows; 21 illustrative blocks (mermaid diagram, troubleshooting examples,
connector-tutorial code) moved to a NEW
quality/catalogs/docs-reproducible-allowlist.json with per-id
reasons. Cross-catalog source citation matching was added to
quality/gates/docs-repro/snippet-extract.py so release-assets rows
citing README.md install lines cover their corresponding fenced blocks
transparently — released this as a Rule 1 fix (without it the drift
detector flagged blocks that ARE catalogued, just in a sibling catalog).

2026-04-27 P59: Wave C container-rehearse.sh ships but the example
run scripts (examples/0[1,2,4,5]-*/run.{sh,py}) assume an external
simulator listening on 127.0.0.1:7878 that the container does not
bring up. Locally the verifier exits non-zero with stderr "sim not
reachable" — same diagnostic for any caller. — Resolution: short-lived
waiver attached (until 2026-05-12) to all 4 container example rows +
the tutorial-replay row, tracked_in "P59 Wave F CI rehearsal in
docker-equipped GH runner with sim service". Pattern mirrors the P58
Wave A clippy-lint-loaded waiver. The container-rehearse.sh driver
itself is correct + tested via the docker-absent skip path; the gap
is plumbing sim-inside-container, which is post-v0.12.0 scope.

2026-04-27 P59: SIMPLIFY-06 closure — scripts/repro-quickstart.sh
deleted (no callers found in .github/, scripts/, docs/, examples/,
CLAUDE.md, README.md). The tutorial-replay.sh canonical home at
quality/gates/docs-repro/ ports the 7-step assertion shape verbatim;
the row's `sources` field references the historical predecessor with
"see commit history" so the lineage is discoverable without keeping
a stub file alive.

2026-04-27 P59: Wave D SIMPLIFY-07 chose SHIM (not delete) for
scripts/dark-factory-test.sh, opposite of P58's SIMPLIFY-04+05 which
DELETED their predecessors. Reason: caller audit found 14 references
across CLAUDE.md "Local dev loop", README.md, docs/reference/cli.md,
docs/reference/simulator.md, docs/reference/crates.md,
docs/development/contributing.md, docs/decisions/001-github-state-mapping.md,
examples/03-claude-code-skill/RUN.md,
examples/04-conflict-resolve/expected-output.md,
examples/05-blob-limit-recovery/{RUN.md, expected-output.md},
scripts/green-gauntlet.sh. Deleting would have broken developer
muscle memory + the canonical examples docs. P63 SIMPLIFY-12 audits
the shim. — Resolution: 7-line shim at scripts/dark-factory-test.sh
that exec's quality/gates/agent-ux/dark-factory.sh "$@". CI workflow
ci.yml updated to invoke canonical path explicitly per OP-1.

2026-04-27 P59: Wave E SIMPLIFY-11 had two pivots in one commit. (1)
Option B underscore: bench_token_economy.py kept underscore at
quality/gates/perf/ (not hyphenated like other-dim entry-points)
because the test file imports `bench_token_economy` as a Python
module — hyphen breaks module syntax. Wave A's hyphenated catalog
row corrected to underscore in same commit (4-char edit). (2)
REPO_ROOT path arithmetic: predecessor used `parent.parent` /
`SCRIPT_DIR/..` assuming scripts/ = one-level. From quality/gates/perf/
that resolves to quality/gates/, breaking benchmarks/fixtures lookups.
— Resolution: Python `parents[3]`; bash `cd "${SCRIPT_DIR}/../../.."`.
9/9 tests pass at new location; bench --offline exits 0 via shim.
Lesson: any __file__-derived REPO_ROOT needs path-arithmetic audit
on migration; the depth changed from 1 to 3.

2026-04-27 P59: Wave F archive rotation — SURPRISES.md crossed 204
lines after Waves B-C landed. Per quality/PROTOCOL.md anti-bloat
rule, archived 5 oldest entries (P56) to quality/SURPRISES-archive-2026-Q2.md.
Active journal now retains P57+ entries. First archive rotation
since the journal was seeded — establishes the quarterly-archive
convention. Active SURPRISES.md header gained pointer paragraph
naming the archive file.

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
