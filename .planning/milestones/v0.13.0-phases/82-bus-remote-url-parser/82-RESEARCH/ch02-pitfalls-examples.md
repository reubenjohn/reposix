# Phase 82 Research — Common Pitfalls, Code Examples, State of the Art

← [back to index](./index.md)

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
