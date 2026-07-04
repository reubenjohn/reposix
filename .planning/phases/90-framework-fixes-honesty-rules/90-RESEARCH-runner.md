# P90 RESEARCH — Runner/validator internals (LANE R1)

**Scope:** implementation-ready design for the runner/validator changes in
P90 (RBF-FW-06/07/08/12 + cross-AI intake H2/H4/M6/M7/M8). Read-only research;
no code changed. All claims cited to `file:line`.

**Load-bearing size facts (measured 2026-07-04):** `run.py` is **429 lines**
(already OVER the ≤350 anti-bloat cap the header + `_freshness.py:4-7` cite);
`verdict.py` is **367 lines** (under the ≤400 cap, ~33 lines of headroom);
`_audit_field.py` 75 lines; `_realbackend.py` 98 lines. **Design consequence:**
any non-trivial logic added to `run.py` MUST land in a sibling helper module
(the `_freshness`/`_realbackend`/`_audit_field` precedent), never inline in
`run.py`. `verdict.py` can absorb a ~25-line reader but nothing larger.

**Catalog census (508 rows across 12 non-allowlist catalogs, script
`scratchpad/audit_catalog.py`):** kinds = mechanical 91, container 8,
asset-exists 6, subagent-graded 5, shell-subprocess 2, manual 3, **None
(docs-alignment) 393**. Zero rows currently carry `coverage_kind` or
`minted_at`.

---

## 1. CURRENT BEHAVIOR

### (a) Verifier-not-found branch — what it preserves and why

`run.py:236-255`. After the skip/WAIVED/STALE short-circuits, the runner
resolves `row.verifier.script` (`run.py:236-237`) and if the path is falsy
or absent on disk (`run.py:238`) it writes an artifact with
`exit_code: None` and `error: "verifier not found at {script_rel}"`
(`run.py:239-246`). The status decision is the load-bearing line:

```python
# run.py:252-254
if row.get("status") not in ("PASS", "FAIL", "PARTIAL"):
    row["status"] = "NOT-VERIFIED"
```

So a row already at PASS/FAIL/PARTIAL **keeps its status**; only NOT-VERIFIED/
WAIVED rows fall through to NOT-VERIFIED. The rationale comment (`run.py:249-251`):
"Don't flip from PASS->NOT-VERIFIED on a missing verifier — that would be a
regression-on-deploy disguised as runner output." This is the P57/v0.12.0
behavior (landed dd458bd; cross-AI H4 at `89-CROSS-AI-REVIEW.md:22` and intake
`SURPRISES-INTAKE.md:258`). The dishonest-GREEN channel: delete/typo a
verifier path and a PASS row stays green forever.

**Empirical blast radius (I actually checked every row):** exactly **5**
catalog rows name a `verifier.script` that does not exist on disk today, and
**all 5 are `status: WAIVED`** — NONE is PASS/FAIL/PARTIAL:

| catalog | row id | status | blast | cadences |
|---|---|---|---|---|
| cross-platform.json | `cross-platform/windows-2022-rehearsal` | WAIVED | P1 | pre-release |
| cross-platform.json | `cross-platform/macos-14-rehearsal` | WAIVED | P1 | pre-release |
| perf-targets.json | `perf/headline-numbers-cross-check` | WAIVED | P2 | weekly |
| security-gates.json | `security/allowlist-enforcement` | WAIVED | P0 | pre-pr |
| security-gates.json | `security/audit-immutability` | WAIVED | P0 | pre-pr |

Because `waiver_active()` (`run.py:197`) short-circuits to WAIVED BEFORE the
verifier-not-found branch is ever reached, these 5 never even hit line 238
while their waivers hold. **Conclusion for RBF-FW-07a design: flipping the
branch to unconditional NOT-VERIFIED flips ZERO currently-green rows today.**
The P57 blast-radius fear is empirically moot for the live catalog — the
"missing script + PASS" combination does not exist. (Caveat: the 2026-07-26
waiver cliff — `SURPRISES-INTAKE.md` entry 2026-07-03 11:00 — will expire
`security/*` and `cross-platform/*`; once un-waived they reach the branch with
status WAIVED, which is already `not in (PASS,FAIL,PARTIAL)` → NOT-VERIFIED
under BOTH old and new logic. Still no behavior delta.)

### (b) pre-release-real-backend skip path + the M8 demote-and-persist repro

Skip decision: `_realbackend.is_skipped(row, env)` (`_realbackend.py:57-64`) —
true iff the row is tagged `pre-release-real-backend` AND (no non-loopback
origin OR no complete cred set). `run.py:183-194` handles the skip: writes a
`skipped_real_backend: True` artifact and sets `row["status"] = "NOT-VERIFIED"`,
`row["_skipped_real_backend"] = True` (transient), `row["last_verified"] =
artifact["ts"]`.

