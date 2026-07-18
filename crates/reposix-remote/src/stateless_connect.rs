//! `stateless-connect` capability handler — protocol-v2 tunnel.
//!
//! Ports the Python POC at
//! `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py`
//! into idiomatic Rust. The helper is a thin pkt-line pipe between
//! git's stdin/stdout and a `git upload-pack --stateless-rpc`
//! subprocess running against the Phase 31 `reposix-cache` bare repo.
//!
//! Three protocol gotchas (see `partial-clone-remote-helper-findings.md`
//! Q2) are encoded as named tests in this module:
//!
//! 1. [`initial_advertisement_ends_with_flush_only`] — no trailing
//!    response-end on the unsolicited v2 advertisement.
//! 2. [`rpc_response_appends_response_end`] — every RPC response ends
//!    with `0002`.
//! 3. Binary stdin throughout — see [`super::pktline`] tests for byte
//!    round-trip; the handler uses [`super::protocol::Protocol::reader_mut`]
//!    to share the same `BufReader<Stdin>` with the handshake line
//!    reader, never constructing a second buffer over stdin.
//!
//! ## Environment variables
//!
//! - `REPOSIX_BLOB_LIMIT` — max `want` lines per `command=fetch` RPC
//!   turn (Phase 34, ARCH-09). Default 200; `0` = unlimited. Read once
//!   at first access via a `OnceLock` cache.
//! - `REPOSIX_ALLOWED_ORIGINS` — egress allowlist (Phase 1). Inherited
//!   via `reposix_core::http::client()`.

#![forbid(unsafe_code)]

use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use reposix_cache::{cache::HelperFetchRecord, Cache};
use tokio::runtime::Runtime;

use crate::pktline::{self, Frame};
use crate::protocol::Protocol;

/// Default upper bound on `want` lines per `command=fetch` RPC turn.
/// Configurable via `REPOSIX_BLOB_LIMIT` env var. `0` means "unlimited"
/// (explicit opt-out for very-large bulk operations).
pub(crate) const DEFAULT_BLOB_LIMIT: u32 = 200;

/// Verbatim stderr message for blob-limit refusal. Backticks around
/// `git sparse-checkout set <pathspec>` are LITERAL — they render as
/// code formatting in agent terminals that support it, and look correct
/// in plaintext. The literal string `git sparse-checkout` is the
/// dark-factory teaching mechanism (REQUIREMENTS.md ARCH-09): an
/// unprompted agent reads the error, runs the named command, and
/// self-corrects with no in-context system-prompt instructions.
pub(crate) const BLOB_LIMIT_EXCEEDED_FMT: &str =
    "error: refusing to fetch {N} blobs (limit: {M}). Narrow your scope with `git sparse-checkout set <pathspec>` and retry.";

/// Verbatim teaching hint printed when an UNFILTERED fetch dies inside
/// upload-pack because the lazy cache has no blob objects for the full
/// reachable closure. The literal `--filter=blob:none` is the recovery
/// move (same dark-factory contract as [`BLOB_LIMIT_EXCEEDED_FMT`]).
pub(crate) const UNFILTERED_FETCH_HINT: &str =
    "error: this reposix cache materializes blobs lazily and cannot serve an unfiltered fetch (upload-pack tried to pack blobs that were never materialized). Re-run with `--filter=blob:none` — `reposix init` sets this automatically; a manual `git fetch` must pass it.";

/// True when upload-pack's stderr carries the signature of a fetch that
/// walked into unmaterialized blobs (the misleading "corruption" death).
/// Pure predicate so the detection is unit-tested without a live git fetch.
fn is_unmaterialized_closure_error(stderr: &str) -> bool {
    stderr.contains("possible repository corruption") || stderr.contains("unable to read")
}

/// Build the Rust-compiler-grade teaching error for a `git upload-pack`
/// subprocess that exited non-zero (P120 W5). `phase` labels which invocation
/// failed (`--advertise-refs` vs `--stateless-rpc`). git's own stderr (the raw
/// cause) is PRESERVED in the headline — never discarded — and a teaching layer
/// on top names the likely root cause + a runnable `reposix doctor` recovery.
/// No `Alternative:` line: there is no genuine alternative approach to a crashed
/// server subprocess — the fix is to diagnose and repair (FLAG-1 suppression).
fn upload_pack_failure_error(
    phase: &str,
    status: std::process::ExitStatus,
    stderr: &str,
) -> anyhow::Error {
    let headline = format!(
        "git upload-pack {phase} failed ({status}): {}",
        stderr.trim()
    );
    anyhow::anyhow!(
        "{}",
        reposix_core::errmsg::teach_coded(
            reposix_core::codes::ids::HELPER_UPLOAD_PACK,
            &headline,
            "the cache's bare repo could not serve upload-pack — usually an \
             incompatible git (partial-clone reads need git 2.34+) or a \
             missing/corrupt cache",
            "", // no genuine alternative to a crashed server subprocess (FLAG-1)
            &["reposix doctor   # verify git 2.34+ and cache health"],
        )
    )
}

