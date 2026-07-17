//! Bus remote handler — dispatch surface for
//! `reposix::<sot>?mirror=<mirror>` URLs (DVCS-BUS-URL-01,
//! DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01).
//!
//! ## Algorithm (architecture-sketch.md § 3 steps 1-3)
//!
//! On the `export` verb (BEFORE reading stdin):
//!
//! - **STEP 0 — resolve local mirror remote name.** Q-A / D-01: scan
//!   `git config --get-regexp '^remote\..+\.url$'`, byte-equal-match
//!   values to `mirror_url` (with trailing-slash normalization),
//!   pick first alphabetically + WARN if multiple. Zero matches →
//!   emit Q3.5 hint and exit BEFORE PRECHECK A.
//! - **PRECHECK A — mirror drift.** `git ls-remote -- <mirror_url>
//!   refs/heads/main` shell-out (D-06). Compare returned SHA to
//!   local `git rev-parse refs/remotes/<name>/main`. Drifted →
//!   emit `error refs/heads/main fetch first` + hint, bail. NO
//!   confluence work. NO stdin read.
//! - **PRECHECK B — `SoT` drift.** [`crate::precheck::precheck_sot_drift_any`]
//!   (T03 substrate). Drifted → emit `error refs/heads/main fetch first`
//!   plus hint citing `refs/mirrors/<sot>-synced-at` (when populated
//!   by P80), bail. NO stdin read.
//!
//! On the `export` verb (AFTER P82's prechecks pass — steps 4-9 of
//! the architecture-sketch's bus algorithm, P83-01 T04): // banned-words: ok
//!
//! - **Step 4 — read fast-import stream from stdin** (verbatim
//!   `parse_export_stream` — same parser `handle_export` uses).
//! - **Step 5 — apply REST writes to `SoT`** via the shared
//!   [`crate::write_loop::apply_writes`] (T02 lift). On `SotOk` the
//!   `refs/mirrors/<sot>-head` ref + `helper_push_accepted` audit row +
//!   `last_fetched_at` cursor are advanced inside `apply_writes`. The
//!   caller (this module) decides what happens to `synced-at` /
//!   `mirror_sync_written` / `log_token_cost` / the `ok` ack.
//! - **Step 6 — `git push <mirror_remote_name> main`** via the
//!   [`push_mirror`] helper. Plain push — NO `--force-with-lease`
//!   (D-08 RATIFIED; P84 owns force-with-lease for the webhook race).
//!   NO retry (Q3.6 RATIFIED — surface, audit, recover on next push or
//!   webhook sync).
//! - **Step 7 — branch on (`WriteOutcome`, `MirrorResult`):** see the
//!   three terminal branches below.
//!
//! On `SotOk` and `MirrorResult::Ok`: write `refs/mirrors/<sot>-synced-at`,
//! write the `mirror_sync_written` audit row, write the `token_cost` row,
//! and emit `ok refs/heads/main` to git.
//!
//! On `SotOk` and `MirrorResult::Failed`: do NOT write `synced-at`
//! (FROZEN at last successful mirror sync — observable lag for the
//! vanilla-`git`-only operator), write the
//! `helper_push_partial_fail_mirror_lag` audit row, write the `token_cost`
//! row, emit stderr WARN, and emit `ok refs/heads/main` to git (Q3.6
//! contract).
//!
//! On non-`SotOk`: mirror push NEVER attempted; reject lines and audit
//! rows already emitted inside `apply_writes`; `state.push_failed` is
//! set to `true`; return cleanly.
//!
//! ## Cwd assumption (Pitfall 6)
//!
//! `git push <mirror_remote_name> main` inherits the helper's cwd (the
//! working tree git invoked the helper from). This is the same git
//! invocation context that resolved `<mirror_remote_name>` in P82's
//! STEP 0; the cwd is implicit but consistent. Tests use temp working
//! trees with `current_dir(...)` set explicitly.
//!
//! ## Confluence non-atomicity (D-09 / Pitfall 3)
//!
//! REST writes via [`crate::write_loop::apply_writes`] are NOT 2PC
//! across actions. A multi-action push that fails mid-loop (PATCH 1
//! succeeds, PATCH 2 fails) leaves `SoT` in a partial state observable
//! to the next push. Recovery is the next-push PRECHECK B reading new
//! `SoT` state via `list_changed_since` and either accepting the local
//! change (if version still matches) or rejecting with conflict.
//!
//! ## Security (T-82-01)
//!
//! `mirror_url` is user-controlled (argv[2]'s bus URL). The
//! `git ls-remote` shell-out mitigates argument injection via:
//! - Reject `mirror_url` whose first byte is `-` BEFORE shell-out.
//! - `--` separator unconditionally before the URL argument.
//! - Byte-pass — no template expansion / shell interpretation.
//!
//! The `git config --get-regexp` regex is helper-controlled (no user
//! input flows to the regex). The `git rev-parse` shell-out's
//! `<name>` is bounded by git's own remote-name validation
//! (config-key match against `^remote\.([^.]+)\.url$`).

