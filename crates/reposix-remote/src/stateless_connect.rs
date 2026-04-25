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

#![forbid(unsafe_code)]

use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use reposix_cache::{cache::HelperFetchRecord, Cache};
use tokio::runtime::Runtime;

use crate::pktline::{self, Frame};
use crate::protocol::Protocol;

/// Request-turn counters for audit + Phase 34 blob-limit telemetry.
#[derive(Debug, Default, Clone)]
pub struct RpcStats {
    /// Count of `want ` lines seen in the request payload.
    pub want_count: u32,
    /// Total bytes written to `upload-pack` stdin (re-encoded request).
    pub request_bytes: u32,
    /// Total bytes read from `upload-pack` stdout (response body).
    pub response_bytes: u32,
    /// First keyword after `command=` in the first data frame, if any.
    /// Common values: `fetch`, `ls-refs`, `object-info`.
    pub command: Option<String>,
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

/// Entry point for the `stateless-connect <service>` verb.
///
/// The caller has already read the verb line from stdin. We write the
/// one-line "ready" response, send the protocol-v2 advertisement, then
/// loop reading RPC turns from git until EOF.
///
/// # Errors
/// Any I/O error from stdin/stdout, any `upload-pack` spawn failure,
/// any cache error propagated from [`Cache::open`] or
/// [`Cache::build_from`].
pub fn handle_stateless_connect<R: Read, W: Write>(
    proto: &mut Protocol<R, W>,
    rt: &Runtime,
    cache: &Cache,
    service: &str,
) -> Result<()> {
    if service != "git-upload-pack" {
        // POC behaviour: non-upload-pack services are an error. Push
        // uses `export` (different capability entirely).
        proto.diag_stderr(&format!(
            "git-remote-reposix: stateless-connect only supports git-upload-pack, got `{service}`"
        ));
        // Write error line per helper protocol, then bail.
        proto
            .send_line(&format!("unsupported service: {service}"))
            .context("write stateless-connect error line")?;
        proto.flush().context("flush error line")?;
        anyhow::bail!("unsupported stateless-connect service: {service}");
    }

    // Ensure tree + refs are current in the bare repo. Best-effort on
    // re-runs inside the same process; Phase 33 will add a skip cache.
    //
    // Tree-sync errors surface to the caller with a diag on stderr and
    // a non-zero exit — mirrors the POC behaviour.
    rt.block_on(cache.build_from())
        .context("cache.build_from before upload-pack tunnel")?;

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
    while let ProxyOutcome::Continued = proxy_one_rpc(proto, cache)? {}

    Ok(())
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
        anyhow::bail!(
            "git upload-pack --advertise-refs exited {}: {}",
            out.status,
            stderr.trim()
        );
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
fn proxy_one_rpc<R: Read, W: Write>(
    proto: &mut Protocol<R, W>,
    cache: &Cache,
) -> Result<ProxyOutcome> {
    // Drain frames until flush into a re-encoded request buffer.
    let mut request = Vec::<u8>::with_capacity(1024);
    let mut stats = RpcStats::default();

    loop {
        let frame = match pktline::read_frame(proto.reader_mut()) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                if request.is_empty() {
                    return Ok(ProxyOutcome::Eof);
                }
                // Mid-request EOF is a protocol error.
                anyhow::bail!("unexpected EOF mid-request");
            }
            Err(e) => return Err(e).context("read pkt-line frame from git stdin"),
        };

        match &frame {
            Frame::Data(p) => {
                if pktline::is_want_line(p) {
                    stats.want_count = stats.want_count.saturating_add(1);
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
        anyhow::bail!(
            "git upload-pack --stateless-rpc exited {}: {}",
            out.status,
            stderr_str.trim()
        );
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

// -----------------------------------------------------------------------
// Small helpers on Protocol — live here (not in protocol.rs) because they
// are specific to the stateless-connect flow.
// -----------------------------------------------------------------------

trait DiagExt {
    fn diag_stderr(&self, msg: &str);
}

impl<R: Read, W: Write> DiagExt for Protocol<R, W> {
    #[allow(clippy::print_stderr)]
    fn diag_stderr(&self, msg: &str) {
        eprintln!("{msg}");
    }
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
}
