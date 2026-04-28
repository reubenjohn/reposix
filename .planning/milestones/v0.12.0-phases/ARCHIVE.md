# v0.12.0 — phases archive

**Shipped:** 2026-04-27 — Quality Gates framework + 8 dimensions homed.

**Release status:** NOT TAGGED. No `crates/` source changed across P56–P63 (framework + docs + scripts + CI only), so a binary release would be bit-for-bit identical to v0.11.3. Workspace `Cargo.toml` stays at `0.11.3`. Next binary release is v0.13.0, which will tag the cumulative worktree. See `CHANGELOG.md` `[v0.12.0]` for the user-facing version of this note.

**Phases:** 56, 57, 58, 59, 60, 61, 62, 63.

**Source-of-truth narrative:**
- `CHANGELOG.md` `[v0.12.0]` section — user-facing release notes.
- `.planning/ROADMAP.md` § v0.12.0 — milestone summary.
- `.planning/milestones/v0.12.0-phases/{ROADMAP.md, REQUIREMENTS.md}` — milestone-level scoping.
- `quality/reports/verdicts/p{57,58,59,60,61,62,63}/VERDICT.md` + `.planning/verifications/p56/VERDICT.md` — per-phase verifier verdicts (all GREEN; P56 pre-dates the `quality/reports/` tree per QG-06 transitional note).
- `git log --oneline v0.11.x..v0.12.0` — implementation commits.

**Per-phase contributions (formerly in CLAUDE.md; archived 2026-04-27 as part of post-milestone cohesion pass):**

## v0.12.0 Quality Gates — phase log

The v0.12.0 milestone migrates ad-hoc `scripts/check-*.sh` and the
conflated `scripts/end-state.py` to a dimension-tagged Quality
Gates system at `quality/{gates,catalogs,reports,runners}/`. **The
framework itself ships in P57** — until then, the catalog format
lives in `.planning/docs_reproducible_catalog.json` (ACTIVE-V0; P57
migrates it to the unified schema). Read
`.planning/research/v0.12.0-{vision-and-mental-model,
naming-and-architecture, roadmap-and-rationale,
autonomous-execution-protocol}.md` before working on any v0.12.0
phase. Read `quality/SURPRISES.md` (append-only pivot journal — seeded
in P56) at the start of every phase to avoid repeating dead ends.
Future phases follow `quality/PROTOCOL.md` (lands P57 — autonomous-mode
runtime contract).

### P56 contribution — release pipeline + install-evidence pattern

`.github/workflows/release.yml` now fires on **both** `v*` AND
`reposix-cli-v*` tag globs (Option A from
`.planning/research/v0.12.0-install-regression-diagnosis.md`),
because release-plz cuts per-crate tags. Archive filenames use the
`v${version}` form (e.g. `reposix-v0.11.3-x86_64-unknown-linux-musl.tar.gz`)
regardless of which tag triggered the workflow; the GH Release object
still uses the actual tag name (e.g. `reposix-cli-v0.11.3`) so
release-plz's existing release object is the one that gets assets
attached. **If you edit release.yml,** preserve the
`release_tag`/`version`/`version_tag` distinction in the `plan` job's
outputs — collapsing them re-introduces the regression. The
install-path contract lives in
`.planning/docs_reproducible_catalog.json` until P58 splits it into
`quality/catalogs/install-paths.json`.

P56 shipped these new artifacts (referenced by future phases):

- `scripts/p56-rehearse-curl-install.sh` — ubuntu:24.04 container rehearsal for the curl one-liner.
- `scripts/p56-rehearse-cargo-binstall.sh` — rust:1.82-slim container rehearsal for `cargo binstall`.
- `scripts/p56-asset-existence.sh` — generic HEAD/range asset reachability check.
- `scripts/p56-validate-install-evidence.py` — install-evidence JSON validator (Wave 4 verifier subagent re-runs this).
- `.planning/verifications/p56/install-paths/<id>.json` — per-row evidence files.
- `.planning/docs_reproducible_catalog.json` — ACTIVE-V0 catalog seeded with 5 install rows + 6 docs/tutorial/benchmark rows.

