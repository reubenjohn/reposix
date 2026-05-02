← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T04 — `bus_handler.rs` module + `main.rs` Route dispatch + capabilities branching + State extension

<read_first>
- `crates/reposix-remote/src/main.rs` (entire file ~700 lines —
  understand the existing dispatch loop, `State` field shape,
  `capabilities`/`list`/`export` arms, and `fail_push`).
  Specifically:
  - lines 24-32 (mod declarations)
  - lines 48-77 (`State` struct)
  - lines 103-140 (`real_main` URL parsing + State init)
  - lines 142-211 (dispatch loop + capabilities/list/export arms)
  - lines 246-258 (`fail_push` helper)
  - lines 305-549 (`handle_export` — sibling pattern; bus_handler
    reuses the diag/fail_push idiom but does NOT call
    parse_export_stream).
- `crates/reposix-remote/src/bus_url.rs` (post-T02; confirm `Route`
  enum + `parse` signature).
- `crates/reposix-remote/src/precheck.rs` (post-T03; confirm
  `SotDriftOutcome` + `precheck_sot_drift_any` signature).
- `crates/reposix-cli/src/doctor.rs:446-944` (DONOR pattern for
  `Command::new("git")` shell-outs — same idiom for ls-remote +
  config + rev-parse).
- `crates/reposix-cache/src/mirror_refs.rs:227`
  (`Cache::read_mirror_synced_at` — used by PRECHECK B's hint
  composition when populated).
