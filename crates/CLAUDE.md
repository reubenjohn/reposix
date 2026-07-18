# crates/CLAUDE.md — Rust workspace rules (auto-loaded under crates/)

Extends root `CLAUDE.md`. Long-form build/memory doctrine + tech stack + workspace
layout live here; root keeps a pointer. Full orchestration doctrine:
`.planning/ORCHESTRATION.md`.

## Workspace layout

```
crates/
├── reposix-core/        # Shared types: Record, Project, RemoteSpec, Error, Tainted<T>.
├── reposix-sim/         # In-process axum HTTP simulator (default backend).
├── reposix-cache/       # On-disk bare-repo cache backed by gix; lazy blob materialization.
├── reposix-remote/      # git-remote-reposix binary (stateless-connect + export).
├── reposix-cli/         # Top-level `reposix` CLI (init, attach, sim, list, refresh, spaces, sync).
├── reposix-github/      # GitHub Issues BackendConnector.
├── reposix-confluence/  # Confluence Cloud BackendConnector.
├── reposix-jira/        # JIRA Cloud BackendConnector.
└── reposix-swarm/       # Multi-agent contention/swarm test harness.
```
(Repo-root siblings: `.planning/` GSD state — do not hand-edit; `docs/` user-facing;
`research/` long-form notes; `runtime/` gitignored sim DB + scratch trees.)

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1. Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- Git: `gix` 0.83 (pinned with `=`, gix is pre-1.0). **Git `2.34+` recommended**
  for reliable partial-clone reads / `stateless-connect` (`extensions.partialClone`);
  the simulator flow runs on older git (verified down to 2.25) — `doctor` WARNs, not
  ERRORs, below 2.34. The modern-git `stateless-connect` (protocol-v2) READ path is
  now PROVEN to reconcile after SoT drift (not just the old-git `import` path):
  P122 W4 runs both drift scenarios via protocol-v2 on git 2.50.1 with a
  `GIT_TRACE_PACKET` wire proof (`command=fetch`+`version 2`, zero import stream) in
  `quality/gates/agent-ux/rebase-recovery-reconciles.sh` — P105 §5 resolved, Branch A
  convergence (GTH-V15-04 / DRAIN-07).
