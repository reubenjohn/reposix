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
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::{sanitize, ServerMetadata, Tainted};
use tokio::runtime::Runtime;

mod bus_handler;
mod bus_url;
mod diff;
mod fast_import;
mod mirror_egress;
mod pktline;
mod precheck;
mod protocol;
mod stateless_connect;
mod write_loop;

use crate::diff::PlannedAction;
use crate::fast_import::{emit_import_stream, parse_export_stream};
use crate::protocol::Protocol;
use crate::stateless_connect::{handle_stateless_connect, StatelessConnectOutcome};
use reposix_remote::backend_dispatch::{self, instantiate};

/// Deferred-exit flag — set by the export path on push refusal. We finish
/// the protocol exchange cleanly (so git doesn't see a torn pipe) and bail
/// after the dispatch loop returns.
///
/// `pub(crate)` (with `pub(crate)` on `rt`/`backend`/`project`/`cache`) so
/// the sibling [`crate::precheck`] module can access these fields without
/// reaching into the binary root via the (invalid) `crate::main::*` path.
/// The other fields stay private — the precheck does not consume them.
pub(crate) struct State {
    pub(crate) rt: Runtime,
    pub(crate) backend: Arc<dyn BackendConnector>,
    /// Short slug used as the cache-key prefix in
    /// `<cache-root>/reposix/<backend_name>-<project>.git`. Set from
    /// [`backend_dispatch::BackendKind::slug`] by the URL-scheme
    /// dispatcher in [`backend_dispatch`] (closes the v0.9.0 Phase 32 carry-forward
    /// where every backend wedged onto the `"sim"` cache prefix).
    /// `pub(crate)` so [`crate::bus_handler`] can compose diagnostic
    /// lines naming the `SoT` (e.g. `<sot> has N change(s)`).
    pub(crate) backend_name: String,
    /// Project identifier passed to [`BackendConnector`] methods —
    /// `demo` for sim, `owner/repo` for GitHub, `TokenWorld` for
    /// Confluence, `TEST` for JIRA. This is the SINGLE identity
    /// (S-260707-gh404): the RAW slug reaches the backend verbatim, and
    /// [`reposix_cache::Cache::open`] sanitizes it to the flat
    /// `<backend>-<owner-repo>.git` cache dir only at path derivation.
    pub(crate) project: String,
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
    pub(crate) cache: Option<Cache>,
    /// Bus-mode mirror URL (DVCS-BUS-URL-01). `Some(url)` when the
    /// helper was invoked with a `reposix::<sot>?mirror=<url>` URL
    /// per Q3.3; `None` for single-backend `reposix::<sot>` URLs.
    /// The capabilities arm gates `stateless-connect` on
    /// `mirror_url.is_none()` (DVCS-BUS-FETCH-01 / Q3.4); the export
    /// arm dispatches to `bus_handler::handle_bus_export` when
    /// `Some` and to `handle_export` when `None`.
    pub(crate) mirror_url: Option<String>,
}

