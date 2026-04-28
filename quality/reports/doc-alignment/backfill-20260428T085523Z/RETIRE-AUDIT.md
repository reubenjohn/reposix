# RETIRE_PROPOSED Audit — v0.12.1 P67

**Reviewer:** unbiased subagent (Path A; Opus 4.7 / 1M ctx)
**Reviewed:** 2026-04-28
**Scope:** 16 rows in `RETIRE_PROPOSED` state, excluding 24 glossary
definitional rows (bulk-confirm pending, not in scope) and 1
already-corrected ADR-003 row (`docs/decisions/003-nested-mount-layout/...`,
flipped to `MISSING_TEST` with `IMPL_GAP` rationale earlier in this session).
Catalog source-of-truth: `quality/catalogs/doc-alignment.json`.

The owner's framing: the original P65 backfill extractor over-retired
rows by **conflating "transport changed" with "feature dropped."** v0.9.0
retired the **FUSE transport** (mount syscalls, `fusermount3`, the
virtual filesystem) — but most of the user-facing **shape promises**
(create/update/delete, page-tree navigation, label views, multi-space
listing, time-travel-style diffs) survive the pivot, just routed
through `git push`/`git fetch` over a partial-clone working tree. The
audit below applies the heuristic table from the prompt:
**FUSE transport detail → CONFIRM_RETIRE; user-facing shape promise →
FLIP_TO_MISSING_TEST_IMPL_GAP.**

Bias preserved per owner directive: "preserve useful features and use
each as an opportunity to improve the codebase or restructure docs."

## Summary

| Recommendation | Count |
|---|---:|
| CONFIRM_RETIRE | 6 |
| FLIP_TO_MISSING_TEST_IMPL_GAP | 8 |
| FLIP_TO_MISSING_TEST_DOC_DRIFT | 2 |
| FLIP_TO_MISSING_TEST_TEST_ONLY | 0 |
| NEEDS_OWNER_DECISION | 0 |
| **Total** | **16** |

The 8 IMPL_GAP flips are clustered by remediation work:

- **Confluence backend parity / page-tree shape (P72-ish):** 2 rows
  (`nav-01` labels, `nav-02` spaces) — surface promises that should
  carry through partial-clone but have no current test binding.
- **Index synthesis / `_INDEX.md` (own cluster):** 2 rows (`index-01`,
  `index-02`) — sitemap synthesis promise that survives the pivot in
  spirit but no current implementation reaches it.
- **Confluence write-path round-trip via `git push` (P72-ish):** 3
  rows (`write-01`, `write-02`, `write-03`) — `create_record` /
  `update_record` / `delete_or_close` ARE implemented and ARE wired
  into the helper push path; missing piece is the dark-factory-style
  test that asserts the round-trip from a working-tree edit.
- **Mount-as-time-machine via sync-tags (own cluster):** 1 row
  (`cache-02`) — superseded *in spirit* by ADR-007 (time-travel via
  git tags) but the doc still claims `git diff HEAD~1` semantics and
  no test binds the time-travel surface.

The 2 DOC_DRIFT flips are JIRA Phase 28 read-only and the playwright
deferral note — both already-shipped follow-on phases mean the prose
is stale, not the impl. Resolution is a tiny doc edit, not a
re-implementation.

The 6 CONFIRM_RETIRE rows are unambiguous FUSE-transport details
(macFUSE CI matrix, swarm-direct mode is already shipped, two
redirect-only stub pages, etc.) — see per-row notes below.

## Per-row breakdown

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-01`
- **Claim:** "Agent can create a new Confluence page by writing a new
  .md file in the FUSE mount (`ConfluenceBackend::create_issue`)"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:19`
