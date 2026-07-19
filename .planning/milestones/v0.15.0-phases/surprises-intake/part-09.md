# v0.15.0 Surprises Intake — Part 9

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain) — `part-08.md`
> reached the 20000-byte `*.md` ceiling once the P127 T1 disposition landed, so these NEW
> P127 Slot-1 noticing entries open a fresh part rather than push part-08 over budget
> (same convention as the part-07→part-08 split). Index: `../SURPRISES-INTAKE.md`.

## 2026-07-19 | discovered-by: P127 T1 lane (gsd-executor, wc -c against reality) | severity: MEDIUM

**What (entry a):** `.planning/ORCHESTRATION.md` has **re-grown to 24119 bytes**, over the
20000-byte `structure/file-size-limits` `*.md` ceiling (currently WARN-only under the
waiver that lapses **2026-08-08**). Phase 119 SC-3 split it under ceiling at close (was
20480B), but the P123–P126 doctrine additions (durable-CI-watch liveness, fresh-LEAVES
phase-close, C2 SendMessage limitation, etc.) pushed it back over. The Phase-119 split
**did not stick**. This is the concrete false-Complete that traceability row **DRAIN-11**
(`REQUIREMENTS.md`, GTH-V15-08) exists to catch, and it is why DRAIN-11 stays HELD in this
phase.

**Why out-of-scope for the discovering session:** P127 T1 is the SIGKILL-fix + intake
lane (docs/markdown-only); re-splitting ORCHESTRATION.md (extract a reference section to a
linked child doc, rebind any doc-alignment rows in the SAME commit) is a separate docs
task that must NOT be bundled into an atomic intake commit. Deferred, not silently skipped;
the waiver is still active so nothing is currently BLOCKED.

**Sketched resolution:** Before 2026-08-08, re-split ORCHESTRATION.md under 20000B by
extracting a section (e.g. the relief/handover long-form or the liveness-doctrine detail)
to a linked sibling doc under `.planning/`, following the Phase-119 pattern; then flip
DRAIN-11 → Complete. Do NOT re-split in P127. Tag **structure** (file-size hygiene);
GTH-V15-89 family (structure-dimension gate cluster); **P128**.

**STATUS:** OPEN — filed for P128 (DRAIN-11 reconcile + re-split before the 2026-08-08
waiver lapse). Tag structure / file-size-hygiene.

## 2026-07-19 | discovered-by: P127 T1 lane (gsd-executor, noticing — same class as the fix) | severity: MEDIUM

**What (entry b):** `quality/gates/agent-ux/zero-shot-onboarding.sh:55` tears down its
ephemeral sim with an **ownership-BLIND `pkill`** as a belt-and-braces fallback:
`pkill -f "reposix-sim.*${SIM_BIND}"`. This is the **same failure class** as the P127 T1
bug just fixed — a pattern-match kill with no check that THIS run started the matched
process. Two concurrent runs bound to the same `SIM_BIND` (or a foreign `reposix-sim`
whose command line happens to match the bind substring) would cross-kill each other's sim.
The comment even acknowledges the sim is "an unmanaged grandchild" reaped "by bind addr,
belt+braces" — exactly the ownership-blindness the T1 fix removed from `sweep_7878()`.

**Why out-of-scope for the discovering session:** Pre-existing, in a DIFFERENT gate
(`agent-ux`, not the `docs-repro` gate T1 fixed) and a DIFFERENT file; per the
scope-boundary rule it does not belong in the T1 atomic commit. FILED per OP-8.

**Sketched resolution (<1h):** Mirror the T1 pattern — start the sim under `setsid` (own
process group; `SIM_PGID == SIM_PID`) and group-kill by PGID on teardown (`kill -- -$PGID`),
then DROP the ownership-blind `pkill -f` fallback entirely. Cross-reference the
`register_owned`/`SWEEP_OWNED_*` ownership-scoping in `container-rehearse-sigkill-safe.sh`
and the `start_sim_in_own_pgroup` helper in `lib/sim-lifecycle.sh`. Tag **code** /
agent-ux gate; **P128** (or eager-fix in a code lane if one opens sooner).

**STATUS:** OPEN — ownership-blind sibling of the T1 class; P128 code lane.

## 2026-07-19 | discovered-by: P127 T1 lane (gsd-executor, self-review of the NEW gate) | severity: LOW