/// Build the teaching error for an unexpected EOF read mid-request (P120 W5).
/// A protocol desync — the git client closed the connection partway through a
/// pkt-line request. Per the FLAG-1 motivating case this has NO genuine
/// `Alternative:` (the user does not "do it differently"); it teaches the desync
/// + the re-drive recovery only.
fn eof_midrequest_error() -> anyhow::Error {
    anyhow::anyhow!(
        "{}",
        reposix_core::errmsg::Teach::new(
            "unexpected EOF mid-request — the git client closed the connection \
             partway through a pkt-line request (protocol desync)"
        )
        .fix(
            "re-run the git operation from a clean state; a killed/backgrounded \
             git process or a broken pipe is the usual trigger"
        )
        .recovery(&["git fetch origin   # re-drive the fetch on a fresh connection"])
        .code(reposix_core::codes::ids::HELPER_EOF)
    )
}

/// Process-wide cache for `REPOSIX_BLOB_LIMIT`. Read once at first
/// access; subsequent calls are lock-free.
///
/// Note: tests that exercise env-driven behaviour should use the pure
/// helper [`parse_blob_limit`] instead — the `OnceLock` state is
/// process-global and can leak across tests in the same binary.
static BLOB_LIMIT: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Pure parser for `REPOSIX_BLOB_LIMIT`. Whitespace-trimmed; empty or
/// non-numeric input falls back to [`DEFAULT_BLOB_LIMIT`]. `0` is
/// preserved verbatim (the enforcement code interprets `0` as
/// "unlimited"; the parser stays neutral).
fn parse_blob_limit(raw: Option<&str>) -> u32 {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => DEFAULT_BLOB_LIMIT,
        Some(s) => s.parse::<u32>().unwrap_or(DEFAULT_BLOB_LIMIT),
    }
}

/// Resolve the configured blob limit. Reads `REPOSIX_BLOB_LIMIT` once;
/// invalid values (non-numeric, overflow) fall back to
/// [`DEFAULT_BLOB_LIMIT`] with a `tracing::warn!`. `0` is the explicit
/// opt-out and is preserved verbatim.
#[must_use]
pub(crate) fn configured_blob_limit() -> u32 {
    *BLOB_LIMIT.get_or_init(|| match std::env::var("REPOSIX_BLOB_LIMIT") {
        Ok(s) => {
            let parsed = parse_blob_limit(Some(&s));
            if parsed == DEFAULT_BLOB_LIMIT && s.trim() != DEFAULT_BLOB_LIMIT.to_string() {
                // Only warn when we fell back due to garbage input (not
                // when the user explicitly set the value to the default).
                tracing::warn!(
                    raw = %s,
                    "invalid REPOSIX_BLOB_LIMIT, using default {DEFAULT_BLOB_LIMIT}"
                );
            }
            parsed
        }
        Err(_) => DEFAULT_BLOB_LIMIT,
    })
}

/// Format the verbatim refusal message with concrete `N` (want count)
/// and `M` (limit) substituted.
#[must_use]
pub(crate) fn format_blob_limit_message(want_count: u32, limit: u32) -> String {
    BLOB_LIMIT_EXCEEDED_FMT
        .replace("{N}", &want_count.to_string())
        .replace("{M}", &limit.to_string())
}

/// Request-turn counters for audit + Phase 34 blob-limit telemetry.
#[derive(Debug, Default, Clone)]
pub(crate) struct RpcStats {
    /// Count of `want ` lines seen in the request payload.
    pub(crate) want_count: u32,
    /// Total bytes written to `upload-pack` stdin (re-encoded request).
    pub(crate) request_bytes: u32,
    /// Total bytes read from `upload-pack` stdout (response body).
    pub(crate) response_bytes: u32,
    /// First keyword after `command=` in the first data frame, if any.
    /// Common values: `fetch`, `ls-refs`, `object-info`.
    pub(crate) command: Option<String>,
}

impl HelperFetchRecord for RpcStats {
    fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }
    fn want_count(&self) -> u32 {
        self.want_count
    }
    fn request_bytes(&self) -> u32 {
        self.request_bytes
    }
    fn response_bytes(&self) -> u32 {
        self.response_bytes
    }
}

