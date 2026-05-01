//! Catalog (de)serialization + atomic file ops for `quality/catalogs/doc-alignment.json`.
//!
//! Schema spec: `quality/catalogs/README.md` § "docs-alignment dimension".
//! Empty-state shape: `quality/catalogs/doc-alignment.json` (shipped by P64 Wave 1).

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Top-level catalog document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// JSON-schema URL (preserved verbatim if present).
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,

    /// Human-readable comment (preserved verbatim if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Dimension name. For doc-alignment.json this is `"docs-alignment"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,

    /// Catalog schema version. Currently `"1.0"`.
    pub schema_version: String,

    /// Aggregate summary block (recomputed on every walk).
    pub summary: Summary,

    /// Catalog rows.
    #[serde(default)]
    pub rows: Vec<Row>,
}

/// Aggregate summary block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub claims_total: u64,
    pub claims_bound: u64,
    pub claims_missing_test: u64,
    pub claims_retire_proposed: u64,
    pub claims_retired: u64,
    pub alignment_ratio: f64,
    pub floor: f64,
    pub trend_30d: String,
    pub last_walked: Option<String>,

    /// Optional time-bounded floor waiver (rare; documented in the schema spec).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor_waiver: Option<FloorWaiver>,

    /// Coverage metric (P66 v0.12.1) -- the second axis alongside `alignment_ratio`.
    /// `coverage_ratio = lines_covered / total_eligible_lines`. 0.0 when total==0.
    #[serde(default)]
    pub coverage_ratio: f64,

    /// Lines covered by at least one row's source citation (after range merge).
    #[serde(default)]
    pub lines_covered: u64,

    /// Total eligible lines across the prose universe (`docs/**/*.md` +
    /// `README.md` + archived REQUIREMENTS).
    #[serde(default)]
    pub total_eligible_lines: u64,

    /// Floor for the `coverage_ratio < coverage_floor` walker BLOCK. Default
    /// 0.10 (low ratchet baseline; bumped via deliberate human commits as
    /// gap-closure phases widen extraction).
    #[serde(default = "default_coverage_floor")]
    pub coverage_floor: f64,
}

/// Default `coverage_floor` for legacy catalogs that omit the field. 0.10 is
/// low enough that even sparse mining usually clears it; future ratchets land
/// via deliberate human commits, never auto-tuning by the walker.
fn default_coverage_floor() -> f64 {
    0.10
}

/// Time-bounded waiver of the `alignment_ratio < floor` pre-push block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorWaiver {
    pub until: String,
    pub rationale: String,
}

/// Row source citation. Either a single object or an array of objects
/// (multi-source rows produced by `merge-shards`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Source {
    Single(SourceCite),
    Multi(Vec<SourceCite>),
}

impl Source {
    #[must_use]
    pub fn as_slice(&self) -> Vec<SourceCite> {
        match self {
            Source::Single(s) => vec![s.clone()],
            Source::Multi(v) => v.clone(),
        }
    }
}

/// One source citation (file + 1-based inclusive line range).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCite {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Catalog row.
///
/// **Parallel-array invariants:**
///
/// - W7 / v0.12.1: `tests.len() == test_body_hashes.len()` at all times.
///   Each `tests[i]` (a `<file>::<fn>` citation) has its corresponding hash
///   in `test_body_hashes[i]`. Empty vec means "no test bound yet" -- the
///   previous `Option<String>` `None` semantics. Mutation must go through
///   [`Row::set_tests`] to preserve the invariant; readers may rely on it.
/// - P78 / MULTI-SOURCE-WATCH-01: `source.as_slice().len() ==
///   source_hashes.len()` post-[`Catalog::load`] backfill. Each
///   `source.as_slice()[i]` (a `SourceCite`) has its corresponding hash in
///   `source_hashes[i]`. Empty vec means "no hash recorded yet" (parallel to
///   the empty-tests semantic). Mutation must go through [`Row::set_source`]
///   to preserve the invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    pub claim: String,
    pub source: Source,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,

    /// Per-source hashes parallel to `source.as_slice()` (path-b per
    /// MULTI-SOURCE-WATCH-01 / P78). `source_hashes[i]` is the
    /// `hash::source_hash` of `source.as_slice()[i]`'s byte range.
    /// Empty vec means "no hashes recorded yet" (matches the empty-tests
    /// semantic of `tests` / `test_body_hashes`).
    ///
    /// **Parallel-array invariant** (P78): `source.as_slice().len() ==
    /// source_hashes.len()` post-load (after [`Catalog::load`]'s one-time
    /// backfill from the legacy `source_hash` field). Mutation must go
    /// through [`Row::set_source`] to preserve the invariant; readers may
    /// rely on it.
    ///
    /// Back-compat: legacy `source_hash` field stays on the struct for one
    /// release cycle. Newer catalogs may have BOTH fields; readers MUST
    /// prefer `source_hashes` post-backfill. Writers SHOULD update both
    /// during the transition (so a downgrade rollback can still load the
    /// catalog).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_hashes: Vec<String>,

    /// Test citations (`<file>::<fn>` form). One claim may bind to many tests.
    /// Empty means no test bound. Parallel-arrays with `test_body_hashes`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<String>,

    /// Hashes parallel to `tests`. `test_body_hashes[i]` is the
    /// `to_token_stream()` sha256 of the fn body cited by `tests[i]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_body_hashes: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,

    pub last_verdict: RowState,

    /// Action that closes this row's gap. Set by extractors at mint time;
    /// updated by graders on refresh. Consumer-side filter for cluster
    /// phase scoping (e.g. P72-P80 select rows where
    /// `next_action == FIX_IMPL_THEN_BIND`). Pre-W4 catalogs without the
    /// field deserialize as `WRITE_TEST` via `serde(default)`.
    #[serde(default = "NextAction::default_for_back_compat")]
    pub next_action: NextAction,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_extracted: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_extracted_by: Option<String>,
}

