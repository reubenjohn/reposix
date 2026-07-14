# Phase 82 Research — Architecture, Patterns, Standard Stack

← [back to index](./index.md)

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| URL recognition (`?mirror=` form) | reposix-remote (helper binary) | reposix-core (URL primitives) | The helper is the single dispatch site — `BusRemote { sot, mirror }` is helper-internal. Keeping the parse function in `reposix-remote::bus_url` lets us evolve it without forcing a cross-crate API on `reposix-core`. |
| PRECHECK A — `git ls-remote` against mirror | reposix-remote | — | Pure `std::process::Command` invocation; no cache, no backend. Lives next to the bus handler. |
| PRECHECK B — SoT drift via `list_changed_since` | reposix-remote (calls precheck.rs) | reposix-cache (`read_last_fetched_at`), reposix-core (`BackendConnector::list_changed_since`) | P81's `precheck.rs` is the substrate. P82 either reuses verbatim or adds a thin coarser wrapper. |
| Capability advertisement branching | reposix-remote (`main.rs::real_main` `"capabilities"` arm) | — | The URL is known at helper startup (argv[2]); branching at first capabilities exchange is the natural seam. |
| No-mirror-configured error | reposix-remote (`bus_handler`) | git config layer (read-only) | The helper reads `git config --get-regexp '^remote\..*\.url$'` and matches against the mirror URL. |
| Dispatch table for single-backend vs bus | reposix-remote (`main.rs`) | — | One match cascade: parse URL → if `?mirror=` present → bus path; else single-backend path. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `std::process::Command` | std | spawn `git ls-remote <mirror> main` | already used in `crates/reposix-cli/src/doctor.rs:446` for `git --version` and L859/909/935/944 for git porcelain calls — **the project's existing idiom** [VERIFIED: grep across repo] |
| `anyhow::{Context, Result}` | 1.x | error propagation | helper crate already on `anyhow` at the binary boundary [VERIFIED: `crates/reposix-remote/src/main.rs:18`] |
| `reposix-cache` (existing) | path | `read_last_fetched_at`, `read_mirror_synced_at` | P80/P81 already shipped these. Re-used as-is. |
| `reposix-core::backend::BackendConnector` | path | `list_changed_since` | shipped v0.9.0 ARCH-06; verified at `crates/reposix-core/src/backend.rs:253` [VERIFIED] |

### Supporting
| Library | Version | Purpose | When to Use |
|---|---|---|---|
| `gix` | =0.83.0 | reading local git config (`remote.<name>.url`) | when scanning `.git/config` for the mirror remote — but `Command::new("git").args(["config", "--get-regexp", ...])` is a 5-line one-shot and matches the existing porcelain idiom. **Recommend Command-shell-out**, not gix. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|---|---|---|
| `Command::new("git").args(["ls-remote", ...])` | `gix::Repository::find_remote(...).connect(direction)` + `default_handshake_v2` | gix-native is more dependency-clean (no `git` binary required) but the bus handler ALREADY runs in a context where `git` is on PATH (the helper is invoked BY `git push`). Shell-out is shorter, easier to test (mock by setting `PATH` to a temp dir with a fake `git`), and matches the project's `doctor.rs` idiom. |
| New `bus_url.rs` module | extend `backend_dispatch::parse_remote_url` to detect `?mirror=` | Branching on `?mirror=` inside the existing parser would force every single-backend URL through a new `Option<String> mirror` field on `ParsedRemote`. Cleaner to keep `ParsedRemote` for single-backend and introduce `BusRemote { sot: ParsedRemote, mirror_url: String }` in a new module. |
| Coarser PRECHECK B wrapper (`precheck_sot_drift_any`) | call existing `precheck_export_against_changed_set` with a synthetic empty `ParsedExport` | A synthetic empty parse defeats the function's docstring (it expects a real export stream). A 10-line coarser sibling is more honest — its name says "I don't need the push set." |

**No new dependencies required.** Phase 82 is pure helper-crate work using crates already in the workspace [VERIFIED: `cargo metadata` need not run; `crates/reposix-remote/Cargo.toml` carries everything].

