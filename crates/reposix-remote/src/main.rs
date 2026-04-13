//! `git-remote-reposix` — git remote helper.
//!
//! Invoked by git when a remote URL begins with `reposix::`. Speaks the git
//! remote-helper protocol on stdin/stdout. Stderr carries diagnostic
//! traffic only — `println!` is mechanically banned outside [`protocol`]
//! by `#![deny(clippy::print_stdout)]` so accidental future writes are a
//! compile error.

#![forbid(unsafe_code)]
#![deny(clippy::print_stdout)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::io::{stdin, stdout, BufReader};
use std::process::ExitCode;

use anyhow::{Context, Result};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{parse_remote_url, sanitize, ServerMetadata, Tainted};
use tokio::runtime::Runtime;

mod client;
mod diff;
mod fast_import;
mod protocol;

use crate::client as api;
use crate::diff::{plan, PlanError, PlannedAction};
use crate::fast_import::{emit_import_stream, parse_export_stream};
use crate::protocol::Protocol;

/// Deferred-exit flag — set by the export path on push refusal. We finish
/// the protocol exchange cleanly (so git doesn't see a torn pipe) and bail
/// after the dispatch loop returns.
struct State {
    rt: Runtime,
    http: HttpClient,
    origin: String,
    project: String,
    agent: String,
    push_failed: bool,
}

#[allow(clippy::print_stderr)]
fn diag(msg: &str) {
    eprintln!("{msg}");
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    match real_main() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            diag(&format!("git-remote-reposix: {e:#}"));
            ExitCode::from(2)
        }
    }
}

fn real_main() -> Result<bool> {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 3 {
        anyhow::bail!("usage: git-remote-reposix <alias> <url>");
    }
    let url = &argv[2];
    let spec = parse_remote_url(url).context("parse remote url")?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    let http = client(ClientOpts::default()).context("build http client")?;

    let agent = format!("git-remote-reposix-{}", std::process::id());
    let mut state = State {
        rt,
        http,
        origin: spec.origin.clone(),
        project: spec.project.as_str().to_owned(),
        agent,
        push_failed: false,
    };

    let stdin_handle = stdin();
    let stdout_handle = stdout();
    let mut proto = Protocol::new(stdin_handle.lock(), stdout_handle.lock());

    while let Some(line) = proto.read_line()? {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let cmd = parts.next().unwrap_or("");
        match cmd {
            "capabilities" => {
                proto.send_line("import")?;
                proto.send_line("export")?;
                proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
                proto.send_blank()?;
                proto.flush()?;
            }
            "option" => {
                proto.send_line("unsupported")?;
                proto.flush()?;
            }
            "list" | "list for-push" => {
                proto.send_line("? refs/heads/main")?;
                proto.send_line("@refs/heads/main HEAD")?;
                proto.send_blank()?;
                proto.flush()?;
            }
            "import" => {
                handle_import_batch(&mut state, &mut proto, &line)?;
            }
            "export" => {
                handle_export(&mut state, &mut proto)?;
            }
            other => {
                diag(&format!("git-remote-reposix: unknown command: {other}"));
                break;
            }
        }
    }
    proto.flush()?;
    Ok(!state.push_failed)
}

fn handle_import_batch<R: std::io::Read, W: std::io::Write>(
    state: &mut State,
    proto: &mut Protocol<R, W>,
    first_line: &str,
) -> Result<()> {
    // The first line is e.g. `import refs/heads/main`. Subsequent
    // import-batch members arrive as additional `import refs/heads/...`
    // lines until a blank terminator.
    let _ = first_line; // we only support one ref for v0.1
    loop {
        let next = proto.read_line()?;
        match next.as_deref() {
            Some("") | None => break,
            Some(s) if s.starts_with("import ") => {}
            Some(other) => {
                diag(&format!(
                    "git-remote-reposix: unexpected line in import batch: {other}"
                ));
                break;
            }
        }
    }
    let issues = state
        .rt
        .block_on(api::list_issues(
            &state.http,
            &state.origin,
            &state.project,
            &state.agent,
        ))
        .context("list issues for import")?;
    // Emit fast-import stream over stdout via the protocol writer.
    let mut sink: Vec<u8> = Vec::with_capacity(1024 + issues.len() * 256);
    emit_import_stream(&mut sink, &issues)?;
    proto.send_raw(&sink)?;
    proto.flush()?;
    Ok(())
}

