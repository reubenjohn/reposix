//! `reposix-quality bind --dimension agent-ux` -- mint or refresh a row in
//! `quality/catalogs/agent-ux.json`.
//!
//! GOOD-TO-HAVES-01 (v0.13.0 -> v0.14.0 carry-forward) Path A: direct-JSON
//! code path that does NOT refactor the `docs-alignment` `Row` struct. Each
//! catalog dimension has its own row shape (agent-ux uses `command` +
//! `expected.asserts` + `verifier.{script,args,timeout_s,container}` instead
//! of `source` + `test` + `source_hash`); a future refactor (Path B) may unify
//! behind a `Row` enum, but for now we serialize directly via
//! `serde_json::Value` to mint agent-ux rows without touching docs-alignment
//! tests.
//!
//! Validation:
//!   - row-id must have the `agent-ux/` prefix
//!   - verifier script path must exist on the live filesystem
//!   - every cited source path must exist on the live filesystem
//!   - `blast_radius` must be one of P0|P1|P2
//!
//! Idempotence: a re-bind that names the same row-id refreshes
//! `last_verified` (and any field newly supplied via flags); a row that
//! already exists with the same shape and same verifier returns 0 with no
//! disk write skipped semantics — we always rewrite the timestamp so the
//! freshness TTL clock resets, matching the `bind`-then-runner-grades flow.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::{json, Map, Value};

