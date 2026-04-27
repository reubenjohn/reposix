# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0-autonomous-execution-protocol.md`
§ "SURPRISES.md format": append one line per unexpected obstacle +
its one-line resolution. **Required reading for the next phase agent.**
The next agent does NOT repeat investigations of things already
journaled here. Format: `YYYY-MM-DD P<N>: <what happened> — <one-line
resolution>`. Anti-bloat: ≤200 lines. When it crosses, archive oldest
50 to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh. Seeded
by P56 (Wave 4-B); P57 takes ownership when the framework skeleton ships.

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

**P59 Wave F archive rotation:** the 5 oldest entries (P56) were archived to `quality/SURPRISES-archive-2026-Q2.md` when this file crossed 204 lines after Waves B-C landed. The active journal now retains P57+ entries.

Anti-bloat: ≤200 lines. When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface" for the rotation rule.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

---

(P56 entries archived 2026-04-27 by P59 Wave F to `quality/SURPRISES-archive-2026-Q2.md`.)

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
