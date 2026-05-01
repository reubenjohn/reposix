# Phase 82: Bus remote — URL parser, prechecks, fetch dispatch — Research

**Researched:** 2026-05-01
**Domain:** git remote helper protocol; URL routing; cheap drift prechecks
**Confidence:** HIGH

## Summary

Phase 82 stands up the **read/dispatch surface** of the bus remote — URL recognition, two cheap prechecks (mirror drift via `git ls-remote`; SoT drift via `list_changed_since`), and a capability advertisement that excludes `stateless-connect`. The write fan-out (steps 4–9 of the bus algorithm) lands in P83.

The work is concentrated in `crates/reposix-remote/`: a new `bus_url.rs` module for parsing the `?mirror=<url>` query-param form, a new `bus_handler.rs` (or extension to `main.rs`'s dispatch loop) for the two prechecks + fail-fast paths, and a small refactor of the `capabilities` arm to branch on whether the remote is single-backend or bus. Single-backend `reposix::<sot-spec>` URLs continue to dispatch to `handle_export` verbatim — the bus is *additive*. PRECHECK B reuses the **same** `precheck::precheck_export_against_changed_set` already shipped in P81 (M1 narrow-deps signature pays off — cache, backend, project, runtime, parsed export are all the function takes; for P82 we call it with an empty `parsed` shape OR a coarser "any change since cursor?" wrapper). Architecture-sketch step 3 in the bus algorithm is *coarser* than P81's intersect-against-push-set semantics, because in P82 stdin has not yet been read and the push set is unknown — see Section 3 below for the resolution.

**Primary recommendation:** Ship URL parsing as a dedicated `bus_url.rs` module so the parse/format round-trip is unit-testable in isolation. Use `std::process::Command::new("git").args(["ls-remote", ...])` for PRECHECK A — gix's native `connect`-handshake API is overkill for fetching a single ref and adds a non-trivial code surface that isn't where the value is. Add a coarser `precheck_sot_drift_any` wrapper around P81's existing function that asks *"did anything change since `last_fetched_at`?"* without needing the parsed export stream; this is a 10-line wrapper, not a refactor. Reserve a single `BusRemote` struct in the dispatch path that carries `{ sot: ParsedRemote, mirror_url: String }`; the existing `parse_remote_url` continues to handle single-backend URLs unchanged.

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

## Common Pitfalls

### Pitfall 1: Stdin read before prechecks
**What goes wrong:** `handle_export`'s existing structure reads `parse_export_stream` from `proto` BEFORE running any precheck (today's PRECHECK B inside `handle_export` runs after stdin is read). For the bus path, that order MUST flip: PRECHECK A and B run *before* any stdin read, so a failed precheck costs zero stdin bytes (and zero confluence work).
**Why it happens:** the natural place to insert prechecks is "right where the existing one runs", but the existing one runs after `parse_export_stream`.
**How to avoid:** Bus path is a sibling of `handle_export`, not a wrapper. Order: capabilities → list → export verb received → STEP 0 (lookup mirror name) → PRECHECK A → PRECHECK B → THEN read stdin (in P83). The bus path never calls `handle_export` for steps 1–3.
**Warning signs:** test assertion that `parse_export_stream` was called when a precheck should have rejected.

### Pitfall 2: Bus URL parsing colliding with existing `split_reposix_url`
**What goes wrong:** `crates/reposix-core/src/remote.rs::split_reposix_url` finds `/projects/` and trims trailing `/`. With `?mirror=...`, the project segment becomes `demo?mirror=git@github.com:org/repo.git` — the `?` is not stripped, and `parse_remote_url`'s subsequent slug validation rejects it.
**Why it happens:** the URL splitter wasn't designed for query strings.
**How to avoid:** strip the query string in `bus_url::parse` BEFORE calling `backend_dispatch::parse_remote_url(base)`. The base path then has no `?` and validates cleanly.
**Warning signs:** test fails with "invalid project slug: `demo?mirror=...`".