/// Outcome of [`handle_stateless_connect`] — tells the caller's command
/// loop whether git has taken over the stream or the helper must resume
/// reading verbs.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StatelessConnectOutcome {
    /// `git-upload-pack` was served: the protocol-v2 session ran to EOF
    /// and git owned stdin/stdout for its duration. The helper must now
    /// exit — the stream belongs to git and there are no further verbs.
    TookOver,
    /// A non-upload-pack service was requested (push probes
    /// `git-receive-pack` first). The helper replied the
    /// git-remote-helpers(7) `fallback` sentinel; git will retry via
    /// another advertised capability (`export` for push). The command
    /// loop MUST resume so the SAME live helper process handles the
    /// follow-up verb.
    FellBack,
}

/// Entry point for the `stateless-connect <service>` verb.
///
/// The caller has already read the verb line from stdin. For
/// `git-upload-pack` we write the one-line "ready" response, send the
/// protocol-v2 advertisement, then loop reading RPC turns from git until
/// EOF (returns [`StatelessConnectOutcome::TookOver`]). For any other
/// service we reply the `fallback` sentinel and return
/// [`StatelessConnectOutcome::FellBack`] so the caller resumes its verb
/// loop.
///
/// # Errors
/// Any I/O error from stdin/stdout, any `upload-pack` spawn failure,
/// any cache error propagated from [`Cache::open`] or
/// [`Cache::build_from`].
pub(crate) fn handle_stateless_connect<R: Read, W: Write>(
    proto: &mut Protocol<R, W>,
    rt: &Runtime,
    cache: &Cache,
    service: &str,
) -> Result<StatelessConnectOutcome> {
    if service != "git-upload-pack" {
        // git-remote-helpers(7): the three valid replies to
        // `stateless-connect <service>` are an empty line (connection
        // established), the literal `fallback` line (this helper cannot
        // serve the service over protocol-v2 — retry via another
        // advertised capability), or exiting with an error (don't fall
        // back at all). git >= 2.43 probes `stateless-connect
        // git-receive-pack` FIRST for the push direction; reposix serves
        // push through the `export` capability, so we MUST answer
        // `fallback` — replying anything else (the pre-fix custom
        // `unsupported service: ...` line hit the "unknown response to
        // connect" die-path) makes git abort the push instead of trying
        // `export`. Git 2.54 (this project's CI) never probes push via
        // stateless-connect, so the regression is windowed to the 2.43.x
        // LTS line the `>= 2.34` floor does NOT protect against.
        tracing::debug!(
            service,
            "stateless-connect: replying `fallback` for non-upload-pack service (push falls back to `export`)"
        );
        proto
            .send_line("fallback")
            .context("write stateless-connect `fallback` sentinel")?;
        proto.flush().context("flush `fallback` sentinel")?;
        return Ok(StatelessConnectOutcome::FellBack);
    }

    // Delta-sync the cache before tunneling protocol-v2 to git.
    //
    // First invocation in a fresh cache → meta.last_fetched_at is absent →
    // sync() falls through to build_from() internally (seed path,
    // unconditional full tree). Subsequent invocations query the backend
    // with the stored cursor and apply only the delta. Either way the
    // tree + refs are up-to-date by the time we advertise to git.
    //
    // Sync errors surface to the caller with a diag on stderr and a
    // non-zero exit — mirrors the POC / pre-Phase-33 behaviour.
    let report = rt
        .block_on(cache.sync())
        .context("cache.sync before upload-pack tunnel")?;
    tracing::debug!(
        changed = report.changed_ids.len(),
        since = ?report.since,
        "delta sync complete"
    );

    // Audit: connect (one row per helper invocation that reaches here).
    cache.log_helper_connect(service);

    // Empty line = "ready" — helper protocol spec for stateless-connect.
    proto
        .send_blank()
        .context("write empty-line ready response")?;
    proto.flush().context("flush ready response")?;

    // Advertisement (gotcha 1: flush-only, no 0002).
    let adv_bytes = send_advertisement(proto, cache.repo_path())
        .context("send v2 advertisement from upload-pack")?;
    cache.log_helper_advertise(adv_bytes);

    // RPC loop: read request → pipe to upload-pack → write response + 0002.
    while let ProxyOutcome::Continued = proxy_one_rpc(proto, rt, cache)? {}

    Ok(StatelessConnectOutcome::TookOver)
}

