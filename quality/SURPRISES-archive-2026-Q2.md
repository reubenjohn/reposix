# quality/SURPRISES-archive-2026-Q2.md — archived entries

Per `quality/PROTOCOL.md` § "Anti-bloat rules per surface": when
`quality/SURPRISES.md` crosses 200 lines, the oldest entries are
archived here. The active journal (`quality/SURPRISES.md`) keeps the
most recent entries at <=200 lines so it stays the first thing a
new-phase agent reads end-to-end.

**Archived 2026-04-27 by P59 Wave F:** the 5 P56 entries below were the
oldest (5 of 16 entries; pre-quality-gates-skeleton). Archived because
SURPRISES.md crossed 204 lines after 4 P59 entries from Waves B-C
landed; further P59 Wave D-F entries would have pushed it over.

The active journal at `quality/SURPRISES.md` retains the 11 entries
from P57 onward (3 P57 + 7 P58 + 4 P59 + Wave D-F additions).

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

