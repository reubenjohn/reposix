# CONSULT-DECISIONS — decision ledger (bounded to LIVE decisions)

Escalation-valve + owner decision ledger. **Holds only OPEN / live / still-relevant
decisions.** A decision that is CLOSED, implemented, or superseded is **DELETED** — `git
log` / `git show` is the archive (reversible). No unbounded growth.
`[SELF]` = decided under the escalation-valve bar (below E1–E4), recorded not escalated.
`[FABLE]`/`[CONSULT]` = fable-consult invoked. `[OWNER]` = owner decision.

Format: `## <date> [SELF|FABLE|OWNER] <one-line>` then rationale + evidence.

---

## 2026-07-07 [SELF] Documented front door `git checkout origin/main` break → HOTFIX-HONEST FALLBACK (truthful banner + git-128 refspec alignment), pure-git ergonomic filed for v0.14.0

- **Lane:** v0.13.1 CHECKOUT-BREAK. **Decision bias:** HOTFIX-conservative.
- **VERDICT: hotfix-honest-fallback** (NOT the root-cause pure-git fix). One-sentence why:
  the additive `refs/remotes/origin/*` population that would make `git checkout origin/main`
  work verbatim is verifiable only on this box's git 2.25 (import path), not the supported
  git >= 2.34 `stateless-connect` fetch path, AND it lands in detached HEAD — an unverifiable,
  ergonomics-changing ref-topology move that a hotfix must not ship.
- **Reproduced (leaf-isolated `/tmp`, freshly built binaries, live `--ephemeral` sim seeded
  from `fixtures/seed.json`, git 2.25.1):** after `reposix init sim::demo /tmp/<uniq>/repo`,
  only `refs/reposix/origin/main` exists (no `refs/remotes/origin/main`), so the documented
  `git checkout origin/main` fails `error: pathspec 'origin/main' did not match any file(s)`;
  the `init` banner ALSO printed that broken command. A SECOND `git fetch` exited git-128
  `fatal: could not read ref refs/reposix/main` (helper advertised
  `refspec refs/heads/*:refs/reposix/*` while fast-import writes `refs/reposix/origin/main`).
- **Canonical working checkout command (handed to the doc-truth lane, verbatim):**
  `git checkout -B main refs/reposix/origin/main`
- **Shipped in this hotfix (all 2.25-verified, none change where refs land):**
  1. `reposix-cli/src/init.rs` success banner → prints the verified-working
     `git checkout -B main refs/reposix/origin/main` (was the broken `git checkout origin/main`).
  2. `reposix-remote/src/main.rs` capabilities → advertised refspec aligned to
     `refs/heads/*:refs/reposix/origin/*` (matches the fast-import write target), closing the
     spurious git-128 on re-fetch. Verified: second `git fetch` now exit 0; push round-trip
     still green (`refs/reposix/origin/*` unchanged; 4 protocol test assertions updated).
  3. `reposix-cli/src/init.rs` `translate_spec_to_url` now honours `REPOSIX_SIM_ORIGIN`
     (attach/sync already did; init was the odd one out) — enables the leaf-isolated
     end-to-end regression test on an isolated port.
- **Test added:** `crates/reposix-cli/tests/agent_flow.rs::checkout_break_front_door_works_end_to_end`
  (leaf-isolated: own seeded sim + `REPOSIX_CACHE_DIR` + tempdir; asserts banner truthfulness,
  git-128-free re-fetch, checkout resolves, `issues/1.md` materialises `id: 1`). Also repaired
  the pre-existing B4 regression in `dark_factory_sim_happy_path` (init now hard-errors on an
  unreachable backend; the test targeted the dead default port) by pointing it at its live sim.
- **Smoke-tested end-to-end (`/tmp`, aligned build):** init → `git checkout -B main
  refs/reposix/origin/main` → `cat issues/1.md` shows real frontmatter (`id: 1`, title,
  body) → local edit → `git push origin main` → `223b444..f756b04 main -> main` (SoT write
  accepted). Additive `refs/remotes/origin/*` probe: both refs populated, `git checkout
  origin/main` resolves but detached HEAD.
- **Deferred:** pure-git `git checkout origin/main` ergonomic → SURPRISES-INTAKE 2026-07-07
  (v0.14.0, MEDIUM). Doc edit spec (canonical checkout command) handed to the doc-truth lane.