**Version verification:** No new versions to verify — every dependency is already pinned in the workspace `Cargo.toml`.

## Architecture Patterns

### System Architecture Diagram

```
git push (user types: git push reposix main)
         │
         ▼
git-remote-reposix <alias> <url>     ── argv[2] = "reposix::sim::demo?mirror=git@github.com:org/r.git"
         │
         ▼
parse_url(argv[2])  ──────────────►  bus_url::parse(...)
         │                              │
         │                              ├─ Success: BusRemote { sot, mirror_url }
         │                              ├─ Reject `+`-delimited: error "use ?mirror= instead"
         │                              └─ Reject unknown query keys (Q-C resolution)
         │
         ▼
dispatch:
  if mirror_url is None  ──────►  single-backend path (existing handle_export)
  else                    ──────►  bus path (NEW)
         │
         ▼ (bus path)
capabilities advertise:
   import / export / refspec / object-format=sha1
   (NO stateless-connect — Q3.4)            ── DVCS-BUS-FETCH-01
         │
         ▼
on `export` command (BEFORE reading stdin):
   ┌────────────────────────────────────────────────────────┐
   │ STEP 0  resolve local git remote name for mirror_url   │
   │   git config --get-regexp '^remote\..+\.url$'          │
   │   match value to mirror_url                            │
   │   none found  →  fail "configure the mirror remote..." │
   └────────────────────────────────────────────────────────┘
         │
         ▼
   ┌────────────────────────────────────────────────────────┐
   │ PRECHECK A  mirror drift                               │
   │   Command::new("git").args(["ls-remote", mirror_url,   │
   │                              "refs/heads/main"])       │
   │   compare returned SHA to local .git/refs/remotes/     │
   │       <name>/main                                       │
   │   drifted → emit "error refs/heads/main fetch first"   │
   │             + hint "your GH mirror has new commits..." │
   │             bail. NO stdin read.                       │
   └────────────────────────────────────────────────────────┘
         │ no drift
         ▼
   ┌────────────────────────────────────────────────────────┐
   │ PRECHECK B  SoT drift (coarse, push-set-blind)         │
   │   precheck_sot_drift_any(cache, backend, project, rt)  │
   │     reads cache.read_last_fetched_at()                 │
   │     calls backend.list_changed_since(since)            │
   │     non-empty? → drift                                 │
   │   drift → emit "error refs/heads/main fetch first"     │
   │           + hint citing refs/mirrors/<sot>-synced-at   │
   │           bail. NO stdin read.                         │
   └────────────────────────────────────────────────────────┘
         │ no drift
         ▼
   ┌────────────────────────────────────────────────────────┐
   │ STEPS 4–9 — read stdin, write SoT, push mirror, audit  │
   │ (DEFERRED to P83)                                       │
   │ For P82: this is the only branch; treat as not-yet-    │
   │ implemented and emit a clean "error" with hint that    │
   │ bus write is P83. Tests assert the prechecks fire and  │
   │ the helper bails BEFORE stdin is read.                 │
   └────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/reposix-remote/
├── src/
│   ├── main.rs              # dispatch loop; capability arm branches on bus-vs-single
│   ├── bus_url.rs           # NEW: parse/format reposix::<sot>?mirror=<url>; reject + form
│   ├── bus_handler.rs       # NEW: STEP 0 + PRECHECK A + PRECHECK B; deferred-write stub
│   ├── precheck.rs          # EXTENDED: add precheck_sot_drift_any wrapper
│   ├── backend_dispatch.rs  # unchanged; ParsedRemote stays single-backend-shaped
│   └── ...
└── tests/
    ├── bus_url.rs           # NEW: parser round-trip, + form rejection, unknown-key rejection
    ├── bus_precheck_a.rs    # NEW: drifting mirror via tempdir bare repo fixture
    ├── bus_precheck_b.rs    # NEW: SoT drift via wiremock list_changed_since (M-style)
    └── bus_capabilities.rs  # NEW: assert stateless-connect omitted for bus URL
```

### Pattern 1: Bus URL parser (the full match cascade)

**What:** Extend `real_main()`'s URL parsing to detect bus URLs at startup and route to a new bus handler.

