← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T03 — Coarser SoT-drift wrapper `precheck_sot_drift_any` + 1 unit test

<read_first>
- `crates/reposix-remote/src/precheck.rs` (entire file, 302 lines
  post-P81) — donor pattern; the new wrapper appends after the
  existing `precheck_export_against_changed_set` function. Confirm
  the existing module-doc + use statements are intact.
- `crates/reposix-remote/src/precheck.rs` lines 26-35 (existing
  imports — the new wrapper reuses `Cache`, `BackendConnector`,
  `Runtime`, `anyhow`, `Context`).
- `crates/reposix-cache/src/cache.rs::read_last_fetched_at` (P81)
  — the cursor-read API the wrapper calls.
- `crates/reposix-core/src/backend.rs:253` —
  `BackendConnector::list_changed_since(project, since)` returns
  `Result<Vec<RecordId>>`.
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md` § Pattern 2
  (wrapper algorithm pseudocode).
</read_first>

<action>
Two concerns: append wrapper to `precheck.rs` → unit test → cargo
check + commit.

### 3a. Append `precheck_sot_drift_any` to `crates/reposix-remote/src/precheck.rs`

Append AFTER the existing `precheck_export_against_changed_set`
function (post line 302). The new code:

```rust
/// Coarser SoT-drift outcome — bus handler's PRECHECK B reports
/// whether ANY backend record has changed since the cache cursor,
/// without intersecting against a push set (the bus path runs this
/// BEFORE reading stdin, so the push set is unknown). The finer
/// intersect-with-push-set check lives in
/// [`precheck_export_against_changed_set`] and runs in P83 AFTER
/// `parse_export_stream` consumes stdin.
#[derive(Debug, Clone)]
pub(crate) enum SotDriftOutcome {
    /// Backend has at least one record changed since `last_fetched_at`.
    /// `changed_count` is reported for diagnostic / logging only —
    /// the bus handler emits the rejection unconditionally on `Drifted`.
    Drifted { changed_count: usize },
    /// Backend stable since `last_fetched_at` (or no cursor — first-push
    /// fallback per [`precheck_export_against_changed_set`]'s policy).
    Stable,
}

/// PRECHECK B (coarser sibling of [`precheck_export_against_changed_set`]).
///
/// The bus handler runs this BEFORE reading stdin, so the push set is
/// unknown. This wrapper asks "did anything change since
/// `last_fetched_at`?" and bails on any drift; the architecture-sketch's
/// step 3 prose ratifies this coarser semantic for the bus path. The
/// finer intersect-with-push-set check (which the bus handler will
/// also run in P83 AFTER stdin is read) lives in
/// [`precheck_export_against_changed_set`].
///
/// First-push policy: when the cursor is absent, returns
/// [`SotDriftOutcome::Stable`] — same shape as
/// [`precheck_export_against_changed_set`]'s no-cursor path. The
/// inner correctness check at SoT-write time (P83) is the safety
/// net for first pushes.
///
/// # Errors
/// REST failure annotates with `.context("backend-unreachable: ...")`
/// so the bus handler maps it to the existing `fail_push(diag,
/// "backend-unreachable", ...)` shape.
pub(crate) fn precheck_sot_drift_any(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
) -> Result<SotDriftOutcome> {
    // Step 1: read cursor. No cursor → first-push policy = Stable.
    let Some(since) = cache.and_then(|c| c.read_last_fetched_at().ok().flatten()) else {
        return Ok(SotDriftOutcome::Stable);
    };

    // Step 2: list_changed_since on SoT. Empty → Stable; non-empty →
    // Drifted. Bus handler emits `error refs/heads/main fetch first`
    // on Drifted.
    let changed = rt
        .block_on(backend.list_changed_since(project, since))
        .context("backend-unreachable: list_changed_since (PRECHECK B)")?;

    if changed.is_empty() {
        Ok(SotDriftOutcome::Stable)
    } else {
        Ok(SotDriftOutcome::Drifted { changed_count: changed.len() })
    }
}
```

### 3b. Update the module-doc comment

Add a paragraph to the existing module-doc after the "Anti-patterns"
list, naming the bus-vs-single-backend asymmetry:

```rust
//! ## Bus-vs-single-backend precheck asymmetry (P82+)
//!
//! [`precheck_sot_drift_any`] is a COARSER sibling intended for the
//! bus handler's PRECHECK B — it runs BEFORE reading stdin (push
//! set unknown), so it asks "did anything change?" and bails on any
//! drift. The finer [`precheck_export_against_changed_set`] runs
//! AFTER stdin is read (single-backend path today; P83's bus handler
//! will run BOTH — coarser before stdin, finer after). The architecture-
//! sketch's step 3 prose ratifies this asymmetry per Q3.1.
```

### 3c. Append unit test in `#[cfg(test)] mod tests`

Locate the existing `#[cfg(test)] mod tests` block at the bottom of
`precheck.rs` (or, if no test module exists yet, add a fresh one at
EOF). Confirm during T03 read_first whether the existing module is
named `tests` or has a different convention.