/// Mint or refresh an agent-ux catalog row.
///
/// Pretty-prints JSON back to disk preserving 2-space indent + trailing
/// newline (matches `Catalog::save` shape and `serde_json::to_vec_pretty`'s
/// 2-space default).
///
/// # Errors
///
/// - row-id missing `agent-ux/` prefix
/// - verifier script missing
/// - any source path missing
/// - `blast_radius` not in {P0, P1, P2}
/// - catalog file unreadable / unparseable
#[allow(
    clippy::too_many_arguments,
    reason = "All flags map 1:1 to a catalog-row field; collapsing into a struct adds an indirection that tests don't need."
)]
#[allow(
    clippy::too_many_lines,
    reason = "Single coherent procedure: validate -> load -> upsert -> write."
)]
pub fn bind(
    catalog: &Path,
    row_id: &str,
    verifier: &Path,
    cadence: &str,
    kind: &str,
    args: &[String],
    timeout_s: u64,
    blast_radius: &str,
    freshness_ttl: Option<&str>,
    sources: &[String],
    owner_hint: Option<&str>,
    asserts: &[String],
) -> Result<i32> {
    // -- validation --------------------------------------------------------

    if !row_id.starts_with("agent-ux/") {
        return Err(anyhow!(
            "bind --dimension agent-ux: --row-id `{row_id}` must start with `agent-ux/` (catalog-id prefix invariant; see quality/catalogs/README.md § 'Per-row schema')"
        ));
    }
    if !verifier.exists() {
        return Err(anyhow!(
            "bind --dimension agent-ux: verifier script `{}` does not exist",
            verifier.display()
        ));
    }
    if sources.is_empty() {
        return Err(anyhow!(
            "bind --dimension agent-ux: at least one --source <path> is required"
        ));
    }
    for s in sources {
        if !Path::new(s).exists() {
            return Err(anyhow!(
                "bind --dimension agent-ux: source `{s}` does not exist"
            ));
        }
    }
    if !matches!(blast_radius, "P0" | "P1" | "P2") {
        return Err(anyhow!(
            "bind --dimension agent-ux: --blast-radius `{blast_radius}` must be one of P0, P1, P2"
        ));
    }

    // -- load --------------------------------------------------------------

    let raw = fs::read_to_string(catalog)
        .with_context(|| format!("reading catalog at {}", catalog.display()))?;
    let mut doc: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing catalog at {} as JSON", catalog.display()))?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let command = format!(
        "bash {}{}",
        verifier.display(),
        if args.is_empty() {
            String::new()
        } else {
            format!(" {}", args.join(" "))
        },
    );
    let artifact_slug = row_id.trim_start_matches("agent-ux/");
    let artifact = format!("quality/reports/verifications/agent-ux/{artifact_slug}.json");

    // -- build the row Value (preserving existing fields where possible) --

    let rows = doc
        .get_mut("rows")
        .ok_or_else(|| anyhow!("catalog {} has no `rows` array", catalog.display()))?
        .as_array_mut()
        .ok_or_else(|| anyhow!("catalog {} `rows` is not an array", catalog.display()))?;

    let existing_idx = rows.iter().position(|r| {
        r.get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == row_id)
    });

    // Preserve fields we don't manage: `_provenance_note` (historical hand-
    // edit annotation, kept verbatim per the task constraints), `waiver`,
    // and any future opaque fields. We rebuild the managed fields fresh.
    let mut row_obj: Map<String, Value> = if let Some(idx) = existing_idx {
        match rows[idx].clone() {
            Value::Object(m) => m,
            _ => Map::new(),
        }
    } else {
        Map::new()
    };

    row_obj.insert("id".into(), json!(row_id));
    row_obj.insert("dimension".into(), json!("agent-ux"));
    row_obj.insert("cadence".into(), json!(cadence));
    row_obj.insert("kind".into(), json!(kind));
    row_obj.insert("sources".into(), json!(sources));
    row_obj.insert("command".into(), json!(command));
    row_obj.insert(
        "expected".into(),
        json!({
            "asserts": asserts,
        }),
    );
    row_obj.insert(
        "verifier".into(),
        json!({
            "script": verifier.to_string_lossy(),
            "args": args,
            "timeout_s": timeout_s,
            "container": Value::Null,
        }),
    );
    row_obj.insert("artifact".into(), json!(artifact));
    // Mint state: status FAIL + fresh last_verified. The runner re-grades
    // to PASS on its next dispatch; matches `_provenance_note` rows that
    // shipped with status PASS only because the runner already graded.
    row_obj.insert("status".into(), json!("FAIL"));
    row_obj.insert("last_verified".into(), json!(now));
    row_obj.insert(
        "freshness_ttl".into(),
        match freshness_ttl {
            Some(ttl) => json!(ttl),
            None => Value::Null,
        },
    );
    row_obj.insert("blast_radius".into(), json!(blast_radius));
    if let Some(hint) = owner_hint {
        row_obj.insert("owner_hint".into(), json!(hint));
    } else if !row_obj.contains_key("owner_hint") {
        row_obj.insert("owner_hint".into(), Value::Null);
    }
    if !row_obj.contains_key("waiver") {
        row_obj.insert("waiver".into(), Value::Null);
    }

    let row_val = Value::Object(row_obj);
    if let Some(idx) = existing_idx {
        rows[idx] = row_val;
    } else {
        rows.push(row_val);
    }

    // -- write -------------------------------------------------------------

    let mut bytes = serde_json::to_vec_pretty(&doc).context("serializing agent-ux catalog")?;
    bytes.push(b'\n');

    // Atomic write: sibling .tmp + rename (matches Catalog::save).
    let tmp = if let Some(name) = catalog.file_name() {
        let mut t = catalog.to_path_buf();
        let mut fname = name.to_os_string();
        fname.push(".tmp");
        t.set_file_name(fname);
        t
    } else {
        let mut t = catalog.to_path_buf();
        t.set_extension("tmp");
        t
    };
    if let Some(parent) = catalog.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating parent dir {}", parent.display()))?;
        }
    }
    fs::write(&tmp, &bytes).with_context(|| format!("writing tmp file {}", tmp.display()))?;
    fs::rename(&tmp, catalog)
        .with_context(|| format!("renaming {} -> {}", tmp.display(), catalog.display()))?;

    Ok(0)
}

/// Default catalog path for the agent-ux dimension (used when --catalog
/// is left as the docs-alignment default and --dimension agent-ux is set).
#[must_use]
pub fn default_catalog() -> PathBuf {
    PathBuf::from("quality/catalogs/agent-ux.json")
}