The M8 demote-and-persist bug (`89-CROSS-AI-REVIEW.md:26`,
`SURPRISES-INTAKE.md` 2026-07-04 05:10 independent leg):

1. `run_row` sets `status = NOT-VERIFIED` on the skip (`run.py:191`).
2. In `main()`, the same-status rollback (`run.py:408-414`) only restores
   `last_verified` when `row.get("status") == orig_status_by_id[rid]`. If the
   catalog's prior status was **PASS**, the skip changed it to NOT-VERIFIED,
   so the statuses DIFFER → rollback does NOT fire → the demoted status stays.
3. `catalog_dirty(original, data)` (`run.py:123-130`) compares status maps;
   PASS→NOT-VERIFIED is a status change → returns True → `save_catalog`
   (`run.py:415-416`) **persists the demotion to the committed catalog**.

So a cred-less `--cadence pre-release-real-backend` re-run rewrites ground
truth. There is no `error`/`why` marker distinguishing "skipped" from a real
regression beyond the transient `_skipped_real_backend` flag that is stripped
at `run.py:410` before persistence — the persisted catalog shows only
`status: NOT-VERIFIED`. This is exactly RBF-FW-07b's remit.

### (c) status/last_verified persistence + catalog_dirty optimization

`catalog_dirty()` (`run.py:123-130`): returns True **iff some row's `status`
changed**; timestamp-only churn is deliberately not a semantic change (docstring
`run.py:124-127`). `main()` deep-copies the catalog via JSON round-trip
(`run.py:375`) so it can diff pre/post, records `orig_status_by_id` +
`orig_lv_by_id` (`run.py:382-383`), and after grading rolls back `last_verified`
for every unchanged-status row (`run.py:408-414`) then persists only if dirty
(`run.py:415-416`). The consequence flagged by 89-07 (`SURPRISES-INTAKE.md`
2026-07-03 21:35 HIGH): a long-PASSing row legitimately keeps
`last_verified: null` forever, because the flip-run's timestamp is never
persisted. Five such null-`last_verified` non-docs-alignment rows exist (named
in that intake entry) and constrain the minted_at design below.

### (d) `_audit_field.validate_row` — date-cutoff anchor, docs-alignment exemption, OD-2 waiver rejection

`_audit_field.py:35-67`. Anchor logic (`_audit_field.py:39-42`):

```python
lv = row.get("last_verified")
cutoff = parse_rfc3339("2026-05-08T00:00:00Z")   # CUTOFF_ISO, line 18
is_new = lv is None or parse_rfc3339(lv) >= cutoff
```

`is_new` rows (null OR ≥ cutoff) MUST carry `claim_vs_assertion_audit` ≥50
chars (`_audit_field.py:42-50`) else `SystemExit`. **The anchor is the mutable
`last_verified` field** — this is the H2 backdate hole
(`89-CROSS-AI-REVIEW.md:20`): a fresh row with hand-set `last_verified <
2026-05-08` and no audit paragraph loads clean, and (c)'s rollback makes the
backdate durable.

Docs-alignment exemption: applied at the **caller**, not inside `validate_row`.
`run.py:98-100` skips the whole validate loop when `data["dimension"] ==
"docs-alignment"` (comment `run.py:94-97`; module docstring `_audit_field.py:9-12`).
Rationale: docs-alignment rows use `last_verdict`/`last_extracted`, no
`last_verified` key, so the cutoff has no anchor (README schema note cited at
`quality/catalogs/README.md:42`).

shell-subprocess transcript rule (`_audit_field.py:51-57`, helper
`_has_transcript_contract` `:22-32`): a `kind: shell-subprocess` row must have
`expected.artifact.transcript_path`, row-level `transcript_path`, OR a
"transcript" mention in `expected.asserts`. The last is the **transitional
fallback** (docstring `_audit_field.py:24`, "transitional fallback"; tested at
`test_audit_field.py:109-117`).

OD-2 waiver rejection (`_audit_field.py:58-67`): any row tagged
`pre-release-real-backend` carrying a truthy `waiver` → `SystemExit` before any
verifier runs (H3 fix; tests `test_audit_field.py:148-176`).

### (e) How kind:shell-subprocess is graded TODAY + what's missing (M6)

Two-layer, both weak:

- **Load-time** (`_audit_field.py:51-57`): only checks that a transcript
  CONTRACT is *declared*. The asserts-mention fallback (`_audit_field.py:31-32`)
  is satisfiable by the literal word "transcript" in prose
  (`test_audit_field.py:109-117`) — no file need exist.