### Pitfall 3: `git ls-remote` against private mirrors hits SSH agent
**What goes wrong:** `mirror_url = git@github.com:org/repo.git` requires SSH agent or key access. CI test environments may not have keys; the `git ls-remote` will block on auth prompt.
**Why it happens:** the helper is invoked in a context where `git push` already authenticated to the SoT, but the mirror auth is independent.
**How to avoid:** for tests, use a local bare-repo fixture (`file:///tmp/.../mirror.git`) — no auth required. Document in CLAUDE.md that production users need their SSH agent set up before bus push. The verifier scripts use the file:// fixture exclusively.
**Warning signs:** integration test hangs waiting for `Are you sure you want to continue connecting (yes/no)?`.

### Pitfall 4: `git config --get-regexp` returns multiple matches for the same URL
**What goes wrong:** a user could have `remote.gh.url = git@github.com:org/repo.git` AND `remote.upstream.url = git@github.com:org/repo.git`. The bus handler must pick one — by what rule?
**Why it happens:** legitimate use case (forks, mirrors-of-mirrors).
**How to avoid:** if multiple remotes match, pick the first one alphabetically and emit a stderr WARNING naming the chosen remote. Document the choice. Q-A in Open Questions surfaces this for the planner.
**Warning signs:** flaky integration test where `<name>` flips between `gh` and `upstream` between runs.

### Pitfall 5: PRECHECK B firing on the same-record self-edit case
**What goes wrong:** a user pushes their own change at T+0; the SoT now has `updated_at = T+0`. They immediately push another edit at T+1. PRECHECK B reads `last_fetched_at` (= T+0 from the first push), runs `list_changed_since(T+0)`, gets back ID 5 (their own edit), and rejects with "fetch first". User is confused — they ARE the change.
**Why it happens:** `last_fetched_at` is updated on push success (per `handle_export` line 495), so it should be > T+0 by the time of the second push. But there's a subtle race: if the SoT's `updated_at` for ID 5 is exactly `T+0`, and `last_fetched_at` is also `T+0`, `list_changed_since` is `>=` or `>`? P81's overrides currently use `>` (verified at `crates/reposix-core/src/backend.rs:261`).
**How to avoid:** confirm `>`-strict semantics (already correct in default impl). Add a regression test for the rapid-double-push case.
**Warning signs:** "fetch first" emitted on the second of two same-second pushes.

### Pitfall 6: Capabilities advertisement not branching at the right layer
**What goes wrong:** the capabilities arm is fired once per helper invocation, and the URL has been parsed by then. If the branching is per-command (inside the match arm) instead of per-helper-invocation, single-backend pushes pay no cost; bus URLs need a one-line `let advertise_stateless = matches!(route, Route::Single(_));`.
**Why it happens:** straightforward — just remember to gate `proto.send_line("stateless-connect")?;` on `advertise_stateless`.
**How to avoid:** branch in the `"capabilities"` arm at `crates/reposix-remote/src/main.rs:150-172`. Single-line `if`.

### Pitfall 7: Mirror URL with `?` in query string colliding with `?mirror=` boundary
**What goes wrong:** `mirror_url = https://gh.com/foo?token=secret` (rare but legal). The parser sees TWO `?` and gets confused.
**Why it happens:** query-string parsing is not robust when the value itself is a URL with its own query.
**How to avoid:** require the mirror URL to be percent-encoded if it contains `?`. The `url` crate's `query_pairs()` handles this; `url::Url::parse(format!("reposix://x{stripped}"))` would round-trip correctly. Document the requirement.
**Warning signs:** test "mirror URL with embedded ?" fails with "unknown query parameter `token`".

## Code Examples

### Example 1: Detecting `+`-delimited rejection