#[allow(clippy::print_stderr)]
pub(crate) fn diag(msg: &str) {
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
    // URL-scheme dispatch (Phase 36-followup, closes Phase 32 carry-forward):
    // identify which backend to instantiate from the remote URL, then
    // build the matching BackendConnector. Credential errors surface
    // here with a doc link to docs/reference/testing-targets.md.
    //
    // P82+ (DVCS-BUS-URL-01 / Q3.3): `bus_url::parse` recognizes
    // `reposix::<sot>?mirror=<mirror>` and dispatches to either
    // `Route::Single(parsed)` (single-backend, existing flow) or
    // `Route::Bus { sot, mirror_url }` (bus mode — `state.mirror_url`
    // is `Some(url)` and `bus_handler::handle_bus_export` runs on
    // the export verb instead of `handle_export`).
    let route = bus_url::parse(url).context("parse remote url")?;
    let (parsed, mirror_url_opt): (backend_dispatch::ParsedRemote, Option<String>) = match route {
        bus_url::Route::Single(p) => (p, None),
        bus_url::Route::Bus { sot, mirror_url } => (sot, Some(mirror_url)),
    };
    let backend = instantiate(&parsed).context("instantiate backend")?;
    let backend_name = parsed.kind.slug().to_owned();
    // S-260707-gh404: keep the RAW project slug (`owner/repo` for GitHub)
    // as the single identity. It reaches the `BackendConnector` verbatim
    // (so `GET /repos/owner/repo/issues` 200s), and `Cache::open` sanitizes
    // it to the flat `owner-repo` dir ONLY at on-disk path derivation.
    let project = parsed.project;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;

    let mut state = State {
        rt,
        backend,
        backend_name,
        project,
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
        mirror_url: mirror_url_opt,
    };

    let stdin_handle = stdin();
    let stdout_handle = stdout();
    let mut proto = Protocol::new(stdin_handle.lock(), stdout_handle.lock());

    while let Some(line) = proto.read_line()? {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() {
            continue;
        }
        tracing::debug!(cmd = %trimmed, "git-remote-reposix: verb");
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
                //
                // P82+ (DVCS-BUS-FETCH-01 / Q3.4): bus URLs are
                // PUSH-only. We omit `stateless-connect` for bus URLs
                // so fetch falls through to the single-backend code
                // path; single-backend URLs continue to advertise it.
                proto.send_line("import")?;
                proto.send_line("export")?;
                proto.send_line("refspec refs/heads/*:refs/reposix/origin/*")?;
                if state.mirror_url.is_none() {
                    proto.send_line("stateless-connect")?;
                }
                proto.send_line("object-format=sha1")?;
                proto.send_blank()?;
                proto.flush()?;
            }
            "option" => {
                // git-remote-helpers(7): a helper that advertises
                // `object-format` MUST accept `option object-format
                // {true|<algo>}` with `ok`. git 2.43.x sends this option
                // BEFORE the push `list`/`export` verbs and treats an
                // `unsupported` reply as FATAL — the real, version-windowed
                // cause of the P94 D2 git-2.43 single-backend-push failure
                // (exit 128, no export attempt). git 2.54 skips the
                // negotiation entirely (the advertised `object-format=sha1`
                // capability suffices), which is why `>= 2.34` did not
                // protect and CI (2.54) never saw it. reposix's cache is
                // sha1-only, so accept sha1/true (and the bare form git
                // 2.43 actually sends) and reject any other algorithm.
                // Every other option stays `unsupported` — git treats that
                // as "ignore this option", the intended behaviour.
                let opt = parts.next().unwrap_or("").trim();
                if let Some(rest) = opt.strip_prefix("object-format") {
                    let algo = rest.trim();
                    if algo.is_empty() || algo == "true" || algo == "sha1" {
                        proto.send_line("ok")?;
                    } else {
                        proto.send_line(&format!(
                            "error reposix cache is sha1-only, cannot honor object-format {algo}"
                        ))?;
                    }
                } else {
                    proto.send_line("unsupported")?;
                }
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
                if state.mirror_url.is_some() {
                    bus_handler::handle_bus_export(&mut state, &mut proto)?;
                } else {
                    handle_export(&mut state, &mut proto)?;
                }
            }
            "stateless-connect" => {
                // Service name is the second whitespace-separated field.
                // `git-upload-pack` (fetch) is served in-band; any other
                // service (push probes `git-receive-pack` first under
                // git >= 2.43) gets the `fallback` sentinel and the loop
                // resumes so the follow-up `export` verb is handled here.
                let service = parts.next().unwrap_or("").trim();
                let service_owned = service.to_owned();
                ensure_cache(&mut state)?;
                let cache_ref = state.cache.as_ref().expect("cache initialised");
                match handle_stateless_connect(&mut proto, &state.rt, cache_ref, &service_owned)? {
                    // upload-pack served: git owned stdin/stdout for the
                    // protocol-v2 session and closed the stream on EOF.
                    // There are no further verbs — exit cleanly.
                    StatelessConnectOutcome::TookOver => return Ok(!state.push_failed),
                    // Non-upload-pack service fell back to `fallback`; git
                    // will retry via another capability. Resume the verb
                    // loop on the SAME live helper process (do NOT exit —
                    // exiting here would tear the pipe git needs for the
                    // follow-up `export`).
                    StatelessConnectOutcome::FellBack => {}
                }
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
///
/// `pub(crate)` so the sibling [`crate::bus_handler`] module can
/// lazy-open the cache during PRECHECK B without round-tripping
/// through `handle_export`'s body.
pub(crate) fn ensure_cache(state: &mut State) -> Result<()> {
    if state.cache.is_some() {
        return Ok(());
    }
    let cache = Cache::open(state.backend.clone(), &state.backend_name, &state.project)
        .context("open reposix-cache")?;
    // Phase 36-followup: best-effort audit row recording which backend
    // served this session. Useful forensics signal — pre-dispatch the
    // helper hardcoded `"sim"` and there was no way to trace which
    // backend a given fetch hit.
    cache.log_helper_backend_instantiated(&state.project);
    state.cache = Some(cache);
    Ok(())
}

/// Emit a clean protocol error line on stdout + a diagnostic on stderr,
/// set the deferred-exit flag, and return `Ok(())` so the dispatch loop
/// can exit with a well-defined non-zero status instead of torn-piping git.
///
/// Used in import/export paths where a backend failure (`list_records`
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
    let issues = match state
        .rt
        .block_on(state.backend.list_records(&state.project))
    {
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
    // Emit fast-import stream over stdout via the protocol writer, using
    // the backend's canonical record bucket (issues/ vs pages/).
    let bucket = reposix_core::path::bucket_for_backend(&state.backend_name);
    let mut sink: Vec<u8> = Vec::with_capacity(1024 + issues.len() * 256);
    emit_import_stream(&mut sink, &issues, bucket)?;
    proto.send_raw(&sink)?;
    proto.flush()?;
    Ok(())
}

fn handle_export<R: std::io::Read, W: std::io::Write>(
    state: &mut State,
    proto: &mut Protocol<R, W>,
) -> Result<()> {
    // Lazy-open the cache for audit-row writes. Best-effort: if the cache
    // can't be opened (misconfigured cache root, permission error), log
    // a WARN and continue — the push path still works, only audit rows
    // are dropped.
    if let Err(e) = ensure_cache(state) {
        tracing::warn!("cache unavailable for push audit: {e:#}");
    }
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_started("refs/heads/main");
    }

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

    // Apply writes via shared write_loop (T02 lift). On reject outcomes,
    // `apply_writes` has already emitted protocol error + audit rows;
    // we just set push_failed and return. On SotOk, we (the
    // single-backend caller) write the synced-at ref + mirror_sync_written
    // audit row + token-cost row + ok ack — D-01 RATIFIED.
    let outcome = write_loop::apply_writes(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.backend_name,
        &state.project,
        &state.rt,
        proto,
        &parsed, // borrow per B1 — apply_writes takes &ParsedExport (matches precheck/plan shape)
    )?;

    let write_loop::WriteOutcome::SotOk { sot_sha, .. } = outcome else {
        state.push_failed = true;
        return Ok(());
    };

    // Single-backend caller writes synced-at + mirror_sync_written +
    // log_token_cost unconditionally on SotOk (D-01).
    if let Some(cache) = state.cache.as_ref() {
        if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, chrono::Utc::now()) {
            tracing::warn!("write_mirror_synced_at failed: {e:#}");
        }
        // OP-3 unconditional: audit-row write fires whether or not the
        // ref writes succeeded. Records the attempt's SHA (or empty
        // string if SHA derivation failed in apply_writes).
        let oid_hex = sot_sha
            .map(|oid| oid.to_hex().to_string())
            .unwrap_or_default();
        cache.log_mirror_sync_written(&oid_hex, &state.backend_name);

        // §3c token-cost: estimate push bytes-in by summing parsed blob
        // payloads. Bytes-out is the single ack line.
        let chars_in: u64 = parsed
            .blobs
            .values()
            .map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX))
            .sum();
        let chars_out: u64 = "ok refs/heads/main\n".len() as u64;
        cache.log_token_cost(chars_in, chars_out, "push");
    }
    proto.send_line("ok refs/heads/main")?;
    proto.send_blank()?;
    proto.flush()?;
    Ok(())
}