- **Grade-time** (`run.py:290-316`): the runner merges any verifier-written
  artifact (`run.py:294-299`, dict `setdefault` preserves a verifier-written
  `transcript_path`), maps exit code to status (`run.py:320-332`), and
  **PASSes on exit 0 with no check that a transcript file exists**. The only
  transcript enforcement lives in the verifier-subagent PROTOCOL prose
  (`quality/PROTOCOL.md:349`) — a human/subagent instruction, not runner code.

So M6 (`89-CROSS-AI-REVIEW.md:24`): a shell-subprocess row can grade PASS on
exit 0 while emitting no transcript at all. RBF-FW-08's runtime leg closes
this: the runner itself must require `artifact.transcript_path` + file
existence (+ an `argv:` line naming a real binary) before flipping PASS.

### (f) verdict.py 3-state contract + where a milestone GREEN-block hooks in

`compute_color(rows)` (`verdict.py:121-132`): red if any P0/P1 not PASS/WAIVED;
else yellow if any P2 not PASS/WAIVED; else brightgreen. `compute_exit_code(
color, fail_on)` (`verdict.py:159-172`): brightgreen→0, red→1, yellow→
(`0 if fail_on=="red" else 1`). Default `fail_on="yellow"` (strict). This is
D-CONV-2 (`SURPRISES.md:46`, `verdict.py:18-40`). `main()` (`verdict.py:330-363`)
and `emit_session_end_rollup` (`verdict.py:305-327`) both compute color from
row statuses ONLY — neither reads an external gate artifact. **RBF-FW-12 hook
point:** milestone-close is where a fresh subagent's adversarial-pass audit
must be able to force RED even when every row status is green. The cleanest
hook is a small guard function called from a milestone-scoped verdict path
(new `--milestone <ver>` branch or reuse of `session-end`) that returns RED if
the adversarial artifact is absent or reports ≥1 failed row — see proposal 6.

### (g) exit-code map + transient-flag stripping

`_realbackend.map_exit_code_to_status(exit_code)` (`_realbackend.py:81-98`):
0→PASS, 2→PARTIAL, 75→NOT-VERIFIED (`EXIT_NOT_VERIFIED = 75`,
`_realbackend.py:54`), else→FAIL. Called at `run.py:322`. Timeout is handled
separately BEFORE the map (`run.py:318-319` sets FAIL). Exit 75 also sets the
transient `_exit75_not_verified` flag (`run.py:323-330`) to distinguish a
verifier that legitimately exited 75 from the verifier-not-found case in the
summary line. Transient flags `_stale` / `_skipped_real_backend` /
`_exit75_not_verified` are all popped at `run.py:409-411` before the rollback +
persist — never written to the catalog.

---

## 2. DESIGN PROPOSALS

> **Cross-cutting size rule:** proposals 07a/07b/08 add branch logic to
> `run.py` which is ALREADY over its 350-line cap. Net-new decision logic goes
> in `_audit_field.py` (validator concerns) or a small new helper. Proposal 07a
> is a ~4-line edit to an existing branch (acceptable in place). 07b + 08 add
> real logic → route to helpers.

### RBF-FW-07a — Missing verifier ⇒ NOT-VERIFIED, always; distinct `error` marker

**Change:** `run.py:252-254`. Replace the status-preserving conditional with an
unconditional flip, and keep the artifact `error` field (already present at
`run.py:243`) as the deploy-glitch discriminator.

```python
# run.py:252-254  BEFORE
if row.get("status") not in ("PASS", "FAIL", "PARTIAL"):
    row["status"] = "NOT-VERIFIED"
# AFTER
row["status"] = "NOT-VERIFIED"
row["_verifier_missing"] = True   # transient; drives summary line + distinct from _stale
```

The artifact already carries `error: "verifier not found at {script_rel}"`
(`run.py:243`) — that IS the "deploy glitch distinguishable from real
regression" marker the intake asks for; a downstream reader (verdict.py NOT-
VERIFIED section, or a subagent) sees `error` present ⇒ missing script, vs
`asserts_failed` populated ⇒ real FAIL. Add `_verifier_missing` to the
transient-strip list at `run.py:409-411` and to the summary-line branch at
`run.py:398-399` (which today already prints "verifier not found at ..." — keep
that string, now reached via the flag rather than the fall-through).

**Backward-compat:** verified empirically (§1a) — 0 currently-green rows flip.
The only rows with missing scripts are WAIVED and never reach this branch. Safe.

**Unit-test plan (`test_realbackend.py`, new `TestVerifierMissing` class, or a
new `test_run_missing_verifier.py` mirroring the `_run_synthetic` harness at
`test_realbackend.py:107-117`):**
- row `status: PASS` + nonexistent script → after `run_row`, `status ==
  "NOT-VERIFIED"` AND artifact has `error` containing "verifier not found"
  (proves the demote now happens).