- **Existing extractor rationale:** none recorded (history_count: 0;
  the row was bulk-flipped to RETIRE_PROPOSED on FUSE keyword match).
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** The FUSE-mount transport is gone, but the
  user-facing capability survives. `ConfluenceBackend::create_record`
  IS implemented (`crates/reposix-confluence/src/lib.rs:280`) and
  IS wired into the helper push path
  (`crates/reposix-remote/src/main.rs:513`) so an agent that creates
  a `.md` file in their partial-clone working tree and runs
  `git push` triggers `create_record` against Confluence. The shape
  promise — "write a new `.md` file → new Confluence page" — survives
  the pivot. Renamed `create_issue` → `create_record` in the trait
  is a refactor, not a retirement. No current test asserts the
  end-to-end round-trip from a working-tree edit; the Confluence
  swarm hits `create_record` directly, not through the helper.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-01 \
    --rationale "IMPL_GAP: ConfluenceBackend::create_record exists (lib.rs:280) and is wired into helper push path (reposix-remote/main.rs:513). The FUSE-mount transport was retired in v0.9.0 (architecture-pivot-summary.md) but the user-facing capability (write .md → Confluence page) persists via git push. No dark-factory-style test asserts the working-tree-edit → git-push → REST-create round-trip end-to-end. Resolution: bind a test in agent_flow_real that creates a page via working-tree write + git push and asserts the page exists on the backend."
  ```
- **v0.12.1 cluster:** Confluence backend parity (push-path round-trip tests)

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-02`
- **Claim:** "Agent can update a Confluence page by editing its .md
  file in the FUSE mount (`ConfluenceBackend::update_issue`)"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:20`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** Same shape as write-01.
  `ConfluenceBackend::update_record`
  (`crates/reposix-confluence/src/lib.rs:335`) is wired into the
  helper push path (`reposix-remote/main.rs:540`). Agent edits a
  `.md` file in working tree → `git push` → REST PUT to Confluence
  with `If-Match`/version. The user-facing promise survives. No
  current test asserts the round-trip from working-tree edit
  through `git push` to backend mutation.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-02 \
    --rationale "IMPL_GAP: ConfluenceBackend::update_record exists (lib.rs:335) with version-conflict handling and is wired into helper push path (reposix-remote/main.rs:540). Transport pivoted from FUSE write-callback to git push/fast-export in v0.9.0; user-facing capability (edit .md → page update) persists. No agent_flow_real test asserts working-tree-edit → git-push → REST-update round-trip with version reconciliation."
  ```
- **v0.12.1 cluster:** Confluence backend parity (push-path round-trip tests)

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-03`
- **Claim:** "Agent can delete/close a Confluence page by unlinking
  its .md file (`ConfluenceBackend::delete_or_close`)"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:21`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** `ConfluenceBackend::delete_or_close`
  (`crates/reposix-confluence/src/lib.rs:406`) wired into helper
  push path (`reposix-remote/main.rs:552`). Agent removes a `.md`
  file → `git rm` + `git push` → REST DELETE. Shape promise
  survives the FUSE retirement. No round-trip test in
  `agent_flow_real` asserts the working-tree-unlink → REST-delete
  pipeline.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-03 \
    --rationale "IMPL_GAP: ConfluenceBackend::delete_or_close exists (lib.rs:406) and is wired into helper push path (reposix-remote/main.rs:552). FUSE unlink callback transport retired in v0.9.0; user-facing capability (rm .md + git push → page deleted) persists. No agent_flow_real test asserts the working-tree-rm → git-push → REST-delete round-trip end-to-end."
  ```
- **v0.12.1 cluster:** Confluence backend parity (push-path round-trip tests)

### Row: `docs/architecture/redirect`
- **Claim:** "Architecture page redirects to How it works
  (filesystem-layer, git-layer, trust-model)"
- **Source:** `docs/architecture.md:7-12`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** Per the heuristic table: "Redirect-only stub doc →
  CONFIRM_RETIRE the row but ALSO file: chunker should follow
  redirects (out of your scope; just note it)." The page is a
  3-link redirect stub (verified by reading
  `docs/architecture.md:1-13`). A redirect doc has no behavioral
  claim to bind a test to; the targets (`how-it-works/*.md`)
  generate their own claim rows. The chunker followed the prose
  literally and minted "redirects to..." as a behavioral claim,
  which is a false positive.
- **Note for chunker improvement (out-of-scope for this audit):**
  the extractor should detect frontmatter `title: "... (moved)"`
  + a body that's purely a bulleted list of internal links and
  emit zero rows for that file (file-level skip), OR follow the
  redirects and only emit rows from the destination pages.
- **Supersession source:** `docs/architecture.md:1-13` itself —
  the file declares its own status as "moved"; the canonical
  pages are `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md`.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id docs/architecture/redirect
  ```

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-02`
- **Claim:** "`cat mount/_INDEX.md` returns a whole-mount overview
  listing all backends, buckets, and top-level entry counts"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:30`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** This is a **user-facing shape promise**, not a
  FUSE transport detail. The `_INDEX.md` file at the working-tree
  root is a perfectly valid construct in a partial-clone world —
  the cache crate can synthesize an `_INDEX.md` blob and include
  it in the tree just like any other file. Verified that NO
  current code synthesizes `_INDEX.md` (`grep -rn "_INDEX"
  crates/reposix-cache/ crates/reposix-remote/` returns zero
  hits). The promise survives the pivot but the implementation
  was dropped during the FUSE deletion. Shipped status in
  REQUIREMENTS.md was `[x]` (claimed shipped in v0.6.0), so this
  is a true regression: feature shipped, then silently lost in
  the v0.9.0 pivot.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-02 \
    --rationale "IMPL_GAP: _INDEX.md whole-mount overview was shipped in v0.6.0 (REQUIREMENTS.md row marked [x]) but is absent from the partial-clone era — no current code synthesizes it. The promise (cat mount/_INDEX.md → overview) is a USER-FACING SHAPE claim, not a FUSE transport detail; cache can mint the blob in the tree. Resolution: either reimplement in reposix-cache as a synthesized blob in the bare-repo tree + bind a working-tree assertion test, OR write an ADR retiring the _INDEX.md feature with documented rationale + supersession (e.g. 'use git ls-tree' if that's the new UX)."
  ```
- **v0.12.1 cluster:** index-synthesis-regression

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-01`
- **Claim:** "`cat mount/tree/<subdir>/_INDEX.md` returns a recursive
  markdown sitemap of that subtree, computed via cycle-safe DFS
  from `TreeSnapshot`"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:29`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** Same logic as index-02. `tree/<subdir>/_INDEX.md`
  is a working-tree shape promise that survives the pivot. The
  cycle-safe-DFS-from-`TreeSnapshot` implementation detail is
  FUSE-era language (TreeSnapshot was a FUSE-internal type), but
  the *output* — a per-subtree sitemap blob — is just another
  blob in the partial-clone tree, easy to synthesize in the cache
  crate. Shipped in v0.6.0 (`[x]` in REQUIREMENTS.md), absent
  today.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-01 \
    --rationale "IMPL_GAP: tree/<subdir>/_INDEX.md per-subtree sitemap was shipped in v0.6.0 ([x] in REQUIREMENTS.md) but absent from partial-clone era. The cycle-safe-DFS-from-TreeSnapshot phrasing is FUSE-era jargon; the user-facing output (cat tree/X/_INDEX.md → sitemap) is a synthesized blob, trivially representable in a bare-repo tree. Resolution: reimplement synthesis in reposix-cache + bind test, OR ADR-retire with documented rationale (e.g. 'use git ls-tree --recurse')."
  ```
- **v0.12.1 cluster:** index-synthesis-regression

### Row: `docs/demo/redirect`
- **Claim:** "Demo page redirects to tutorials/first-run.md"
- **Source:** `docs/demo.md:7-9`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** Same as `docs/architecture/redirect`. The page
  is a 3-line "moved" stub pointing at `tutorials/first-run.md`.
  No behavioral claim. The first-run tutorial generates its own
  rows from its own page. Redirect-only stub.
- **Supersession source:** `docs/demo.md:1-9` declares "moved" in
  frontmatter; canonical page is `docs/tutorials/first-run.md`.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id docs/demo/redirect
  ```

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/cache-02`
- **Claim:** "`git diff HEAD~1` in the mount shows what changed at
  the backend since the last refresh (mount-as-time-machine)"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:38`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
  (lean) **/ NEEDS_OWNER_DECISION** (alternate framing — see below)