- `crates/reposix-remote/src/protocol.rs` — `Protocol` API for
  `send_line`, `send_blank`, `flush`.
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md`
  § Pattern 1 (capability branching) + Pattern 3 (PRECHECK A
  shell-out) + Pitfall 4 (multi-match) + § Security (T-82-01).
</read_first>

<action>
Five concerns in this task; keep ordering: State extension → main.rs
URL dispatch → main.rs capabilities + export branching →
bus_handler.rs body → cargo check + commit.

### 4a. State extension — `crates/reposix-remote/src/main.rs:48`

Add ONE new field to the existing `State` struct:

```rust
pub(crate) struct State {
    pub(crate) rt: Runtime,
    pub(crate) backend: Arc<dyn BackendConnector>,
    backend_name: String,
    pub(crate) project: String,
    cache_project: String,
    push_failed: bool,
    #[allow(dead_code)]
    last_fetch_want_count: u32,
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
```

Update the `State` initializer in `real_main` (line 127-136):

```rust
    let mut state = State {
        rt,
        backend,
        backend_name,
        project: project_for_backend,
        cache_project: project_for_cache,
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
        mirror_url: None,  // NEW: defaults to None; set to Some(url) below for Route::Bus
    };
```

### 4b. URL dispatch via `bus_url::parse` — `crates/reposix-remote/src/main.rs:103-126`

Replace the existing URL-parse block. Current:

```rust
    let url = &argv[2];
    let parsed = parse_dispatch_url(url).context("parse remote url")?;
    let backend = instantiate(&parsed).context("instantiate backend")?;
    let backend_name = parsed.kind.slug().to_owned();
    let project_for_cache = sanitize_project_for_cache(&parsed.project);
    let project_for_backend = parsed.project;
```

New shape:

```rust
    let url = &argv[2];
    let route = bus_url::parse(url).context("parse remote url")?;
    // For both Route::Single and Route::Bus, the SoT side is consumed
    // by the existing `instantiate` path. The mirror_url (if any) is
    // captured into `mirror_url_opt` and stamped onto `State` after
    // initialization (D-05 — single Option field, not a new BusState
    // type-state).
    let (parsed, mirror_url_opt): (ParsedRemote, Option<String>) = match route {
        bus_url::Route::Single(p) => (p, None),
        bus_url::Route::Bus { sot, mirror_url } => (sot, Some(mirror_url)),
    };
    let backend = instantiate(&parsed).context("instantiate backend")?;
    let backend_name = parsed.kind.slug().to_owned();
    let project_for_cache = sanitize_project_for_cache(&parsed.project);
    let project_for_backend = parsed.project;
```

Update the `State` initializer to consume `mirror_url_opt`:

```rust
    let mut state = State {
        rt,
        backend,
        backend_name,
        project: project_for_backend,
        cache_project: project_for_cache,
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
        mirror_url: mirror_url_opt,  // CHANGED: from `None` to `mirror_url_opt`
    };
```

Add `ParsedRemote` to the existing `use crate::backend_dispatch::...`
import line if it isn't already imported.

### 4c. Capabilities branching — `crates/reposix-remote/src/main.rs:150-172`

Wrap the existing `proto.send_line("stateless-connect")?;` line (S1
/ Pitfall 6 / DVCS-BUS-FETCH-01). Current arm:

```rust
"capabilities" => {
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    proto.send_line("stateless-connect")?;
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

New shape (5-line edit — the wrapping `if`):

```rust
"capabilities" => {
    // DVCS-BUS-FETCH-01 / Q3.4 — bus URL is PUSH-only; fetch falls
    // through to the single-backend code path. We omit
    // `stateless-connect` for bus URLs; single-backend URLs continue
    // to advertise it. Per `transport-helper.c::process_connect_service`,
    // `stateless-connect` is dispatched only for git-upload-pack /
    // git-upload-archive — push (`git-receive-pack`) falls through
    // to `export` regardless.
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    if state.mirror_url.is_none() {
        proto.send_line("stateless-connect")?;
    }
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

### 4d. Export-verb branching — `crates/reposix-remote/src/main.rs:186-188`

Replace the existing single-line export arm:

```rust
"export" => {
    handle_export(&mut state, &mut proto)?;
}
```

with the two-branch dispatch:

```rust
"export" => {
    if state.mirror_url.is_some() {
        bus_handler::handle_bus_export(&mut state, &mut proto)?;
    } else {
        handle_export(&mut state, &mut proto)?;
    }
}
```

Add `mod bus_handler;` to the top-of-file mod declarations
(alphabetical placement — between `mod bus_url;` (added in T02) and
`mod diff;`):

```rust
mod backend_dispatch;
mod bus_handler;      // NEW
mod bus_url;
mod diff;
mod fast_import;
mod pktline;
mod precheck;
mod protocol;
mod stateless_connect;
```

### 4e. New `bus_handler.rs` module — `crates/reposix-remote/src/bus_handler.rs`

Author the new file. Estimated 180-220 lines including module-doc,
the helpers, and the `handle_bus_export` body.

```rust
//! Bus remote handler — dispatch surface for
//! `reposix::<sot>?mirror=<mirror>` URLs (DVCS-BUS-URL-01,
//! DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01).
//!
//! ## Algorithm (architecture-sketch.md § 3 steps 1-3)
//!
//! On the `export` verb (BEFORE reading stdin):
//!
//! 1. **STEP 0 — resolve local mirror remote name.** Q-A / D-01: scan
//!    `git config --get-regexp '^remote\..+\.url$'`, byte-equal-match
//!    values to `mirror_url` (with trailing-slash normalization),
//!    pick first alphabetically + WARN if multiple. Zero matches →
//!    emit Q3.5 hint and exit BEFORE PRECHECK A.
//! 2. **PRECHECK A — mirror drift.** `git ls-remote -- <mirror_url>
//!    refs/heads/main` shell-out (D-06). Compare returned SHA to
//!    local `git rev-parse refs/remotes/<name>/main`. Drifted →
//!    emit `error refs/heads/main fetch first` + hint, bail. NO
//!    confluence work. NO stdin read.
//! 3. **PRECHECK B — SoT drift.** `precheck::precheck_sot_drift_any(...)`
//!    (T03 substrate). Drifted → emit `error refs/heads/main fetch first`
//!    + hint citing `refs/mirrors/<sot>-synced-at` (when populated by
//!    P80), bail. NO stdin read.
//! 4–9. **WRITE fan-out (DEFERRED to P83).** P82 emits a clean
//!    "P83 not yet shipped" error per Q-B / D-02 after prechecks
//!    pass. The user sees a clear diagnostic; tests assert prechecks
//!    fired. P83 replaces this stub with the SoT-write + mirror-write
//!    + audit + ref-update logic.
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
/// steps 1–3; the deferred-shipped error closes step 4 (Q-B / D-02).
///
/// # Errors
/// All errors are `anyhow::Error`. Reject paths reuse the existing
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
    let mirror_remote_name = match resolve_mirror_remote_name(&mirror_url)? {
        Some(name) => name,
        None => {
            // Q3.5 RATIFIED: emit the verbatim hint, do NOT auto-mutate
            // the user's git config. NO PRECHECK A run.
            return bus_fail_push(
                proto,
                state,
                "no-mirror-remote",
                &format!(
                    "configure the mirror remote first: `git remote add <name> {mirror_url}`"
                ),
            );
        }
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
            return bus_fail_push(proto, state, "fetch first", "mirror drift detected (PRECHECK A)");
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
        crate::diag(&format!(
            "{sot} has {changed_count} change(s) since your last fetch (PRECHECK B)",
            sot = state.backend_name,
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
                    sot = state.backend_name,
                    ts = synced_at.to_rfc3339(),
                ));
            }
        }
        crate::diag(
            "hint: run `git pull --rebase` to incorporate backend changes, then retry",
        );
        return bus_fail_push(proto, state, "fetch first", "SoT drift detected (PRECHECK B)");
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
        let Some((key, value)) = line.splitn(2, char::is_whitespace).fold(
            (None::<&str>, None::<&str>),
            |acc, part| match acc {
                (None, _) => (Some(part), None),
                (Some(k), _) => (Some(k), Some(part)),
            },
        ) else {
            continue;
        };
        let Some(key) = key else { continue };
        let Some(value) = value else { continue };
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
fn precheck_mirror_drift(
    mirror_url: &str,
    mirror_remote_name: &str,
) -> Result<MirrorDriftOutcome> {
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
    let remote_sha = stdout
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_owned();
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

    if local_sha != remote_sha {
        Ok(MirrorDriftOutcome::Drifted {
            local: local_sha,
            remote: remote_sha,
        })
    } else {
        Ok(MirrorDriftOutcome::Stable)
    }
}

/// Q-B / D-02: emit the deferred-shipped error after prechecks pass.
/// P82 is dispatch-only; P83 replaces this with the SoT-first-write
/// + mirror-best-effort algorithm. The protocol-level error is
/// `bus-write-not-yet-shipped` so a downstream test-harness can
/// distinguish "prechecks fired AND succeeded; write deferred" from
/// "prechecks rejected".
fn emit_deferred_shipped_error<R: std::io::Read, W: std::io::Write>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
) -> Result<()> {
    crate::diag(
        "bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83",
    );
    proto.send_line("error refs/heads/main bus-write-not-yet-shipped")?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    Ok(())
}

