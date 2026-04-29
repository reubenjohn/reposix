# quality/SURPRISES-archive-2026-Q2.md — archived entries

Per `quality/PROTOCOL.md` § "Anti-bloat rules per surface": when
`quality/SURPRISES.md` crosses 200 lines, the oldest entries are
archived here. The active journal (`quality/SURPRISES.md`) keeps the
most recent entries at <=200 lines so it stays the first thing a
new-phase agent reads end-to-end.

Contains entries from P56 through P63, archived across multiple
rotation waves: P59 Wave F (P56, 5 entries), P62 Wave 4 (P57+P58,
10 entries), P63 Wave 6 (P59, 7 entries), and the 2026-04-29 prep
rotation (P60-P63, 18 entries / 192 lines). Each wave followed the
anti-bloat rule in `quality/PROTOCOL.md` § "Anti-bloat rules per
surface" — when the active journal at `quality/SURPRISES.md`
crossed 200 lines, the oldest phase boundary was rotated here.

The active journal at `quality/SURPRISES.md` is the canonical
start-of-phase reading; this file is reference material for
forensic dives into prior pivots.

---

## 2026-Q2 archived entries

### P56 (5 entries; archived 2026-04-27 P59 Wave F)

2026-04-27 P56: GitHub's `releases/latest/download/...` pointer follows
release recency, but release-plz cuts ~8 per-crate releases per version
bump. A non-cli per-crate release published after the cli release moves
the latest pointer and re-breaks `releases/latest/download/reposix-installer.sh`
until the next cli release. — Tracked under MIGRATE-03 (v0.12.1
carry-forward). Recovery options: (a) `gh release create --latest` to
pin the pointer to the cli release in release.yml, or (b) configure
release-plz to publish reposix-cli last in its per-crate sequence.

2026-04-27 P56: release-plz GITHUB_TOKEN-pushed tags do NOT trigger
downstream `on.push.tags` workflows — GH loop-prevention rule for
GITHUB_TOKEN-pushed refs. release.yml's `reposix-cli-v*` glob is
correct, but the tag push from release-plz never fires the workflow.
— Workaround: `gh workflow run release.yml --ref reposix-cli-v0.11.3`
(workflow_dispatch) used as Wave 3 stop-gap. Fix path under MIGRATE-03
(v0.12.1 carry-forward): release-plz workflow uses fine-grained PAT
(non-GITHUB_TOKEN) OR adds a post-tag dispatch step. ~5 LOC.