/// Apply a single [`PlannedAction`] to the backend.
///
/// Narrow-deps signature (P83-01 T02 refactor): takes `(backend, // banned-words: ok
/// project, rt, cache, action)` rather than `&mut State` so
/// `crate::write_loop::apply_writes` (which has no `State` access)
/// can call it directly.
///
/// `pub(crate)` so the `write_loop` module can call this via
/// `crate::execute_action`.
///
/// # Errors
/// Returns `Err` from any backend REST call (`create_record`,
/// `update_record`, `delete_or_close`).
pub(crate) fn execute_action(
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
    cache: Option<&Cache>,
    action: PlannedAction,
) -> Result<()> {
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
            let _new = rt
                .block_on(backend.create_record(project, untainted))
                .context("create issue")?;
            Ok(())
        }
        PlannedAction::Update {
            id,
            prior_version,
            new,
        } => {
            // ARCH-10: every Update implicitly sanitizes server-controlled
            // frontmatter fields (id/created_at/updated_at/version). Emit a
            // best-effort audit row so an operator can see the sanitize
            // boundary without re-deriving it from the diff. `version` is
            // the dominant server-controlled field; the row is per-issue
            // (not per-field).
            if let Some(cache) = cache {
                cache.log_helper_push_sanitized_field(&id.0.to_string(), "version");
            }
            let meta = ServerMetadata {
                id,
                created_at: new.created_at,
                updated_at: new.updated_at,
                version: prior_version,
            };
            let untainted = sanitize(Tainted::new(new), meta);
            rt.block_on(backend.update_record(project, id, untainted, Some(prior_version)))
                .with_context(|| format!("patch issue {}", id.0))?;
            Ok(())
        }
        PlannedAction::Delete { id, .. } => {
            match rt.block_on(backend.delete_or_close(project, id, DeleteReason::Abandoned)) {
                // Fork B (P94 defense-in-depth): a delete-time "not found" means
                // the record is ALREADY gone — exactly the desired end state.
                // Treat it as an IDEMPOTENT SUCCESS, not a failure, so a
                // phantom/ghost Delete that slips past the Fork-A prune gate
                // (e.g. a residual oid_map row on an over-cap project where the
                // prune is skipped) cannot force a FALSE SotPartialFail on every
                // push. Bounds the residual blast to audit noise. See
                // `.planning/CONSULT-DECISIONS.md` 2026-07-05 [FABLE]
                // pagination-truncation prune-safety fork (Fork B).
                Err(e) if is_delete_notfound(&e) => {
                    diag(&format!(
                        "delete issue {}: already absent on the backend — \
                         idempotent no-op (Fork B)",
                        id.0
                    ));
                    Ok(())
                }
                other => other.with_context(|| format!("delete issue {}", id.0)),
            }
        }
    }
}

