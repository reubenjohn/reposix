//! POC reconciler for path (a). Throwaway code per CARRY-FORWARD POC-DVCS-01.
//!
//! Walks a working tree of `.md` files, parses YAML frontmatter via
//! `reposix_core::frontmatter`, and reconciles each parsed `id` against
//! the simulator's `/projects/<slug>/issues` endpoint. Prints a 5-row
//! reconciliation classification, exits 0 only if all 5 cases observed.
//!
//! Five cases (architecture-sketch.md § "Reconciliation cases"):
//!   1. MATCH            local id matches a backend record
//!   2. BACKEND_DELETED  local id has no backend record
//!   3. NO_ID            local file has no parseable frontmatter `id`
//!   4. DUPLICATE_ID     two local files claim the same id
//!   5. MIRROR_LAG       backend has a record with no local file

use anyhow::{Context, Result};
use clap::Parser;
use reposix_core::frontmatter;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(about = "POC reconciler for path (a). Throwaway.")]
struct Args {
    /// Working tree to walk.
    #[arg(long)]
    working_tree: PathBuf,
    /// SoT origin URL, e.g. http://127.0.0.1:7888
    #[arg(long)]
    sot_origin: String,
    /// Project slug, e.g. demo
    #[arg(long)]
    project: String,
}

/// Subset of the simulator's record shape we read off the wire.
#[derive(Debug, Deserialize)]
struct BackendRecord {
    id: u64,
    #[serde(default)]
    title: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 1. List backend records via the sim's `/projects/<slug>/issues` endpoint.
    //    NOTE: The architecture sketch refers abstractly to "records"; the sim's
    //    concrete route is `/issues`. This naming inconsistency is itself a
    //    POC finding (see POC-FINDINGS.md § Path (a)).
    let url = format!("{}/projects/{}/issues", args.sot_origin, args.project);
    eprintln!("[poc-reconciler] GET {url}");
    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("non-2xx from {url}"))?;
    let backend: Vec<BackendRecord> = resp
        .json()
        .await
        .with_context(|| format!("parse JSON from {url}"))?;
    let backend_ids: HashMap<u64, String> = backend
        .into_iter()
        .map(|r| (r.id, r.title))
        .collect();
    eprintln!(
        "[poc-reconciler] backend has {} record(s): {:?}",
        backend_ids.len(),
        backend_ids.keys().collect::<Vec<_>>()
    );

    // 2. Walk the working tree; for each .md file, parse frontmatter; collect
    //    id -> [paths] map. Files that fail to parse fall into no_id_files.
    let mut local_ids: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut no_id_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&args.working_tree) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        // Skip files inside .git/ if any
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str() == ".git")
        {
            continue;
        }
        let bytes = std::fs::read_to_string(entry.path())
            .with_context(|| format!("read {}", entry.path().display()))?;
        match frontmatter::parse(&bytes) {
            Ok(rec) => {
                let id: u64 = rec.id.0;
                local_ids
                    .entry(id)
                    .or_default()
                    .push(entry.path().to_path_buf());
            }
            Err(_) => {
                no_id_files.push(entry.path().to_path_buf());
            }
        }
    }

    // 3. Print the reconciliation table.
    println!("=== Reconciliation table (POC) ===");
    let mut duplicate_seen = false;
    let mut backend_deleted_seen = false;
    let mut no_id_seen = false;
    let mut match_seen = false;
    let mut mirror_lag_seen = false;

    let mut local_id_keys: Vec<u64> = local_ids.keys().copied().collect();
    local_id_keys.sort_unstable();
    for id in local_id_keys {
        let paths = &local_ids[&id];
        if paths.len() > 1 {
            let names: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            println!("DUPLICATE_ID id={id} files=[{}]", names.join(","));
            duplicate_seen = true;
        } else if backend_ids.contains_key(&id) {
            println!("MATCH id={id} file={}", paths[0].display());
            match_seen = true;
        } else {
            println!(
                "BACKEND_DELETED id={id} local_file={}",
                paths[0].display()
            );
            backend_deleted_seen = true;
        }
    }
    let mut no_id_paths: Vec<PathBuf> = no_id_files;
    no_id_paths.sort();
    for f in &no_id_paths {
        println!("NO_ID file={}", f.display());
        no_id_seen = true;
    }
    let mut backend_only: Vec<u64> = backend_ids
        .keys()
        .copied()
        .filter(|id| !local_ids.contains_key(id))
        .collect();
    backend_only.sort_unstable();
    for backend_id in backend_only {
        println!("MIRROR_LAG id={backend_id} (backend has, local lacks)");
        mirror_lag_seen = true;
    }

    let all_observed = match_seen
        && backend_deleted_seen
        && no_id_seen
        && duplicate_seen
        && mirror_lag_seen;
    println!();
    println!("ALL_5_CASES_OBSERVED={all_observed}");
    println!(
        "  match={match_seen} backend_deleted={backend_deleted_seen} \
         no_id={no_id_seen} duplicate={duplicate_seen} mirror_lag={mirror_lag_seen}"
    );

    // Exit 0 only if all 5 cases observed; non-zero is a finding.
    std::process::exit(if all_observed { 0 } else { 1 });
}
