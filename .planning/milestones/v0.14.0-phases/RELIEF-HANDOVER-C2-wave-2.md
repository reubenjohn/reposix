# RELIEF-HANDOVER-C2-wave-2.md — v0.14.0 wave-2 hardening, C2 relief, 2026-07-12

Written by the C2 coordinator-of-coordinators for the v0.14.0 wave-2 hardening
milestone, relieving proactively at the ~100k absolute-token line (ORCHESTRATION §3).
This **overwrites the stale predecessor handover** at this same path (that one was
written at the P105-closed boundary; ten more commits have landed since — read this
file, not the one you might have cached).

## SHARED-TREE HAZARD (read before ANY tool call)

`main == origin/main == b037876`, shared working tree
`/home/reuben/workspace/reposix`. A herdr manager session (read-only) + other panes
SHARE this tree. FOUR untracked foreign dirs are present and NOT ours:
`.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/`,
`.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/`,
`scripts/demos/`, `scripts/dev/`. There is also a foreign `stash@{0}` ("WIP on main:
faf3d16 docs(22): create phase plan — 3 plans across 2 waves for BENCH-01..04").
Do **NOT** touch it. At session start a foreign `M quality/catalogs/doc-alignment.json`
was also observed in the wider environment snapshot, but as of this handover the
tracked tree is CLEAN of that modification (see §1) — re-check before trusting either
state.

Hard rules for the successor:
- NEVER `git add .` / `git add -A` / `git commit -a` / `git clean` / `git stash drop`.
  Every commit uses EXPLICIT paths only.
- ONE cargo invocation machine-wide (build-memory budget; VM has OOM-crashed on
  parallel builds).
- One-tree-writer-at-a-time discipline: serialize writer lanes; read-only prep may
  run in parallel.
- Leaf isolation HARD-STOP: `reposix init` / sim-seed / `git commit`/`config` for
  ANY test/fixture setup MUST `cd` into a `/tmp` clone in the SAME Bash invocation —
  never in this shared tree. (This was the exact root cause the P0 leaf-isolation
  incident this session closed — see §2 Phase 0.)

## 1. Ground truth (git)

- `HEAD == origin/main == b037876619804ee04aa867a02719399d061062da7`. Branch `main` is
  up to date with `origin/main` (no ahead/behind).
- `git status` on the tracked tree: **CLEAN** — zero modified/staged tracked files.
  Only the four foreign untracked dirs listed above appear (verified live this
  session via `git status` — no `M quality/catalogs/doc-alignment.json` present at
  this moment).
- Owner-gated branch `backup-lost-update-424d367` exists **locally only** — do NOT
  push it; landing it to GitHub main is an owner-named-target decision routed to L0
  (see §5).
- Commits landed this C2 rotation (newest first, all pushed, all unbiased-verifier
  PASS before landing):
  1. `b037876` docs(good-to-haves): file GOOD-TO-HAVES-10 — orphaned Fork-B assert on
     P94 Fork-A row
  2. `13de686` docs(p108): close prune-completeness-gate paperwork; file slug→id
     remainder
  3. `10bd508` fix(P109): RBF-FW-11 grandfather keys off landing commit, not null
     last_verified
  4. `1cb9dd1` test(P109): GREEN contract for RBF-FW-11 grandfather-commit rule
     (catalog-first)
  5. `72ae517` docs(good-to-haves): file 4 fleet-safety persist-gate follow-ups
     (v0.14.0 GTH-05..08)
  6. `309f0b6` fix(quality): deterministic shell-subprocess verdicts — stop
     fleet-safety JSON re-dirty (D-P96-01 extended)
  7. `3206a2b` fix(d2): binary-side reposix-init refusal + un-break sim-server start
  8. `2ad2bf5` fix(d2): re-seal leaf-isolation guard — config-read false-positive +
     git-init-bare + cargo-sim-seed + file live repro
  9. `9d78d62` docs(planning): refresh manager handover — post-incident (shared-tree
     corruption resolved)
  10. `f2d527a` docs(handover): v0.14.0 C2 relief at clean wave-2 boundary — P105
      closed GREEN (the STALE predecessor this file replaces)

Numbered deviations the successor MUST know:
1. Phase 0 (D2 leaf-isolation re-seal) was NOT on the original wave-2 plan — it was
   an emergent P0 incident (a P106 leaf ran `reposix init` inside
   `.claude/worktrees/...` of the SHARED repo instead of `/tmp`, bypassing the
   Bash-tool-only PreToolUse hook). It consumed the first slice of this rotation and
   is now closed at the product layer (see §2).
2. The D2 guard fix ALSO fixed a false-positive: read-only `git config --get` /
   `git config --list` are now correctly ALLOWED (previously flagged).

## 2. Wave/cycle state

| Wave / Item | Plan | State | Commits |
|---|---|---|---|
| Phase 0 — D2 leaf-isolation re-seal | emergent (not pre-planned) | DONE, 11/11 verifier PASS | `2ad2bf5`, `3206a2b` |
| fleet-safety persist-gate churn fix | wave-2 planned | DONE, 4/4 verifier PASS | `309f0b6`, `72ae517` |
| P109(a) RBF-FW-11 | wave-2 planned | DONE, catalog row PASS | `1cb9dd1`, `10bd508` |
| P108 prune-gate paperwork closure | wave-2 planned | DONE, catalog row PASS | `13de686`, `b037876` |
| tutorials 01/02/04/05 self-start-sim rewrite | wave-2 planned | **NOT STARTED** | — |
| git-2.34 CI boundary job | wave-2 planned | **NOT STARTED** (scope brief ready, see §6) | — |
| shell-coverage floor (~11.98% < 13%) | wave-2 planned | NOT STARTED (pre-existing, non-blocking) | — |
| OP-8 file splits (SURPRISES-INTAKE / GOOD-TO-HAVES oversize) | milestone slot | NOT STARTED | — |
| OP-9 distill into RETROSPECTIVE.md | milestone-close | NOT STARTED | — |
| OD-4 launch-readiness | scope-only stub | DO NOT START | — |
| 9th probe `pre-release-real-backend` + STATE cursor advance | milestone-close | NOT STARTED | — |

**Named incident to read before dispatching any executor near test/fixture setup:**
the Phase 0 leaf-isolation incident (SURPRISES-INTAKE.md, ~line 354, HIGH severity).
Root cause: a subprocess/worktree code path ran `reposix init` inside a
`.claude/worktrees/` copy of the SHARED repo rather than an isolated `/tmp` clone,
which is invisible to the Bash-tool-only PreToolUse hook. Fix landed at the product
layer: `reposix init` now refuses to run inside an existing git-worktree root
(fail-closed), plus guard extensions covering `git init --bare` and cargo-sim-seed
paths. **RESIDUAL, still open, RAISED to L0** (not closed this session): (a)
worktree-shared `.git` object-store self-safety, and (b) the non-Bash subprocess
boundary in general (defense-in-depth — the one observed vector is closed, but the
class of hazard is not exhaustively proven closed).

## 3. Binding constraints (unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide; never `--no-verify`;
push only at green (`git push origin main` BEFORE the verifier-subagent dispatch, per
phase); commit-trailer format (`Co-Authored-By` + `Claude-Session`); model tiering
(fable → opus/sonnet/haiku by complexity, never fable at a leaf). Leaf isolation
HARD-STOP (see hazard box above) is the constraint that bit this session — treat it
as load-bearing, not advisory.

## 4. Litmus / gate / REOPEN state

- D2 leaf-isolation guard: 11/11 verifier PASS this session (`2ad2bf5`, `3206a2b`).
- fleet-safety persist-gate: 4/4 verifier PASS (`309f0b6`, `72ae517`); both verdict
  producers now converge through `quality/runners/_shell_verdict.py`; volatile
  timestamp/transcript_path fields removed from committed JSON, proven idempotent
  across 2 consecutive runs.
- `structure/claim-vs-assertion-audit-required` catalog row: PASS (`1cb9dd1`,
  `10bd508`).
- `agent-ux/p94-pagination-prune-completeness-gate` catalog row: PASS — verifier ran
  `cargo test -p reposix-cache --test pagination_prune_safety` live, 3/3 passed
  (`13de686`, `b037876`).
- **Open waiver expiry clocks:**
  - Tutorials 01/02/04/05 self-start-sim waiver: expires **2026-09-15**.
  - OP-8 structure waiver (SURPRISES-INTAKE / GOOD-TO-HAVES oversize): expires
    **2026-08-08**.
  - ADR-010 slug→id waiver (filed as GTH-09, MEDIUM-HIGH, this session): expiry not
    yet set — routed as an owner scope question (see §5), do not assume a default
    clock.
- No REOPEN state currently active; nothing failed and got reopened this rotation.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**De-facto decisions made live this rotation:**
- Chose product-layer fix (refuse-on-worktree-root) over hook-layer-only patch for
  the D2 incident, on the reasoning that the hook can't see non-Bash subprocess
  paths — this is a real architectural call, not yet written into `crates/CLAUDE.md`
  or `docs/how-it-works/trust-model.md` as a standing pattern. Successor/L0 should
  decide whether this warrants a CLAUDE.md doctrine update (fix-it-twice, OP-3
  meta-rule) beyond the SURPRISES-INTAKE entry.
- Rejected a naive fix for RBF-FW-11 (would have opened a dodge hole) in favor of a
  frozen `GRANDFATHERED_NULL_LV` set keyed off landing commit — deliberate, already
  landed, no further action needed.

**Noticed-not-yet-filed, now triaged (do not re-triage):**
- examples/README stale `reposix-sim` ref → **FALSE ALARM**, binaries match
  Cargo.toml, no action.
- P106/P113 collision concern → **FALSE ALARM**, P113 does not exist.
- RBF-FW-07a → already resolved at P90, no action.
- git-2.34 "verified down to 2.25" prose claim → **REAL**, routed into the git-2.34
  CI-boundary queue item (§6 item 2) — this is the one open thread from noticing that
  still needs code/doc action.

**New owner-ask surfaced this rotation (not yet answered):** ADR-010 slug→id
durable-create (filed as GTH-09) reads on the ROADMAP as a v0.14.0 HEADLINE item but
is currently unstarted. Someone with milestone-scope authority needs to decide: ship
it this milestone, or explicitly defer past v0.14.0. Do not silently assume either —
raise to L0/owner before scheduling an executor against it.

## 6. Precise next steps (successor runbook)

1. **Spot-check ground truth first.** Re-run `git log --oneline -5`, `git status`,
   `git stash list` yourself before trusting §1 of this file — foreign panes may have
   moved state since this was written.
2. **Next writer lane: tutorials 01/02/04/05 self-start-sim rewrite.** Waiver expires
   2026-09-15, so it is not on fire, but it's the next queued wave-2 item with no
   open scope questions. Dispatch a phase-coordinator (C1) for it. Needs sim runs —
   MUST use `cargo run -p reposix-sim` + `/tmp`-clone isolation per the leaf-isolation
   hard-stop; do not repeat the Phase 0 incident.
3. **Then: git-2.34 CI boundary job.** Scope brief (already gathered, ready to hand a
   C1 verbatim):
   - The claim is prose-only AND self-inconsistent: `README.md` / `CONTRIBUTING.md:24`
     say git ≥2.34 is "required"; `CLAUDE.md:98-101` + `crates/CLAUDE.md` say
     "recommended, WARN not ERROR"; "verified down to 2.25" is anecdotal (dev box),
     not CI-gated.
   - No git-version matrix exists in CI today; all jobs run `ubuntu-latest` (~git
     2.43).
   - `dark-factory.sh sim` (`ci.yml:156`) is the existing e2e stateless-connect proof
     — run it under a pinned older git to prove the 2.34 boundary.
   - `doctor.rs:513` is where version classification (WARN vs ERROR) lives.
   - Pinning precedent exists: gitleaks binary pin + kcov built-from-tag
     (`ci.yml:68-79`, `:390-402`) — no git pin exists yet.
   - **ROI call for the executor to make explicitly:** building git 2.25 from source
     in CI is a real cost. Prefer: (a) fix the doc inconsistency (pick one true
     floor, update all three docs to match), (b) cheaply prove the 2.34 boundary
     (e.g. pin git 2.34 exactly, not 2.25), (c) if a full 2.25-floor CI job is judged
     a big lift, file it to GOOD-TO-HAVES rather than build it now. This lane also
     OWNS closing the noticed-item #2 prose-only-2.25-claim thread from §5.
4. **shell-coverage floor** — MEDIUM, already filed, ~11.98% < 13% floor,
   pre-existing/non-blocking, needs `kcov`. Low priority; pick up after the above.
5. **OP-8 file splits** — SURPRISES-INTAKE.md and GOOD-TO-HAVES.md are 5-6x
   oversize; structure waiver expires 2026-08-08. **Caution:** both files are
   actively appended by concurrent sessions and the foreign stash may also touch
   them — sequence the split carefully (read-modify-write with a fresh diff check
   immediately before commit, explicit path only) to avoid clobbering a concurrent
   append.
6. **OP-9 distill** — at milestone-close, distill SURPRISES-INTAKE + GOOD-TO-HAVES +
   run findings into a new `.planning/RETROSPECTIVE.md` section BEFORE archiving.
   Do this after items 2-5, not before (need the full intake list settled first).
7. **OD-4 launch-readiness** — scope-only stub. Do NOT start implementation; if
   touched at all this rotation, only scope/clarify, then hand back.
8. **Milestone-close checklist** (last, after 1-7 land): run the non-skippable 9th
   probe `python3 quality/runners/run.py --cadence pre-release-real-backend`
   (`agent-ux/milestone-close-vision-litmus-real-backend`, never waived); confirm
   `git push origin main` landed before any verifier dispatch; advance the
   `.planning/STATE.md` cursor (note: STATE.md has no discrete P108 cursor field
   today — this is a narrative advance, coordinate wording with the herdr-manager
   session to avoid a stomped edit); dispatch OP-9 distill (item 6) if not already
   done; write the close-out RAISE LIST for L0 covering the two owner-gated items
   below.
9. **Owner-gated items — raise to L0, do not execute yourself:**
   - Land `424d367` (branch `backup-lost-update-424d367`) to GitHub main;
     dependabot #64/#65/#66 (cargo audit currently shows 0 live vulnerabilities →
     recommend close-as-redundant, but owner confirms); gh404 live-GitHub
     read-only verify.
   - ADR-010 slug→id durable-create (GTH-09) ship-this-milestone-or-defer decision
     (§5).
