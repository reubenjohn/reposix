# Phase 105 — RBF-LR-03 rebase-recovery reconciliation

**Requirement:** RBF-LR-03 (rebase-recovery facet — the documented `git pull --rebase &&
git push` recovery that does NOT reconcile). Umbrella label shared with ADR-010's
`SotPartialFail` / slug→id facets; THIS phase scopes ONLY the pull-rebase abort.
**Mode:** yolo · **nyquist_validation:** true · **Backend:** sim only
**Planner-researcher:** single lane (research + plan fused per dispatch)
**Ground truth at plan time:** HEAD == origin/main == 86dbf94, tree clean.

---

## 1. Root cause (empirically confirmed, file:line)

**`crates/reposix-remote/src/fast_import.rs:61-79` — `emit_import_stream` mints the
client tracking commit `refs/reposix/origin/main` ("Sync from REST snapshot") as a
PARENTLESS commit (no `from` directive) with a fixed identity (`committer … 0 +0000`).**

Because the imported commit has no parent, its SHA is a pure function of its tree +
fixed identity + fixed message. Consequences:

- **SoT tree unchanged between two fetches** → identical tree → identical SHA → git sees
  the ref already at that value → no-op → fetch appears to work.
- **SoT tree changed (drift, from ANY source)** → different tree → different *root*
  commit SHA that does **not** contain the client's current `refs/reposix/origin/main`
  tip → `git fast-import` refuses the ref update:
  ```
  warning: Not updating refs/reposix/origin/main (new tip <X> does not contain <Y>)
  fatal: error while running fast-import
  ```
  `git pull --rebase` aborts here; the local commits can never be replayed; `git push`
  is then a no-op (ref unchanged). Recovery is impossible without a re-clone.

This is the `import`-capability fetch path (`crates/reposix-remote/src/main.rs:191`
advertises `import`; `handle_import_batch` at `main.rs:326-371` calls
`list_records` (FULL list, `main.rs:350`) → `emit_import_stream` at `main.rs:367`).

### Reconciling the two candidate framings — they are TWO DIFFERENT bugs

| Framing (from dispatch) | Verdict | Evidence |
|---|---|---|
| **Docs framing** — cache rebuilds a snapshot commit not a descendant of the tracking tip → fast-import aborts | **CORRECT mechanism, WRONG trigger scope.** The non-descendant commit is minted by the *client import path* (`fast_import.rs`), NOT the cache. The cache's `refs/heads/main` is a correct linear chain (verified: `031cb65→bdfd8d4→b88398a`, every `parent = prior tip`). The break happens on ANY SoT tree change — a peer `git push` breaks it exactly as an external REST write does. | `repro2.sh`: both scenarios fail identically with the exact quoted error. |
| **Code framing** — `last_fetched_at` cursor advanced only on `SotOk` (`write_loop.rs:309`), not on reject; stale cursor → incomplete `list_changed_since` delta → stale advertised head | **RED HERRING for the pull-rebase abort.** The import path uses `list_records` (full), never `list_changed_since` — the cursor cannot influence the advertised head on this path. HOWEVER the cursor bug is REAL and manifests as a **separate lost-update** (see §6 RAISE). | `main.rs:350` = full list; `repro4.sh` = confirmed silent lost update. |

**The docs claim "a git-push conflict recovers fine; only an external REST write breaks
it" is FALSE.** The true discriminator is not the drift *source* but whether the SoT
tree changed at all — and (secondarily) `import` vs `stateless-connect` (see §5).

### Fix mechanism — empirically validated in isolation

`verify_fix.sh` fed `git fast-import` two streams against a seeded tracking ref:
- parentless changed snapshot → `does not contain`, exit 1, ref unchanged (bug reproduced);
- identical snapshot **+ `from <current-tip>`** → exit 0, ref fast-forwards, linear
  2-commit history.

Chaining the snapshot onto the current tracking tip makes every fetch a fast-forward, so
`git pull --rebase` replays local commits cleanly and `git push` converges.

---

## 2. Fix approach (git-native; owner taste honored)