- **Reasoning:** This is a USER-FACING shape promise: "diff your
  working tree against the previous backend state and see what
  changed." The naive `git diff HEAD~1` literal phrasing is
  superseded by ADR-007 (time-travel via private git tags per
  `Cache::sync` — `refs/reposix/sync/<ISO8601>`), which is a
  STRICTLY STRONGER realization of the same intent (every sync is
  a checkable ref, not just the last one). The v0.6.0 row was
  marked `[ ]` (unshipped), and v0.11.0 ADR-007 ships the
  superior generalization. So either:
  - **(A) IMPL_GAP** — feature is shipped via sync-tags, the
    REQUIREMENTS.md prose just hasn't been updated to "git diff
    refs/reposix/sync/<earlier> refs/reposix/sync/<now>". Bind a
    test that walks the sync-tag history and asserts diff
    semantics. **Default: this.**
  - **(B) CONFIRM_RETIRE** with ADR-007 supersession — the
    `HEAD~1` literal is wrong (sync tags are the API surface),
    so retire the row outright and let the ADR-007 page generate
    its own claim rows.
  Owner can flip if (B) is preferred; I default to (A) because
  the underlying capability ("see what changed at backend") is a
  load-bearing UX promise and an explicit test binding strengthens
  the gate against future regressions.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/cache-02 \
    --rationale "IMPL_GAP: mount-as-time-machine UX is shipped in v0.11.0 via ADR-007 sync-tags (refs/reposix/sync/<ISO8601> per Cache::sync) which is a strictly stronger generalization of the v0.6.0 git-diff-HEAD~1 promise. The user-facing capability ('see what changed at the backend') survives the FUSE-to-partial-clone pivot. No test currently binds the sync-tag history surface to a 'time-travel via git diff between refs' assertion. Resolution: either update REQUIREMENTS.md prose to refer to sync-tags + bind a test that exercises diff between two sync refs, OR confirm-retire and let ADR-007 page mint its own claim rows."
  ```
- **v0.12.1 cluster:** time-travel-via-sync-tags (own cluster, or
  fold into the v0.11.0 ADR-007 binding work)

### Row: `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/helper-sim-backend-tech-debt-closed`
- **Claim:** "Helper-hardcodes-SimBackend tech debt closed in v0.11.0"
- **Source:** `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:28`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** This is a status-claim row (the file declares its
  own status: "closed in v0.11.0 (commit `cd1b0b6`, ADR-008)"),
  not a behavioral promise. ADR-008
  (`docs/decisions/008-helper-backend-dispatch.md`) is the
  authoritative documented decision; it shipped, the helper now
  does URL-scheme dispatch. Per the heuristic table: "Phase X
  completed and Phase Y added the feature → CONFIRM_RETIRE (with
  Phase Y supersession cite)." The behavior ("helper instantiates
  the right BackendConnector") is now claim-bound via rows from
  `docs/decisions/008-helper-backend-dispatch.md`, not from this
  archival REQUIREMENTS.md status note.
- **Supersession source:**
  `docs/decisions/008-helper-backend-dispatch.md` (Status:
  Accepted, dated 2026-04-24, closes Phase 32 carry-forward debt;
  commit `cd1b0b6`). The feature claim is owned by ADR-008 and
  its descendants in `quality/catalogs/doc-alignment.json`.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id planning-milestones-v0-10-0-phases-REQUIREMENTS-md/helper-sim-backend-tech-debt-closed
  ```

