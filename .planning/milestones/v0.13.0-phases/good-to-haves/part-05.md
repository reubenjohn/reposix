# v0.13.0 GOOD-TO-HAVES — Part 5 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## 2026-07-05 | `verdict.py --phase N` is a pure rollup and does NOT scope the P0/P1 gate to phase-N rows | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P2

**What:** `quality/runners/verdict.py --phase N` reads the FULL catalog rollup and reports
overall RED/GREEN against the global P0/P1 gate — it does not filter or scope that gate to
rows tagged/owned by phase N. Concretely, at the P93 phase-close grading session,
`verdict.py --phase 93` reported RED with "103/112 P0/P1 green", but the 9 red rows mixed
P93's own (at-the-time) ungraded rows together with unrelated, pre-existing stale rows from
OTHER phases (`real-git-push-e2e`, `t4-conflict-rebase-ancestry`, `cargo-binstall-resolves`,
`subjective/dvcs-cold-reader`, `p92-mid-stream-litmus-t1-t4`, etc.). A verifier or executor
skimming the rollup's headline RED could easily misattribute the failure to the phase being
graded, or — in the opposite direction — rubber-stamp a genuine phase-N regression as "not
mine" and dismiss it because the rollup doesn't say which rows belong to N.

**Benefit if done:** a phase-close verifier gets a `--phase N` output that actually answers
"is phase N's own contract green," not "is the whole catalog green as of today" — removing
a class of misdiagnosis (both false-attribution and false-dismissal) at exactly the moment
(phase-close grading) where an accurate signal matters most.

