---
title: Exit codes
---

# Exit codes

Reposix follows POSIX exit-code conventions. All `reposix` subcommands
exit with `0` on success and `1` on any handled error (anyhow
propagation through `fn main() -> Result<()>`). The `git-remote-reposix`
helper is the one binary with a richer 3-tier code, because git itself
inspects the helper's exit status when deciding how to surface failures
to the user.

This page lists every distinct exit code emitted by reposix today.
Structured JSON output (`--json` / `--format=json`) is queued for
v0.12.0 — see the **Future** section at the bottom. Until then,
machine integrations should anchor on **exit code + stderr prefix**,
both of which are stable.

## TL;DR for harness authors

| Binary | Code | Meaning |
|--------|------|---------|
| `reposix <any subcommand>` | `0` | success |
| `reposix <any subcommand>` | `1` | any handled error (parse, IO, network, validation, conflict) |
| `reposix doctor` | `0` | no `ERROR`-severity findings (may still emit `WARN`/`INFO`) |
| `reposix doctor` | `1` | at least one `ERROR`-severity finding |
| `git-remote-reposix` | `0` | protocol session completed; no push rejection |
| `git-remote-reposix` | `1` | push rejected (e.g. push-time conflict) — git treats this as `! [remote rejected]` |
| `git-remote-reposix` | `2` | helper crashed / unrecoverable error (anyhow `Err` from `real_main`) |

If you only have time for one rule: **`exit == 0` means success;
anything else means parse stderr.**

## Per-subcommand exit codes (`reposix` CLI)

Every `reposix` subcommand handler returns `anyhow::Result<()>`, which
the top-level `#[tokio::main] async fn main() -> Result<()>` converts
to exit `0` (Ok) or exit `1` (Err). There are no custom exit codes for
recoverable-vs-fatal errors at this layer; all failures collapse to
`1`. The single exception is `reposix doctor`, which calls
`std::process::exit(report.exit_code())` after printing its findings
table.

| Code | Subcommand | Condition | Source |
|------|------------|-----------|--------|
| `0` | all | success | `Ok(())` from handler |
| `1` | `init` | invalid `<backend>::<project>` spec, unknown backend, `git init`/`git fetch` failure, time-travel `--since` finds no matching sync tag | `crates/reposix-cli/src/init.rs` (multiple `bail!` sites) |
| `1` | `sim` | child `reposix-sim` process exited non-zero, bind-port already in use, seed-file parse error | `crates/reposix-cli/src/sim.rs:194` |
| `1` | `list` | missing required env vars for non-sim backends (`ATLASSIAN_*`, `JIRA_*`, `GITHUB_TOKEN` per backend), allowlist denial, REST 4xx/5xx | `crates/reposix-cli/src/list.rs:204,254` |
| `1` | `refresh` | `--offline` (Phase 21 not implemented), backend error, `git commit`/`git init` failure, working-tree validation failure | `crates/reposix-cli/src/refresh.rs:67,192,215,265,278,309` |
| `1` | `spaces` | `--backend sim`, `--backend github`, or `--backend jira` (only Confluence supported) | `crates/reposix-cli/src/spaces.rs:26-32` |
| `0` | `doctor` | no `ERROR`-severity findings (clean tree, or only `INFO`/`WARN`) | `crates/reposix-cli/src/doctor.rs:164-166` |
| `1` | `doctor` | at least one `ERROR`-severity finding (e.g. missing `extensions.partialClone`, `git rev-parse` failure, cache directory missing) | `crates/reposix-cli/src/main.rs:387-390` (calls `std::process::exit(report.exit_code())`) |
| `1` | `log` | called without `--time-travel` (the bare form is reserved for a future commit-graph view) | `crates/reposix-cli/src/main.rs:363-367` |
| `1` | `history` / `at` | working tree missing, no sync tags in cache, RFC-3339 parse failure, no tag at-or-before timestamp | `crates/reposix-cli/src/history.rs:26` |
| `1` | `tokens` | working tree missing, no `cache.db` audit log alongside the cache | `crates/reposix-cli/src/tokens.rs:63,81` |
| `1` | `cost` | working tree missing, no `cache.db`, `--since` parse failure | `crates/reposix-cli/src/cost.rs:113,260` |
| `1` | `gc` | invalid strategy combo, working tree missing, IO failure walking the cache | `crates/reposix-cli/src/gc.rs:72` |
| `0` | `version` | always | `crates/reposix-cli/src/main.rs:314-317` |

### What "anyhow propagation" looks like in practice

Almost every error path in the CLI is shaped like:

```rust
anyhow::bail!("confluence backend requires these env vars; \
              currently unset: {}. Required: ATLASSIAN_EMAIL ...", missing);
```

`bail!` returns `Err(anyhow::Error)`, which propagates up through the
`?` operator until it reaches `fn main() -> Result<()>`. Anyhow's
default `Termination` impl then prints the error chain to stderr
(prefixed with `Error: `) and exits with code `1`. There is no `try`/
`catch` layer in between — every error becomes exit `1`.