## 2026-07-07 [SELF] Post-conflict recovery crash after an EXTERNAL REST write is the KNOWN RBF-LR-03 deep-reconciliation limitation → docs-honest, NOT a code hotfix

- **Lane:** v0.13.1 onboarding-hotfix, B5 TRIAGE. **Decision bias:** HOTFIX-conservative.
- **Reproduced (leaf-isolated in `/tmp`, sim `--ephemeral` seeded, `git-remote-reposix`
  on PATH, git 2.25.1 on this box):**
  1. `reposix init sim::demo /tmp/<uniq>/repo` → `git checkout origin/main` (tracking tip
     `bd848c1 "Sync from REST snapshot"`).
  2. Local edit + commit issue `1.md` → local tip `ff2877f` (child of `bd848c1`).
  3. **External REST write** (NOT a git push): `PATCH /projects/demo/issues/1` bumps the
     SoT to version 2.
  4. FULL documented recovery from `docs/guides/troubleshooting.md` §"Bus-remote `fetch
     first` rejection" + `docs/concepts/dvcs-topology.md`:
     `reposix sync --reconcile` → `git pull --rebase` → `git push`.
- **Exact failing command + stderr (load-bearing):** `git pull --rebase` (STEP B, AFTER
  a clean `reposix sync --reconcile` exit 0) aborts:
  ```
  warning: Not updating refs/reposix/origin/main (new tip 764e1a70… does not contain bd848c1…)
  fatal: error while running fast-import
  ```
- **Does the FULL documented sequence (with `sync --reconcile` first) recover? NO — it
  CRASHES.** `sync --reconcile` returns exit 0 but is itself the trigger: it mints a fresh
  synthesis commit (`5f15b93…`) that is NOT a descendant of the tracking tip `bd848c1`.
  When `git pull`'s fetch re-invokes the helper's fast-import (`import`/stateless-connect),
  git refuses to advance `refs/reposix/origin/main` to a non-descendant tip → the fatal.
  The BARE `git pull --rebase` (no reconcile) crashes IDENTICALLY (`new tip 1b3d82e… does
  not contain bd848c1…`, same `fatal`). So the original break did NOT merely skip
  `sync --reconcile`; the reconcile step does not help and cannot help — it manufactures
  the exact non-descendant condition that fast-import rejects.
- **Data-safety note:** after the pull crash, the local tree still has the local edit but
  NEVER incorporated the external REST edit; the follow-on `git push` returned exit 0
  (`[new branch] main -> main`), i.e. it would push the stale-relative-to-SoT tree —
  silently overwriting the external writer's edit. This is the very overwrite
  `sync --reconcile` was documented to prevent.
- **CALL: KNOWN LIMITATION (RBF-LR-03 class), docs-honest — NOT a cheap regression.** The
  fix is to make the cache's "Sync from REST snapshot" commit a *descendant* of the prior
  synthesis tip (lineage + dedup + conflict semantics) — exactly the v0.14.0
  reconciliation-redesign already ratified as the RBF-LR-03 pivot (owner entry
  2026-07-06 above; SESSION-HANDOVER §5). No `<1h`/no-new-dependency fix exists; a point
  patch here would re-entrench the placeholder-synthesis design the pivot is chartered to
  replace. Per HOTFIX-conservative bias: do not touch reconciliation in v0.13.1.
- **Actions taken:** (a) DOC-HONESTY EDIT SPEC produced for `troubleshooting.md` +
  `dvcs-topology.md` (below) — handed to the doc-truth lane; this lane edited NO docs.
  (b) Filed the deep fix as `S-260707-rbf-lr03-external-write-crash` in
  `v0.13.0-phases/SURPRISES-INTAKE.md`, tagged for the v0.14.0 RBF-LR-03 pivot.
- **Verify against reality:** two leaf-isolated `/tmp` repro scripts run this session
  (full recovery + bare-pull/fresh-reinit characterization); the FRESH `reposix init`
  into a new dir correctly shows the external edit (`grep EXTERNAL REST EDIT` ✓) — this is
  the honest current workaround (re-clone; re-apply your unpushed local commit by hand).
- **Reversibility:** planning-file-only; no code/docs touched by this lane.

### DOC-HONESTY EDIT SPEC (for the doc-truth lane — apply verbatim, this lane did NOT edit docs)

**File 1: `docs/guides/troubleshooting.md`, §"Bus-remote `fetch first` rejection"
(~lines 246-256).** The "Fix" block currently advertises the sequence as a reliable
recovery. It is NOT reliable when the SoT moved via an *external REST write* (vs a
git-side push): `git pull --rebase` aborts with `fatal: error while running fast-import`.

- OLD (fix block + "Why two commands" para, ~246-254):
  ```
  Fix:

  ```bash
  reposix sync --reconcile          # full list_records walk against the SoT
  git pull --rebase                 # replay your commits on top of the fresh state
  git push                          # bus remote retries; precheck B passes
  ```

  Why two commands: `git pull` from the GH mirror gives you the mirror's lagging view. `reposix sync --reconcile` walks the SoT directly via REST and updates your cache to match — the ground-truth refresh needed before rebasing. Once the cache is fresh, `git pull --rebase` becomes a local-only rebase and `git push` succeeds.
  ```
- NEW:
  ```
  Fix (works when the conflict came from another git-side *push*):

  ```bash
  reposix sync --reconcile          # full list_records walk against the SoT
  git pull --rebase                 # replay your commits on top of the fresh state
  git push                          # bus remote retries; precheck B passes
  ```

  Why two commands: `git pull` from the GH mirror gives you the mirror's lagging view. `reposix sync --reconcile` walks the SoT directly via REST and updates your cache to match — the ground-truth refresh needed before rebasing. Once the cache is fresh, `git pull --rebase` becomes a local-only rebase and `git push` succeeds.

  > **Known limitation (v0.13.x) — an EXTERNAL REST write (not a git push) breaks this recovery.**
  > If the SoT moved because someone edited the record *directly* (web UI / REST PATCH)
  > rather than via a reposix `git push`, the sequence above does **not** recover — and
  > `reposix sync --reconcile` does not help. `git pull --rebase` aborts with:
  > ```
  > warning: Not updating refs/reposix/origin/main (new tip … does not contain …)
  > fatal: error while running fast-import
  > ```
  > Root cause: the cache rebuilds a "Sync from REST snapshot" commit that is not a
  > descendant of your current tracking tip, so git's fast-import refuses to advance the
  > ref. This is the RBF-LR-03 deep-reconciliation limitation, scheduled for the v0.14.0
  > reconciliation redesign.
  >
  > **Current workaround:** re-clone into a fresh tree —
  > ```bash
  > reposix init <backend>::<project> /tmp/fresh-tree
  > cd /tmp/fresh-tree && git checkout origin/main
  > ```
  > The fresh tree reflects the external edit correctly. **You lose any local commits you
  > had not yet pushed** — re-apply them by hand (e.g. copy your edited `.md`, re-commit).
  ```

**File 2: `docs/concepts/dvcs-topology.md` (~lines 89-93).** The reject-hint example is
followed by "The recovery is the same `git pull --rebase` you already know." — which
overstates reliability for the external-REST-write case.

- OLD (~line 93):
  ```
  The reject message reads its own cache's ref state and translates the staleness window into a human sentence. The recovery is the same `git pull --rebase` you already know.
  ```
- NEW:
  ```
  The reject message reads its own cache's ref state and translates the staleness window into a human sentence. The recovery is the same `git pull --rebase` you already know — **with one caveat.** When the SoT moved via a *git-side push*, `reposix sync --reconcile && git pull --rebase` recovers cleanly. When it moved via an **external REST write** (web UI / direct PATCH), the cache rebuilds a non-descendant "Sync from REST snapshot" commit and `git pull --rebase` aborts with `fatal: error while running fast-import` — the RBF-LR-03 deep-reconciliation limitation (§Cache-coherence, "a same-second write racing the cursor makes it silently wrong"), fixed in the v0.14.0 reconciliation redesign. Workaround until then: re-clone with a fresh `reposix init` and re-apply unpushed local commits. See [troubleshooting — Bus-remote `fetch first` rejection](../guides/troubleshooting.md#bus-remote-fetch-first-rejection).
  ```

## 2026-07-06 [OWNER] RBF-LR-03 pivot — model create/multi-step interactions as a commit sequence with slug→ID translation

- **Context:** The v0.13.0 tag was gated on RBF-LR-03 (ADR-010 §3): a create-partial-fail
  against an id-reassigning real backend (GitHub/JIRA/Confluence) can duplicate a record
  on retry, because the placeholder-id → backend-id mapping has no home and id-matching
  re-plans the already-done create. Offered the owner document-and-defer vs. three point
  fixes (content-match / persist-map / idempotency-key). The owner rejected the framing as
  a point fix and directed a design **pivot** instead.
- **Status of this vision — DIRECTIONAL INSPIRATION, NOT A SPEC.** The slug/symlink/
  commit-sequence model below is the owner's *inspiration for the direction*, captured
  faithfully. The v0.14.0 coordinator-of-coordinators exploration **OWNS the outcome** and
  may converge on a *different* mechanism (idempotency-key, content-match, the
  commit-sequence model, or a synthesis) after prototyping on real backends. The
  exploration is NOT bound to implement this sketch literally — it is bound to solve the
  root problem (placeholder-id has no home → partial-fail duplicates) cleanly.
- **Decision (owner vision, captured faithfully):** Backends OWN their UIDs; the current
  agent-picks-a-placeholder-id mechanism is bad design. Replace it with a **user-authored
  slug** model:
  1. The user creates their own **slug** and pushes.
  2. On push the virtual remote synthesizes a **commit sequence**: (a) a commit that
     translates slug → backend-assigned ID, (b) the correctly ID-named record file, (c) the
     slug becomes a **symlink** under `slugs/` pointing at the ID-named file, (d) an
     invariant that no other slug in `slugs/` points to that ID, (e) a **merge commit** so
     the agent only ever has to **fast-forward**.
  3. **Generalization:** ANY multi-step client↔server interaction is modeled as a
     **series of commits**, so a partial failure leaves a well-defined intermediate state
     the cache + backend can **reconcile by replaying/continuing the sequence** — no
     torn-state ambiguity, no lost mapping.
  4. **Open question (unresolved):** on full success, optionally **squash** the sequence
     for efficiency — owner is unsure whether squashing reintroduces reconciliation
     complications. To be settled by the exploration, not assumed.
- **Directive:** This is "complex and crucial — exactly the kind of thing I meant by
  pivots." Run a **coordinator-of-coordinators** effort that EXPLORES candidate mechanisms,
  PROTOTYPES the top few **against a real backend**, STRESS-TESTS surviving approaches on
  **all available backends** via prototypes before convergence, then implements the
  strategic, clean, debt-free version — accepting potentially large refactors + docs +
  quality-infra/CI changes. Do NOT converge on paper; converge on prototypes that survived
  a real backend. **~Milestone-sized; gate the spend before the prototype phase.**
- **Rationale:** Point fixes each patch the symptom while leaving the placeholder-id
  design — the actual root cause — in place. The commit-sequence model makes partial-fail
  reconciliation a property of the data model rather than a special case.
- **Reversibility:** Fully reversible — this ledger entry + exploration artifacts only; NO
  code or ADR-010 change yet (ADR-010 §3 is revised only AFTER the exploration converges).
  Tag-timing settled separately below (T1).
- **Commit:** 131315c (+ amendment).

## 2026-07-06 [SELF] Tag-timing: T1 — ship v0.13.0 now, RBF-LR-03 pivot becomes v0.14.0

- **Context:** With RBF-LR-03 escalated to a milestone-sized pivot (explore→prototype→
  stress-test→converge→clean-impl, not a point fix), "solve before tag" would delay the
  v0.13.0 tag by a full milestone. The owner delegated the call explicitly ("will
  ultimately leave this to you… least complex/confusing way") and named the constraint:
  do NOT suppress gates.
- **Decision:** **T1.** Tag v0.13.0 now. RBF-LR-03 ships as an **honestly-WAIVED,
  documented known-limitation** (narrow: real backend + mid-batch-create network drop →
  one hand-deletable duplicate). The reconciliation pivot becomes the **v0.14.0 headline
  milestone**. T1 requires **NO gate suppression** — the waiver is honest and owner-signed;
  a completed, honest milestone ships rather than being held hostage to a large redesign.
- **Rationale:** Owner design taste — ship honest milestones + document known limitations
  out loud, don't hold a green milestone for a big redesign. The footgun is narrow and
  recoverable; the pivot is too valuable to rush under a tag deadline.
- **Reversibility:** Fully reversible — sequencing only; the tag can be cut at any HEAD.
- **Commit:** (this entry; handover encodes the sequencing).

## 2026-07-06 [SELF] Real-backend 9th probe — VERIFIED (owner's "it's all set" is correct)

- **Context:** Owner asked why the 9th probe reads NOT-VERIFIED when an earlier agent
  concluded it "is all set" — env/perms/worktree suspected. All three ruled out (.env
  present + readable, correct worktree).
- **Crux:** The real-Confluence probe **genuinely ran GREEN.** Committed catalog row
  `agent-ux/milestone-close-vision-litmus-real-backend` (quality/catalogs/agent-ux.json)
  carries `last_real_grade: "PASS"` / `last_real_verified: 2026-07-05T02:23:17Z`, and a
  FRESH ephemeral PASS exists at `quality/reports/…/…-2026-07-06T06-28-00Z.json` (exit 0,
  real Confluence page 2818063 round-trip, dual-table audit, mirror refs advanced). The
  mechanical `status: NOT-VERIFIED` is **honest-by-design**: this P0 row has NO waiver and
  fails-closed to NOT-VERIFIED whenever re-graded in a shell without creds (env-gate, exit
  75), while **preserving** `last_real_grade`. NOT-VERIFIED ≠ never-passed; it means "the
  last mechanical grading context had no creds." Nothing is misconfigured.
- **Decision:** Treat the real-backend probe as SATISFIED for the tag via committed
  `last_real_grade: PASS` + the 07-06 green transcript. No NEW real-backend call is
  required to tag; the owner need not re-run it. (`quality/reports/**` is gitignored /
  ephemeral by design — the durable record is the catalog's `last_real_grade`.)
- **Reversibility:** N/A (finding, not a change).
- **Commit:** (this entry).

## 2026-07-07 [SELF] D1 — v0.13.1 onboarding hotfix, sequenced BEFORE the v0.14.0 pivot

- **Context:** Zero-shot human-simulation testing (3 independent fresh-agent
  reproductions) found binary-install onboarding is 100% broken: `reposix-sim` (the
  documented DEFAULT backend, OP-1) ships in no prebuilt distribution, and `reposix init`
  silently exits 0 when the backend is unreachable, masking the failure.
- **Decision:** Ship a scoped v0.13.1 hotfix BEFORE the v0.14.0 reconciliation pivot.
  Acceptance (end-state; mechanism converges in discuss/plan): (i) the documented
  getting-started flow completes end-to-end on the shipped binary, not a source build;
  (ii) `reposix init` exits non-zero on an unreachable backend; (iii) the release-path
  sim→cargo fallback that hides the gap is removed; (iv) verified by a fresh zero-shot
  human-simulation agent (D3). Bias toward shipping `reposix-sim` in the release matrix
  (OP-1 makes it canonical), but honest de-advertisement is an acceptable convergence if
  shipping sim proves disproportionate.
- **Rationale:** An adoption-blocker on `releases/latest` cannot wait behind a
  milestone-sized pivot; ship honest milestones, don't let an urgent regression sit
  behind a large redesign (owner design taste).
- **Reversibility:** Fully reversible — sequencing + scope only.
- **Commit:** (this entry; SESSION-HANDOVER.md encodes the runbook).

## 2026-07-07 [SELF] D2 — v0.14.0 hardening: reject-t@t-identity hook + worktree isolation are P0

- **Context:** A dispatched sim/seed leaf corrupted the local shared repo TWICE this
  session (flipped `core.bare=true`; set `user.email`/`user.name` to `t<t@t>`). Root
  cause: agent worktrees are not isolated (shared `.git/config` + object store) plus cwd
  resets between Bash calls. The doc-only HARD-STOP rule (ORCHESTRATION.md § "Leaf
  isolation") did not prevent the recurrence.
- **Decision:** Treat a commit-time guard + real worktree isolation as P0 for v0.14.0
  hardening scoping. Sketch: a pre-commit/pre-push hook that hard-rejects any commit
  authored by `t<t@t>` (or any non-allowlisted identity), plus per-leaf isolated `/tmp`
  clones and unique `REPOSIX_CACHE_DIR` enforcement per leaf.
- **Rationale:** A doc rule alone did not stop a second recurrence in the same session;
  the guard needs to be code-enforced, not convention-enforced.
- **Reversibility:** Fully reversible — new hook + isolation convention, additive.
- **Commit:** (this entry; anchor intake `S-260707-pr-08`, HIGH).

## 2026-07-07 [SELF] D3 — zero-shot human-simulation testing becomes a standing milestone-close gate

- **Context:** This session's zero-shot human-simulation testing (fresh, context-free
  agents following only the published docs) is what caught the sim-onboarding break
  (D1) — a gap no in-context agent or existing catalog gate had surfaced.
- **Decision:** Institutionalize as a STANDING milestone-close gate (new agent-ux catalog
  row), not a one-off session activity. Every milestone-close dispatches N fresh,
  context-free agents that install the shipped artifact the way the docs say and attempt
  the documented workflows (read path: init/attach → clone → grep/cat; write path:
  edit → commit → push; recovery: conflict-rebase, blob-limit sparse-checkout). Any
  doc-lie or broken path grades RED.
- **Rationale:** In-context agents share the session's accumulated assumptions and won't
  independently rediscover a docs/reality gap the way a fresh agent following only the
  docs will; this class of gap is exactly what a milestone-close should catch before
  shipping.
- **Reversibility:** Fully reversible — new catalog row + gate, additive.
- **Commit:** (this entry; catalog row to be filed as part of v0.13.1 or v0.14.0
  scoping).

## 2026-07-12 [SELF] D4 — fleet-safety verdict JSONs: UNTRACK over byte-stabilize

[SELF] (2026-07-12): fleet-safety verification JSONs re-dirty CI checkout → chose UNTRACK (git rm --cached) over byte-stabilize. (a) Investigation confirmed NOTHING reads the committed JSON content as a baseline — run.py write-back-merges the copy it just regenerated this run (never diffs committed bytes); catalog expected.artifact fields are write-targets not read-back baselines; no verifier/verdict.py/test_audit_field.py compares committed bytes. Pure per-run outputs. (b) .gitignore:72 verifications/*/*.json ALREADY ignores them; force-added despite the pattern; p93/perf baselines carry explicit ! re-includes, these do not. (c) Exact P102 precedent fbe02c8 git rm --cached on force-added per-run transcripts; shell-coverage.json/*.cobertura.xml already gitignored+untracked. (d) Byte-stable (309f0b6) is fundamentally fragile: the JSON's only non-static fields (asserts_passed/failed, exit_code) derive from live guard-scenario ASSERT outcomes that can flip PASS↔FAIL across git-version/env between CI and local — 309f0b6 removed the ts field but cannot make asserts environment-invariant. Untracking removes the failure class entirely.

## 2026-07-12 [SELF] D5 — should a RED release-plz block phase-close via ci-green-on-main P0 bar?

[SELF] (2026-07-12): should a RED release-plz block phase-close via the ci-green-on-main P0 bar? YES in principle — this BLOCKER proves an unwatched red release workflow rots silently (Global CLAUDE.md: health is a maintained asset; never let a metric you don't watch decay). But NOT implemented inline: (a) ci-green-on-main.sh hardcodes WORKFLOW=ci.yml; a clean fold = parameterize into a required-workflow list or a sibling code/release-green-on-main row (catalog-first ordering + a verifier grade) — non-trivial. (b) Open semantic question needing verification before P0-wiring: does release-plz run on EVERY push to main, and does a 'no release needed' outcome conclude success or skipped? A false-RED would block UNRELATED phases → warrants owner gate. FILED to SURPRISES-INTAKE with sketch.

## 2026-07-12 [OWNER] dependabot #64-66 closed-as-redundant

[OWNER] (2026-07-12): dependabot PRs #64 (tower-http 0.6.8→0.7.0), #65 (gix 0.83.0→0.85.0), #66 (rusqlite 0.39.0→0.40.1) closed as redundant — `cargo audit` reports 0 live advisories, none touch the flagged crates (memmap2/quinn-proto), bases superseded on current main.