### Row: `docs/reference/jira.md/read-only-phase-28`
- **Claim:** "In Phase 28, `create_record`, `update_record`, and
  `delete_or_close` return not supported"
- **Source:** `docs/reference/jira.md:96-99`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_DOC_DRIFT**
- **Reasoning:** Per the heuristic table: "Phase X completed and
  Phase Y added the feature → CONFIRM_RETIRE (with Phase Y
  supersession cite)." But the source doc (`docs/reference/jira.md`)
  STILL CLAIMS read-only. This is **not a retirement** — Phase 29
  shipped the JIRA write path
  (`crates/reposix-jira/src/lib.rs:8`: "Phase 29 ships the full
  write path: `create_record`, `update_record`, and
  `delete_or_close` (via transitions API with DELETE fallback)";
  trait impls at lib.rs:197/279/334 with extensive tests at
  client.rs:836/863/872/900/933). The doc text in
  `jira.md:96-99` still says "Phase 28: read-only,
  not supported" which is a **doc-drift bug**: the docs say X
  doesn't work but the impl does X. Resolution is a doc edit, not
  a code change.
  - This is *exactly* the canonical doc-alignment failure mode
    (per `01-rationale.md`): doc claims X, impl does Y, no test
    binds either. The bias-toward-preservation rule applies: keep
    the row, flip to DOC_DRIFT, fix the doc.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id docs/reference/jira.md/read-only-phase-28 \
    --rationale "DOC_DRIFT: docs/reference/jira.md:96-99 claims Phase 28 read-only ('create_record/update_record/delete_or_close return not supported') but Phase 29 shipped the JIRA write path (reposix-jira/src/lib.rs:8 docstring; trait impls at lib.rs:197/279/334 with tests at client.rs:836/863/872/900/933). Resolution: update docs/reference/jira.md §Limitations to reflect Phase 29 write-path completion, then bind a test that asserts JIRA writes via dark-factory pattern (already covered by dark_factory_real_jira; rebind row to that test)."
  ```
- **v0.12.1 cluster:** JIRA shape (doc fix + test rebinding)

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-01`
- **Claim:** "`ls mount/labels/<label>/` lists all issues/pages
  carrying that label as read-only symlinks pointing to the
  canonical file in the bucket"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:33`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** USER-FACING SHAPE promise. The phrasing "FUSE
  symlinks" is FUSE-era language but the *concept* — a `labels/`
  directory in the working tree with one entry per labeled
  issue — is a tree-shape claim that's perfectly representable in
  a partial-clone bare-repo. git supports symlinks natively in
  trees (mode 120000). Today, NO code synthesizes `labels/`
  entries (`grep -rn "labels/" crates/reposix-cache/` returns
  zero). Marked `[ ]` in REQUIREMENTS.md, never shipped — but the
  promise survived the v0.8/9 transitions in archived prose and
  was inherited as a shape commitment. Per the heuristic table,
  this is a CHECK CAREFULLY case landing on IMPL_GAP.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-01 \
    --rationale "IMPL_GAP: labels/<label>/ tree shape (one entry per labeled issue, as symlinks to canonical paths) is a USER-FACING shape promise from v0.6.0 REQUIREMENTS.md, never shipped (marked [ ]). git natively supports symlinks in trees (mode 120000); reposix-cache could synthesize this in the bare-repo tree just like any other directory. The FUSE-symlink phrasing is jargon, not a transport binding. No code currently emits labels/ entries (grep -rn 'labels/' crates/reposix-cache/ returns zero). Resolution: either implement label-bucket synthesis in reposix-cache + bind a working-tree assertion test, OR write an ADR retiring the labels/ feature (e.g. 'agents use git grep on frontmatter labels: instead' — would need supersession rationale)."
  ```
- **v0.12.1 cluster:** Confluence backend parity / page-tree shape

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-02`
- **Claim:** "`ls mount/spaces/<key>/` lists all pages in that
  Confluence space (multi-space mount support)"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:34`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_IMPL_GAP**
