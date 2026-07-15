# Phase 114: t4 Confluence oid-drift fix-first + reconcile audit - Research

**Researched:** 2026-07-14
**Domain:** Confluence REST v2 adapter (body-representation asymmetry) + git-native
lazy-blob cache coherence (`reposix-cache`)
**Confidence:** HIGH (root cause and fix locus — verified against the actual codebase,
not assumed) / MEDIUM (Atlassian API pagination-cursor param preservation — WebSearch
cross-verified, not live-called in this research session)

## Summary

The oid-drift abort is NOT an eventual-consistency race (the code comment's
assumption) and NOT specific to page 7766017. It is a deterministic, structural
representation mismatch: `crates/reposix-confluence/src/client.rs::list_issues_impl`
(the `list_records`/`list_records_complete` implementation) requests
`GET /wiki/api/v2/spaces/{id}/pages?limit=100` with **no `body-format` query
parameter at all**, so Confluence Cloud returns every page's `body` as an empty
object — `ConfPage.body` decodes to `None`/`Some(ConfPageBody{storage:None,adf:None})`
either way. `crates/reposix-confluence/src/translate.rs::translate()` then falls
through to `page.body.and_then(|b| b.storage).map(|s| s.value).unwrap_or_default()` =
`String::new()` for every listed page. Meanwhile `get_record`
(`crates/reposix-confluence/src/lib.rs:253-325`) requests
`?body-format=atlas_doc_format` and returns the REAL page content (ADF→Markdown, or a
storage-HTML fallback for pre-ADF pages). `crates/reposix-cache/src/builder.rs`'s
`build_from` (the seed AND `--reconcile` path) computes the tree-blob oid from the
`list_records` render (body=""); `read_blob` (same file, lines 573-634) later
materializes the blob from `get_record`'s render (body=REAL content) and hard-aborts
with `Error::OidDrift` when the two hashes disagree — which they always will for any
Confluence page with non-empty ADF content. This is already independently documented
inside the codebase in three separate places (a doc comment on `ConfPage`, a comment in
`list_issues_impl`'s sibling test, and a dedicated real-backend regression test's doc
comment) — this research corroborates and connects those three observations into the
single root cause the requirement asks for.