/// Bus-handler-local fail_push wrapper. Reuses the parent crate's
/// `fail_push` (which is `fn`-private to main.rs). Since `bus_handler`
/// is a sibling module of main.rs, we replicate the body here rather
/// than widening `fail_push` to `pub(crate)` — the body is 5 lines
/// and the duplication is local + intentional.
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
```

**HARD-BLOCK on `crate::fail_push` visibility.** During T04
read_first, confirm `fail_push` (`crates/reposix-remote/src/main.rs:246`)
is private. The bus_handler.rs above defines a local `bus_fail_push`
to avoid widening `fail_push` to `pub(crate)`. If during T04 the
executor finds the duplication awkward, the alternative is to widen
`fail_push` to `pub(crate)` — same shape as P81's H3 widening of
`State`/`issue_id_from_path`. EITHER approach is acceptable; the
plan body's `<must_haves>` does NOT pin which one. Pick the one
that minimizes diff size.

**HARD-BLOCK on `crate::diag` visibility.** `diag` (line 80) is
already `fn`-private. The bus_handler above calls `crate::diag(...)`
which won't compile if `diag` stays private. Widen `diag` to
`pub(crate) fn diag(msg: &str)` as part of T04 (a single-line
visibility edit). This is analogous to P81's `State` widening.

**HARD-BLOCK on `crate::ensure_cache` visibility.** `ensure_cache`
(line 219) is `fn`-private. Widen to `pub(crate)` likewise. Without
this, the bus_handler cannot lazy-open the cache. The widening is
purely additive — no behavior change for `handle_export` which
already calls `ensure_cache`.

Build serially:

```bash
cargo check -p reposix-remote
cargo clippy -p reposix-remote -- -D warnings
cargo nextest run -p reposix-remote      # smoke — no new tests yet (T05 lands them)
```

If `cargo check` fires `unused field` on `state.mirror_url` (because
nothing reads it yet beyond capabilities + export branching — both
of which are now wired in main.rs), the field is in active use and
the warning won't fire. If it does, investigate before committing.

### 4f. Stage and commit

```bash
git add crates/reposix-remote/src/bus_handler.rs \
        crates/reposix-remote/src/main.rs
git commit -m "$(cat <<'EOF'
feat(remote): bus_handler + main.rs Route dispatch + capabilities branching + State extension (DVCS-BUS-PRECHECK-01..02 + DVCS-BUS-FETCH-01)