```rust
// crates/reposix-remote/src/bus_url.rs (NEW)
#[test]
fn rejects_plus_delimited_bus_url() {
    let err = parse("reposix::sim::demo+git@github.com:org/repo.git").unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("`+`-delimited"), "expected reject message: {msg}");
    assert!(msg.contains("?mirror="), "should suggest correct form: {msg}");
}
```

### Example 2: PRECHECK A integration test fixture

```rust
// crates/reposix-remote/tests/bus_precheck_a.rs (NEW)
fn make_drifting_mirror(tmp: &Path) -> (PathBuf, String) {
    // Build two bare repos, give one an extra commit so it looks like
    // someone else pushed to the mirror between our last fetch and now.
    let mirror_dir = tmp.join("mirror.git");
    Command::new("git").args(["init", "--bare", mirror_dir.to_str().unwrap()]).status().unwrap();
    // ... seed a commit, then add a divergent one ...
    let url = format!("file://{}", mirror_dir.display());
    let drifted_sha = run_git_in(&mirror_dir, &["rev-parse", "HEAD"]);
    (mirror_dir, drifted_sha)
}

#[test]
fn bus_precheck_a_emits_fetch_first_on_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let (_mirror_dir, _) = make_drifting_mirror(tmp.path());
    // Set up a working tree whose refs/remotes/mirror/main is BEHIND
    // the bare repo's HEAD. Point the bus URL at the bare repo.
    // Run the helper via assert_cmd, send "capabilities\nexport\n", read
    // stderr for "your GH mirror has new commits" and stdout for
    // "error refs/heads/main fetch first".
    // ...
}
```

### Example 3: Capability branching

```rust
// crates/reposix-remote/src/main.rs (proposed; current code at L150-172)
"capabilities" => {
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    if matches!(route, Route::Single(_)) {
        proto.send_line("stateless-connect")?;
    }
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| `reposix::bus://<sot>+<mirror>` (architecture-sketch original) | `reposix::<sot>?mirror=<mirror>` (Q3.3) | 2026-04-30 (decisions.md) | URL-safe in all contexts; reuses existing `split_reposix_url`. |
| Bus handles fetch | Bus PUSH-only; fetch via single-backend (Q3.4) | 2026-04-30 | reposix-cache + stateless-connect are unchanged for reads. |
| Helper-side retry on transient mirror failure | Surface, no retry (Q3.6) | 2026-04-30 | P83 territory; mentioned for context. |

**Deprecated/outdated:**
- `bus://` scheme keyword — dropped per Q3.3.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `Command::new("git")` is on PATH when the helper runs | Standard Stack | LOW — git invokes the helper, so it's PATH-resolvable. Verified by existing `doctor.rs` shell-outs. |
| A2 | `git rev-parse refs/remotes/<name>/main` is the right way to read the local mirror SHA | Pattern 3 | LOW — porcelain handles packed-refs correctly. Could also use `git ls-remote .` against the local working tree but rev-parse is more direct. |
| A3 | The `url` crate's `query_pairs()` handles `mirror=git@github.com:org/repo.git` correctly without percent-encoding | Don't Hand-Roll | MEDIUM — `@` and `:` are reserved in URL contexts. Verify in a parser unit test; if `url` chokes, fall back to manual `split_once('=')` after `split_once('?')`. |
| A4 | `precheck_sot_drift_any` returning `Stable` on no-cursor (first-push) is the right policy | Pattern 2 | LOW — matches first-push behaviour in P81's `precheck_export_against_changed_set`; the inner correctness check at SoT-write time (P83) is the safety net. |
| A5 | Multiple remotes pointing at the same `mirror_url` is a real edge case worth handling | Pitfall 4 | LOW — unusual but legal. First-alphabetical pick + WARN is conservative. |

## Open Questions

