---
phase: 35
status: research_complete
date: 2026-04-24
researcher: phase-runner (inline)
---

# Phase 35 — Research

Compiled from existing repo conventions (32-SUMMARY, 34-VERIFICATION, `crates/reposix-cli/src`, `reposix-confluence/tests/contract.rs::skip_if_no_env!`) plus the architecture-pivot summary §5. Goal: surface the *cleanest* path through the four plans.

## R1. Replacing `reposix mount` with `reposix init`

**Approach:** add a new `Init` clap variant; keep the `Mount` variant as a stub that emits a one-line migration message and exits 2. Phase 36 will delete both `Mount` and `mount.rs`.

**`reposix init <backend>::<project> <path>` — implementation shape:**

1. Parse the positional `<backend>::<project>` (split on `::`). Accepted backends: `sim`, `github`, `confluence`, `jira`. (For `github::owner/repo`, split-after-colon yields `owner/repo`.)
2. Translate to a remote-helper-compatible URL:
   - `sim::<slug>` → `reposix::http://127.0.0.1:7878/projects/<slug>` (matches existing `parse_remote_url` shape).
   - `github::<owner>/<repo>` → `reposix::https://api.github.com/projects/<owner>/<repo>` (placeholder; the helper currently hardcodes `SimBackend` — the URL is what we set today, but the helper itself remains sim-only until a future phase teaches it real backends).
   - `confluence::<space>` → `reposix::https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/projects/<space>` when the tenant env var is set.
   - `jira::<key>` → `reposix::https://${REPOSIX_JIRA_INSTANCE}.atlassian.net/projects/<key>` when set.
3. Run six git invocations in sequence via `std::process::Command`:
   - `git init <path>`
   - `git -C <path> config extensions.partialClone origin`
   - `git -C <path> config remote.origin.url <url>`
   - `git -C <path> config remote.origin.promisor true`
   - `git -C <path> config remote.origin.partialclonefilter blob:none`
   - `git -C <path> fetch --filter=blob:none origin` *(best-effort — sim backend is the default; real backends require creds. If the fetch fails we still succeed at "init" because the local repo is configured, and the failure mode for `git fetch` is informative)*.

   Per the architecture-pivot-summary §5 the working tree is left empty (no auto-`git checkout`); the agent's first read or `git checkout origin/main` triggers blob materialization.

4. On any git invocation failure: bubble the error up via `anyhow::Result` so the user sees the root cause.

**No need for `git2`/`gix`** — six shell-out calls is simpler than pulling in a 2MB dependency.

**Mount-stub message (locked from CONTEXT):**
> `reposix mount has been removed in v0.9.0. Use 'reposix init <backend>::<project> <path>' — see CHANGELOG and docs/reference/testing-targets.md.`

Exit code 2 (CLI usage convention). Implement as a fresh `Mount` variant returning `Err(anyhow!(...))` so existing `cli::help_lists_all_subcommands` test still finds `mount` in `--help`.

## R2. Dark-factory regression test (sim, no Claude subagent)

**Decision:** the Phase 35 dark-factory test is a **shell script** that simulates an agent's stderr-reading + retry behaviour. The point is to prove the *error-message UX* is self-teaching, not to spawn a real Claude (too brittle, too slow). Phase 36's skill spec can land that as a follow-up.

**Test shape:** integration test in `crates/reposix-cli/tests/agent_flow.rs` that:

1. Spawns `reposix-sim --ephemeral` on a free port (use `tempfile::TempDir`).
2. Runs `reposix init sim::demo /tmp/test-repo` via `assert_cmd::Command`.
3. Validates: the dir is a git working tree, `git config remote.origin.url` returns the expected `reposix::http://...` URL, `extensions.partialClone` is set.
4. Edits an issue file, runs `git commit`, runs `git push` — asserts ok.
5. **Conflict path:** mutate one issue via direct REST POST against the sim mid-flight, agent's `git push` rejects, agent runs `git pull --rebase`, retries.
6. **Blob-limit path:** with `REPOSIX_BLOB_LIMIT=3`, attempt a checkout that requests >3 blobs → fails → grep stderr for `git sparse-checkout` → run `git sparse-checkout set …` → retry → succeeds.

