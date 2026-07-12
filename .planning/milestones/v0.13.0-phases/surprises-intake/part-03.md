# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 3 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-05 | `quality/gates/docs-build/mkdocs-strict.sh` under-reports broken internal anchors (swallowed at INFO log level) | discovered-by: P93 Wave 2a executor | severity: MEDIUM

**What:** `quality/gates/docs-build/mkdocs-strict.sh` runs `mkdocs build --strict` and
then greps the build log + rendered HTML for the literal string `"Syntax error in
text"` (the mermaid HTML-entity failure mode) — but does nothing to catch broken
internal anchor/fragment links (e.g. `[text](page.md#stale-heading-slug)`). mkdocs's own
link-validation machinery logs anchor-resolution misses at `INFO` level, not
`WARNING`, so `mkdocs build --strict` (which only promotes `WARNING`-and-above to a
build failure) does not fail on them and the wrapper script's own grep never looks for
them either. Net effect: the gate is named `mkdocs-strict.sh` and is wired into
`pre-push`/`pre-pr` as the docs-build authority, but a broken `#anchor` fragment inside
`docs/**` can land on `main` with a fully GREEN `docs-build` dimension — the strict gate
silently under-reports exactly the class of link rot it implies it catches.

**Why out-of-scope for P93 (Wave 2a):** confirming and fixing this needs (a) a
reproduction — a synthetic doc with a genuinely broken internal anchor, run through the
real `mkdocs build --strict` to confirm the INFO-vs-WARNING behavior empirically rather
than from documentation of mkdocs's log-level defaults, and (b) a considered change to
either the script (parse `mkdocs build`'s INFO-level output for anchor misses and fail
on them) or the mkdocs config (a plugin/log-level override that promotes anchor-miss
INFO records to WARNING so `--strict` already catches them). Both are `docs-build`-gate
surgery requiring a real `mkdocs build` run to verify the fix actually changes the
gate's behavior — orthogonal to Wave 2a's push-unblock + de-risk charter, and this
executor was the sole cargo/tree-writer for a different, narrower fix (Task A).

**Sketched resolution:** Promote broken-anchor detection to a real FAIL (preferred: grep
the build log for mkdocs's own anchor-miss message pattern — e.g. text containing
"contains a link to ... which is not found" or the specific phrasing mkdocs emits for
unresolved fragments — and `exit` non-zero when found, mirroring the existing
`Syntax error in text` grep pattern already in the script) or, at minimum, a `WARN` line
the runner surfaces in its summary output rather than a fully silent INFO log line
nobody reads. Add a synthetic-fixture regression test (a throwaway doc page with a
`[text](other.md#does-not-exist)` link) proving the tightened gate actually catches it,
mirroring the mermaid-regression fixture pattern already used for POLISH-03.

**Default disposition:** MEDIUM — fold into the P94–P97 debt-drain window or the next
`docs-build`-gate-touching phase; natural pairing with the already-filed `badges-resolve`
flake investigation (same dimension, same debt window).

**STATUS:** OPEN

---

## 2026-07-05 | Pagination-truncation safety of sync's `prune_oid_map` — a truncated `list_records()` can DELETE oid_map rows for LIVE records beyond the cap | discovered-by: P93 DP-2 REOPEN re-review (relayed via coordinator, independently re-verified) | severity: HIGH

**What:** D-P93-02's shipped fix (`meta::prune_oid_map`, commit `272882c`/`e246e84`)
DELETEs `oid_map` rows whose `issue_id` is absent from a `keep_ids` set built from
`self.backend.list_records(&self.project)`. But `list_records()` on the GitHub, JIRA, and
Confluence connectors can silently return a **truncated** `Ok(partial_list)` at a
pagination/size cap (`github/lib.rs` `MAX_ISSUES_PER_LIST=500` / `MAX_RAW_ITEMS_PER_LIST`;
`jira/lib.rs` non-strict `list_issues_impl`; the Confluence equivalent) — the caller has no
way to distinguish "the project only has 40 records" from "the project has 4000 records and
we truncated at 500." Feeding a truncated `keep_ids` set into the prune's DELETE wipes
`oid_map` rows for **live records that exist beyond the cap** — a real record now looks
ghost/deleted to `Cache::list_record_ids()`, and it recurs on EVERY sync, not once. The
sim backend (the project default per OP-1) never truncates, so every sim-run gate —
including all of P93's own GREEN test runs — is structurally blind to this. Completeness is
known-but-DROPPED inside all three real connectors: `BackendConnector::list_records`
returns a bare `Result<Vec<Record>>` with no completeness/has-more signal in its type.
Before `272882c` a truncated list only under-populated the working tree (an accepted,
documented HARD-02 tradeoff); the prune fix turned that same truncation into an active
data-loss operation. No existing test guards this (all P93 tests are sim-backed with small
record counts). Full analysis + a 4-option decision tree already drafted:
`.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md` §4-6.