If you need to discriminate between "missing env var" and "REST 5xx",
**parse the stderr message**, not the exit code. The error messages
are stable (changing them would break this file's table), but they are
not yet machine-formatted.

## `git-remote-reposix` exit codes

The helper has a 3-tier code, set in `crates/reposix-remote/src/main.rs:78-95`:

```rust
fn main() -> ExitCode {
    match real_main() {
        Ok(true)  => ExitCode::SUCCESS,   // 0 — protocol session OK, no push rejected
        Ok(false) => ExitCode::from(1),   // 1 — push rejected (state.push_failed = true)
        Err(e)    => { diag(...); ExitCode::from(2) }  // 2 — helper crashed
    }
}
```

| Code | Meaning | Typical trigger | Recovery |
|------|---------|-----------------|----------|
| `0` | Success. The helper completed every requested verb (`capabilities` / `list` / `import` / `export` / `stateless-connect`) and did not refuse a push. | Normal `git fetch` and `git push`. | n/a |
| `1` | Push refused at the protocol layer. The helper sent `error refs/heads/main fetch first` to git, which surfaces to the user as `! [remote rejected] main -> main (fetch first)`. | Push-time conflict detection (`crates/reposix-remote/src/main.rs:402`): backend version drifted from the local base since the last fetch. | `git pull --rebase && git push`. |
| `2` | Unrecoverable helper error. `real_main` returned `Err`. The helper writes `git-remote-reposix: <error chain>` to stderr before exiting. | Argv parse failure, URL parse failure, backend instantiation failure (missing creds, unknown scheme), egress allowlist denial during cache materialization, IO failure on the cache path. | Read stderr, fix the underlying issue. |

### Recognizable failure modes inside exit `2`

These are the three failure modes a harness author is most likely to
hit. All of them currently exit with `2` because they propagate as
`anyhow::Error`. Anchor on the stderr substring, not the exit code,
to discriminate.

- **Blob-limit refusal.** Stderr contains the literal substring
  ``error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry.``
  (defined in `crates/reposix-remote/src/stateless_connect.rs:55`).
  The helper aborts the `command=fetch` RPC before materializing any
  blobs and writes an audit row (`log_blob_limit_exceeded`).
  Recovery: `git sparse-checkout set <pathspec>` then re-run the
  fetch. Default limit is `200`; override with `REPOSIX_BLOB_LIMIT`.

- **Egress-allowlist denial.** The cache or backend connector tried to
  reach an origin not in `REPOSIX_ALLOWED_ORIGINS`. The error chain
  contains `blocked origin: <url>` (defined in
  `crates/reposix-core/src/error.rs:39-40`). Recovery: extend
  `REPOSIX_ALLOWED_ORIGINS` to include the origin, or point the remote
  URL at an allowlisted origin (the simulator at `http://127.0.0.1:7878`
  is allowlisted by default).

- **Missing backend credentials.** Backends that need auth env vars
  (`ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`
  for Confluence; `JIRA_EMAIL` + `JIRA_API_TOKEN` +
  `REPOSIX_JIRA_INSTANCE` for JIRA; `GITHUB_TOKEN` for GitHub above the
  unauthenticated rate limit) bail at instantiation with a single
  message that lists every missing var. Recovery: set the vars (see
  `docs/reference/testing-targets.md`) and retry.

### What git itself shows the user

Git surfaces the helper's exit code through its own UI:

- Helper exit `0` with a successful push → `Writing objects: 100% ...`
  followed by a green ref-update line. Nothing reposix-specific.
- Helper exit `1` (push rejected) → `! [remote rejected] main -> main
  (fetch first)` followed by `error: failed to push some refs to ...`.
  This is the standard git "fetch first" path, which is by design — an
  agent does not need to know reposix exists to recover (`git pull
  --rebase && git push`).
- Helper exit `2` → git prints `fatal: ...` with whatever the helper
  wrote to stderr, prefixed with `git-remote-reposix:`. This is the
  path harness authors need to special-case.

## Stderr machine-readability

Reposix writes errors to stderr in two forms:

- **Anyhow `Error:` prefix** (CLI subcommands). The default
  `Termination` impl for `anyhow::Error` prints `Error: <message>` (note
  the capital `E` and trailing colon-space), followed by the cause
  chain on subsequent lines. For machine parsing, anchor on the literal
  prefix `Error: `.
- **Lowercase `error:` prefix** (helper, blob-limit, allowlist). Lines
  written via `eprintln!("error: ...")` use the conventional lowercase
  POSIX form. Anchor on the literal prefix `error: `.

The two prefixes are distinguishable by case. If your harness needs a
single regex, `^[Ee]rror: ` matches both.

Stderr lines are `\n`-terminated, UTF-8, and never re-emitted to
stdout. Stdout is reserved for command output (JSON from `reposix
list`, the rendered table from `reposix doctor` / `reposix tokens` /
`reposix cost`, the protocol stream from `git-remote-reposix`).

## Future: `--json` / `--format=json`

POLISH2-18 (this milestone) only documents what exists today. Per-row
structured output is queued for v0.12.0 and tracked alongside the
remaining v0.11.x polish in `.planning/REQUIREMENTS.md`. When that
ships, the plan is:

- Every CLI subcommand will accept `--format=json` (today only `reposix
  list` does). On error, the JSON object will carry a stable `code`
  field so harnesses can discriminate without parsing English.
- The helper will keep its 3-tier exit code (git's protocol shape
  cannot change), but will additionally write a single
  `{"event":"error","code":"<stable-id>","message":"..."}` line to
  stderr before exiting. Existing English error messages will continue
  to work for the lifetime of the v1.0.x line.

Until then, treat exit code + stderr prefix as the contract, and treat
the English messages as best-effort.