use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::precheck::{precheck_sot_drift_any, SotDriftOutcome};
use crate::protocol::Protocol;
use crate::State;

/// Mirror-drift outcome from PRECHECK A.
#[derive(Debug, Clone)]
enum MirrorDriftOutcome {
    /// Local `refs/remotes/<name>/main` matches `git ls-remote`'s
    /// returned SHA, OR `git ls-remote` returned nothing (empty
    /// mirror — no drift possible; P84 handles first-push to empty).
    Stable,
    /// Local SHA differs from remote SHA.
    Drifted { local: String, remote: String },
}

/// Bus-mode export handler — dispatches the algorithm above.
///
/// Called from `main.rs`'s `"export"` arm when `state.mirror_url.is_some()`.
/// Emits stdout/stderr per the architecture-sketch's bus algorithm
/// steps 1-9: P82 shipped steps 1-3 (URL parse, prechecks A + B);
/// P83-01 shipped steps 4-9 (read stdin, apply REST writes via // banned-words: ok
/// [`crate::write_loop::apply_writes`], `git push` mirror via
/// [`push_mirror`], branch on `(WriteOutcome, MirrorResult)` for ref
/// + audit + ack writes).
///
/// # Errors
/// All errors are [`anyhow::Error`]. Reject paths reuse the existing
/// `crate::fail_push` shape via the bus handler's local
/// `bus_fail_push` helper.
#[allow(clippy::too_many_lines)] // narrow steps 1-9; readability beats split fns here
pub(crate) fn handle_bus_export<R: std::io::Read, W: std::io::Write>(
    state: &mut State,
    proto: &mut Protocol<R, W>,
) -> Result<()> {
    let mirror_url = state
        .mirror_url
        .clone()
        .expect("handle_bus_export called without mirror_url; main.rs dispatch invariant violated");

    // T-82-01: reject `-`-prefixed mirror URLs BEFORE any shell-out.
    if mirror_url.starts_with('-') {
        // Redact any embedded userinfo before echoing (Wave-5.5).
        let (mirror_display, _) = reposix_core::http::strip_url_userinfo(&mirror_url);
        return bus_fail_push(
            proto,
            state,
            "bad-mirror-url",
            &format!("mirror URL cannot start with `-`: {mirror_display}"),
        );
    }

    // QL-006 — mirror-egress allowlist gate. The bus push shells out
    // `git ls-remote <mirror>` (PRECHECK A) and `git push` (STEP 6), both
    // of which send issue content to the mirror host. Per the threat model
    // (CLAUDE.md § Threat model) every outbound channel that can carry
    // tainted issue content must be gated by REPOSIX_ALLOWED_ORIGINS. Run
    // this BEFORE STEP 0 / PRECHECK A / any stdin read — a non-allowlisted
    // mirror never touches the network. Local mirrors (`file://`, bare
    // paths) are exempt: no egress leaves the machine.
    match crate::mirror_egress::check_mirror_allowed(&mirror_url)
        .context("evaluate mirror egress allowlist")?
    {
        Ok(()) => {}
        Err(denied) => {
            crate::diag(&denied.teaching_message());
            return bus_fail_push(
                proto,
                state,
                "egress-denied",
                &format!("mirror egress blocked: {}", denied.origin),
            );
        }
    }

    // STEP 0 — resolve local mirror remote name (Q-A / D-01).
    let Some(mirror_remote_name) = resolve_mirror_remote_name(&mirror_url)? else {
        // Q3.5 RATIFIED: emit the verbatim hint, do NOT auto-mutate
        // the user's git config. NO PRECHECK A run.
        // Redact any embedded userinfo before echoing the URL (Wave-5.5
        // credential-leak intake — stderr is an exfiltration leg).
        let (mirror_display, _) = reposix_core::http::strip_url_userinfo(&mirror_url);
        return bus_fail_push(
            proto,
            state,
            "no-mirror-remote",
            &format!("configure the mirror remote first: `git remote add <name> {mirror_display}`"),
        );
    };

    // PRECHECK A — mirror drift (DVCS-BUS-PRECHECK-01).
    match precheck_mirror_drift(&mirror_url, &mirror_remote_name)? {
        MirrorDriftOutcome::Stable => {}
        MirrorDriftOutcome::Drifted { local, remote } => {
            // Per architecture-sketch step 2 + RESEARCH.md Pattern 3:
            // emit the canned `error refs/heads/main fetch first`
            // status string on stdout (git's standard form;
            // `git pull --rebase` will be suggested by git), and the
            // human hint on stderr.
            // Redacted display form — never echo embedded userinfo (Wave-5.5).
            let (mirror_display, _) = reposix_core::http::strip_url_userinfo(&mirror_url);
            crate::diag(&format!(
                "your GH mirror has new commits: \
                 local refs/remotes/{mirror_remote_name}/main = {local}; \
                 remote {mirror_display} HEAD = {remote}"
            ));
            crate::diag(&format!(
                "hint: run `git fetch {mirror_remote_name}` first, \
                 then retry the push"
            ));
            return bus_fail_push(
                proto,
                state,
                "fetch first",
                "mirror drift detected (PRECHECK A)",
            );
        }
    }

    // PRECHECK B — SoT drift (DVCS-BUS-PRECHECK-02).
    //
    // Lazy-open cache like `handle_export` does — best-effort.
    // PRECHECK B's no-cursor path returns Stable, so a cache-open
    // failure (non-fatal) collapses to "first-push policy" via the
    // wrapper's `cache: None` arm.
    if let Err(e) = crate::ensure_cache(state) {
        // Cache-open failure is non-fatal for the push itself (PRECHECK B
        // degrades to first-push policy), but it silently disables ALL
        // OP-3 bookkeeping for this push (helper_push_* audit rows,
        // refs/mirrors/<sot>-{head,synced-at}, token_cost). Never swallow
        // that silently — say so on stderr with the cause.
        crate::diag(&format!(
            "warning: reposix cache unavailable for this push — audit rows and \
             refs/mirrors/* will NOT be recorded. Cause: {e:#}"
        ));
    }
    let drift = precheck_sot_drift_any(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.project,
        &state.rt,
    )
    .context("PRECHECK B failed")?;

    if let SotDriftOutcome::Drifted { changed_count } = drift {
        let sot = state.backend_name.clone();
        crate::diag(&format!(
            "{sot} has {changed_count} change(s) since your last fetch (PRECHECK B)"
        ));
        // Cite `refs/mirrors/<sot>-synced-at` when populated by P80.
        // First-push case (refs absent): omit the hint cleanly.
        if let Some(cache) = state.cache.as_ref() {
            if let Ok(Some(synced_at)) = cache.read_mirror_synced_at(&state.backend_name) {
                let ago = chrono::Utc::now().signed_duration_since(synced_at);
                let mins = ago.num_minutes().max(0);
                crate::diag(&format!(
                    "hint: GH mirror was last synced from {sot} at {ts} \
                     ({mins} minutes ago); see refs/mirrors/{sot}-synced-at",
                    ts = synced_at.to_rfc3339(),
                ));
            }
        }
        crate::diag("hint: run `git pull --rebase` to incorporate backend changes, then retry");
        return bus_fail_push(
            proto,
            state,
            "fetch first",
            "SoT drift detected (PRECHECK B)",
        );
    }

    // STEPS 4-9 — write fan-out (P83-01 T04 / D-01 / D-08 / Q3.6 / D-09). // banned-words: ok
    //
    // PRECHECK B passed. Now: read stdin, write SoT, push mirror,
    // branch on outcomes, ack git.
    let parsed = {
        let mut buffered = std::io::BufReader::new(crate::ProtoReader::new(proto));
        let parse_result = crate::fast_import::parse_export_stream(&mut buffered);
        drop(buffered);
        match parse_result {
            Ok(v) => v,
            Err(e) => {
                return bus_fail_push(
                    proto,
                    state,
                    "parse-error",
                    &format!("parse export stream: {e:#}"),
                );
            }
        }
    };

    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_started("refs/heads/main");
    }

    let outcome = crate::write_loop::apply_writes(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.backend_name,
        &state.project,
        &state.rt,
        proto,
        &parsed, // borrow per B1 — apply_writes takes &ParsedExport
    )?;

    let crate::write_loop::WriteOutcome::SotOk { sot_sha, .. } = outcome else {
        // apply_writes already emitted the protocol error + audit rows.
        // Mirror push NEVER attempted on any non-SotOk outcome.
        state.push_failed = true;
        return Ok(());
    };

    let mirror_result = push_mirror(&mirror_remote_name)?;

    // chars_in is the same in both arms: count of all blob payload bytes
    // from the fast-import stream. chars_out is the count of stdout bytes
    // ack'd to git (the `ok refs/heads/main\n` line) — emitted in BOTH
    // arms per Q3.6 contract; stderr (the WARN on the failure arm) is
    // NOT counted in chars_out, keeping the token_cost ledger consistent
    // across success and partial-fail (M4).
    let chars_in: u64 = parsed
        .blobs
        .values()
        .map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX))
        .sum();
    let chars_out: u64 = "ok refs/heads/main\n".len() as u64;

    match mirror_result {
        MirrorResult::Ok => {
            if let Some(cache) = state.cache.as_ref() {
                if let Err(e) =
                    cache.write_mirror_synced_at(&state.backend_name, chrono::Utc::now())
                {
                    tracing::warn!("write_mirror_synced_at failed: {e:#}");
                }
                let oid_hex = sot_sha.map(|o| o.to_hex().to_string()).unwrap_or_default();
                cache.log_mirror_sync_written(&oid_hex, &state.backend_name);
                cache.log_token_cost(chars_in, chars_out, "push");
            }
            proto.send_line("ok refs/heads/main")?;
            proto.send_blank()?;
            proto.flush()?;
        }
        MirrorResult::Failed {
            exit_code,
            stderr_tail,
        } => {
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string()).unwrap_or_default();
            // WR-01 (P120): redact ONCE, then fan out to BOTH OP-3 sinks
            // (audit row + operator diag). See `record_mirror_partial_fail`.
            record_mirror_partial_fail(
                state.cache.as_ref(),
                &oid_hex,
                exit_code,
                &stderr_tail,
                chars_in,
                chars_out,
            );
            proto.send_line("ok refs/heads/main")?;
            proto.send_blank()?;
            proto.flush()?;
        }
    }
    Ok(())
}

