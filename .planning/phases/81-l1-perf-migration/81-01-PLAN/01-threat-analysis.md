← [back to index](./index.md) · phase 81 plan 01

## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| helper → cache.db (`meta` table) | The helper writes the `last_fetched_at` cursor via `Cache::write_last_fetched_at` → `meta::set_meta` (parameterized SQL). Trust direction: helper-controlled byte sources (`chrono::Utc::now().to_rfc3339()`) flow into the cursor value. No untrusted input from SoT propagates to cursor content. |
| helper → SoT (`list_changed_since` REST call) | UNCHANGED from existing `list_records` call site — same `BackendConnector` trait, same `client()` factory, same `REPOSIX_ALLOWED_ORIGINS` allowlist. The `since` parameter is a controlled timestamp (helper-generated `Utc::now()` written by THIS helper). |
| cache prior-blob parse (Tainted bytes) | The precheck reads `cache.read_blob_cached(prior_oid)` (NEW sync gix-only primitive — H1 fix; returns `Ok(Some(Tainted<Vec<u8>>))` or `Ok(None)` on cache miss). Parsing tainted bytes is fine (no I/O side effects); care is needed not to leak the body into a log line. The existing `log_helper_push_rejected_conflict` shape already enforces "id + versions only" — preserved verbatim. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-81-01 | Tampering | Cursor write (`Cache::write_last_fetched_at` → `meta::set_meta`) | mitigate | `meta::set_meta` is parameterized SQL (`?1, ?2, ?3` placeholders; `crates/reposix-cache/src/meta.rs:13-19`) — no string concatenation, no SQL injection. Cursor value is `chrono::Utc::now().to_rfc3339()` — helper-generated, never user-controlled. Unit test `read_last_fetched_at_round_trips` (T02) pins the contract. |
| T-81-02 | Information Disclosure | Tainted prior-blob bytes leaking into log lines via `log_helper_push_rejected_conflict` | mitigate | The existing `log_helper_push_rejected_conflict` shape (`crates/reposix-cache/src/cache.rs:256-275`) already records `id + local_v + backend_v` ONLY — no body bytes. T02's precheck preserves this contract: `Tainted::inner_ref()` is used only for `frontmatter::parse(...)` to extract the `version` field (a `u64`); the parsed struct's body field is never logged. Code review checkpoint: T02's PR diff is grepped for `tracing::*!.*body` patterns BEFORE merge. |
| T-81-03 | Denial of Service | Clock-skew false-positive on own-write race after cursor tick | accept | RESEARCH.md § Pitfall 2: agent's clock 30s behind backend's clock causes the next push to see its own just-written record in `list_changed_since(T0)`'s output (because backend-time is `T0 + 30s > T0`). The precheck's `find_oid_for_record` against the just-synced cache will report version-equal (we just wrote our own version into the cache via the post-write step), so the precheck passes. Self-healing on next tick. T02 inline comment names this as a known L1 quirk; v0.14.0 OTel work measures incidence. |

No new HTTP origin in scope; no new Tainted<T> propagation path; no new
shell-out from the cache or helper; no new sanitization branch.
</threat_model>

---