/// Fork B (P94): does this error mean the record a delete targeted is ALREADY
/// absent on the backend? Such a delete is idempotently satisfied — the record
/// is in the desired end state — so it must NOT count as a write failure.
///
/// Matches both the typed [`reposix_core::Error::NotFound`] and the legacy
/// stringly `Error::Other("not found: …")` shape some adapters still emit
/// during the typed-error migration (see `reposix_core::error`).
fn is_delete_notfound(e: &reposix_core::Error) -> bool {
    matches!(e, reposix_core::Error::NotFound { .. })
        || matches!(e, reposix_core::Error::Other(msg) if msg.trim_start().starts_with("not found"))
}

/// Adapter so `BufReader` can pull from the same underlying stdin we own
/// inside `Protocol`. We provide a one-line-at-a-time bridge — the
/// `parse_export_stream` parser uses `read_line` and `read_exact` only.
///
/// `pub(crate)` (and `pub(crate) fn new`) so the sibling
/// [`crate::bus_handler`] module can construct one in the bus write
/// fan-out path (T04) — same stdin parser substrate as `handle_export`.
pub(crate) struct ProtoReader<'a, R: std::io::Read, W: std::io::Write> {
    proto: &'a mut Protocol<R, W>,
    buf: Vec<u8>,
    pos: usize,
}

