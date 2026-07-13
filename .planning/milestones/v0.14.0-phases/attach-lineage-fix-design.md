# Design: `reposix attach` DVCS-lineage fix (Pattern-C round-trip recovery)

READ-ONLY design pass. Implements NOTHING. All line cites verified against current code
(git 2.25.1 on this VM).

## 0. Files actually read (8)
- `crates/reposix-cli/src/attach.rs` (full)
- `crates/reposix-remote/src/main.rs` (full — `resolve_import_parent` L400-419, import batch L335-388)
- `crates/reposix-remote/src/fast_import.rs` (full — `emit_import_stream`, `ImportParent`, no-op guard, deleteall)
- `crates/reposix-cli/src/init.rs` (full — seed refspec + `refuse_existing_repo_root` exemplar)
- `docs/concepts/dvcs-topology.md` (full — Pattern C L133-156, promise L93)
- `.planning/research/v0.13.0-dvcs/architecture-sketch/index.md` (index; chapters are child docs)
- `crates/reposix-remote/tests/partial_failure_recovery.rs` (full — closest recovery test)
- `crates/reposix-remote/tests/common.rs` + `crates/reposix-cli/tests/attach.rs` (harness)
- Confluence: `adf.rs` (L105-119), `translate.rs` (L100-123), `types.rs` (L217-223)

## 1. Root-cause chain (verified)

1. `attach.rs` Step 4 (L253-260) does `git remote add reposix reposix::…?mirror=…`. It sets
   `remote.pushDefault` (L299-320) and `extensions.partialClone` (L262-266) but **never runs
   `git update-ref refs/reposix/origin/main …`**. Grep confirms: attach.rs contains zero
   `update-ref` / ref-seed calls. So an attach tree has NO `refs/reposix/origin/main`.
2. On a later `git fetch`/`git pull` from the reposix remote, git 2.25 selects the `import`
   verb → `handle_import_batch` (main.rs L335) → `resolve_import_parent()` (L400).
3. `resolve_import_parent` reads exactly `refs/reposix/origin/main` via `git rev-parse`. In an
   attach tree that ref is **absent → returns `None`** (L414: `let commit = rev_parse(REF)?;`).
4. `emit_import_stream(parent=None)` (fast_import.rs L156, L205) emits a **parentless** "Sync
   from REST snapshot" root commit (no `from`, no `deleteall`).
5. That parentless snapshot shares **no common ancestor** with the agent's `refs/heads/main`
   (which descends from the vanilla-mirror root). `git rebase` of the agent's edits onto it →
   two unrelated roots each adding `issues/<id>.md` → **cross-root add/add wall**. Pattern-C's
   documented `git pull --rebase && git push` recovery (dvcs-topology.md L88-91, L133-156)
   cannot complete.

The init flow does not hit this because init's first `git fetch` seeds
`refs/reposix/origin/main` via the `+refs/heads/*:refs/reposix/origin/*` refspec
(init.rs L319-325), so `resolve_import_parent` always finds a parent to chain onto.

`resolve_import_parent`'s doc comment (main.rs L390-399) frames ref-absent as "first fetch →
parentless seed" — TRUE for init (seeded on first fetch), but for **attach** the ref is never
seeded, so EVERY fetch is parentless. That comment hides the attach defect (fix-twice target).

## 2. ARCHITECTURAL-RISK VERDICT — **BOUNDED-ELEGANT-FIX**

Seeding `refs/reposix/origin/main` at attach + (optionally) a `resolve_import_parent` fallback
stays **inside** the ratified RBF-LR-03 architecture and forces **no** unsettled
synthetic-history-determinism / ref-identity decision. Evidence:

- **No new determinism requirement.** RBF-LR-03 deliberately does NOT require the synthesized
  snapshot commit to have a stable cross-machine SHA: the commit chains onto a *per-machine*
  tracking tip, its SHA varies by parent, and only the **tree** is compared (no-op guard, and
  `snapshot_tree_oid` — fast_import.rs L119-124, L156-164). The seed reuses this verbatim: it
  points the same per-machine tracking ref at a **real, already-present local commit** (the
  mirror merge-base). The subsequent fetch chains the snapshot onto it exactly as init does.
  Nothing about ref identity becomes global/shared/reproducible-across-clones.