- row `status: NOT-VERIFIED` + nonexistent script → stays NOT-VERIFIED
  (unchanged).
- assert `_verifier_missing` is NOT in the persisted row after a `main()` run
  (transient-strip regression).

**LOC:** ~6 in run.py + ~30 test. **Risk:** low. Only residual risk is a future
row that goes PASS then loses its script mid-session; that is precisely the
regression we WANT surfaced.

### RBF-FW-07b — Skip-events preserve the prior real grade (M8)

**Problem:** §1b — a cred-less real-backend run demotes a prior PASS and
persists it. **Design: skip is a no-op on ground truth, not a re-grade.**

Preferred mechanism, consistent with existing transient-flag stripping: on the
skip branch (`run.py:183-194`), **do not overwrite `status` when the catalog
already holds a real grade.** Mirror 07a's inverse:

```python
# run.py, skip branch ~191
prior = row.get("status")
if prior in ("PASS", "FAIL", "PARTIAL", "WAIVED"):
    # Preserve the last real grade; skip is not a re-grade event.
    row["_skipped_real_backend"] = True      # transient (already stripped)
    row["_skip_preserved_status"] = prior     # transient; drives summary line
    # status left untouched; last_verified left untouched (see below)
else:
    row["status"] = "NOT-VERIFIED"
    row["_skipped_real_backend"] = True
    row["last_verified"] = artifact["ts"]
```

Critically, on the preserve path DO NOT set `row["last_verified"] =
artifact["ts"]` — leave it as the last real grade's timestamp so `catalog_dirty`
sees no status change AND the same-status rollback (`run.py:408-414`) restores
`last_verified` unchanged. Net effect: a cred-less run leaves the committed
catalog byte-identical. The skip artifact is still written (honest record that
a skip occurred this run) but the CATALOG ground truth is untouched.

**Staleness signal (the intake's "mark-stale" ask,
`SURPRISES-INTAKE.md` 05:10 M8 sketch):** the skip artifact already carries
`skipped_real_backend: True` + `skip_reason` (`run.py:185-186`). verdict.py's
NOT-VERIFIED / STALE rendering reads artifacts (`verdict.py:252-255`); the
milestone-close verdict can surface "last real grade was PASS at
`<last_verified>`; this run skipped (env absent)". No new persisted field
needed — the artifact IS the staleness record. This respects D-CONV-1..8: no
schema re-fragmentation, reuses the existing artifact channel.

**Interaction with OD-2 (must not regress milestone-close hard-RED):** the P0
`agent-ux/milestone-close-vision-litmus-real-backend` row is `status:
NOT-VERIFIED` today (`agent-ux.json:1327`), NOT a prior real grade — so the
`else` branch fires and it stays NOT-VERIFIED → milestone-close stays RED while
substrate is absent. Preserve-path only protects rows that ALREADY earned a
real grade, so it cannot manufacture a false GREEN at milestone-close. Confirm
this with a test.

**Backward-compat:** rows currently at NOT-VERIFIED (the normal SLOT state) are
unaffected (else branch = today's behavior). Only prior-PASS/FAIL/PARTIAL rows
change — and for those the new behavior is strictly more honest.

**Unit-test plan (`test_realbackend.py` `TestRunRowEnvGateIntegration` extend,
around `:128-146`):**
- prior `status: PASS` + env scrubbed + tagged real-backend → after `run_row`,
  status STILL PASS; after a full `main()` cycle the catalog is unchanged
  (assert via `catalog_dirty(original, data) == False`).
- prior `status: NOT-VERIFIED` (the P0 SLOT) + env scrubbed → stays
  NOT-VERIFIED (OD-2 hard-RED preserved).
- assert `_skip_preserved_status` stripped before persist.

**LOC:** ~10 in run.py + ~35 test. **Risk:** medium — the branch touches the
persistence path; the `catalog_dirty == False` assertion is the guard that the
M8 churn is gone.

### minted_at (cross-AI H2) — write-once anchor for the audit-field cutoff

**Schema:** add optional `minted_at` (RFC3339 UTC) to the row schema
(`quality/catalogs/README.md` schema table, after the `last_verified` row at
`:37`). Written ONCE at mint time by the catalog-first commit; never updated by
the runner (unlike `status`/`last_verified`).

**Validator change (`_audit_field.py:39-42`, the anchor):**

```python
minted = row.get("minted_at")
lv = row.get("last_verified")
cutoff = parse_rfc3339(CUTOFF_ISO)
if minted is not None:
    is_new = parse_rfc3339(minted) >= cutoff        # immutable anchor wins
else:
    is_new = lv is None or parse_rfc3339(lv) >= cutoff   # legacy heuristic
# NEW: post-P90 rows MUST carry minted_at
P90_MINT_CUTOFF = "2026-07-04T00:00:00Z"   # new module const
if minted is None and lv is not None and parse_rfc3339(lv) >= parse_rfc3339(P90_MINT_CUTOFF):
    raise SystemExit(f"FAIL: {catalog_path}: row {row.get('id','?')} minted after P90 lacks minted_at ...")
```

Rules:
- When `minted_at` present → it is the sole anchor (immutable, closes the
  backdate hole; a hand-edited `last_verified` no longer moves the cutoff).
- When absent → fall back to today's null-or-`last_verified` heuristic (legacy
  rows keep validating; nothing breaks).