### Container-rehearsal evidence schema (P56 → P58 forward path)

Every install-path verifier writes JSON to
`.planning/verifications/p56/install-paths/<id>.json` with the shape:

```jsonc
{
  "claim_id": "install/<slug>",
  "phase": "p56",
  "verifier_kind": "asset-existence | container-rehearsal | shell-against-checkout",
  "verified_at": "<RFC3339-UTC>",
  "container_image": "<image:tag>",     // present when verifier_kind contains container-rehearsal
  "container_image_digest": "<sha256:…>",// optional but recommended
  "verifier_script": "scripts/<committed-script>.sh",
  "asserts": { /* per-row assertion booleans + observed values */ },
  "evidence": { /* container_log_path, log excerpts, exit codes, observed bytes */ },
  "status": "PASS | FAIL | PARTIAL | NOT-VERIFIED"
}
```

Future-shape note: P58 ports this schema to `quality/reports/verifications/release/`
under the unified Quality Gates layout. New install-path or release-pipeline
verifiers should write the same shape there from day one — same `claim_id`,
same `asserts`/`evidence` discipline.

### Meta-rule (extension of OP-1 close-the-loop)

When a release-pipeline regression is fixed, the same PR ships
container-rehearsal evidence under `.planning/verifications/p56/`
(or `quality/reports/verifications/release/` after P58). "Fix landed
green" without a re-run-from-scratch evidence JSON is incomplete —
release.yml can land green and still surface trigger-semantics gaps
the next phase has to journal.

### Carry-forward to v0.12.1 (tracked under MIGRATE-03)

Four items discovered in P56 deferred to v0.12.1 carry-forward:

1. **release-plz GITHUB_TOKEN-pushed tags don't trigger downstream workflows** —
   GH loop-prevention rule. Workaround: `gh workflow run --ref <tag>` (workflow_dispatch).
   Fix path: release-plz workflow uses fine-grained PAT or adds a post-tag dispatch step.
2. **install/cargo-binstall metadata misaligned** with release.yml archive shape —
   ~10 LOC `[package.metadata.binstall]` fix in `crates/reposix-cli/Cargo.toml` +
   `crates/reposix-remote/Cargo.toml`; row stays PARTIAL until v0.12.1.
3. **Rust 1.82 MSRV can't `cargo install reposix-cli` from crates.io** because
   transitive `block-buffer-0.12.0` requires `edition2024` (unstable on 1.82).
   Orthogonal MSRV bug — fix is either cap dep at <0.12 or raise MSRV to 1.85.
4. **Latest-pointer caveat** — `releases/latest/download/...` follows release
   recency; release-plz cuts per-crate releases; a non-cli release published
   after the cli release moves the pointer and re-breaks the curl URL until the
   next cli release. Long-term fix: `gh release create --latest` to pin the
   pointer, or configure release-plz to publish reposix-cli last.

Cross-references:

- `.planning/REQUIREMENTS.md` ## v0.12.0 (active milestone scope, MIGRATE-03 carry-forward list)
- `.planning/ROADMAP.md` ## v0.12.0 Quality Gates (PLANNING) (Phases 56-63)
- `quality/PROTOCOL.md` (lands P57 — autonomous-mode runtime contract)
- `quality/SURPRISES.md` (append-only pivot journal; ≤200 lines, archives at 200)

### P58 — Release dimension live (added 2026-04-27)

The release dimension is now actively enforcing.

| Verifier | Catalog rows | Cadence |
|---|---|---|
| `quality/gates/release/gh-assets-present.py` | `release/gh-assets-present` | weekly |
| `quality/gates/release/installer-asset-bytes.py` | `install/curl-installer-sh`, `install/powershell-installer-ps1`, `install/build-from-source` | weekly |
| `quality/gates/release/brew-formula-current.py` | `release/brew-formula-current`, `install/homebrew` | weekly |
| `quality/gates/release/crates-io-max-version.py` | `release/crates-io-max-version/<crate>` (8 rows; `reposix-swarm` excluded — `publish=false`) | weekly |
| `quality/gates/release/cargo-binstall-resolves.py` | `release/cargo-binstall-resolves` | post-release |

