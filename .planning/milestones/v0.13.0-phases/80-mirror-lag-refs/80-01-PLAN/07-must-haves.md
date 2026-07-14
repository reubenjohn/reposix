← [back to index](./index.md) · phase 80 plan 01


<must_haves>
- New file `crates/reposix-cache/src/mirror_refs.rs` (≤ 250 lines).
  Mirrors `crates/reposix-cache/src/sync_tag.rs` shape verbatim where
  possible — copy-and-adapt, not from-scratch design.
- Three new public Cache APIs in `mirror_refs.rs` (each with `# Errors`
  doc + clippy::pedantic clean):
  - `pub fn write_mirror_head(&self, sot_host: &str, sot_sha: gix::ObjectId) -> Result<String>`
    — direct ref at `refs/mirrors/<sot_host>-head` pointing at `sot_sha`.
    Returns the full ref name. Uses `RefEdit` + `PreviousValue::Any` +
    `Target::Object(sot_sha)` (verbatim sync_tag.rs Pattern 1 shape).
  - `pub fn write_mirror_synced_at(&self, sot_host: &str, ts: chrono::DateTime<chrono::Utc>) -> Result<String>`
    — annotated tag at `refs/mirrors/<sot_host>-synced-at`. Tag message
    body's first line is `mirror synced at <RFC3339>` (RFC3339 with
    second precision + trailing `Z`). Returns the full ref name.
    Implementation uses gix `Repository::tag(...)` if available at the
    workspace pin; else falls back to two `RefEdit`s (RESEARCH.md A1
    fallback path).
  - `pub fn read_mirror_synced_at(&self, sot_host: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>>`
    — resolves the tag; reads message body's first line; parses
    RFC3339; returns `Some(ts)` on success, `None` if the ref is
    absent (first-push case) OR if the message body fails to parse
    (defensive — log WARN, return None rather than poison the reject
    path).
- Ref name validation via `gix::refs::FullName::try_from(&str)` — per
  RESEARCH.md "Don't Hand-Roll" row 1. NO regex, no manual byte checks.
  `sot_host` slug from `state.backend_name` enum (controlled input).