### Q-A: by-remote-name vs. by-URL-match for the no-remote-configured check
**What we know:** the bus URL carries `mirror=<url>` only — no remote NAME. The user's local git config has `remote.<name>.url = <url>` keyed by name.
**What's unclear:** to find the local remote, we either (a) scan `git config --get-regexp '^remote\..+\.url$'` and match values to `mirror_url`, or (b) require the bus URL to carry a NAME (`?mirror_name=gh&mirror_url=...`). Q3.5's hint *"`git remote add <name> <mirror-url>`"* doesn't pin which one.
**Recommendation:** **(a) by-URL-match.** The user has already named the remote; making them name it again in the bus URL is friction. URL-match is the canonical UX. Pitfall 4 documents the multi-match resolution (alphabetical + WARN). Planner ratifies.

### Q-B: does the bus handler reuse `handle_export` for write fan-out (P83) or have its own write loop?
**What we know:** P82 is dispatch-only — the question's resolution doesn't block P82 success criteria. P83 ROADMAP suggests *"the existing single-backend `handle_export` whose write logic the bus wraps verbatim per architecture-sketch § Tie-back to existing helper code"*.
**What's unclear:** whether the wrapping is at function-call granularity (`bus_export()` calls `handle_export()` after prechecks) or finer (factor `handle_export`'s body into helpers and call them from a new `bus_handle_export()`).
**Recommendation:** **P82 makes no commitment.** The bus path in P82 emits prechecks then a *temporary* "P83 not yet shipped" error to keep the surface minimal. P83's planner decides reuse strategy. P82's job is to land the URL parser, prechecks, and capability branching — every line of those is independent of P83's choice.

### Q-C: should bus URLs allow query params other than `mirror=`?
**What we know:** today only `mirror=` is meaningful. v0.13.0 RATIFIED `mirror=` as the sole syntax. Future params (`priority=`, `retry=`, `mirror_name=`) are not in scope.
**What's unclear:** silently ignore unknown keys vs. reject.
**Recommendation:** **REJECT unknown keys with a clear error.** Forward-compatibility-via-silent-ignore is a footgun (a typo `?mirorr=` becomes a no-op precheck-against-mirror-less remote). Reject lets us add new keys later without ambiguity. Pattern 1 code shows this.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| `git` binary on PATH | `Command::new("git")` shell-outs | ✓ | v2.34+ (project requirement) | none — git is the runtime contract |
| `cargo` workspace | building helper + tests | ✓ | 1.82+ | none |
| Rust crates already in workspace (`anyhow`, `chrono`, `tempfile`, `assert_cmd`, `wiremock`, `tokio`) | tests | ✓ | as-pinned | none |
| `url` crate | query-string parsing in `bus_url.rs` | ✓ (transitive via `reqwest`) | confirm in Cargo.lock | hand-rolled `split_once` if missing — cheap fallback |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `url` is transitive — if a future hygiene phase trims it, fall back to manual splitting. For now, prefer `url::form_urlencoded::parse(query.as_bytes())` if `url::Url::parse` is finicky on the `reposix::` scheme.

## Validation Architecture

