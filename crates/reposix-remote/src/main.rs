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
use crate::fast_import::{emit_import_stream, parse_export_stream, ImportParent};
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
        anyhow::bail!(
            "{}",
            reposix_core::errmsg::teach_coded(
                reposix_core::codes::ids::HELPER_USAGE,
                "git-remote-reposix was invoked with too few arguments (it needs <alias> and <url>).",
                "git-remote-reposix is a git REMOTE HELPER — git runs it automatically for \
                 `reposix::` remotes; you normally never invoke it by hand.",
                "just use git against a reposix remote (`git fetch` / `git push`); to create one, \
                 run `reposix init <backend>::<project> <path>`.",
                &[
                    "reposix init sim::demo /tmp/demo   # creates a reposix:: remote git can drive",
                    "git -C /tmp/demo fetch origin",
                ],
            )
        );
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
    // No `.context(...)` wrapper here (P120 W4): `bus_url::parse` now returns a
    // full 3-part teaching body (`malformed_bus_url_error`); a terse context
    // prefix would bury the headline behind "parse remote url: …" in git stderr.
    let route = bus_url::parse(url)?;
    let (parsed, mirror_url_opt): (backend_dispatch::ParsedRemote, Option<String>) = match route {
        bus_url::Route::Single(p) => (p, None),
        bus_url::Route::Bus { sot, mirror_url } => (sot, Some(mirror_url)),
    };
    // No `.context(...)` wrapper (P120 W4): `instantiate` surfaces either a
    // teaching `missing_env_error` (credentials unset) or a self-describing
    // constructor error; both read cleanly without a burying prefix.
    let backend = instantiate(&parsed)?;
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
                // RBF-LR-03 layer-2: advertise the helper's PRIVATE import
                // namespace as the fast-import write target. git maps this into
                // the caller's tracking ns via `remote.origin.fetch`
                // (`+refs/heads/*:refs/reposix/origin/*`, init.rs), so git fetch
                // stays the SOLE writer of `refs/reposix/origin/*`. Collapsing
                // both onto `refs/reposix/origin/*` made the helper AND git
                // fetch race on one ref → `cannot lock ref … is at T1 but
                // expected T0` aborting `git pull --rebase`. Two disjoint
                // namespaces = the canonical remote-helper contract.
                proto.send_line("refspec refs/heads/*:refs/reposix-import/*")?;
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

/// Build the 3-part teaching detail for a fetch/import backend-unreachable
/// failure (RPX-0507). Split out so a unit test can pin that the `[RPX-0507]`
/// tag + `Explain:` nudge render on this path WITHOUT a live protocol exchange
/// (the detail otherwise flows to real stderr via `diag`, which is not
/// in-process capturable). The `underlying` connector error is surfaced, not
/// swallowed (leverage #3); it carries no userinfo — backend credentials live
/// in request headers, never in the origin URL (OP-2).
fn import_unreachable_detail(underlying: &str) -> String {
    reposix_core::errmsg::teach_coded(
        reposix_core::codes::ids::HELPER_IMPORT_UNREACHABLE,
        &format!("could not list records from the backend to import: {underlying}"),
        "the SoT backend could not be reached or read — usually the simulator is not \
         running, or a real backend's credentials are unset.",
        "for a no-network check, start the simulator and target a `sim::` remote.",
        &[
            "reposix sim      # start the simulator, if this is a sim:: remote",
            "reposix doctor   # check backend reachability + credentials",
        ],
    )
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
            // The DIAG detail is the 3-part teaching (P120 W4), now CODED with
            // RPX-0507 (P121 W3.6). The `error refs/heads/main backend-unreachable`
            // PROTOCOL line (emitted by `fail_push`) stays verbatim — git parses it
            // — while this teaching rides the accompanying stderr `error:` diag.
            let detail = import_unreachable_detail(&format!("{e:#}"));
            return fail_push(proto, state, "backend-unreachable", &detail).map_err(Into::into);
        }
    };
    // Emit fast-import stream over stdout via the protocol writer, using
    // the backend's canonical record bucket (issues/ vs pages/).
    let bucket = reposix_core::path::bucket_for_backend(&state.backend_name);
    // RBF-LR-03: chain the synthesized tracking commit onto the client's
    // current `refs/reposix/origin/main` tip so the fetch is a fast-forward
    // and `git pull --rebase && git push` reconciles after SoT drift. git
    // sets GIT_DIR for the helper RPC, so a bare `git rev-parse` resolves
    // against the caller's repo (same proven pattern as the bus handler's
    // mirror-drift check). Ref genuinely absent (first fetch) → Ok(None) →
    // parentless root; a NON-absence git failure errors LOUDLY (RPX-0508) via
    // fail_push below rather than silently degrading to the parentless overlay.
    // The ref name is a static literal, never a remote byte (OP-2 taint).
    let parent = match resolve_import_parent() {
        Ok(p) => p,
        Err(e) => {
            // resolve_import_parent already built the loud RPX-0508 teaching; surface
            // it via fail_push (protocol `error` line + teaching diag) rather than a
            // torn pipe or a silent parentless overlay (the RBF-LR-03 `does not
            // contain` regression). Mirrors the RPX-0507 backend-unreachable arm above.
            let detail = format!("{e:#}");
            return fail_push(proto, state, "import-parent-resolve-failed", &detail)
                .map_err(Into::into);
        }
    };
    let mut sink: Vec<u8> = Vec::with_capacity(1024 + issues.len() * 256);
    emit_import_stream(&mut sink, &issues, bucket, parent.as_ref())?;
    proto.send_raw(&sink)?;
    proto.flush()?;
    Ok(())
}