- Module-doc on `mirror_refs.rs`:
  - Documents the cache-as-source-of-truth contract (refs live in the
    cache's bare repo, NOT the working tree).
  - Documents the annotated-tag message-body shape (`mirror synced at
    <RFC3339>` — single human-readable line; future structured fields
    ride additional `key: value` lines without breaking the first-line
    contract).
  - One-line note re reflog growth on long-lived caches as a v0.14.0
    operational concern (RESEARCH.md pitfall 6 deferral target).
  - Cross-reference to `sync_tag.rs` as the donor pattern.
- New `audit::log_mirror_sync_written` in `crates/reposix-cache/src/audit.rs`
  mirroring `log_sync_tag_written` (lines 340-363):
  - Op string: `mirror_sync_written`.
  - Columns: `ts`, `op`, `backend`, `project`, `oid` (the `sot_sha`
    written), `reason` (the ref-name pair: `refs/mirrors/<sot>-head,refs/mirrors/<sot>-synced-at`).
  - Best-effort SQL semantics; SQL errors WARN-log via
    `tracing::warn!`. Function returns `()`.
- Re-exports in `crates/reposix-cache/src/lib.rs`:
  ```rust
  pub mod mirror_refs;
  pub use mirror_refs::{
      MIRROR_REFS_HEAD_PREFIX, MIRROR_REFS_SYNCED_AT_PREFIX,
      format_mirror_head_ref_name, format_mirror_synced_at_ref_name,
  };
  ```
  Public namespace constants + name formatters, mirroring `sync_tag.rs`'s
  `SYNC_TAG_PREFIX` re-export precedent.
- 4 unit tests in `mirror_refs.rs` `#[cfg(test)] mod tests`:
  1. `write_mirror_head_round_trips` — write + read back via
     `repo.find_reference(...).peel_to_id()`; oid matches.
  2. `write_mirror_synced_at_round_trips` — write at `ts1`, read,
     assert `ts1` parses back from message body (DateTime<Utc> equality).
  3. `read_mirror_synced_at_returns_none_when_absent` — fresh tempdir
     cache; `read_mirror_synced_at("sim")` returns `Ok(None)`.
  4. `mirror_ref_names_validate_via_gix` — assert
     `gix::refs::FullName::try_from(format_mirror_head_ref_name("sim"))`
     succeeds (positive); assert it FAILS for an invalid sot_host
     (e.g., `"foo:bar"` — colons forbidden in ref names). Negative
     test pins the validation contract to gix's enforcement, not our
     own logic.
- Helper wiring in `crates/reposix-remote/src/main.rs::handle_export`:
  - Insert at the SUCCESS branch (current line 470, immediately AFTER
    `cache.log_helper_push_accepted(...)` and BEFORE the existing
    `log_token_cost` call):
    ```rust
    // Mirror-lag refs (DVCS-MIRROR-REFS-02). Best-effort: a ref-write
    // failure WARN-logs and does not poison the push ack. The audit
    // row is UNCONDITIONAL per OP-3 — written even on ref-write
    // failure. SoT SHA is the cache's post-write synthesis-commit
    // OID (build_from runs again to capture the post-write tree;
    // P80 cost is one extra REST list_records — P81 L1 replaces this
    // with list_changed_since per architecture-sketch.md § Performance
    // subtlety).
    let sot_sha_opt = match state.rt.block_on(cache.refresh_for_mirror_head()) {
        Ok(oid) => Some(oid),
        Err(e) => {
            tracing::warn!("mirror-head SHA derivation failed: {e:#}");
            None
        }
    };
    let now = chrono::Utc::now();
    let head_result = sot_sha_opt
        .as_ref()
        .map(|sha| cache.write_mirror_head(&state.backend_name, *sha));
    if let Some(Err(e)) = &head_result {
        tracing::warn!("write_mirror_head failed: {e:#}");
    }
    if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, now) {
        tracing::warn!("write_mirror_synced_at failed: {e:#}");
    }
    cache.log_mirror_sync_written(
        sot_sha_opt.as_ref().map(gix::ObjectId::to_hex_with_len).unwrap_or_default().to_string().as_str(),
        &state.backend_name,
    );
    ```
  - The `cache.refresh_for_mirror_head()` accessor is a thin wrapper
    on `Cache::build_from()` introduced in T02 to give the helper a
    stable, named accessor without exposing `build_from` directly to
    the wiring site. Implementation: `pub async fn refresh_for_mirror_head(&self) -> Result<gix::ObjectId> { self.build_from().await }`.
    Rationale: the helper today does NOT call `build_from` itself
    (the cache lifecycle is managed elsewhere); naming this accessor
    makes the call site grep-discoverable and the cost commentary
    targeted.
  - The reject paths (current lines 384-407 conflict, 411-432 plan
    errors) gain the new hint composition. T03 implementation detail
    below.
  - **Existing comment at line 472-485 (token-cost block) is
    preserved verbatim.**
- Helper reject-path hint composition: the conflict-reject branch
  (lines 384-407) AFTER the existing `diag(&format!(...))` call and
  BEFORE the `proto.send_line("error refs/heads/main fetch first")`
  call, emits ADDITIONAL stderr lines:
  ```rust
  if let Some(cache) = state.cache.as_ref() {
      if let Ok(Some(synced_at)) = cache.read_mirror_synced_at(&state.backend_name) {
          let ago = chrono::Utc::now().signed_duration_since(synced_at);
          let mins = ago.num_minutes().max(0);
          diag(&format!(
              "hint: your origin (GH mirror) was last synced from {sot} at {ts} ({mins} minutes ago)",
              sot = state.backend_name,
              ts = synced_at.to_rfc3339(),
              mins = mins,
          ));
          diag(&format!(
              "hint: run `reposix sync` to update local cache from {sot} directly, then `git rebase`",
              sot = state.backend_name,
          ));
      }
      // None case: omit the hint lines cleanly (RESEARCH.md pitfall 7).
  }
  ```
- Helper `stateless-connect` advertisement widened to include
  `refs/mirrors/*`. Investigation BEFORE wiring (T03 read_first):
  read `crates/reposix-remote/src/stateless_connect.rs` to find where
  `ls-refs` advertisement composes the visible-ref set; widen the
  filter to include `refs/mirrors/*` alongside `refs/heads/*`. If
  the helper tunnels protocol-v2 fetch traffic through `git
  upload-pack --stateless-rpc` against the cache's bare repo, the
  cache's bare repo's refs ARE the advertisement — no helper-side
  filtering needed; vanilla `git fetch` will pick them up
  automatically. T03 verifies this by reading the file BEFORE adding
  any filter logic. RESEARCH.md A3 marked this as a one-line edit;
  if it turns out to be zero-line (already correct), document the
  finding in T03's commit message and proceed.
- 3 verifier shells in `quality/gates/agent-ux/` (each ≤ 60 lines;
  TINY shape mirroring `quality/gates/agent-ux/reposix-attach.sh`):
  - `mirror-refs-write-on-success.sh` — start sim, init working tree,
    seed sim with one record, run a `reposix init` + edit + `git push`
    cycle, assert `git -C <cache-bare-repo> for-each-ref refs/mirrors/`
    returns BOTH refs and `git -C <cache> log refs/mirrors/sim-synced-at -1 --format=%B`
    matches `mirror synced at <RFC3339>`.
  - `mirror-refs-readable-by-vanilla-fetch.sh` — same setup as #1; then
    a fresh `git clone` of the cache's bare repo (or `git fetch` from
    an existing clone via the helper), assert `git for-each-ref
    refs/mirrors/` from the new clone returns both refs.
  - `mirror-refs-cited-in-reject-hint.sh` — successful push first
    (refs populated), then a SECOND push with a stale prior, assert
    the reject stderr contains both `refs/mirrors/sim-synced-at` AND
    a parseable RFC3339 timestamp + `(N minutes ago)`.