### Test Framework
| Property | Value |
|---|---|
| Framework | `cargo test` (unit + integration); `cargo nextest run` for full-workspace runs (CLAUDE.md memory budget) |
| Config file | none — uses workspace `Cargo.toml` |
| Quick run command | `cargo test -p reposix-remote --test bus_url --test bus_precheck_a --test bus_precheck_b --test bus_capabilities` |
| Full suite command | `cargo nextest run -p reposix-remote` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| DVCS-BUS-URL-01 | URL parses `?mirror=` form; rejects `+` form | unit | `cargo test -p reposix-remote --test bus_url` | ❌ Wave 0 |
| DVCS-BUS-PRECHECK-01 | mirror drift emits `fetch first` + hint | integration | `cargo test -p reposix-remote --test bus_precheck_a` | ❌ Wave 0 |
| DVCS-BUS-PRECHECK-02 | SoT drift emits `fetch first` + cites mirror-lag refs | integration | `cargo test -p reposix-remote --test bus_precheck_b` | ❌ Wave 0 |
| DVCS-BUS-FETCH-01 | bus URL omits `stateless-connect` from capabilities | unit | `cargo test -p reposix-remote --test bus_capabilities` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-remote` (per-crate, per CLAUDE.md memory budget)
- **Per wave merge:** `cargo nextest run -p reposix-remote`
- **Phase gate:** Pre-push hook runs workspace-wide; verifier subagent runs catalog gates.

### Wave 0 Gaps
- [ ] `crates/reposix-remote/src/bus_url.rs` — new module
- [ ] `crates/reposix-remote/src/bus_handler.rs` — new module
- [ ] `crates/reposix-remote/tests/bus_url.rs` — new test file
- [ ] `crates/reposix-remote/tests/bus_precheck_a.rs` — new test file (with file:// bare-repo fixture)
- [ ] `crates/reposix-remote/tests/bus_precheck_b.rs` — new test file (wiremock-backed; mirror precheck_a.rs's wiremock idiom)
- [ ] `crates/reposix-remote/tests/bus_capabilities.rs` — new test file
- [ ] `quality/catalogs/agent-ux.json` — 5 new rows minted BEFORE implementation per QG-06
- [ ] `quality/gates/agent-ux/bus-*.sh` — 5 new verifier scripts (one per row)
- [ ] CLAUDE.md update — § Architecture (mention bus URL form) + § Commands (the new push form)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | yes (transitively) | `git ls-remote` against private mirrors uses SSH agent / git creds. The helper does NOT store or transmit credentials directly — relies on the user's existing git config. |
| V3 Session Management | no | helper is per-invocation. |
| V4 Access Control | yes | `REPOSIX_ALLOWED_ORIGINS` allowlist already gates SoT egress. PRECHECK B's `list_changed_since` call goes through the same allowlisted client. |
| V5 Input Validation | yes | URL parsing must reject malformed forms; the `?mirror=` value is bound for SHELL-OUT (`git ls-remote <mirror_url>`) so injection is the canonical risk. **MUST** treat `mirror_url` as untrusted; reject characters that could be interpreted as flags by `git ls-remote` (leading `--`). |
| V6 Cryptography | no | helper does no crypto directly. |

### Known Threat Patterns for the bus path

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Argument-injection via `mirror_url` (e.g., `--upload-pack=evil`) | Tampering | Reject `mirror_url` values starting with `-`; pass `--` separator before arguments to `git ls-remote`. Idiom: `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"])`. |
| Tainted `mirror_url` from a malicious git config | Tampering | The user controls their own git config; treating it as trusted is in keeping with the project threat model. But the BUS URL is in `argv` from `git push` — same trust origin (user). Document explicitly in `bus_url.rs` module-doc that `mirror_url` is treated as user-controlled, not attacker-controlled. |
| Mirror SHA from `git ls-remote` is attacker-influenced | Tampering | The SHA is only compared (byte-equal) to the local SHA; it's not parsed, executed, or committed. Tainted-bytes flow is bounded — no `Tainted<T>` wrapper needed for SHAs. |
| Allowlist bypass via SoT URL change | Tampering | The SoT URL is parsed and the SoT BackendConnector is built before any precheck runs. Existing allowlist enforcement (`reposix_core::http::client()`) covers PRECHECK B's `list_changed_since` call. |

## Catalog Row Design

Per QG-06 (catalog-first), the FIRST commit of the implementing phase mints these rows in `quality/catalogs/agent-ux.json` (or a new `bus-remote.json` — recommend keeping in `agent-ux.json` to match the existing `agent-ux/dark-factory-sim`, `agent-ux/reposix-attach-against-vanilla-clone`, `agent-ux/mirror-refs-*` neighbours).

**Five proposed rows** (verifier scripts under `quality/gates/agent-ux/`):