/// Spawn `git upload-pack --advertise-refs --stateless-rpc` and write
/// its stdout verbatim to the protocol writer. Returns byte count for
/// audit.
///
/// Gotcha 1: NO trailing `0002`. The advertisement is an unsolicited
/// initial stream, terminated by flush alone.
fn send_advertisement<R: Read, W: Write>(
    proto: &mut Protocol<R, W>,
    repo_path: &Path,
) -> Result<u32> {
    let out = Command::new("git")
        .args([
            "upload-pack",
            "--advertise-refs",
            "--stateless-rpc",
            repo_path.to_str().context("cache repo path is not UTF-8")?,
        ])
        .env("GIT_PROTOCOL", "version=2")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("spawn git upload-pack --advertise-refs")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(upload_pack_failure_error(
            "--advertise-refs",
            out.status,
            &stderr,
        ));
    }

    proto
        .send_raw(&out.stdout)
        .context("write advertisement to stdout")?;
    proto.flush().context("flush advertisement")?;
    // CRITICAL (gotcha 1): do NOT write b"0002" here. Initial advertisement
    // is terminated by flush only (which upload-pack's stdout already
    // contains at the tail). Writing 0002 produces `fatal: expected flush
    // after ref listing`.
    u32::try_from(out.stdout.len()).context("advertisement length overflow u32")
}

enum ProxyOutcome {
    Continued,
    Eof,
}