**When to use:** This is the single dispatch decision point for the entire helper.

```rust
// Source: crates/reposix-remote/src/main.rs (proposed extension; current
// dispatch at lines 108-114 calls parse_dispatch_url which returns
// ParsedRemote; we layer bus detection ABOVE it.)
let url = &argv[2];

let route = match bus_url::parse(url) {
    Ok(bus_url::Route::Bus { sot, mirror_url }) => Route::Bus { sot, mirror_url },
    Ok(bus_url::Route::Single(parsed)) => Route::Single(parsed),
    Err(e) => anyhow::bail!("parse remote url: {e}"),
};

// bus_url::parse returns Route::Bus when ?mirror= is present, Route::Single
// otherwise. The +-delimited form is rejected with a specific error inside
// bus_url::parse, never reaching this match.
```

```rust
// Source: crates/reposix-remote/src/bus_url.rs (NEW)
pub(crate) enum Route {
    Single(ParsedRemote),  // existing single-backend behaviour
    Bus { sot: ParsedRemote, mirror_url: String },
}

pub(crate) fn parse(url: &str) -> Result<Route> {
    let stripped = strip_reposix_prefix(url);

    // Reject +-delimited bus form with specific message (Q3.3).
    if stripped.contains('+') && !stripped.contains('?') {
        // Heuristic: + appears before query-string boundary.
        // Real check: split on '+' between origin and project parts.
        return Err(anyhow!(
            "the `+`-delimited bus URL form is dropped — \
             use `reposix::<sot-spec>?mirror=<mirror-url>` instead"
        ));
    }

    // Split off query string. If absent → Single.
    let (base, query) = match stripped.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (stripped, None),
    };

    let parsed = backend_dispatch::parse_remote_url(base)?;

    let Some(query) = query else {
        return Ok(Route::Single(parsed));
    };

    let params = parse_query(query)?;
    let Some(mirror_url) = params.get("mirror") else {
        // Query string present but no mirror= — that's an error.
        return Err(anyhow!(
            "bus URL query string present but `mirror=` parameter missing; \
             expected `reposix::<sot-spec>?mirror=<mirror-url>`"
        ));
    };
    // Q-C resolution: reject unknown keys (see Open Questions).
    for (k, _) in &params {
        if k != "mirror" {
            return Err(anyhow!(
                "unknown query parameter `{k}` in bus URL; only `mirror=` is supported"
            ));
        }
    }
    Ok(Route::Bus { sot: parsed, mirror_url: mirror_url.clone() })
}
```

[CITED: `crates/reposix-core/src/remote.rs::strip_reposix_prefix` already exists — re-use.]

### Pattern 2: Coarser PRECHECK B wrapper

```rust
// Source: crates/reposix-remote/src/precheck.rs (NEW addition; existing
// precheck_export_against_changed_set untouched)
pub(crate) enum SotDriftOutcome {
    Drifted { changed_count: usize },
    Stable,
}

pub(crate) fn precheck_sot_drift_any(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
) -> Result<SotDriftOutcome> {
    let Some(since) = cache.and_then(|c| c.read_last_fetched_at().ok().flatten()) else {
        // No cursor — first push. Treat as Stable; the inner correctness
        // check at write time (P83) is the safety net. This matches the
        // first-push policy in precheck_export_against_changed_set.
        return Ok(SotDriftOutcome::Stable);
    };
    let changed = rt
        .block_on(backend.list_changed_since(project, since))
        .context("backend-unreachable: list_changed_since (PRECHECK B)")?;
    if changed.is_empty() {
        Ok(SotDriftOutcome::Stable)
    } else {
        Ok(SotDriftOutcome::Drifted { changed_count: changed.len() })
    }
}
```

This is **strictly coarser** than P81's `precheck_export_against_changed_set` because it doesn't intersect with the push set. That's intentional: in the bus path, stdin is unread when we run this, so we don't yet know what the push set is. **The architecture-sketch's step 3 prose is correct** — bail on any change. P83's `handle_export` body (after stdin is read) will run the **finer** intersect-with-push-set check via `precheck_export_against_changed_set` for write-time conflict detection. Both checks are useful: the coarse one fails fast on the cheap path; the fine one catches the rare race where the SoT changes between the precheck and the SoT write.