- **Reasoning:** Multi-space mount is a USER-FACING shape promise,
  not a FUSE transport detail. In partial-clone language, this is
  "agent does `reposix init confluence::* /path` and the
  partial-clone has `spaces/<KEY>/...` directories at the root."
  Today, `reposix init` requires a single project (`<backend>::<project>`)
  per the CLAUDE.md elevator pitch, and the multi-space form
  isn't supported. Marked `[ ]` in REQUIREMENTS.md (and reaffirmed
  in v0.7.0 "Future Requirements" at REQUIREMENTS.md:43). The
  promise persists in prose; no impl. Per heuristic table:
  user-facing shape promise → IMPL_GAP.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-02 \
    --rationale "IMPL_GAP: spaces/<KEY>/ multi-space mount (one root directory per Confluence space) is a v0.6.0/v0.7.0 user-facing shape promise (REQUIREMENTS.md:34, restated at :43 as 'Future Requirements'), never shipped. The FUSE-mount phrasing is jargon; in partial-clone terms this means 'reposix init confluence::* materializes spaces/<KEY>/ directories in the partial-clone tree'. No current init path supports the wildcard. Resolution: either implement multi-space init + bind a test that asserts the spaces/ tree shape, OR write an ADR retiring the multi-space promise with supersession (e.g. 'one mount per space; multi-space is a workspace-level concern, out of scope')."
  ```
- **v0.12.1 cluster:** Confluence backend parity / page-tree shape

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/hard-04`
- **Claim:** "macFUSE parity: CI matrix entry for macOS with
  macFUSE, `fusermount3` -> `umount -f` conditional swap"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:78`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** Per the heuristic table top row: "Crate deleted
  (e.g. `reposix-fuse`) + claim is about that crate's syscall
  surface → CONFIRM_RETIRE." `macFUSE`, `fusermount3`, and
  `umount -f` are all syscall-level FUSE-transport details that
  evaporate completely with the FUSE → partial-clone pivot. The
  v0.9.0 architecture-pivot-summary.md §"Delete" explicitly lists
  these: "All FUSE-related runtime concerns: `/dev/fuse`
  permissions, `fusermount3` requirement, WSL2 kernel module
  configuration, stale mount cleanup. FUSE integration tests."
  No partial-clone equivalent of "macFUSE CI matrix" — the
  partial-clone runs anywhere git runs, by design.
  Shipped status was `[x]` in v0.7.0 REQUIREMENTS.md, but the
  feature ceased to apply with the pivot.
- **Supersession source:**
  `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`
  §"Delete" (lines 301–306) explicitly retires
  `fusermount3`/macFUSE/`/dev/fuse` requirements; v0.9.0 milestone
  closed with `crates/reposix-fuse/` deleted. The replacement
  ("partial-clone runs anywhere git runs, no system FUSE") is
  documented in §"Why this is better" (lines 53–63).
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/hard-04
  ```

### Row: `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/playwright-screenshots-deferred`
- **Claim:** "Playwright screenshots deferred to v0.11.0 due to
  cairo system libs unavailable on dev host (POLISH-11)"
- **Source:** `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:27`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **FLIP_TO_MISSING_TEST_DOC_DRIFT**
- **Reasoning:** Per the heuristic table: "'Deferred to v0.X'
  where v0.X has shipped → CONFIRM_RETIRE if shipped feature
  replaces; **FLIP_TO_MISSING_TEST_DOC_DRIFT if shipped feature's
  docs need an update**." `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:27`
  (the explicit "Carry-forward (closed in v0.11.x)" section
  immediately below the row) already says "Playwright screenshots
  — closed in v0.11.0 Phase 53 (POLISH-11)." So the deferral was
  resolved; this archival REQUIREMENTS.md prose is duplicating
  status with the line directly below it. The doc itself isn't
  wrong — v0.11.0 Phase 53 ships the screenshots — it's that this
  row says "deferred" while the very next bullet says "shipped."
  This is doc redundancy / drift inside an archived file. Default
  bias toward preservation: keep the row, flip to DOC_DRIFT,
  resolution is to either delete the duplicative line or rebind
  to the v0.11.0 Phase 53 verifier artifacts.
  - **Alternative:** owner could pick CONFIRM_RETIRE with
    Phase-53 supersession; the gain is small either way (this is
    a status-note row, not a behavioral promise).
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment mark-missing-test \
    --row-id planning-milestones-v0-10-0-phases-REQUIREMENTS-md/playwright-screenshots-deferred \
    --rationale "DOC_DRIFT: archival REQUIREMENTS.md row at line 27 says 'deferred to v0.11.0' but line 28 immediately below says 'closed in v0.11.0 Phase 53 (POLISH-11)' — the deferral status is already resolved by the Carry-forward note one line below. Resolution: either fold the deferred row into the resolved row (single source of truth in the archival REQUIREMENTS.md) OR rebind the claim to the v0.11.0 Phase 53 verifier artifacts. Low-stakes; this is not a behavioral promise."
  ```
