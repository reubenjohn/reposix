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
use std::sync::Arc;

use anyhow::{Context, Result};
use reposix_cache::Cache;
use reposix_core::backend::{sim::SimBackend, BackendConnector, DeleteReason};
use reposix_core::{parse_remote_url, sanitize, ServerMetadata, Tainted};
use tokio::runtime::Runtime;

mod diff;
mod fast_import;
mod pktline;
mod protocol;
mod stateless_connect;

use crate::diff::{plan, PlanError, PlannedAction};
use crate::fast_import::{emit_import_stream, parse_export_stream};
use crate::protocol::Protocol;
use crate::stateless_connect::handle_stateless_connect;

/// Deferred-exit flag — set by the export path on push refusal. We finish
/// the protocol exchange cleanly (so git doesn't see a torn pipe) and bail
/// after the dispatch loop returns.
struct State {
    rt: Runtime,
    backend: Arc<dyn BackendConnector>,
    /// Short slug used as the cache-key prefix in
    /// `<cache-root>/reposix/<backend_name>-<project>.git`. For the
    /// v0.9.0 sim-only phase this is hardcoded to `"sim"`; Phase 35
    /// will derive it from the parsed remote URL for real backends.
    backend_name: String,
    project: String,
    push_failed: bool,
    /// Monotonic counter: total `want ` lines observed across every
    /// RPC turn handled by the `stateless-connect` tunnel. Wired in
    /// Phase 32 for instrumentation; Phase 34 will enforce a limit.
    #[allow(dead_code)]
    last_fetch_want_count: u32,
    /// Backing bare-repo cache. Lazily initialised inside
    /// `handle_stateless_connect` (the capabilities/list/import/export
    /// verbs don't need it), then cached for the remainder of the
    /// helper's lifetime.
    cache: Option<Cache>,
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

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::with_agent_suffix(spec.origin, Some("remote"))?);
    // v0.9.0 sim-only: hardcode "sim" as the cache backend slug.
    // Phase 35 will derive this from the parsed URL scheme/host for
    // real backends (github, confluence, jira).
    let backend_name = "sim".to_owned();
    let mut state = State {
        rt,
        backend,
        backend_name,
        project: spec.project.as_str().to_owned(),
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
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
                // Hybrid advertisement for v0.9.0 architecture pivot:
                //   - `import`/`export` preserved for the push path and
                //     the v0.8 `import` capability (deprecated — one
                //     release cycle; Phase 36 removes).
                //   - `stateless-connect` is the v0.9 read path,
                //     tunnelling protocol-v2 fetch traffic to the
                //     Phase 31 cache via `git upload-pack --stateless-rpc`.
                //   - `object-format=sha1` is required by protocol-v2;
                //     without it, git 2.34+ warns and falls back.
                // Per `transport-helper.c::process_connect_service`,
                // `stateless-connect` is only dispatched for
                // `git-upload-pack` and `git-upload-archive` — push
                // (`git-receive-pack`) falls through to `export`, so
                // both capabilities coexist.
                proto.send_line("import")?;
                proto.send_line("export")?;
                proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
                proto.send_line("stateless-connect")?;
                proto.send_line("object-format=sha1")?;
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
            "stateless-connect" => {
                // Service name is the second whitespace-separated
                // field. An unknown or empty service becomes a clean
                // error inside the handler.
                let service = parts.next().unwrap_or("").trim();
                let service_owned = service.to_owned();
                ensure_cache(&mut state)?;
                let cache_ref = state.cache.as_ref().expect("cache initialised");
                handle_stateless_connect(&mut proto, &state.rt, cache_ref, &service_owned)?;
                // Per the helper-protocol spec, stateless-connect is
                // always the last verb of a helper invocation: git
                // takes over stdin/stdout for the duration of the
                // protocol-v2 session and closes the stream on EOF.
                return Ok(!state.push_failed);
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

/// Lazily open the backing `reposix_cache::Cache` for the helper's
/// `(backend, project)` tuple. Called once on first need (currently
/// only from the `stateless-connect` dispatch arm; the import/export
/// paths do not require the bare repo). The cache is then kept on
/// `State` for the remainder of the helper's lifetime.
fn ensure_cache(state: &mut State) -> Result<()> {
    if state.cache.is_some() {
        return Ok(());
    }
    let cache = Cache::open(state.backend.clone(), &state.backend_name, &state.project)
        .context("open reposix-cache")?;
    state.cache = Some(cache);
    Ok(())
}

/// Emit a clean protocol error line on stdout + a diagnostic on stderr,
/// set the deferred-exit flag, and return `Ok(())` so the dispatch loop
/// can exit with a well-defined non-zero status instead of torn-piping git.
///
/// Used in import/export paths where a backend failure (`list_issues`
/// 5xx, timeout, allowlist rejection) would otherwise bubble up via `?`
/// and close stdout mid-protocol — leaving git to see a confusing
/// "fast-import failed" error with no actionable context.
fn fail_push<R: std::io::Read, W: std::io::Write>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
    kind: &str,
    detail: &str,
) -> std::io::Result<()> {
    diag(&format!("error: {detail}"));
    proto.send_line(&format!("error refs/heads/main {kind}"))?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    Ok(())
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
    let issues = match state.rt.block_on(state.backend.list_issues(&state.project)) {
        Ok(v) => v,
        Err(e) => {
            return fail_push(
                proto,
                state,
                "backend-unreachable",
                &format!("cannot list issues for import: {e:#}"),
            )
            .map_err(Into::into);
        }
    };
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
    let parse_result = parse_export_stream(&mut buffered);
    drop(buffered);
    let parsed = match parse_result {
        Ok(v) => v,
        Err(e) => {
            return fail_push(
                proto,
                state,
                "parse-error",
                &format!("parse export stream: {e:#}"),
            )
            .map_err(Into::into);
        }
    };

    let prior = match state.rt.block_on(state.backend.list_issues(&state.project)) {
        Ok(v) => v,
        Err(e) => {
            return fail_push(
                proto,
                state,
                "backend-unreachable",
                &format!("cannot list prior issues: {e:#}"),
            )
            .map_err(Into::into);
        }
    };
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
                .block_on(state.backend.create_issue(&state.project, untainted))
                .context("create issue")?;
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
                .block_on(state.backend.update_issue(
                    &state.project,
                    id,
                    untainted,
                    Some(prior_version),
                ))
                .with_context(|| format!("patch issue {}", id.0))?;
            Ok(())
        }
        PlannedAction::Delete { id, .. } => {
            state
                .rt
                .block_on(state.backend.delete_or_close(
                    &state.project,
                    id,
                    DeleteReason::Abandoned,
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
            // Pull one line at a time as RAW BYTES. read_raw_line strips the
            // trailing `\n` but preserves any `\r` and never UTF-8-decodes,
            // so blob bodies containing CRLF or non-UTF-8 bytes round-trip
            // byte-for-byte. We re-add the `\n` for the downstream parser.
            self.buf.clear();
            self.pos = 0;
            match self.proto.read_raw_line()? {
                Some(line) => {
                    self.buf.extend_from_slice(&line);
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