**Structural fix at the fetch/export level — NO client-side ID remapping, NO protocol
change, NO new dependency.** Backend still owns identity (record ids unchanged); the
client works in slugs (`issues/<id>.md`); we only give the synthesized tracking commit a
correct parent pointer so the fetch is a self-reconciling fast-forward.

**Change:** `emit_import_stream` gains a `parent: Option<gix::ObjectId>` (or `&str` hex)
parameter and emits a `from <parent>` line in the commit block when `Some`. The caller
`handle_import_batch` resolves the parent by shelling `git rev-parse --verify
refs/reposix/origin/main` (GIT_DIR is set by git during the helper RPC — the SAME
proven pattern `bus_handler.rs:455-462` already uses for `git rev-parse`). First fetch
(ref absent) → `None` → parentless root commit (correct seed behavior, unchanged).

**No-op churn guard (required, not optional).** With a parent, an *unchanged* SoT would
still mint a new empty commit every fetch (ref grows unboundedly). Before emitting the
commit, compare the freshly-built tree against the parent commit's tree
(`git rev-parse <parent>^{tree}`, or compute the tree oid in-process and compare): if
equal, emit a stream that leaves the ref at `<parent>` (a bare `reset
refs/reposix/origin/main` + `from <parent>` with no `commit`) so the fetch is a true
no-op. This mirrors the existing export-side no-op detection (`saw_commit`,
`fast_import.rs:96-112`).

**Taint note (OP-2).** The `git rev-parse` argument is the static literal
`refs/reposix/origin/main` — not a remote-influenced byte — so no `Tainted<T>` routing
concern. Keep it a fixed string; never interpolate a remote value into the ref name.