- **v0.12.1 cluster:** archival-doc-cleanup (or optionally CONFIRM_RETIRE)

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-01`
- **Claim:** "`reposix-swarm --mode confluence-direct` exercises
  `ConfluenceBackend` directly (no FUSE overhead), mirroring
  `SimDirectWorkload` pattern"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:25`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** This claim is verifiably **already shipped**
  via the `confluence-direct` mode in `reposix-swarm`:
  - `crates/reposix-swarm/src/main.rs:31` enumerates
    `Mode::ConfluenceDirect` with CLI flag mapping at line 40
    (`"confluence-direct"`).
  - `crates/reposix-swarm/src/confluence_direct.rs:1` —
    "`confluence-direct` workload: each client drives
    [`ConfluenceBackend`]" (mirrors `SimDirectWorkload`).
  - `crates/reposix-swarm/src/metrics.rs:238` — mode-name
    enumeration includes `confluence-direct`.
  The phrase "no FUSE overhead" is now tautological (no FUSE
  exists), and the row is owned by the swarm crate's own claim
  rows (which the chunker should mint when `reposix-swarm`'s
  source / docs are extracted). The archival REQUIREMENTS.md row
  is now duplicative with shipped behavior.
  - Strictly this is "feature shipped, claim row owned elsewhere
    now" rather than "feature retired" — but per the heuristic
    table, "Phase X completed and Phase Y added the feature →
    CONFIRM_RETIRE." The shipped-elsewhere ownership is the
    supersession.
- **Supersession source:**
  - `crates/reposix-swarm/src/main.rs:31, 40` (Mode enum +
    `"confluence-direct"` CLI literal)
  - `crates/reposix-swarm/src/confluence_direct.rs:1, 22, 33, 72`
    (workload impl)
  - `crates/reposix-swarm/src/metrics.rs:238` (metrics handle)
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-01
  ```

### Row: `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-02`
- **Claim:** "Swarm run against confluence-direct produces summary
  metrics + audit-log rows, matching the sim-direct output format"
- **Source:** `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:26`
- **Existing extractor rationale:** none recorded.
- **My recommendation:** **CONFIRM_RETIRE**
- **Reasoning:** Same as swarm-01 — the metrics module
  (`crates/reposix-swarm/src/metrics.rs:238`) explicitly
  enumerates `confluence-direct` alongside `sim-direct` and
  `contention`, and the `Workload` trait shape (audit + metrics
  emission per workload) is consistent across modes. Shipped;
  ownership lives in the swarm crate's own claim rows.
- **Supersession source:** same as swarm-01.
- **Proposed action:**
  ```
  target/release/reposix-quality doc-alignment confirm-retire \
    --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-02
  ```

## Bulk-action scripts

### confirm-retire script (owner runs from TTY — env-guarded by `confirm-retire`)

```bash
#!/usr/bin/env bash
# v0.12.1 P67 -- bulk confirm legitimate retirements identified by audit.
# Owner runs this from a TTY; `reposix-quality doc-alignment confirm-retire`
# refuses to run from agent contexts per quality/PROTOCOL.md "Retirement
# requires explicit human signature" (01-rationale.md §"What we explicitly
# chose for").
#
# Pre-conditions:
#   - target/release/reposix-quality is built (cargo build -p reposix-quality --release)
#   - cwd is the reposix repo root
#   - terminal is a real TTY (not piped, not subprocess)

set -euo pipefail

# 6 rows: 2 redirect-only stub docs + 1 status-note (helper-sim-backend) +
# 1 transport-detail (macFUSE) + 2 shipped-elsewhere (swarm-01, swarm-02).
target/release/reposix-quality doc-alignment confirm-retire \
  --row-id docs/architecture/redirect

target/release/reposix-quality doc-alignment confirm-retire \
  --row-id docs/demo/redirect

target/release/reposix-quality doc-alignment confirm-retire \
  --row-id planning-milestones-v0-10-0-phases-REQUIREMENTS-md/helper-sim-backend-tech-debt-closed