1. **`agent-ux/bus-url-parses-query-param-form`**
   - `verifier`: `quality/gates/agent-ux/bus-url-parse.sh` — runs `cargo test -p reposix-remote --test bus_url -- --exact bus_url_parses_query_param_form`
   - `kind`: mechanical
   - `cadence`: pre-pr
   - asserts: parser returns `Route::Bus { sot: <expected>, mirror_url: <expected> }` for `reposix::sim::demo?mirror=git@github.com:org/r.git`
2. **`agent-ux/bus-url-rejects-plus-delimited`**
   - `verifier`: `quality/gates/agent-ux/bus-url-reject-plus.sh`
   - asserts: helper exits non-zero with stderr containing `"+`-delimited"` AND `"?mirror="`
3. **`agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`**
   - `verifier`: `quality/gates/agent-ux/bus-precheck-a-mirror-drift.sh`
   - kind: mechanical (uses file:// fixture, sim, assert_cmd-style invocation)
   - asserts: stdout contains `error refs/heads/main fetch first`; stderr contains `your GH mirror has new commits`; helper exited before any stdin was read (`fast-import` parsing not invoked — assertable via cache audit row count `helper_push_started=1, helper_push_accepted=0`)
4. **`agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`**
   - `verifier`: `quality/gates/agent-ux/bus-precheck-b-sot-drift.sh`
   - kind: mechanical
   - asserts: stdout `error refs/heads/main fetch first`; stderr cites `refs/mirrors/<sot>-synced-at` (when populated) — uses the existing `read_mirror_synced_at` helper from P80
5. **`agent-ux/bus-fetch-not-advertised`**
   - `verifier`: `quality/gates/agent-ux/bus-fetch-not-advertised.sh`
   - kind: mechanical
   - asserts: capability list emitted on stdout for a bus URL contains `import`, `export`, `refspec`, `object-format=sha1` but NOT `stateless-connect` (DVCS-BUS-FETCH-01)

**Sixth row (recommended, not required by phase requirements):**

6. **`agent-ux/bus-no-mirror-remote-configured-error`** — covers Q3.5 / DVCS-BUS-WRITE-05's P82 portion (success criterion 5 of P82 ROADMAP).
   - asserts: bus URL referencing a `mirror_url` not in any local `remote.<name>.url` fails with verbatim hint `"configure the mirror remote first: git remote add <name> <mirror-url>"`. NO auto-mutation of git config.

The roadmap explicitly lists 5 + the no-remote case as success criteria. Rows 1-5 + row 6 = full coverage. Plan to mint 6 rows.

## Test Fixture Strategy (PRECHECK A — drifting mirror)

**Two options:**

**(a) Two local bare repos.** `mktemp -d`; `git init --bare mirror.git`; seed a commit; `git init --bare working-copy.git`; clone mirror; commit + push to mirror with `--force` to make it diverge from working-copy's `refs/remotes/mirror/main`. Bus URL points at `file:///tmp/.../mirror.git`. **No network. No SSH agent. No wiremock.** ~30 lines of bash.

**(b) wiremock-backed mirror via reposix-sim.** sim doesn't speak smart-HTTP; this would require a brand-new mock server speaking the git smart-HTTP protocol. Out of scope; non-trivial.

**Recommend (a).** It's the project's existing idiom (`scripts/dark-factory-test.sh` uses local bare repos for the same reason). The fixture also makes the test fast (<2s) and offline.

For PRECHECK B, **wiremock** is the right answer — `precheck.rs` already uses wiremock in `crates/reposix-remote/tests/perf_l1.rs`. The bus_precheck_b test mirrors that idiom: spawn wiremock, mock `list_changed_since` to return `[id 5]`, set `last_fetched_at` in the cache, run the helper against the bus URL, assert the rejection message + that NO writes hit wiremock (`Mock::expect(0)` on PATCH).

## Plan Splitting

**Recommend: SINGLE plan.** Phase 82 has 7 success criteria but they all sit in `crates/reposix-remote/`. Cargo-heavy task count:

| Task | Cargo work | Notes |
|---|---|---|
| T1: catalog rows + CLAUDE.md update | none | doc/JSON only |
| T2: `bus_url.rs` + unit tests | per-crate `cargo test -p reposix-remote --test bus_url` | small |
| T3: `precheck_sot_drift_any` wrapper + unit test | per-crate `cargo test -p reposix-remote --lib precheck` | small |
| T4: `bus_handler.rs` (STEP 0 + PRECHECK A + PRECHECK B + capability branching) + integration tests | per-crate `cargo test -p reposix-remote --test bus_*` | medium — 4 integration tests bundled |
| T5: verifier scripts (5+1) + catalog refresh | shell only | doc/script work |
| T6: phase close — `git push origin main`; verifier subagent | none | gate |

Six tasks; four are doc/JSON/shell. Two cargo-heavy tasks (T2 + T4) — well under the ≤4 cargo-heavy ceiling per CLAUDE.md memory budget. **Single plan stands.**

The `bus_handler.rs` body (T4) is where complexity sits — but the prechecks are independent (a fail in PRECHECK A short-circuits before B even runs), so the test surface can be parallelized across `bus_precheck_a.rs` and `bus_precheck_b.rs` without sharing state.

## Sources

### Primary (HIGH confidence)
- `crates/reposix-remote/src/main.rs` (full body, 687 lines) — current dispatch loop, capabilities arm, `handle_export`, lazy cache, ProtoReader.
- `crates/reposix-remote/src/precheck.rs` (302 lines) — P81's L1 precheck function; M1 narrow-deps signature.
- `crates/reposix-remote/src/backend_dispatch.rs` (537 lines) — `parse_remote_url`, `BackendKind`, `instantiate`, env-var-keyed credential checks.
- `crates/reposix-cache/src/mirror_refs.rs` (371 lines) — P80's mirror-lag refs, `read_mirror_synced_at`.
- `crates/reposix-core/src/backend.rs:253` — `BackendConnector::list_changed_since` signature.
- `crates/reposix-core/src/remote.rs:43` — `split_reposix_url`.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § 3 — the bus algorithm.
- `.planning/research/v0.13.0-dvcs/decisions.md` § Phase-N+2 — Q3.1–Q3.6.
- `.planning/ROADMAP.md` lines 124–143 — phase scope + 7 success criteria.
- `.planning/REQUIREMENTS.md` lines 71–80 — DVCS-BUS-* IDs.
- `quality/gates/agent-ux/reposix-attach.sh` — verifier idiom for new agent-ux rows.

### Secondary (MEDIUM confidence)
- `crates/reposix-cli/src/doctor.rs:446-944` — existing `Command::new("git")` shell-out idiom.
- `crates/reposix-remote/tests/perf_l1.rs` (lines 1–90) — wiremock idiom for SoT-side tests; `NoSinceQueryParam` / `HasSinceQueryParam` matchers.

### Tertiary (LOW confidence)
- None — every claim above ties to a verified source.

## Metadata

**Confidence breakdown:**
- URL parser design: HIGH — every API call cited to a verified source line.
- PRECHECK A choice (shell-out): HIGH — matches project idiom (doctor.rs).
- PRECHECK B reuse strategy: HIGH — wrapper pattern preserves P81 behaviour; uses existing `last_fetched_at` cursor.
- Capabilities advertisement branching: HIGH — current code at `main.rs:150-172` is a 5-line edit.
- No-remote-configured lookup (URL-match vs name): MEDIUM — Q-A is real, recommendation is conservative.
- Catalog row design: HIGH — mirrors P80 / P81 row shape verbatim.
- Test fixture strategy: HIGH — file:// fixture + wiremock are both pre-existing idioms.

**Research date:** 2026-05-01
**Valid until:** 2026-05-31 (30 days; bus URL spec is RATIFIED — only invalidates if Q3.x decisions reopen)