- **No two-namespace-contract change.** The helper still writes only its private
  `refs/reposix-import/*`; git fetch remains the sole writer of `refs/reposix/origin/*`
  (main.rs L193-202; init.rs L294-325). The seed is a one-time `update-ref` by the CLI at
  attach time (git is not fetching then), not a helper-side write — it does not reintroduce the
  double-writer `cannot lock ref` race RBF-LR-03 layer-2 fixed.
- **The `deleteall` full-rebuild (CR-01, fast_import.rs L207-218) already absorbs the only
  divergence.** When the SoT snapshot tree differs from the seed commit's tree, the chained
  commit is a full rebuild → a normal fast-forward, and any content overlap with the agent's
  edit surfaces as an ordinary rebase **content** conflict (correct, resolvable) — never the
  cross-root add/add wall.

**The one place that could escalate — and does NOT force a pivot:** Part B (auto-healing
pre-fix trees via a runtime fallback in the git-trusted import path). Its safe form is bounded;
its unsafe form (HEAD fallback) is a *correctness* bug, not an architecture pivot (see §4).

**Bounded owner decision (NOT a STOP):** ship Part B as documentation-only ("re-run `reposix
attach`") vs. an automatic runtime mirror-ref fallback. This is a scope/risk call, does not
block Part A, and is not a synthetic-history pivot.

**What WOULD have been a STOP (and is not required):** if a correct seed demanded a
globally-reproducible synthetic root, or if Part B required auto-chaining onto HEAD. Neither is
needed — a real local mirror commit is a valid, safe anchor.

## 3. Fix Part A — seed `refs/reposix/origin/main` for NEW attach trees

### 3.1 The seed VALUE is load-bearing: mirror merge-base, NOT HEAD

Pattern C **commits before attaching** (dvcs-topology.md L140-142: `$EDITOR … && git commit`
precede `reposix attach`). So at attach time HEAD is already the edited tip `M'`, while the
mirror tracking ref `refs/remotes/<mirror>/main` is still the pre-edit base `M`.

Required invariant: the seeded commit must be an **ancestor of `refs/heads/main`** (so the
agent's edits replay ON TOP of the fetched snapshot). The SoT snapshot chains onto the seed, so
it is automatically a descendant of the seed.

- Seed = `refs/remotes/<mirror>/main` (= `M`) → snapshot `S` descends from `M`; agent's `M'`
  also descends from `M`; `git rebase S` replays `M→M'` onto `S`. **Clean reconciliation.**
- Seed = HEAD (= `M'`) → `S` descends from `M'`; git sees `M'` already contained in upstream →
  rebase fast-forwards `main` to `S`, whose tree lacks the un-pushed edit → **the agent's edit
  is silently reverted (data loss).** REJECTED.

So Part A must seed the **mirror tracking ref**, falling back to HEAD only when there is no
divergence (no mirror ref, or HEAD == that ref).

### 3.2 Seed-source resolution (reuse `git_config_get` / `run_git_in` already in attach.rs)

Resolve the first that verifies (all local refs → already in the object store, so `update-ref`
never hits the "refuses to point at unknown SHA" problem the bus_precheck tests work around):

1. `git rev-parse --verify --quiet refs/remotes/<mirror_name>/main` (the merge-base; canonical
   Pattern-C vanilla-clone case). `<mirror_name>` = `args.mirror_name` (default `origin`).
2. else `git rev-parse --verify --quiet HEAD` (no mirror ref, e.g. `--no-bus` / hand-edited
   tree with no separate base — best effort; only correct when HEAD == base).
3. else (unborn HEAD, empty tree) → **skip seeding**; first fetch parentless-seeds as today
   (correct for an empty tree — nothing to anchor, no cross-root risk).

### 3.3 Idempotency + insertion point

- **Only seed when `refs/reposix/origin/main` is ABSENT.** If it already exists (a prior real
  fetch, or a re-attached init tree), leave it — clobbering would rewind the tracking tip and
  cause re-fetch churn. Re-attach idempotency (Q1.3) preserved.
- New private helper in `attach.rs`, e.g.
  `fn seed_tracking_ref(work: &Path, mirror_name: &str) -> Result<()>` using the existing
  `git_config_get` (L361) and `run_git_in` (L378) — no new machinery.
- **Insertion point:** immediately after Step 5 `extensions.partialClone` (L262-266) and BEFORE
  the H1 `remote.pushDefault` block (L275). Rationale: the reposix remote now exists; the seed
  logically completes "establish the reposix remote's initial tracking state." Emit an
  informational line on success (e.g. `refs/reposix/origin/main seeded at <short> (mirror base)`).
- **Seed command:** `git -C <work> update-ref refs/reposix/origin/main <oid>`.

## 4. Fix Part B — heal EXISTING (pre-fix) attach trees

### 4.1 Is a `resolve_import_parent` HEAD/tracking-ref fallback deterministic/safe?

- **HEAD fallback: NOT safe.** At fetch time HEAD = the agent's edited/committed tip. Chaining
  the SoT snapshot onto HEAD makes the agent's edits ancestors of the upstream → rebase
  silently reverts un-pushed edits (§3.1 data-loss mode). Deterministic but wrong. **REJECT.**
- **Mirror tracking-ref fallback (`refs/remotes/<mirror>/main`): deterministic and safe-ish.**
  Reads a fixed ref name (no remote byte → no `Tainted` concern, same as today's static-ref
  read, main.rs L396-399). It IS the merge-base of the working branch in a Pattern-C tree, so
  it is an ancestor of HEAD → safe. Failure modes:
  - No mirror remote (single-SoT / `--no-bus`) → ref absent → still `None` → today's parentless
    behavior (no regression, no heal).
  - Agent fetched the mirror AFTER committing (mirror ref advanced past the true base) → may
    yield ordinary rebase **content** conflicts (acceptable) but never cross-root.
  - Prior-`init` tree with no `refs/remotes/*` → no heal via this ref (but such trees already
    have `refs/reposix/origin/main` from init's fetch, so they never needed Part B).

### 4.2 Recommendation

- **Primary heal (safe, zero new machinery): re-run `reposix attach <same-sot>`.** Same-SoT
  re-attach is already idempotent (attach.rs L131-141), and with Part A it now seeds
  `refs/reposix/origin/main` = mirror merge-base. Correct even after edits (seeds to the base,
  not HEAD). Document this as the pre-fix-tree recovery in troubleshooting + the reject hint.
- **Optional runtime auto-heal:** if wanted, add to `resolve_import_parent` — when
  `refs/reposix/origin/main` absent, try `refs/remotes/<mirror>/main` (mirror ref), **never
  HEAD**. This touches the git-trusted import path and adds a heuristic, so gate behind owner
  approval + the extra tests in §5, and surface it as the §2 bounded decision. `<mirror>` is
  not known to the helper today (it only knows the `?mirror=` URL, not the local remote name) —
  it would hardcode `origin` or scan `refs/remotes/*/main`, another reason to prefer re-attach.

## 5. Sim-first test plan (no real backend)

git 2.25 selects the `import` verb natively, so the whole round-trip is drivable on this VM.

### 5.1 Headline: attach seeds the merge-base (CLI unit/integration — `crates/reposix-cli/tests/attach.rs`)
New `#[ignore]`-tagged test `attach_seeds_tracking_ref_at_mirror_base` mirroring
`attach_against_vanilla_clone_sets_partial_clone` (L286) but with a REAL base + edit + mirror
ref (the current harness has an unborn HEAD + no `refs/remotes/origin/main`, so it must build
these):
1. `git init` a `/tmp` leaf; `write_record_md issues/1.md`; `git add && git commit` → base `M`;
   `git update-ref refs/remotes/origin/main M` (models the vanilla-clone mirror tracking ref);
   `git remote add origin <file-or-invalid-url>`.
2. Edit `issues/1.md`; `git commit` → `M'` (Pattern-C "commit before attach").
3. `run_attach(sim::demo, --remote-name reposix)`.
4. Assert `git rev-parse refs/reposix/origin/main` == `M` (the mirror base) and **!=** `M'`
   (HEAD) — pins the §3.1 seed-value decision and guards the silent-revert regression.

### 5.2 Round-trip recovery proof (remote crate — new `attach_pattern_c_roundtrip_recovers.rs`)
Model on `partial_failure_recovery.rs` (wiremock SoT + `run_helper_export` + `CacheDirGuard` +
`export_stdin`), adding the fetch/import leg. In a `/tmp` leaf:
1. Build base `M` (issues/1.md), seed `refs/reposix/origin/main = M`, config the reposix remote
   (URL + `remote.origin.fetch +refs/heads/*:refs/reposix/origin/*` as init.rs L319-325 does).
2. Agent edits issue 1 → commit `M'`.
3. Drive a real `git -C <leaf> fetch reposix` (import verb) against a wiremock SoT whose issue 1
   diverged out-of-band → assert `refs/reposix/origin/main` **fast-forwards** from `M`
   (`rev-parse main~1 == M`), NOT a parentless root.
4. `git rebase refs/reposix/origin/main` → assert exit 0, **no `CONFLICT (add/add)`**, agent's
   edit preserved.
5. `git push` (helper export) → assert `ok refs/heads/main`, SoT converged.
6. **Falsifier half** (the bug, guarded): repeat with the seed step OMITTED → assert the fetch
   yields a parentless root and the rebase reports unrelated-histories / add-add. This is the
   exact analog of `git_fast_import_roundtrip_with_parent_fast_forwards` (fast_import.rs L540)
   which already proves parented=fast-forward vs parentless=reject at the fast-import level.

### 5.3 Heal-existing regression
- If re-attach heal (recommended): extend §5.1 — start from a tree that has the reposix remote
  but NO `refs/reposix/origin/main` (pre-fix state), re-run `reposix attach <same-sot>`, assert
  the ref is now seeded to the mirror base; then run §5.2 steps 3-5 to prove recovery.
- If runtime mirror-ref fallback (optional): a `resolve_import_parent`-level unit test — ref
  absent + `refs/remotes/origin/main` present → returns the mirror oid (not None); and a
  negative test — with HEAD carrying un-pushed edits, the fallback does NOT return HEAD
  (guards the silent-revert data-loss mode).

### 5.4 Optional pure-unit falsifier (`fast_import.rs` tests, git-version-agnostic)
Extend the existing real-`git fast-import` suite: seed a "mirror root" `M`, agent commit `M'`
(edit), emit the SoT snapshot WITH `parent=M` → feed real `git fast-import` → `git rebase M'`
onto it succeeds; emit PARENTLESS → rebase reports add/add. Proves the anchor is what breaks the
wall without needing the helper process.

## 6. Item 4(b) — Confluence `adf_to_markdown` silent empty-body substitution (fail-closed)

`adf_to_markdown` (adf.rs L105-119) returns `Err` only when the root node type != `"doc"` (all
other malformed ADF degrades inline to `[unsupported ADF node type=…]` markers). Two callers
turn that `Err` into a **silent empty body**: `translate.rs` L111-116
(`Err(e) => { warn!; String::new() }`) and `types.rs` L217-223 (comments, same shape). The
hazard is a data-integrity one, not just a lossy read: an empty body flows into the working
tree; the agent commits and pushes; the export path PATCHes the SoT body to empty → **silent
destruction of real Confluence page content** — and the body is attacker-influenced (a crafted
non-`doc` root could weaponize this). The `T-16-C-05` graceful-degradation intent (don't wedge
the whole `list_records` on one bad page — a DoS mitigation) must be preserved, so the fix is
**scoped to the single record, fail-closed, and non-empty**: (1) first fall through to the raw
`storage` HTML value when present (translate.rs already has that branch at L118-123 — prefer it
over `String::new()` on ADF failure); (2) when no storage fallback exists, substitute a
conspicuous **non-empty sentinel** (e.g. `[reposix: unreadable ADF body — see recovery]`) that a
downstream push can detect and refuse rather than blanking the SoT; (3) surface a teaching
message modeled on `init.rs::refuse_existing_repo_root` — name what happened (ADF root type was
`"<type>"`, not `"doc"`) and the page id, suggest the alternative (open the page in the browser /
re-fetch storage format), and give a copy-paste recovery. Net: one bad page degrades to a loud,
push-safe sentinel for that record only, never a silent blank that destroys SoT content, and the
`list` still returns every other page (DoS mitigation intact).

## 7. NOTICED (lying docs / weak tests / dead code near this surface)

- **Doc over-claim (dvcs-topology.md L93):** "the recovery … reconciles regardless of *how* the
  SoT moved … Regression-guarded by `agent-ux/rebase-recovery-reconciles`" and Pattern C's "the
  recovery is the same `git pull --rebase && git push` you already know" (L88-91, L152) are
  asserted for the **init** topology; the **attach** (Pattern-C) topology is exactly where the
  unseeded-ref cross-root wall lives. The L93 prose is the promise this fix must MAKE true for
  attach — until then it is a lying doc for round-trippers. Fix-twice: update this prose + the
  Pattern C section when Part A lands.
- **Root-cause comment (main.rs L390-399):** `resolve_import_parent`'s "Ref absent (first fetch)
  → None → parentless root … so a fresh clone still bootstraps" frames absence as a benign
  first-fetch state. For attach trees the ref is NEVER seeded, so absence is permanent and the
  "parentless root" is the bug, not a bootstrap. Update this comment (fix-twice) so the next
  reader sees the attach case.
- **Weak/mis-scoped test (`partial_failure_recovery.rs`):** its module doc says "the agent's
  working tree is the post-`git pull --rebase` state" and it FABRICATES that state by hand
  (`render_issue_blob` with forged versions, L385-391) — it never drives an actual
  `git pull --rebase`, so it does **not** cover the cross-root add/add wall at all. There is
  currently NO regression test that a Pattern-C rebase fast-forwards. §5.2 fills this gap.
- **adf silent-empty-body** (translate.rs L114, types.rs L222): §6 — data-loss-on-push hazard;
  the `# Errors` doc on `adf_to_markdown` (L100-105) documents root-not-doc as the only Err, but
  callers convert it to silent empty content.
- **Adjacent known-gap (init.rs L310-318):** the `git checkout origin/main` non-resolution
  (`refs/reposix/origin/*` vs `refs/remotes/origin/*`) is filed for v0.14.0 — related tracking-
  ref-namespace surface; the attach seed writes `refs/reposix/origin/main`, consistent with init.
  Out of scope for this fix, noted for adjacency.

## 8. Addendum — B1/attach blast-radius acceptance-criteria gap (2026-07-13, folded from B3 noticing)

B3's `attach-sync-real-backend` real-backend PASS is **coverage-hollow**: `run_attach_real` /
`assert_attach_configured` / `assert_sync_reconcile_ok`
(`crates/reposix-cli/tests/agent_flow_real.rs:237-342`) assert only local git-config after
`reposix attach` + a cache-only `reposix sync --reconcile` exit code — **neither smoke ever
drives a `git checkout`/`fetch`/`pull` against the configured `reposix` remote**, so this gate
cannot confirm or deny whether §3/§5's fix actually closes the B1 round-trip gap. Full finding:
`.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` (2026-07-13 15:52 entry,
"attach-sync-real-backend real-backend PASS is inconclusive by construction"). **Binding on
this design's acceptance criteria:** item 4a's fix + item 5 (litmus re-green) MUST add a real
`git checkout`/`fetch`-after-`attach` round-trip assertion (§5.2 above already specifies this
at the wiremock-sim level; the real-backend arm — extending
`attach_real_confluence`/`sync_real_confluence` or a new sibling test — must be added too) as
acceptance evidence. Do not declare the attach gap closed on the existing B3 smokes alone. Also
noticed: `attach_real_confluence`/`sync_real_confluence` test names overclaim relative to what
they assert (config-only, not round-trip) — rename or add an explicit `test-name-honesty`
marker when touched.