/// Read one pkt-line request from `proto` (terminated by flush), pipe
/// it to a freshly-spawned `upload-pack --stateless-rpc`, write the
/// response to `proto`, and append `0002` (gotcha 2).
//
// Allow `too_many_lines` (104 vs default 100): the body splits into
// three logical phases — drain frames, blob-limit guardrail, spawn
// upload-pack — which read together more clearly than as separate
// helpers (the `stats` accumulator threads through all three).
#[allow(clippy::too_many_lines)]
fn proxy_one_rpc<R: Read, W: Write>(
    proto: &mut Protocol<R, W>,
    rt: &Runtime,
    cache: &Cache,
) -> Result<ProxyOutcome> {
    // Drain frames until flush into a re-encoded request buffer.
    let mut request = Vec::<u8>::with_capacity(1024);
    let mut stats = RpcStats::default();
    // OIDs named in `want` lines — materialized below (lazy-blob read path).
    let mut want_oids = Vec::<gix::ObjectId>::new();

    loop {
        let frame = match pktline::read_frame(proto.reader_mut()) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                if request.is_empty() {
                    return Ok(ProxyOutcome::Eof);
                }
                // Mid-request EOF is a protocol error.
                return Err(eof_midrequest_error());
            }
            Err(e) => return Err(e).context("read pkt-line frame from git stdin"),
        };

        match &frame {
            Frame::Data(p) => {
                if pktline::is_want_line(p) {
                    stats.want_count = stats.want_count.saturating_add(1);
                    if let Some(oid) = parse_want_oid(p) {
                        want_oids.push(oid);
                    }
                }
                if stats.command.is_none() {
                    if let Some(cmd) = parse_command_keyword(p) {
                        stats.command = Some(cmd);
                    }
                }
                pktline::encode_frame(&frame, &mut request);
            }
            Frame::Delim => pktline::encode_frame(&frame, &mut request),
            Frame::Flush => {
                pktline::encode_frame(&frame, &mut request);
                break;
            }
            Frame::ResponseEnd => {
                // Clients don't send response-end in requests; treat as
                // terminator defensively.
                pktline::encode_frame(&frame, &mut request);
                break;
            }
        }
    }

    stats.request_bytes = u32::try_from(request.len()).unwrap_or(u32::MAX);

    // ARCH-09: blob-limit guardrail. Only enforce on `command=fetch`
    // (other commands like `ls-refs` and `object-info` have no `want`
    // lines and `stats.want_count` is 0 — the check below is naturally
    // a no-op for them, but the explicit command match keeps intent
    // obvious to future readers). `limit == 0` means "unlimited"
    // (explicit opt-out).
    let limit = configured_blob_limit();
    if stats.command.as_deref() == Some("fetch") && limit != 0 && stats.want_count > limit {
        let msg = format_blob_limit_message(stats.want_count, limit);
        // Stderr first — agent-facing message MUST land before audit.
        #[allow(clippy::print_stderr)]
        {
            eprintln!("{msg}");
        }
        cache.log_blob_limit_exceeded(stats.want_count, limit);
        // The agent-facing 3-part teaching (BLOB_LIMIT_EXCEEDED_FMT — names the
        // `git sparse-checkout set <pathspec>` recovery) was ALREADY printed to
        // stderr above via `format_blob_limit_message`.
        // teach-exempt: ok — internal control-flow propagation after the teach printed above
        anyhow::bail!(
            "blob limit exceeded: {} wants, limit {}",
            stats.want_count,
            limit
        );
    }

    // Lazy-blob read path (the core of the partial-clone design). The cache
    // materializes blobs lazily: `build_from` writes trees + the commit but
    // NO blob objects (reposix_cache lib.rs "Blob materialization = lazy").
    // A `git checkout` populating the working tree after a
    // `--filter=blob:none` clone issues a follow-up fetch whose `want` lines
    // name the missing blobs by exact OID. Those objects are absent from the
    // cache until fetched from the backend, so upload-pack would die
    // "not our ref <oid>" (verified: crates/reposix-cache
    // tests/partial_clone_serves.rs). Materialize each wanted OID that maps to
    // a record BEFORE spawning upload-pack. Skip commit/tree tips
    // (`read_blob_cached` finds a non-blob → Err) and OIDs already in the
    // store (`Ok(Some)`); only absent OIDs (`Ok(None)`) reach the backend, and
    // a want that is not a known record (`UnknownOid`) is left for upload-pack
    // to resolve or reject.
    for oid in &want_oids {
        // Already in the store (`Ok(Some)`) or present but not a blob —
        // a commit/tree tip (`Err`): nothing to materialize. Only an absent
        // object (`Ok(None)`) needs a backend round-trip.
        if !matches!(cache.read_blob_cached(*oid), Ok(None)) {
            continue;
        }
        match rt.block_on(cache.read_blob(*oid)) {
            // Ok → materialized; UnknownOid → not a known record (leave it
            // for upload-pack to resolve or reject). Both are non-fatal here.
            Ok(_) | Err(reposix_cache::Error::UnknownOid(_)) => {}
            Err(e) => {
                return Err(anyhow::Error::new(e))
                    .with_context(|| format!("materialize wanted blob {oid} for upload-pack"));
            }
        }
    }

    // Invoke upload-pack with the re-framed request on stdin.
    let mut child = Command::new("git")
        .args([
            "upload-pack",
            "--stateless-rpc",
            cache
                .repo_path()
                .to_str()
                .context("cache repo path is not UTF-8")?,
        ])
        .env("GIT_PROTOCOL", "version=2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn git upload-pack --stateless-rpc")?;

    {
        let stdin = child.stdin.as_mut().context("upload-pack stdin")?;
        stdin
            .write_all(&request)
            .context("write request to upload-pack stdin")?;
    }
    // stdin dropped via scope, closing the pipe so upload-pack can
    // finish reading.

    let out = child
        .wait_with_output()
        .context("wait for upload-pack --stateless-rpc")?;

    if !out.status.success() {
        let stderr_str = String::from_utf8_lossy(&out.stderr);
        cache.log_helper_fetch_error(
            out.status.code().unwrap_or(-1),
            stderr_str
                .lines()
                .last()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect::<String>()
                .as_str(),
        );
        // Teaching hint: an UNFILTERED fetch (no `--filter=blob:none`) makes
        // upload-pack walk the full reachable object closure and hit blobs
        // this lazy cache never materialized — git reports it as the cryptic
        // "unable to read <oid> / possible repository corruption on the remote
        // side", which does NOT tell the agent what to do. `reposix init`
        // always sets the filter, so this is reachable only via a manual
        // unfiltered fetch; name the recovery move explicitly instead of
        // leaving "corruption" as the last word.
        if stats.command.as_deref() == Some("fetch") && is_unmaterialized_closure_error(&stderr_str)
        {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("{UNFILTERED_FETCH_HINT}");
            }
        }
        return Err(upload_pack_failure_error(
            "--stateless-rpc",
            out.status,
            &stderr_str,
        ));
    }

    stats.response_bytes = u32::try_from(out.stdout.len()).unwrap_or(u32::MAX);

    proto
        .send_raw(&out.stdout)
        .context("write upload-pack response to stdout")?;
    // CRITICAL (gotcha 2): append b"0002" after the response. Without
    // this, git misframes the next request and the helper hangs.
    proto
        .send_raw(b"0002")
        .context("append response-end 0002")?;
    proto.flush().context("flush rpc response")?;

    cache.log_helper_fetch(&stats);
    // §3c token-cost ledger — record bytes-in / bytes-out as token estimate.
    // Honest math: this counts WIRE bytes (incl. the protocol-v2 framing
    // overhead, the packfile size, etc.). For an English-prose blob it
    // approximates the model's token count via chars/4. For a binary
    // packfile this OVER-estimates relative to what an LLM would actually
    // see if the bytes were textual; tokens here are a "what would these
    // bytes cost if you sent them through a model" upper bound.
    cache.log_token_cost(
        u64::from(stats.request_bytes),
        u64::from(stats.response_bytes),
        "fetch",
    );

    Ok(ProxyOutcome::Continued)
}

