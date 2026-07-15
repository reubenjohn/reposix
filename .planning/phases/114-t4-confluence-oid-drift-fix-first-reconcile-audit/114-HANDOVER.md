# 1-HANDOVER.md — P114 C1 relief, Wave-1/Wave-2 boundary, 2026-07-15

Written by the outgoing P114 C1 phase-coordinator (dispatched by L0 workhorse #26),
relieving at ~116k tokens of own context — past the ~100k soft-relief line, at a clean
Wave-1-complete / Wave-2-not-started boundary. Successor is a fresh C1 coordinator
identity dispatched by L0 (this is a single-phase milestone slice; no C2 is in play).
**Do not** touch `.planning/MANAGER-HANDOVER.md` or `.planning/SESSION-HANDOVER.md` —
those belong to the concurrent w1:p7 manager session sharing this tree.

**Read order:** this file in full → `114-01-SUMMARY.md` (Wave-1 close, code-review
verdict) → `114-RESEARCH.md` OQ1 (pre-ADF risk to SC1) → `114-02-PLAN.md` (Wave-2 scope,
not yet executed) → `.planning/ORCHESTRATION.md` §3 if the template itself is in doubt.

## 1. Ground truth (git)

HEAD = `eaf24d9`, `origin/main` matches (0 ahead / 0 behind). Working tree CLEAN at
relief. `eaf24d9` + `276beb8` are **the MANAGER's** 2 doctrine commits (polling-model /
no-background-watcher) — do NOT amend, drop, or rebase past them; all Wave-2 commits go
ON TOP, unmodified.

Commits since the last known-clean sha (`fb38189`, prior L0 relief), in order:

| Commit | Summary |
|---|---|
| `4652c7d` | docs(planning): file 2 carry-forward noticings (gsd-sdk footgun, stale v0.12.0 catalog example) |
| `88f2e2c` | docs(planning): file pre-push shell-coverage timing-creep noticing (amendment 4) |
| `47fa803` | test(114-01): add list_and_get_render_parity RED for Confluence oid-drift |
| `9908fcc` | fix(114-01): request body-format=atlas_doc_format on Confluence list path |
| `bf005bc` | docs(114-01): render-parity SUMMARY + RESEARCH OQ fold-in + nextest GTH |
| `db12187` | docs(114-01): correct RESEARCH OQ1 pre-ADF overstatement + file cursor-guard GTH |
| `6f15138` | test(114-01): lock cursor re-append carries body-format on >100-page follow |
| `276beb8` | docs(planning): manager watch switches to polling model, <=1h cap (owner directive 2026-07-15) — **NOT this coordinator's commit** |
| `eaf24d9` | docs(planning): polling-model doctrine moves to /herdr-manager skill; handover keeps pointer — **NOT this coordinator's commit** |

At relief time, `gh run list --branch main --limit 5` showed the newest `main` push CI
(`eaf24d9`, docs-only, run `29431686591`) as **in_progress** (started ~16:16:02Z);
`release-plz` also in_progress on the same push; `CodeQL` on that push already
**success**. The prior push's (`276beb8`) CI had not yet been re-checked for GREEN by
this coordinator before relief — **the successor's first move must be to confirm
main's newest CI run concluded success** (the `code/ci-green-on-main` P0 post-push
probe) before dispatching any Wave-2 tree-writer, per the standing "never open the next
phase/wave over a red main" rule. No deviation from plan requiring a revert exists in
this commit range — all six 114-01 commits are a straight-line Wave-1 execution.

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| 1 — FIX-01 (Confluence list/get render-parity) | `114-01-PLAN.md` | **DONE, PUSHED, CI GREEN** at `6f15138` (verified before the two manager commits landed on top) | `47fa803`, `9908fcc`, `bf005bc`, `db12187`, `6f15138` |
| 2 — FIX-02 (reconcile-audit: prove --reconcile does NOT heal systematic oid-drift + doc corrections) | `114-02-PLAN.md` | **NOT STARTED** | none |

Wave-1 code-review verdict: **APPROVE-WITH-NITS**. All 4 critical checks PASS (Pitfall 1
drift-check left intact, Pitfall 2 zero added round-trips, pagination correctness on the
>100-page cursor path, test-asserts-its-claim). Nits #1/#2 were absorbed inline; nit #3
was filed as GTH-11 (see §5). No named-incident post-mortem for Wave-1 — it closed clean.

## 3. Binding constraints (unchanged)