- crates/reposix-remote/src/bus_handler.rs (new) — pub(crate) fn handle_bus_export orchestrating STEP 0 (resolve mirror remote name by URL match per Q-A / D-01) + PRECHECK A (mirror drift via git ls-remote shell-out per D-06; T-82-01 mitigations: reject `-`-prefixed URLs + `--` separator) + PRECHECK B (precheck::precheck_sot_drift_any) + deferred-shipped error stub (Q-B / D-02 — P82 is dispatch-only; P83 replaces the stub with write fan-out)
- bus_handler::resolve_mirror_remote_name shells out `git config --get-regexp '^remote\..+\.url$'`; first-alphabetical + WARN on multi-match (Pitfall 4)
- bus_handler::precheck_mirror_drift uses `git rev-parse refs/remotes/<name>/main` for local SHA read (handles packed-refs correctly; raw fs reads would miss them)
- bus_handler::emit_deferred_shipped_error emits stderr "bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83" + stdout `error refs/heads/main bus-write-not-yet-shipped`
- crates/reposix-remote/src/main.rs — `mod bus_handler;` declaration (alphabetical); State.mirror_url: Option<String> field (D-05 — single Option, not a new BusState type-state); real_main URL parsing dispatched via `bus_url::parse` returning Route::Single|Bus; capabilities arm gates `stateless-connect` on `state.mirror_url.is_none()` (S1 / DVCS-BUS-FETCH-01); export arm dispatches to `bus_handler::handle_bus_export` when `state.mirror_url.is_some()` else `handle_export`
- main.rs visibility widening (analogous to P81 H3): `fn diag` and `fn ensure_cache` widened to `pub(crate)` so the sibling bus_handler module can call them; `fail_push` stays private — bus_handler defines a local 5-line `bus_fail_push` wrapper to avoid widening
- NO new error variants — anyhow::Result throughout (per main.rs:18 `use anyhow::{Context, Result}`)

Phase 82 / Plan 01 / Task 04 / DVCS-BUS-PRECHECK-01 + DVCS-BUS-PRECHECK-02 + DVCS-BUS-FETCH-01.
EOF
)"
```
</action>

<verify>
  <automated>cargo check -p reposix-remote && cargo clippy -p reposix-remote -- -D warnings && cargo nextest run -p reposix-remote && grep -q "mod bus_handler" crates/reposix-remote/src/main.rs && grep -q "pub(crate) fn handle_bus_export" crates/reposix-remote/src/bus_handler.rs && grep -q 'mirror_url.is_none' crates/reposix-remote/src/main.rs && grep -q 'pub(crate) fn diag' crates/reposix-remote/src/main.rs && grep -q '"--"' crates/reposix-remote/src/bus_handler.rs && grep -q "starts_with('-')" crates/reposix-remote/src/bus_handler.rs</automated>
</verify>

<done>
- `crates/reposix-remote/src/bus_handler.rs` exists with
  `pub(crate) fn handle_bus_export`, `resolve_mirror_remote_name`,
  `precheck_mirror_drift`, `emit_deferred_shipped_error`,
  `bus_fail_push` (or equivalent if the executor chose to widen
  `fail_push` instead).
- `crates/reposix-remote/src/main.rs` declares `mod bus_handler;`.
- `State` has new `mirror_url: Option<String>` field; initialized to
  `None` by default; set to `Some(url)` for `Route::Bus`.
- Capabilities arm at lines 150-172 wraps `proto.send_line("stateless-connect")?;`
  in `if state.mirror_url.is_none() { ... }` (DVCS-BUS-FETCH-01).
- Export arm dispatches to `bus_handler::handle_bus_export` when
  `state.mirror_url.is_some()`.
- `crate::diag` and `crate::ensure_cache` widened to `pub(crate)`.
- `bus_handler::handle_bus_export` rejects `mirror_url` starting with
  `-` BEFORE shell-out (T-82-01).
- `bus_handler::precheck_mirror_drift` passes `--` separator to
  `git ls-remote` (T-82-01).
- Q-B / D-02: `emit_deferred_shipped_error` emits the verbatim
  "bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped"
  stderr + `error refs/heads/main bus-write-not-yet-shipped` stdout.
- `cargo check -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.
- `cargo nextest run -p reposix-remote` passes (existing tests
  unchanged; T05 adds 4 integration tests on top).
- NO new error variants — `anyhow::Result` throughout.
- Cargo serialized: T04 cargo invocations run only after T03's commit
  has landed.
</done>

---

