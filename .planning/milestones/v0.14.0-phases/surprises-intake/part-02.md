# v0.14.0 Surprises Intake (P110 source-of-truth) — Part 2 of 2

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-12 08:35 | discovered-by: P105 (RBF-LR-03 rebase-recovery gate, Lane 2) | severity: HIGH

**What:** The P105 parent-chaining fix (`90ddaff`, `fast_import.rs` emits `from
<tracking-tip>`) is INCOMPLETE. It eliminated the `fatal: error while running fast-import` /
`does not contain` abort — verified — but driving the FULL documented recovery
end-to-end through a real `git pull --rebase` (not the isolated `git fast-import` that
`verify_fix.sh` / the `git_fast_import_roundtrip_with_parent_fast_forwards` unit test
exercise) surfaces a SECOND fetch-time abort:
```
error: cannot lock ref 'refs/reposix/origin/main': is at <X> but expected <Y>
 ! <Y>..<X>  main -> refs/reposix/origin/main  (unable to update local ref)
```
**Root cause (file:line):** the `import` helper's fast-import stream writes `commit
refs/reposix/origin/main` DIRECTLY (`crates/reposix-remote/src/fast_import.rs:165`, and
the no-op-guard `reset` at `:142`), while the helper ALSO advertises `refspec
refs/heads/*:refs/reposix/origin/*` (`crates/reposix-remote/src/main.rs:193`). So git
applies its OWN refspec update to `refs/reposix/origin/main` AFTER import. On drift the
stream fast-forwards the ref underneath git; git's post-import ref transaction (old-value
= the PRE-fetch tip) then finds the ref already moved → `cannot lock ref` → `git fetch`
(hence `git pull --rebase`) exits non-zero. Because the documented recovery is the single
command `git pull --rebase && git push`, the non-zero pull SHORT-CIRCUITS at the `&&` and
`git push` never runs — the agent's unpushed edit is LOST until an UNDOCUMENTED second
`git pull --rebase` (the ref is now settled + the no-op guard makes the re-fetch a clean
fast-forward, so pull #2 exits 0 and push converges). **Empirically reproduced** (live
sim, git 2.25.1, independent caches, both scenarios — peer git-push AND external REST
PATCH drift): `git pull --rebase && git push` exits 1, SoT record version unchanged; a
plain `git fetch origin` on drift ALSO exits 1 with the same `cannot lock ref`; a second
`git pull --rebase && git push` exits 0 and converges. `reposix sync --reconcile` does NOT
help (it rebuilds the CACHE, not the client's tracking ref). Repro:
`.planning/phases/105-rbf-lr-03-rebase-recovery/repro/repro-fetch-ref-lock.sh`. Gate that
catches it (currently NOT-VERIFIED): `quality/gates/agent-ux/rebase-recovery-reconciles.sh`
(transcript under `quality/reports/transcripts/rebase-recovery-reconciles-*.txt`).

**Why out-of-scope for P105:** P105's charter + landed fix target the parentless
non-descendant commit (the `does not contain` abort), which is fixed. This second abort is
in git's remote-helper `import` ref-update contract — the helper double-writes the tracking
ref that git also owns. A correct fix (e.g. NOT naming `refs/reposix/origin/main` in the
stream and letting git's refspec do the single authoritative update, or writing to a
neutral staging ref) touches load-bearing protocol code where a naive change risks
CLOBBERING the caller's local branch on a lights-out system — it needs its own design +
cross-git-version testing (import vs stateless-connect, §5 still unverified on modern git).
That is a dedicated fix phase, not a <1h in-lane eager fix; folding it into P105 would
double the phase scope and re-open the exact "client-side special-case patch" smell the
P105 dispatch forbade.

**Sketched resolution:** (a) Stop emitting `commit/reset refs/reposix/origin/main` in the
`import` stream; write the commit to the ref name git requested (the `import <ref>` LHS,
e.g. `refs/heads/main`) or a helper-private staging ref, and let git's advertised refspec
perform the SINGLE authoritative `refs/reposix/origin/main` update — matching the
git-remote-testgit reference-helper pattern — so there is no double-write. Verify the
caller's local `main` branch is NOT touched. (b) Failing that, drop `refs/reposix/origin/*`
from the advertised import refspec so git does not attempt its own update after the stream
writes the ref. Either way: re-run `quality/gates/agent-ux/rebase-recovery-reconciles.sh`
— it is written to flip to PASS (exit 0) WITHOUT edits once the single documented
`git pull --rebase && git push` exits 0 and the edit reaches the SoT. Also resolve PLAN §5
(does stateless-connect on git >= 2.34 exhibit the same or a different failure?) on a
modern-git CI runner before closing.

**STATUS:** RESOLVED-in-P105, fix commit `bd5b9cb` (`fix(105): helper import writes private ref ns
— RBF-LR-03 ref-lock`) — **VERIFIED AGAINST REALITY, corrects the intake's 08:35 snapshot.** This
entry was filed at 08:35 during P105 Lane-2 discovery and reads "OPEN / still-live"; the fix landed
LATER in the SAME P105 close and the drain missed it. Confirmed real in committed source: the
double-write is GONE — `crates/reposix-remote/src/main.rs:202` now advertises
`refspec refs/heads/*:refs/reposix-import/*` and `fast_import.rs:127-130` writes the PRIVATE import
namespace `refs/reposix-import/*` (disjoint from the user tracking ns `refs/reposix/origin/*`), so
`git fetch` is the SOLE writer of `refs/reposix/origin/main` — exactly the sketched-resolution
option (a)/(b) above. Phase-close proof: `.planning/phases/105-rbf-lr-03-rebase-recovery/VERIFICATION.md`
(GREEN, HEAD `8afb52d`) grades `agent-ux/rebase-recovery-reconciles` exit 0, 13/13 asserts — the
single documented `git pull --rebase && git push` now reconciles across peer-push drift (A),
REST-PATCH drift (B), and record-deletion (C). The committed repro
`.planning/phases/105-rbf-lr-03-rebase-recovery/repro/repro-fetch-ref-lock.sh` (4234 bytes,
git-tracked) captured the PRE-fix failure on git 2.25.1.
**RESIDUAL — DEFERRED-TO-v0.15.0 (verification-only, NOT a live bug):** PLAN §5 remains — the gate
ran on git 2.25.1; whether `stateless-connect` on git >= 2.34 exhibits the same/different behavior
is not yet exercised on a modern-git CI runner. That is a coverage extension, not an unfixed
push-correctness defect. **OWNER SURFACE:** the coordinator's original disposition ("still-live
HIGH bug, dedicated debug phase") is superseded — the bug is fixed + gate-GREEN; only the modern-git
verification residual carries forward.

---

## 2026-07-12 09:40 | discovered-by: P105 (RBF-LR-03 docs fix-twice lane, ownership noticing) | severity: MEDIUM

**What:** `resolve_import_parent()` (`crates/reposix-remote/src/main.rs:400-419`) degrades
to the parentless path (`None` → no `from`, no `deleteall`) on **any** git error, not just
ref-absence. Two conflations: (1) the `rev_parse` closure returns `None` via `.ok()?`
(`main.rs:407`) when `Command::new("git").output()` itself fails — git binary missing, spawn
/ I/O error — swallowing a real environmental fault as "no parent." (2) `!out.status.success()`
(`main.rs:408`) treats *every* non-zero rev-parse exit as ref-absent, when
`rev-parse --verify --quiet` also exits non-zero for other failure modes. Either way the
fetch silently falls back to a parentless overlay — precisely the non-descendant
"does not contain" abort P105 just fixed (`fast_import.rs` parent-chaining). A future
regression that makes rev-parse fail for a non-absence reason would silently re-open the
RBF-LR-03 bug with no error surfaced to the operator.

**Sketch:** Distinguish ref-absent (the legitimate parentless case: `rev-parse --verify
--quiet <ref>` exits 1 with empty stdout AND the git spawn succeeded) from other rev-parse
/ spawn failures. On the latter, error the fetch loudly (`fatal:` + recovery hint) instead
of degrading to parentless — a spawn failure or a malformed-arg exit is an operator-facing
fault, not a valid "fresh clone, no tracking tip yet" signal. Keep the existing empty-stdout
→ `None` path for the genuine first-fetch case. Add a unit test that injects a non-absence
git failure and asserts the fetch errors rather than emitting a parentless overlay.

**Why out-of-scope for the P105 docs lane:** code change in load-bearing helper protocol
(cargo build + a new test) — this lane is docs-only, cargo-free. Small (<1h) but needs a
cargo window; file rather than eager-fix.

**STATUS:** DEFERRED-TO-v0.15.0 (helper-hardening phase). This is a distinct, still-latent
robustness gap in `crates/reposix-remote/src/main.rs:400-419` `resolve_import_parent()` — it
degrades to the parentless path on ANY git error (spawn failure / non-absence rev-parse exit), not
just ref-absence, and is NOT addressed by the P105 `bd5b9cb` disjoint-namespace fix above (that
fixed the ref-lock double-write, a different failure mode). A future regression making rev-parse
fail for a non-absence reason would silently re-open the RBF-LR-03 non-descendant abort with no
operator-facing error. Fix (distinguish ref-absent from spawn/other rev-parse failure; error
loudly instead of degrading to a parentless overlay; add an injected-non-absence-failure unit test)
is small (<1h) but needs a cargo window — out of scope for this cargo-free planning drain. Belongs
to the same `crates/reposix-remote` helper-hardening phase as the row-6/row-5 residual verification.

---

## 2026-07-12 | discovered-by: D2 re-seal Wave 1 (shell/planning lane) | severity: HIGH

**Title:** live D2 repro — post-P102 shared-tree corruption via subprocess/worktree bypass.

**What:** A live recurrence of the founding S-260707-pr-08 corruption class occurred
AFTER P102 shipped GREEN. A P106 leaf subagent created a git worktree INSIDE
`.claude/worktrees/agent-a98058321e3f649a7` of the SHARED repo (not a `/tmp` clone) and ran
`reposix init` / sim-seed via a path that does NOT go through the Claude Code Bash tool, so
the PreToolUse leaf-isolation hook (Bash-tool-only — coverage boundary documented at
`.claude/hooks/leaf-isolation-guard.sh` header, "COVERAGE BOUNDARY") never fired. Symptoms:
shared `.git/config` `core.bare=true`, `origin` repointed to the sim at
`127.0.0.1:7988`, `HEAD` thrashed to `e18df81`, and `refs/reposix/*` polluted. The shared
tree was repaired at commit `9d78d62`. This is a NEW recurrence, distinct from the founding
`S-260707-pr-08` — same corruption end-state, different bypass path (non-Bash-tool
subprocess + in-shared-repo worktree rather than a "forgot-to-cd" Bash-tool leaf).

**Guard defects found + FIXED in this Wave-1 shell/planning lane** (Bash-tool-layer
defense-in-depth, `.claude/hooks/leaf-isolation-guard.sh`; regression asserts in
`quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh` Cases 9-11):
- **(A) config-read false-positive** — Guard C misclassified a `git config` READ as a WRITE
  whenever the guarded key was followed by ANY trailing token (`2>/dev/null`, `&& echo`,
  `| grep`), LIVE-BLOCKing coordinators that merely read `core.bare`/`user.email`. Fixed:
  read-flag detection (`--get*`/`--list`/`-l`) + segment isolation (cut at first shell
  separator, strip redirections) before the value-token write-heuristic. Real writes
  (`git config core.bare true`, `--unset`, `--replace-all`) still BLOCK.
- **(B) git-init-bare gap** — bare/`--bare` `git init` in the shared tree (the founding
  `core.bare=true` end-state) was not blocked by any guard. Fixed: added to Guard B.
- **(C) cargo sim-seed spelling gap** — `cargo run -p reposix-sim -- seed …` slipped Guard B
  (`reposix-sim` sits at an ARGUMENT position under `cargo run`, not command position).
  Fixed: added `cargo run … -p reposix-sim` + `cargo run … -- seed` to Guard B.

**Why the hook fix is NOT the whole cut:** the hook is Bash-tool-only by construction. The
live repro bypassed it entirely (subprocess + in-repo worktree). The hook hardening closes
the Bash-tool spellings, but a subprocess bypass remains reachable — the honest coverage
boundary is preserved in the hook header, not deleted.

**THE REAL CUT (follow-up — v0.14.0 Wave 2):** a BINARY-SIDE refusal in `reposix init` /
sim-seed itself, since only that layer can stop a subprocess bypass. Sketch: `reposix init`
(NOT `attach` — attach legitimately adopts an existing checkout) refuses when its effective
target would nest inside the reposix SOURCE checkout / shared dev tree, WITHOUT breaking the
sanctioned `/tmp` dark-factory flow. Pair with a self-safety check that refuses to operate
when the effective `.git` is the shared repo's object store (worktree-shared config detected).

**STATUS:** DEFERRED-TO-v0.15.0. **The ACTIVE corruption vector is already CLOSED.** The PRIMARY
cut for this recurrence — the leaf-isolation hook Cases 9-11 (config-read false-positive,
git-init-bare, cargo-sim-seed spelling) — shipped in P102's D2 re-seal (commit `2ad2bf5`), and the
shared tree was repaired at `9d78d62`. What remains is a defense-in-depth BINARY-SIDE refusal
(`reposix init` — NOT `attach` — refuses when its effective target nests inside the reposix source
checkout / shared dev tree, plus a self-safety check refusing a shared-`.git` object store) that is
the ONLY layer able to stop a non-Bash-tool subprocess bypass. That is new binary code >1h, not an
eager-fix; a partial binary-side check landed at `3206a2b` (`fix(d2): binary-side reposix-init
refusal`) but the full defense-in-depth cut + cross-flow testing is a dedicated v0.15.0 hardening
phase. Deferring here does NOT re-open the active vector — it hardens the already-closed one.

## 2026-07-12 15:57 | discovered-by: C2-wave-2 (CI-gate fix-twice) | severity: MEDIUM

**What:** `.github/workflows/release.yml` (tag `v*` trigger) is CI-UNGATED — a tag cut over
a red main would still publish crates. This session gated the phase-close path and `docs.yml`
(D-CONV-4 `workflow_run`/`if success` pattern), but the tag-publish path has no CI-green
precondition.

**Why out-of-scope for the discovering work:** surfaced during the CI-gating fix-twice work,
not owned by any wave-2 implementation phase; the aggregate `v*` tag is owner-cut at
milestone-close (P111), so the fix belongs on the P110/P111 radar, not inside P106.

**Sketched resolution:** gate `release.yml` on CI green BEFORE the next `v*` tag — mirror the
`docs.yml` D-CONV-4 `workflow_run` + `if: success` pattern, or add a pre-tag CI-green
assertion step. If gating turns out <1h with no new dependency during P110/P111, eager-fix;
otherwise leave filed for the owner (the tag itself is owner-cut).

**STATUS:** RESOLVED, commit `0d05d7f` (`fix(ci): gate release.yml publish on green CI for the
tagged commit — P110 OP-8 intake row 8`; SHA verified real via `git log`). A `ci-green-gate` job
was added to `.github/workflows/release.yml` and the `plan` job now `needs` it, so a `v*` tag over
a red main will not publish. **HONESTY CAVEAT (carried forward for the owner):** the gate is
tag-triggered, so its RUNTIME behavior cannot be exercised until the P111 `v*` tag is cut — the
YAML was parse-verified but the FIRST LIVE FIRE is P111. Treat as RESOLVED-pending-first-fire.

## 2026-07-12 | discovered-by: GSD-quick (release-plz RED fix) | severity: MEDIUM

**What:** Fold release-plz (and other required workflows) into the ci-green-on-main phase-close
bar. This session fixed a persistently-RED release-plz on main (it refused a dirty CI checkout
re-dirtied by self-regenerating fleet-safety verdict JSONs) — but the RED sat UNNOTICED because
the phase-close `code/ci-green-on-main` (P0) probe hardcodes `WORKFLOW=ci.yml` and watches ONLY
ci.yml. release-plz was never on the phase-close radar, so an unwatched red release workflow
rotted silently (Global CLAUDE.md: health is a maintained asset; never let a metric you don't
watch decay). Sibling of the release.yml-CI-ungated entry above (that one is about GATING the
tag-publish on green; this one is about WATCHING release-plz's outcome at phase-close).

**Why out-of-scope for the discovering quick:** the quick's charter was the dirty-checkout fix;
widening the phase-close bar is a catalog + verifier change with open semantic questions (below)
that warrant an owner gate before P0-wiring — not inline scope.

**Sketched resolution:** parameterize `quality/gates/code/ci-green-on-main.sh`'s hardcoded
`WORKFLOW=ci.yml` into a required-workflow LIST, OR add a sibling `code/release-green-on-main`
row at post-push cadence reusing the same latest-run-conclusion logic. Catalog-first (write the
GREEN-contract row before impl) + a verifier grade.

**Open questions to resolve FIRST (a false-RED would block UNRELATED phases → owner gate):**
(1) Does release-plz run on EVERY push to main? (2) Is a 'no release needed' run concluded
`success` / `skipped` / other — so the probe treats non-failure correctly and does not false-RED
unrelated phases?

**STATUS:** DEFERRED (owner gate required). Both candidate fixes (parameterize
`ci-green-on-main.sh`'s hardcoded `WORKFLOW=ci.yml` into a required-workflow LIST, OR add a sibling
`code/release-green-on-main` row) need a `quality/catalogs/code.json` edit — and `code.json` is
FOREIGN-LOCKED in this contended tree (a concurrent lane holds uncommitted changes; the P110 drain
charter forbids touching it). Beyond the lock, the two open semantic questions above (does
release-plz run on EVERY push to main? is a 'no release needed' run concluded success/skipped?)
must be answered BEFORE wiring a P0 probe — a false-RED would block UNRELATED phases. **OWNER ASK:**
resolve those two questions, then choose list-parameterize vs a sibling `code/release-green-on-main`
row; the wiring lands once `code.json` is unlocked.

## 2026-07-12 | discovered-by: GSD-quick (fleet-safety untrack fix) | severity: MEDIUM

**Title:** Runner unit tests (`quality/runners/test_*.py`) are not collected by CI — durable
guards never run automatically.

**What:** The fleet-safety untrack fix (`3d3e60e`) shipped its DP-2 regression guard
`quality/runners/test_fleet_safety_verdicts_untracked.py` — but none of the 6
`quality/runners/test_*.py` unittest files are wired into `.github/workflows/ci.yml`. CI never
collects them, so the guard is inert in CI: a metric generated-but-not-watched, GREEN only on a
local run.

**Why out-of-scope for the discovering quick:** the quick's charter was the dirty-checkout /
untrack fix; wiring a CI collection step (with per-file env-safety triage + an owner gate on
which subset is hermetic) is a workflow + catalog change, not inline scope.

**Sketched resolution:** triage the 6 test files for env-safety (which need creds/network, e.g.
`test_realbackend`, vs pure/hermetic) then add a CI step running the env-safe subset (e.g.
`python3 -m unittest` over the vetted files) OR promote them to catalog gates under an
appropriate cadence. `test_fleet_safety_verdicts_untracked.py` MUST be one that gets wired. Do
NOT blanket `unittest discover` — that surfaces env-dependent tests.

**STATUS:** RESOLVED, commit `3f1458d` (`fix(ci): collect env-safe quality/runners unit tests in
CI — P110 OP-8 intake row 10`; SHA verified real via `git log`). A hermetic `runner-unit-tests` job
was added to `.github/workflows/ci.yml` wiring the 6 env-safe tests (explicit file list, NOT a
blanket `unittest discover`); `test_realbackend.py` is EXCLUDED — it belongs to the credentialed
`pre-release-real-backend` cadence, not the hermetic CI collection. The DP-2 guard
`test_fleet_safety_verdicts_untracked.py` is among the wired set, so it now runs automatically in
CI rather than only on a local run.

---

## 2026-07-12 20:59 | discovered-by: P111 (milestone-close CI-wait) | severity: MEDIUM

**Title:** Background `gh run watch` HANGS on already-concluded runs — flaky CI-wait in
autonomous loops (reliability).

**What:** Autonomous CI-wait loops backgrounded `gh run watch <id>`, which blocks
INDEFINITELY when pointed at a run that has ALREADY concluded — it waits for an
in-progress→completed transition that will never arrive. This hung two sessions — hang IDs
`bulqmsyrv` and `biy9yxt33` — leaving a GREEN-already run's watcher blocked forever and
stalling the phase. A reliability defect (severity MEDIUM), not a data-loss/security bug.

**Why out-of-scope for the discovering context:** the hang surfaced mid-milestone-close while
waiting on CI; swapping the wait mechanism is a reusable-tooling change (a committed helper +
its catalog verifier), not inline scope for whatever task was blocked at the time.

**Sketched resolution:** promote the ad-hoc `gh run watch` into a committed bounded-poll helper
(CLAUDE.md OP-4): poll `gh run view <id> --json status,conclusion` on an interval up to a hard
timeout, return IMMEDIATELY on an already-`completed` run, exit 0 only on conclusion `success`,
and use a distinct exit code on hard-timeout so an indefinite hang is impossible.

**STATUS:** RESOLVED, landed in this phase's ci-wait commit (COMMIT 2, subject `feat(scripts):
ci-wait.sh — bounded-poll CI helper replacing flaky gh run watch (P111)`; resolve its SHA via
`git log --grep='ci-wait.sh' --oneline` — a self-referential commit cannot embed its own final
hash, so this row names the commit by its stable subject rather than a hardcoded SHA that the
rewrite would falsify). `scripts/ci-wait.sh` implements the already-`completed` fast-path,
proven real: `scripts/ci-wait.sh 29207305260` returns exit 0 in ~1s against a concluded GREEN
run — the exact case that hung `gh run watch` at `bulqmsyrv` / `biy9yxt33`. The catalog-first
contract row `agent-ux/p111-ci-wait-helper` (minted FAIL) flips PASS on this commit.

---

## 2026-07-13 | discovered-by: B1 (v0.14.0 tag-remediation lane, mirror-reconcile investigation) | severity: MEDIUM

**Title:** Root CLAUDE.md § "Mirror-head refresh promise" conflates two distinct "mirrors"; the `reposix sync --reconcile` manual-catch-up prose is empirically wrong. **[fix-twice]**

**What:** The root `CLAUDE.md` § "Mirror-head refresh promise (qualified, ADR-010 RBF-LR-04)"
conflates TWO different things both called "mirror": (a) the cache's `refs/mirrors/<sot>-head`
OBSERVABILITY ref inside the local bare-repo cache, and (b) the EXTERNAL GitHub mirror REPO
whose content a fresh `git clone` actually reads. The § implies `reposix sync --reconcile` is
the operator catch-up move for a stale mirror, but B1 established empirically that `--reconcile`
heals ONLY the LOCAL cache (`oid_map` / cursor) plus the cache-internal `refs/mirrors/*` ref —
it does NOT refresh the external GitHub mirror repo content that a fresh clone reads. So the
"Manual catch-up if it ever needs a forced refresh: `reposix sync --reconcile`" prose points
operators at a command that will not fix the symptom they are chasing (a stale external mirror).
Cross-refs to correct in lockstep: `docs/concepts/dvcs-topology.md` (same manual-catch-up claim).

**Why out-of-scope / why FILE not eager-fix:** the CORRECT rewrite depends on a PENDING MANAGER
DECISION on the mirror-refresh mechanism (escalated by B1) — until the owner blesses what the
authoritative external-mirror-refresh path actually IS (a dedicated `reposix` verb? webhook-driven?
a documented "clone reads SoT-current, mirror lags" acceptance?), any doc edit would just encode a
guess. Bottom-up triage disposition: FILE, resolve AFTER the manager blesses the mirror-refresh
mechanism (B1 escalation).

**Sketched resolution:** once the manager blesses the mechanism, rewrite the § to (1) name the two
mirrors distinctly (cache observability ref vs external GitHub mirror repo content), (2) state which
refresh path is authoritative for the EXTERNAL mirror, and (3) correct the `--reconcile` catch-up
claim to its true scope (rebuilds local cache `oid_map`/cursor + the `refs/mirrors/*` cache ref, NOT
external mirror repo content). Fix-twice: doc + the `docs/concepts/dvcs-topology.md` twin in the same
change.

**STATUS:** DEFERRED — pending the B1 mirror-refresh manager decision. Resolve AFTER the manager
blesses the mirror-refresh mechanism (B1 escalation).

---

## 2026-07-13 | discovered-by: B1 (v0.14.0 tag-remediation lane, mirror-reconcile investigation) | severity: MEDIUM

**Title:** Remote-helper lost-update teaching string prints `git pull --rebase`, but `attach` wires pull to the STALE mirror `origin` — the printed recovery does not resolve backend drift (Rust-compiler-grade-UX violation). **[fix-twice]**

**What:** On a lost-update rejection the `reposix-remote` git helper prints a teaching string of the
form `Run: git pull --rebase`. But `reposix attach` wires `fetch`/`pull` to read from the STALE
mirror `origin` — so a naive `git pull --rebase` re-pulls the SAME stale version and does NOT
resolve the backend drift that triggered the rejection. Un-sticking actually requires fetching
backend-current THROUGH the reposix bus remote (SoT-authoritative), not the mirror `origin`. The
printed recovery command therefore does not teach the WORKING recovery — a direct violation of the
project's Rust-compiler-grade-UX north star (every user-facing error must give a copy-paste recovery
command that actually works).

**Why out-of-scope / why FILE not eager-fix:** the correct teaching string names the authoritative
backend-current fetch path, which is exactly the mirror-refresh mechanism the manager decision (B1
escalation) is still pending on — editing the helper string now would hard-code an unratified
recovery move into load-bearing helper code. Bottom-up triage disposition: FILE.

**Sketched resolution:** once the manager blesses the mechanism, update the helper's lost-update
teaching string (`crates/reposix-remote`) to name the working recovery — fetch backend-current via
the reposix bus remote (SoT-authoritative), NOT `git pull --rebase` against the stale mirror
`origin` — with a copy-paste command per the error-message convention (`crates/CLAUDE.md` §
Error-message convention). Fix-twice: helper string + the `docs/guides/troubleshooting.md` §
"DVCS push/pull issues" recovery prose in the same change. Add a test asserting the string names
the bus-remote fetch.

**STATUS:** DEFERRED — pending the B1 mirror-refresh manager decision. Resolve AFTER the manager
blesses the mirror-refresh mechanism (B1 escalation).
