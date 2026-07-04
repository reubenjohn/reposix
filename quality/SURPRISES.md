# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0/autonomous-execution-protocol.md` § "SURPRISES.md format": append one line per unexpected obstacle + its one-line resolution. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`.

Anti-bloat: ≤200 lines. When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface".

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

**Archive rotations:** past rotation history retained in git log. The 2026-04-29 prune deleted `quality/SURPRISES-archive-2026-Q2.md` after auditing every archived entry as codified into CLAUDE.md / `quality/PROTOCOL.md` / catalog rows / verifier scripts / v0.12.1 REQUIREMENTS.md carry-forwards. Recoverable via `git show <pre-prune-sha>:quality/SURPRISES-archive-2026-Q2.md` if forensic detail is needed.

---

_Compressed 2026-04-29 from multi-paragraph entries; per-line format per file's own header rule._

2026-04-28 P64 Wave 3: walker writes `summary.last_walked` on every invocation, mutating `doc-alignment.json` even when rows == [] — accepted for v0.12.0; v0.12.1 MIGRATE-03 either moves field to artifact or extends `catalog_dirty()` to ignore it.
2026-04-28 P65: backfill shard 016 wrote 17 rows to live catalog instead of shard catalog (default-path bug) — recovered via jq move + reset; v0.12.1 MIGRATE-03 (j) makes binary refuse default-path mutation when invoked via skill.
2026-04-28 P65: walker waiver scope mismatch — `summary.floor_waiver` only covers `alignment_ratio<floor` BLOCK, not `MISSING_TEST`/`RETIRE_PROPOSED` rows; resolved via separate row-level waiver on `docs-alignment/walk` in `freshness-invariants.json` (TTL 2026-07-31); v0.12.1 should consolidate behind one initial-backfill grace period.
2026-04-28 P65: backfill envelope overshoot (388 rows vs planned 100-200) driven by glossary doc extracting 24 RETIRE_PROPOSED rows (one per term) — under "wildly off" threshold, no halt; future backfills should bias-conservative or filter definitional/glossary docs from the manifest.
2026-04-28 v0.12.1: orchestrator falsely logged a safeguard breach (`confirm-retire` running from agent context) — retracted; owner had run the bulk-confirm script from a real TTY in parallel. Lesson: check git reflog and ask the owner before logging a safeguard breach.
## 2026-07-04 — Quality Convergence: unification decisions (owner mandate OD-4)

Context: pre-P90 quality-convergence session (170-row audit ledger; 6 BLOCKERs
dispositioned same session). Owner directive: cross-cutting simplification
accepting trivial capability loss for major complexity reduction; iterate
until convergence. Decisions below are committed contracts — implementation
commits cite this entry.

### D-CONV-1 — pre-pr cadence gets real CI wiring (was: 61 rows, zero executions)
Evidence: no workflow invokes `run.py --cadence pre-pr`; only the
green-gauntlet.sh shim does (itself invoked by nothing). 61 rows across 7
catalogs carry the tag; PROTOCOL.md:177 promises "CI tier-1 (PR check)".
DECISION: add a `quality-pre-pr` job to ci.yml running
`python3 quality/runners/run.py --cadence pre-pr`; STRIP the pre-pr tag from
rows whose substance ci.yml already hand-wires as dedicated jobs (fmt,
clippy, cargo test, dark-factory, latency-bench) — hand-wired jobs stay for
per-check UI granularity and rust-cache reuse; the run.py job covers the
cheap mechanical remainder. Kill `code/cargo-fmt-clean` (exact duplicate of
`code/cargo-fmt-check`, QL-031) and fix both rows' phantom
`scripts/hooks/pre-push` source cites. A tag that never fires is worse than
no tag: it teaches agents the catalog lies.

### D-CONV-2 — verdict exit semantics: 3-state honest contract (was: weekly never green)
Evidence: verdict.py computes red/yellow/brightgreen but exits nonzero on
yellow while its docstring documents a binary; quality-weekly.yml has no
tolerance → the ONLY workflow consuming yellow has never been green. Root
yellow: 2 P2 kind:manual rows (benchmark-claim/8ms-cached-read,
89.1-percent-token-reduction) with verifier.script null — structurally
unable to PASS.
DECISION: (i) verdict.py grows `--fail-on {red,yellow}` (default yellow —
strict by default, callers opt into tolerance); docstring documents all
three states. (ii) quality-weekly.yml passes `--fail-on red`: yellow renders
as a yellow badge + alerting, not a failed job crying wolf. (iii) badge JSON
text must state the true counts (no "N/N GREEN" text on a yellow badge —
QL-178). (iv) The two headline-number rows are NOT softened: verifying them
mechanically is exactly "CI-verified headline numbers" from the
launch-readiness milestone (OD-4 §3); routed to intake, tracked, and the
badge stays honestly yellow until then.