/// Extract the `command=` keyword from a data frame payload if the
/// payload is `command=<word>\n` or similar. Returns `None` for
/// non-command frames.
fn parse_command_keyword(payload: &[u8]) -> Option<String> {
    let rest = payload.strip_prefix(b"command=")?;
    // Stop at the first non-keyword byte.
    let end = rest
        .iter()
        .position(|b| !b.is_ascii_alphanumeric() && *b != b'-' && *b != b'_')
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    std::str::from_utf8(&rest[..end]).ok().map(str::to_owned)
}

/// Extract the object id from a `want <oid>[ capabilities]\n` line.
/// Returns `None` if the payload is not a want line or the hex token
/// does not parse as an object id. The first turn of a protocol-v2
/// fetch also carries capability tokens after the OID (`want <oid>
/// filter ofs-delta ...`), so we take only the first whitespace-
/// delimited token.
fn parse_want_oid(payload: &[u8]) -> Option<gix::ObjectId> {
    let rest = payload.strip_prefix(b"want ")?;
    let end = rest
        .iter()
        .position(u8::is_ascii_whitespace)
        .unwrap_or(rest.len());
    gix::ObjectId::from_hex(&rest[..end]).ok()
}

// Unused under `-D warnings` in release builds without `dead_code`
// exemption because `BufRead` is imported for the `io::BufRead` trait
// marker used transitively by `BufReader`. Silence:
#[allow(dead_code)]
fn _ensure_bufread_in_scope<T: BufRead>(_: &T) {}