- One tree-writer at a time (single-writer discipline, ORCHESTRATION §2).
- **ONE cargo invocation machine-wide** — prefer `-p reposix-core` / `-p reposix-cache`
  over `--workspace`; `cargo-nextest` is **not installed** in this environment, use
  `cargo test` (see GTH-10).
- Leaf reposix/sim/git test setup MUST run in a throwaway `/tmp` clone with `cd` in the
  SAME Bash invocation — mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh`
  (fail-closed, exit 2).
- No `--no-verify`, ever.
- No tag push — manager/L0 cuts tags, not this coordinator.
- **TARGETED staging only** (`git add <specific path>`) — never `git add -A`/`.`; never
  touch `.planning/MANAGER-HANDOVER.md` / `.planning/SESSION-HANDOVER.md`.
- Push cadence: `git push origin main` BEFORE the verifier subagent; if push is
  REJECTED because origin advanced, `git pull --rebase origin main && git push` — NEVER
  force, NEVER stash the manager's concurrent work. If the tree is unexpectedly dirty on
  pickup (manager mid-edit), STOP and report to L0 rather than proceeding.
- CI-confirmation is **inline bounded polling only, ≤1h cap** — **no background
  self-resume watchers** (owner directive 2026-07-15; they are a liveness risk). Confirm
  `code/ci-green-on-main` (P0, via `python3 quality/runners/run.py --cadence post-push
  --persist`) GREEN before ever opening the verifier or the next wave.
- Commit-trailer format: `Co-Authored-By: Claude <model> <noreply@anthropic.com>`.
- Model tiering: this coordinator ran fable-tier; dispatch gsd-executor (sonnet default)
  → gsd-code-reviewer → gsd-verifier per ORCHESTRATION §1.

## 4. Litmus / gate / REOPEN state

- No litmus gate is currently in a REOPEN state for P114.
- Wave-1's own gate history: the `list_and_get_render_parity` test went RED-then-GREEN
  within Wave-1 itself (`47fa803` RED → `9908fcc` fix → GREEN), fully resolved before
  push; no open waiver.
- Wave-1 `git push` **pre-push hook timing**: 127s observed — see §5 RAISE item, not a
  gate failure but a WARN-worthy regression against the known ~91s baseline.
- Post-push `code/ci-green-on-main` (P0) probe: **not yet re-run by this coordinator**
  against `eaf24d9` (in_progress at relief, run `29431686591`) — successor's first
  action, §6 step 1.
- Real-backend acceptance gates (SC1 live TokenWorld checkout, SC2
  `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`) have **not yet
  been run this phase** — they gate phase-close, after Wave 2 and the verifier, not
  before.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

1. **Doc-honesty fold-in still outstanding, raised by the Wave-1 follow-up executor,
   MUST NOT be dropped:** `114-01-SUMMARY.md` around line ~100 still carries the same
   "pre-ADF pages handled by translate's storage fallback" overstatement that was
   already corrected in `114-RESEARCH.md` OQ1 (`db12187`). It needs tightening to align
   with the SUMMARY's own honest "Residual gap" section — render-parity resolves
   ADF-native drift only; pre-ADF drift persists, answerable only by the live SC1 gate.
   This is a small (<1h) fix — fold it into Wave-2's doc-correction pass, not a separate
   lane.
2. **Two GTHs already filed this phase, do NOT re-file:** GTH-10 (cargo-nextest not
   installed, XS) and GTH-11 (Wave-1 code-review nit #3 — the cursor
   `contains("body-format=")` guard could false-skip on a foreign value that happens to
   contain that substring, XS).
3. **RAISE for L0/manager — pre-push hook timing regression.** Wave-1's `git push` took
   **127s**; the last known-filed baseline is ~91s (see commit `88f2e2c`, "file pre-push
   shell-coverage timing-creep noticing"). +40% is materially worse than the existing
   filed WARN. Watch the Wave-2 push: if it lands consistently around ~127s too, this
   needs a real look (kcov shell-coverage / full-workspace clippy creep), not just
   another WARN entry.
4. **Known open risk to SC1 (not yet a decision, a live unknown):** if live TokenWorld
   page `7766017` turns out to be pre-ADF (storage format, not ADF-native), the Wave-1
   render-parity fix (list path now requests `atlas_doc_format` only, no storage
   fallback) will NOT cover it — `OidDrift` would persist on that page and SC1 would go
   RED, requiring a Wave-2-adjacent follow-up (list-path storage fallback). This is
   `114-RESEARCH.md` open question OQ1, resolvable only by actually running the live
   SC1 checkout — do not pre-emptively "fix" this without first observing the real
   failure mode (Pitfall 3, §6).

## 6. Precise next steps (successor runbook)

1. **Confirm main is GREEN before touching anything.** `gh run list --branch main
   --limit 5` (or equivalent) to check run `29431686591` (CI on `eaf24d9`) and the
   `release-plz` run concluded — then run `python3 quality/runners/run.py --cadence
   post-push --persist` and confirm `code/ci-green-on-main` (P0) is GREEN. If red,
   STOP and diagnose before dispatching Wave 2 — never open Wave 2 over a red main.
2. **Dispatch a gsd-executor (sonnet) for Wave-2 (114-02, FIX-02)** per `114-02-PLAN.md`
   scope:
   - New test file `crates/reposix-cache/tests/oid_drift_reconcile.rs` with a
     `DriftingMock` `BackendConnector` (holds `full` bodies + an `aligned: AtomicBool`;
     list returns EMPTY bodies when not aligned, get always returns real bodies). Three
     tests: (a) `pre_fix_divergent_bodies_trigger_oid_drift` — drift repro →
     `Error::OidDrift`; (b) `reconcile_does_not_clear_stale_list_oid_while_bodies_diverge`
     — a second `build_from()` leaves the stale oid unchanged, `read_blob` still errors
     (empirical proof reconcile does NOT recover this class — backs SC4); (c)
     `aligned_bodies_resolve_without_drift` — aligned bodies → no drift.
   - Doc corrections (currently OVERCLAIM that `--reconcile` heals oid-drift generally):
     `crates/reposix-cache/src/error.rs` `OidDrift` doc — name BOTH causes
     (eventual-consistency race, reconcile CAN heal; systematic backend
     rendering-representation mismatch, reconcile CANNOT) — include the verbatim strings
     `"systematic backend rendering-representation mismatch"` and `"CANNOT"`.
     `crates/reposix-cli/src/sync.rs` (module doc + `run` fn doc) — caveat `--reconcile`
     heals tree↔oid_map coherence + races but NOT systematic rendering mismatches;
     include `"systematic"` at both sites. `crates/reposix-cli/src/main.rs` `Sync` clap
     doc — consistency pass, include `"systematic"`. **Do NOT edit**
     `crates/reposix-cache/src/cache.rs::write_last_fetched_at` (~L575-592) — confirmed
     accurate already, read-only.
   - **Deviation trap (Pitfall 3):** do not over-correct docs to imply "reconcile never
     heals oid-drift" — it DOES heal the eventual-consistency-race class; the fix is
     precision, not a blanket claim reversal.
   - Fold in the §5 item 1 SUMMARY correction as part of the same doc pass.
3. **Dispatch gsd-code-reviewer on the Wave-2 diff.** RED loops back to a fresh executor
   pass; do not push over a RED review.
4. **Push (targeted staging), then inline-poll CI (≤1h cap, no background watcher),
   then re-run `python3 quality/runners/run.py --cadence post-push --persist`** and
   confirm `code/ci-green-on-main` (P0) GREEN before proceeding.
5. **Dispatch gsd-verifier** to produce `VERIFICATION.md` with goal-backward
   catalog-row grading. RED loops back to the offending wave (1 or 2).
6. **Real-backend acceptance (SC1/SC2), only after step 5 is GREEN:**
   - `source .env` in the SAME shell invocation as the gate/run.py call.
   - Run `scripts/refresh-tokenworld-mirror.sh` FIRST as a pre-step (else false-negative
     on mirror lag).
   - SC1: live TokenWorld `git checkout -B main` INCLUDING page `7766017` — zero
     oid-drift abort.
   - SC2: `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` GREEN.
   - Protected fixture pair `7766017`/`7798785` — NEVER delete.
   - If creds/substrate are absent (no `.env` creds or default allowlist), report
     **NOT-VERIFIED honestly** — never fake or skip-as-pass, never hit a real backend
     without creds.
   - If SC1 goes RED on `7766017` specifically because it's pre-ADF (§5 item 4), that is
     an expected/named risk, not a surprise — escalate as a genuine follow-up scope
     decision (list-path storage fallback), not a silent patch.
7. **Advance `STATE.md`** — cursor + `completed_phases` reflect P114 complete — as the
   final committed step of phase-close, after 6 confirms GO.
8. **Report to L0**: RAISE LIST (carry forward §5 item 3, the pre-push timing
   regression, if it recurs on the Wave-2 push too), intake disposition (GTH-10, GTH-11
   already filed — do not re-file), final verdict, and this handover's commit SHA for
   provenance.
