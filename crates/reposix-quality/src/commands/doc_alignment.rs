//! `reposix-quality doc-alignment <verb>` -- 9 verbs.
//!
//! Verb shapes are byte-equivalent to the skill prompts at
//! `.claude/skills/reposix-quality-doc-alignment/prompts/{extractor,grader}.md`.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Subcommand;

use crate::catalog::Catalog;

/// All 9 docs-alignment verbs.
#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Bind a row: validate citations, compute hashes, persist with `last_verdict=BOUND`.
    Bind {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        /// Source citation: `<file>:<line_start>-<line_end>`.
        #[arg(long)]
        source: String,
        /// Test fn citation (`<file>::<fn>`). Repeatable -- one per binding.
        #[arg(long, action = clap::ArgAction::Append, num_args = 1, required = true)]
        test: Vec<String>,
        /// Grade verdict (must be `GREEN`).
        #[arg(long)]
        grade: String,
        #[arg(long)]
        rationale: String,
    },

    /// Propose retiring a row: persist with `last_verdict=RETIRE_PROPOSED`.
    #[command(name = "propose-retire")]
    ProposeRetire {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        rationale: String,
    },

    /// Confirm a retirement -- HUMAN ONLY (env-guarded).
    #[command(name = "confirm-retire")]
    ConfirmRetire {
        #[arg(long = "row-id")]
        row_id: String,
        /// EXPLICIT human authorization to bypass the env-guard. Use only
        /// when you are a human running confirm-retire from a Claude Code
        /// session rather than a fresh TTY. Audit-trailed in
        /// `last_extracted_by` as `confirm-retire-i-am-human`. Subagents
        /// must not pass this.
        #[arg(long = "i-am-human", default_value_t = false)]
        i_am_human: bool,
    },

    /// Mark a row's binding as missing/misaligned (`last_verdict=MISSING_TEST`).
    #[command(name = "mark-missing-test")]
    MarkMissingTest {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        rationale: Option<String>,
        /// Explicit `next_action` override. Accepts one of:
        /// `WRITE_TEST`, `FIX_IMPL_THEN_BIND`, `UPDATE_DOC`, `RETIRE_FEATURE`,
        /// `BIND_GREEN`. When omitted, the value is derived from the
        /// rationale prefix (`IMPL_GAP:` -> `FIX_IMPL_THEN_BIND`,
        /// `DOC_DRIFT:` -> `UPDATE_DOC`, otherwise `WRITE_TEST`).
        #[arg(long = "next-action")]
        next_action: Option<String>,
    },

    /// Emit stale-row JSON manifest for one doc to stdout.
    #[command(name = "plan-refresh")]
    PlanRefresh {
        /// Doc file whose stale rows should be planned.
        doc_file: PathBuf,
    },

    /// Write a deterministic backfill MANIFEST.json under quality/reports/doc-alignment/.
    #[command(name = "plan-backfill")]
    PlanBackfill,

    /// Merge per-shard JSONs from a backfill run-dir into the catalog.
    #[command(name = "merge-shards")]
    MergeShards {
        /// Run directory with `shards/*.json` produced by the backfill subagents.
        run_dir: PathBuf,
    },

    /// Hash drift walker -- updates `last_verdict` only.
    Walk,

    /// Print summary block + per-file coverage table (table by default; `--json` for machine-readable).
    Status {
        #[arg(long)]
        json: bool,

        /// Show top N worst-covered files (default 20). Ignored if `--all` is set.
        #[arg(long, default_value_t = 20)]
        top: usize,

        /// Show every eligible file (overrides `--top`).
        #[arg(long)]
        all: bool,
    },
}

/// Dispatch a doc-alignment verb. Returns the process exit code on success.
pub fn dispatch(catalog: &Path, verb: Verb) -> Result<i32> {
    match verb {
        Verb::Bind {
            row_id,
            claim,
            source,
            test,
            grade,
            rationale,
        } => verbs::bind(catalog, &row_id, &claim, &source, &test, &grade, &rationale),
        Verb::ProposeRetire {
            row_id,
            claim,
            source,
            rationale,
        } => verbs::propose_retire(catalog, &row_id, &claim, &source, &rationale),
        Verb::ConfirmRetire { row_id, i_am_human } => {
            verbs::confirm_retire(catalog, &row_id, i_am_human)
        }
        Verb::MarkMissingTest {
            row_id,
            claim,
            source,
            rationale,
            next_action,
        } => verbs::mark_missing_test(
            catalog,
            &row_id,
            &claim,
            &source,
            rationale.as_deref(),
            next_action.as_deref(),
        ),
        Verb::PlanRefresh { doc_file } => verbs::plan_refresh(catalog, &doc_file),
        Verb::PlanBackfill => verbs::plan_backfill(catalog),
        Verb::MergeShards { run_dir } => verbs::merge_shards(catalog, &run_dir),
        Verb::Walk => verbs::walk(catalog),
        Verb::Status { json, top, all } => verbs::status(catalog, json, top, all),
    }
}

/// Read-only `verify --row-id <id>` -- prints row state to stdout.
///
/// For rows with `tests.len() >= 2`, also prints a per-test drift table
/// (W7 / v0.12.1) so operators can see which specific binding drifted
/// without having to re-run `walk` and parse stderr diagnostics.
pub fn verify(catalog: &Path, row_id: &str) -> Result<i32> {
    let cat = Catalog::load(catalog)?;
    if let Some(r) = cat.row(row_id) {
        let body = serde_json::to_string_pretty(r)?;
        println!("{body}");
        // Surface last_verdict + next_action together (W4 / v0.12.1 P68)
        // so operators can scope cluster-closure work without re-grepping.
        println!();
        println!(
            "== summary ==  last_verdict={}  next_action={}",
            r.last_verdict.as_str(),
            r.next_action.as_str(),
        );
        if r.tests.len() >= 2 {
            println!();
            println!("== per-test drift (multi-test row) ==");
            println!(
                "  {:<5}  {:<40}  test ref",
                "idx", "stored hash (16-prefix)"
            );
            println!("  {}", "-".repeat(5 + 2 + 40 + 2 + 32));
            for (i, t) in r.tests.iter().enumerate() {
                let prefix = r.test_body_hashes.get(i).map_or_else(
                    || "<missing>".to_string(),
                    |h| h.chars().take(16).collect::<String>(),
                );
                println!("  {i:<5}  {prefix:<40}  {t}");
            }
        }
        Ok(0)
    } else {
        eprintln!(
            "verify: row-id `{row_id}` not found in {}",
            catalog.display()
        );
        Ok(1)
    }
}