- Rows minted **on/after P90** (detected by `last_verified >= P90 cutoff`, or by
  a phase-close check) MUST carry `minted_at` — validator rejects otherwise.

**Must-not-break checks (both explicitly in the brief):**
1. **docs-alignment exemption:** unchanged — the whole validate loop is skipped
   for that dimension at `run.py:98-100` BEFORE `validate_row` is reached.
   `minted_at` logic lives inside `validate_row`, never consulted for
   docs-alignment. No 393-row breakage.
2. **The 5 null-`last_verified` legacy rows** (89-07,
   `SURPRISES-INTAKE.md` 2026-07-03 21:35): they have no `minted_at` and
   `last_verified: null`. With `minted_at` absent → fall back → `is_new =
   (lv is None) = True` → they must carry `claim_vs_assertion_audit` (which
   89-07 already backfilled onto all 5). AND the P90-mint rejection uses `lv is
   not None and lv >= P90_cutoff` — null `lv` fails the guard's first clause,
   so these 5 are NOT forced to add `minted_at`. Both properties preserved.

**Interaction with 07b:** 07b stops persisting `last_verified` churn on skips,
which makes `last_verified` even less trustworthy as an anchor — reinforcing
why `minted_at` (write-once) is the right long-term anchor.

**Unit-test plan (`test_audit_field.py`, extend `TestValidateRow` `:32-77`):**
- `minted_at: "2026-04-01"` + backdated `last_verified: "2026-04-01"` + no
  audit → PASSES (pre-cutoff mint). This is the legacy shape.
- `minted_at: "2026-06-01"` + backdated `last_verified: "2026-04-01"` + no
  audit → FAILS (immutable anchor defeats the backdate — the H2 fix, the key
  new test).
- `minted_at` absent + `last_verified: "2026-07-05"` (post-P90) → FAILS with
  "lacks minted_at".
- `minted_at` absent + `last_verified: null` → NOT forced to add minted_at
  (5-legacy-row regression guard); still requires audit field as today.

**LOC:** ~15 in `_audit_field.py` + ~40 test + README schema row. **Risk:**
medium — the P90-mint cutoff constant is a policy choice; set it to the P90
landing date. Keep the fallback branch so no legacy row load breaks.

### RBF-FW-06 — `coverage_kind` on transport/perf-claim rows

**Schema:** add optional `coverage_kind` enum (`real-backend` | `sim-only` |
`mechanical` | `manual`) to the row schema (README `:37` region). Enforcement:
a **transport/perf row MUST carry `coverage_kind: real-backend`** (or a
compliant `WAIVED + until_date`; NO PASS-with-comment path).

**Detection (the brief's explicit requirement — explicit field + regex, not a
hard-coded row list):** two-signal detection in a new
`_audit_field.is_transport_or_perf_row(row) -> bool`:

1. **Explicit opt-in field** `transport_claim: true` on flagged rows (authors
   mark known transport/perf rows deliberately). Highest-precedence signal.
2. **Description regex** over `comment` + `expected.asserts` + `id`, the same
   corpus the RAISE-LIST walker (`test-name-vs-asserts.sh`, RBF-FW-09) scans:

```python
_TRANSPORT_RE = re.compile(
    r"\b(push|fetch|round-?trip|clone|p50|p95|latency|throughput|"
    r"real[- ]backend|transport|list_records|list_changed_since)\b", re.I)
```

A row matches if `transport_claim` is truthy OR the regex hits the description
corpus. (Dimension is a coarse pre-filter: only consider `dimension in
{"perf", "agent-ux", "security"}` rows for the regex to cut false positives on
docs rows; perf-targets rows `perf/latency-bench`, `perf/token-economy-bench`,
`perf/handle-export-list-call-count` at `perf-targets.json:7,47,124` are the
obvious catches.)

