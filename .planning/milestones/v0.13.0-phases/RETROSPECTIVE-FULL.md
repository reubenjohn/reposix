# v0.13.0 — DVCS over REST (full retrospective)

Distilled lessons live in `.planning/RETROSPECTIVE.md`. This file holds the
full narrative (What Was Built, Cost Observations, verbose Carry-forward
prose) that was trimmed from RETROSPECTIVE.md per the file-size budget.

**Shipped:** 2026-05-01 (autonomous run; owner-driven tag pending)
**Phases:** 11 (P78–P88) | **Plans:** 14 | **Sessions:** multi-session autonomous

The thesis-shifting milestone: from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains source-of-truth, but a plain-git mirror on GitHub becomes the universal-read surface. Devs `git clone git@github.com:org/repo.git` with vanilla git (no reposix install), get all markdown, edit, commit. Install reposix only to write back; `reposix attach` reconciles the existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

## What Was Built

- **`reposix attach <backend>::<project>` subcommand (P79)** — adopt an existing checkout (vanilla GH-mirror clone, hand-edited tree, prior `reposix init`) and bind it to a SoT backend. Builds the cache from REST, walks the working-tree HEAD, reconciles records by frontmatter `id` (5 cases per architecture-sketch). New `cache_reconciliation` table; audit-row trail via `audit_events_cache op = 'attach_walk'`. Real GH mirror endpoint wired at `reubenjohn/reposix-tokenworld-mirror`.
- **Mirror-lag refs `refs/mirrors/<sot-host>-{head,synced-at}` (P80)** — observability via plain-git refs that vanilla `git fetch` brings along. Three TINY shell verifiers as thin wrappers over `cargo test -p reposix-remote --test mirror_refs <name>` (the layered coverage shape that became the sanctioned house pattern).
- **L1 perf migration (P81)** — replaced full `list_records` walk with `list_changed_since`-based conflict detection in `handle_export`. New `reposix sync --reconcile` cache-desync escape hatch. The success branch's `refresh_for_mirror_head` is no-op-skipped when `files_touched == 0` (eager-resolution patch); perf row asserts ZERO `list_records` calls on the hot push path.
- **Bus remote URL parser + cheap prechecks (P82)** — `reposix::<sot>?mirror=<mirror-url>` form. Precheck A (mirror drift via `ls-remote`) and B (SoT drift via `list_changed_since`) bail before reading stdin. Capability branching: bus URLs omit `stateless-connect` advertisement.
- **Bus remote write fan-out (P83)** — SoT-first algorithm with mirror-best-effort fallback. NEW audit op `helper_push_partial_fail_mirror_lag`. Full fault-injection coverage (SoT mid-stream fail, post-precheck 409, mirror fail). Fixture-fix in P83-02 immunizes shell-hook fault injection from user-global `core.hooksPath`.
- **Webhook-driven mirror sync (P84)** — `.github/workflows/reposix-mirror-sync.yml` template + live copy on `reubenjohn/reposix-tokenworld-mirror`. `--force-with-lease` race protection + first-run handling. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic`.
- **DVCS docs (P85)** — `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md` + troubleshooting matrix entries. Cold-reader pass via `/doc-clarity-review`.
- **Dark-factory third-arm regression (P86)** — `dvcs-third-arm` scenario in `scripts/dark-factory-test.sh`: vanilla-clone + attach + bus-push at the agent-UX surface. 17 asserts. Layered coverage: shell harness for agent UX surface + cargo tests for wire path. TokenWorld arm SUBSTRATE-GAP-DEFERRED.
- **Pre-DVCS hygiene (P78)** — gix bumped from yanked `=0.82.0` to `=0.83.0` (closes upstream gix #29 + #30). Three WAIVED structure rows resolved before TTL. MULTI-SOURCE-WATCH-01 walker schema migration: `source_hashes: Vec<String>` parallel field + per-source AND-compare closes the v0.12.1 P75 path-(a) tradeoff.
- **+2 reservation slots operational (P87, P88)** — P87 drained 5 SURPRISES-INTAKE entries with terminal STATUS + verifier honesty spot-check sampling 5 phases (exceeded the >=3 floor). P88 drained 1 GOOD-TO-HAVES entry (DEFERRED to v0.14.0 with rationale).

## Cost Observations

- Model: claude-opus-4-7[1m] (1M context, milestone-close + several phases)
- Mid-milestone phase-execution model: claude-sonnet-4-5 (per-phase work)
- Sessions: multi-session autonomous (P78–P88 spread across 2026-04-30 → 2026-05-01)
- Notable: per-phase push cadence kept the unpushed-stack from accumulating; pre-push gate caught fmt drift in the discovering phase rather than at session-end. v0.12.1's 115-commit-stack failure mode did not recur. CHANGELOG entry length (~30 non-blank lines) suggests milestones with broader scope should consider a "see RETROSPECTIVE.md" callout to keep CHANGELOG skimmable.

## Carry-forward to v0.14.0 (verbose)

See `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` for the canonical list. The condensed entries below were trimmed from RETROSPECTIVE.md:

- **DVCS-CF-01 — binstall + yanked-gix release substrate** (P84 SURPRISES-INTAKE Entry 5; severity HIGH). Cutting v0.13.x with non-yanked `gix = "=0.83.x"` (P78 already bumped the workspace pin) AND confirming `.github/workflows/release.yml` produces per-target binstall tarballs unblocks both legs of the webhook setup-guide install path. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic` ready for re-measurement once v0.13.x ships.
- **DVCS-CF-02 — extend `reposix-quality bind` to all catalog dimensions** (P88 GOOD-TO-HAVES-01 DEFERRED). Closes the cleaner Principle A provenance story; today every non-`docs-alignment` row carries `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. Operationally tolerable; provenance flag + audit trail intact. ~30-50 lines Rust + tests + cross-dimension schema design.
- **DVCS-CF-03 — L2/L3 cache-desync hardening** (P81 deferral per `architecture-sketch.md` § "Performance subtlety"). L1 ships in v0.13.0 (the `list_changed_since` precheck + the `refresh_for_mirror_head` no-op skip). L2 (background async cache rebuild on detect) and L3 (cache-vs-SoT divergence audit) defer to v0.14.0 alongside the observability dashboards.
- **CLAUDE.md "Subagent delegation rules" sign-posting for cargo-test-as-verifier shape.** The pattern is sanctioned (P80 → P86 trail) but not yet explicitly named in CLAUDE.md as the default for env-propagation-sensitive surfaces. Future planners might benefit from a "before proposing `reposix init`+`git fetch`+`git push` end-to-end shells, check CLAUDE.md § Quality Gates layered-coverage default" callout.

---

# v0.13.0-extension (P89–P97) — full retrospective

Distilled lessons live in `.planning/RETROSPECTIVE.md` § "Milestone:
v0.13.0-extension (P89–P97)". This section holds the verbose narrative + commit
trail trimmed from that distilled section per the file-size budget.

**Shipped:** 2026-07-06 (autonomous close-out drive P92→P97, no-fable regime)
**Phases:** 9 (P89–P97) | **HEAD at close:** `30b4910` on `origin/main`
**Primary sources:** `.planning/CONSULT-DECISIONS.md` (`[SELF]` ledger D-P92..D-P96) ·
`quality/reports/verdicts/p9{4,5,6}/VERDICT.md` · the P89–P97 intake ARCHIVEs.

A post-tag *hardening* extension: the v0.13.0 DVCS surfaces (P78–P88) shipped, but
the milestone-close drive uncovered a DVCS-quality-framework gap, two reproduced
cache-coherence bugs at the L2/L3 boundary, and a self-mutating quality runner. P89–P97
close those, drain the OP-8 +2 slots, and run the OP-9 milestone-close ritual.

## What Was Built (P89–P97)

- **P89–P91 — DVCS quality framework + honesty rules.** Structure-dimension gates
  (`banned-production-tokens.sh`, `deferral-pointer-linter.sh`), the RBF-FW-06..12 honesty
  rules, and the write-once `minted_at` anchor (`91bec9a`) that closes the
  backdated-`last_verified` audit dodge — `_audit_field.validate_row` anchors
  `claim_vs_assertion_audit` on `minted_at` when present.
- **P92 — T4 two-writer ancestry litmus** (`agent-ux/t4-conflict-rebase-ancestry`,
  `858330a`). Prove-before-fix: the HIGH-1 fresh-root/no-ancestry scenario is GREEN against
  current `main` (the root cause landed pre-P92 in `cb630e5`, env-scrub before the bare-cache
  `git config` shell-outs). Two new findings surfaced-and-filed (rebase 3-way `not our ref`;
  Ubuntu-24.04 git 2.43 `stateless-connect` fallback-sentinel) rather than hand-waved.
- **P93 — ADR-010 L2/L3 cache-coherence + `SotPartialFail` recovery.** Two independently
  reproduced bugs, both DP-2 prove-before-fixed then repaired: (a) the fetch-side
  dangling-tree amplifier (RBF-LR-01) — `Cache::sync` built the tree from the full
  `list_records` set while blob-materialization + `oid_map` covered only the
  `list_changed_since` delta → `read_blob UnknownOid` → helper `not our ref`; (b) the
  ghost-`oid_map`-row false `SotPartialFail` (RBF-LR-03, D-P93-01/02) — an upstream-deleted
  record's `oid_map` row was never pruned (`INSERT OR REPLACE`, never `DELETE`), so
  `list_record_ids()` resurrected the dead id and the planner emitted a phantom `Delete` →
  404 → false `helper_push_partial_fail_sot` on every push, forever. **Strategy-1
  (prune-on-sync)** chosen over Strategy-2 (reclassify delete-`NotFound` as success) —
  `272882c` — because it fixes the root cache-incoherence, emits zero phantom outbound
  DELETEs, and has a narrower semantic blast radius; Strategy-2 filed as defense-in-depth.
- **P94 — pagination-truncation prune-safety** ([FABLE] fork). Gate `prune_oid_map` on a
  new `list_records` completeness signal (Fork A) so a truncated listing never deletes live
  rows beyond the cap; 3 verdict-GREEN rows re-minted clean at P96 (`0bdd752`).
- **P95 — marker-footgun doc pass + docs-alignment refresh.** Documented the
  test-name-honesty 6-line lookback window (`3e9b2b2`); re-bound 8 drifted rows (16/17
  targeted) with real recomputed `source_hashes` (`bd18827`). Verdict:
  `quality/reports/verdicts/p95/VERDICT.md` (GREEN).
- **P96 (OP-8 Slot-1) — quality-runner self-mutation fix (keystone) + intake split.** Split
  GRADE from PERSIST (`2359c63`): cadence/gate runs are validate-only (compute status
  in-memory, write gitignored per-row artifacts, still block RED), and only an explicit
  `run.py --cadence <c> --persist` mint writes catalog `status`. Retired the recurring
  `git checkout HEAD -- quality/catalogs/` band-aid (`f19abb6`); catalog-first row
  `structure/catalog-immutable-on-read` (`1a9f2f2`) predates the fix. Also: `source_hashes`
  bind-state keying (`5a72980`), `bind` refreshes `summary.last_walked`, same-second CREATE
  cache-coherence repro (`889c922`), and the SURPRISES/GOOD-TO-HAVES terminal↔active split
  (`31d84e3`, zero row loss). Verdict: `quality/reports/verdicts/p96/VERDICT.md` (GREEN,
  catalogs byte-immutable across the entire grading pass).
- **P97 (OP-8 Slot-2 + milestone-close) — good-to-haves drain + 9th probe.** Drained the
  Slot-2 good-to-haves (doc-XS closes `302e8ec`; docs-alignment-coupled GTH-10/GTH-12 reverted
  `676d3a0`); ran the non-skippable 9th probe `run.py --cadence pre-release-real-backend`
  (`f37f468`, exit 75 → honest NOT-VERIFIED, creds + `REPOSIX_ALLOWED_ORIGINS` unset —
  `last_real_grade=PASS` preserved); wrote this retrospective; milestone verdict is the
  verifier's lane (`quality/reports/verdicts/milestone-v0.13.0/`).

## What Worked (verbose)

- **DP-2 prove-before-fix was the load-bearing discipline of the whole drive.** A
  code-read chain is a hypothesis, not evidence — the drive repeatedly demanded an executed
  repro before any fix. Two payoffs stand out: (1) the `list_changed_since`
  "under-materialization" MEDIUM was downgraded to a RESOLVED false-alarm by an executed
  CREATE-path repro (`same_second_created_record_resolvable_after_delta_sync`), and the
  executor **REJECTED** the intake's sketched "materialize all changed blobs" fix because it
  would break the ARCH-01 lazy `blob:none` invariant — trading a self-healing non-bug for a
  real regression; (2) the ghost-`oid_map` HIGH (raised by a code-reviewer who never
  executed) was CONFIRMED only by driving the real `git-remote-reposix export` path end-to-end
  (`precheck`/`diff`/`write_loop` are `pub(crate)` — only the compiled helper can drive the
  chain), landing a real 404 at the sim's DELETE route.
- **Catalog-first + the validate-only grade/persist split.** GREEN-contract rows predate
  their implementation, so the unbiased verifier grades a row it did not author (P96 keystone
  `1a9f2f2` precedes fix `2359c63`); and the P96 keystone gate snapshotted every catalog byte
  at entry and exit → CATALOGS_UNCHANGED_ACROSS_ENTIRE_VERIFICATION, proving the very fix
  under test.
- **The ownership charter caught wrong instructions.** This retrospective task's own scaffold
  asserted the `minted_at` instance-fix landed at `30b4910` (the file-size waiver enumeration);
  the real instance-fix is `f37f468` (backfilled the missing `minted_at` on
  `agent-ux/cadence-pre-release-real-backend` after the env-skip advanced `last_verified` past
  the P90 cutoff). Verifying against reality before writing caught the discrepancy.
- **Honesty instruments held the line:** snapshot→mint→restore for honest catalog surgery,
  the phantom-green grep as a milestone-honesty gate, and env-gated NOT-VERIFIED that fails
  closed (never skip-counts-as-pass, per PROTOCOL.md OD-2).

## What Was Inefficient (verbose)

- **The self-mutation bug taxed every phase before P96.** Because a read-only pre-push run
  persisted grades, a manual `git checkout HEAD -- quality/catalogs/` workaround recurred
  across P78/P91/P93/P94 (5×) and was never permanently fixed until the grade/persist split.
- **`--persist` has no `--row` filter.** Every legitimate mint re-grades all in-scope rows,
  so subjective/badge rows re-flip off stale artifacts on unrelated mints, forcing repeated
  collateral-restore surgery (SURPRISES:710) — a hazard the upcoming milestone mint inherits.
- **Intake bloat made reads expensive.** The working SURPRISES/GOOD-TO-HAVES corpus exceeded
  180k before the P96 terminal↔active split; even post-split the active corpus (63k/81k) sits
  over the 20k file-size budget under a WAIVED row (expires 2026-08-08).
- **"Safe doc-only XS" is a false category.** A doc edit that carries a bound docs-alignment
  P0 claim is NOT low-risk (GTH-10/GTH-12 had to be reverted, `676d3a0`).
- **Single-cargo + single-push serialized throughput** — the standing VM-OOM tax (one cargo
  invocation machine-wide) and per-phase push cadence bound the drive's wall-clock.

## Patterns Established (verbose)

- **Grade/persist split** — validate-only is the default cadence; catalog `status` writes
  ONLY behind an explicit `--persist` mint. The distinction is "grading run (may persist)" vs
  "gate run (must not)", not a sixth `git checkout` workaround.
- **DP-2 (prove-before-fix) / DP-3 (3-question evidence) in practice + the `[SELF]`
  CONSULT-DECISIONS ledger** as the sub-escalation-threshold decision record (reversible
  internal strategy, in-family with ADR-010, no E2 escalation).
- **Catalog-first GREEN-contract rows predating implementation** — the verifier reads rows it
  did not write.
- **reader-digester / Explore recon** for coordinator context discipline in the no-fable
  close-out regime.
- **Frozen-PASS vigilance** — validate-before-persist + fresh mints; the p94 VERDICT froze a
  transient badge PASS, so the milestone verdict must grade `docs-build/p94-badges` honestly
  NOT-VERIFIED rather than inherit it.

## Key Lessons (verbose)

1. **A metric you generate but don't watch decays silently.** Catalog grades self-mutated for
   phases before anyone named the push-dirtying pattern; the runner's own P91 "Honest callout"
   had confessed the side-effect but mis-framed it as a missing `--dry-run` rather than a
   gate/grade conflation bug.
2. **A coded workaround never converges.** The diff-and-revert / `git checkout` band-aid
   recurred 5× across phases; only SPLITTING the concern (grade vs persist) — not patching the
   symptom a sixth time — converged. Diff-and-revert also erases genuine P0/P1 regressions the
   dirty-check exists to surface.
3. **Frozen/optimistic PASSes are a recurring honesty hazard.** Transient badge flakes and
   stale subjective rubrics both produce lucky-moment PASSes; the antidote is grade-fresh +
   fail-closed-to-NOT-VERIFIED, and never inheriting a prior phase's optimistic grade.
4. **Continuous intake hygiene.** A 180k working intake is debt, not an archive; relocate
   terminal entries promptly so the active drain queue stays cheap to read.