/// STEP 0 helper. Returns the local remote name whose `.url` value
/// byte-equals `mirror_url` (with trailing-slash + embedded-userinfo
/// normalization), or `None` if zero matches. Picks first alphabetical +
/// emits stderr WARNING if multiple matches (Pitfall 4 / D-01).
///
/// Userinfo normalization (Wave-5.5): `reposix attach` strips embedded
/// credentials from the `?mirror=` param before persisting it, but the
/// LOCAL remote's own URL may still carry `user:token@` (a token-in-URL
/// clone). Both sides are stripped before comparing so the credential-
/// bearing local remote still resolves against the cred-free bus param.
fn resolve_mirror_remote_name(mirror_url: &str) -> Result<Option<String>> {
    let out = Command::new("git")
        .args(["config", "--get-regexp", r"^remote\..+\.url$"])
        .output()
        .context("spawn `git config --get-regexp`")?;
    // Exit code 1 from `git config --get-regexp` means "no match" —
    // not an error from our perspective. Higher exit codes are real
    // failures.
    if !out.status.success() {
        let exit = out.status.code().unwrap_or(-1);
        if exit == 1 {
            return Ok(None);
        }
        // `git config --get-regexp` exited with >1 (not the "no match" exit-1 handled
        // above) — git's own diagnostic for a local config read that failed
        // pathologically (disk/permission), not a workflow the user drives.
        // teach-exempt: ok — internal subprocess failure, not a user-actionable teaching target
        return Err(anyhow!(
            "`git config --get-regexp` exited {exit}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let (mirror_stripped, _) = reposix_core::http::strip_url_userinfo(mirror_url);
    let mirror_norm = mirror_stripped.trim_end_matches('/').to_owned();
    let mut matched: Vec<String> = Vec::new();
    for line in stdout.lines() {
        // Each line: `remote.<name>.url <value>`. Use splitn(2, ...)
        // because URL values may contain whitespace (rare but legal).
        let mut parts = line.splitn(2, char::is_whitespace);
        let Some(key) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next() else {
            continue;
        };
        let (value_stripped, _) = reposix_core::http::strip_url_userinfo(value);
        let value_norm = value_stripped.trim_end_matches('/');
        if value_norm != mirror_norm {
            continue;
        }
        let Some(name) = key
            .strip_prefix("remote.")
            .and_then(|s| s.strip_suffix(".url"))
        else {
            continue;
        };
        matched.push(name.to_owned());
    }

    matched.sort();
    match matched.len() {
        0 => Ok(None),
        1 => Ok(Some(matched.into_iter().next().unwrap())),
        _ => {
            let chosen = matched.first().cloned().unwrap();
            // mirror_norm (cred-stripped) — never echo raw userinfo to stderr.
            crate::diag(&format!(
                "warning: multiple local remotes point at {mirror_norm}: {matched:?}; \
                 picking first alphabetical (`{chosen}`)"
            ));
            Ok(Some(chosen))
        }
    }
}

/// PRECHECK A helper (DVCS-BUS-PRECHECK-01).
///
/// Shells out `git ls-remote -- <mirror_url> refs/heads/main`,
/// compares the returned SHA to `git rev-parse
/// refs/remotes/<name>/main`. Empty `git ls-remote` output → Stable.
fn precheck_mirror_drift(mirror_url: &str, mirror_remote_name: &str) -> Result<MirrorDriftOutcome> {
    // T-82-01: `--` separator unconditionally; mirror_url is byte-passed.
    let out = Command::new("git")
        .args(["ls-remote", "--", mirror_url, "refs/heads/main"])
        .output()
        .context("spawn `git ls-remote`")?;
    if !out.status.success() {
        // SECURITY (P120 W5 / T-120-02): the mirror URL may embed `user:token@`
        // userinfo — redact it BEFORE it reaches stderr (an exfiltration leg;
        // CLAUDE.md § Threat model). Sibling reject paths in this file already
        // strip; this one used to echo raw. The recovery command references the
        // mirror by its git REMOTE NAME (creds-free AND runnable — never the
        // redacted URL, which is not a valid remote spec).
        let safe_url = crate::backend_dispatch::redact_userinfo(mirror_url);
        // Redact git's OWN stderr too: modern git (2.34+) strips userinfo from its
        // "unable to access '<url>'" line, but an older git could echo it — make the
        // no-leak guarantee independent of the git version, not reliant on it.
        let stderr = String::from_utf8_lossy(&out.stderr);
        let safe_stderr = crate::backend_dispatch::redact_userinfo(stderr.trim());
        let headline = format!("git ls-remote {safe_url} failed: {safe_stderr}");
        let ls_cmd = format!("git ls-remote {mirror_remote_name}   # confirm the mirror answers");
        return Err(anyhow!(
            "{}",
            reposix_core::errmsg::teach(
                &headline,
                "the mirror is unreachable or misconfigured — confirm the mirror \
                 URL is correct and the host is reachable",
                "push to the SoT alone by dropping `?mirror=<url>` from the remote \
                 URL (the mirror fan-out is best-effort)",
                &[
                    ls_cmd.as_str(),
                    "# mirror recovery playbook: docs/guides/dvcs-mirror-setup.md",
                ],
            )
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let remote_sha = stdout.split_whitespace().next().unwrap_or("").to_owned();
    if remote_sha.is_empty() {
        // Empty mirror — no drift possible. P84 webhook sync handles
        // first-push-to-empty-mirror via separate code path.
        return Ok(MirrorDriftOutcome::Stable);
    }

    // Local SHA via `git rev-parse refs/remotes/<name>/main` (handles
    // packed-refs correctly; raw fs reads of `.git/refs/remotes/<name>/main`
    // would miss them — RESEARCH.md § "Don't Hand-Roll").
    let local_ref = format!("refs/remotes/{mirror_remote_name}/main");
    let out = Command::new("git")
        .args(["rev-parse", &local_ref])
        .output()
        .with_context(|| format!("spawn `git rev-parse {local_ref}`"))?;
    if !out.status.success() {
        // No local ref — treat as Drifted (the user has a remote URL
        // configured but never fetched). Reject path will tell them
        // to fetch.
        return Ok(MirrorDriftOutcome::Drifted {
            local: format!("(no local ref {local_ref})"),
            remote: remote_sha,
        });
    }
    let local_sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();

    if local_sha == remote_sha {
        Ok(MirrorDriftOutcome::Stable)
    } else {
        Ok(MirrorDriftOutcome::Drifted {
            local: local_sha,
            remote: remote_sha,
        })
    }
}

/// Outcome of the mirror push subprocess (`git push <mirror_remote_name>
/// main`). Pattern 2 of `83-RESEARCH.md`. The non-zero-exit case is
/// `Failed`, NOT a propagated error — `bus_handler::handle_bus_export`
/// branches on this enum to write the partial-fail audit row + still
/// ack `ok` to git per Q3.6.
#[derive(Debug)]
enum MirrorResult {
    /// `git push <mirror_remote_name> main` exited zero.
    Ok,
    /// Non-zero exit. `stderr_tail` is the last <= 3 lines of the
    /// subprocess stderr (T-83-02 — bound the operator-readable
    /// info-leak surface). `exit_code` is the process exit code
    /// (`-1` if signaled — `Command::ExitStatus::code()` returns
    /// `None` on signal termination on Unix).
    Failed { exit_code: i32, stderr_tail: String },
}

/// Run `git push <mirror_remote_name> main` from the helper's cwd
/// (Pitfall 6 — the working tree git invoked the helper from). NO
/// RETRY (Q3.6 RATIFIED — surface, no helper-side retry; user retries
/// the whole push or webhook sync recovers). NO `--force-with-lease`
/// (D-08 RATIFIED — P84 owns force-with-lease for the webhook race).
///
/// `mirror_remote_name` is helper-resolved via P82's STEP 0
/// (`resolve_mirror_remote_name`) and bounded by git's own
/// remote-name validation. Defensive-in-depth (T-83-01): reject
/// `-`-prefixed names BEFORE shell-out — git would interpret a leading
/// `-` as a flag, so an attacker who somehow injected a remote-name
/// like `-foo` could otherwise convert that into a flag injection.
///
/// # Errors
/// Returns `Err` on `Command::output()` spawn failure (e.g. git not
/// on PATH) OR on the defensive `mirror_remote_name.starts_with('-')`
/// reject. A non-zero `git push` exit is `Ok(MirrorResult::Failed { ... })`,
/// NOT a propagated error — that's the partial-fail path the bus
/// caller branches on.
fn push_mirror(mirror_remote_name: &str) -> Result<MirrorResult> {
    if mirror_remote_name.starts_with('-') {
        // Defense-in-depth (T-83-01) flag-injection guard: `mirror_remote_name` is
        // helper-resolved via STEP 0 and already bounded by git's own remote-name
        // validation, so this arm is effectively unreachable in normal flow — it
        // exists to fail-closed if that invariant is ever violated.
        // teach-exempt: ok — unreachable defense-in-depth guard, not a user-facing teaching path
        return Err(anyhow!(
            "mirror_remote_name cannot start with `-`: {mirror_remote_name}"
        ));
    }
    let out = Command::new("git")
        .args(["push", mirror_remote_name, "main"])
        .output()
        .with_context(|| format!("spawn `git push {mirror_remote_name} main`"))?;
    if out.status.success() {
        Ok(MirrorResult::Ok)
    } else {
        // T-83-02: trim stderr to <= 3 lines, joined with " / ". This
        // bounds the operator-readable info-leak (git stderr can include
        // hook output, ref names, commit SHAs).
        let all = String::from_utf8_lossy(&out.stderr);
        let lines: Vec<&str> = all.lines().collect();
        let tail: Vec<&str> = lines.iter().rev().take(3).rev().copied().collect();
        let stderr_tail = tail.join(" / ");
        Ok(MirrorResult::Failed {
            exit_code: out.status.code().unwrap_or(-1),
            stderr_tail,
        })
    }
}

/// Redact ONCE, then fan a mirror partial-fail out to BOTH OP-3 sinks — the
/// append-only `helper_push_partial_fail_mirror_lag` audit row AND the operator
/// stderr diag. Returns the diag line it emitted (so tests can assert on it
/// without capturing stderr).
///
/// WR-01 (P120 SECURITY): `stderr_tail` is RAW `git push` stderr. For a
/// token-in-URL mirror, git emits its auth-failure prose
/// `fatal: Authentication failed for 'https://<user>:<TOKEN>@host/…'` — the
/// token sits in the URL's USERNAME position, which git does NOT redact there.
/// (Modern git *does* strip userinfo from the connection-refused
/// `unable to access '<url>'` line — verified on git 2.50 — but NOT from the
/// auth line, and older gits strip neither.) Both sinks are exfiltration legs
/// under the threat model (CLAUDE.md § Threat model), so the raw tail used to
/// leak the token to EITHER an operator's terminal OR the persisted audit row.
/// Scrub it ONCE here with the same-crate [`crate::backend_dispatch::redact_userinfo`]
/// scanner (strips `scheme://user:secret@` userinfo anywhere in free-form text —
/// the right tool for git's prose, unlike `strip_url_userinfo`, which only
/// parses a single well-formed `http(s)` URL) and feed the REDACTED tail to
/// BOTH sinks — never the raw one.
fn record_mirror_partial_fail(
    cache: Option<&reposix_cache::Cache>,
    oid_hex: &str,
    exit_code: i32,
    stderr_tail: &str,
    chars_in: u64,
    chars_out: u64,
) -> String {
    let redacted_tail = crate::backend_dispatch::redact_userinfo(stderr_tail);
    if let Some(cache) = cache {
        cache.log_helper_push_partial_fail_mirror_lag(oid_hex, exit_code, &redacted_tail);
        cache.log_token_cost(chars_in, chars_out, "push");
    }
    let diag = format!(
        "warning: SoT push succeeded; mirror push failed \
         (will retry on next push or via webhook sync). \
         Reason: exit={exit_code}; tail={redacted_tail}"
    );
    crate::diag(&diag);
    diag
}

/// Bus-handler-local `fail_push` wrapper. The parent crate's
/// `fail_push` (`crates/reposix-remote/src/main.rs`) is `fn`-private;
/// since `bus_handler` is a sibling module, we replicate the body
/// here rather than widening visibility — the body is 5 lines and
/// the duplication is local + intentional.
fn bus_fail_push<R: std::io::Read, W: std::io::Write>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
    kind: &str,
    detail: &str,
) -> Result<()> {
    crate::diag(&format!("error: {detail}"));
    proto.send_line(&format!("error refs/heads/main {kind}"))?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P120 W5 SECURITY REGRESSION (T-120-02). PRECHECK A's `git ls-remote`
    /// failure used to echo the RAW `mirror_url` — including any embedded
    /// `user:token@` userinfo — to the error/stderr, while every sibling reject
    /// path stripped it. This drives the REAL `git ls-remote` subprocess against a
    /// credentialed URL pointed at an unreachable LOOPBACK port (127.0.0.1:1 →
    /// connection refused instantly; hermetic, no real egress, no shared repo) and
    /// asserts (a) the secret NEVER reaches the error, (b) a redacted host
    /// survives so the operator still sees which mirror failed, and (c) the 3-part
    /// teaching is emitted with a creds-free, runnable recovery.
    #[test]
    // test-name-honesty: ok — drives a real `git ls-remote` subprocess (connection-refused) and asserts no-leak + 3-part teaching
    fn precheck_mirror_drift_redacts_credentials_and_teaches_on_ls_remote_failure() {
        let url = "https://x-access-token:SECRETTOKEN123@127.0.0.1:1/mirror.git";
        let err = precheck_mirror_drift(url, "mirror")
            .expect_err("git ls-remote against an unreachable port must fail");
        let msg = format!("{err:#}");
        assert!(
            !msg.contains("SECRETTOKEN123"),
            "CREDENTIAL LEAKED to the error/stderr:\n{msg}"
        );
        assert!(
            !msg.contains("x-access-token"),
            "username leaked to the error/stderr:\n{msg}"
        );
        assert!(
            msg.contains("<redacted>@127.0.0.1:1"),
            "expected the redacted host to survive; got:\n{msg}"
        );
        assert!(
            msg.contains("Fix:") && msg.contains("Recovery:"),
            "must teach the 3-part recovery; got:\n{msg}"
        );
        assert!(
            msg.contains("git ls-remote mirror"),
            "recovery must reference the creds-free remote NAME (runnable), not the \
             redacted URL; got:\n{msg}"
        );
    }

    /// P120 CLOSE WR-01 SECURITY REGRESSION. The `MirrorResult::Failed` arm of
    /// `handle_bus_export` fed RAW `git push` stderr into TWO OP-3 sinks: the
    /// append-only `helper_push_partial_fail_mirror_lag` AUDIT ROW and the
    /// operator diag/stderr echo. git's auth-failure prose for a token-in-URL
    /// mirror — `fatal: Authentication failed for 'https://<user>:<TOKEN>@host/…'`
    /// — puts the token in the URL USERNAME position, which git does NOT redact,
    /// so the token leaked to BOTH sinks. This drives the REAL sink helper
    /// (`record_mirror_partial_fail`, the exact code the arm runs) against a
    /// real `Cache` with such a tail and asserts the secret reaches NEITHER (a)
    /// the PERSISTED audit row NOR (b) the diag string — while the redacted host
    /// survives so the operator still sees which mirror failed. The audit-row
    /// assertion is the load-bearing half: stderr alone would not have caught a
    /// leak into the durable forensic table.
    #[test]
    // test-name-honesty: ok — drives the real MirrorResult::Failed sink helper with a
    // token-bearing git-auth-prose tail and asserts no-leak to BOTH the persisted audit
    // row AND the operator diag (the two OP-3 exfil legs WR-01 closes)
    fn wr01_mirror_partial_fail_scrubs_token_from_both_audit_row_and_diag() {
        use std::sync::Arc;

        use reposix_cache::Cache;
        use reposix_core::backend::sim::SimBackend;
        use reposix_core::BackendConnector;

        // git's auth-failure prose: the token sits in the URL USERNAME position,
        // the exact shape the RAW stderr_tail carried into BOTH sinks pre-fix.
        //
        // NOTE: deliberately NOT `ghp_`-prefixed. `redact_userinfo` strips URL
        // userinfo STRUCTURALLY (the `scheme://user:secret@host` shape), format-
        // agnostic to what the token itself looks like — so a non-prefixed fake
        // token exercises the identical redaction path a real `ghp_`-shaped token
        // would, while not colliding with `quality/gates/structure/cred-hygiene.sh`'s
        // `ghp_[A-Za-z0-9]{20,}` P0 pre-push pattern (a real provider-prefixed
        // secret in this position is exactly what that gate exists to catch, even
        // in a "fake" fixture — see GOOD-TO-HAVES for the inline-allow-marker
        // follow-up so redaction tests CAN use realistic fixtures later).
        const SECRET: &str = "mirror-pushtoken-REDACTME-abc123def456";
        let poison_tail = format!(
            "remote: Invalid username or password / \
             fatal: Authentication failed for \
             'https://x-access-token:{SECRET}@github.com/o/r.git/'"
        );

        // Real cache in an isolated dir. nextest runs each test in its own
        // process, so setting REPOSIX_CACHE_DIR here does not race siblings
        // (same pattern as precheck.rs's shared-cursor regression test).
        let cache_dir = tempfile::tempdir().expect("cache tempdir");
        std::env::set_var("REPOSIX_CACHE_DIR", cache_dir.path());
        let backend: Arc<dyn BackendConnector> =
            Arc::new(SimBackend::new("http://127.0.0.1:0".to_owned()).expect("sim backend"));
        let cache = Cache::open(backend, "sim", "demo").expect("open cache");

        // Drive the REAL arm sink helper.
        let diag =
            record_mirror_partial_fail(Some(&cache), "deadbeefcafe", 128, &poison_tail, 10, 19);

        // (b) operator-diag leg: token + username gone, redacted host survives.
        assert!(
            !diag.contains(SECRET),
            "token LEAKED to the operator diag/stderr:\n{diag}"
        );
        assert!(
            !diag.contains("x-access-token"),
            "username LEAKED to the operator diag/stderr:\n{diag}"
        );
        assert!(
            diag.contains("<redacted>@github.com"),
            "diag must keep the redacted host so the operator sees which mirror failed:\n{diag}"
        );

        // (a) PERSISTED audit-row leg — the load-bearing half of WR-01. Read the
        // row back from cache.db and assert the token never reached disk. Derive
        // the path from the OWNED `cache_dir` (mirrors `resolve_cache_path`'s
        // `<root>/reposix/<backend>-<project>.git` layout) rather than re-reading
        // the global REPOSIX_CACHE_DIR env — under threaded `cargo test` a sibling
        // test can overwrite that env between here and Cache::open. (nextest runs
        // each test in its own process, so the write above is race-free there.)
        let db_path = cache_dir
            .path()
            .join("reposix")
            .join("sim-demo.git")
            .join("cache.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open cache.db");
        let reason: String = conn
            .query_row(
                "SELECT reason FROM audit_events_cache \
                 WHERE op = 'helper_push_partial_fail_mirror_lag'",
                [],
                |r| r.get(0),
            )
            .expect("audit row must exist");
        assert!(
            !reason.contains(SECRET),
            "token LEAKED to the PERSISTED audit row (audit_events_cache.reason):\n{reason}"
        );
        assert!(
            !reason.contains("x-access-token"),
            "username LEAKED to the PERSISTED audit row:\n{reason}"
        );
        assert!(
            reason.contains("<redacted>@github.com"),
            "audit row must keep the redacted host:\n{reason}"
        );
    }
}