fn handle_export<R: std::io::Read, W: std::io::Write>(
    state: &mut State,
    proto: &mut Protocol<R, W>,
) -> Result<()> {
    // The export verb has no arguments — the next thing on stdin is the
    // fast-export stream from git, terminated by `done`.
    let mut buffered = BufReader::new(ProtoReader::new(proto));
    let parsed = parse_export_stream(&mut buffered).context("parse export stream")?;
    drop(buffered);

    let prior = state
        .rt
        .block_on(api::list_issues(
            &state.http,
            &state.origin,
            &state.project,
            &state.agent,
        ))
        .context("list prior issues for export")?;
    let actions = match plan(&prior, &parsed) {
        Ok(a) => a,
        Err(PlanError::BulkDeleteRefused {
            count, limit, tag, ..
        }) => {
            diag(&format!(
                "error: refusing to push (would delete {count} issues; cap is {limit}; commit message tag '{tag}' overrides)"
            ));
            proto.send_line("error refs/heads/main bulk-delete")?;
            proto.send_blank()?;
            proto.flush()?;
            state.push_failed = true;
            return Ok(());
        }
        Err(PlanError::InvalidBlob { path, source }) => {
            diag(&format!(
                "error: invalid issue at {path}: {source}; refusing push"
            ));
            proto.send_line(&format!("error refs/heads/main invalid-blob:{path}"))?;
            proto.send_blank()?;
            proto.flush()?;
            state.push_failed = true;
            return Ok(());
        }
    };

    // Execute. Order = creates → updates → deletes (per diff::plan).
    let mut any_failure = false;
    for action in actions {
        match execute_action(state, action) {
            Ok(()) => {}
            Err(e) => {
                diag(&format!("error: {e:#}"));
                any_failure = true;
            }
        }
    }
    if any_failure {
        proto.send_line("error refs/heads/main some-actions-failed")?;
        proto.send_blank()?;
        proto.flush()?;
        state.push_failed = true;
    } else {
        proto.send_line("ok refs/heads/main")?;
        proto.send_blank()?;
        proto.flush()?;
    }
    Ok(())
}

fn execute_action(state: &mut State, action: PlannedAction) -> Result<()> {
    match action {
        PlannedAction::Create(issue) => {
            let now = issue.created_at;
            let meta = ServerMetadata {
                id: issue.id,
                created_at: now,
                updated_at: now,
                version: 0,
            };
            let untainted = sanitize(Tainted::new(issue), meta);
            let _new = state
                .rt
                .block_on(api::post_issue(
                    &state.http,
                    &state.origin,
                    &state.project,
                    untainted,
                    &state.agent,
                ))
                .context("post issue")?;
            Ok(())
        }
        PlannedAction::Update {
            id,
            prior_version,
            new,
        } => {
            let meta = ServerMetadata {
                id,
                created_at: new.created_at,
                updated_at: new.updated_at,
                version: prior_version,
            };
            let untainted = sanitize(Tainted::new(new), meta);
            state
                .rt
                .block_on(api::patch_issue(
                    &state.http,
                    &state.origin,
                    &state.project,
                    id,
                    prior_version,
                    untainted,
                    &state.agent,
                ))
                .with_context(|| format!("patch issue {}", id.0))?;
            Ok(())
        }
        PlannedAction::Delete { id, .. } => {
            state
                .rt
                .block_on(api::delete_issue(
                    &state.http,
                    &state.origin,
                    &state.project,
                    id,
                    &state.agent,
                ))
                .with_context(|| format!("delete issue {}", id.0))?;
            Ok(())
        }
    }
}

/// Adapter so `BufReader` can pull from the same underlying stdin we own
/// inside `Protocol`. We provide a one-line-at-a-time bridge — the
/// `parse_export_stream` parser uses `read_line` and `read_exact` only.
struct ProtoReader<'a, R: std::io::Read, W: std::io::Write> {
    proto: &'a mut Protocol<R, W>,
    buf: Vec<u8>,
    pos: usize,
}

impl<'a, R: std::io::Read, W: std::io::Write> ProtoReader<'a, R, W> {
    fn new(proto: &'a mut Protocol<R, W>) -> Self {
        Self {
            proto,
            buf: Vec::new(),
            pos: 0,
        }
    }
}

impl<R: std::io::Read, W: std::io::Write> std::io::Read for ProtoReader<'_, R, W> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.buf.len() {
            // Pull one line at a time. read_line strips the trailing \n,
            // so we re-add it for the downstream parser.
            self.buf.clear();
            self.pos = 0;
            match self.proto.read_line()? {
                Some(line) => {
                    self.buf.extend_from_slice(line.as_bytes());
                    self.buf.push(b'\n');
                }
                None => return Ok(0),
            }
        }
        let avail = &self.buf[self.pos..];
        let n = avail.len().min(out.len());
        out[..n].copy_from_slice(&avail[..n]);
        self.pos += n;
        Ok(n)
    }
}