- 3 catalog rows in `quality/catalogs/agent-ux.json` (T01) — joined
  to the existing `rows` array. Each row uses the exact P79 row's
  shape (verified at planning time via `quality/catalogs/agent-ux.json`):
  - `agent-ux/mirror-refs-write-on-success` — kind: mechanical, cadence:
    pre-pr, verifier: `quality/gates/agent-ux/mirror-refs-write-on-success.sh`,
    initial status: FAIL.
  - `agent-ux/mirror-refs-readable-by-vanilla-fetch` — kind: mechanical,
    cadence: pre-pr, verifier:
    `quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh`,
    initial status: FAIL.
  - `agent-ux/mirror-refs-cited-in-reject-hint` — kind: mechanical,
    cadence: pre-pr, verifier:
    `quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh`,
    initial status: FAIL.
  Each row's `_provenance_note` annotates "Hand-edit per documented
  gap (NOT Principle A): see GOOD-TO-HAVES-01" — verbatim from the P79
  precedent.
- 4 integration tests in `crates/reposix-remote/tests/mirror_refs.rs`
  (T04):
  1. `write_on_success_updates_both_refs` — after a successful single-
     backend push, BOTH `refs/mirrors/sim-head` and
     `refs/mirrors/sim-synced-at` are resolvable in the cache's bare
     repo; tag message body matches `mirror synced at <RFC3339>`;
     `audit_events_cache` has a row with `op = 'mirror_sync_written'`.
  2. `vanilla_fetch_brings_mirror_refs` — after a successful push, a
     fresh `git clone --bare` of the cache repo (or `git fetch` from
     an existing clone) brings both refs along; `git for-each-ref
     refs/mirrors/` from the new clone returns both refs.
  3. `reject_hint_cites_synced_at_with_age` — after a successful push
     (refs populated), a second push with a stale prior triggers the
     conflict-reject path; stderr contains both
     `refs/mirrors/sim-synced-at` AND a parseable RFC3339 timestamp +
     `(N minutes ago)`.
  4. `reject_hint_first_push_omits_synced_at_line` — first-ever push
     with a stale prior (no refs yet); reject stderr does NOT contain
     a "synced at" hint line (cleanly omitted per RESEARCH.md pitfall 7).
- The catalog rows' `status` field flips from `FAIL` to `PASS` via the
  runner during T04 BEFORE the per-phase push commits.
- CLAUDE.md updated in T04: § Architecture (or the closest section to
  the existing audit-log description) gains a paragraph documenting
  the `refs/mirrors/<sot-host>-{head,synced-at}` namespace convention
  + the Q2.2 staleness-window doc-clarity carrier (one-line — full
  docs treatment defers to P85).
- Plan terminates with `git push origin main` (per CLAUDE.md push
  cadence) with pre-push GREEN.
- All cargo invocations in this plan are SERIAL (one at a time per
  CLAUDE.md Build memory budget).
- NO new error variants on `RemoteError` or `cache::Error`. Ref-write
  failures are best-effort `tracing::warn!`; audit-write failures
  follow the existing `log_*` family WARN-log pattern.
- **Q2.2 verbatim phrase carrier (disambiguation).** The Q2.2 verbatim
  phrase ("`refs/mirrors/<sot>-synced-at` is the timestamp the mirror
  last caught up to `<sot>` — NOT a current SoT state marker") lives
  in **CLAUDE.md** (T04 epilogue) per the doc-clarity contract; the
  reject stderr cites the ref name + age rendering only. Decision
  rationale: `decisions.md` Q2.2 names `docs/concepts` and `docs/guides`
  as the verbatim-phrase targets, NOT stderr. Full docs treatment
  defers to P85.
