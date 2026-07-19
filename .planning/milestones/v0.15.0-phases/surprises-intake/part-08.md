# v0.15.0 Surprises Intake — Part 8

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain) — `part-07.md`
> was already at 30153 bytes (over the 20000-byte `*.md` ceiling, currently WARN-only
> per `structure/file-size-limits`'s waiver); this entry opens a fresh part rather than
> growing an already-over-budget file further. Index: `../SURPRISES-INTAKE.md`.

## 2026-07-19 | discovered-by: Cycle-2 bundled `/gsd-quick` task (d) executor | severity: MEDIUM | **RESOLVED same commit**

**What:** `quality/runners/test_freshness_synth.py` invoked `run.py --cadence weekly` as
a subprocess to exercise the STALE-P2 label mechanism for exactly ONE row
(`subjective/headline-numbers-sanity`), but `--cadence weekly` iterates EVERY
weekly-tagged row across EVERY catalog file, not just the target row. Two classes of
unrelated noise leaked into the test's result:

1. `quality/catalogs/release-assets.json` carries 14 weekly rows (8x
   `release/crates-io-max-version/*`, `install/homebrew`, `release/brew-formula-current`,
   `release/gh-assets-present`, `install/curl-installer-sh`,
   `install/powershell-installer-ps1`, `install/build-from-source`) that all make live
   `urllib.request` HTTP calls (crates.io API, GitHub API). A flaky or sandboxed-offline
   network turns any of these P1 rows FAIL, which flips `compute_exit_code` to 1 —
   nondeterministically breaking the test's `returncode == 0` assertion (the "stale-P2
   flake" surfaced in the PR#77 family).
2. `quality/catalogs/docs-reproducible.json` carries 2 weekly rows
   (`benchmark-claim/8ms-cached-read`, `benchmark-claim/89.1-percent-token-reduction`)
   with a deliberate catalog-first `verifier.script: null` stub and no waiver — every
   weekly run of `run_row` on these emits a `verifier not found at None` NOT-VERIFIED
   artifact, printed straight into the runner's stdout as unrelated leakage.

Locate-or-file search performed first (per this task's charter):
`grep -rn "test_freshness_synth\|stale-P2\|verifier not found at None"
.planning/milestones/v0.15.0-phases/` returned only the Cycle-2 handover pointer
(`RELIEF-HANDOVER-C2-wave-3.md:239-254`, itself the charter for this fix) — no prior
exact-match tracking row existed. `GTH-V15-55`
(`good-to-haves/part-06.md:26`, the `cdn.simpleicons.org` embed) is confirmed a
DIFFERENT live-network smell, not this one.

**Why out-of-scope for the discovering session:** N/A — fixed inline in the same session
that filed this entry (Rule 1/eager-fix; single Python test file, no new dependency,
well under the 1-hour threshold).

**Fix landed (same commit as this entry):**
`quality/runners/test_freshness_synth.py` gained `_neutralize_other_weekly_rows` — before
invoking the `--cadence weekly` subprocess, every weekly-cadence row OTHER than the
target row (across every catalog under `quality/catalogs/`) gets a temporary near-future
waiver, which makes `run_row` short-circuit to WAIVED BEFORE it can reach either the
live-network branch or the null-verifier-missing branch. The existing
`backup_catalog` fixture was generalized to `backup_catalogs` (backs up/restores EVERY
catalog file, not just `subjective-rubrics.json`) so the mutation never survives the
test, pass or fail. Verified network-denied:
`unshare -rn -- python3 -m pytest quality/runners/test_freshness_synth.py -v` passes in
~0.07s under a network namespace with no route to the internet (sanity-checked
separately: a bare `urllib.request.urlopen` call under the same `unshare -rn` fails with
`Temporary failure in name resolution`, confirming the isolation is real, not
coincidental). Before the fix, the same subprocess made 20+ live HTTP round trips and
took multiple seconds even when the network was healthy.

**Hermeticity convention this establishes (fix-twice, tagged in the SAME commit):** any
test that reaches a live network resource, directly or transitively (e.g. via a
subprocess that fans out into unrelated live-network catalog rows), must pass
deterministically OFFLINE — mock, stub, or short-circuit the probe before it fires.
Catalog row: `structure/hermetic-test-network-isolation`
(`quality/catalogs/freshness-invariants.json`, `verifier.script: null`,
`kind: manual`, `blast_radius: P2`) records this contract as a manual/documentation
gate (no automated cross-repo test-corpus scanner exists yet — that would be the
natural follow-up, filed separately if it doesn't already have a GOOD-TO-HAVE). CLAUDE.md
update: `quality/CLAUDE.md` § "Hermetic test convention" (new subsection, same commit).

**STATUS:** RESOLVED (same commit that introduces this entry — see
`quality/runners/test_freshness_synth.py` diff).

## 2026-07-19 | discovered-by: Cycle-2 task (d) executor (own `--persist` verification pass) | severity: HIGH | **STATUS: CLOSED (P126 W1)**

**What:** Minting the new `structure/hermetic-test-network-isolation` row's PASS status
(via `python3 quality/runners/run.py --cadence pre-pr --persist`, run once to verify the
row grades correctly) tripped a genuine, pre-existing, latent bug in
`quality/catalogs/agent-ux.json` row `agent-ux/real-git-push-e2e`: its `comment` field
documents an environment precondition ("On this dev box (git 2.25.1) the verifier's
version precondition short-circuits to NOT-VERIFIED (exit 75)") that is now FALSE — the
executing box's git has since been upgraded to **2.50.1** (>= the row's 2.34 gate). With
the precondition no longer met, the verifier actually EXECUTES for real on `--persist`,
producing a genuine status change and a fresh `last_verified` timestamp >= the P90 mint
cutoff (2026-07-05) — but the row is legacy (no `minted_at`, per its own comment:
"legacy row (no minted_at, D91-11 / PLAN-CHECK MF-2)"). `_audit_field.validate_row`
correctly rejects this at the NEXT catalog **load** with `SystemExit` ("has last_verified
>= 2026-07-05T00:00:00Z but lacks a write-once minted_at anchor") — but because
`load_catalog` raises `SystemExit` uncaught, this doesn't just FAIL that one row, it
**crashes the entire `run.py` process before grading begins**, for EVERY cadence that
loads `agent-ux.json` (i.e. every cadence except ones that never touch that catalog).
Reproduced live: this exact crash is what caused my own new gate
(`structure/hermetic-test-network-isolation`, whose verifier subprocess-invokes
`run.py --cadence weekly`) to grade a false FAIL on the first `--persist` attempt — not a
flake in my gate, a real self-inflicted catalog corruption from `agent-ux.json`'s OWN
persist-write landing (alphabetically) before my row was graded in the same run.

**Why out-of-scope for the discovering session:** Pre-existing (not caused by this
task's changes to `test_freshness_synth.py` or the new hermeticity gate) and in an
unrelated dimension (`agent-ux`, not `structure`/hermeticity). Per the scope-boundary
rule, fixed the fallout in my own working tree (reverted the corrupted `agent-ux.json` /
`code.json` via `git checkout --`, re-minted my own row cleanly via `--cadence pre-push`
instead, which does not touch this row) rather than editing `agent-ux.json` outside my
charter.

**Sketched resolution:** Add `minted_at` (RFC3339, now) + a `claim_vs_assertion_audit`
(>=50 chars) to `agent-ux/real-git-push-e2e` in the SAME commit that refreshes its stale
"git 2.25.1" comment to reflect the box's actual git version — matching the pattern used
by every other post-P90 row in this catalog. Until this lands, **any future
`--persist` mint that includes `pre-pr`/`pre-release` cadence on a box with git >= 2.34
will crash `run.py` entirely** (not just this row) the first time it re-grades this row
for real; the next agent who hits this should not re-diagnose from scratch — this entry
has the root cause and the fix.

**STATUS:** CLOSED — resolved in P126 W1 exactly as sketched. `65e8c497` added the
write-once `minted_at` anchor to `agent-ux/real-git-push-e2e` (+ refreshed its stale
"git 2.25.1" comment to the box's real 2.50.1; `claim_vs_assertion_audit` was already
present) so `validate_row` takes the `minted is not None` branch and the lv-based crash
raise is unreachable forever; the SAME commit hardened the write path
(`run.py::save_catalog` now takes a required `persist=` keyword and raises `RuntimeError`
on a `persist=False` write, so a non-persist cadence cannot round-trip any catalog — the
class that staled/un-waived `subjective-rubrics.json`'s `headline-numbers-sanity` row in
entry 1's family). `d0753ef6` recorded the fix-twice doctrine in `quality/CLAUDE.md`.
Whole-corpus regression lock: `test_run.py::TestNoArmedMintedAtLandmine` (FAILs if any
row carries `last_verified` >= the P90 cutoff without `minted_at`) +
`TestSaveCatalogPersistGuard` + `TestValidateOnlyMultiCatalogByteIdentical`. Fresh DP-2
review confirmed mechanism-vs-symptom (`5d097937`).

## 2026-07-19 | discovered-by: P126 close-bookkeeping lane (coordinator, phase-close gate sweep) | severity: LOW | **RESOLVED (confirmed flake)**

**What:** During the P126 close-bookkeeping gate sweep, the
`docs-repro/container-rehearse-sigkill-safe` gate SIGKILLed mid-run once. A clean rerun of
the SAME cadence passed GREEN with no change — the SIGKILL was a **confirmed flake**, not a
regression (consistent with the P124-review de-flake that widened the SIGKILL control-leg
budget; the row is deliberately excluded from the pre-push battery and is P1 cargo/sim-
dependent, so a loaded VM can nondeterministically trip its control leg).

**Why out-of-scope for the discovering session:** N/A — resolved by the documented
rerun-first recovery; no code change needed for the flake itself.

**Sketched resolution:** RESOLVED — the documented recovery is rerun-first; a clean rerun
passed. (The DEEPER structural bug the flake exposed — the kill blast radius — is filed as
its own HIGH INFRA-BUG entry immediately below; THIS entry only records the flake outcome.)

**STATUS:** RESOLVED (confirmed flake; clean rerun GREEN).

## 2026-07-19 | discovered-by: P126 close-bookkeeping lane (coordinator, observed at the SIGKILL flake above) | severity: HIGH

**What:** When the `docs-repro/container-rehearse-sigkill-safe` flake (entry above) fired,
it did NOT kill only its own docker/sim child — it SIGKILLed the **ENTIRE `run.py`
process**, taking down all ~83 gates in the cadence run, not just the one flaky row. The
kill logic is process-group/parent-scoped rather than child-PID/subtree-scoped, so a signal
meant for the gate's own ephemeral child propagates up to the shared runner process. This is
the **same failure class** as the fd-inheritance deadlock fixed at `cef3a2ea` (a
backgrounded grandchild holding a shared resource — there a capture pipe, here a process
group — that the runner also depends on). A **leaked orphan `reposix sim` process (PID
11014, ~18MB RSS)** was observed still alive at kill time — a symptom of the same
under-reaping family: the gate's ephemeral sim was not torn down cleanly when the group kill
landed, leaving an orphan bound to (or racing for) port 7878 for the next run.

**Why out-of-scope for the discovering session:** This is a runner/gate CODE change
(process-group → child-subtree kill scoping) plus validation (a selftest proving a killed
gate child does not take down the parent `run.py`, and that the ephemeral sim is reaped),
which does not belong in an atomic docs/planning-only phase-close commit. FILED per OP-8.

**Sketched resolution:** Scope the kill logic in `container-rehearse-sigkill-safe.sh` (and
any sibling gate that `setsid`/process-group-kills an ephemeral child) to target ONLY its
own child PID / subtree — never the shared process group that `run.py` lives in. Cross-
reference `cef3a2ea` (the fd-inheritance sibling fix) for the reaping+isolation pattern.
Add a selftest that (1) drives the gate's child to the kill path and asserts the parent
`run.py` survives, and (2) asserts no orphan `reposix sim` / port-7878 binding survives the
run (fold in a fail-loud pre-run stale-orphan gate on 7878 if not already present — P124's
DRAIN-23 added one for its own harness; verify coverage here). Slot into P127 (Surprises
absorption, OP-8 Slot 1).

**STATUS:** OPEN — tag code / CI infra (runner kill-scoping); P127 Slot 1 candidate.

## 2026-07-19 | discovered-by: P126 gsd-verifier (WARN-1, phase-close grade) | severity: MEDIUM

**What:** The `structure/hermetic-test-network-isolation` row carries a **STALE local PASS
mint** in `quality/catalogs/*.json`, but the gate **FAILS DETERMINISTICALLY at ~0.02s** on
the newest `main` CI run AND on `dc60cc21` — a CI-portability fast-fail: it passes locally
(poisoned-proxy denial available) but fast-fails in the CI sandbox (the sandbox restricts
whatever the gate needs to deny the network deterministically). It is **P2 non-blocking**
(pre-pr exit=0) and **PRE-EXISTING** — the gate + its test were created by `f1959373`, an
ancestor of P126's first commit `44783ebe` — so it did NOT roll P126 RED. But a committed
`PASS` riding green on a gate that deterministically FAILs in CI is a **lying catalog row**
(honesty dimension): the local mint asserts a green the CI environment cannot reproduce.

**Why out-of-scope for the discovering session:** Pre-existing (created by `f1959373`,
before P126) and requires either a re-mint (honest WARN/known-fail) or a real
CI-sandbox-portability fix to the gate + a re-grade — neither belongs in an atomic
docs/planning-only close commit. FILED per OP-8 / scope-boundary.

**Sketched resolution:** Either (a) re-mint the row honestly — `NOT-VERIFIED` /
`WAIVED + until_date` with a `skip_reason` naming the CI-sandbox restriction, so the catalog
stops asserting an unreproducible green — or (b) fix the CI-sandbox portability of
`hermetic-test-network-isolation.sh` (the gate already documents a poisoned-proxy-vs-
`unshare -rn` tradeoff in its header for exactly this CAP_SYS_ADMIN/namespace-restriction
reason; the ~0.02s fast-fail suggests the chosen mechanism is unavailable in the CI sandbox,
not that the test itself is wrong). Slot into P127 alongside the `code/shell-coverage`
stale-mint work if the two share the `minted_at`→F-K4b activation family (they may not —
verify: this one is a mint-vs-CI-reality gap, not an F-K4b congruence demote).

**P127 T2 DISPOSITION (2026-07-19):** Took option (a) — re-minted the row honestly to
`WAIVED` (metadata write: `status: WAIVED` + a `waiver` block, `until: 2026-09-15`,
`dimension_owner: P127 / intake part-08`) in `quality/catalogs/freshness-invariants.json`.
The catalog now stops asserting an unreproducible green: `run_row`'s `waiver_active`
short-circuit grades the row `WAIVED` (proven: graded WAIVED in 0.0006s at pre-push
without running the verifier) instead of the lying local `PASS`. Verified the gate still
passes LOCALLY (exit 0) so the WARN-only disposition is honest, not masking a real
regression. Option (b) — the deeper CI-sandbox-portability CODE fix to
`hermetic-test-network-isolation.sh` (>1h, requires reproducing the CI-sandbox
restriction and choosing a denial mechanism the sandbox permits) — **STAYS FILED** here;
it is out of scope for this Slot-1 metadata drain. When (b) lands and the gate is
confirmed green IN CI (not just locally), re-mint to PASS via
`run.py --cadence pre-push --persist` and drop the waiver.

**STATUS:** PARTIAL — honesty half CLOSED (WAIVED re-mint, P127 T2); CI-portability CODE
fix OPEN (>1h, kept-filed). Tag structure / honesty (catalog-mint-vs-CI-reality).

## 2026-07-19 | discovered-by: P127 T4 (traceability reconcile, gsd-executor) | severity: MEDIUM

**What:** `.planning/REQUIREMENTS.md`'s traceability table marked
DRAIN-11/13/14/22/23/24/25 "Pending" although their assigned Phases 119 & 124 both
closed GREEN — a stale-traceability gap. A naive "phase closed GREEN → mark all its
DRAIN rows Complete" pass would have LAUNDERED OPEN BUGS into false Completes. P127 T4
reconciled ONLY the unentangled structural rows against reality, and HELD the rest:

- **FLIPPED → Complete (verified against reality):** DRAIN-14 (`.sim-*.log` gitignored —
  `.gitignore:95` present; Phase 124 SC4 `d83bbe32`) and DRAIN-25 (six P79-P84 `**Plan:**`
  links repointed to the `NN-PLAN-OVERVIEW/index.md` directory form — all six resolve on
  disk; Phase 119 SC-4, close 2026-07-17).
- **HELD Pending (reconcile in P128 after the entangled item drains):**
  - **DRAIN-11** — Phase 119 SC-3 split ORCHESTRATION.md under ceiling AT CLOSE (was
    20480B), but it has since **RE-GROWN to 24119B > the 20000B ceiling** (P123–126
    doctrine additions); the under-ceiling steady-state is NOT met now, so the flip
    condition "verify under ceiling now" FAILS. Re-split before the file-size waiver
    lapses 2026-08-08. (This is the false-Complete the drain exists to catch.)
  - **DRAIN-13** — exit-from-artifact half verified (Phase 124 SC4 `d83bbe32`), but the
    "pre-`docker run` port-7878-free + sim-reachability readiness gate" half shares the
    port-7878-orphan surface with the OPEN SIGKILL blast-radius (intake #1 above / T1) —
    reasonable doubt → HOLD.
  - **DRAIN-22** — F-K4b container-class congruence tautology; entangled with the OPEN
    F-K4b-container-tautology item (same class as P127 T3). Phase 124 SC1 claims it EARNED
    via ASSERT-PASS harvesting, but the class stays open per the coordinator's guard.
  - **DRAIN-23** — SIGKILL sim-leak / EXIT-trap orphan; entangled with intake #1 (SIGKILL
    process-group blast-radius, OPEN, under DP-2 / T1). Phase 124 SC2 claims SIGKILL-proof
    teardown, but P126 re-discovered the blast radius kills the whole `run.py` — NOT closed.
  - **DRAIN-24** — `target/debug/reposix` provenance; entangled with the OPEN binary-
    provenance-unconfirmed item (part-01). Phase 124 claims a guaranteed build step; the
    provenance question stays filed.

**Why out-of-scope for the discovering session:** The traceability staleness itself is
reconciled here (T4). The three HELD code-fix families (SIGKILL kill-scoping, F-K4b
container tautology, binary provenance) are the OPEN intake items themselves (>1h each,
code changes) — not a metadata reconcile. DRAIN-11's re-split is a docs task under the
active file-size waiver umbrella (warn-only until 2026-08-08), deferred not silently
skipped.

**Sketched resolution:** P128 re-checks each HELD row after its entangled intake item
drains: DRAIN-22/23/24 flip when the SIGKILL/F-K4b/provenance code fixes land and grade
green; DRAIN-13 flips when the 7878-orphan surface is decoupled from the open SIGKILL
item; DRAIN-11 flips when ORCHESTRATION.md is re-split under 20000B (before 2026-08-08).

**STATUS:** PARTIAL — traceability reconcile DONE (DRAIN-14/25 flipped, DRAIN-11/13/22/23/24
held with entanglement annotations in REQUIREMENTS.md); the five HELD rows tracked for
P128. Tag structure / traceability-hygiene.