**Cadence wiring.** `.github/workflows/quality-weekly.yml` (cron Monday 09:00 UTC + workflow_dispatch) drives weekly. `.github/workflows/quality-post-release.yml` (workflow_run on `release` + workflow_dispatch) drives post-release. Both pass `GH_TOKEN: ${{ github.token }}` so verifiers calling `gh` CLI auth correctly.

**QG-09 P58 GH Actions badge.** README.md and docs/index.md link the live workflow status alongside the existing CI badge: `https://github.com/reubenjohn/reposix/actions/workflows/quality-weekly.yml/badge.svg`. The shields.io endpoint badge ships in P60 alongside mkdocs publishing badge.json.

**Code dimension absorption (P58 Wave C).** `quality/gates/code/` ships three verifiers: `clippy-lint-loaded.sh` (SIMPLIFY-04 migration of `scripts/check_clippy_lint_loaded.sh`); `check-fixtures.py` (SIMPLIFY-05 Option A migration of `scripts/check_fixtures.py`); `ci-job-status.sh` (POLISH-CODE P58-stub thin gh-CLI wrapper backing `code/cargo-test-pass` + `code/cargo-fmt-clean`). Both old script paths DELETED — caller analysis returned zero shell/yaml/toml callers.

**Orphan-scripts ledger.** `quality/catalogs/orphan-scripts.json` is empty after Wave E removed the `release/crates-io-max-version` waiver row. The dimension provides active enforcement now. SIMPLIFY-12 P63 audits this file at milestone close — goal is to keep it empty.