/// One `git rev-parse` subprocess outcome, normalized for
/// [`resolve_import_parent`]'s tri-state logic and injectable in tests without a
/// real `git` on `PATH`.
///
/// `code` is the process exit status (`None` when git was terminated by a signal
/// — also a non-absence failure); `stdout` is the trimmed standard output (the
/// resolved oid on a success exit). Neither field carries a remote byte: the only
/// inputs are a static ref name and git's own local output/status for it (OP-2).
struct RevParseRun {
    /// Process exit code, or `None` when git was killed by a signal.
    code: Option<i32>,
    /// Trimmed stdout — the resolved object id on a success exit.
    stdout: String,
}

/// Build the RPX-0508 teaching for a NON-absence `git rev-parse` failure while
/// resolving the client's import parent. Split out (mirroring
/// [`import_unreachable_detail`] for RPX-0507) so a unit test can pin the
/// `[RPX-0508]` tag + 3-part body directly — the detail otherwise flows to real
/// stderr via `diag`, which is not in-process capturable.
///
/// `what` is a LOCAL diagnostic (a spawn `io::Error`, an exit-code integer, or a
/// static anomaly description) — never git's stderr and never a remote byte, so no
/// redaction is required (T-122-04): this path reads only a static ref name and
/// git's exit status/stdout for it, and deliberately does NOT surface `out.stderr`.
fn import_parent_resolve_detail(what: &str) -> String {
    reposix_core::errmsg::teach_coded(
        reposix_core::codes::ids::HELPER_IMPORT_PARENT_RESOLVE,
        &format!(
            "could not resolve the client's `refs/reposix/origin/main` tracking tip \
             to chain the import onto: {what}"
        ),
        "make sure `git` is installed and on PATH and the directory git invoked the \
         helper in is a valid git repository, then re-drive the fetch.",
        "for a tree that was never bootstrapped by reposix, re-run `reposix init` / \
         `reposix attach` to lay down a valid partial-clone tree instead of fetching \
         into a hand-made one.",
        &[
            "git --version                          # confirm git is installed and on PATH",
            "git rev-parse --is-inside-work-tree    # confirm the caller dir is a valid git repo",
            "git fetch                              # re-drive once git + the repo are healthy",
        ],
    )
}

/// Run one real `git rev-parse --verify --quiet <arg>` against the caller's repo.
/// git sets `GIT_DIR` for the remote-helper RPC, so a bare `rev-parse` (no `-C`)
/// resolves against the caller's repo (the same pattern the bus mirror-drift check
/// relies on). This is the production runner behind [`resolve_import_parent`];
/// tests inject a fake in its place.
fn real_rev_parse(arg: &str) -> std::io::Result<RevParseRun> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", arg])
        .output()?;
    Ok(RevParseRun {
        code: out.status.code(),
        // Only git's stdout (the resolved oid) is read — never `out.stderr`, so no
        // git message can ride the RPX-0508 diag (T-122-04).
        stdout: String::from_utf8_lossy(&out.stdout).trim().to_owned(),
    })
}