impl Row {
    /// Set both `tests` and `test_body_hashes` together. Validates the
    /// parallel-array invariant; rejects mismatched lengths so writer paths
    /// cannot accidentally store an inconsistent row.
    ///
    /// # Errors
    ///
    /// Errors if `tests.len() != hashes.len()`.
    pub fn set_tests(&mut self, tests: Vec<String>, hashes: Vec<String>) -> Result<()> {
        if tests.len() != hashes.len() {
            return Err(anyhow::anyhow!(
                "Row::set_tests: tests.len ({}) != test_body_hashes.len ({})",
                tests.len(),
                hashes.len(),
            ));
        }
        self.tests = tests;
        self.test_body_hashes = hashes;
        Ok(())
    }

    /// Set `source` and `source_hashes` together (P78 MULTI-SOURCE-WATCH-01).
    /// Validates the parallel-array invariant; rejects mismatched lengths so
    /// writer paths cannot accidentally store an inconsistent row. Mirrors
    /// [`Row::set_tests`]'s discipline for the `tests` / `test_body_hashes` pair.
    ///
    /// Also keeps the legacy `source_hash` field in sync with
    /// `source_hashes[0]` for one release cycle — back-compat for downgrade
    /// rollback. The legacy field can be retired post-v0.14.0.
    ///
    /// # Errors
    ///
    /// Errors if `source.as_slice().len() != hashes.len()`.
    pub fn set_source(&mut self, source: Source, hashes: Vec<String>) -> Result<()> {
        if source.as_slice().len() != hashes.len() {
            return Err(anyhow::anyhow!(
                "Row::set_source: source.as_slice().len ({}) != source_hashes.len ({})",
                source.as_slice().len(),
                hashes.len(),
            ));
        }
        self.source = source;
        self.source_hashes = hashes;
        // Back-compat: keep source_hash in sync with the first element for
        // one release cycle. Drop after v0.14.0 once the field is unused.
        self.source_hash = self.source_hashes.first().cloned();
        Ok(())
    }

    /// Validate the parallel-array invariants on this row. Cheap to call after
    /// any direct mutation that bypasses [`Row::set_tests`] / [`Row::set_source`]
    /// (deserializers, for instance, populate fields independently).
    ///
    /// Validates BOTH parallel-array invariants:
    /// - W7: `tests.len() == test_body_hashes.len()`
    /// - P78: `source.as_slice().len() == source_hashes.len()` (only when
    ///   `source_hashes` is non-empty — empty `source_hashes` is the
    ///   "no hashes recorded yet" semantic, parallel to empty `tests`).
    ///
    /// # Errors
    ///
    /// Errors if either invariant is violated.
    pub fn validate_parallel_arrays(&self) -> Result<()> {
        if self.tests.len() != self.test_body_hashes.len() {
            return Err(anyhow::anyhow!(
                "Row {} parallel-array invariant violated: tests.len ({}) != test_body_hashes.len ({})",
                self.id,
                self.tests.len(),
                self.test_body_hashes.len(),
            ));
        }
        if !self.source_hashes.is_empty()
            && self.source.as_slice().len() != self.source_hashes.len()
        {
            return Err(anyhow::anyhow!(
                "Row {} parallel-array invariant violated: source.as_slice().len ({}) != source_hashes.len ({})",
                self.id,
                self.source.as_slice().len(),
                self.source_hashes.len(),
            ));
        }
        Ok(())
    }