**What (entry c):** The new regression gate `quality/gates/docs-repro/sweep-7878-ownership-scoped.sh`
(and the `sweep_7878()` it exercises) accumulates killed identities in the `SWEEP_OWNED_*`
arrays but does NOT remove an identity after its explicit kill — the identities linger to
the final EXIT-time sweep. If the OS **recycles a dead pid/pgid** within the gate's ~13s
run, the final `kill -KILL -- -${p}` could land on an unrelated, freshly-spawned process
group that reused the number. This is the SAME ownership-blind-kill CLASS as the original
bug but **strictly narrower**: it requires a pid/pgid recycle inside a ~13s window onto a
previously-owned-then-reaped identity, versus the original bug which killed ANY foreign sim
unconditionally on every run.

**Why out-of-scope for the discovering session:** It is a further hardening of code the T1
fix just landed (not a regression the fix introduced — the fix is a strict improvement over
the ownership-blind sweep); folding a second code change into the T1 atomic commit would
muddy the mechanism-not-symptom review that already PASSed. FILED per OP-8.

**Sketched resolution (<1h):** After each explicit `kill` in `sweep_7878()`, remove that
identity from the `SWEEP_OWNED_*` array so a recycled pid/pgid can never be re-killed at the
final EXIT sweep (kill-once semantics). Add a selftest leg asserting a killed identity is
not re-signalled. Tag **code** / docs-repro gate; **P128**.

**STATUS:** OPEN — strictly-narrower residual of the T1 class in the new gate; P128.

## 2026-07-19 | discovered-by: P127 T1 lane (gsd-executor, wc -c against reality) | severity: LOW

**What (entry d):** `quality/gates/docs-repro/container-rehearse-sigkill-safe.sh` is now
**16972 bytes = ~170% of the 10000-byte `.sh` `structure/file-size-limits` ceiling**
(WARN-only, waived until 2026-08-08). The T1 fix added the ownership-scoping helpers
(`register_owned`/`sweep_7878`) inline, growing an already-over-budget script further.

**Why out-of-scope for the discovering session:** Refactoring the script into a `lib/`
helper is a code-structure change, not part of the atomic T1 SIGKILL fix; per the
scope-boundary rule it is filed, not bundled. The waiver is active so nothing is BLOCKED.

**Sketched resolution:** Before the 2026-08-08 waiver lapse, factor the port/ownership
helpers + the STATIC/DYNAMIC selftest legs out into a `lib/` module (mirroring the existing
`container-rehearse.sh` → `lib/sim-lifecycle.sh` split), bringing the entry script under
10000B. Tag **structure** / file-size-hygiene; **P128**.

**STATUS:** OPEN — file-size debt filed for P128 (before 2026-08-08). Tag structure.

## 2026-07-19 | discovered-by: P127 T1 lane (gsd-executor, noticing — same class as the fix) | severity: LOW

**What (entry e):** The operator recovery hints in `container-rehearse.sh:223`
(the DRAIN-13 sim-readiness `write_fail_artifact` message) and `lib/sim-lifecycle.sh:61,64`
(the DRAIN-23 stale-orphan fail-loud path) teach an **ownership-BLIND recovery command**:
`kill $(lsof -ti:7878)` — which kills EVERY listener on 7878, including a legitimate
concurrent session's sim. This is the human-facing edge of the same ownership-blindness
class the T1 fix removed from the machine path; a Rust-compiler-grade recovery hint (North
star OD-3.5) should teach the narrower, safe form.

**Why out-of-scope for the discovering session:** These are documentation/hint strings in
files the T1 atomic commit did not touch; a hint-copy edit is filed, not bundled. Low
blast radius (operator convenience text, not an executed kill path).

**Sketched resolution (<1h):** Narrow the hint to name the LISTEN socket explicitly, e.g.
`kill $(lsof -ti:7878 -sTCP:LISTEN)` (still names the recovery, but scopes to the listener),
or teach `fuser -k 7878/tcp` with a caveat. Fold alongside entry (b)'s ownership-blind-kill
cleanup. Tag **docs** / agent-ux recovery-hint (North-star UX); **P128**.

**STATUS:** OPEN — LOW operator-hint note, same ownership-blind class as (b); P128.