target/release/reposix-quality doc-alignment confirm-retire \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/hard-04

target/release/reposix-quality doc-alignment confirm-retire \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-01

target/release/reposix-quality doc-alignment confirm-retire \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-02

echo "==> confirm-retire complete: 6 rows confirmed."
```

### mark-missing-test script (orchestrator can run; not env-guarded)

```bash
#!/usr/bin/env bash
# v0.12.1 P67 -- correct over-retired rows identified by audit.
# 8 IMPL_GAP + 2 DOC_DRIFT = 10 flips.
# Orchestrator can run this; mark-missing-test is not env-guarded.
#
# Pre-conditions:
#   - target/release/reposix-quality is built
#   - cwd is the reposix repo root

set -euo pipefail

# ─── IMPL_GAP cluster: Confluence write-path round-trip (P72) ────────
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-01 \
  --rationale "IMPL_GAP: ConfluenceBackend::create_record exists (lib.rs:280) and is wired into helper push path (reposix-remote/main.rs:513). The FUSE-mount transport was retired in v0.9.0 (architecture-pivot-summary.md) but the user-facing capability (write .md → Confluence page) persists via git push. No dark-factory-style test asserts the working-tree-edit → git-push → REST-create round-trip end-to-end. Resolution: bind a test in agent_flow_real that creates a page via working-tree write + git push and asserts the page exists on the backend."

target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-02 \
  --rationale "IMPL_GAP: ConfluenceBackend::update_record exists (lib.rs:335) with version-conflict handling and is wired into helper push path (reposix-remote/main.rs:540). Transport pivoted from FUSE write-callback to git push/fast-export in v0.9.0; user-facing capability (edit .md → page update) persists. No agent_flow_real test asserts working-tree-edit → git-push → REST-update round-trip with version reconciliation."

target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-03 \
  --rationale "IMPL_GAP: ConfluenceBackend::delete_or_close exists (lib.rs:406) and is wired into helper push path (reposix-remote/main.rs:552). FUSE unlink callback transport retired in v0.9.0; user-facing capability (rm .md + git push → page deleted) persists. No agent_flow_real test asserts the working-tree-rm → git-push → REST-delete round-trip end-to-end."

# ─── IMPL_GAP cluster: Confluence backend parity / page-tree shape (P72) ─
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-01 \
  --rationale "IMPL_GAP: labels/<label>/ tree shape (one entry per labeled issue, as symlinks to canonical paths) is a USER-FACING shape promise from v0.6.0 REQUIREMENTS.md, never shipped (marked [ ]). git natively supports symlinks in trees (mode 120000); reposix-cache could synthesize this in the bare-repo tree just like any other directory. The FUSE-symlink phrasing is jargon, not a transport binding. No code currently emits labels/ entries (grep -rn 'labels/' crates/reposix-cache/ returns zero). Resolution: either implement label-bucket synthesis in reposix-cache + bind a working-tree assertion test, OR write an ADR retiring the labels/ feature (e.g. 'agents use git grep on frontmatter labels: instead' — would need supersession rationale)."

target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-02 \
  --rationale "IMPL_GAP: spaces/<KEY>/ multi-space mount (one root directory per Confluence space) is a v0.6.0/v0.7.0 user-facing shape promise (REQUIREMENTS.md:34, restated at :43 as 'Future Requirements'), never shipped. The FUSE-mount phrasing is jargon; in partial-clone terms this means 'reposix init confluence::* materializes spaces/<KEY>/ directories in the partial-clone tree'. No current init path supports the wildcard. Resolution: either implement multi-space init + bind a test that asserts the spaces/ tree shape, OR write an ADR retiring the multi-space promise with supersession (e.g. 'one mount per space; multi-space is a workspace-level concern, out of scope')."

# ─── IMPL_GAP cluster: index-synthesis-regression ──────────────────────
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-01 \
  --rationale "IMPL_GAP: tree/<subdir>/_INDEX.md per-subtree sitemap was shipped in v0.6.0 ([x] in REQUIREMENTS.md) but absent from partial-clone era. The cycle-safe-DFS-from-TreeSnapshot phrasing is FUSE-era jargon; the user-facing output (cat tree/X/_INDEX.md → sitemap) is a synthesized blob, trivially representable in a bare-repo tree. Resolution: reimplement synthesis in reposix-cache + bind test, OR ADR-retire with documented rationale (e.g. 'use git ls-tree --recurse')."

target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-02 \
  --rationale "IMPL_GAP: _INDEX.md whole-mount overview was shipped in v0.6.0 (REQUIREMENTS.md row marked [x]) but is absent from the partial-clone era — no current code synthesizes it. The promise (cat mount/_INDEX.md → overview) is a USER-FACING SHAPE claim, not a FUSE transport detail; cache can mint the blob in the tree. Resolution: either reimplement in reposix-cache as a synthesized blob in the bare-repo tree + bind a working-tree assertion test, OR write an ADR retiring the _INDEX.md feature with documented rationale + supersession (e.g. 'use git ls-tree' if that's the new UX)."