    /// Clear all test bindings (used by `mark-missing-test` and similar verbs
    /// that detach a row from any test).
    pub fn clear_tests(&mut self) {
        self.tests.clear();
        self.test_body_hashes.clear();
    }
}

/// Row state machine -- 8 variants from `02-architecture.md` § "Row state machine".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RowState {
    Bound,
    MissingTest,
    StaleDocsDrift,
    StaleTestDrift,
    StaleTestGone,
    TestMisaligned,
    RetireProposed,
    RetireConfirmed,
}

/// Action that closes a row's gap. Set by extractors at mint time;
/// updated by graders on refresh; consumer-side filter for cluster
/// phase scoping (e.g. "all rows where `next_action == FIX_IMPL_THEN_BIND`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NextAction {
    /// Test missing for a clear claim; write the test.
    WriteTest,
    /// Implementation regressed or never landed; fix impl, then bind.
    FixImplThenBind,
    /// Prose names a stale shape; update the doc, then rebind.
    UpdateDoc,
    /// Feature was intentionally dropped; needs `RETIRE_PROPOSED` -> `RETIRE_CONFIRMED`.
    RetireFeature,
    /// Already bound to a green test; nothing to do.
    BindGreen,
}

impl NextAction {
    /// Default used by `serde(default)` for back-compat with W7 (and
    /// earlier) catalogs that lack the field.
    #[must_use]
    pub fn default_for_back_compat() -> Self {
        Self::WriteTest
    }

    /// Display name for stderr / status output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            NextAction::WriteTest => "WRITE_TEST",
            NextAction::FixImplThenBind => "FIX_IMPL_THEN_BIND",
            NextAction::UpdateDoc => "UPDATE_DOC",
            NextAction::RetireFeature => "RETIRE_FEATURE",
            NextAction::BindGreen => "BIND_GREEN",
        }
    }

    /// Parse from the `SCREAMING_SNAKE_CASE` string. Used by the
    /// `--next-action <value>` CLI flag.
    ///
    /// # Errors
    ///
    /// Errors if `s` is not one of the five known variants.
    pub fn parse_cli(s: &str) -> Result<Self> {
        match s {
            "WRITE_TEST" => Ok(NextAction::WriteTest),
            "FIX_IMPL_THEN_BIND" => Ok(NextAction::FixImplThenBind),
            "UPDATE_DOC" => Ok(NextAction::UpdateDoc),
            "RETIRE_FEATURE" => Ok(NextAction::RetireFeature),
            "BIND_GREEN" => Ok(NextAction::BindGreen),
            _ => Err(anyhow::anyhow!(
                "invalid --next-action value `{s}` (expected one of WRITE_TEST, FIX_IMPL_THEN_BIND, UPDATE_DOC, RETIRE_FEATURE, BIND_GREEN)"
            )),
        }
    }
}

impl RowState {
    /// Returns true if the state blocks pre-push.
    #[must_use]
    pub fn blocks_pre_push(self) -> bool {
        matches!(
            self,
            RowState::MissingTest
                | RowState::StaleDocsDrift
                | RowState::StaleTestGone
                | RowState::TestMisaligned
                | RowState::RetireProposed
        )
    }

    /// Display name for stderr messages.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RowState::Bound => "BOUND",
            RowState::MissingTest => "MISSING_TEST",
            RowState::StaleDocsDrift => "STALE_DOCS_DRIFT",
            RowState::StaleTestDrift => "STALE_TEST_DRIFT",
            RowState::StaleTestGone => "STALE_TEST_GONE",
            RowState::TestMisaligned => "TEST_MISALIGNED",
            RowState::RetireProposed => "RETIRE_PROPOSED",
            RowState::RetireConfirmed => "RETIRE_CONFIRMED",
        }
    }
}

impl Catalog {
    /// Load a catalog from disk. Errors if the file is missing or malformed.
    /// Validates the [`Row`] parallel-array invariant on every row -- a
    /// catalog that violates it is rejected at load rather than carrying a
    /// silent corruption forward.
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading catalog at {}", path.display()))?;
        let mut cat: Catalog = serde_json::from_str(&raw)
            .with_context(|| format!("parsing catalog at {}", path.display()))?;

