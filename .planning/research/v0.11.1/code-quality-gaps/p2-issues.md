← [back to index](./index.md)

# P2 issues (polish)

### P2-1. FUSE-era doc-comment residue still present

The CATALOG-v2 #10 sweep ("Sweep ~12 FUSE-era doc comments") was missed. Surviving sites:

| File:line | Comment |
|---|---|
| `crates/reposix-core/src/lib.rs:3` | `"This crate is the seam between the simulator … the FUSE daemon, the git remote helper …"` |
| `crates/reposix-core/src/backend.rs:7,9,22,72,106` | "FUSE daemon historically", "FUSE layer", "fuse, remote", "Used by FUSE", "will be used by the FUSE daemon" |
| `crates/reposix-core/src/backend/sim.rs:499,629-632,909` | comments referencing `crates/reposix-fuse/...` (deleted crate) |
| `crates/reposix-core/src/path.rs:3,13` | "FUSE boundary (Phase 3)", "FUSE `tree/` overlay" |
| `crates/reposix-core/src/project.rs:8` | "render it as a directory name in the FUSE mount" |
| `crates/reposix-confluence/src/lib.rs:433,1484` | "FUSE `read()` callback", "used by FUSE for the `tree/` overlay" |
| `crates/reposix-remote/src/fast_import.rs:5` | "the SAME function the FUSE read path uses" |
| `crates/reposix-swarm/src/metrics.rs:238-240` | `Mode name (sim-direct or fuse). … FUSE mount path)` (misleading — `Mode::Fuse` was deleted) |

All cosmetic but rustdoc renders this verbatim — it's the first thing a new reader sees.

### P2-2. Stale phase markers + version markers (v0.1.5 / v0.2 / "Plan 02 / Phase 31")

29 references to specific phase numbers and pre-1.0 versions in source comments (`grep -rn "Phase 13\|Phase 31\|POLISH-\|v0\.[12345]\b" crates/*/src/`). Examples:
- `crates/reposix-core/src/backend.rs:2` — "v0.1.5 ships [`sim::SimBackend`] only; v0.2 will add a `GithubReadOnlyBackend` in `crates/reposix-github`." — both shipped long ago.
- `crates/reposix-cache/src/lib.rs:17` — "Audit log, tainted-byte discipline, and egress allowlist enforcement land in Plan 02 and Plan 03 of Phase 31." — they landed.
- `crates/reposix-cli/src/refresh.rs:155` — `// TODO(Phase-21): populate commit_sha …` — Phase 21 shipped without populating it; either populate or delete the TODO.

Any phase marker that's been completed should either be deleted or downgraded to "(see git log; …)" framing.

### P2-3. Stringly-typed egress detection in `Cache::classify_backend_error`

`crates/reposix-cache/src/builder.rs:510-524`:

```rust
let is_egress = matches!(e, reposix_core::Error::InvalidOrigin(_))
    || emsg.contains("blocked origin")
    || emsg.contains("invalid origin")
    || emsg.contains("allowlist");
```

Comment admits *"both typed and stringly to handle backend adapters that wrap the core error in `Error::Other(String)`"*. Rooted in P1-1 (Error::Other overuse). Once the typed variants land, the substring branches go.

### P2-4. `reposix-sim/src/main.rs` lacks `#![forbid(unsafe_code)]`

`crates/reposix-sim/src/main.rs` is the only binary-entry point in the workspace without the attribute. Every other `lib.rs` and `main.rs` has it. One-line fix at line 1.

### P2-5. `reposix-cli/src/main.rs` `#[allow(clippy::too_many_lines)]` on `main` is fine — but the file is at 420 LOC and growing

The function is fine because it's a clap-derive dispatch. But every new subcommand adds another arm. Consider extracting each arm body into a `dispatch_<verb>(args)` function in `main.rs` so `main()` becomes a strict 80-line match-and-await. Pure refactor; no behaviour change.

### P2-6. `Cache::log_token_cost` and friends only WARN on SQL failure, but the WARN goes to a target other tests don't filter on

`crates/reposix-cache/src/audit.rs:33-36` (and 14 sister sites) emits `tracing::warn!(target: "reposix_cache::audit_failure", …)`. Integration tests (e.g. `crates/reposix-cache/tests/audit_is_append_only.rs`) check the *table* state, not the WARN events. A test that wants to assert "audit was best-effort skipped on a poisoned cache" has no hook today. Add a `cfg(test)` global counter or expose a `Metrics::audit_failures()` getter.

### P2-7. `reposix-cli::tokens::with_commas` and `reposix-cli::cost::with_commas` are byte-identical 16-line functions

`crates/reposix-cli/src/tokens.rs:267-283` and `crates/reposix-cli/src/cost.rs:221-237`. Pull into `worktree_helpers.rs` (or a new `reposix-cli::format` module). Same pattern as the four `cache_path_from_worktree` triplets.

### P2-8. `reposix-confluence::ConfPage` etc. derive `Debug` on response shapes containing potentially sensitive content

`crates/reposix-confluence/src/lib.rs:228 (ConfPage), 268 (ConfPageBody), 281 (ConfBodyStorage)` — `#[derive(Debug, Deserialize)]`. If anyone ever logs a `ConfPage` at debug level, the body content (which may contain auth tokens pasted by the user, secrets, etc.) lands in logs. The `ConfluenceCreds` struct has the manual-Debug redaction discipline (line 123); apply it consistently to wire types that carry user content.

### P2-9. `pub fn parse_next_cursor` and `pub fn basic_auth_header` are crate-public despite no external consumer

`crates/reposix-confluence/src/lib.rs:542,558` — both `pub fn`, both used only inside the file. `pub(crate)` if you need them from `tests/`, or move into `impl ConfluenceBackend` as private methods. Same for `crates/reposix-jira/src/lib.rs:957,971`.
