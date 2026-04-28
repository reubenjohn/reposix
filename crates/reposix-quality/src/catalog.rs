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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    pub claim: String,
    pub source: Source,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_body_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,

    pub last_verdict: RowState,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_extracted: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_extracted_by: Option<String>,
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
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading catalog at {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("parsing catalog at {}", path.display()))
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