- Storage: `rusqlite` 0.32 with `bundled` (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Build memory budget (load-bearing — read before parallelizing)

The VM has OOM-crashed **three times** this project from parallel cargo workspace
builds. The workspace links large crates (`gix` chain, `rusqlite-bundled`,
`reqwest+rustls`); a single `cargo check --workspace` peaks ~4–6 GB; `cargo test
--workspace` is worse (rustc + N test binaries link in parallel).

**Hard rules (both apply, no exception without explicit owner override):**
1. **Never more than one cargo invocation at a time, machine-wide** — check/build/test/
   clippy. Two subagents needing compile run SEQUENTIALLY. At most one phase-executor
   doing cargo work at a time. This is now ALSO enforced by
   `.claude/hooks/cargo-mutex.sh` (PreToolUse/Bash → exit 2 if a live cargo/rustc build
   is detected) — the hook is a backstop, orchestration discipline is the primary control.
2. **Prefer per-crate over workspace-wide** — `cargo check -p reposix-cli`, not
   `--workspace`, when the change is scoped. Pre-push covers the workspace gate.

**Pre-push is a hidden cargo lane — serialize pushes machine-wide (fix-twice, v0.14.0 P111).**
`.githooks/pre-push` runs a FULL workspace quality validate — `cargo fmt --check` +
`cargo clippy` unconditionally, plus gitleaks — on EVERY push, regardless of diff size
(measured **~102s including gitleaks on a docs-only commit**; the cost is a fixed
whole-repo walk, not per-changed-file — see `quality/CLAUDE.md` § "Runtime does NOT scale
with diff size"). So a code-landing lane that pushes while ANOTHER session already holds
the machine-wide cargo token spawns a SECOND concurrent workspace compile — the exact OOM
trap Hard rule 1 exists to prevent, just triggered by `git push` instead of a bare `cargo`.
Treat a push like a cargo invocation: **serialize pushes** across all lanes (at most one
`git push` in flight machine-wide, and never push while another lane is mid-cargo). When a
lane only needs the change LANDED (not locally re-linted), prefer relying on post-push CI
(`code/ci-green-on-main` is the authoritative green) over a redundant local full-workspace
clippy — it keeps the single cargo slot free for the lane that actually needs it.

**Soft rules:** `CARGO_BUILD_JOBS=2` (committed in `.cargo/config.toml`); `cargo nextest
run` links test binaries one at a time (prefer for full-workspace tests); close idle
rust-analyzer editors (2–3 GB); schedule docs/playwright work in a no-cargo window.

**If the VM crashes again:** suspect parallel cargo, rust-analyzer, or leftover
background processes (`ps aux | grep -E "cargo|rustc"`). Blame the orchestration that
let two run at once, not the linker.

## Code conventions

- `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` in every crate; allow-list
  specific lints with rationale, never blanket-allow pedantic.
- All public items documented (missing-doc lint on for `reposix-core`); every
  `Result`-returning fn has a `# Errors` section.
- Tests next to code (`#[cfg(test)] mod tests`); integration tests in `tests/`.
- Frontmatter = `serde_yaml` 0.9 + Markdown body; never JSON-on-disk for issues.
- Times are `chrono::DateTime<Utc>`; no `SystemTime` in serialized form.
- Banned in production `crates/**/*.rs` (outside `tests/`): `\bP\d{2,3}-\d+\b` phase-ID
  tokens (`banned-production-tokens.sh`); use `// banned-words: ok` for justified refs.
  The regex CATCHES v0.13+ phase numbers (`P79-02`, `P150-01`) and INTENTIONALLY MISSES
  v0.8/v0.9-era audit IDs `P\d-\d` (`P1-1` in `error.rs` — code-quality refs, not phase
  IDs). Forward convention: new audit-ID schemes adopt `P\d{2,3}-` numbering or a distinct
  prefix (`AUD-1`). Full rationale in the script header. Separately,
  `deferral-pointer-linter.sh` (pre-push) requires every deferral pointer in `crates/`
  (phrases like "lands in P<N>") to name a real downstream phase with a PLAN artifact
  under `.planning/phases/N-*/`; see the script header for the exact patterns.

## Error-message convention (Rust-compiler-grade UX)

Every user-facing error — every `reposix` CLI subcommand AND the `reposix-remote` git
helper — meets a three-part bar: (1) **teach the fix**, (2) **suggest the alternative**,
(3) **give a copy-paste recovery command**. The pattern to copy is
`reposix-cli/src/init.rs::refuse_existing_repo_root`: it refuses fail-closed, names the
corruption shape, points at `reposix attach` as the alternative, and prints runnable
recovery lines. A bare `bail!("usage: …")` or a terse `.context("parse remote url")` that
surfaces raw to the user does NOT meet the bar — wrap it in a teaching message. Dimension:
**agent-ux / docs-alignment** (route audits to `quality/gates/agent-ux/`). Scheduled as a
first-class v0.15.0 phase — see `.planning/milestones/v0.15.0-phases/ROADMAP.md`.

### Credential redaction before ANY error/stderr echo (threat-model exfil leg)

Every remote byte is attacker-influenced and stderr is an exfiltration leg (root CLAUDE.md
§ Threat model), so **never interpolate a raw URL or raw `git` stderr into an error**.
There are TWO redactors — pick by the SHAPE of the string, not by the site:

- **A single, well-formed `http(s)` URL** (e.g. a bare `?mirror=<url>` value already parsed
  out) → `reposix_core::http::strip_url_userinfo(url).0`. It `Url::parse`s ONE URL and strips
  the authority userinfo; non-`http(s)` inputs pass through unchanged.
- **A `reposix::`-prefixed remote URL, a bus URL with an embedded `?mirror=<url>`, git's
  free-form stderr, or ANY string where a credential may sit past the leading authority** →
  `reposix_remote::backend_dispatch::redact_userinfo(s)`. It SCANS and strips
  `scheme://user:secret@` userinfo ANYWHERE in the text.

**The trap (verified, P120):** `strip_url_userinfo` PASSES a `reposix::`-prefixed URL through
UNCHANGED — the outer `reposix::` scheme defeats `Url::parse` — so echoing a bus URL through it
LEAKS the `?mirror=` credential. Likewise it never touches a token in git's `Authentication
failed for 'https://<TOKEN>@host/…'` prose (git puts the token in the USERNAME position and does
NOT redact it; modern git strips the connection-refused `unable to access` line but not the auth
line). When in doubt for anything not provably a lone `http(s)` URL, use the `redact_userinfo`
scanner. Regression exemplars: `backend_dispatch.rs` (`redact_userinfo_*`), `bus_handler.rs`
(`wr01_*`, `precheck_mirror_drift_redacts_*`), `worktree_helpers.rs` (`wr03_*`), `tests/sync.rs`
(`sync_parse_error_redacts_*`). Unifying the two into one canonical `reposix_core::http` redactor
is a tracked GOOD-TO-HAVE (v0.15.0).

### The shared `errmsg::teach` primitive

`reposix_core::errmsg::{Teach, teach}` is the ONE builder every 3-part error routes
through — pure string formatting (no `anyhow`, `forbid(unsafe_code)`-clean). Renders
`<headline>\nFix: …\nAlternative: …\nRecovery:\n  <cmd>…`, with the `Fix:`/`Alternative:`/
`Recovery:` limbs each independently omitted when empty (a hollow `Alternative:` line
never appears for an error with no genuine alternative). Full API + the `.code()`
slot that attaches an `RPX-xxxx` code (now LIVE — see § The `RPX-xxxx` error-code
registry below): doc comments in `crates/reposix-core/src/errmsg.rs`.

### The `RPX-xxxx` error-code registry + `reposix explain` (P121, live)

Every stable error code lives ONCE in `crates/reposix-core/src/codes.rs`: an
`ids::*` name constant (typo-proof — call sites reference `ids::CACHE_BUILD`, never
the bare `"RPX-0201"`) plus one `ExplainEntry` in `REGISTRY` carrying the extended
cause/fix/alternative/recovery that `reposix explain <code>` prints (the
`rustc --explain`-grade half of the north star). All fields are `&'static str` — no
remote byte reaches the code slot or the explain output (OP-2).

**Render path (`errmsg.rs`):** `teach_coded(ids::NAME, headline, fix, alt, recovery)`
(or `Teach::new(headline)…​.code(ids::NAME)`). The `[RPX-xxxx]` tag rides the FIRST
headline line and an `Explain: reposix explain RPX-xxxx` nudge trails the body; an
UNSET code renders byte-identical to the P120 no-code shape (the ~40 uncoded sites
are untouched). Terse limbs come from the CALL SITE, extended prose from the
REGISTRY — the two tiers are SUPPOSED to differ, so there is no single-render
coherence gate.

**Adding a code:** (1) add a `pub const` to `ids` + an `ExplainEntry` to `REGISTRY`
in `codes.rs`; (2) emit it at the call site via `teach_coded(ids::NAME, …)` /
`.code(ids::NAME)`. The gate `quality/gates/agent-ux/rpx_registry_check.py` enforces
BOTH directions: every EMITTED code is registered, AND — leg 5, reverse-completeness
— every REGISTERED code is actually emitted somewhere (a registered-but-unused code
FAILS the gate; `EMISSION_EXEMPT` is empty). It also requires each entry to teach a
non-empty cause/fix/recovery and each code to be a unique four-digit `RPX-\d{4}`.
User-facing index: `docs/reference/error-codes.md`; always-current runtime
enumerator: `reposix explain --list`. (`codes.rs` is a deliberate oversized single
source of truth — see GOOD-TO-HAVES GTH-V15-68.)

**RPX-0508 (P122 W2 / DRAIN-08)** covers the helper's NON-absence import-parent-resolve
failure: `main.rs::resolve_import_parent` now returns `anyhow::Result<Option<ImportParent>>`
and distinguishes a genuine ref-absent first fetch (`Ok(None)` → parentless seed) from a
loud non-absence `git rev-parse` fault (spawn failure / non-1 non-zero exit / signal /
anomalous exit-0-empty-stdout → `Err` coded RPX-0508 via `import_parent_resolve_detail`,
surfaced through `fail_push`), instead of silently degrading to the parentless overlay.

**RPX-0406 (P122 W3 / DRAIN-09)** is the `reposix init` binary-side backstop for the D2
shared-tree-corruption recurrence (the sibling of RPX-0401's existing-repo-root refusal).
`init.rs` refuses a FRESH target that nests inside a non-/tmp git working tree
(`refuse_nested_in_worktree`, canonicalized via `canonicalize_lexical_existing` with
`realpath -m` semantics, mirroring `.claude/hooks/leaf-isolation-guard.sh::is_safe`'s /tmp
safe zone) AND self-checks its git-dir after `git init` but before any `git config` write
(`assert_own_git_dir`) — both emit RPX-0406, both run in `run_with_since`, init-only (never
`attach`). Only a refusal INSIDE the binary cuts a subprocess/worktree bypass of the
Bash-tool hook. Verifier: `quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh`.

### `errors.rs` shape helpers + scan scope

`reposix-cli/src/errors.rs` wraps `teach()` in the ~4 recurring failure shapes shared
across subcommands (`spec_parse_error`, `missing_env_var_error`, `cache_build_error`,
`missing_cache_db_error`) so `init`/`attach`/`sync`/`refresh`/`tokens`/`cost`/`gc`/
`history` emit the SAME teaching for the SAME failure instead of ~8 hand-rolled
near-duplicates. `quality/gates/agent-ux/teach_scan.py` enforces the bar over an
EXPLICIT, enumerated file list (`CLI_SCOPE` / `HELPER_SCOPE` constants in the script) —
adding a new subcommand file to the surface is a reviewable one-line change to that
list, never a silent gap.

### `doctor.rs` is exempt from the teach-scan by scope, not by marker

`reposix doctor` is deliberately OUTSIDE `teach_scan.py`'s `CLI_SCOPE` list: it emits a
structured `DoctorReport`/`DoctorFinding` (each finding already carries its own
copy-pastable fix command) rather than raising a `bail!`/`anyhow!` error, so the 3-part
teaching-error bar does not apply to it — there is nothing to scan. Don't add
`doctor.rs` to `CLI_SCOPE` expecting the scanner to grade it; the report SHAPE (not the
scanner) is doctor's teaching contract.

### The `// teach-exempt: ok — <reason>` marker

A `bail!`/`anyhow!` block that is intentionally NOT a user-facing teaching error (an
internal filesystem/git-subprocess wrapper surfacing someone else's message verbatim, a
machine-derived path already validated upstream, a per-action wrap whose terminal
teaching happens at the caller) is dispositioned inline with a `// teach-exempt: ok —
<reason>` comment rather than silently skipped — `teach_scan.py` requires the marker to
sit on the block's own line or within 2 comment/blank lines above it, or the block RAISEs
as un-dispositioned. Grep `teach-exempt: ok` across `crates/` for ~20 live examples
(`gc.rs`, `init.rs`, `backend_dispatch.rs`, `main.rs`) showing the reasoning style to
match — name WHY the site is exempt, not just that it is.

### Bin-target vs integration-target test location (the `cargo test -p` seam)

A `#[cfg(test)]` module inside a crate's **bin target** (`src/main.rs` or a module it
pulls in, e.g. `reposix-remote/src/bus_handler.rs`) is invisible to `cargo test -p
<crate> --test <name>` — that flag runs ONLY the named **integration-target** file under
`tests/`. It IS covered by the bare `cargo test -p <crate>` (no `--test` flag), which
runs the lib target, every bin target's own unit tests, AND every integration target.
This bit a real gate: `quality/gates/agent-ux/helper-errors-teach-recovery.sh` originally
scoped leg (a) to `--test errors_teach_recovery`, so the W5/WR-01 credential-leak
regression test (`bus_handler.rs::tests::wr01_mirror_partial_fail_scrubs_token_from_both_audit_row_and_diag`,
a bin-target unit test) silently sat outside the gate's coverage — fixed P120 close by
widening to the bare `cargo test -p reposix-remote`. When a regression test's home is a
bin-target `#[cfg(test)]` module rather than a `tests/*.rs` file, grade/gate it with the
bare per-crate invocation, not a `--test <name>`-scoped one.