        // P78 MULTI-SOURCE-WATCH-01 backfill: legacy catalogs have
        // `source_hash: Option<String>` and lack `source_hashes`. Promote
        // `source_hash` into `source_hashes[0]` so every read path enters
        // the new world. Idempotent: if `source_hashes` is already
        // populated (newer catalog), the backfill is a no-op.
        //
        // Multi-source legacy rows: pre-P78 the bind verb only stored
        // `source_hash = hash(first source)` (P75 first-source invariant);
        // the OTHER source hashes were never recorded under path-(a).
        // Backfilling `source_hashes = [legacy_hash]` for an N-cite Multi
        // row would violate the parallel-array invariant
        // (`source.len() != source_hashes.len()`). Instead, leave such
        // rows with `source_hashes: []` -- the "no-hash-recorded-yet"
        // semantic. The walker treats empty `source_hashes` as "skip
        // drift compare" (preserving the path-(a) tradeoff for these
        // legacy rows until they re-bind through P78-aware bind logic,
        // which populates the full parallel array).
        //
        // Rows with `source_hash: None` keep `source_hashes: []`
        // (no-hash-recorded-yet semantic, unchanged).
        for row in &mut cat.rows {
            if row.source_hashes.is_empty() {
                if let Some(legacy) = row.source_hash.clone() {
                    // Only backfill when the parallel-array invariant will
                    // hold. Single-source rows: legacy_hash matches the
                    // single cite. Multi-source rows: backfill would create
                    // an inconsistent shape; skip and let re-bind heal.
                    if row.source.as_slice().len() == 1 {
                        row.source_hashes.push(legacy);
                    }
                }
            }
        }

        for row in &cat.rows {
            row.validate_parallel_arrays()
                .with_context(|| format!("validating row in catalog at {}", path.display()))?;
        }
        Ok(cat)
    }

    /// Atomic write: serialize to a sibling `.tmp` then rename onto the target.
    pub fn save(&self, path: &Path) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(self).context("serializing catalog")?;
        bytes.push(b'\n');

        // Sibling tmp keeps the rename atomic on POSIX.
        let tmp = if let Some(name) = path.file_name() {
            let mut t = path.to_path_buf();
            let mut fname = name.to_os_string();
            fname.push(".tmp");
            t.set_file_name(fname);
            t
        } else {
            let mut t = path.to_path_buf();
            t.set_extension("tmp");
            t
        };

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating parent dir {}", parent.display()))?;
            }
        }

        {
            let mut f = fs::File::create(&tmp)
                .with_context(|| format!("opening tmp file {}", tmp.display()))?;
            f.write_all(&bytes)
                .with_context(|| format!("writing tmp file {}", tmp.display()))?;
            f.sync_all().ok();
        }

        fs::rename(&tmp, path)
            .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    }

    /// Recompute summary counters from the current row states.
    pub fn recompute_summary(&mut self) {
        let mut total: u64 = 0;
        let mut bound: u64 = 0;
        let mut missing: u64 = 0;
        let mut retire_prop: u64 = 0;
        let mut retired: u64 = 0;

        for r in &self.rows {
            total += 1;
            match r.last_verdict {
                RowState::Bound => bound += 1,
                RowState::MissingTest | RowState::TestMisaligned | RowState::StaleTestGone => {
                    missing += 1;
                }
                RowState::RetireProposed => retire_prop += 1,
                RowState::RetireConfirmed => retired += 1,
                RowState::StaleDocsDrift | RowState::StaleTestDrift => {}
            }
        }

        self.summary.claims_total = total;
        self.summary.claims_bound = bound;
        self.summary.claims_missing_test = missing;
        self.summary.claims_retire_proposed = retire_prop;
        self.summary.claims_retired = retired;

        // alignment_ratio = claims_bound / max(1, claims_total - claims_retired)
        let denom = (total.saturating_sub(retired)).max(1);
        #[allow(clippy::cast_precision_loss)]
        let ratio = (bound as f64) / (denom as f64);
        // Snap to 1.0 when total==0 to match the empty-state seed.
        self.summary.alignment_ratio = if total == 0 { 1.0 } else { ratio };
    }

    /// Find a mutable row by id.
    #[must_use]
    pub fn row_mut(&mut self, id: &str) -> Option<&mut Row> {
        self.rows.iter_mut().find(|r| r.id == id)
    }

    /// Find a row by id.
    #[must_use]
    pub fn row(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.id == id)
    }
}