**Enforcement split (brief-specified):**
- Rows **minted ≥ P90** (via the `minted_at` anchor above, or `last_verified >=
  P90 cutoff`) that are transport/perf AND lack `coverage_kind: real-backend`
  (and lack a compliant waiver) → **hard-block** (`SystemExit` in
  `validate_row`, same failure shape as the audit-field check).
- **Legacy** transport/perf rows lacking the field → **RAISE-only**: emit to
  `quality/reports/raise-list-p90.md` (the walker's output the ROADMAP SC#5
  demands), do NOT block load. P95 RBF-D-06 drains the legacy backlog.

This split mirrors the existing date-cutoff exemption philosophy in
`_audit_field.py` exactly (new rows held to the bar, legacy grandfathered until
a dedicated backfill phase) — no new enforcement paradigm.

**Where the code lives:** detection + the hard-block in `_audit_field.py`
(validator concern, called from `run.py:98-100` loop, already dimension-gated).
The RAISE-only legacy emission is NOT a load-time concern — it belongs in the
`test-name-vs-asserts.sh` / RAISE-LIST walker (RBF-FW-09, LANE-adjacent), which
writes `raise-list-p90.md`. Keep `validate_row` hard-block narrow; the walker
owns the soft list.

**Unit-test plan (`test_audit_field.py`, new `TestCoverageKind`):**
- transport row (regex hit on "push") minted post-P90, no `coverage_kind` →
  SystemExit.
- same row with `coverage_kind: "real-backend"` → passes.
- same row legacy (`minted_at`/`lv` pre-cutoff) → passes (RAISE-only, no block).
- non-transport mechanical row post-P90 with no `coverage_kind` → passes
  (detection must not over-fire).
- `transport_claim: true` explicit flag with no regex hit → still requires
  `coverage_kind` (explicit signal honored).

**LOC:** ~25 in `_audit_field.py` (detection + block) + ~45 test + README.
**Risk:** medium — regex false-positives; the dimension pre-filter + the
minted-≥-P90 gate bound the blast radius to newly-authored rows the author
controls. Legacy rows only RAISE.

### RBF-FW-08 runtime leg (M6) — runner requires real transcript before PASS

**Problem:** §1e — shell-subprocess rows PASS on exit 0 with no transcript
file. **Change:** in the subprocess grade path (`run.py:318-332`), after
computing a would-be PASS for a `kind: shell-subprocess` row, gate it on real
transcript evidence.

Route the check to a helper (run.py is over-cap):
`_audit_field.transcript_evidence_ok(row, artifact, repo_root) -> (bool, str)`:

```python
def transcript_evidence_ok(row, artifact, repo_root):
    tp = artifact.get("transcript_path") or (
        row.get("expected", {}).get("artifact", {}) or {}).get("transcript_path")
    if not tp:
        return False, "no transcript_path in artifact"
    p = repo_root / tp
    if not p.exists():
        return False, f"transcript file missing at {tp}"
    text = p.read_text(errors="replace")
    if "argv:" not in text:
        return False, "transcript has no argv: line"
    # argv must name a real binary, not a python fn / assert_cmd envelope
    return True, ""
```

Call site (`run.py` after `map_exit_code_to_status` at `:322`):

```python
if row.get("kind") == "shell-subprocess" and row["status"] == "PASS":
    ok, why = _audit_field.transcript_evidence_ok(row, artifact, REPO_ROOT)
    if not ok:
        row["status"] = "FAIL"
        artifact.setdefault("asserts_failed", []).append(
            f"shell-subprocess PASS blocked: {why}")
```

**Note on REPO_ROOT vs `repo_root` param:** `run_row` takes `repo_root`
(`run.py:173`); pass that, not the module global, so the synthetic-tempdir
tests still work.

**Backward-compat:** only 2 shell-subprocess rows exist
(`agent-ux/kind-shell-subprocess-worked-example` status PASS `agent-ux.json:1288`,
and `agent-ux/milestone-close-vision-litmus-real-backend` status NOT-VERIFIED
`:1327`). The worked-example's verifier `shell-subprocess-example.sh` DOES emit
a transcript via `lib/transcript.sh` (PROTOCOL `:45`), so it keeps PASSing —
BUT this must be verified against a real run before landing (OP-1: run
`bash quality/gates/agent-ux/shell-subprocess-example.sh` and confirm the
artifact carries `transcript_path` pointing at an existing file with an `argv:`
line). The litmus row is NOT-VERIFIED so never reaches the PASS gate.

**Load-time fallback tightening (optional, pairs with runtime leg):** the
transitional asserts-mention fallback (`_audit_field.py:31-32`) can now be
deprecated for post-P90 rows — require `expected.artifact.transcript_path` (or
row-level) and drop the "transcript"-in-prose acceptance for new rows. Keep it
for the 2 existing rows to avoid a re-mint. Gate on `minted_at`/cutoff.