### Pattern 3: PRECHECK A via `git ls-remote`

```rust
// Source: crates/reposix-remote/src/bus_handler.rs (NEW)
fn precheck_mirror_drift(mirror_url: &str, repo_path: &Path, mirror_remote_name: &str) -> Result<MirrorDriftOutcome> {
    let out = std::process::Command::new("git")
        .args(["ls-remote", mirror_url, "refs/heads/main"])
        .output()
        .context("spawn git ls-remote")?;
    if !out.status.success() {
        return Err(anyhow!(
            "git ls-remote {mirror_url} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let remote_sha = stdout.split_whitespace().next().unwrap_or("").to_owned();
    if remote_sha.is_empty() {
        // Empty mirror — no drift possible; treat as Stable. P84 handles
        // first-push to empty mirror via webhook sync.
        return Ok(MirrorDriftOutcome::Stable);
    }

    // Read local refs/remotes/<name>/main from the working tree's .git/
    let local_sha = read_local_remote_ref(repo_path, mirror_remote_name, "main")?;

    if local_sha != remote_sha {
        Ok(MirrorDriftOutcome::Drifted { local: local_sha, remote: remote_sha })
    } else {
        Ok(MirrorDriftOutcome::Stable)
    }
}
```

The local ref read (`refs/remotes/<name>/main`) can be done with another shell-out: `git rev-parse refs/remotes/<name>/main`. Stay consistent with the shell-out idiom already established by `doctor.rs`.

### Anti-Patterns to Avoid
- **Mutating user's git config in the helper.** Q3.5 ratified: NO `git remote add` autorun. Print the verbatim hint and bail.
- **Reading stdin before prechecks.** Both PRECHECK A and PRECHECK B run BEFORE any `proto.read_line()` call inside the export arm. The cost-savings claim collapses if stdin reading happens first.
- **Calling `precheck_export_against_changed_set` for the bus path's PRECHECK B.** The function expects a `&ParsedExport`. Faking one defeats its docstring. Add the coarser `precheck_sot_drift_any` wrapper.
- **Caching the `git ls-remote` result across helper invocations.** Q3.2 RATIFIED: cache layer deferred. Each push runs the full ls-remote — measure first.
- **Using gix for ls-remote.** `gix::Repository::find_remote(...).connect(...)` introduces ~50 lines of refspec/connection-state management for a one-line `git ls-remote` shell-out. Project idiom is shell-out (`doctor.rs`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| URL query-string parsing | regex on `&str` for `?mirror=...` | the `url` crate (already in workspace via `reqwest`) — `url::Url::parse(...).query_pairs()` | `url` handles percent-decoding (`?mirror=git%40github.com:org%2Frepo.git`), edge cases like trailing `&`, and trailing `=` correctly. Hand-rolled splits will be subtly wrong. [VERIFIED: `url` is a transitive dep via `reqwest` 0.12; check Cargo.lock] |
| Reading remote SHAs from working tree's `.git/refs/remotes/` | open the file with `fs::read_to_string` and trim | `git rev-parse refs/remotes/<name>/main` (shell-out) OR `gix::Repository::find_reference(...)` | packed-refs handling is non-trivial; gix and porcelain both handle it; raw fs reads will miss packed refs and break in production. |
| Comparing two SHAs for drift | byte-equal compare | byte-equal compare | This one IS hand-rolled; SHAs are stable strings. The wrong move would be hashing or trimming. [ASSUMED] |
| Git config scanning for remote URL match | regex over `.git/config` | `git config --get-regexp '^remote\..+\.url$'` shell-out | Same idiom; `.git/config` format has line-continuation, comment, and quoted-value rules. |

**Key insight:** every "small string operation on a git artifact" hides packed-refs, line-continuations, or percent-encoding edge cases. Stay on the `Command::new("git")` rails or use `gix`; do not regex `.git/` files yourself.

## Runtime State Inventory

> Phase 82 is greenfield helper-crate work — no rename/refactor/migration. Section omitted per template guidance.