**Rejected alternatives (documented so the executor doesn't reopen them):**
- *`feature force` in the import stream* — makes fast-import force-update the ref. Works,
  but discards the descendant invariant (a force is a lie about history); it is the
  "client-side special-case patch" smell the dispatch forbids. Rejected.
- *Delete the `import` path, force `stateless-connect`* — stateless-connect reads the
  cache's already-chained history and likely fast-forwards on modern git (§5), but it is
  BROKEN on git 2.25.1 (`fatal: protocol error: bad line length 2`, the `0002`-framing
  gotcha) and the sim quickstart supports git ≥ 2.25. Removing `import` breaks old-git
  users. Rejected — fix `import`, don't remove it.

---

## 3. Regression-test design (reproduces drift→reject→pull--rebase→push-succeeds against reality)

Two layers, both sim-backed, both against the live helper (no wiremock-only shortcut —
wiremock covers reject *emission* but cannot exercise the client-side fast-import ref
update that IS the bug):

**A. End-to-end shell gate (`quality/gates/agent-ux/rebase-recovery-reconciles.sh`).**
Faithful port of `repro2.sh` (committed as the phase artifact). Starts the sim on an
isolated port + `REPOSIX_CACHE_DIR`, two `reposix init` clones in `/tmp`, sets a non-t@t
fixture identity locally. Asserts BOTH:
- **Scenario A (peer git push drift):** clone A edits+pushes; clone B (stale) edits;
  `git pull --rebase` in B **succeeds** (exit 0, no `fatal: error while running
  fast-import`), replays B's commit, `git push` converges; final SoT reflects a
  reconciled state (no `does not contain`).
- **Scenario B (external REST PATCH drift):** clone C makes a local commit; a direct
  `curl -X PATCH /projects/demo/issues/<id>` moves the SoT; `git pull --rebase` in C
  **succeeds** and `git push` converges.
- **Negative guard (proves the test bites):** a git-level assertion that the pre-fix
  binary produces the `does not contain` / `fatal: error while running fast-import`
  string — captured as the RED baseline in the verifier's transcript so a future
  regression re-RED-s concretely.

**B. Rust unit test (`crates/reposix-remote/src/fast_import.rs` `#[cfg(test)]`).**
`emit_import_stream_with_parent_emits_from_line` — feeds a parent oid, asserts the stream
contains `from <oid>` in the commit block; `emit_import_stream_no_parent_is_parentless`
— asserts no `from` on the `None` path; `unchanged_tree_emits_no_commit` — asserts the
no-op guard emits a reset-only stream. Plus a `git fast-import` round-trip test (port of
`verify_fix.sh`) proving the with-parent stream fast-forwards a seeded ref and the
parentless-changed stream does not.

**Runs on the pinned git (2.25.1) which uses the `import` path** — this is a feature, not
a limitation: the gate exercises exactly the broken path. Add a comment in the gate
noting that on git ≥ 2.34 the fetch may route via `stateless-connect` (§5); the gate
forces the `import` path by asserting against the sim-init tree the same way the live
repro did (git 2.25 selects `import` unaided; on CI's modern git, pin
`-c protocol.version=0` or the helper's non-stateless path — executor to confirm which
knob forces `import` on the CI git and document it in the gate header).

---

## 4. Lane slicing (each lane < 100 tool calls; cargo = single machine-wide slot)

> Cargo discipline: ONE cargo invocation at a time, `-p reposix-remote`, jobs=2. Lanes
> that compile run SEQUENTIALLY through the coordinator's cargo slot.

**Lane 0 — catalog-first (no cargo).** Commit the GREEN-contract row
`agent-ux/rebase-recovery-reconciles` (NOT-VERIFIED) + the verifier script skeleton
(exits non-zero until the fix lands). FIRST commit of the phase. Names the exact row the
verifier grades. ~15 calls.

**Lane 1 — the fix (cargo).** `emit_import_stream` parent parameter + `from` emission +
no-op tree-equality guard; `handle_import_batch` rev-parse wiring. Unit tests in
`fast_import.rs`. `cargo test -p reposix-remote`. ~50 calls.

**Lane 2 — end-to-end gate (cargo for the helper build only).** Flesh out
`quality/gates/agent-ux/rebase-recovery-reconciles.sh` (port `repro2.sh`), wire it to the
catalog row, capture the transcript, flip the row to PASS with a `claim_vs_assertion_audit`
proving it bites (temporarily revert the fix, confirm RED, restore). ~40 calls.

**Lane 3 — docs + honesty reconciliation (no cargo).** Remove/replace the "Known
limitation (v0.13.x) — an EXTERNAL REST write breaks this recovery" block in
`docs/guides/troubleshooting.md:256-274`; correct `docs/concepts/dvcs-topology.md:93`,
`docs/index.md:154`, and the ADR-010 pull-rebase framing to reflect the fix; correct the
FALSE "git-push conflict recovers, external-REST-write breaks" distinction wherever it
appears. File the §6 lost-update surprise. `mkdocs-strict.sh` + `mermaid-renders.sh` if
docs/** touched. ~30 calls.

**Lane ordering:** 0 → (1 ‖ nothing) → 2 → 3. Lanes 1 and 2 share the cargo slot so they
serialize; Lane 3 is cargo-free and can overlap Lane 2's non-cargo portions.

---

## 5. Open question the executor MUST resolve (does NOT block the fix)

**Does the `stateless-connect` (protocol-v2) fetch path — used by git ≥ 2.34 — ALSO
break, or does it already fast-forward off the cache's chained history?** The cache's
`refs/heads/main` is a correct linear chain (verified), so a stateless-connect fetch
*likely* fast-forwards and the bug may be `import`-path-only (i.e. git-version-scoped).
This researcher could NOT verify: only git 2.25.1 is installed, and forcing
`protocol.version=2` on 2.25 errors before the fetch (`bad line length 2`). CI runs modern
git. **Executor action:** on CI's git, run both scenarios via the *stateless-connect* path
and record the result in the gate. If stateless-connect already works, the `import` fix
still ships (old-git support) and the gate must force the `import` path to keep guarding
it. If stateless-connect ALSO breaks, that is a SECOND fix site (cache-side) — file it,
do not silently expand this lane.

---

## 6. RAISE — separate severe bug discovered (out of THIS charter; FILE, do not fold in)

**Shared-cache `last_fetched_at` false-negative → SILENT LOST UPDATE (HIGH).**
Empirically confirmed (`repro4.sh`): clones A and B share one cache (keyed by
`(backend, project)` per `resolve_cache_path`). A pushes an edit → `write_loop.rs:309`
advances the shared cursor to `now`. B then pushes a *conflicting* stale-base edit; its
PRECHECK B runs `list_changed_since(now)` → empty → **no conflict detected** → B's PATCH
lands and silently clobbers A. Observed: issue-1 title A-CHANGED-TITLE (v2) →
B-CHANGED-TITLE (v3), no `fetch first`, no error. The ARCH-08 protection that
`push_conflict.rs::stale_base_push_emits_fetch_first…` proves in isolation FAILS
end-to-end under a shared cache. This is data loss, strictly worse than the RBF-LR-03
friction, and is the *real* manifestation of the "code framing" cursor concern. It needs
its own phase (cursor semantics / per-writer base tracking) — likely > 1h and coupled to
the v0.14.0 reconciliation redesign. **Action: file to SURPRISES-INTAKE.md at HIGH with
this repro; do not fix under Phase 105.**

---

## 7. Catalog rows the verifier will grade

| Row id | Dimension | Kind | Status at scaffold | Contract |
|---|---|---|---|---|
| `agent-ux/rebase-recovery-reconciles` | agent-ux | shell-subprocess | NOT-VERIFIED (catalog-first) | `bash quality/gates/agent-ux/rebase-recovery-reconciles.sh` exits 0: both drift scenarios (peer-push + external-REST) recover via `git pull --rebase && git push` with no `fatal: error while running fast-import`; transcript records argv/env-names/cwd/exit; a negative guard proves the pre-fix binary RED-s. |

Row body (drop into `quality/catalogs/agent-ux.json`, `kind: shell-subprocess`,
`transport_claim: false`, `coverage_kind: mechanical` — sim-backed, not a real backend)
is authored in Lane 0. `expected.asserts` (each must map to an `asserts_passed` string
per `agent-ux/test-name-vs-asserts`):
- `bash quality/gates/agent-ux/rebase-recovery-reconciles.sh exits 0 against the local cargo workspace + live sim`
- `Scenario A (peer git push drift): clone B `git pull --rebase` exits 0, replays B's commit, `git push` converges — NO `does not contain` / `fatal: error while running fast-import``
- `Scenario B (external REST PATCH drift): clone C `git pull --rebase` exits 0, `git push` converges`
- `NEGATIVE GUARD: the pre-fix emit_import_stream (parentless) reproduces `fatal: error while running fast-import` — captured in the transcript as the RED baseline`
- `emit_import_stream emits `from <parent>` when the tracking ref exists; parentless only on first fetch`

---

## 8. Acceptance criteria (phase close)

1. `emit_import_stream` chains onto the current tracking tip; unit tests green
   (`cargo test -p reposix-remote`).
2. `agent-ux/rebase-recovery-reconciles` graded PASS by the unbiased verifier subagent
   from committed artifacts, with a transcript and a bites-proof `claim_vs_assertion_audit`.
3. Both drift scenarios recover against the live sim (gate exit 0).
4. Docs no longer claim the recovery is broken; the FALSE push-vs-REST distinction is
   corrected everywhere it appears.
5. The lost-update RAISE is filed to SURPRISES-INTAKE.md at HIGH (not fixed here).
6. Full-workspace pre-push gate green; `git push origin main` lands BEFORE the verifier.

## 9. Provenance of claims

- Root cause + both scenarios: `repro2.sh` (empirical, live sim, git 2.25.1) — VERIFIED.
- Cache chain linearity: `git -C <cache>.git log --graph refs/heads/main` — VERIFIED.
- Fix mechanism (`from <parent>` fast-forwards): `verify_fix.sh` manual fast-import — VERIFIED.
- Lost-update RAISE: `repro4.sh` — VERIFIED.
- stateless-connect path behavior on modern git: UNVERIFIED (no modern git available) — §5 open question.
- Repro scripts committed under `.planning/phases/105-rbf-lr-03-rebase-recovery/repro/`.