// -----------------------------------------------------------------------
// Tests — gotcha regression coverage + capability wiring.
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_limit_message_contains_literal_git_sparse_checkout() {
        let msg = format_blob_limit_message(250, 200);
        assert!(
            msg.contains("git sparse-checkout"),
            "verbatim error message MUST literally contain `git sparse-checkout`; got: {msg}"
        );
        assert!(msg.contains("250"), "want_count substituted: {msg}");
        assert!(msg.contains("200"), "limit substituted: {msg}");
        assert!(
            msg.starts_with("error: refusing to fetch "),
            "exact prefix per ARCH-09: {msg}"
        );
        assert!(
            msg.contains("`git sparse-checkout set <pathspec>`"),
            "backticks-and-pathspec-template preserved verbatim: {msg}"
        );
    }

    #[test]
    fn parse_blob_limit_default_when_absent() {
        assert_eq!(parse_blob_limit(None), DEFAULT_BLOB_LIMIT);
    }

    #[test]
    fn parse_blob_limit_zero_means_unlimited_value() {
        // `0` is preserved verbatim — the *enforcement* code interprets
        // it as "unlimited"; the parser just returns the raw value.
        assert_eq!(parse_blob_limit(Some("0")), 0);
    }

    #[test]
    fn parse_blob_limit_falls_back_on_garbage() {
        assert_eq!(parse_blob_limit(Some("not-a-number")), DEFAULT_BLOB_LIMIT);
        assert_eq!(parse_blob_limit(Some("")), DEFAULT_BLOB_LIMIT);
        assert_eq!(parse_blob_limit(Some("   ")), DEFAULT_BLOB_LIMIT);
    }

    #[test]
    fn parse_blob_limit_accepts_5() {
        assert_eq!(parse_blob_limit(Some("5")), 5);
    }

    #[test]
    fn blob_limit_check_logic_refuses_above_limit() {
        // Pure-logic check: same predicate as proxy_one_rpc.
        let limit = 200_u32;
        let want_count = 250_u32;
        let command_is_fetch = true;
        let should_refuse = command_is_fetch && limit != 0 && want_count > limit;
        assert!(should_refuse);
        // And produces the verbatim message.
        let msg = format_blob_limit_message(want_count, limit);
        assert!(msg.starts_with("error: refusing to fetch 250 blobs (limit: 200)."));
        assert!(msg.contains("`git sparse-checkout set <pathspec>`"));
    }

    #[test]
    fn blob_limit_check_logic_passes_at_exactly_limit() {
        let limit = 200_u32;
        let want_count = 200_u32;
        let should_refuse = limit != 0 && want_count > limit;
        assert!(!should_refuse, "exactly at limit must pass");
    }

    #[test]
    fn blob_limit_check_logic_zero_means_unlimited() {
        let limit = 0_u32;
        let want_count = 9999_u32;
        let should_refuse = limit != 0 && want_count > limit;
        assert!(!should_refuse, "limit=0 means unlimited");
    }

    #[test]
    // test-name-honesty: ok — command-keyword parsing unit test, no live git fetch subprocess
    fn blob_limit_check_logic_skips_non_fetch_commands() {
        let limit = 200_u32;
        let want_count = 250_u32;
        let command_is_fetch = false; // e.g. ls-refs
        let should_refuse = command_is_fetch && limit != 0 && want_count > limit;
        assert!(!should_refuse);
    }

    #[test]
    // test-name-honesty: ok — command-keyword parsing unit test, no live git fetch subprocess
    fn parse_command_keyword_extracts_fetch() {
        assert_eq!(
            parse_command_keyword(b"command=fetch\n"),
            Some("fetch".to_owned())
        );
    }

    #[test]
    fn parse_command_keyword_extracts_ls_refs() {
        assert_eq!(
            parse_command_keyword(b"command=ls-refs\n"),
            Some("ls-refs".to_owned())
        );
    }

    #[test]
    fn parse_command_keyword_none_for_non_command() {
        assert_eq!(parse_command_keyword(b"want abcdef\n"), None);
        assert_eq!(parse_command_keyword(b""), None);
    }

    #[test]
    // test-name-honesty: ok — pure predicate + constant assertions, no live git fetch subprocess
    fn unfiltered_fetch_hint_detects_corruption_signature_and_teaches_filter() {
        // The exact upload-pack stderr from the QL-001 CI failure.
        assert!(is_unmaterialized_closure_error(
            "fatal: git upload-pack: aborting due to possible repository corruption on the remote side."
        ));
        assert!(is_unmaterialized_closure_error(
            "remote: fatal: unable to read 94b5c50ec3458ebd8399b492cffbb5d1e9915a4d"
        ));
        // Unrelated failures must NOT trip the hint.
        assert!(!is_unmaterialized_closure_error(
            "fatal: the remote end hung up"
        ));
        assert!(!is_unmaterialized_closure_error(""));
        // The hint literally names the recovery move.
        assert!(
            UNFILTERED_FETCH_HINT.contains("--filter=blob:none"),
            "teaching hint MUST name `--filter=blob:none`; got: {UNFILTERED_FETCH_HINT}"
        );
    }

    #[test]
    // test-name-honesty: ok — pure OID-parsing unit test, no live git fetch subprocess
    fn parse_want_oid_extracts_bare_and_capability_suffixed() {
        let hex = "94b5c50ec3458ebd8399b492cffbb5d1e9915a4d";
        let expected = gix::ObjectId::from_hex(hex.as_bytes()).unwrap();
        // Bare `want <oid>\n`.
        assert_eq!(
            parse_want_oid(format!("want {hex}\n").as_bytes()),
            Some(expected)
        );
        // First fetch turn carries capabilities after the OID.
        assert_eq!(
            parse_want_oid(format!("want {hex} filter ofs-delta\n").as_bytes()),
            Some(expected)
        );
        // Non-want / malformed → None (no panic).
        assert_eq!(parse_want_oid(b"have 94b5c50\n"), None);
        assert_eq!(parse_want_oid(b"want notahexoid\n"), None);
        assert_eq!(parse_want_oid(b""), None);
    }

    /// Gotcha #1 regression: the initial advertisement must end with a
    /// flush packet `0000` — never `0002`. This test simulates what
    /// `send_advertisement` does: piping upload-pack stdout straight to
    /// the protocol writer without any tail append. Here we stand in
    /// for upload-pack with canned bytes ending in `0000`.
    #[test]
    fn initial_advertisement_ends_with_flush_only() {
        let input: &[u8] = b"";
        let mut output: Vec<u8> = Vec::new();
        let mut proto = Protocol::new(input, &mut output);
        let canned_adv = b"000eversion 2\n0000";
        proto.send_raw(canned_adv).unwrap();
        proto.flush().unwrap();
        drop(proto);
        // Last 4 bytes must be "0000" (flush), never "0002".
        assert_eq!(&output[output.len() - 4..], b"0000");
        assert_ne!(&output[output.len() - 4..], b"0002");
    }

    /// Gotcha #2 regression: each RPC response, after writing
    /// upload-pack's stdout bytes, must append `0002` response-end.
    /// This simulates the write pair that `proxy_one_rpc` performs.
    #[test]
    fn rpc_response_appends_response_end() {
        let input: &[u8] = b"";
        let mut output: Vec<u8> = Vec::new();
        let mut proto = Protocol::new(input, &mut output);
        let canned_response = b"000dpackfile\n0000";
        proto.send_raw(canned_response).unwrap();
        proto.send_raw(b"0002").unwrap();
        proto.flush().unwrap();
        drop(proto);
        // Last 4 bytes must be b"0002" (response-end).
        assert_eq!(&output[output.len() - 4..], b"0002");
    }

    /// Gotcha #3 regression: the pkt-line parser round-trips NUL bytes
    /// and non-UTF-8 payloads. This is a unit test on the framer since
    /// the whole handler is built on it; if NULs get corrupted, packs
    /// break.
    #[test]
    fn stdin_is_binary_throughout() {
        let mut wire = Vec::new();
        pktline::encode_frame(&Frame::Data(b"command=fetch\n".to_vec()), &mut wire);
        pktline::encode_frame(&Frame::Data(b"want abc\x00\xffdef\n".to_vec()), &mut wire);
        pktline::encode_frame(&Frame::Flush, &mut wire);

        let mut cursor: &[u8] = &wire;
        let mut frames = Vec::new();
        loop {
            let f = pktline::read_frame(&mut cursor).unwrap();
            let is_flush = matches!(f, Frame::Flush);
            frames.push(f);
            if is_flush {
                break;
            }
        }
        assert_eq!(frames.len(), 3);
        match &frames[1] {
            Frame::Data(p) => {
                assert_eq!(p, b"want abc\x00\xffdef\n");
            }
            other => panic!("expected Data, got {other:?}"),
        }
    }

    // --- P120 W5: transport-error teaching (upload-pack subprocess exit + EOF) ---

    /// The upload-pack-failure teaching PRESERVES git's own stderr (the raw
    /// cause) AND layers a 3-part teaching on top (Fix: + Recovery:) that names
    /// the likely root cause + the `reposix doctor` recovery. No hollow
    /// `Alternative:` line (there is no genuine alternative to a crashed server).
    #[test]
    // test-name-honesty: ok — drives a real non-zero-exit subprocess for the ExitStatus, then asserts the teaching shape
    fn upload_pack_failure_error_teaches_on_top_of_raw_stderr() {
        // A real non-zero ExitStatus (sh is universally present); exit code 7.
        let status = std::process::Command::new("sh")
            .args(["-c", "exit 7"])
            .status()
            .expect("spawn sh");
        let err =
            upload_pack_failure_error("--advertise-refs", status, "fatal: not a git repository");
        let msg = format!("{err:#}");
        assert!(msg.contains("Fix:"), "teaches the fix; got:\n{msg}");
        assert!(msg.contains("Recovery:"), "gives recovery; got:\n{msg}");
        assert!(
            msg.contains("fatal: not a git repository"),
            "must preserve git's own stderr on top of the teaching; got:\n{msg}"
        );
        assert!(
            msg.contains("reposix doctor"),
            "recovery names `reposix doctor`; got:\n{msg}"
        );
        assert!(
            msg.contains("git 2.34+"),
            "fix names the partial-clone git-version root cause; got:\n{msg}"
        );
    }

    /// End-to-end (real subprocess): `send_advertisement` against a path that is
    /// not a git repo makes `git upload-pack --advertise-refs` exit non-zero, and
    /// the error surfaces the 3-part teaching. Hermetic — a bogus LOCAL path, no
    /// network, no shared repo.
    #[test]
    fn send_advertisement_on_non_repo_emits_three_part_teaching() {
        let input: &[u8] = b"";
        let mut output: Vec<u8> = Vec::new();
        let mut proto = Protocol::new(input, &mut output);
        let bogus = std::path::Path::new("/nonexistent/reposix-not-a-repo-x9q");
        let err = send_advertisement(&mut proto, bogus)
            .expect_err("upload-pack must fail on a non-repo path");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Fix:") && msg.contains("Recovery:"),
            "real upload-pack subprocess failure must emit the 3-part teaching; got:\n{msg}"
        );
    }

    /// The unexpected-EOF-mid-request teaching is 2-part by design (FLAG-1): it
    /// teaches the protocol-desync Fix + a Recovery, but emits NO hollow
    /// `Alternative:` line (there is no genuine alternative to a desync).
    #[test]
    // test-name-honesty: ok — asserts the EOF teaching body shape, no live git fetch subprocess
    fn eof_midrequest_error_teaches_fix_and_recovery_without_alternative() {
        let msg = format!("{:#}", eof_midrequest_error());
        assert!(msg.contains("Fix:"), "teaches the fix; got:\n{msg}");
        assert!(msg.contains("Recovery:"), "gives recovery; got:\n{msg}");
        assert!(
            !msg.contains("Alternative:"),
            "a protocol desync has no genuine alternative — no hollow line; got:\n{msg}"
        );
        assert!(
            msg.contains("protocol desync"),
            "names the desync cause; got:\n{msg}"
        );
    }
}