**Acceptance:** `verdict.py --phase N` gains a phase-scoped sub-line (e.g. "phase-N rows:
X/Y green") computed from rows whose `id` or a row-level `phase` field matches N, printed
ALONGSIDE (not replacing) the existing global rollup line. Minimal-viable: derive the
phase-N row set from the existing `pNN-*`/`RBF-*` id-prefix convention already used across
catalogs (no new schema field required); if a durable per-row `phase` field is preferred
instead, land it as a superset covering both this and future phases' id-prefix drift.

**Why deferred:** discovered while grading, not implementing — fixing `verdict.py` itself is
a `quality/runners/`-framework change outside the P93 RED-loop mechanical re-run charter
(mint artifacts, don't touch runner code). Filed for the P94–P97 debt-drain window,
alongside the other `run.py`/`verdict.py` surface-area items already queued there
(GOOD-TO-HAVES `--dry-run` flag, `--row`/`--dimension` scope flags, the recurring
self-mutation bug in SURPRISES-INTAKE).

**Default disposition:** P2 — fold into the same P94–P97 `quality/runners/`-touching debt
window as the sibling `run.py` scope-flag work.

**STATUS:** OPEN

---

## 2026-07-05 | `dark-factory.sh sim` T1-T3 emits a confusing `blocked origin` WARN on git < 2.34 that reads like a failure at first glance | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P3

**What:** On a box whose on-path `git` is below the script's documented `>=2.34` floor
(e.g. 2.25.1), `dark-factory.sh sim`'s T1-T3 leg emits a `WARN git fetch --filter=blob:none
failed with status exit status: 128` plus a raw `error: cannot list issues for import:
blocked origin: http://127.0.0.1:7878/...` / `fatal: Unsupported command` stderr block
during the on-box init fetch attempt — yet the script still exits 0. This is by design: the
sim arm validates config wiring + the recovery-hint message text, not a full end-to-end
fetch (the real fetch is exercised separately, e.g. in a git-2.54 container for T4). But the
raw WARN + `fatal:`/`blocked origin` stderr, un-annotated, reads exactly like a fetch
failure on first glance — a future reader (human or agent) skimming the transcript could
reasonably conclude the gate is broken or that sim connectivity failed, when in fact exit 0
is correct and expected.

**Benefit if done:** a one-line annotation ("expected on git < 2.34 — validating config +
recovery-hint text only, not a live fetch; see T4 container arm for the real fetch") next to
the WARN removes a recurring "is this actually broken?" double-take for anyone reading a
dark-factory-sim transcript on an old-git box, without changing the gate's pass/fail logic.

**Acceptance:** `quality/gates/agent-ux/dark-factory.sh` (sim arm) emits an explanatory note
alongside the WARN when the on-box git fetch fails due to sub-2.34 version detection (it
already detects the version to decide script behavior elsewhere), OR the note is added to
the row's `owner_hint` / a comment near the WARN's emission site so a transcript reader has
the context inline instead of needing to cross-reference this file.

**Why deferred:** cosmetic / documentation-of-intent only — does not change the gate's
correctness or its exit code; discovered while grading (verifier read the transcript), not
implementing. Filed rather than eager-fixed because the RED-loop charter is a mechanical
artifact-minting re-run, not a `quality/gates/` script edit.

**Default disposition:** P3 — pick up alongside other `dark-factory.sh` polish items in the
P94–P97 debt window.

**STATUS:** OPEN

---

## 2026-07-05 | Arm the F-K4b congruence gate for the 5 P93 agent-ux verification artifacts — all carry `asserts_passed: []` | discovered-by: P93 phase-close verifier (unbiased re-verify) | severity: P3

**What:** All five P93 runner-minted verification artifacts (RBF-LR-01/02/04/05 +
D-P92-03) carry `asserts_passed: []`. This is legitimate today — `asserts_congruent` is a
documented no-op when either the expected or actual asserts list is empty
(`_audit_field.py:169-170`) — so these agent-ux mechanical gates inherit the fleet-wide
"exit-0-IS-the-assertion" posture, and the P93 verdict confirmed the gate honesty
line-by-line. But it means the F-K4b per-expected-assert congruence protection is
**dormant** on these five P0 rows: a gate that emitted structured `asserts_passed` entries
would arm F-K4b's real per-assertion congruence check instead of relying on bare exit-code
honesty alone.

**Benefit if done:** teaching the five agent-ux gate wrappers
(`p93-l2-l3-coherence-adr.sh`, `p93-cache-coherence.sh`, `p93-delta-sync-coherence.sh`,
`p93-l1-promise-reconciled.sh`, and the mid-stream litmus T1-T4 wrapper) to emit a
structured `asserts_passed` list (one entry per catalog-row `expected.asserts` item) arms
F-K4b's per-assertion congruence protection on these P0 rows, closing the same class of
"test name lies" gap that `agent-ux/test-name-vs-asserts` already polices for Rust tests.

**Why deferred:** discovered while grading (unbiased phase-close verify), not
implementing — editing the five gate scripts' output-parsing/emission logic is
`quality/gates/agent-ux/`-touching work outside the RED-loop/verify charter, and each
wrapper's assert-list needs a deliberate per-row pass rather than a blind mechanical edit.

**Default disposition:** P3 — fold into the next `quality/gates/agent-ux/`-touching phase
or the P94–P97 debt-drain window, alongside the sibling `verdict.py --phase` scoping item
already filed above.

**STATUS:** OPEN

---

## 2026-07-05 | `.planning/CONSULT-DECISIONS.md` is 25,074 chars, above the 20k soft limit | discovered-by: P94 catalog-first planning lane | severity: P3

**Source:** `.planning/CONSULT-DECISIONS.md` measures 25,074 chars — above the 20k-char
pre-commit WARN threshold (warns, does not block), same window as the already-filed
`raise-list-p90.md` file-bloat item above (24,679 chars). The file is an append-only log
of fable/E2 consult decisions (each entry is decision-ready evidence: chosen fork +
rationale + rejected alternatives + spot-checks), so trimming risks losing the audit trail
a future planner keys off. The most recent entry (the ratified pagination-truncation
prune-safety fork) is what P94 D1 executes against.

**Acceptance:** Fold into the intake-file-bloat split during the P96/P97 milestone-close
window, alongside the sibling `raise-list-p90.md` entry above. Either (a) split
`CONSULT-DECISIONS.md` into per-quarter or per-milestone shards under
`.planning/consult-decisions/` with an index (each under 20k chars), or (b) archive the
already-superseded/closed entries to a `.planning/archive/` companion, keeping only LIVE
decisions in the root file under 20k. Preserve every chosen-fork + rationale verbatim; do
not summarize away the rejected-alternatives detail. Pre-commit WARN clears.

**Default disposition:** P3 — maintainability, no runtime impact; fold into the P96/P97
intake-file-bloat split (sibling of `raise-list-p90.md`).

**STATUS:** OPEN

## 2026-07-05 | GitHub `list_records` → `list_records_complete` delegation is a self-recursion footgun | discovered-by: P94 Finish lane A | severity: P3

**What:** `crates/reposix-github/src/lib.rs::list_records` delegates UP to the
completeness-aware form and drops the flag: `Ok(self.list_records_complete(project).await?.records)`.
GitHub is safe ONLY because it ALSO provides a CONCRETE `list_records_complete` override (the
real pagination loop) right below. But the `BackendConnector` trait's DEFAULT
`list_records_complete` (`crates/reposix-core/src/backend.rs:280`) delegates the OTHER way —
it calls `self.list_records()`. So the two defaults form a delegation cycle broken only by
GitHub's concrete override: if a future edit ever REMOVES GitHub's `list_records_complete`
override (e.g. "the trait default is fine now"), `list_records` → default `list_records_complete`
→ `list_records` → … infinite-loops / stack-overflows at runtime, with no compile-time guard.
The inline comment ("Concrete override below, so no recursion through the trait default")
documents the hazard but does not enforce it.

**Acceptance:** Restructure so the default cannot self-delegate into a live cycle. Options:
(a) invert the direction so `list_records_complete` is the primitive and `list_records` is
the only delegator (make the trait's default `list_records` call `list_records_complete`, and
require backends to implement `list_records_complete` — the opposite of today's default), or
(b) keep the shape but add a `#[test]` / debug-assert that a mock backend using the trait
default for BOTH methods does NOT recurse (a recursion-guard sentinel), or (c) at minimum a
doc-comment cross-link on the core default warning that overriding `list_records` to delegate
to `list_records_complete` requires a concrete `list_records_complete`. Pure hardening — no
current runtime bug.

**Default disposition:** P3 — latent footgun, zero current runtime impact (GitHub's concrete
override is present); fold into a v0.14.0 connector-trait cleanup or the OP-8 good-to-haves
drain.

**STATUS:** OPEN

---

## 2026-07-05 | Split the `doc_alignment.rs` 71k monolith into per-verb modules (bind/walk/status/merge) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**Size:** M (module carve-out + re-export shim; no behavior change)

**Source:** `crates/reposix-quality/src/commands/doc_alignment.rs` is **71,288 chars / 1,716
lines** on HEAD `889c922` — the single largest source file in the workspace, hosting every
`doc-alignment` verb (bind, propose-retire, confirm-retire, mark-missing-test, plan-refresh,
plan-backfill, merge-shards, walk, status) plus the walker's drift state machine. The prose
`≤350`-style caps that pressure `run.py`/`verdict.py` (GOOD-TO-HAVES-06) have no analogue here,
and the file-size gate does NOT catch it: `quality/gates/structure/file-size-limits.sh` excludes
`^crates/.*\.rs$` outright (deferred to a future milestone's crates-source-budget cleanup, per
the gate's own exclusion comment) — so this is not "warn-only," it is currently UNMEASURED. The
monolith makes every walker/bind change a merge-conflict magnet and buries the drift-state logic
(the `source_hashes.is_empty()` false-negative in the sibling SURPRISES entry lives here).

**Acceptance:** carve per-verb modules (`doc_alignment/{bind,walk,status,merge,...}.rs`) behind a
thin `doc_alignment/mod.rs` re-export so callers are untouched; the walker's drift state machine
becomes its own unit. No behavior change; existing `reposix-quality` tests stay green. Pairs with
the eventual removal of the `^crates/.*\.rs$` file-size-gate exclusion (then this file would fail
a real budget).

**Why deferred:** cargo-touching refactor of a load-bearing binary, orthogonal to this no-cargo
hygiene window; best done in a quality-framework phase that already has the crate built.

**Default disposition:** LOW/M — no runtime impact, pure maintainability; do it in the same
window that retires the crates-source file-size-gate exclusion so the split is enforced, not just
performed.

**STATUS:** OPEN

---

## 2026-07-05 | Split `cache_coherence.rs` (23.4k) when the crates-source file-size budget is enforced | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**Size:** S (test-file split by scenario cluster)

**Source:** `crates/reposix-cache/tests/cache_coherence.rs` is **23,415 chars** on HEAD `889c922`
— over the generic `*.md`/`*.rs` 20k progressive-disclosure budget. **Accurate scope note (a
correction to the loose "20k soft-limit waiver expires" framing):** this file is a `crates/**.rs`
path, which `quality/gates/structure/file-size-limits.sh` EXCLUDES entirely (`^crates/.*\.rs$`),
so it is neither flagged nor warned today — the relevant trigger is the future milestone that
RETIRES that crates-source exclusion (the same cleanup GOOD-TO-HAVES-06 and the `doc_alignment.rs`
split above both wait on), not a warn-only waiver expiry. When that budget lands, this test file
(same-second CREATE/UPDATE/DELETE cache-coherence repros, incl.
`same_second_created_record_resolvable_after_delta_sync`) should split by scenario cluster.

**Acceptance:** split into per-scenario test modules (e.g. `cache_coherence/{create,update,delete,
delta_sync}.rs` or split files) each under the source budget, preserving every existing test fn
verbatim; `cargo nextest run -p reposix-cache` stays green.

**Why deferred:** cargo-touching test refactor, no correctness value on its own; only worth doing
alongside the crates-source-budget enforcement so it is checked, not aspirational.

**Default disposition:** LOW/S — cosmetic/maintainability; bundle with the crates-source file-size
budget rollout.

**STATUS:** OPEN

## 2026-07-05 | `catalog-immutable-on-read` gate covers only the `pre-commit` cadence in its real-tree check, not `pre-release`/`pre-push` | discovered-by: P96 phase-close (verdict NOTICED #2 review) | severity: LOW

**Size:** XS (extend one gate's real-tree assert across cadences)

**Source:** `quality/gates/structure/catalog-immutable-on-read.sh` proves the self-mutation fix with
4 asserts — 3 hermetic synthetic-flip cases + 1 real-tree breadth check. The real-tree check runs
`python3 quality/runners/run.py --cadence pre-commit` (validate-only) and asserts zero catalog bytes
change. But the bug it guards actually bit the **`pre-push`** cadence (the `docs-build.json` flip),
and the milestone mint runs the **`pre-release`** cadence — neither is exercised by the gate's
real-tree assert. The P96 verdict NOTICED #2 flagged this: `pre-commit` was chosen to avoid
recursion + double-cargo, and the verifier closed the residual gap MANUALLY with a one-off real
`--cadence pre-push` validate-only run (zero drift). So the byte-immutability invariant is only
*automatically* regression-guarded on `pre-commit`; `pre-release`/`pre-push` rest on the hermetic
synthetic-flip proof.

**Acceptance:** extend the real-tree breadth assert to loop `--cadence` over
`{pre-commit, pre-push, pre-release}` (validate-only, no `--persist`), each asserting zero catalog
byte drift — while PRESERVING the cargo-free / no-recursion property that motivated the original
`pre-commit`-only choice (skip or stub any cargo-shelling row so the gate stays fast + hermetic).
Document the cadence coverage in the gate header.

**Why deferred:** widening a blocking structure gate deserves its own change with a check that the
added cadences don't drag cargo verifiers into the gate's runtime (the reason `pre-commit` was
picked) — not a P96-close rider.

**Default disposition:** XS/LOW — fold into the next `quality/runners`- or `catalog-immutable`-
touching window; pairs with the run.py persist-gate extraction (GOOD-TO-HAVES-06).

**STATUS:** OPEN

---