/// Resolve the client's current `refs/reposix/origin/main` tip (commit + tree
/// oids) via `git rev-parse`, for RBF-LR-03 parent chaining. TRI-STATE:
///
/// - `Ok(Some(ImportParent))` — the ref resolved (a present tracking tip).
/// - `Ok(None)` — the ref is genuinely ABSENT (spawn-success + exit 1 + empty
///   stdout, the `--verify --quiet` contract): a legitimate first fetch →
///   parentless seed, so a fresh clone still bootstraps.
/// - `Err(_)` — a NON-absence `git rev-parse` failure: a spawn failure (git not
///   on PATH), a non-1 non-zero exit (e.g. 128 corrupt-repo / bad-revision), a
///   signal, or an anomalous exit-0 with empty stdout (`--verify --quiet` always
///   prints the oid on success, so an empty stdout on exit 0 is a broken-git
///   signal, not a benign absence). These error LOUDLY (coded RPX-0508) rather
///   than silently degrading to the parentless overlay, which would hide a real
///   environmental fault and could re-open the RBF-LR-03 non-descendant `does not
///   contain` fast-import abort with no operator-facing error.
///
/// Ref-absence is a genuine first-fetch / uninitialized state for BOTH bootstrap
/// paths: `reposix init` seeds this ref via its `+refs/heads/*:refs/reposix/origin/*`
/// refspec on the initial fetch, and (as of the v0.14.0 attach-lineage fix)
/// `reposix attach` seeds it to the mirror merge-base at attach time
/// (`crates/reposix-cli/src/attach.rs::seed_tracking_ref`). So on a
/// correctly-bootstrapped tree the ref is present by the SECOND fetch and this
/// returns a real parent — the parentless branch is the true first-sync seed, not
/// a silent attach defect. (Pre-fix attach trees with the ref never seeded heal by
/// re-running `reposix attach`; runtime auto-heal here is v0.15.0-deferred.)
///
/// The ref name is a fixed literal — never a remote-influenced byte — so there is
/// no `Tainted<T>` concern (OP-2).
fn resolve_import_parent() -> anyhow::Result<Option<ImportParent>> {
    resolve_import_parent_with(real_rev_parse)
}