/// Verbs implementation namespace.
pub(crate) mod verbs {
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::IsTerminal;
    use std::path::{Path, PathBuf};

    use anyhow::{anyhow, Context, Result};
    use chrono::Utc;
    use serde_json::json;

    use super::{parse_source, parse_test};
    use crate::catalog::{Catalog, NextAction, Row, RowState, Source, SourceCite};
    use crate::coverage;
    use crate::hash;

    fn now_iso() -> String {
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Single coherent procedure: validate citations -> compute hashes -> dispatch new vs existing row -> persist. P78 added per-index source_hashes tracking that pushed the line count over the default cap; splitting hurts readability of the existing-row branch's heal logic."
    )]
    pub fn bind(
        catalog: &Path,
        row_id: &str,
        claim: &str,
        source: &str,
        tests: &[String],
        grade: &str,
        rationale: &str,
    ) -> Result<i32> {
        if grade != "GREEN" {
            return Err(anyhow!(
                "bind: --grade must be GREEN (got `{grade}`); regrade via the grader subagent first"
            ));
        }
        if tests.is_empty() {
            return Err(anyhow!(
                "bind: at least one --test <file::fn> is required (W7: bind binds a claim to >=1 tests)"
            ));
        }
        let (src_file, lstart, lend) = parse_source(source)?;
        if !src_file.exists() {
            return Err(anyhow!(
                "bind: source file `{}` does not exist",
                src_file.display()
            ));
        }

        // Validate every test citation BEFORE mutating the catalog so a
        // multi-test bind with one bogus entry leaves the catalog untouched.
        // Index-naming on failure (`--test #<i> "<value>"`) lets operators
        // see which specific entry tripped a multi-test invocation.
        let mut tests_clean: Vec<String> = Vec::with_capacity(tests.len());
        let mut hashes: Vec<String> = Vec::with_capacity(tests.len());
        for (i, t) in tests.iter().enumerate() {
            let test_ref = parse_test(t)
                .with_context(|| format!("bind: --test #{i} `{t}` failed to parse"))?;
            let test_file = test_ref.file();
            if !test_file.exists() {
                return Err(anyhow!(
                    "bind: --test #{i} `{t}`: test file `{}` does not exist",
                    test_file.display()
                ));
            }
            let body_hash = test_ref
                .compute_hash()
                .with_context(|| format!("bind: --test #{i} `{t}`: computing test hash failed"))?;
            tests_clean.push(t.clone());
            hashes.push(body_hash);
        }

        let src_hash = hash::source_hash(&src_file, lstart, lend)
            .with_context(|| format!("computing source_hash for {source}"))?;

        let mut cat = Catalog::load(catalog)?;
        let now = now_iso();
        let new_source = SourceCite {
            file: src_file.to_string_lossy().to_string(),
            line_start: lstart,
            line_end: lend,
        };

        if let Some(row) = cat.row_mut(row_id) {
            // Preserve multi-source rows: if existing source is Multi or
            // is a different Single citation, promote to Multi.
            let prior_cites = row.source.as_slice();
            let prior_hashes = std::mem::take(&mut row.source_hashes);
            let mut sources = prior_cites.clone();
            let already_present = sources.iter().any(|c| {
                c.file == new_source.file
                    && c.line_start == new_source.line_start
                    && c.line_end == new_source.line_end
            });
            if !already_present {
                sources.push(new_source.clone());
            }
            // BIND-VERB-FIX-01 (P75) + MULTI-SOURCE-WATCH-01 (P78):
            // Walker now AND-compares per-source hashes via
            // `source_hashes` (path-b -- closed in P78-03). The legacy
            // `source_hash` field tracks `source_hashes[0]` for one
            // release cycle (back-compat for downgrade rollback);
            // post-v0.14.0 `source_hash` can be retired.
            //
            // Heal paths (per-index):
            //   - Single result: refresh source_hashes[0] AND source_hash
            //     (heals Single rows whose prose drifted -- P74 finding).
            //   - Multi append: push freshly-bound hash; preserve prior
            //     index hashes (Multi rows accumulate; new source = new
            //     index).
            //   - Multi same-source rebind: refresh JUST that index in
            //     source_hashes (heals individual Multi entries without
            //     disturbing siblings).
            //
            // Locate where the freshly-bound source sits in `sources`
            // BEFORE moving the vec into the Source enum.
            let new_index = sources
                .iter()
                .position(|c| {
                    c.file == new_source.file
                        && c.line_start == new_source.line_start
                        && c.line_end == new_source.line_end
                })
                .expect("new_source must appear in sources after the append/heal logic");
            // Rebuild source_hashes parallel to sources. Reuse prior
            // entries where the cite is unchanged; insert/overwrite at
            // new_index for the freshly-bound source.
            let mut new_hashes: Vec<String> = Vec::with_capacity(sources.len());
            for (i, c) in sources.iter().enumerate() {
                if i == new_index {
                    new_hashes.push(src_hash.clone());
                } else if let Some(prior_idx) = prior_cites.iter().position(|p| {
                    p.file == c.file && p.line_start == c.line_start && p.line_end == c.line_end
                }) {
                    // Carry forward the prior hash for unchanged cites.
                    new_hashes.push(
                        prior_hashes
                            .get(prior_idx)
                            .cloned()
                            .unwrap_or_else(|| src_hash.clone()),
                    );
                } else {
                    // Cite never seen before; this branch shouldn't fire
                    // under the current bind algorithm (sources is built
                    // from the prior source.as_slice() + the new one)
                    // but defends against future shape changes.
                    new_hashes.push(src_hash.clone());
                }
            }
            let new_source_enum = if sources.len() == 1 {
                Source::Single(sources.into_iter().next().expect("len==1"))
            } else {
                Source::Multi(sources)
            };
            row.set_source(new_source_enum, new_hashes)?;
            row.claim = claim.to_string();
            row.set_tests(tests_clean, hashes)?;
            row.rationale = Some(rationale.to_string());
            row.last_verdict = RowState::Bound;
            row.next_action = NextAction::BindGreen;
            row.last_run = Some(now.clone());
            row.last_extracted = Some(now.clone());
            row.last_extracted_by = Some("bind-call".to_string());
        } else {
            let mut new_row = Row {
                id: row_id.to_string(),
                claim: claim.to_string(),
                source: Source::Single(new_source),
                source_hash: Some(src_hash.clone()),
                source_hashes: vec![src_hash],
                tests: Vec::new(),
                test_body_hashes: Vec::new(),
                rationale: Some(rationale.to_string()),
                last_verdict: RowState::Bound,
                next_action: NextAction::BindGreen,
                last_run: Some(now.clone()),
                last_extracted: Some(now.clone()),
                last_extracted_by: Some("bind-call".to_string()),
            };
            new_row.set_tests(tests_clean, hashes)?;
            cat.rows.push(new_row);
        }

        cat.recompute_summary();
        cat.save(catalog)?;
        Ok(0)
    }

    pub fn propose_retire(
        catalog: &Path,
        row_id: &str,
        claim: &str,
        source: &str,
        rationale: &str,
    ) -> Result<i32> {
        let (src_file, lstart, lend) = parse_source(source)?;
        if !src_file.exists() {
            return Err(anyhow!(
                "propose-retire: source file `{}` does not exist",
                src_file.display()
            ));
        }
        let mut cat = Catalog::load(catalog)?;
        let now = now_iso();
        let cite = SourceCite {
            file: src_file.to_string_lossy().to_string(),
            line_start: lstart,
            line_end: lend,
        };
        if let Some(row) = cat.row_mut(row_id) {
            row.claim = claim.to_string();
            row.rationale = Some(rationale.to_string());
            row.last_verdict = RowState::RetireProposed;
            row.next_action = NextAction::RetireFeature;
            row.last_run = Some(now.clone());
            row.last_extracted = Some(now);
            row.last_extracted_by = Some("propose-retire-call".to_string());
        } else {
            cat.rows.push(Row {
                id: row_id.to_string(),
                claim: claim.to_string(),
                source: Source::Single(cite),
                source_hash: None,
                // No source hash recorded for propose-retire rows; empty
                // source_hashes preserves the "no-hash-recorded-yet" semantic
                // (parallel to empty `tests`). P78 MULTI-SOURCE-WATCH-01.
                source_hashes: Vec::new(),
                tests: Vec::new(),
                test_body_hashes: Vec::new(),
                rationale: Some(rationale.to_string()),
                last_verdict: RowState::RetireProposed,
                next_action: NextAction::RetireFeature,
                last_run: Some(now.clone()),
                last_extracted: Some(now),
                last_extracted_by: Some("propose-retire-call".to_string()),
            });
        }
        cat.recompute_summary();
        cat.save(catalog)?;
        Ok(0)
    }

    pub fn confirm_retire(catalog: &Path, row_id: &str, i_am_human: bool) -> Result<i32> {
        let agent_env = std::env::var_os("CLAUDE_AGENT_CONTEXT").is_some();
        let non_tty = !std::io::stdin().is_terminal();
        if !i_am_human && (agent_env || non_tty) {
            eprintln!(
                "error: confirm-retire is human-only -- running under agent context (CLAUDE_AGENT_CONTEXT set) or non-tty stdin. Run from a fresh terminal, or pass --i-am-human if you are a human explicitly authorizing this retirement from a Claude Code session."
            );
            return Ok(1);
        }
        if i_am_human {
            eprintln!(
                "WARNING: --i-am-human bypassing env-guard for confirm-retire on row {row_id}. This is recorded as `confirm-retire-i-am-human` in the audit trail."
            );
        }
        let mut cat = Catalog::load(catalog)?;
        let now = now_iso();
        let row = cat
            .row_mut(row_id)
            .ok_or_else(|| anyhow!("confirm-retire: row-id `{row_id}` not found"))?;
        if row.last_verdict != RowState::RetireProposed {
            return Err(anyhow!(
                "confirm-retire: row `{row_id}` is in state {} -- only RETIRE_PROPOSED can be confirmed",
                row.last_verdict.as_str()
            ));
        }
        row.last_verdict = RowState::RetireConfirmed;
        row.last_run = Some(now.clone());
        // Audit trail: distinguish the human-bypass path from the strict path
        // so a commit-diff reviewer can spot --i-am-human use in retrospect.
        let by = if i_am_human {
            "confirm-retire-i-am-human"
        } else {
            "confirm-retire"
        };
        row.last_extracted = Some(now);
        row.last_extracted_by = Some(by.to_string());
        cat.recompute_summary();
        cat.save(catalog)?;
        Ok(0)
    }

    pub fn mark_missing_test(
        catalog: &Path,
        row_id: &str,
        claim: &str,
        source: &str,
        rationale: Option<&str>,
        next_action_override: Option<&str>,
    ) -> Result<i32> {
        let (src_file, lstart, lend) = parse_source(source)?;
        if !src_file.exists() {
            return Err(anyhow!(
                "mark-missing-test: source file `{}` does not exist",
                src_file.display()
            ));
        }
        let src_hash = hash::source_hash(&src_file, lstart, lend)
            .with_context(|| format!("computing source_hash for {source}"))?;

        // Resolve next_action: explicit override wins; else heuristic from
        // rationale prefix (`IMPL_GAP:` -> FIX_IMPL_THEN_BIND, `DOC_DRIFT:`
        // -> UPDATE_DOC, otherwise WRITE_TEST).
        let next_action = if let Some(s) = next_action_override {
            NextAction::parse_cli(s)?
        } else {
            match rationale {
                Some(r) if r.trim_start().starts_with("IMPL_GAP:") => NextAction::FixImplThenBind,
                Some(r) if r.trim_start().starts_with("DOC_DRIFT:") => NextAction::UpdateDoc,
                _ => NextAction::WriteTest,
            }
        };

        let mut cat = Catalog::load(catalog)?;
        let now = now_iso();
        let cite = SourceCite {
            file: src_file.to_string_lossy().to_string(),
            line_start: lstart,
            line_end: lend,
        };
        if let Some(row) = cat.row_mut(row_id) {
            row.claim = claim.to_string();
            // P78 MULTI-SOURCE-WATCH-01: keep source_hashes parallel-array
            // invariant by using set_source. mark-missing-test re-cites a
            // single source so source_hashes always becomes a 1-element vec.
            row.set_source(Source::Single(cite), vec![src_hash])?;
            // mark-missing-test detaches a row from any test bindings; clear
            // both parallel arrays atomically via the helper.
            row.clear_tests();
            if let Some(r) = rationale {
                row.rationale = Some(r.to_string());
            }
            row.last_verdict = RowState::MissingTest;
            row.next_action = next_action;
            row.last_run = Some(now.clone());
            row.last_extracted = Some(now);
            row.last_extracted_by = Some("mark-missing-test-call".to_string());
        } else {
            cat.rows.push(Row {
                id: row_id.to_string(),
                claim: claim.to_string(),
                source: Source::Single(cite),
                source_hash: Some(src_hash.clone()),
                // P78 MULTI-SOURCE-WATCH-01: parallel-array hash for the
                // single source citation (one element matching Source::Single).
                source_hashes: vec![src_hash],
                tests: Vec::new(),
                test_body_hashes: Vec::new(),
                rationale: rationale.map(std::string::ToString::to_string),
                last_verdict: RowState::MissingTest,
                next_action,
                last_run: Some(now.clone()),
                last_extracted: Some(now),
                last_extracted_by: Some("mark-missing-test-call".to_string()),
            });
        }
        cat.recompute_summary();
        cat.save(catalog)?;
        Ok(0)
    }

    /// Stale-row JSON for `<doc-file>` (rows whose `source.file` matches AND
    /// `last_verdict` is one of the stale/misaligned states).
    pub fn plan_refresh(catalog: &Path, doc_file: &Path) -> Result<i32> {
        let cat = Catalog::load(catalog)?;
        let target = doc_file.to_string_lossy().to_string();
        let stale_states = [
            RowState::StaleDocsDrift,
            RowState::StaleTestDrift,
            RowState::StaleTestGone,
            RowState::TestMisaligned,
        ];

        let stale_rows: Vec<_> = cat
            .rows
            .iter()
            .filter(|r| {
                stale_states.contains(&r.last_verdict)
                    && r.source.as_slice().iter().any(|c| c.file == target)
            })
            .map(|r| {
                let cite = r
                    .source
                    .as_slice()
                    .into_iter()
                    .find(|c| c.file == target)
                    .expect("filter ensured at least one matching cite");
                json!({
                    "id": r.id,
                    "claim": r.claim,
                    "source": format!("{}:{}-{}", cite.file, cite.line_start, cite.line_end),
                    "tests": r.tests,
                    "last_verdict": r.last_verdict.as_str(),
                    "next_action": r.next_action.as_str(),
                    "prior_rationale": r.rationale,
                })
            })
            .collect();

        let out = json!({ "stale_rows": stale_rows });
        println!("{}", serde_json::to_string_pretty(&out)?);
        Ok(0)
    }

    /// Deterministic backfill chunker.
    ///
    /// Walks `docs/**/*.md`, `README.md`, and v0.6.0 -- v0.11.0 archived
    /// REQUIREMENTS.md files. Groups by directory affinity, ≤3 files per
    /// shard. Output: MANIFEST.json under
    /// `quality/reports/doc-alignment/backfill-<UTC-iso>/`.
    pub fn plan_backfill(_catalog: &Path) -> Result<i32> {
        let ts = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let run_dir = PathBuf::from("quality/reports/doc-alignment").join(format!("backfill-{ts}"));
        fs::create_dir_all(&run_dir)
            .with_context(|| format!("creating run dir {}", run_dir.display()))?;

        let inputs = collect_backfill_inputs()?;
        let shards = chunk_by_dir_affinity(&inputs, 3);

        let manifest_shards: Vec<_> = shards
            .iter()
            .enumerate()
            .map(|(i, files)| {
                let id = format!("{:03}", i + 1);
                // Namespace: derive from the shared dir prefix or first file.
                let namespace = files
                    .first()
                    .map(|f| {
                        let mut buf = String::with_capacity(f.len());
                        for c in f.chars() {
                            buf.push(if c == '/' || c == '.' { '-' } else { c });
                        }
                        buf.trim_start_matches('-').to_string()
                    })
                    .unwrap_or_default();
                json!({
                    "id": id,
                    "files": files,
                    "row_id_namespace": namespace,
                })
            })
            .collect();

        let manifest = json!({
            "ts": ts,
            "shards": manifest_shards,
        });

        let manifest_path = run_dir.join("MANIFEST.json");
        let mut bytes = serde_json::to_vec_pretty(&manifest)?;
        bytes.push(b'\n');
        fs::write(&manifest_path, &bytes)
            .with_context(|| format!("writing {}", manifest_path.display()))?;
        println!("{}", manifest_path.display());
        Ok(0)
    }

    /// Collect backfill input files (deterministic order).
    fn collect_backfill_inputs() -> Result<Vec<String>> {
        let mut out = Vec::new();

        // README.md (if present)
        if Path::new("README.md").exists() {
            out.push("README.md".to_string());
        }

        // docs/**/*.md
        if Path::new("docs").is_dir() {
            walk_md(Path::new("docs"), &mut out)?;
        }

        // .planning/milestones/v0.6.0..v0.11.0-phases/REQUIREMENTS.md
        for v in &["v0.6.0", "v0.7.0", "v0.8.0", "v0.9.0", "v0.10.0", "v0.11.0"] {
            let p = format!(".planning/milestones/{v}-phases/REQUIREMENTS.md");
            if Path::new(&p).exists() {
                out.push(p);
            }
        }

        out.sort();
        out.dedup();
        Ok(out)
    }

    fn walk_md(dir: &Path, out: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir).with_context(|| format!("reading dir {}", dir.display()))? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                walk_md(&p, out)?;
            } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(p.to_string_lossy().to_string());
            }
        }
        Ok(())
    }

    /// Chunk files by directory affinity, then alphabetical fallback,
    /// max `cap` files per shard.
    fn chunk_by_dir_affinity(inputs: &[String], cap: usize) -> Vec<Vec<String>> {
        // Group by parent directory. BTreeMap keeps the keys sorted ->
        // deterministic iteration.
        let mut by_dir: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for f in inputs {
            let parent = Path::new(f)
                .parent()
                .map_or_else(|| ".".to_string(), |p| p.to_string_lossy().to_string());
            by_dir.entry(parent).or_default().push(f.clone());
        }
        for v in by_dir.values_mut() {
            v.sort();
        }

        let mut shards: Vec<Vec<String>> = Vec::new();
        // Pass 1: each dir's files get packed into shards of size `cap`.
        for files in by_dir.values() {
            for chunk in files.chunks(cap) {
                shards.push(chunk.to_vec());
            }
        }
        shards
    }

    /// Merge per-shard JSONs in `<run-dir>/shards/*.json` into the catalog.
    #[allow(
        clippy::too_many_lines,
        reason = "Single coherent procedure: load shards -> dedup-key -> conflict-detect -> upsert. Splitting hurts readability of a deterministic pipeline."
    )]
    pub fn merge_shards(catalog: &Path, run_dir: &Path) -> Result<i32> {
        let shards_dir = run_dir.join("shards");
        if !shards_dir.is_dir() {
            return Err(anyhow!(
                "merge-shards: shards dir `{}` does not exist",
                shards_dir.display()
            ));
        }
        let mut shard_paths: Vec<PathBuf> = fs::read_dir(&shards_dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
            .collect();
        shard_paths.sort();

        // (claim_normalized, test_or_empty) -> Vec<Row>
        let mut groups: BTreeMap<(String, String), Vec<Row>> = BTreeMap::new();
        for sp in &shard_paths {
            let raw = fs::read_to_string(sp)
                .with_context(|| format!("reading shard {}", sp.display()))?;
            let rows: Vec<Row> = serde_json::from_str(&raw)
                .with_context(|| format!("parsing shard {} as Vec<Row>", sp.display()))?;
            for r in rows {
                // Dedup-key the test side: pre-W7 a single `test: Option<String>`
                // sufficed; post-W7 multi-test rows fold by the join key
                // `tests.join(",")` so two shards proposing the SAME ordered
                // multi-test set merge cleanly. Single-test rows degrade to
                // the original key shape when `tests.len() == 1`.
                let key_tests = if r.tests.is_empty() {
                    String::new()
                } else {
                    r.tests.join(",")
                };
                let key = (r.claim.trim().to_lowercase(), key_tests);
                groups.entry(key).or_default().push(r);
            }
        }

        // Detect conflicts: same claim_normalized, different test bindings.
        let mut by_claim: BTreeMap<String, Vec<(String, Vec<Row>)>> = BTreeMap::new();
        for ((claim_norm, test_str), rows) in &groups {
            by_claim
                .entry(claim_norm.clone())
                .or_default()
                .push((test_str.clone(), rows.clone()));
        }

        let mut conflicts: Vec<String> = Vec::new();
        for (claim_norm, bucket) in &by_claim {
            // bound bindings = entries with non-empty test that have at least one BOUND row
            let bound_tests: Vec<&String> = bucket
                .iter()
                .filter(|(t, rows)| {
                    !t.is_empty() && rows.iter().any(|r| r.last_verdict == RowState::Bound)
                })
                .map(|(t, _)| t)
                .collect();
            if bound_tests.len() > 1 {
                use std::fmt::Write as _;
                let mut detail = format!("- claim_normalized: `{claim_norm}`\n  bindings:\n");
                for t in &bound_tests {
                    let _ = writeln!(detail, "    - {t}");
                }
                conflicts.push(detail);
            }
        }

        if !conflicts.is_empty() {
            let mut body = String::from(
                "# Merge conflicts\n\nMultiple shards bound the same claim to different tests. Resolve by editing the shard JSONs in `shards/` and re-running `merge-shards`.\n\n",
            );
            body.push_str(&conflicts.join("\n"));
            let conflicts_path = run_dir.join("CONFLICTS.md");
            fs::write(&conflicts_path, body.as_bytes())
                .with_context(|| format!("writing {}", conflicts_path.display()))?;
            eprintln!(
                "merge-shards: conflicts detected -- see {}",
                conflicts_path.display()
            );
            return Ok(1);
        }

        // Clean merge: build the final row set. Same claim_normalized + same test ->
        // single row with multi-source citations (de-duped).
        let mut cat = Catalog::load(catalog)?;

        for bucket in by_claim.values() {
            for (_test, rows) in bucket {
                if rows.is_empty() {
                    continue;
                }
                // Pick the first row as the prototype; collect every source citation.
                let mut prototype = rows[0].clone();
                let mut all_sources: Vec<SourceCite> = Vec::new();
                for r in rows {
                    for cite in r.source.as_slice() {
                        let already = all_sources.iter().any(|c| {
                            c.file == cite.file
                                && c.line_start == cite.line_start
                                && c.line_end == cite.line_end
                        });
                        if !already {
                            all_sources.push(cite);
                        }
                    }
                }
                // P78 MULTI-SOURCE-WATCH-01: compute per-source hashes
                // parallel to all_sources. Each cite hashed via
                // hash::source_hash; failures (missing file, unreadable
                // range) surface here at merge time rather than carrying
                // a stale hash forward into the catalog.
                let mut all_source_hashes: Vec<String> = Vec::with_capacity(all_sources.len());
                for cite in &all_sources {
                    let p = PathBuf::from(&cite.file);
                    let h = crate::hash::source_hash(&p, cite.line_start, cite.line_end)
                        .with_context(|| {
                            format!(
                                "merge-shards: hashing cite {}:{}-{} for row {}",
                                cite.file, cite.line_start, cite.line_end, prototype.id,
                            )
                        })?;
                    all_source_hashes.push(h);
                }
                let new_source = if all_sources.len() == 1 {
                    Source::Single(all_sources.into_iter().next().expect("len==1"))
                } else {
                    Source::Multi(all_sources)
                };
                prototype.set_source(new_source, all_source_hashes)?;
                // Upsert by id.
                if let Some(existing) = cat.row_mut(&prototype.id) {
                    *existing = prototype;
                } else {
                    cat.rows.push(prototype);
                }
            }
        }

        cat.recompute_summary();
        cat.save(catalog)?;

        // Write a MERGE.md summary.
        let merge_path = run_dir.join("MERGE.md");
        let mut summary = format!(
            "# Merge summary\n\n- shards processed: {}\n- catalog rows after merge: {}\n",
            shard_paths.len(),
            cat.rows.len(),
        );
        summary.push_str("- conflicts: 0\n");
        fs::write(&merge_path, summary.as_bytes())
            .with_context(|| format!("writing {}", merge_path.display()))?;
        Ok(0)
    }

    /// Hash drift walker. Re-computes `source_hash` + `test_body_hash` for
    /// every row from the live filesystem and updates `last_verdict`.
    /// NEVER modifies the stored hashes -- those refresh only via `bind`.
    #[allow(
        clippy::too_many_lines,
        reason = "Walker iterates every row through one state-machine arm; splitting hurts readability."
    )]
    pub fn walk(catalog: &Path) -> Result<i32> {
        let mut cat = Catalog::load(catalog)?;
        let mut blocking_lines: Vec<String> = Vec::new();

        for row in &mut cat.rows {
            // Skip already-retired rows.
            if row.last_verdict == RowState::RetireConfirmed {
                continue;
            }
            // Don't walk RETIRE_PROPOSED (it's already blocking until human acts).
            if row.last_verdict == RowState::RetireProposed {
                let cite_str = row
                    .source
                    .as_slice()
                    .first()
                    .map_or_else(String::new, |c| c.file.clone());
                blocking_lines.push(format!(
                    "docs-alignment: {} on {} -- run /reposix-quality-refresh {}",
                    row.last_verdict.as_str(),
                    cite_str,
                    cite_str,
                ));
                continue;
            }

            // P78 MULTI-SOURCE-WATCH-01: AND-compare per-source hashes.
            // Walker iterates every cite in source.as_slice() and compares
            // against source_hashes[i]. Any-index drift fires
            // STALE_DOCS_DRIFT; the drifted source's index + file path
            // surface in the diagnostic line for forensic clarity (mirrors
            // the existing drifted_indices pattern for tests below).
            //
            // Path-(b) closure: pre-P78 the walker only watched
            // source.as_slice()[0] (path-(a)); drift in non-first sources
            // of a Multi row was a false-negative window. The per-index
            // loop below closes that window.
            let cites = row.source.as_slice();
            let mut drifted_source_indices: Vec<usize> = Vec::new();
            let source_drift: Option<bool> = if row.source_hashes.is_empty() {
                // No hashes recorded yet (e.g. retire-proposed rows or
                // legacy rows without a stored hash). Skip drift compare.
                None
            } else if cites.len() != row.source_hashes.len() {
                // Parallel-array invariant violated. Catalog::load
                // backfill should prevent this; defend against
                // hand-edited catalogs by treating mismatched lengths as
                // drift.
                Some(true)
            } else {
                let mut any_drift = false;
                for (i, c) in cites.iter().enumerate() {
                    let stored = &row.source_hashes[i];
                    let p = PathBuf::from(&c.file);
                    let drifted = if p.exists() {
                        match hash::source_hash(&p, c.line_start, c.line_end) {
                            Ok(now) => &now != stored,
                            Err(_) => true,
                        }
                    } else {
                        true
                    };
                    if drifted {
                        any_drift = true;
                        drifted_source_indices.push(i);
                    }
                }
                Some(any_drift)
            };

            // Compute per-test drift state (W7 / v0.12.1). Each `tests[i]`
            // is hashed independently against `test_body_hashes[i]`. Three
            // outcomes per element:
            //   - drifted: file exists, fn parses, hash != stored
            //   - gone: file missing OR fn not found
            //   - clean: hash == stored
            // The row's verdict aggregates: any "gone" -> STALE_TEST_GONE,
            // else any "drifted" -> STALE_TEST_DRIFT. The drifted/gone index
            // sets are surfaced in the diagnostic line for forensic clarity.
            let mut drifted_indices: Vec<usize> = Vec::new();
            let mut gone_indices: Vec<usize> = Vec::new();
            let has_test_bindings = !row.tests.is_empty();
            if has_test_bindings && row.tests.len() == row.test_body_hashes.len() {
                for (i, t) in row.tests.iter().enumerate() {
                    match super::parse_test(t) {
                        Ok(test_ref) => {
                            if !test_ref.file().exists() {
                                gone_indices.push(i);
                                continue;
                            }
                            match test_ref.compute_hash() {
                                Ok(now) => {
                                    if now != row.test_body_hashes[i] {
                                        drifted_indices.push(i);
                                    }
                                }
                                Err(_) => gone_indices.push(i),
                            }
                        }
                        Err(_) => gone_indices.push(i),
                    }
                }
            }

            let any_test_gone = !gone_indices.is_empty();
            let any_test_drift = !drifted_indices.is_empty();

            let new_state = if any_test_gone {
                RowState::StaleTestGone
            } else if source_drift == Some(true) {
                RowState::StaleDocsDrift
            } else if any_test_drift {
                RowState::StaleTestDrift
            } else if row.last_verdict == RowState::MissingTest
                || row.last_verdict == RowState::TestMisaligned
            {
                // Walker doesn't transition out of MISSING_TEST / TEST_MISALIGNED.
                row.last_verdict
            } else if has_test_bindings {
                RowState::Bound
            } else {
                // No test bindings + no drift signal: preserve existing
                // verdict. New rows lacking tests should arrive as
                // MISSING_TEST via mark-missing-test, not via walk.
                row.last_verdict
            };
            row.last_verdict = new_state;

            if new_state.blocks_pre_push() {
                // For STALE_DOCS_DRIFT, surface the FIRST drifted source's
                // file path (P78 MULTI-SOURCE-WATCH-01) so the operator
                // sees which source drifted on Multi rows -- not just the
                // first source unconditionally. Falls back to source[0]
                // when no per-source drift is recorded (e.g. mismatched-
                // length rows that hit the catch-all Some(true)).
                let cite_str = if new_state == RowState::StaleDocsDrift
                    && !drifted_source_indices.is_empty()
                {
                    let first_drifted = drifted_source_indices[0];
                    cites
                        .get(first_drifted)
                        .map_or_else(String::new, |c| c.file.clone())
                } else {
                    row.source
                        .as_slice()
                        .first()
                        .map_or_else(String::new, |c| c.file.clone())
                };
                let detail = match new_state {
                    RowState::StaleDocsDrift if !drifted_source_indices.is_empty() => {
                        format!(" sources_drifted={drifted_source_indices:?}")
                    }
                    RowState::StaleTestDrift => {
                        format!(" drifted={drifted_indices:?}")
                    }
                    RowState::StaleTestGone => {
                        format!(" gone={gone_indices:?}")
                    }
                    _ => String::new(),
                };
                blocking_lines.push(format!(
                    "docs-alignment: {} row={}{} on {} -- run /reposix-quality-refresh {}",
                    new_state.as_str(),
                    row.id,
                    detail,
                    cite_str,
                    cite_str,
                ));
            }
        }

        cat.summary.last_walked = Some(now_iso());
        cat.recompute_summary();

        // Coverage metric (P66): populate global summary fields.
        // Walker NEVER auto-tunes coverage_floor (human-tuned via deliberate commit).
        let per_file = coverage::compute_per_file(&cat.rows);
        let (covered, total, cov_ratio) = coverage::compute_global(&per_file);
        cat.summary.lines_covered = covered;
        cat.summary.total_eligible_lines = total;
        cat.summary.coverage_ratio = cov_ratio;

        // Floor check (alignment_ratio < floor BLOCKs unless waiver active).
        let waiver_active = cat
            .summary
            .floor_waiver
            .as_ref()
            .is_some_and(|w| w.until.as_str() > now_iso().as_str());
        let floor_block = cat.summary.alignment_ratio + 1e-9 < cat.summary.floor && !waiver_active;
        if floor_block {
            blocking_lines.push(format!(
                "docs-alignment: alignment_ratio {:.4} below floor {:.4} -- run /reposix-quality-backfill OR ratchet floor explicitly",
                cat.summary.alignment_ratio, cat.summary.floor,
            ));
        }

        // Coverage floor BLOCK (P66) -- separate from alignment floor; both
        // can fire on the same walk. NO waiver semantics for coverage_floor;
        // it is human-tuned via deliberate commits.
        let coverage_block = cat.summary.coverage_ratio + 1e-9 < cat.summary.coverage_floor;
        if coverage_block {
            blocking_lines.push(format!(
                "docs-alignment: coverage_ratio {:.4} below coverage_floor {:.4} -- run /reposix-quality-backfill to widen extraction OR ratchet floor down via deliberate human commit",
                cat.summary.coverage_ratio, cat.summary.coverage_floor,
            ));
        }

        cat.save(catalog)?;

        if blocking_lines.is_empty() {
            Ok(0)
        } else {
            for line in &blocking_lines {
                eprintln!("{line}");
            }
            Ok(1)
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Single coherent procedure: load + compute counters + per-file table + json/text emit. Splitting hurts readability of a deterministic pipeline."
    )]
    pub fn status(catalog: &Path, json_mode: bool, top: usize, all: bool) -> Result<i32> {
        let cat = Catalog::load(catalog)?;

        // Per-file coverage is computed from the live row set + filesystem.
        // We do this in BOTH modes (json + table) so subagents can read the
        // gap-target view machine-readable.
        let per_file = coverage::compute_per_file(&cat.rows);

        // Multi-test row count (W7 / v0.12.1): how many rows bind to >=2
        // tests. Surfaces the new schema's reach without changing the
        // headline summary numbers.
        let multi_test_rows: u64 =
            u64::try_from(cat.rows.iter().filter(|r| r.tests.len() >= 2).count())
                .unwrap_or(u64::MAX);

        // next_action breakdown (W4 / v0.12.1 P68): counts per variant.
        // Emitted in fixed order so output is stable across runs.
        let mut na_write_test: u64 = 0;
        let mut na_fix_impl: u64 = 0;
        let mut na_update_doc: u64 = 0;
        let mut na_retire: u64 = 0;
        let mut na_bind_green: u64 = 0;
        for r in &cat.rows {
            match r.next_action {
                NextAction::WriteTest => na_write_test += 1,
                NextAction::FixImplThenBind => na_fix_impl += 1,
                NextAction::UpdateDoc => na_update_doc += 1,
                NextAction::RetireFeature => na_retire += 1,
                NextAction::BindGreen => na_bind_green += 1,
            }
        }

        if json_mode {
            // Emit { global: {...}, per_file: [...] } -- the per_file is
            // sorted ascending by ratio (worst-first) by compute_per_file.
            let payload = serde_json::json!({
                "global": &cat.summary,
                "multi_test_rows": multi_test_rows,
                "next_action_breakdown": {
                    "WRITE_TEST": na_write_test,
                    "FIX_IMPL_THEN_BIND": na_fix_impl,
                    "UPDATE_DOC": na_update_doc,
                    "RETIRE_FEATURE": na_retire,
                    "BIND_GREEN": na_bind_green,
                },
                "per_file": per_file.iter().map(|p| serde_json::json!({
                    "path": p.path.to_string_lossy(),
                    "total_lines": p.total_lines,
                    "covered_lines": p.covered_lines,
                    "ratio": p.ratio,
                    "row_count": p.row_count,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
            return Ok(0);
        }

        let s = &cat.summary;
        println!("== global ==");
        println!("  claims_total           {}", s.claims_total);
        println!("  claims_bound           {}", s.claims_bound);
        println!("  claims_missing_test    {}", s.claims_missing_test);
        println!("  claims_retire_proposed {}", s.claims_retire_proposed);
        println!("  claims_retired         {}", s.claims_retired);
        println!(
            "  alignment_ratio        {:.4}   ({} bound / {} non-retired)",
            s.alignment_ratio,
            s.claims_bound,
            s.claims_total.saturating_sub(s.claims_retired),
        );
        println!("  alignment_floor        {:.4}", s.floor);
        println!(
            "  coverage_ratio         {:.4}   ({} covered / {} total eligible lines)",
            s.coverage_ratio, s.lines_covered, s.total_eligible_lines,
        );
        println!("  coverage_floor         {:.4}", s.coverage_floor);
        println!("  multi_test_rows        {multi_test_rows}");
        println!("  trend_30d              {}", s.trend_30d);
        if let Some(lw) = &s.last_walked {
            println!("  last_walked            {lw}");
        }
        println!();
        println!("== next_action breakdown ==");
        println!("  WRITE_TEST          : {na_write_test}");
        println!("  FIX_IMPL_THEN_BIND  : {na_fix_impl}");
        println!("  UPDATE_DOC          : {na_update_doc}");
        println!("  RETIRE_FEATURE      : {na_retire}");
        println!("  BIND_GREEN          : {na_bind_green}");
        println!();
        println!("== per-file (worst-covered first) ==");
        let limit = if all {
            per_file.len()
        } else {
            top.min(per_file.len())
        };
        if limit == 0 {
            println!("  (no eligible files)");
        } else {
            // Header. Field widths: path 56, total 6, covered 7, ratio 7, rows 6.
            println!(
                "  {:<56}  {:>6}  {:>7}  {:>7}  {:>6}",
                "file", "total", "covered", "ratio", "rows",
            );
            println!("  {}", "-".repeat(56 + 2 + 6 + 2 + 7 + 2 + 7 + 2 + 6));
            for p in per_file.iter().take(limit) {
                let path_disp = p.path.to_string_lossy();
                let trimmed: String = if path_disp.len() > 56 {
                    format!("...{}", &path_disp[path_disp.len() - 53..])
                } else {
                    path_disp.to_string()
                };
                let zero_hint = if p.row_count == 0 && p.total_lines > 50 {
                    "   <-- ZERO ROWS"
                } else {
                    ""
                };
                println!(
                    "  {:<56}  {:>6}  {:>7}  {:>7.3}  {:>6}{}",
                    trimmed, p.total_lines, p.covered_lines, p.ratio, p.row_count, zero_hint,
                );
            }
            if !all && per_file.len() > limit {
                println!("  ... ({} more; use --all)", per_file.len() - limit);
            }
        }
        Ok(0)
    }
}

/// Parse a source citation `<file>:<line_start>-<line_end>` into its parts.
pub(crate) fn parse_source(s: &str) -> Result<(PathBuf, usize, usize)> {
    let (file, range) = s
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("source `{s}` is not `<file>:<line_start>-<line_end>`"))?;
    let (lstart, lend) = range
        .split_once('-')
        .ok_or_else(|| anyhow!("source range `{range}` is not `<line_start>-<line_end>`"))?;
    let lstart: usize = lstart
        .parse()
        .map_err(|_| anyhow!("line_start `{lstart}` is not a positive integer"))?;
    let lend: usize = lend
        .parse()
        .map_err(|_| anyhow!("line_end `{lend}` is not a positive integer"))?;
    Ok((PathBuf::from(file), lstart, lend))
}

/// Parsed `--test` argument: either a Rust `<file>::<fn>` citation or a
/// non-Rust verifier path (shell, Python, YAML, ...).
///
/// `<file>::<fn>` form: the file part must end in `.rs` and `<fn>` must be
/// non-empty -- hashed via [`crate::hash::test_body_hash`] (syn parser,
/// fn-body token-stream sha256).
///
/// `<file>` form (no `::`, OR a `::` is present but the prefix does not end
/// in `.rs`): hashed via [`crate::hash::file_hash`] (full-file sha256). Lets
/// rows bind to shell-script / Python / YAML verifiers without requiring a
/// Rust test fn wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TestRef {
    RustFn { file: PathBuf, fn_name: String },
    File { file: PathBuf },
}

impl TestRef {
    pub(crate) fn file(&self) -> &Path {
        match self {
            TestRef::RustFn { file, .. } | TestRef::File { file } => file,
        }
    }

    /// Compute the hash for this test reference.
    ///
    /// `RustFn` -> [`crate::hash::test_body_hash`]; `File` -> [`crate::hash::file_hash`].
    ///
    /// # Errors
    ///
    /// Propagates errors from the underlying hash fn (file missing, parse
    /// failure, fn not found, etc.).
    pub(crate) fn compute_hash(&self) -> Result<String> {
        match self {
            TestRef::RustFn { file, fn_name } => crate::hash::test_body_hash(file, fn_name),
            TestRef::File { file } => crate::hash::file_hash(file),
        }
    }
}

/// Parse a test citation. Accepts two forms:
///
/// - `<file>::<fn>` where `<file>` ends in `.rs` -> [`TestRef::RustFn`].
/// - `<file>` (no `::`, OR `::` with non-Rust prefix) -> [`TestRef::File`].
///
/// Rationale: ~17-25 catalog rows have shell-script / Python / YAML verifiers
/// (mkdocs-strict, latency-bench, check-badges, banned-words). The `::` form
/// stays Rust-only so the existing fn-body-hash semantics are preserved.
///
/// # Errors
///
/// Errors if the input parses as `<file.rs>::` with an empty fn name (the
/// only invalid form). Other shapes fall through to [`TestRef::File`].
pub(crate) fn parse_test(s: &str) -> Result<TestRef> {
    if let Some((file, fn_name)) = s.rsplit_once("::") {
        let is_rust = Path::new(file)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
        if is_rust {
            if fn_name.is_empty() {
                return Err(anyhow!("test fn name is empty in `{s}`"));
            }
            return Ok(TestRef::RustFn {
                file: PathBuf::from(file),
                fn_name: fn_name.to_string(),
            });
        }
    }
    Ok(TestRef::File {
        file: PathBuf::from(s),
    })
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn source_parses() {
        let (f, a, b) = parse_source("docs/index.md:10-12").unwrap();
        assert_eq!(f.to_string_lossy(), "docs/index.md");
        assert_eq!((a, b), (10, 12));
    }

    #[test]
    fn source_rejects_bad_shape() {
        assert!(parse_source("foo").is_err());
        assert!(parse_source("foo:10").is_err());
        assert!(parse_source("foo:abc-def").is_err());
    }

    #[test]
    fn parse_test_accepts_rust_file_fn() {
        let parsed = parse_test("crates/foo/src/lib.rs::my_test").unwrap();
        match parsed {
            TestRef::RustFn { file, fn_name } => {
                assert_eq!(file.to_string_lossy(), "crates/foo/src/lib.rs");
                assert_eq!(fn_name, "my_test");
            }
            TestRef::File { .. } => panic!("expected RustFn, got File"),
        }
    }

    #[test]
    fn parse_test_accepts_shell_script() {
        let parsed = parse_test("scripts/check-foo.sh").unwrap();
        match parsed {
            TestRef::File { file } => {
                assert_eq!(file.to_string_lossy(), "scripts/check-foo.sh");
            }
            TestRef::RustFn { .. } => panic!("expected File, got RustFn"),
        }
    }

    #[test]
    fn parse_test_accepts_python_script() {
        let parsed = parse_test("quality/gates/perf/latency-bench.py").unwrap();
        match parsed {
            TestRef::File { file } => {
                assert_eq!(
                    file.to_string_lossy(),
                    "quality/gates/perf/latency-bench.py"
                );
            }
            TestRef::RustFn { .. } => panic!("expected File, got RustFn"),
        }
    }

    #[test]
    fn parse_test_rejects_rust_with_empty_fn() {
        // .rs file with `::` and empty fn name -- the only invalid form.
        assert!(parse_test("crates/foo.rs::").is_err());
    }

    #[test]
    fn parse_test_path_only_accepts_yaml_toml_json_md() {
        // Sanity: any non-`::` form parses as File, regardless of extension.
        for path in &[
            "config.yaml",
            "Cargo.toml",
            "data.json",
            "docs/index.md",
            "scripts/no-extension",
        ] {
            match parse_test(path).unwrap() {
                TestRef::File { file } => assert_eq!(file.to_string_lossy(), *path),
                TestRef::RustFn { .. } => panic!("{path} should parse as File"),
            }
        }
    }

    #[test]
    fn parse_test_non_rust_with_double_colon_is_file() {
        // `::` present but the prefix is not .rs -- treat the whole thing
        // as a file path. Edge case; in practice file paths don't contain
        // `::`, but the parser is defensive.
        let parsed = parse_test("scripts/foo.sh::bar").unwrap();
        match parsed {
            TestRef::File { file } => {
                assert_eq!(file.to_string_lossy(), "scripts/foo.sh::bar");
            }
            TestRef::RustFn { .. } => {
                panic!("non-.rs with `::` should fall through to File")
            }
        }
    }
}