**Unit-test plan (`test_realbackend.py` `_run_synthetic` harness `:107-117`,
extended to write/omit a transcript):**
- shell-subprocess row, verifier exits 0, artifact has `transcript_path` →
  existing file with `argv:` → PASS.
- exits 0, no `transcript_path` → FAIL with "no transcript_path".
- exits 0, `transcript_path` points at missing file → FAIL "transcript file
  missing".
- exits 0, transcript exists but no `argv:` line → FAIL.
- non-shell-subprocess row exits 0 → PASS (gate does not over-fire).

**LOC:** ~14 in run.py call site + ~18 helper in `_audit_field.py` + ~50 test.
**Risk:** medium — MUST run the worked-example verifier for real before landing
(the one row this could accidentally break).

### RBF-FW-12 — Milestone-close adversarial-pass GREEN-block

**Artifact location (brief precedent `quality/reports/verifications/`):**
`quality/reports/verifications/milestone-adversarial/<version>.json`. Shape
(minimal, no new subsystem):

```json
{
  "milestone": "v0.13.0",
  "dispatched_at": "2026-07-...Z",
  "subagent": "<author distinct from orchestrator per F-K5>",
  "rows_audited": 47,
  "rows_failed": [ {"id": "...", "reason": "assertion would not falsify description"} ],
  "verdict": "PASS"
}
```

The rubric file lands at `quality/dispatch/milestone-adversarial.md` (ROADMAP
SC#6; sibling of the existing `quality/dispatch/milestone-close-verdict.md`).
The fresh subagent reads catalog row DESCRIPTIONS only (no impl context) and
grades whether each row's assertion would falsify its description — writing the
JSON above.

**Minimal verdict.py hook (fits the ~33-line headroom):** a guard invoked only
on a milestone-scoped verdict. Add a `--milestone <version>` arg to `main()`
(`verdict.py:335-346`) and a reader:

```python
def milestone_adversarial_gate(repo_root, version):
    """Return (blocked: bool, reason). Blocks GREEN if the adversarial-pass
    artifact is absent or reports >=1 failed row (RBF-FW-12 / Decision 3)."""
    p = repo_root / "quality/reports/verifications/milestone-adversarial" / f"{version}.json"
    if not p.exists():
        return True, f"adversarial-pass artifact absent at {p.relative_to(repo_root)}"
    data = json.loads(p.read_text())
    failed = data.get("rows_failed", [])
    if failed:
        return True, f"{len(failed)} row(s) failed adversarial audit: {[r['id'] for r in failed]}"
    return False, ""
```

Wire into the milestone path: when `--milestone` is set, after computing
`color`, if the gate blocks, force `color = "red"` (so the badge + exit both go
red) and print the reason. This keeps the D-CONV-2 3-state contract intact — the
gate can only DARKEN the verdict (green→red), never lighten it, so it composes
cleanly with `compute_color`/`compute_exit_code`.

**Why not run.py:** run.py grades individual rows; the adversarial pass is a
milestone-level meta-verdict over descriptions, which is verdict.py's altitude.
And run.py has no line budget.

**Unit-test plan (`test_verdict.py`):**
- artifact absent → `milestone_adversarial_gate` returns `(True, ...)` and the
  milestone verdict is red even when all rows PASS.
- artifact with empty `rows_failed` → `(False, "")` → verdict follows
  `compute_color`.
- artifact with 1 failed row → `(True, ...)` → red.

**LOC:** ~18 in verdict.py (stays under 400: 367+18=385) + ~30 test + rubric md.
**Risk:** low — additive, darken-only, no existing caller passes `--milestone`.

### Sanctioned-target residual (cross-AI H1 residual, MED) — defer to P91

**Recommendation: DEFER to P91's litmus body. Do NOT add a sanctioned-host
allowlist check in `_realbackend` in P90.**

Rationale:
1. The intake sketch itself homes it in P91: "P91's litmus implementation MUST
   itself assert the resolved target is one of the sanctioned three ... optionally
   `_realbackend` gains a sanctioned-host allowlist check" (`SURPRISES-INTAKE.md`
   05:10 Claude leg; `89-CROSS-AI-REVIEW.md:19` H1 residual, home P91).