# ─── IMPL_GAP cluster: time-travel-via-sync-tags (folds with ADR-007) ──
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-8-0-phases-REQUIREMENTS-md/cache-02 \
  --rationale "IMPL_GAP: mount-as-time-machine UX is shipped in v0.11.0 via ADR-007 sync-tags (refs/reposix/sync/<ISO8601> per Cache::sync) which is a strictly stronger generalization of the v0.6.0 git-diff-HEAD~1 promise. The user-facing capability ('see what changed at the backend') survives the FUSE-to-partial-clone pivot. No test currently binds the sync-tag history surface to a 'time-travel via git diff between refs' assertion. Resolution: either update REQUIREMENTS.md prose to refer to sync-tags + bind a test that exercises diff between two sync refs, OR confirm-retire and let ADR-007 page mint its own claim rows."

# ─── DOC_DRIFT cluster: JIRA shape (P72) ───────────────────────────────
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id docs/reference/jira.md/read-only-phase-28 \
  --rationale "DOC_DRIFT: docs/reference/jira.md:96-99 claims Phase 28 read-only ('create_record/update_record/delete_or_close return not supported') but Phase 29 shipped the JIRA write path (reposix-jira/src/lib.rs:8 docstring; trait impls at lib.rs:197/279/334 with tests at client.rs:836/863/872/900/933). Resolution: update docs/reference/jira.md §Limitations to reflect Phase 29 write-path completion, then bind a test that asserts JIRA writes via dark-factory pattern (already covered by dark_factory_real_jira; rebind row to that test)."

# ─── DOC_DRIFT cluster: archival-doc-cleanup ───────────────────────────
target/release/reposix-quality doc-alignment mark-missing-test \
  --row-id planning-milestones-v0-10-0-phases-REQUIREMENTS-md/playwright-screenshots-deferred \
  --rationale "DOC_DRIFT: archival REQUIREMENTS.md row at line 27 says 'deferred to v0.11.0' but line 28 immediately below says 'closed in v0.11.0 Phase 53 (POLISH-11)' — the deferral status is already resolved by the Carry-forward note one line below. Resolution: either fold the deferred row into the resolved row (single source of truth in the archival REQUIREMENTS.md) OR rebind the claim to the v0.11.0 Phase 53 verifier artifacts. Low-stakes; this is not a behavioral promise."

echo "==> mark-missing-test complete: 10 rows flipped (8 IMPL_GAP + 2 DOC_DRIFT)."
```

## Notes for the chunker / extractor (out-of-scope but worth filing)

The two `redirect`-stub false positives (`docs/architecture/redirect`,
`docs/demo/redirect`) suggest the chunker should detect frontmatter
`title: "... (moved)"` + a body that's purely a bulleted list of
internal links and emit zero rows for that file (file-level skip),
OR follow the redirects and only emit rows from the destination
pages. This would prevent ~2 retirement-noise rows per docs reorg
in future backfills.

The 8 IMPL_GAP flips also expose a structural extractor weakness:
the prose phrase "FUSE mount" appears in REQUIREMENTS.md as the
*transport* in claims that are really about *user-facing tree
shape*. A future extractor pass could explicitly **rewrite**
FUSE-era transport jargon ("FUSE mount", "fusermount3",
"TreeSnapshot DFS", "FUSE symlinks") into partial-clone language
("partial-clone working tree", "git checkout", "synthesized blob",
"git tree mode 120000") and re-grade — that single rewrite would
have prevented at least the 8 IMPL_GAP misclassifications in this
audit.

## Cluster summary for v0.12.1 planning

| Cluster | Rows | Phase placement |
|---|---|---|
| Confluence backend parity / push-path round-trip | write-01, write-02, write-03 | P72 (Confluence backend parity) |
| Confluence backend parity / page-tree shape | nav-01, nav-02 | P72 (Confluence backend parity) — could split |
| index-synthesis-regression | index-01, index-02 | own cluster (P72.x or P73) |
| time-travel-via-sync-tags | cache-02 | folds with v0.11.0 ADR-007 binding work |
| JIRA shape (doc fix + test rebinding) | docs/reference/jira.md/read-only-phase-28 | P72 (JIRA shape) |
| archival-doc-cleanup | playwright-screenshots-deferred | low-stakes; can batch with confirm-retires if owner prefers |
| (CONFIRM_RETIRE — no v0.12.1 work) | architecture/redirect, demo/redirect, helper-sim-backend, hard-04, swarm-01, swarm-02 | n/a |

End of audit.