impl<'a, R: std::io::Read, W: std::io::Write> ProtoReader<'a, R, W> {
    pub(crate) fn new(proto: &'a mut Protocol<R, W>) -> Self {
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

#[cfg(test)]
mod fork_b_tests {
    //! Fork B (P94 pagination-prune-safety, defense-in-depth): a delete against
    //! an already-absent record is an IDEMPOTENT SUCCESS at the write boundary,
    //! never a `SotPartialFail`-forcing failure. `execute_action` is
    //! `pub(crate)`, so this in-crate unit test drives it directly — the full
    //! push-loop end-to-end is covered by
    //! `tests/deleted_record_ghost_oid_map_row_forces_false_partial_fail.rs`.

    use super::{execute_action, is_delete_notfound};
    use crate::diff::PlannedAction;
    use async_trait::async_trait;
    use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
    use reposix_core::{Error as CoreError, Record, RecordId, Result as CoreResult, Untainted};
    use tokio::runtime::Runtime;

    /// Backend whose `delete_or_close` returns a caller-chosen error.
    struct DeleteBackend {
        err: fn() -> CoreError,
    }

    #[async_trait]
    impl BackendConnector for DeleteBackend {
        fn name(&self) -> &'static str {
            "delete-backend-stub"
        }
        fn supports(&self, _f: BackendFeature) -> bool {
            false
        }
        async fn list_records(&self, _: &str) -> CoreResult<Vec<Record>> {
            Ok(vec![])
        }
        async fn get_record(&self, _: &str, _: RecordId) -> CoreResult<Record> {
            Err(CoreError::Other("unused".into()))
        }
        async fn create_record(&self, _: &str, _: Untainted<Record>) -> CoreResult<Record> {
            Err(CoreError::Other("unused".into()))
        }
        async fn update_record(
            &self,
            _: &str,
            _: RecordId,
            _: Untainted<Record>,
            _: Option<u64>,
        ) -> CoreResult<Record> {
            Err(CoreError::Other("unused".into()))
        }
        async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
            Err((self.err)())
        }
    }

    fn run_delete(err: fn() -> CoreError) -> anyhow::Result<()> {
        let rt = Runtime::new().unwrap();
        let backend = DeleteBackend { err };
        let action = PlannedAction::Delete {
            id: RecordId(2),
            prior_version: 1,
        };
        execute_action(&backend, "demo", &rt, None, action)
    }

    #[test]
    fn typed_notfound_is_classified_idempotent() {
        assert!(is_delete_notfound(&CoreError::NotFound {
            project: "demo".into(),
            id: "2".into(),
        }));
    }

    #[test]
    fn stringly_not_found_is_classified_idempotent() {
        // Legacy adapters still emit the not-found condition via Error::Other.
        assert!(is_delete_notfound(&CoreError::Other(
            "not found: demo/2".into()
        )));
    }

    #[test]
    fn genuine_failures_are_not_classified_idempotent() {
        assert!(!is_delete_notfound(&CoreError::Other("boom".into())));
        assert!(!is_delete_notfound(&CoreError::Other(
            "version mismatch: current=2 requested=1".into()
        )));
    }

    #[test]
    fn delete_of_already_absent_record_is_idempotent_success() {
        // Fork B core contract: a NotFound on delete resolves to Ok(()), so
        // write_loop's failed_ids stays empty and no false SotPartialFail fires.
        let out = run_delete(|| CoreError::NotFound {
            project: "demo".into(),
            id: "2".into(),
        });
        assert!(
            out.is_ok(),
            "delete of an already-absent record must be an idempotent success (Fork B), got {out:?}"
        );
    }

    #[test]
    fn delete_with_genuine_error_still_fails() {
        // A non-NotFound backend error on delete must STILL surface as Err —
        // Fork B must not swallow real failures.
        let out = run_delete(|| CoreError::Other("500 internal".into()));
        assert!(
            out.is_err(),
            "a genuine (non-NotFound) delete error must still fail the action, not be swallowed"
        );
    }
}