/// [`resolve_import_parent`] with the `git rev-parse` runner injected, so a unit
/// test can drive the tri-state logic (spawn failure / exit 128 / exit-0-empty /
/// exit-1-absent / present) without a real `git` on `PATH`.
fn resolve_import_parent_with<F>(run: F) -> anyhow::Result<Option<ImportParent>>
where
    F: Fn(&str) -> std::io::Result<RevParseRun>,
{
    const REF: &str = "refs/reposix/origin/main";
    let rev_parse = |arg: &str| -> anyhow::Result<Option<String>> {
        let what: String = match run(arg) {
            // (a) spawn failure — git not on PATH / could not exec.
            Err(e) => format!("the `git rev-parse` subprocess could not be spawned ({e})"),
            // present: a success exit WITH a resolved oid.
            Ok(RevParseRun {
                code: Some(0),
                stdout,
            }) if !stdout.is_empty() => {
                return Ok(Some(stdout));
            }
            // (c) anomalous: a success exit but NO oid printed — a broken-git signal,
            // NOT a benign absence. `--verify --quiet` always prints the oid on exit 0.
            Ok(RevParseRun { code: Some(0), .. }) => {
                "`git rev-parse --verify --quiet` exited 0 but printed no object id \
                 (a broken-git / malformed-subprocess signal)"
                    .to_owned()
            }
            // benign ABSENCE: the `--verify --quiet` contract for a missing ref.
            Ok(RevParseRun { code: Some(1), .. }) => return Ok(None),
            // (b) any OTHER non-zero exit (e.g. 128 corrupt-repo / bad-revision).
            Ok(RevParseRun {
                code: Some(other), ..
            }) => {
                format!("`git rev-parse --verify --quiet` exited with status {other}")
            }
            // (b') terminated by a signal — also a non-absence failure.
            Ok(RevParseRun { code: None, .. }) => {
                "the `git rev-parse` subprocess was terminated by a signal".to_owned()
            }
        };
        // The terminal RPX-0508 teaching (3-part, coded) lives in import_parent_resolve_detail;
        // teach-exempt: ok — this thin anyhow! only surfaces that helper's teaching (teach_scan can't resolve the helper indirection — a documented residual — so the bar is met there, not inline).
        Err(anyhow::anyhow!("{}", import_parent_resolve_detail(&what)))
    };
    let Some(commit) = rev_parse(REF)? else {
        return Ok(None);
    };
    // `<ref>^{tree}` peels the commit to its tree oid; if the commit resolved this
    // should too. A NON-absence failure on the peel errors loudly via `?`; only a
    // benign exit-1 absence falls back to the parentless seed (stay defensive).
    let Some(tree) = rev_parse(&format!("{REF}^{{tree}}"))? else {
        return Ok(None);
    };
    Ok(Some(ImportParent { commit, tree }))
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
            // teach-exempt: ok — git's own malformed fast-import stream; internal protocol
            // error, not a user-actionable teaching target (git generated the stream).
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
            // teach-exempt: ok — per-action REST wrap; terminal teaching is emitted at the
            // write_loop reject diag (W5), this is an internal annotation on the backend call.
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
            // teach-exempt: ok — per-action REST wrap; terminal teaching at the write_loop
            // reject diag (W5). `id.0` is a numeric record id — no URL/credential to redact.
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
                // teach-exempt: ok — per-action REST wrap; terminal teaching at the write_loop
                // reject diag (W5). `id.0` is a numeric record id — no URL/credential to redact.
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

#[cfg(test)]
mod import_teach_tests {
    //! P121 W3.6 (SC1): the fetch/import backend-unreachable teaching carries the
    //! RPX-0507 code. The detail flows to real stderr via `diag` (an `eprintln!`
    //! not in-process capturable), so we pin the string-builder — the exact bytes
    //! `handle_import_batch`'s `Err` arm hands to `fail_push` — directly. The live
    //! `import`-command → dead-backend path is verified against reality by driving
    //! the built helper binary with an `import` command (see the W3.6 evidence).

    use super::import_unreachable_detail;

    #[test]
    fn import_unreachable_detail_renders_rpx0507_tag_and_explain_nudge() {
        let detail = import_unreachable_detail("connection refused (os error 111)");
        // The code tag rides the FIRST line of the headline.
        assert!(
            detail.starts_with("could not list records from the backend to import: ")
                && detail.lines().next().unwrap().ends_with("[RPX-0507]"),
            "the [RPX-0507] tag must ride the first headline line, got:\n{detail}"
        );
        // The full 3-part teaching shape survives the code render.
        assert!(detail.contains("Fix:"), "missing Fix: limb:\n{detail}");
        assert!(
            detail.contains("Recovery:"),
            "missing Recovery: block:\n{detail}"
        );
        // The trailing explain nudge (the codified north-star touch).
        assert!(
            detail.contains("Explain: reposix explain RPX-0507"),
            "missing the `reposix explain RPX-0507` nudge:\n{detail}"
        );
        // The underlying connector error is surfaced, not swallowed.
        assert!(
            detail.contains("connection refused (os error 111)"),
            "underlying connector error must be surfaced:\n{detail}"
        );
    }
}

#[cfg(test)]
mod resolve_import_parent_tests {
    //! P122 W2 (DRAIN-08 / GTH-V15-05): `resolve_import_parent` must FAIL LOUD
    //! (coded RPX-0508) on a NON-absence `git rev-parse` failure instead of
    //! silently degrading to the parentless overlay — while still treating a
    //! genuine ref-absent first fetch (exit 1, empty stdout) as `Ok(None)`. The
    //! git runner is injected so these drive the tri-state logic with no real
    //! `git` on `PATH`. These live in a bin-target `#[cfg(test)]` module → graded
    //! by the BARE `cargo test -p reposix-remote` (a `--test <name>` scope would
    //! miss bin-target unit tests, per crates/CLAUDE.md).

    use super::{resolve_import_parent_with, RevParseRun};
    use std::io;

    const RPX: &str = "RPX-0508";

    /// Test A: an injected NON-absence failure — a fake git that exits 128
    /// (corrupt-repo / bad-revision) — makes `resolve_import_parent` return a LOUD
    /// `Err` carrying the RPX-0508 coded teaching (tag + Fix + Recovery + Explain
    /// nudge), NOT `Ok(None)` / a parentless overlay.
    #[test]
    fn non_absence_exit_128_errors_loud_with_rpx0508() {
        let run = |_arg: &str| {
            Ok(RevParseRun {
                code: Some(128),
                stdout: String::new(),
            })
        };
        let err = resolve_import_parent_with(run)
            .expect_err("a non-absence exit-128 git failure must error loud, not Ok(None)");
        let msg = format!("{err:#}");
        assert!(
            msg.contains(RPX),
            "loud error must carry the {RPX} tag, got:\n{msg}"
        );
        assert!(
            msg.contains("Fix:"),
            "loud error must teach a Fix:, got:\n{msg}"
        );
        assert!(
            msg.contains("Recovery:"),
            "loud error must teach a Recovery:, got:\n{msg}"
        );
        assert!(
            msg.contains("Explain: reposix explain RPX-0508"),
            "loud error must carry the explain nudge, got:\n{msg}"
        );
    }

    /// Test A': a SPAWN failure (git not on PATH / could not exec) is likewise a
    /// non-absence failure → a loud RPX-0508 `Err`, never `Ok(None)`.
    #[test]
    fn spawn_failure_errors_loud_with_rpx0508() {
        let run = |_arg: &str| {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "git: command not found",
            ))
        };
        let err = resolve_import_parent_with(run)
            .expect_err("a git spawn failure must error loud, not Ok(None)");
        assert!(
            format!("{err:#}").contains(RPX),
            "a spawn failure must carry the {RPX} teaching, not silently degrade"
        );
    }

    /// Test B: a GENUINE ref-absent first fetch — spawn-success + exit 1 + empty
    /// stdout (the `--verify --quiet` contract) — still degrades to `Ok(None)` (the
    /// parentless seed), so a fresh clone bootstraps. No regression to bootstrap.
    #[test]
    fn ref_absent_exit_1_returns_ok_none() {
        let run = |_arg: &str| {
            Ok(RevParseRun {
                code: Some(1),
                stdout: String::new(),
            })
        };
        let out = resolve_import_parent_with(run);
        assert!(
            matches!(out, Ok(None)),
            "a genuine ref-absent first fetch (exit 1, empty stdout) must be Ok(None), got {out:?}"
        );
    }

    /// Test C: an ANOMALOUS success — exit 0 with EMPTY stdout — is a broken-git
    /// signal (`--verify --quiet` always prints the oid on a success exit), so it
    /// errors LOUD (RPX-0508) rather than silently degrading to the parentless
    /// overlay. Locks the exit-0-empty-stdout branch so the row cannot grade GREEN
    /// while that branch silently returns `Ok(None)`.
    #[test]
    fn anomalous_exit_0_empty_stdout_errors_loud_with_rpx0508() {
        let run = |_arg: &str| {
            Ok(RevParseRun {
                code: Some(0),
                stdout: String::new(),
            })
        };
        let err = resolve_import_parent_with(run)
            .expect_err("exit-0 + empty stdout is anomalous and must error loud, not Ok(None)");
        assert!(
            format!("{err:#}").contains(RPX),
            "the anomalous exit-0-empty-stdout path must carry the {RPX} teaching, not Ok(None)"
        );
    }

    /// The present-ref happy path: both `<ref>` and `<ref>^{{tree}}` resolve to a
    /// real oid on a success exit → `Ok(Some(ImportParent{{commit, tree}}))`.
    /// Confirms the loud path did not regress the present case.
    #[test]
    fn present_ref_returns_some_import_parent() {
        let run = |arg: &str| {
            let oid = if arg.contains("^{tree}") {
                "1111111111111111111111111111111111111111"
            } else {
                "2222222222222222222222222222222222222222"
            };
            Ok(RevParseRun {
                code: Some(0),
                stdout: oid.to_owned(),
            })
        };
        let parent = resolve_import_parent_with(run)
            .expect("a present ref must resolve without error")
            .expect("a present ref must be Some(ImportParent)");
        assert_eq!(parent.commit, "2222222222222222222222222222222222222222");
        assert_eq!(parent.tree, "1111111111111111111111111111111111111111");
    }
}