### D-CONV-3 — scripts/ collapse: delete shims + dead one-shots; inverse registry gate
Evidence: 33 files in scripts/; ~13 referenced by any live surface. 6 shims
exec quality/gates/* equivalents; ~14 one-shot phase scripts (p56/p74/p82/
p83/w4/w7 era) invoked by nothing; near-duplicate pair check-quality-catalogs.py
vs check_quality_catalogs.py; orphan-scripts.json uses the retired scalar
`cadence` key, is skipped by verdict.py, and its auditor iterates only
registered rows (structurally blind to new orphans).
DECISION: delete the shims (updating CLAUDE.md/docs invocations to canonical
quality/gates paths) and the dead one-shots (git history is the archive);
migrations go under scripts/migrations/; keep the duplicate-pair member that
matches current schema, fix its enums (SURPRISES-14), delete the other;
convert orphan-scripts.json rows to the `cadences` list schema; mint
`structure/scripts-registry-complete` — an INVERSE scan (every file in
scripts/ excl. migrations/ must have a registry row or live wiring) so new
orphans fail loud. Capability lost: none (shims were exec one-liners; the
one-shots already ran).

### D-CONV-4 — cred-hygiene: two-layer scanning (committed gitleaks + grep fallback)
Evidence: grep gate covers 5 prefixes; known blind spots (AWS, Slack,
OpenAI, PEM). gitleaks 8.30.1 exists on the owner's machine only — its
"Secret scan: clean" is invisible to CI and other contributors; no
.gitleaks.toml committed. The AIza incident was exactly this asymmetry.
DECISION: commit .gitleaks.toml (fixture allowlist; runtime-assembled test
keys stay invisible by construction) + a pinned gitleaks CI job; extend the
grep gate with AWS AKIA/Slack xox/OpenAI sk-/PEM patterns as the zero-dep
local layer. The grep gate stays because a committed hook must not
hard-depend on an uninstalled binary; CI provides the full-ruleset backstop.

### D-CONV-5 — doc-alignment stabilization (partially shipped this session)
Shipped already: bind same-file cite relocation (5da5a68 — phantom-cite
accumulation was structural); per-row waiver verb (af11847 — time-boxed,
loud, tracked; MISSING_TEST had no honest escape valve and blocked ALL
pushes on an unfixable-today claim).
DECISION (remainder): walk verb skips catalog save when only `last_walked`
changed (mirrors the runner's catalog_dirty guard; kills the
telemetry-tick commit churn — 75 commits touch the 405KB catalog).
Content-anchored (non-positional) citations are real surgery → intake with
sketch, not now. Cross-file cite retarget verb: filed GTH-03(a).

### D-CONV-6 — test-pre-push.sh: guard, don't re-wire
Evidence: ledger claimed "runs never"; REFUTED — ci.yml:68 runs it in the
test job. Real gap: no dirty-tree guard while its cleanup trap runs
`git reset --hard` unconditionally.
DECISION: add an abort-if-dirty guard at entry. No catalog row — a row
duplicating existing CI wiring is bookkeeping, not safety.

### D-CONV-7 — CLAUDE.md compaction via progressive disclosure (39,742B → target ≤35k)
DECISION: real-backend env blocks → pointer to docs/reference/
testing-targets.md (which CLAUDE.md already cites before duplicating);
threat-model table → 3-line summary + pointer to
docs/how-it-works/trust-model.md; build commands dedupe with CONTRIBUTING;
+2-phase practice long-form → .planning/PRACTICES.md with a 5-line summary
in place; shim references updated per D-CONV-3. FOLD IN: the ownership
charter (owner directive 2026-07-03) into § Subagent delegation rules.
Net: every removed block gains a named home that was already the
authoritative source.

### D-CONV-8 — journal revival (this entry)
quality/SURPRISES.md was dead since 2026-04-29 while PROTOCOL.md:19 calls it
required reading (QL-042). This entry revives it; per-milestone
SURPRISES-INTAKE.md remains the intake surface for phase-scoped findings,
and PROTOCOL.md gets a cross-reference clarifying the split.

2026-07-04 P90: F-K4b asserts-congruence shipped GATED ON minted_at (new-regime
rows only), not "both lists non-empty" — legacy prose asserts would false-RED
under any token threshold; congruence is armed-but-dormant on legacy rows until
P95 RBF-D-06 migrates them (D90-05 split).

2026-07-04 P90: agent-ux/milestone-adversarial-pass was minted with cadence
[pre-release-real-backend] although its verifier is a creds-free unittest
wrapper — under the env gate it could never honestly flip PASS. Re-cadenced to
[pre-push, pre-pr] by coordinator (e968d36); the milestone-close GREEN-block is
enforced by the verdict.py --milestone hook, not the row's cadence.

2026-07-04 P90: rustfmt relocates over-length trailing comments into fn bodies,
which would have silently broken test-name-honesty marker detection — markers
seeded as standalone comment lines above each fn instead (90-03).

2026-07-04 P90: R2's "2 dangling security verifier scripts" was a
substring-filter miss — comprehensive tests already existed (http_allowlist.rs,
audit_schema.rs, audit_is_append_only.rs); real wrapper scripts written at
quality/gates/security/ (e702822), exit codes to be confirmed by P92.

2026-07-04 P90: release/cargo-binstall-resolves' planned ~10-LOC pkg-url fix
was already shipped pre-P90 (33dd41f, QL-003) — waiver cleared without re-work
per the D90-11 already-landed pattern.