2. `_realbackend.is_skipped` is deliberately a **skip heuristic, not the proof**
   (`_realbackend.py:26-30` explicitly: "Sanctioned-target MEMBERSHIP is NOT
   checked here — that belongs to the litmus verifier itself (P91)"). Adding
   membership to the skip gate would blur the intended layering the P89
   reviewers just ratified.
3. The proof obligation (real execution against the sanctioned target) requires
   the litmus BODY to exist — which is P91's deliverable. A P90 host-allowlist in
   `_realbackend` would gate a code path (`is_skipped`) that only decides
   skip-vs-run; it cannot verify what the run actually TALKED to. A mis-pointed
   origin (`example.com` + a real token) still un-skips today, but there is no
   executable litmus for it to mis-target until P91 — so the exposure window is
   zero until P91 lands. Adding the check in P90 is premature and would ship a
   test with no live failure mode to exercise.
4. ROI: the sanctioned-host list lives in `docs/reference/testing-targets.md`;
   the natural consumer is the litmus verifier that resolves + asserts the
   target, co-located with the assertion it protects. Splitting it into
   `_realbackend` creates a second source of truth.

**If P90 wants a cheap down-payment** (optional, XS): extend
`_realbackend.skip_reason` / a comment noting that membership is P91's, and add
a one-line TODO-pointer test asserting the current non-membership behavior is
intentional. But the substantive check belongs in P91. Net P90 recommendation:
file nothing new; the existing intake entry already homes it correctly.

---

## 3. NOTICING

- **run.py is 429 lines — 79 over the 350 cap the codebase's own docstrings
  cite** (`_freshness.py:4-7`, run.py header `:6-7`). The cap is silently
  breached and no gate enforces it. Every P90 proposal that touches run.py is
  forced into helper modules by this, which is the right outcome — but the cap
  itself is aspirational prose, not a checked invariant. **Candidate GOOD-TO-HAVE:**
  a structure-dimension line-count gate on `run.py`/`verdict.py` (XS: one catalog
  row + a `wc -l` verifier), so the cap is real. Filed here, not fixed (read-only lane).

- **The verifier-not-found `error` marker already exists but nothing consumes
  it.** `run.py:243` writes `error: "verifier not found at ..."`, yet
  `verdict.py`'s NOT-VERIFIED section (`:235-247`) renders only `last_verified`
  + `freshness_ttl` — it never surfaces `error`. So today a missing-script
  NOT-VERIFIED and a stale NOT-VERIFIED look identical in the verdict. RBF-FW-07a
  should also teach verdict.py to render `error` when present (2-line add at
  `verdict.py:244`).

- **`subjective/dvcs-cold-reader` has `"status": "NOT_VERIFIED"` (underscore,
  not hyphen)** — a real data bug (`SURPRISES-INTAKE.md` 2026-07-03 21:35 LOW).
  `print_row_summary`'s exact-match label branches (`run.py:348`) mis-render it;
  `compute_exit_code` treats it as non-green only by accident (`not in (PASS,
  WAIVED)`). Trivial one-char fix; homed P95 but any P90 catalog touch could
  eager-fix it.

- **`catalog_dirty` compares ONLY `status`, so a `claim_vs_assertion_audit` text
  edit is never persisted-diffed by the runner** — the hash drift check
  (`run.py:312-314`) writes the hash to the ARTIFACT, but if an author edits the
  audit text without changing status, no catalog re-write is triggered and the
  drift is invisible until a verifier subagent recomputes. This is M7
  (`89-CROSS-AI-REVIEW.md:25`, "works-as-designed, thin"). It is genuinely thin;
  RBF-FW-12's adversarial dispatch is the designed consumer. Flagging that the
  thinness is load-bearing and undocumented in the runner itself.

- **`verdict.py:269` imports `from run import parse_duration` INSIDE a function**
  (the STALE days-expired calc). This is a lazy import to dodge a cycle, but
  `run.py` is a script with side-effect-free top-level (safe) — still, it's a
  latent fragility: if run.py ever grows import-time work, verdict.py's STALE
  table silently breaks. A shared `_timeparse` module (both already import
  `_freshness.parse_duration`) would be cleaner; `verdict.py` should import from
  `_freshness` directly, not `run`.

- **The shell-subprocess load-time check and the (missing) runtime check are in
  two different files with no cross-reference.** `_audit_field._has_transcript_contract`
  (`:22-32`) checks the CONTRACT exists; the runtime existence check (RBF-FW-08)
  will live in run.py's grade path. A future reader will not know these are two
  halves of one guarantee. The RBF-FW-08 helper should live in `_audit_field.py`
  next to `_has_transcript_contract` with a docstring naming the load-vs-runtime
  split, so the two halves are co-located.

- **`_realbackend._CRED_SETS` (`:21-25`) and the cred lists in `skip_reason`
  (`:74-78`) duplicate the same three credential tuples as prose** — a JIRA var
  rename would need editing both, and the prose could drift from the checked
  set. Minor DRY smell; the test `test_no_creds_names_every_cred_set`
  (`test_realbackend.py:78-82`) guards the NAMES appear but not that they match
  `_CRED_SETS`. Not P90 scope; noted.