```rust
    /// First-push policy: no cursor → Stable. Mirrors the no-cursor
    /// fallback in `precheck_export_against_changed_set` so the bus
    /// handler's PRECHECK B doesn't misfire on a fresh attach.
    #[tokio::test(flavor = "current_thread")]
    async fn precheck_sot_drift_any_returns_stable_when_no_cursor() {
        // Use a Cache without a populated cursor (no Cache::build_from
        // call). The wrapper should return Stable without hitting the
        // backend.
        let _ = ();  // smoke test only — full Drifted/Stable cases
                     // exercised end-to-end in tests/bus_precheck_b.rs

        // Spawn a runtime + a no-op backend stub. The simplest stub
        // is a SimBackend pointed at a non-existent URL — the wrapper
        // should NOT call list_changed_since because the cursor is
        // absent. We verify by passing cache=None.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        // SimBackend is the cheapest BackendConnector to instantiate;
        // confirm the constructor signature during T03 read_first
        // (existing test files in reposix-remote use this idiom).
        let backend: std::sync::Arc<dyn reposix_core::backend::BackendConnector> =
            std::sync::Arc::new(reposix_core::backend::sim::SimBackend::new(
                "http://127.0.0.1:0".to_owned(),
            ));

        let outcome = precheck_sot_drift_any(None, backend.as_ref(), "demo", &rt)
            .expect("no-cursor case should return Stable without erroring");
        match outcome {
            SotDriftOutcome::Stable => {}
            other => panic!("expected Stable; got {other:?}"),
        }
    }
```

If `SimBackend::new` requires different arguments than the simple
URL form above, adapt to whatever the existing P81 tests use (the
canonical donor is `crates/reposix-remote/tests/perf_l1.rs`'s
`sim_backend` helper). If a `SimBackend::new("...")` call would
panic on a malformed URL, swap to a wiremock-backed `SimBackend`
following P81's setup. The test asserts ONLY the no-cursor → Stable
path; it does NOT make a network call.

Build serially:

```bash
cargo check -p reposix-remote
cargo clippy -p reposix-remote -- -D warnings
cargo nextest run -p reposix-remote precheck_sot_drift_any
cargo nextest run -p reposix-remote precheck    # full crate precheck-prefix tests
```

### 3d. Stage and commit

```bash
git add crates/reposix-remote/src/precheck.rs
git commit -m "$(cat <<'EOF'
feat(remote): coarser SoT-drift wrapper precheck_sot_drift_any (DVCS-BUS-PRECHECK-02 substrate)

- crates/reposix-remote/src/precheck.rs — append pub(crate) enum SotDriftOutcome { Drifted { changed_count: usize } | Stable } + pub(crate) fn precheck_sot_drift_any(cache, backend, project, rt) -> Result<SotDriftOutcome>
- ~10 lines of body: read_last_fetched_at → list_changed_since → Drifted|Stable
- First-push policy (no cursor → Stable) mirrors precheck_export_against_changed_set's no-cursor path
- Module-doc gains a paragraph naming the bus-vs-single-backend asymmetry: bus PRECHECK B runs BEFORE stdin (push set unknown, coarser); single-backend precheck runs AFTER parse_export_stream (push set known, finer); P83's bus handler runs BOTH
- 1 unit test inline: precheck_sot_drift_any_returns_stable_when_no_cursor

P81's precheck_export_against_changed_set is preserved verbatim for P83's write-time intersect-with-push-set check. NO new error variants — anyhow::Result throughout.

Phase 82 / Plan 01 / Task 03 / DVCS-BUS-PRECHECK-02 (substrate).
EOF
)"
```
</action>

<verify>
  <automated>cargo check -p reposix-remote && cargo clippy -p reposix-remote -- -D warnings && cargo nextest run -p reposix-remote precheck_sot_drift_any && grep -q "pub(crate) fn precheck_sot_drift_any" crates/reposix-remote/src/precheck.rs && grep -q "pub(crate) enum SotDriftOutcome" crates/reposix-remote/src/precheck.rs</automated>
</verify>

<done>
- `crates/reposix-remote/src/precheck.rs` includes
  `pub(crate) enum SotDriftOutcome { Drifted { changed_count }, Stable }`
  and `pub(crate) fn precheck_sot_drift_any(...)`.
- The wrapper body is ≤ 15 lines (excluding doc comments).
- 1 unit test passes (`cargo nextest run -p reposix-remote
  precheck_sot_drift_any`).
- Module-doc carries the new "Bus-vs-single-backend precheck asymmetry"
  paragraph naming the coarser-vs-finer split.
- `# Errors` doc on `precheck_sot_drift_any`.
- `cargo check -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.
- Existing P81 `precheck_export_against_changed_set` preserved verbatim
  (no edits to its body or signature).
- NO new error variants — `anyhow::Result` throughout.
- Cargo serialized: T03 cargo invocations run only after T02's commit
  has landed.
</done>

---