**Why out-of-scope for P93:** P93's charter (mint the missing verification artifacts +
close the phase) is not the place to make an architectural connector-contract decision.
The safe fix genuinely forks: either (a) an **E2 connector-contract change** — add a
completeness/`has_more` signal to `BackendConnector::list_records`'s return type (or a
sibling method) so the prune can be gated on "listing is known-complete" — or (b) a
**truncation-safe prune redesign** that restricts pruning to a dedicated full-paginated
reconcile path (e.g. `reposix sync --reconcile`, which already forces a full rebuild) and
skips pruning on the normal delta path entirely, or reverts to the pre-`272882c` Strategy 2
(reclassify delete-time `NotFound` as idempotent success — already filed separately above
as a deliberate, NOT-chosen defense-in-depth alternative). Both forks are real API-surface /
architectural decisions (Rule 4 territory), not a same-phase mechanical fix.

**Sketched resolution:** P94 should (1) build a mock/capped-backend test proving the data-
loss reproduction (not just a code-read assertion) and check whether any connector already
exposes a completeness signal a truncation-safe fix could key off, then (2) package an E2
consult with the four options already sketched in the RELIEF-HANDOFF (§6 step 2): **A** —
add a completeness signal to the `list_records` contract and gate the prune on it; **B** —
revert `272882c`, ship Strategy 2 instead; **C** — restrict the prune to the dedicated
full-paginated reconcile path only; **D** — a cap-count heuristic that skips the prune when
`keep_ids.len()` is suspiciously close to a known cap. Plan for an E2 consult in P94 rather
than self-deciding.

**Default disposition:** HIGH — real data-loss hazard against live upstream records once a
real-backend project exceeds its connector's pagination cap; P94 (real-backend frictions)
is the natural next phase to prove + package this for an E2 decision.

**STATUS:** OPEN

---

## 2026-07-05 | ROADMAP.md § "Phase 94"–"Phase 97" prose is STALE/orphaned vs the LIVE STATE.md cursor | discovered-by: P94 catalog-first planning lane | severity: MEDIUM

