← [back to index](./index.md) · phase 31 plan 02

# Task 2: Lazy Blob Materialization

Implement `Cache::read_blob` with Tainted return, egress-denial audit, and materialize-row test.

## Files

- `crates/reposix-cache/src/builder.rs`
- `crates/reposix-cache/src/error.rs` (add `Error::Egress`, `Error::OidDrift` variants)
- `crates/reposix-cache/tests/materialize_one.rs`
- `crates/reposix-cache/tests/egress_denied_logs.rs`

## Read first

- `crates/reposix-cache/src/builder.rs` — the `Cache` type and `build_from` method (Plan 01 output).
- `crates/reposix-cache/src/cache.rs` — helper types.
- `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md` § Pitfall 1 (OID drift detection).

## Behavior

The new `read_blob` method materializes one blob from the backend and returns it wrapped in `Tainted<Vec<u8>>`. It audits every materialization, detects egress denial and OID mismatches, and returns typed errors for each case.

**Signature:**
```rust
pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>
```

**Steps:**

1. Call `self.backend.get_record(self.project, issue_id_for_oid)` to fetch the latest record from the backend.
2. Render the record to markdown (via `crate::render::frontmatter` or similar — reuse Plan 01's rendering pipeline).
3. Write the rendered bytes to a git blob object via `self.git_repo.write_blob(&bytes)`.
4. Assert the returned OID matches the requested OID; if not, return `Error::OidDrift { requested, actual, issue_id }`.
5. On success, call `log_materialize(...)` to audit.
6. Return `Ok(Tainted::new(bytes))`.

**Error handling:**

- `reposix_core::Error::InvalidOrigin(origin)` detected during step 1 → call `log_egress_denied(origin, self.backend.name())`, return `Error::Egress(origin)`.
- Substring match as a fallback: if the error message contains `"invalid origin"` or `"allowlist"`, treat it as egress denial (for wrapped errors from other backends).
- `write_blob` OID mismatch → `Error::OidDrift`.
- Other backend errors → convert to `Error::Backend(...)` as usual.

**Typed errors:**

Add three variants to `crates/reposix-cache/src/error.rs`:
```rust
#[error("Egress denied for origin: {0}")]
Egress(String),

#[error("OID drift: requested {requested}, got {actual} for issue {issue_id}")]
OidDrift { requested: String, actual: String, issue_id: String },

#[error("Backend error: {0}")]
Backend(String),
```

## Action

**Step 1 — Add error variants to `crates/reposix-cache/src/error.rs`:**

```rust
#[error("Egress denied for origin: {0}")]
Egress(String),

#[error("OID drift: requested {requested}, got {actual} for issue {issue_id}")]
OidDrift {
    requested: String,
    actual: String,
    issue_id: String,
},
```

**Step 2 — Implement `read_blob` in `crates/reposix-cache/src/builder.rs`:**

```rust
pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>> {
    // Lookup which issue_id corresponds to this OID from oid_map.
    let (issue_id, _backend_name) = meta::get_issue_for_oid(&self.db, oid)
        .context("oid_map lookup failed")?
        .ok_or_else(|| Error::OidNotFound(oid.to_string()))?;
    
    // Fetch the record from the backend.
    let record = match self.backend.get_record(&self.project, &issue_id).await {
        Ok(r) => r,
        Err(reposix_core::Error::InvalidOrigin(origin)) => {
            // Egress denied — audit and return typed error.
            audit::log_egress_denied(&self.db, &origin, self.backend.name())?;
            return Err(Error::Egress(origin));
        }
        Err(e) => {
            // Check for substring match as a fallback (for wrapped errors).
            let msg = e.to_string();
            if msg.contains("invalid origin") || msg.contains("allowlist") {
                audit::log_egress_denied(&self.db, &msg, self.backend.name())?;
                return Err(Error::Egress(msg));
            }
            return Err(Error::Backend(msg));
        }
    };
    
    // Render to markdown.
    let body = crate::render::frontmatter(&record, &self.project)?;
    
    // Write blob.
    let actual_oid = self.git_repo.write_blob(&body)
        .context("write_blob failed")?;
    
    // Check OID match.
    if actual_oid != oid {
        return Err(Error::OidDrift {
            requested: oid.to_string(),
            actual: actual_oid.to_string(),
            issue_id,
        });
    }
    
    // Audit the materialization.
    audit::log_materialize(&self.db, oid, &issue_id, self.backend.name())?;
    
    // Return tainted bytes.
    Ok(Tainted::new(body))
}
```

**Step 3 — Create `crates/reposix-cache/tests/materialize_one.rs`:**

```rust
#[tokio::test]
async fn materialize_one_blob_writes_audit_row() {
    // Setup: fixture with simulator backend, one record in the sim.
    let sim = reposix_sim::Simulator::new();
    let (cache, oid) = setup_cache_with_one_record(&sim).await;
    
    // Before: no audit rows.
    let initial_count = query_audit_count(&cache, "materialize").unwrap();
    assert_eq!(initial_count, 0);
    
    // Call read_blob.
    let result = cache.read_blob(oid).await;
    assert!(result.is_ok());
    let tainted = result.unwrap();
    
    // Verify Tainted wrapper is present (not just Vec<u8>).
    assert!(!tainted.inner_ref().is_empty());
    
    // After: exactly one materialize row.
    let final_count = query_audit_count(&cache, "materialize").unwrap();
    assert_eq!(final_count, 1);
}
```

**Step 4 — Create `crates/reposix-cache/tests/egress_denied_logs.rs`:**

```rust
#[tokio::test]
async fn egress_denied_returns_error_and_logs_audit() {
    // Setup: cache pointed at non-allowlisted origin (simulator outside allowlist).
    let cache = setup_cache_non_allowlisted().await;
    
    // Call read_blob against the bad origin.
    let result = cache.read_blob(some_oid).await;
    
    // Verify typed error.
    assert!(matches!(result, Err(reposix_cache::error::Error::Egress(_))));
    
    // Verify audit row was written before the error returned.
    let egress_count = query_audit_count(&cache, "egress_denied").unwrap();
    assert_eq!(egress_count, 1);
}
```

**Step 5 — Verify no `reqwest::Client` constructors:**

```bash
grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new|Client::)" crates/reposix-cache/src/
# Should return empty (no matches).
```

If any match is found, remove it; the cache must use only `BackendConnector` methods which route HTTP through `reposix_core::http::client()`.

**Step 6 — Run:**

```bash
cargo test -p reposix-cache
cargo clippy -p reposix-cache --all-targets -- -D warnings
```

## Acceptance criteria

- `grep -qE "pub async fn read_blob\s*\(\s*&self,\s*oid: gix::ObjectId\s*\)\s*->\s*Result<Tainted<Vec<u8>>>" crates/reposix-cache/src/builder.rs` returns 0 (modulo formatting; the literal `Tainted<Vec<u8>>` substring must be present in the signature region).
- `grep -q "log_materialize" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "log_egress_denied" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "Tainted::new" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "OidDrift" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/` returns empty (no matches — enforced).
- `cargo test -p reposix-cache --test materialize_one` exits 0.
- `cargo test -p reposix-cache --test egress_denied_logs` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

## Verify

Automated:
```
cargo test -p reposix-cache && cargo clippy -p reposix-cache --all-targets -- -D warnings && ! grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/
```

## Done

`Cache::read_blob` materializes exactly one blob per call, audits it, and wraps the result in `Tainted<Vec<u8>>`. Egress denial is detected, audited with `op='egress_denied'`, and returned as `Error::Egress`. Zero `reqwest::Client` constructors in the new crate.