**Recovery patterns** (the regressions this dimension catches):
- `release/brew-formula-current` RED with stale version: `gh workflow run release.yml --ref reposix-cli-vX.Y.Z` (P56 SURPRISES.md row 2 latest-pointer pattern).
- `release/gh-assets-present` RED with missing assets: same recovery (release.yml didn't fire on the latest tag).
- `release/cargo-binstall-resolves` PARTIAL: documented expected per P56 SURPRISES.md row 3 + waived until 2026-07-26; MIGRATE-03 v0.12.1 ships the binstall metadata fix.

**Meta-rule extension (P58).** When a release-pipeline regression is fixed, the same PR ships container-rehearsal evidence under `quality/reports/verifications/release/`. The artifact JSON IS the proof; the verifier subagent reads it.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/release/README.md` — release-dim verifier table + conventions.
- `quality/gates/code/README.md` — code-dim verifier table + SIMPLIFY-04/05 absorption record.
- `quality/catalogs/release-assets.json` — 15-row catalog (P58 active enforcement).
- `quality/catalogs/code.json` — 4-row catalog (clippy + fixtures PASS; test + fmt WAIVED until P63).

### P59 — Docs-repro + agent-ux + perf-relocate dimensions live (added 2026-04-27)

Three more dimensions land. The docs-repro dimension is the deepest (9 rows + drift detector); agent-ux is intentionally sparse (1 row); perf is file-relocate-only at v0.12.0.

**Docs-repro home:** `quality/gates/docs-repro/`. 3 verifiers + 1 manual-spec checker:

| Verifier | Catalog rows | Cadence |
|---|---|---|
| `snippet-extract.py` (--list / --check / --write-template) | `docs-repro/snippet-coverage` | pre-push |
| `container-rehearse.sh <id>` | 4 example container rows + `docs-repro/tutorial-replay` | post-release |
| `tutorial-replay.sh` (SIMPLIFY-06; `scripts/repro-quickstart.sh` deleted) | `docs-repro/tutorial-replay` | post-release |
| `manual-spec-check.sh <id>` | `docs-repro/example-03-claude-code-skill` | on-demand |

The 4 container rows + tutorial-replay are WAIVED until 2026-05-12 — the example scripts assume an external simulator that the container does not bring up; sim-inside-container plumbing is post-v0.12.0 work. Snippet-coverage row PASS (drift detector); example-03-claude-code-skill PASS (manual-spec-check).

**Agent-ux home:** `quality/gates/agent-ux/`. Intentionally sparse — dark-factory regression is the only gate at v0.12.0; perf and security stubs land v0.12.1 per MIGRATE-03.

| Verifier | Catalog row | Cadence |
|---|---|---|
| `dark-factory.sh sim` (SIMPLIFY-07; migrated from `scripts/dark-factory-test.sh`) | `agent-ux/dark-factory-sim` | pre-pr |

The v0.9.0 dark-factory invariant (helper stderr-teaching strings on conflict + blob-limit paths) is preserved verbatim. `.github/workflows/ci.yml` invokes the canonical path explicitly; `scripts/dark-factory-test.sh` survives as a 7-line shim per OP-5 reversibility (CLAUDE.md "Local dev loop" command keeps working unchanged).

**Perf home (relocate-only):** `quality/gates/perf/`. File-relocate stubs only at v0.12.0; full gate logic deferred to v0.12.1 stub per MIGRATE-03. 3 catalog rows all WAIVED until 2026-07-26.

| Source | Migrated to | Cadence | Status |
|---|---|---|---|
| `scripts/latency-bench.sh` | `quality/gates/perf/latency-bench.sh` | weekly | WAIVED v0.12.1 |
| `scripts/bench_token_economy.py` | `quality/gates/perf/bench_token_economy.py` | weekly | WAIVED v0.12.1 |
| `scripts/test_bench_token_economy.py` | `quality/gates/perf/test_bench_token_economy.py` | (test) | n/a |

`benchmarks/fixtures/*` stays in place (test inputs, not gates). The 2 perf shims at `scripts/{latency-bench.sh, bench_token_economy.py}` exec/subprocess.run the canonical paths; the test file is renamed-only (no shim — pytest auto-discovers).

**SIMPLIFY-06/07/11 absorption record:**
- SIMPLIFY-06: `scripts/repro-quickstart.sh` DELETED (no callers; tutorial-replay.sh ports the 7-step assertion shape verbatim).
- SIMPLIFY-07: `scripts/dark-factory-test.sh` SHIM (CLAUDE.md "Local dev loop" command + 14 doc/example refs keep working unchanged).
- SIMPLIFY-11: 3 perf scripts MIGRATED via git mv; 2 shims at old paths; test file deleted.

**Recovery patterns:**
- snippet drift detected: `python3 quality/gates/docs-repro/snippet-extract.py --write-template <derived-id>` → paste output into `quality/catalogs/docs-reproducible.json`.
- container rehearsal RED: read `quality/reports/verifications/docs-repro/<row-id>.json`; `stderr` field has the docker output.
- dark-factory regression RED: run `bash quality/gates/agent-ux/dark-factory.sh sim` with `set -x`; the v0.9.0 invariant is the teaching-strings assertion.
- docker-absent (local dev): the runner emits stderr warning + non-fatal exit; container rows grade WAIVED. CI dispatch is the actual rehearsal.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/docs-repro/README.md` — docs-repro verifier table + conventions.
- `quality/gates/agent-ux/README.md` — agent-ux thin home + intentional sparsity note.
- `quality/gates/perf/README.md` — perf relocate-only stubs + waiver explanation.
- `quality/PROTOCOL.md` — runtime contract (cadences + waiver protocol + verifier-subagent template).
- MIGRATE-03 in `.planning/REQUIREMENTS.md` — v0.12.1 carry-forward (perf full implementation + container sim plumbing).

### P60 — Docs-build dimension live + composite cutover (added 2026-04-27)

Docs-build dimension lands with 4 verifiers backing 4 catalog rows in `quality/catalogs/docs-build.json` plus the back-compat `structure/badges-resolve` row in `quality/catalogs/freshness-invariants.json` (P57 pre-anchored; P60 implements via shared verifier). Same wave hard-cuts the pre-push hook to a runner one-liner and supplants the green-gauntlet composite.

| Verifier | Catalog row(s) | Cadence |
|---|---|---|
| `quality/gates/docs-build/mkdocs-strict.sh` | `docs-build/mkdocs-strict` | pre-push |
| `quality/gates/docs-build/mermaid-renders.sh` | `docs-build/mermaid-renders` | pre-push |
| `quality/gates/docs-build/link-resolution.py` | `docs-build/link-resolution` | pre-push |
| `quality/gates/docs-build/badges-resolve.py` | `docs-build/badges-resolve` + `structure/badges-resolve` | pre-push + weekly |

**SIMPLIFY-08/09/10 absorption (3 closures in one phase):**
- **SIMPLIFY-08**: 3 git-mv migrations preserve history — `scripts/check-docs-site.sh` → `mkdocs-strict.sh`; `scripts/check-mermaid-renders.sh` → `mermaid-renders.sh`; `scripts/check_doc_links.py` → `link-resolution.py`. Thin shims at old paths per OP-5; pre-push hook + CI workflows continue working unchanged through migration.
- **SIMPLIFY-09**: `scripts/green-gauntlet.sh` rewritten as a thin shim that delegates to `python3 quality/runners/run.py --cadence pre-pr`. The 3 modes (`--quick`, default, `--full`) collapse to runner cadence calls.
- **SIMPLIFY-10**: `scripts/hooks/pre-push` body collapsed from 229 lines (6 chained verifiers) to 40 lines total / 10 body lines — a cred-hygiene wrapper invocation (P0 fail-fast on the stdin push-ref ranges) followed by a single `exec python3 quality/runners/run.py --cadence pre-push`. Warm-cache profile 5.3s; well under the 60s pivot threshold so cargo fmt + clippy stay routed THROUGH the runner via the Wave D code-dimension wrappers.

**BADGE-01 closure**: `quality/gates/docs-build/badges-resolve.py` HEADs every badge URL extracted from `README.md` + `docs/index.md` (8 unique URLs) and asserts HTTP 200 + Content-Type matches `image/*` OR `application/json`. Stdlib-only `urllib.request`; ~165 lines. Wave C handled the QG-09 chicken-and-egg via `WAVE_F_PENDING_URLS` (skip-and-log); Wave F cleared the set after the publish landed.

**QG-09 P60 closure**: `docs/badge.json` seeded from `quality/reports/badge.json` and auto-included in mkdocs build → `https://reubenjohn.github.io/reposix/badge.json` resolves. `README.md` (8 badges) + `docs/index.md` (3 badges) gain `![Quality](https://img.shields.io/endpoint?url=...)`. The Quality (weekly) GH Actions badge from P58 stays alongside; the two convey complementary signals (workflow status vs catalog rollup). GH Pages publish completed within ~90s of the Wave F push commit.

**POLISH-DOCS-BUILD broaden-and-deepen (Wave G)**: 4 cadences GREEN at sweep entry (zero RED to fix; Waves A-F left the dimension pristine). 19 PASS pre-push, 1+2 WAIVED pre-pr, 14+3+2 weekly, 6 WAIVED post-release. New artifact `quality/runners/check_p60_red_rows.py` reads the 3 P60-relevant catalogs and reports per-row grades for the 8 P60-touched rows — promoted from ad-hoc bash per CLAUDE.md §4.

**Recovery patterns:**
- mkdocs --strict RED: read `/tmp/check-docs-site*/build.log`; common causes are broken cross-refs, admonition syntax, mermaid HTML-entity escaping, missing nav entry.
- mermaid-renders RED: refresh artifacts via `mcp__playwright__*` walks per CLAUDE.md "Whole-nav-section playwright walks"; fresh-context (cache disabled) walks only.
- link-resolution RED with >5 broken: fix worst, file `.planning/notes/v0.12.1-link-backlog.md` for rest, do NOT block phase close.
- badges-resolve PARTIAL during GH Pages propagation lag: P2 row; doesn't block; document timing in SUMMARY.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/docs-build/README.md` — docs-build verifier table + conventions.
- `quality/PROTOCOL.md` — cadence routing + verifier-subagent template + waiver protocol.
- `quality/SURPRISES.md` — P60 entries (mkdocs auto-include, hook one-liner, Wave G zero-RED).
- MIGRATE-03 in `.planning/REQUIREMENTS.md` — v0.12.1 carry-forward (auto-sync `docs/badge.json` from `quality/reports/badge.json` on every runner emit).

### P61 — Subjective gates skill + freshness-TTL (added 2026-04-27)

The Quality Gates `subjective` cross-dimension shipped in P61. 3 rubrics catalogued in `quality/catalogs/subjective-rubrics.json`; verifier impls live at `.claude/skills/reposix-quality-review/`.

| Rubric | Implementation skill | Cadence | freshness_ttl |
|---|---|---|---|
| `subjective/cold-reader-hero-clarity` | `doc-clarity-review` (existing global skill) | pre-release | 30d |
| `subjective/install-positioning` | `reposix-quality-review` (P61 ships at `.claude/skills/reposix-quality-review/`) | pre-release | 30d |
| `subjective/headline-numbers-sanity` | `reposix-quality-review` | weekly | 30d |

**reposix-quality-review skill (SUBJ-02 closure).** NEW skill scaffold + 3 rubric prompts + 2 dispatchers (~700 LOC total). The ONLY skill change pre-approved for v0.12.0 per `.planning/research/v0.12.0-open-questions-and-deferrals.md` line 124. Invocation: `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>` / `--all-stale` / `--force`. Path A (Task tool from a Claude session) preferred; Path B (claude -p subprocess) fallback. Path B from the runner subprocess writes a stub artifact (FAIL) until Path A re-runs from a session — the runner-subprocess-overwrites-Path-A-artifact issue is filed as v0.12.1 MIGRATE-03 carry-forward (e).

**Freshness-TTL extension (SUBJ-03 closure).** `quality/runners/run.py` grew `parse_duration` + `is_stale` + STALE-counts-as-NOT-VERIFIED branch + `[STALE]` label (388 LOC final; parse_duration extracted to sibling `_freshness.py` per the Wave B 390-LOC pivot rule). `quality/runners/verdict.py` grew STALE sub-table inside the NOT-VERIFIED rollup. `quality/runners/test_freshness.py` ships 11 pytest tests + `test_freshness_synth.py` 1 end-to-end synthetic regression.

**Pre-release cadence wiring (SUBJ-03 closure).** `.github/workflows/quality-pre-release.yml` (NEW; 75 LOC; triggers on tag-push `v*` + `reposix-cli-v*` + workflow_dispatch). `.github/workflows/release.yml` gains a parallel-execution comment block — hard-gate chaining (release waits for pre-release verdict) is filed as v0.12.1 MIGRATE-03 carry-forward (g) per P56 SURPRISES row 1 GH-Actions cross-workflow `needs:` limitation. Auto-dispatch from CI (Anthropic API auth on GH Actions runner) is v0.12.1 MIGRATE-03 carry-forward (f).

**POLISH-SUBJECTIVE Wave G (broaden-and-deepen).** 3 rubrics graded via Path B (in-session Claude grading; full disclosure header in artifact `dispatched_via=Wave-G-Path-B-in-session`): cold-reader 8 CLEAR; install-positioning 9 CLEAR; headline-numbers 9 CLEAR. ZERO P0/P1 findings — confirmed user-facing prose already CLEAR. 4 P2 polish items (MCP acronym un-glossed; "promisor remote" jargon; docs/index.md target-arch surfacing; "5-line install" approximation) deferred to v0.12.1 docs polish. Catalog rows extended to WAIVED-2026-07-26 with documented evidence + carry-forward citations.

**Recovery patterns** (the regressions this dimension catches):
- `subjective/cold-reader-hero-clarity` RED with score < 7: README.md hero or docs/index.md hero confused the cold reader; read the rubric artifact for friction points + rewrite the offending paragraphs.
- `subjective/install-positioning` RED: install copy regressed (refactor moved package-manager command below `git clone`); restore package-manager-first ordering per CLAUDE.md "Freshness invariants — Install path leads with package manager".
- `subjective/headline-numbers-sanity` RED: a headline number drifted from the benchmarks/ source-of-truth; either update README/docs to match or update benchmarks (whichever is the actual ground truth).

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/subjective/README.md` — dimension home + rubric conventions + pivot rules.
- `.claude/skills/reposix-quality-review/SKILL.md` — skill entry point + 5-step process.
- `quality/PROTOCOL.md` — cadence routing + verifier-subagent template + waiver protocol.
- MIGRATE-03 in `.planning/REQUIREMENTS.md` — v0.12.1 carry-forwards (e/f/g): subjective dispatch-and-preserve runner invariant; auto-dispatch from CI; hard-gate chaining release.yml -> quality-pre-release.yml.

### P62 — Repo-org-gaps cleanup + audit closure (added 2026-04-28)

P62 closes ORG-01 + POLISH-ORG. The forgotten-todo-list document `.planning/research/v0.11.1-repo-organization-gaps.md` (snapshot 2026-04-26) was audited rec-by-rec; the closure record is `quality/reports/audits/repo-org-gaps.md` (99 items; 50 closed-by-existing-gate, 26 closed-by-relocation, 13 closed-by-deletion, 8 out-of-scope; zero open Wave-3 items).

**3 new structure-dimension catalog rows** in `quality/catalogs/freshness-invariants.json` lock the recurrence guards for the gaps that actually moved this phase:

| Row id | What it asserts | Owner hint |
|---|---|---|
| `structure/no-loose-top-level-planning-audits` | no `*MILESTONE-AUDIT*.md` or `SESSION-END-STATE*` at `.planning/` top level | new milestone-audits / session-end-state docs land under `.planning/milestones/audits/` or `.planning/archive/` |
| `structure/no-pre-pivot-doc-stubs` | every `docs/*.md` stub <500 bytes is in `mkdocs.yml` `nav:` / `not_in_nav:` / `redirect_maps` | new top-level `docs/<slug>.md` stubs <500 B — add to redirect map or remove |
| `structure/repo-org-audit-artifact-present` | `quality/reports/audits/repo-org-gaps.md` exists + `scripts/check_repo_org_gaps.py` exit 0 | future repo-org audits land at the same canonical path |

Verifier branches at `quality/gates/structure/freshness-invariants.py` (P62 Wave 3; file grew to 402 lines — helper-module extraction deferred to v0.12.1 unless Wave 6 verifier flags it). Short-lived waivers until 2026-05-15 expire harmlessly after Wave 3 (catalog-first commit pattern).

**Relocations + deletions (P62 Wave 3 commit `8842d48`):**
- `git mv .planning/v0.{9,10}.0-MILESTONE-AUDIT.md .planning/milestones/audits/` (history preserved).
- `git mv .planning/SESSION-END-STATE{.md,.json,-VERDICT.md} .planning/archive/session-end-state/` + new `README.md` naming `quality/PROTOCOL.md` as the supersession path. The §0.8 SESSION-END-STATE framework is superseded by `quality/PROTOCOL.md`.
- `scripts/__pycache__/*.pyc` removed from working tree (`.gitignore:30` already covers `__pycache__/` recursively; files were never tracked).

**SURPRISES.md rotation (P62 Wave 4 commit `2413f13`):** active journal trimmed 302 → 219 lines via P59 precedent. 10 P57+P58 entries (106 lines) archived to `quality/SURPRISES-archive-2026-Q2.md` verbatim. Active retains P59 onward (P59 + P60 + P61 + P62). Banner consolidated.

**Recovery patterns** (the regressions this dimension catches):
- audit doc has unclosed `closed-by-Wave-3-fix` disposition: run `python3 scripts/check_repo_org_gaps.py` for the diagnostic; the script prints which dispositions remain open + the full Wave-3 input list.
- `structure/no-loose-top-level-planning-audits` RED: relocate the new top-level audit doc under `.planning/milestones/audits/` (or under `.planning/archive/` if the doc is historical-only).
- `structure/no-pre-pivot-doc-stubs` RED: a new top-level `docs/<slug>.md` stub <500 B was added without a `mkdocs.yml` mapping — add to `nav:` / `not_in_nav:` / `redirect_maps` or remove.

**Meta-rule (extension of QG-07):** Every audit/cleanup phase MUST commit its closure record under `quality/reports/audits/<topic>.md` with a machine-checkable verifier (`scripts/check_<topic>.py`). Forgotten-todo-list documents are not closed by silent absorption — the closure must be a committed artifact the next agent can grade against.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/reports/audits/repo-org-gaps.md` — the closure audit (99 items; per-gap disposition table + Wave-3 closure record).
- `quality/catalogs/freshness-invariants.json` — 3 P62 rows (lines anchored after the existing 10 rows).
- `quality/gates/structure/README.md` — dimension home with the P62 row table + verifier conventions.
- `quality/gates/structure/freshness-invariants.py` — verifier branches (`verify_no_loose_top_level_planning_audits`, `verify_no_pre_pivot_doc_stubs`, `verify_repo_org_audit_artifact_present`).
- `scripts/check_repo_org_gaps.py` — the audit-completeness verifier (stdlib; exit 0 on PASS; `--json` for machine-readable summary).
- `quality/SURPRISES-archive-2026-Q2.md` — Q2 archive (P56 + P57 + P58 entries).

### P63 — MIGRATE-02 cohesion pass + SIMPLIFY-12 + POLISH-CODE final + v0.12.1 carry-forward (added 2026-04-28)

Closes the v0.12.0 milestone. Cohesion pass per MIGRATE-02:

- **Cross-link audit verifier landed** at `quality/gates/structure/cross-link-audit.py` — walks CLAUDE.md + `quality/PROTOCOL.md` + 8 dim READMEs, asserts every relative path mention exists. Reports at `quality/reports/audits/cross-link-audit-p63.md`. Wave 6 verifier subagent re-runs from zero-context.
- **Per-dim READMEs normalized** to verifier-table + conventions only. Runtime detail (cadence semantics, waiver TTL, verifier-subagent template) lives canonically in `quality/PROTOCOL.md`; dim READMEs cross-link. New `quality/gates/security/README.md` filled the missing security home.
- **POLISH-CODE final state:** `code/cargo-fmt-clean` PASS via direct `cargo fmt --all -- --check` invocation (read-only, ~5s, safe under ONE cargo at a time rule). `code/cargo-test-pass` stays as `ci-job-status` canonical — running `cargo nextest run --workspace` from a verifier violates the memory-budget rule + exceeds the pre-pr 10-min cadence cap; CI is the canonical enforcement venue. Tracked-forward to v0.12.1 MIGRATE-03 for per-row local cargo alternatives.
- **SIMPLIFY-12 audit complete:** `quality/reports/audits/scripts-retirement-p63.md` records every non-hooks `scripts/` file's disposition (5 DELETE / 13 SHIM-WAIVED / 4 KEEP-AS-CANONICAL) with caller-scan + commit SHA. Surviving shims have rows in `quality/catalogs/orphan-scripts.json` asserting shim-shape; verifier at `quality/gates/structure/orphan-scripts-audit.py` mechanizes re-grading.
- **v0.12.1 milestone scaffolds in Wave 5** at `.planning/milestones/v0.12.1-phases/` per the `.planning/milestones/` convention. Carry-forward scope (PERF-*, SEC-*, CROSS-*, MSRV / binstall / latest-pointer fixes, Error::Other 156→144 completion) anchored in 3 stub catalog files (`quality/catalogs/perf-targets.json`, `quality/catalogs/security-gates.json`, `quality/catalogs/cross-platform.json`).

Meta-rule extension (when an owner catches a miss): fix the issue, update CLAUDE.md, AND tag the dimension. The dimension tag routes the next agent to the right `quality/catalogs/` file + `quality/gates/` home.