**Test gating:** `#[ignore]`'d by default because it requires a real `git` binary on PATH (we already assume git ≥ 2.34 — see ARCH-13 requirements list). Run via `cargo test -p reposix-cli --test agent_flow -- --ignored`.

**Why a shell wrapper too?** Per the CONTEXT decisions, `scripts/dark-factory-test.sh <backend>` lets the same flow be invoked from CI and from a local dev `/reposix-agent-flow` skill (Phase 36). Make the Rust test call the script when the script exists; otherwise inline the steps. This keeps the script as the canonical agent-facing artifact (per OP-4 "ad-hoc bash is a missing-tool signal" — promote the bash to a committed script).

## R3. Capturing wall-clock latency per git step

**Approach:** a benchmark Rust test (`crates/reposix-cli/tests/latency.rs`) that:

1. Spawns the sim.
2. Times each step using `std::time::Instant::now()` deltas:
   - clone (`git clone --filter=blob:none`)
   - first-blob fetch (`git cat-file -p HEAD:issues/<first>.md`)
   - sparse-checkout batched checkout of 10 blobs
   - edit + commit
   - push
3. Writes `docs/benchmarks/v0.9.0-latency.md` with a Markdown table.

For real backends, the same harness gated by `skip_if_no_env!`. Each backend's column populated when creds are present; empty otherwise.

**Format chosen for sales-asset readability:**

```
| Step                                | sim    | github | confluence | jira |
|-------------------------------------|--------|--------|------------|------|
| Cold clone (--filter=blob:none)     | 142ms  | 980ms  | 1.4s       |      |
...
```

Two-paragraph "How to read this" intro before the table, "Reproduce" command block at the end. Soft thresholds (sim cold clone < 500ms) asserted via `assert!`-with-warning (we use `eprintln!("WARN: …")` rather than `panic!`) so a regression flags but does not break CI.

## R4. Detecting missing creds and skipping gracefully

Use the existing `skip_if_no_env!` macro pattern verbatim from `reposix-confluence/tests/contract.rs` lines 61–74. Copy into the new test file (the macro is locally defined per-test-file in the existing pattern; we follow the same idiom). The macro:

- Iterates the env-var list, collects missing keys (no values logged — per T-11B-01).
- If any are missing, prints `SKIP: env vars unset: …` to stderr and `return`s.

**Per-backend env contracts:**
- GitHub: `GITHUB_TOKEN`.
- Confluence: `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`.
- JIRA: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`. JIRA project key from `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`, default `TEST`.

## R5. Markdown benchmark format that doubles as a sales asset

The artifact at `docs/benchmarks/v0.9.0-latency.md` follows this skeleton:

```markdown
# v0.9.0 Latency Envelope

## How to read this
[2 paragraphs on what the numbers mean, what's measured, what's NOT measured (network jitter, runner variance, etc.), and the comparison to MCP/REST baselines from existing v0.7.0 benchmarks.]

## Latency table
| Step | sim | github | confluence | jira |
|------|-----|--------|------------|------|
| ...  | ... | ...    | ...        | ...  |

## Soft thresholds
- sim cold clone < 500ms (regression-flagged, not CI-blocking)
- real-backend < 3s

## Reproduce
\`\`\`bash
cargo test -p reposix-cli --test latency -- --ignored
\`\`\`
```

The "Reproduce" block is the OP-4 dogfood path — the same script the user uses locally is the script the doc references.

## R6. `docs/reference/testing-targets.md` structure

Three H2 sections (one per target) + an env-var subsection + a cleanup subsection per target. Owner-quote at top:

> "TokenWorld is for testing — go crazy, it's safe."
> — project owner, 2026-04-24

Cross-link to CLAUDE.md commands section. Per ARCH-18 each section enumerates: env vars (named only — never values), rate-limit notes, cleanup procedure, "go crazy, it's safe" permission statement (verbatim per the literal-string requirement in the prompt).

## RESEARCH COMPLETE