2026-04-27 P56: install/cargo-binstall metadata in
`crates/reposix-cli/Cargo.toml` + `crates/reposix-remote/Cargo.toml`
is misaligned with release.yml's archive shape (4 mismatches: tag
prefix `v` vs `reposix-cli-v`, archive basename `reposix-cli` vs
`reposix`, target glibc vs musl, `.tgz` vs `.tar.gz`). binstall falls
back to source build, which then itself fails because of the MSRV
bug below. — Catalog row marked PARTIAL (blast_radius P1, "works just
slow"); ~10 LOC `[package.metadata.binstall]` fix tracked under
MIGRATE-03 (v0.12.1 carry-forward).

2026-04-27 P56: Rust 1.82 (project MSRV) cannot `cargo install reposix-cli`
from crates.io — transitive dep `block-buffer-0.12.0` requires
`edition2024` which is unstable on 1.82. cargo install ignores the
project's pinned Cargo.lock so this is invisible to ci.yml's `test`
job (which builds against the workspace lockfile). — Orthogonal MSRV
bug; fix under MIGRATE-03 (v0.12.1 carry-forward) is either cap dep
at `<0.12` or raise MSRV to 1.85.

2026-04-27 P56: curl rehearsal `curl -sLI URL | head -20` under
`set -euo pipefail` exits 23 (FAILED_WRITING_OUTPUT) when GitHub's
HEAD response exceeds 20 lines (their content-security-policy
header is huge); pipefail propagates and bash exits before running
the installer step. — Fixed in `scripts/p56-rehearse-curl-install.sh`:
capture HEAD to a tempfile, then `head -20` on the static file. Same
diagnostic value, no SIGPIPE. Lesson for future verifiers: tempfile-then-grep,
not pipe-into-head, when the upstream response size is unbounded.


### P57 (3 entries; archived 2026-04-28 P62 Wave 4)

2026-04-27 P57: Wave B runner had idempotency bug — em-dashes in catalog
note fields were being escape-mangled across runs (`—` re-encoded to
`—`), AND every invocation was rewriting the catalog `last_verified`
even when no row state changed. Two pre-push runs back-to-back produced
a non-empty `git diff` on the catalog file, breaking the GREEN-clean
invariant. — Fixed in commit `dd458bd` (fix(p57): runner idempotency).
Reproduction promoted to `scripts/test-runner-invariants.py` so the
invariant is enforceable from CI; ad-hoc bash → committed test artifact
per CLAUDE.md §4 (Self-improving infrastructure).

2026-04-27 P57: Wave B catalog amendment — initial catalog row schema
emitted unicode-escaped em-dashes (`—`) on first write; later
runs produced literal `—` (preserve-unicode mode). One-time
normalization sweep brought all rows to literal-em-dash form.
— Resolved in same Wave B; subsequent runs are idempotent.

2026-04-27 P57: phase shipped without further pivots — POLISH-STRUCT
(Wave D) closed cleanly with the chunky 480-line ROADMAP move (3 details
wrappers + the v0.11.0 H2 section + `<details>` blocks for Phase 30
SUPERSEDED + v0.1.0–v0.7.0 archive + v0.8.0). v0.10.0 + v0.9.0
per-milestone files were preserved verbatim per the verify-before-edit
rule (markers + line counts confirmed). SIMPLIFY-03 (Wave E) audit
confirmed Wave A's boundary doc was sufficient — no edit to
`quality/catalogs/README.md` needed. — All 9 catalog rows GREEN or
WAIVED; verdict at `quality/reports/verdicts/p57/VERDICT.md`.

### P58 (7 entries; archived 2026-04-28 P62 Wave 4)

2026-04-27 P58: Wave A's release-assets.json catalog included 9
crates-io max-version rows on the assumption that all 9 reposix
crates publish. Wave B's self-test sweep showed reposix-swarm
returns HTTP 404 from crates.io; `crates/reposix-swarm/Cargo.toml`
has `publish = false` (intentional — internal multi-agent contention
test harness). The verifier surfaces the genuine fact (FAIL with
"GET .../reposix-swarm HTTP 200 — got status=404"). — Per stop
condition, left as-discovered for Wave E to reconcile: either
remove the row (catalog drift fix) or convert to a permanent
waiver (`tracked_in: 'reposix-swarm publish=false (internal-only)'`).
Other 8 crates PASS at 0.11.3.

2026-04-27 P58: Wave A pre-push runner reported NOT-VERIFIED for
new P1 pre-push row code/clippy-lint-loaded because Wave A commits
the catalog row before Wave C ships the verifier wrapper at
`quality/gates/code/clippy-lint-loaded.sh`. The runner's
verifier-not-found branch sets NOT-VERIFIED, which fails exit on
P0+P1 rows. — Resolved by attaching a short-lived waiver
(`until: 2026-05-11T00:00:00Z`, `tracked_in: P58 Wave C (58-03)`)
to the catalog row. Wave C removes the waiver and flips to active
enforcement. Rule 3 deviation; recorded in 58-01-SUMMARY.md.

2026-04-27 P58: Wave D first dispatch of quality-weekly.yml exposed
install/build-from-source RED in CI: gh CLI in GH Actions environment
requires explicit `GH_TOKEN: ${{ github.token }}` env var on each
runner step (the system-default GITHUB_TOKEN is NOT auto-passed to
gh CLI, only to actions/* uses). Locally the same row PASSes because
gh CLI uses the user's stored auth. — Fixed in commit 664b533:
GH_TOKEN env added to runner + verdict steps in both quality-weekly.yml
and quality-post-release.yml. Run 25020034212 confirmed fix
(install/build-from-source PASS). Lesson: verifiers calling `gh`
CLI must always run with GH_TOKEN env in GH Actions; treat this as
the default workflow shape going forward.

2026-04-27 P58: Wave D first dispatch of quality-post-release.yml
exposed cargo-binstall-resolves verifier bug. The PARTIAL_SIGNALS
tuple (`Falling back to source`, `Falling back to install via
'cargo install'`, `compiling reposix`) did NOT match the actual
binstall stdout, which says `will be installed from source (with
cargo)`. The verifier graded FAIL on what should have been the
documented PARTIAL state per P56 SURPRISES.md row 3. — Fixed in
commit e0e5645: added 3 additional fallback signals (`will be
installed from source`, `running \`cargo install`,
`running '/home/runner/.rustup`). Run 25020150833 confirms fix:
PARTIAL graded correctly. Lesson: when a "documented expected"
PARTIAL state lands in production, exercise the verifier against
the real production output, not just the design-intent string.

2026-04-27 P58: Wave E reconciled the catalog-drift RED that
Wave A intentionally surfaced — `release/crates-io-max-version/
reposix-swarm` was a design-time mistake (the crate has
`publish = false`; internal multi-agent contention test harness
that is never published to crates.io). — Resolved by REMOVING the
row from quality/catalogs/release-assets.json (15 rows now;
8 crates-io-max-version/<crate> rows for the published crates).
quality/gates/release/README.md gained a "reposix-swarm exclusion"
section acknowledging the design-time error. Lesson: the
catalog-first rule (write rows before code) IS load-bearing —
the verifier surfaced the drift on first dispatch; the fix landed
in the same milestone instead of leaking to v0.12.1.

2026-04-27 P58: Wave E waived release/cargo-binstall-resolves
explicitly (until 2026-07-26, tracked_in MIGRATE-03 v0.12.1).
The runner's exit-code logic treats PARTIAL on P1 as fail — the
documented expected PARTIAL needs to be a waiver, not a status.
The waiver text covers BOTH cases: (a) CI-with-binstall observes
"will be installed from source" → PARTIAL; (b) local-without-binstall
sees `error: no such command: 'binstall'` → FAIL with diagnostic.
Both are acceptable in the waiver window. — Resolution: not a
pivot per se, but a contract clarification: PARTIAL acceptable
under the framework's exit-code rules requires an explicit waiver.

2026-04-27 P58: phase shipped with documented pivots — release-dim
gates landed clean (15 weekly rows + 1 post-release row), code-dim
absorption confirmed (SIMPLIFY-04 + SIMPLIFY-05 closed; check_fixtures
audit chose Option A), quality-weekly + quality-post-release
validated end-to-end (4 dispatches; 2 verifier/workflow fixes
applied), QG-09 P58 GH Actions badge live in README + docs/index.md.
— All catalog rows GREEN or WAIVED; verdict at
quality/reports/verdicts/p58/VERDICT.md (Wave F).


## P59 entries (rotated 2026-04-28 P63 Wave 6, active 282 -> ~214 lines)

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


## P60-P63 entries (rotated 2026-04-29 prep, active 351 -> 159 lines)

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