**Primary recommendation:** Fix in the Confluence adapter (render-parity, Success
Criterion 3 option 1): add `&body-format=atlas_doc_format` to the LIST url built in
`list_issues_impl` (`crates/reposix-confluence/src/client.rs`, the `format!(...)` at
what is currently lines 195-200). `list_issues_impl` already funnels every listed page
through the SAME `translate()` function `get_record` uses (client.rs line ~251-252), so
this is a **single-query-param, zero-new-code-path fix** that makes the LIST body and
the primary-path GET body byte-identical for every ADF-native page — closing the drift
for the reproduction case (7766017) and (per the ADF-only-since-2020s nature of
Confluence Cloud) very likely the whole TokenWorld space. Do NOT fix this in
`build_from`'s oid computation (Option 2) — that would require calling `get_record` per
page during the cheap `list_records` walk, defeating the lazy-blob/L1-perf architecture
ADR-010 establishes (`build_from`'s own doc comment: "Does NOT materialize blobs").

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Confluence body-representation selection (`body-format` param) | Backend adapter (`reposix-confluence`) | — | The adapter owns how it talks to the Confluence REST API; representation choice is a transport-layer concern, not a cache concern |
| Tree-blob oid computation at seed/reconcile time | Cache builder (`reposix-cache::builder::build_from`) | Backend adapter (indirectly, via the `Record` it receives) | The cache computes oids from whatever `Record.body` the adapter handed it — it has no visibility into HOW the adapter got that body |
| Lazy blob materialization + drift detection | Cache builder (`reposix-cache::builder::read_blob`) | — | The safety net that VALIDATES cache↔backend coherence at read time; correctly refuses to serve mismatched content (this is NOT the bug — it is working as designed) |
| `sync --reconcile` recovery semantics + doc accuracy | CLI (`reposix-cli::sync`) + Cache (`reposix-cache::builder::build_from`, `error.rs::OidDrift` doc) | — | The CLI surface makes the user-facing promise; the underlying `build_from` call is what actually determines what gets healed |

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FIX-01 | Fix the Confluence `list_records`-vs-`get_record` oid-drift product defect (root cause `builder.rs:610-618` `read_blob`'s drift check; actual defect origin is `client.rs::list_issues_impl`'s missing `body-format` param). | Root-cause section below cites exact file:line for both the drift-check site and the defect-origin site; recommends the adapter render-parity fix locus with a one-line code change + residual-risk analysis (pre-ADF pages) |
| FIX-02 | Audit `sync --reconcile`'s oid-drift recovery claim; correct `builder.rs`/`cache.rs` doc comments if the claim doesn't hold for the systematic drift class. | FIX-02 section below identifies the 3 real doc-comment locations (not literally in `builder.rs`/`cache.rs` alone — also `reposix-cli/src/sync.rs` and `reposix-cache/src/error.rs`), proves (by code-reading, not assumption) that `--reconcile` cannot recover the systematic class pre-FIX-01, and gives the exact corrected scope each comment needs |

## Root Cause (verified against the codebase, file:line)

### The drift-check site (working as designed — NOT the bug)

`crates/reposix-cache/src/builder.rs::read_blob` (async fn, lines 573-634):

```rust
// line 590-599: fetch the CURRENT content via get_record
let issue = self.backend.get_record(&self.project, RecordId(issue_num)).await?;

// line 602-608: render + hash it
let rendered = reposix_core::frontmatter::render(&issue)?;
let bytes = rendered.into_bytes();
let written_oid = self.repo.write_blob(&bytes)?.detach();

// line 610-618: compare against the oid computed at build_from time
if written_oid != oid {
    return Err(Error::OidDrift { requested: oid_hex, actual: written_oid.to_hex().to_string(), issue_id: issue_id_str });
}
```

`oid` here is the tree-blob oid computed and written into `oid_map` by `build_from`
(`crates/reposix-cache/src/builder.rs`, seed-path loop at lines 107-122):

```rust
for issue in &issues {                       // `issues` came from list_records_complete
    let rendered = frontmatter::render(issue)?;
    let bytes = rendered.into_bytes();
    let oid = gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)?;
    ...
}
```

This drift-check is a CORRECT, intentional safety net — comparing "what we promised the
git tree" against "what we can actually serve" and refusing to silently serve a lie. Do
not weaken or remove it. The bug is that the two renders are guaranteed to disagree,
not that the check itself is wrong.

### The actual defect (root cause)

`crates/reposix-confluence/src/client.rs::list_issues_impl` (pagination loop backing
`list_records`/`list_records_complete`/`list_records_strict`), the initial-page URL:

```rust
let first = format!(
    "{}/wiki/api/v2/spaces/{}/pages?limit={}",
    self.base(), space_id, PAGE_SIZE
);
```

**No `body-format` query parameter.** [VERIFIED: codebase] Per Confluence REST v2
semantics, omitting `body-format` means the API does not include page body content at
all in the list response — confirmed by THREE independent, pre-existing doc comments
already in this codebase (not written for this research; corroborating, not
introducing, the finding):

1. `crates/reposix-confluence/src/types.rs:108-110` — `ConfPage`'s own doc comment: "A
   single page as returned by both the list endpoint (with `body: {}` empty) and the
   single-page endpoint (with `body.storage.value` populated when `?body-format=storage`
   is requested)."
2. `crates/reposix-confluence/src/client.rs:1259` — test-section comment: "status
   `"current"` → `Open` (via `get_record`, **since list omits body**)".
3. `crates/reposix-cli/tests/agent_flow_real.rs:210-223` — a LIVE real-backend test's
   doc comment (see "Files to touch" below), explaining in detail why it must drive
   `get_record` and not `list_records` to exercise the ADF-decode path, because "`list_records`
   requests NO body-format, so every listed record carries an EMPTY body."

Contrast with `get_record` (`crates/reposix-confluence/src/lib.rs:253-325`), whose
FIRST attempt is:

```rust
let url_adf = format!("{}/wiki/api/v2/pages/{}?body-format=atlas_doc_format", self.base(), id.0);
```

…and only falls back to `?body-format=storage` if the ADF value is null (pre-ADF legacy
page, lib.rs:293-299).

**Why the drift is deterministic, not a race:** `crates/reposix-core/src/record.rs::frontmatter::render`
(lines 130-144) serializes a `Frontmatter` struct (`id`, `title`, `status`, `assignee`,
`labels`, `created_at`, `updated_at`, `version`, `parent_id`, `extensions` — everything
EXCEPT `body`) as YAML, then appends `body` verbatim. All of those frontmatter fields
come from the SAME Confluence page resource in both the list and get responses (same
`id`, unmutated between calls) — so for an unmutated page, list-render and get-render
differ ONLY in the `body` bytes: `""` (list) vs. real content (get). Two re-runs of the
same `list_records`→hash computation therefore always produce the SAME oid (confirmed
independently in `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`'s
2026-07-14 20:40 entry: "Deterministic: re-ran validate-only, byte-identical oids both
runs"). This is why "page 7766017 is an UNMUTATED protected fixture" rules out
eventual-consistency: nothing about the backend's state changed between the two calls;
only the REQUEST shape differed.

### Data flow (list vs. get divergence)

```
reposix init confluence::TokenWorld  ──▶  Cache::build_from()
                                             │
                                             ▼
                              backend.list_records_complete(project)
                                             │  GET /wiki/api/v2/spaces/{id}/pages?limit=100
                                             │  (NO body-format param)
                                             ▼
                              translate(page)  body = page.body.storage?.value ?? ""
                                             │
                                             ▼
                         oid_A = hash(frontmatter::render(Record{body: ""}))
                         written into oid_map + the git tree (NOT materialized yet)
                                             .
                                             .  (later, on first `git checkout`)
                                             ▼
                              git-remote-reposix stateless-connect fetch
                                             │
                                             ▼
                                    Cache::read_blob(oid_A)
                                             │
                                             ▼
                              backend.get_record(project, id)
                                             │  GET /wiki/api/v2/pages/{id}?body-format=atlas_doc_format
                                             │  (real ADF → Markdown, or storage-HTML fallback)
                                             ▼
                              translate(page)  body = <REAL page content>
                                             │
                                             ▼
                         oid_B = hash(frontmatter::render(Record{body: <real>}))
                                             │
                                    oid_A != oid_B  ──▶  Error::OidDrift  ──▶  git checkout ABORTS
```

## Recommended fix locus (Success Criterion 3, Option 1 — adapter render-parity)

**Change:** `crates/reposix-confluence/src/client.rs::list_issues_impl`, initial-page
`format!` (and verify the subsequent `next_url` — see Open Questions):

```rust
let first = format!(
    "{}/wiki/api/v2/spaces/{}/pages?limit={}&body-format=atlas_doc_format",
    self.base(), space_id, PAGE_SIZE
);
```

**Why this is correct, not a workaround:**
- It targets the actual defect origin (the LIST request shape), not the drift-check
  (which stays intact as a coherence guard).
- It reuses the EXISTING `translate()` function unchanged — `list_issues_impl` already
  calls `translate(tainted.into_inner())` per page (client.rs ~line 251-252), so ADF
  decode, the item-4b fail-closed sentinel, and the storage-fallback logic are all
  ALREADY correctly wired for whatever body Confluence returns; no new translation code
  path is introduced.
- It preserves the lazy-blob architecture: `build_from` still does zero per-page GETs
  (one extra query param on the SAME paginated LIST call, not N extra round-trips) —
  consistent with `build_from`'s own doc comment ("Does NOT materialize blobs") and
  ADR-010's L1 perf goal.
- [CITED: developer.atlassian.com / community.developer.atlassian.com, cross-verified via
  WebSearch, MEDIUM confidence — not a live API call in this research session] Confluence
  REST v2's `GET /wiki/api/v2/spaces/{id}/pages` endpoint DOES accept `body-format` with
  values `storage` and `atlas_doc_format` (NOT `view` — that's GET-single-page only, per
  Jira ticket CONFCLOUD-79781 "Ability to specify `?body-format=view` for V2 'Get pages'"
  implying storage/atlas_doc_format are already supported on the list endpoint today).

**Why NOT Option 2 (`build_from`'s oid computation using the get-representation):**
Doing so would require `build_from` to call `get_record` per page during what is
documented and architected as a cheap, non-materializing listing walk — a 1-call → N-call
amplification (TokenWorld is small, but GitHub/JIRA-scale spaces up to
`MAX_ISSUES_PER_LIST = 500` would turn one paginated list into up to 500 individual GETs)
that directly contradicts `build_from`'s doc comment and the ADR-010 lazy-materialization
invariant the `git-remote-reposix` stateless-connect handler depends on
(`crates/reposix-cache/src/builder.rs:26-29`). Confirm this contradiction with the
planner before considering Option 2 for any residual gap Option 1 doesn't close.

## Residual gap (must be documented honestly, not hidden)

Requesting `atlas_doc_format` on the LIST call achieves render-parity only for
**ADF-native pages** (the primary path both `list_issues_impl` and `get_record` share).
For a **pre-ADF legacy page** (rare on modern Confluence Cloud, but not impossible):
- `get_record` detects `adf_present == false` and does a SECOND GET with
  `?body-format=storage`, returning real storage HTML (lib.rs:293-324).
- The fixed `list_issues_impl` has no equivalent per-page fallback — a page whose ADF
  value is null in the LIST response has NO storage fallback available (storage was
  never requested), so `translate()` falls into the item-4b fail-closed sentinel path
  (translate.rs:117-138) instead of empty string. This is a DIFFERENT non-empty render
  than get_record's real storage HTML — still a drift, just for a narrower, likely-empty-
  in-practice subset of pages.

This is [ASSUMED: pre-ADF pages are rare-to-absent in a 2026 Confluence Cloud tenant
created for testing (TokenWorld)] — not verified against TokenWorld's actual page
inventory in this research session. Recommend the plan explicitly test against the WHOLE
TokenWorld space (not just page 7766017) to confirm no pre-ADF pages exist there, and
document the residual gap in the `Error::OidDrift` doc comment (see FIX-02) rather than
silently claiming universal Confluence compatibility. If pre-ADF pages DO turn up, the
planner has two escalation options: (a) accept the residual OidDrift abort for that
narrow subset (still strictly better than the current 100%-broken state) and file a
GOOD-TO-HAVE, or (b) extend `list_issues_impl` with a bounded per-page storage-format
GET ONLY for pages whose ADF came back null (not all pages) — a scoped enhancement, not
required to satisfy SC1/SC2 if TokenWorld has none.

## FIX-02: what `sync --reconcile` actually recovers (verified against code, not assumed)

`reposix sync --reconcile` (`crates/reposix-cli/src/sync.rs::run`, lines 53-121) calls
`Cache::build_from()` DIRECTLY (line 112-115) — the exact SAME function, exact same
`list_records_complete` call, exact same render/hash logic analyzed above. There is no
separate "reconcile" code path; `--reconcile` is simply "force the seed-path full
rebuild instead of the delta path."

**What it DOES recover (verified TRUE):**
1. **Tree↔`oid_map` coherence drift** (ghost rows from a pre-fix binary, missing rows,
   stale prune state) — a full `list_records` walk + `meta::prune_oid_map` in the SAME
   transaction (builder.rs:132-183) brings `oid_map` back in sync with whatever
   `list_records` CURRENTLY reports. This is the claim `sync.rs`'s doc comment
   (lines 11-14, 40-43) is actually about, and it is accurate for this class.
2. **`meta.last_fetched_at` cursor drift** — `cache.rs::write_last_fetched_at`'s doc
   comment (lines 575-582, "Cursor drift is recoverable via `reposix sync --reconcile`")
   is about a DIFFERENT, narrower kind of drift (temporal cursor staleness, not
   blob-content oid mismatch) and remains accurate as written — but the term "drift" is
   overloaded across `sync.rs`/`error.rs`/`cache.rs` in a way that invites exactly the
   GTH-V15-19 confusion; worth tightening language even where the underlying claim holds.
3. **Genuine eventual-consistency races** — the scenario `error.rs::Error::OidDrift`'s
   doc comment (lines 63-65) was originally written for: backend content changed BETWEEN
   `build_from`'s list computation and a later `get_record` fetch, and the backend has
   since caught up. Re-listing recomputes the list-oid from CURRENT backend state, which
   (once the race resolves) matches `get_record`'s current state again.

**What it does NOT recover (verified FALSE, pre-FIX-01):** The systematic
list-vs-get representation-drift class this phase targets. Re-running `build_from` calls
`list_records_complete` AGAIN with the SAME missing-`body-format` request shape, so it
recomputes the SAME empty-body-derived oid, deterministically, every time — proven by
the render-bytes analysis above (nothing time-dependent about the mismatch) and by the
SURPRISES-INTAKE entry's "byte-identical oids across two validate-only re-runs." The doc
comments' general phrasing ("heals a cache that has already drifted from the
tree↔`oid_map` coherence invariant") is TRUE for its own stated invariant (coherence)
but gets read by an adjacent audience (GTH-V15-19's filer) as "heals oid-drift in
general," which is FALSE for this class. **After FIX-01 lands**, `--reconcile` DOES
correctly heal any residual instance of this class too (list and get renders now agree
for ADF-native pages, so a re-list produces the matching oid) — the doc fix should state
this precondition explicitly rather than make an unconditional claim.

### Exact doc-comment locations needing correction

| File:line | Current claim | Correction needed |
|---|---|---|
| `crates/reposix-cache/src/error.rs:63-65` (`Error::OidDrift` doc) | "Indicates an eventual-consistency race on the backend side (same issue id, different content between `list_records` and `get_record`)." | Name BOTH causes: (1) a genuine eventual-consistency race (backend content changed between calls — reconcile CAN heal this), and (2) a systematic backend rendering-representation mismatch, e.g. pre-FIX-01 Confluence list-vs-get body-format asymmetry (reconcile CANNOT heal this without the adapter fix; post-FIX-01 it's closed for ADF-native pages). |
| `crates/reposix-cli/src/sync.rs:1-24` (module doc) + `:34-52` (`run` fn doc) | "`--reconcile` can heal a cache that has already drifted from the tree↔`oid_map` coherence invariant... which a delta sync cannot do." | Scope explicitly to tree↔`oid_map` coherence (ghost/missing rows) + genuine eventual-consistency races; add a caveat that a SYSTEMATIC backend-side rendering mismatch (the class `Error::OidDrift`'s doc now names) requires the adapter/backend fix itself, not a reconcile. |
| `crates/reposix-cli/src/main.rs:167-180` (`Sync` clap doc, lower priority — same family, keep consistent) | Similar general "cache desync" framing | Optional consistency pass in the same phase; not required to satisfy FIX-02's acceptance criterion but flagged for "noticing is a deliverable." |

`crates/reposix-cache/src/cache.rs:575-582` (`write_last_fetched_at`) — reviewed, claim
is ACCURATE for cursor drift specifically; no change required, but the planner should
confirm this stays true after editing the neighboring files so the reader doesn't
conflate cursor-drift-recovery with oid-drift-recovery.

## Files to touch + closest analogs

| File | Role | Existing pattern/signature to respect |
|---|---|---|
| `crates/reposix-confluence/src/client.rs` (`list_issues_impl`, ~line 195-200) | PRIMARY FIX — add `&body-format=atlas_doc_format` to the initial list URL | Mirror `get_record`'s URL construction style (`format!("{}/wiki/api/v2/pages/{}?body-format=atlas_doc_format", self.base(), id.0)`, lib.rs:255-259) — same query-param spelling, same base()/trim pattern |
| `crates/reposix-confluence/src/client.rs` (cursor-follow loop, `next_url = next_cursor.map(...)`, ~line 269-279) | VERIFY (see Open Questions) whether Confluence's own `_links.next` preserves `body-format`; if not, defensively re-append it here the same way `base()` is prepended | Existing pattern: `if relative.starts_with("http")... else format!("{}{}", self.base(), relative)` — extend with a `body-format` param check/append if the live TokenWorld response omits it across pages |
| `crates/reposix-confluence/tests/contract.rs` (wiremock-based, e.g. patterns at lines 385-420, 458-517) | Existing LIST mocks match `.and(query_param("limit","100"))` only — wiremock's `query_param` matcher does NOT reject extra params, so these should keep passing unmodified; ADD a new dedicated test asserting the list mock now ALSO receives `body-format=atlas_doc_format` (`.and(query_param("body-format","atlas_doc_format"))`) and that `list_records`'s returned `Record.body` is non-empty/matches `get_record`'s body for the same fixture page | Mirror the `adversarial_webui_link_does_not_trigger_outbound_call` test's list+get dual-mock shape (lines 437-539) for a natural "list body == get body" parity assertion |
| `crates/reposix-cache/tests/oid_drift_reconcile.rs` (NEW file, cache-crate-level, backend-agnostic) | Proves FIX-01 (drift resolves once list/get renders agree) AND FIX-02 (a second `build_from()` call does NOT change a stale list-oid when list/get renders still diverge) via a mock `BackendConnector`, without touching real Confluence | Direct analog: `crates/reposix-cache/tests/pagination_prune_safety.rs`'s `CappingMock` (implements `BackendConnector` with `list_records` returning a `Mutex<Vec<Record>>` independent from `get_record`'s resolution against a separate `full` set) — build a sibling `DriftingMock` where `list_records`'s `Record.body` differs from `get_record`'s `Record.body` for the SAME id, toggle-able like `CappingMock::truncate_to`. Reuse `crates/reposix-cache/tests/common/mod.rs::CacheDirGuard` + `sample_issues` |
| `crates/reposix-cli/tests/agent_flow_real.rs:210-223, 245-246` (comment only, no assertion change needed) | STALE after the fix — the comment "list_records only to ENUMERATE ids (its bodies are empty — see the doc comment)" becomes inaccurate for ADF-native pages once FIX-01 lands | Update the comment to describe the POST-fix state (list bodies now populated for ADF-native pages via `?body-format=atlas_doc_format`); the test's actual assertions (driving `get_record` for the decode-path regression lock) remain correct and do not need behavior changes |
| `crates/reposix-confluence/src/types.rs:108-110` (`ConfPage` doc comment) | STALE after the fix — "as returned by both the list endpoint (with `body: {}` empty)" | Update to describe the new list-endpoint request shape (body IS populated when a page has ADF content) |
| `crates/reposix-cache/src/error.rs:63-65` (`Error::OidDrift` doc) | FIX-02 — scope correction (see table above) | — |
| `crates/reposix-cli/src/sync.rs:1-24, 34-52` (module + fn doc) | FIX-02 — scope correction (see table above) | — |
| `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` (NOT required by FIX-01/FIX-02, but adjacent noticing) | Its `hard_fail_exit` calls on `git checkout -B main` failure hardcode `"requires git >= 2.34 stateless-connect fetch"` as the detail message even when the REAL cause is an oid-drift abort (lines 47-50 of `lib/t4-real-backend-flow.sh`) — this is the SEPARATE MEDIUM-severity SURPRISES-INTAKE entry (2026-07-14 20:41) about a misleading error message. Out of scope for FIX-01/FIX-02 per that entry's own "why out-of-scope" note, but worth a one-line pointer in the plan so the executor doesn't conflate the two while working in this file's neighborhood. | — |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ADF→Markdown conversion for the newly-populated list bodies | A parallel/duplicate translation path inside `list_issues_impl` | The EXISTING `translate()` function (`translate.rs:100-144`), already called per-page in `list_issues_impl` (client.rs ~line 251-252) | It already handles ADF decode, item-4b fail-closed sentinel, and storage fallback correctly — the fix needs zero new translation logic, only a URL query-param change |
| Cursor-pagination query-param propagation | A hand-rolled URL query-string merge/rebuild for `next_url` | Trust `_links.next` as Confluence returns it (existing pattern, `parse_next_cursor` + `self.base()` prepend) UNLESS the reproduction proves body-format is dropped across pages — then defensively re-append using the SAME `format!` idiom already used for the base-URL prepend, not a new URL-parsing dependency | Confluence's v2 cursor is designed to be followed as-is (general REST cursor convention, WebSearch-cross-verified); avoid `url::Url` query-merge machinery unless the repro proves it's needed |

## Common Pitfalls

### Pitfall 1: Fixing the drift-check instead of the defect origin
**What goes wrong:** Weakening or removing the `written_oid != oid` check in `read_blob`
(e.g. logging-and-continuing, or silently accepting the get-time bytes) would make
checkout "succeed" but breaks git's content-addressing invariant — a tree entry's oid
would no longer correspond to the blob's actual content in the object DB, which is a
correctness/coherence regression, not a fix. It would also mask FUTURE genuine
eventual-consistency races that this check is designed to catch.
**Why it happens:** It's the smallest code change and directly touches the file the
requirement's Root Cause line (`builder.rs:610-618`) points at.
**How to avoid:** Fix the RENDER INPUT (the Confluence adapter's list request), not the
CHECK. Success Criterion 3 explicitly forbids a workaround.
**Warning signs:** A diff that touches `read_blob`'s comparison logic, error variant, or
turns `Error::OidDrift` into a warning, is very likely the wrong fix for this phase.

### Pitfall 2: Assuming the fix universally closes ALL Confluence oid-drift
**What goes wrong:** Claiming "Confluence oid-drift is fully solved" in commit messages
or doc comments when only the ADF-native-page path is closed leaves the pre-ADF residual
gap (see "Residual gap" above) undocumented, setting up a FUTURE surprise.
**Why it happens:** SC1/SC2 only require page 7766017 + the t4 gate to pass, which the
fix satisfies without touching the pre-ADF edge case.
**How to avoid:** Scope claims precisely — "closes the systematic list-vs-get drift for
ADF-native pages" — in both the code comments (FIX-02) and any commit/PR message.

### Pitfall 3: Reconcile-doc fix that flips the claim to "reconcile never recovers oid-drift"
**What goes wrong:** Over-correcting FIX-02 into "reconcile does not recover oid-drift,
period" would ALSO be false — it DOES recover tree↔oid_map coherence drift and genuine
eventual-consistency races, and (post-FIX-01) the systematic class too.
**Why it happens:** The temptation to write the simplest possible corrected sentence
once the systematic-class failure is proven.
**How to avoid:** Scope the corrected doc comment to name what IS and ISN'T recovered,
per the FIX-02 table above — three coexisting drift classes, not one.

## Code Examples

### The one-line fix (illustrative — exact surrounding code may shift; use client.rs's
current `list_issues_impl` as source of truth at execution time)

```rust
// Source: crates/reposix-confluence/src/client.rs::list_issues_impl (~line 195-200)
// BEFORE:
let first = format!(
    "{}/wiki/api/v2/spaces/{}/pages?limit={}",
    self.base(), space_id, PAGE_SIZE
);
// AFTER:
let first = format!(
    "{}/wiki/api/v2/spaces/{}/pages?limit={}&body-format=atlas_doc_format",
    self.base(), space_id, PAGE_SIZE
);
```

### Existing shared render/translate pattern the fix relies on (no change needed here)

```rust
// Source: crates/reposix-confluence/src/client.rs::list_issues_impl (~line 249-252)
for page in list.results {
    let tainted = Tainted::new(page);           // SG-05: taint wrap before translate
    let issue = translate(tainted.into_inner())?; // Same translate() get_record uses
    out.push(issue);
    ...
}
```

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | TokenWorld's Confluence space contains no pre-ADF (legacy storage-only) pages, so the residual gap doesn't block SC1/SC2. | Residual gap | If wrong, the t4 gate could still fail on a DIFFERENT page than 7766017 after the fix lands; the planner should verify against a live `list_records` walk of the whole space, or accept and document the narrower residual. |
| A2 | Confluence's `_links.next` cursor on `GET /wiki/api/v2/spaces/{id}/pages` preserves the original request's `body-format` query param across pages, so appending it once to the FIRST page URL is sufficient. | "Files to touch" table, cursor-follow loop | If wrong, pages beyond the first 100 (TokenWorld is well under this, so SC1/SC2 are unaffected either way) would silently revert to empty bodies, reintroducing drift for a LARGER space in a future phase/backend. Low risk to THIS phase's acceptance criteria; the executor should still verify with a real multi-page TokenWorld-sized fixture if feasible, or add the defensive re-append proactively. |
| A3 | The three doc-comment locations identified for FIX-02 (`error.rs`, `sync.rs`, `main.rs`) are the complete set the requirement's "builder.rs/cache.rs doc comments" language refers to — the literal file names in the requirement/GTH-V15-19 filing were written from memory and are not 100% precise about which file. | FIX-02 section | If a planner searches ONLY `builder.rs`/`cache.rs` literally and finds no matching claim there, they might conclude FIX-02 is a no-op; this research's grep-verified table should be treated as authoritative over the requirement's literal file-name wording. |

**Assumption A2 is the only one with any bearing on SC1/SC2 correctness, and it is
LOW risk because TokenWorld's page count is well under `PAGE_SIZE=100` (pagination
never triggers in the reproduction).**

## Open Questions

1. **Does the real TokenWorld space contain any pre-ADF pages?**
   - What we know: page 7766017 is ADF-native (the repro/regression-lock test
     `get_record_real_confluence_body_is_real_markdown_not_unreadable_sentinel` already
     proves `get_record(7766017)` decodes real ADF markdown, not the sentinel).
   - What's unclear: whether ANY other page in the space is legacy storage-only.
   - Recommendation: the executor's real-backend verification step (t4 gate re-run)
     will empirically answer this — if it passes GREEN with zero drift across every
     page in the space, the residual gap is proven absent in this fixture; if a
     DIFFERENT page id trips `OidDrift` after the fix, that page is very likely
     pre-ADF, confirming the gap and giving a concrete follow-up target.

2. **Does `_links.next` preserve `body-format`?**
   - What we know: general Atlassian pagination guidance says "use the next link as
     returned, don't reconstruct it" (implying it should self-contain everything
     needed).
   - What's unclear: not verified with a live multi-page Confluence v2 response in
     this research session.
   - Recommendation: non-blocking for SC1/SC2 (TokenWorld has < 100 pages, no second
     page is ever fetched in the repro). If the executor wants a fully robust fix
     immune to future TokenWorld growth, add a defensive check/re-append on `next_url`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Live Confluence TokenWorld (`ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, `REPOSIX_ALLOWED_ORIGINS`) | SC1, SC2 (real-backend proof) | Not probed in this research session (constraint: do not hit any real backend) | — | Read-only repro (`reposix init confluence::TokenWorld /tmp/x && cd /tmp/x && git checkout -B main`) needs NO mutation-authorization beyond the standing TokenWorld sanction already documented in `docs/reference/testing-targets.md`; per project convention, `.env` must be sourced in the SAME bash invocation as the command that uses it (shell state does not persist across tool calls) |
| `git >= 2.34` | SC2 (t4 gate's own precondition) | Per `.planning/MANAGER-HANDOVER.md`, the executing VM already runs git 2.50.1 — precondition already satisfied | 2.50.1 (per handover) | — |
| Wiremock (`wiremock` crate, already a dev-dependency of `reposix-confluence`) | New/updated unit tests | Already in `Cargo.toml` (used throughout `contract.rs`) | — | — |
| `cargo build -p reposix-cli -p reposix-remote` (t4 gate's own build step) | SC2 | Must respect the ONE-cargo-invocation-machine-wide rule (root CLAUDE.md) | — | — |

**Missing dependencies with no fallback:** none — this phase does not need any new
external tool; it needs the ALREADY-sanctioned TokenWorld credentials the executing
session presumably already has per `.planning/MANAGER-HANDOVER.md`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo nextest` (per crates/CLAUDE.md) + `#[tokio::test]` / wiremock integration tests |
| Config file | none dedicated — standard `Cargo.toml` dev-dependencies per crate |
| Quick run command | `cargo test -p reposix-confluence` (wiremock, no network) / `cargo test -p reposix-cache` |
| Full suite command | `cargo nextest run` (workspace) — respect the ONE-cargo-invocation rule; prefer `-p <crate>` scoped runs during iteration, full workspace only at the pre-push gate boundary |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FIX-01 | `list_issues_impl` requests `body-format=atlas_doc_format`; wiremock LIST stub receives the new param | unit (wiremock) | `cargo test -p reposix-confluence --lib list_issues_impl` (or the specific new/updated test name once authored) | ❌ Wave 0 — new/updated test in `crates/reposix-confluence/tests/contract.rs` |
| FIX-01 | `list_records`'s returned `Record.body` matches `get_record`'s returned `Record.body` for the same ADF-native fixture page (render-parity assertion) | unit (wiremock) | `cargo test -p reposix-confluence --test contract` | ❌ Wave 0 — new test |
| FIX-01 | A mock backend with divergent list/get bodies triggers `Error::OidDrift` pre-fix, and resolves cleanly once bodies are aligned (regression lock, backend-agnostic) | unit (cache-crate integration test w/ mock `BackendConnector`) | `cargo test -p reposix-cache --test oid_drift_reconcile` | ❌ Wave 0 — new file, modeled on `crates/reposix-cache/tests/pagination_prune_safety.rs`'s `CappingMock` |
| FIX-01 (SC1) | `git checkout -B main` against live Confluence TokenWorld (incl. page 7766017) completes with zero oid-drift abort | shell-subprocess, real-backend | `bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` (env-gated; see Environment Availability) — OR the cheaper read-only repro: `reposix init confluence::TokenWorld /tmp/x && cd /tmp/x && git checkout -B main` (leaf-isolation: run in a throwaway `/tmp` clone per root CLAUDE.md's leaf-isolation rule, `cd` in the SAME bash invocation) | ✅ exists (gate script); read-only repro is ad hoc, not a committed test |
| FIX-01 (SC2) | The P0 gate `agent-ux/t4-conflict-rebase-ancestry-real-backend` passes GREEN against live Confluence TokenWorld | shell-subprocess, real-backend, catalogued | `python3 quality/runners/run.py --row agent-ux/t4-conflict-rebase-ancestry-real-backend` (or the cadence it's tagged under) | ✅ exists |
| FIX-02 | A mock backend proves a SECOND `build_from()` call (simulating `--reconcile`) does NOT change a stale list-derived oid when list/get bodies still diverge (proves the pre-FIX-01 non-recovery claim empirically) | unit (cache-crate integration test) | `cargo test -p reposix-cache --test oid_drift_reconcile` (same new file as above, additional test fn) | ❌ Wave 0 — same new file |
| FIX-02 (SC4) | `error.rs`/`sync.rs` doc comments accurately scope the recovery claim (3 drift classes named) | manual / docs-alignment review | `cargo doc -p reposix-cache -p reposix-cli --no-deps` (visual read) + code review | N/A — doc-only, no automated assertion possible for prose accuracy; the verifier subagent reads the corrected comments against this research's FIX-02 table |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-confluence -p reposix-cache` (scoped, fast — respects the one-cargo-invocation-machine-wide rule)
- **Per wave merge:** `cargo nextest run` (full workspace) + `bash quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` (sim-arm sibling, must stay green — the fix must not regress the sim path, which is unaffected since sim's `BackendConnector` doesn't route through Confluence's `list_issues_impl`)
- **Phase gate:** Full suite green + the real-backend t4 gate (`agent-ux/t4-conflict-rebase-ancestry-real-backend`) GREEN before `/gsd-verify-work`, per SC2's explicit acceptance criterion — this is one of the rare phases where a real-backend row is part of the PHASE'S OWN success criteria, not just the milestone-close 9th probe.

### Wave 0 Gaps
- [ ] `crates/reposix-confluence/tests/contract.rs` — add/extend a test asserting the LIST wiremock stub receives `body-format=atlas_doc_format` and that `list_records`'s body matches `get_record`'s body for an ADF-native fixture page — covers FIX-01
- [ ] `crates/reposix-cache/tests/oid_drift_reconcile.rs` (NEW) — `DriftingMock` `BackendConnector` (list/get body divergence, toggle-able like `pagination_prune_safety.rs`'s `CappingMock`) proving both the pre-fix `Error::OidDrift` reproduction AND the FIX-02 non-recovery-via-reconcile finding — covers FIX-01 regression lock + FIX-02
- [ ] No framework install needed — `wiremock`, `tokio::test`, `cargo nextest` are already wired for both crates

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Fix touches only the LIST request's query string; `standard_headers()` (Basic-auth) is unchanged |
| V3 Session Management | no | N/A — stateless REST calls |
| V4 Access Control | no | No authorization-boundary change |
| V5 Input Validation | no (unchanged) | The fix adds a STATIC, hardcoded query-param VALUE (`atlas_doc_format`) — not user/attacker-controlled input; no new validation surface |
| V6 Cryptography | no | N/A |

### Known Threat Patterns for this stack (Confluence adapter + cache builder)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Untrusted Confluence page body reaching the working tree unsanitized | Tampering / Info Disclosure | Already handled: `Tainted::new(page)` wraps EVERY page (list AND get) before `translate()` (SG-05) — the fix does not add a new ingestion point, it only changes WHICH representation is requested on an ALREADY-taint-wrapped call. No change needed to taint handling. |
| SSRF via `_links.next` cursor URL | Spoofing / Tampering | Already handled: `list_issues_impl` only prepends `self.base()` to a RELATIVE cursor path (client.rs:269-279); the SG-01 allowlist gate inside `HttpClient` re-checks every outbound URL regardless. The fix does not touch this logic. |
| Silently serving mismatched blob content (weakening the drift-check) | Tampering | This is exactly what Pitfall 1 above warns against — the fix must NOT touch `read_blob`'s `written_oid != oid` comparison; that check is a security-relevant coherence guard (a tree oid that doesn't match its blob would let a future backend response silently swap content post-commit). |

**No weakening of the `Tainted<T>`/`sanitize()` boundary is required or acceptable for
this fix.** The data flow through `list_issues_impl` → `translate()` → `Record` is
UNCHANGED by adding a query parameter; only the CONTENT of `page.body` (still wrapped in
`Tainted::new` before translation, per the existing SG-05 pattern) differs. Confirmed
against `docs/how-it-works/trust-model.md`'s SG-01/SG-05 rows — no cross-references need
updating.

## Sources

### Primary (HIGH confidence — verified against the actual codebase in this session)
- `crates/reposix-cache/src/builder.rs` (`build_from` lines 68-250, `read_blob` lines 573-634)
- `crates/reposix-confluence/src/client.rs` (`list_issues_impl` lines 189-284)
- `crates/reposix-confluence/src/lib.rs` (`get_record` lines 253-325)
- `crates/reposix-confluence/src/translate.rs` (`translate` lines 100-144)
- `crates/reposix-confluence/src/types.rs` (`ConfPage` doc lines 108-110)
- `crates/reposix-core/src/record.rs` (`frontmatter::render` lines 130-144)
- `crates/reposix-cache/src/error.rs` (`Error::OidDrift` lines 63-71)
- `crates/reposix-cli/src/sync.rs` (full file)
- `crates/reposix-cache/src/cache.rs` (`write_last_fetched_at` lines 575-592)
- `crates/reposix-cli/tests/agent_flow_real.rs` (lines 204-290)
- `crates/reposix-cache/tests/pagination_prune_safety.rs` (`CappingMock` pattern, lines 42-157)
- `crates/reposix-cache/tests/common/mod.rs`
- `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` + `quality/gates/agent-ux/lib/t4-real-backend-flow.sh`
- `docs/reference/testing-targets.md`
- `docs/how-it-works/trust-model.md`
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (2026-07-14 20:40, 20:41 entries — the original discovery)
- `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (GTH-V15-19)
- `.planning/MANAGER-HANDOVER.md` (spot-verification of the drift-check trace site)
- `.planning/REQUIREMENTS.md` (FIX-01/FIX-02 rows)

### Secondary (MEDIUM confidence — WebSearch, cross-verified against multiple results)
- [Confluence Cloud REST API v2 - Page API group](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) — general v2 API surface
- Atlassian Developer Community thread on `atlas_doc_format` representation:
  [Confluence REST API v2 — Create page with atlas_doc_format representation](https://community.developer.atlassian.com/t/confluence-rest-api-v2-create-page-with-atlas-doc-format-representation/67565)
- Jira ticket confirming `body-format=storage`/`atlas_doc_format` support (and the `view`
  LIMITATION) on the `Get pages` endpoint:
  [CONFCLOUD-79781 — Ability to specify `?body-format=view` for V2 "Get pages"](https://jira.atlassian.com/browse/CONFCLOUD-79781)
- General Atlassian pagination guidance (follow `_links.next`/`prev` as-is):
  [Pagination in the REST API](https://developer.atlassian.com/server/confluence/pagination-in-the-rest-api/),
  [What is the correct way to handle pagination in the Confluence REST API v2](https://community.developer.atlassian.com/t/what-is-the-correct-way-to-handle-pagination-in-the-confluence-rest-api-v2/86716)

### Tertiary (LOW confidence — none used; every claim above was either code-verified or WebSearch-cross-referenced)

## Project Constraints (from CLAUDE.md)

- **Enter through a GSD command.** This research is being consumed by
  `/gsd-plan-phase 114`; the planner must not hand-edit code outside the phase.
- **One cargo invocation machine-wide.** All test/build commands in this research
  (Validation Architecture section) assume the executor serializes cargo calls and
  prefers `-p <crate>` over `--workspace` during iteration.
- **Leaf isolation for any repro.** Any `reposix init`/`git checkout`/`git commit`
  reproduction MUST run in a throwaway `/tmp` clone, `cd`-ing in the SAME bash
  invocation — never the shared repo. The read-only repro recipe above already follows
  this.
- **Tainted by default (OP-2).** Confirmed above: the fix does not introduce any new
  egress path or weaken `Tainted<T>`/`sanitize()`.
- **Audit log non-optional (OP-3).** No change to audit-emission points — `list_records`
  and `get_record` calls are not currently audited as backend-mutation events (they're
  reads), and this fix doesn't change that; unaffected.
- **Uncommitted = didn't happen.** The plan must commit the fix + tests before any
  relief/handover boundary.
- **Ownership charter — error messages teach the fix.** Noted but OUT OF SCOPE for
  FIX-01/FIX-02: `Error::OidDrift`'s `Display` impl (`"oid drift: requested {requested},
  backend returned {actual} for issue {issue_id}"`) does not currently suggest a
  recovery command (e.g. pointing at `reposix sync --reconcile` where applicable, or
  explaining WHY). This is adjacent noticing, not required by this phase's acceptance
  criteria — worth filing to `GOOD-TO-HAVES.md` if not eagerly fixed in the same phase
  (OP-8: <1h + no new dependency → fix in place is plausible here since it's a single
  `#[error(...)]` string edit).
- **Push cadence.** Phase close requires `git push origin main` BEFORE the verifier
  subagent, and main's LATEST CI run GREEN afterward — plus, uniquely for this phase,
  the SC2 real-backend gate itself GREEN (not just CI).

## Metadata

**Confidence breakdown:**
- Root cause + fix locus: HIGH — every claim traced to an exact file:line in the current
  codebase, cross-corroborated by 3 independent pre-existing doc comments/tests already
  in the repo (not introduced by this research).
- Reconcile recovery scope (FIX-02): HIGH — derived by reading `sync.rs::run`'s actual
  call graph (`build_from()`, not a separate reconcile implementation), not assumed.
- Atlassian API `body-format` support on the LIST endpoint: MEDIUM — WebSearch
  cross-verified against 2+ independent sources (official docs page + a specific Jira
  ticket naming the exact limitation), but not confirmed via a live API call in this
  session (constraint: do not hit real backends).
- Pre-ADF residual-gap risk to TokenWorld specifically: LOW/ASSUMED — flagged honestly
  in the Assumptions Log, not blocking for SC1/SC2 reasoning but the executor should
  verify empirically via the real t4 gate re-run.

**Research date:** 2026-07-14
**Valid until:** 14 days (real-backend API behavior + TokenWorld fixture state can shift;
re-verify body-format pagination-preservation and TokenWorld's page inventory if this
research is reused past that window)