**What:** `.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 94: Bus-push
compatibility with documented mirror setup (Cluster D)" (RBF-C-01..07) and its
downstream P95–P97 prose describe work that no longer matches what those phases deliver.
STATE.md frontmatter — the LIVE machine-readable cursor — says `next_phase: P94 …
real-backend frictions (pagination-truncation E2-fork + git-2.43 fallback-sentinel)`. The
bus-push / mirror-setup / Cluster-D work the ROADMAP prose describes already shipped in
P82–P86. A future planner grepping the ROADMAP for "Phase 94" would plan against
orphaned prose (RBF-C-* requirement IDs that no longer map to P94's true scope). P94's
PLAN.md carries an explicit `<scope-correction>` block flagging this so THIS phase does
not mis-execute, but the ROADMAP itself is still uncorrected.

**Why deferred (not fixed here):** the P94 catalog-first lane is a SPEC/mint lane
(markdown + catalog JSON only, no re-authoring of milestone ROADMAP prose). Reconciling
the ROADMAP's P94–P97 phase descriptions against the delivered reality is a milestone-close
docs-reconciliation task (freshness/structure dimension), which is exactly what the P96/P97
absorption + milestone-close slots exist for. Fixing it mid-P94 would be scope-creep on a
mint lane.

**Sketched resolution:** During the P96/P97 milestone-close docs reconciliation, rewrite
ROADMAP.md § Phase 94–97 to reflect delivered scope (P94 = real-backend frictions:
pagination-prune fix + git-2.43 fallback-sentinel + badge determination + freshness
sweep), retire or remap the orphaned RBF-C-* requirement IDs (confirm they were satisfied
in P82–P86 or move them to their true home), and reconcile the P95–P97 prose against
STATE. Cross-check REQUIREMENTS.md traceability so no RBF-C-* row points at a phase that
never touched it.

**Default disposition:** MEDIUM — no runtime impact, but a real mis-direction hazard for
the next planner; route to the P96/P97 milestone-close docs reconciliation (freshness /
structure dimension).

**STATUS:** OPEN

---

## 2026-07-05 | STATE.md frontmatter has no strict-YAML parseability guard — a bare `: ` regresses it silently | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**What:** `.planning/STATE.md` opens with a `---`-fenced YAML frontmatter block that is the
LIVE machine-readable cursor (`status`, `last_updated`, `workstreams.workstream_a.next_phase`,
`phases_completed`) consumed by `gsd-sdk` state handlers and by ad-hoc probes. Nothing in the
pre-commit / pre-push gate set asserts that block still parses as strict YAML, so a hand-edit
that introduces a bare `: ` (unquoted colon-space inside a scalar), a tab, or a mis-indented
key silently produces an unparseable block — the failure is invisible until a downstream
`yaml.safe_load` consumer chokes mid-session. This class of breakage has bitten before in the
P94 era (a bare `: ` around `eea309f`, since repaired). **Verified against reality this window:**
STATE.md frontmatter parses CLEAN on HEAD `889c922` (`status: executing-p95-post-close-drain`,
`next_phase: P96`, `phases_completed: 18`) — so this is a PREVENTIVE guard against silent
regression, not a live break.

**Why out-of-scope for P96 Wave 3a:** this window is planning-artifact hygiene (no new gate
wiring / catalog rows — those are Wave 3b + the verifier's territory, and adding a gate touches
`quality/`). Filing the guard sketch keeps the footgun visible for the next `quality/gates/`
window.

**Sketched resolution:** add a tiny `scripts/check-state-yaml.py` (or a `structure`-dimension
pre-commit row) that `yaml.safe_load`s the STATE.md frontmatter block and exits non-zero on any
parse error, naming the offending line. Cheap, deterministic, no cargo. Pairs naturally with the
STATE-vs-ROADMAP staleness entry (`ROADMAP.md § Phase 94–97 prose is STALE` above) — one is
semantic drift, this is syntactic breakage; both harden the single most load-bearing planning
cursor.

**STATUS:** OPEN

---

## 2026-07-05 | Committed catalog `status` lags the live grade between explicit `--persist` mints (by P96 design) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**What:** The P96 grade/persist split (D-P96-01 / CONSULT-DECISIONS `36dad20`) makes a bare
`run.py --cadence <c>` validate-only: it computes status in-memory + writes per-row artifacts
under `quality/reports/verifications/` (gitignored) but does NOT call `save_catalog`. This is
the correct fix for the recurring self-mutation bug. The DESIGNED consequence: any consumer that
reads the COMMITTED catalog `status` field — `verdict.py`'s badge rollup, dashboards, and
load-time phantom-green checks — sees the LAST-MINTED value, not the live grade, until the next
explicit `--persist` mint. Between mints the committed status can be stale relative to what the
runner just measured (related to the `Catalog-freshness sweep needed` entry above, which
observed the same staleness pre-P96 for a different root cause).

**Why out-of-scope for P96 Wave 3a:** filing only — the operational rule below is a runner-docs
+ milestone-close-checklist note, and this window does not touch `quality/` runners or catalogs.

**Sketched resolution:** (a) a one-line note in the runner docs (`quality/PROTOCOL.md` runner
section) stating that committed `status` is authoritative only as of the last `--persist` mint;
(b) an operational rule — milestone-close MUST run an explicit `run.py --cadence <c> --persist`
mint BEFORE reading the milestone verdict / regenerating badges, so the committed status the
verdict rolls up is fresh. LOW: no correctness hazard (gate integrity is preserved in-memory);
purely a "don't trust a stale committed badge between mints" reporting-freshness note.

**STATUS:** OPEN

---

## 2026-07-05 | `--persist` mint path should refuse to write a row it would reject at load (write-path load-refusal hardening) | discovered-by: P96 Wave 3a (residual split out of the D-P96-01 self-mutation fix) | severity: MEDIUM

**What:** The P96 grade/persist split (RESOLVED entry `Recurring quality-runner self-mutation
bug` above) fixed core requirement #2 (cadence runs never persist phase-row grades). Its named
residual — requirement #1, the `--persist` MINT path should REFUSE to write a row it would itself
reject at LOAD (e.g. a `minted_at`-less legacy row that `_audit_field.validate_row` would fail on
load) — is still OPEN and is filed here as its own item (the RESOLVED entry explicitly invited
"file as its own item if pursued"). Filed standalone so it stays visible in the working intake
rather than buried inside a now-terminal entry.

**Why out-of-scope for P96 Wave 3a:** this is a `quality/runners/run.py` write-path code change
(add a pre-`save_catalog` validation pass) with its own test obligation — a runner-touching fix,
orthogonal to this no-cargo hygiene window.

**Sketched resolution:** in the `--persist` branch, before `save_catalog`, run each to-be-written
row through the same `validate_row` predicate the loader applies; abort the mint (loud, naming
the offending row + reason) rather than persisting a row that the very next load would reject.
Closes the "mint writes an un-loadable row" asymmetry. Add a regression: `run.py --cadence <c>
--persist` over a catalog whose in-memory grade would produce a `minted_at`-less row must exit
non-zero and leave the file byte-identical. Route to the next `run.py`-touching quality-framework
window (P97 or v0.14.0).

**STATUS:** OPEN

---

## 2026-07-05 | `--persist` mint re-flips `subjective/*` rows off a STALE rubric artifact on every mint (pre-release cadence collateral churn) | discovered-by: P96 phase-close (post-close drain / verdict NOTICED review) | severity: MEDIUM

**What:** The P96 grade/persist split fixed the *validate-only* leak — cadence runs no longer
persist status flips. But the legitimate `--persist` MINT path still re-grades every in-scope row
in-memory, INCLUDING the `subjective`-kind rows (`subjective/dvcs-cold-reader` and its `pre-release`
siblings) whose status comes from a subagent RUBRIC artifact, not a mechanical verifier. That rubric
artifact is only refreshed by an explicit `/reposix-quality-review` dispatch (30-day TTL), so every
UNRELATED `--persist` mint recomputes the subjective row off the STALE artifact, flips its status,
and dirties `subjective-rubrics.json` with a spurious change the mint author must then hand-restore.
This bit the P96 mint and — left unfixed — will bite the **upcoming P97 milestone mint**: the
non-skippable `pre-release-real-backend` 9th probe runs `--persist` in exactly the cadence these
subjective rows live in. Distinct from the RESOLVED self-mutation bug (that was the validate-only
path) and from the `--persist` load-refusal entry above (that is about un-loadable `minted_at`-less
rows).

**Why out-of-scope for P96:** the clean fix is a `quality/runners/run.py` change (drop subjective
rows from the `pre-release` mint scope, OR make `--persist` treat manual/subagent-graded rows as
no-ops that preserve their prior status) carrying its own test obligation — a runner-touching change
orthogonal to the P96 no-cargo hygiene window, which deliberately left the mint path untouched beyond
the grade/persist split.

**Sketched resolution:** make `--persist` a **no-op for `kind: subjective`/`kind: manual` rows** —
never overwrite a subagent-graded status/`last_verified` from a mechanical mint; only the
rubric-dispatch path (which actually re-ran the rubric) may write them. Alternatively drop the
subjective rows from `pre-release` cadence membership so the milestone mint never touches them. Pairs
with GOOD-TO-HAVES-03's per-row `--row` filter (a scoped mint avoids fanning across subjective rows
at all). **Explicit P97 note:** until this lands, P97's milestone `--persist` mint MUST restore the
subjective-row collateral (git-checkout the spurious flips on `subjective-rubrics.json`) as a known,
expected step — do NOT let it ride into the tagged tree.

**STATUS:** OPEN — P97 Wave A reconciliation, 2026-07-05: CONFIRMED KEPT OPEN. The clean fix is a
`run.py` change (runner-touching, cargo) → **DEFERRED-v0.14.0**. **Load-bearing hand-off to Wave B:
the P97 milestone `--persist` mint MUST restore the `subjective-rubrics.json` collateral** —
git-checkout the spurious `subjective/*` status flips off the STALE (unrefreshed) rubric artifact so
they do NOT ride into the tagged tree. Wave A is planning-only and does not run the mint; this note
is the explicit instruction for whichever agent runs the Wave-B 9th-probe `--persist`.

---

## S-260706-rbf-01 — ADR-010 Consequences item-4 note is itself stale (LOW / cosmetic)

**Found during:** quick 260706-rbf (RBF-LR-03 known-limitation docs).
**Severity:** low / cosmetic.
**Issue:** `docs/decisions/010-l2-l3-cache-coherence.md` Consequences item 4 (~line 319) says
`docs/guides/troubleshooting.md:352` points at a dvcs-topology "Out of scope" anchor and says L3
"defers to v0.14.0" — but troubleshooting.md:352 now correctly reads "L3 … is shipped … Only L2 …
remains deferred." The P93 fix wave already fixed that doc line; the ADR's item-4 note now describes
a defect that no longer exists (the note is stale, not the doc).
**Sketch:** trim/annotate the item-4 note to reflect that the troubleshooting cross-ref was fixed.
NOT eager-fixed: editing a ratified ADR's Consequences section for a cosmetic staleness risks
rewriting decision history; belongs in an OP-8 drain, not a docs quick.

