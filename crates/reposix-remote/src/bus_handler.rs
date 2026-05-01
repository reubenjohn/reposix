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
//! Steps 4-9 — the WRITE fan-out — are DEFERRED to P83. P82 emits a
//! clean "P83 not yet shipped" error per Q-B / D-02 after prechecks
//! pass. The user sees a clear diagnostic; tests assert prechecks
//! fired. P83 replaces this stub with the `SoT`-write + mirror-write
//! + audit + ref-update logic.
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
/// steps 1-3; the deferred-shipped error closes step 4 (Q-B / D-02).
///
/// # Errors
/// All errors are [`anyhow::Error`]. Reject paths reuse the existing
/// `crate::fail_push` shape via the bus handler's local
/// `bus_fail_push` helper.
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
        return bus_fail_push(
            proto,
            state,
            "bad-mirror-url",
            &format!("mirror URL cannot start with `-`: {mirror_url}"),
        );
    }

    // STEP 0 — resolve local mirror remote name (Q-A / D-01).
    let Some(mirror_remote_name) = resolve_mirror_remote_name(&mirror_url)? else {
        // Q3.5 RATIFIED: emit the verbatim hint, do NOT auto-mutate
        // the user's git config. NO PRECHECK A run.
        return bus_fail_push(
            proto,
            state,
            "no-mirror-remote",
            &format!("configure the mirror remote first: `git remote add <name> {mirror_url}`"),
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
            crate::diag(&format!(
                "your GH mirror has new commits: \
                 local refs/remotes/{mirror_remote_name}/main = {local}; \
                 remote {mirror_url} HEAD = {remote}"
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
    let _ = crate::ensure_cache(state);
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

    // STEPS 4-9 — write fan-out (DEFERRED to P83 per Q-B / D-02).
    emit_deferred_shipped_error(proto, state)
}

/// STEP 0 helper. Returns the local remote name whose `.url` value
/// byte-equals `mirror_url` (with trailing-slash normalization), or
/// `None` if zero matches. Picks first alphabetical + emits stderr
/// WARNING if multiple matches (Pitfall 4 / D-01).
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
        return Err(anyhow!(
            "`git config --get-regexp` exited {exit}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mirror_norm = mirror_url.trim_end_matches('/');
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
        let value_norm = value.trim_end_matches('/');
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
            crate::diag(&format!(
                "warning: multiple local remotes point at {mirror_url}: {matched:?}; \
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
        return Err(anyhow!(
            "git ls-remote {mirror_url} failed: {}",
            String::from_utf8_lossy(&out.stderr)
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

/// Q-B / D-02: emit the deferred-shipped error after prechecks pass.
/// P82 is dispatch-only; P83 replaces this with the `SoT`-first-write
/// plus mirror-best-effort algorithm. The protocol-level error is
/// `bus-write-not-yet-shipped` so a downstream test-harness can
/// distinguish "prechecks fired AND succeeded; write deferred" from
/// "prechecks rejected".
fn emit_deferred_shipped_error<R: std::io::Read, W: std::io::Write>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
) -> Result<()> {
    crate::diag("bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83");
    proto.send_line("error refs/heads/main bus-write-not-yet-shipped")?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    Ok(())
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
