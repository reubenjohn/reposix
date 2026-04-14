---
phase: 11-confluence-adapter
plan: B
subsystem: cli-dispatch
tags: [cli, confluence, fuse, ci, SG-01]
dependency_graph:
  requires:
    - reposix_confluence::ConfluenceReadOnlyBackend
    - reposix_confluence::ConfluenceCreds
    - reposix_core::IssueBackend
  provides:
    - reposix list --backend confluence
    - reposix mount --backend confluence
    - reposix-fuse --backend-kind confluence
    - CI job `integration-contract-confluence`
  affects:
    - Cargo.lock (new reposix-confluence dep in cli + fuse)
tech-stack:
  added: []
  patterns:
    - "pure-fn read_confluence_env_from(get) for test-safety (avoids Rust's unsound-on-parallel env mutation)"
    - "fail-fast env + allowlist guard before spawning reposix-fuse child"
    - "CI job gated on ALL FOUR Atlassian secrets being non-empty (fork-safe + unconfigured-tenant-safe)"
key-files:
  created: []
  modified:
    - crates/reposix-cli/Cargo.toml
    - crates/reposix-cli/src/list.rs
    - crates/reposix-cli/src/mount.rs
    - crates/reposix-fuse/Cargo.toml
    - crates/reposix-fuse/src/main.rs
    - .github/workflows/ci.yml
decisions:
  - "Pure-fn env reader: `read_confluence_env_from(get: impl Fn(&str)->String)` keeps the unit test from touching real process env (Rust 1.66+ `std::env::set_var` is `unsafe` under Miri-detected UB). The live wrapper is a 1-line `|n| std::env::var(n).unwrap_or_default()`."
  - "Error messages name env vars but never echo values (T-11B-01). Unit test asserts tenant + email values do not appear in the error string."
  - "Mount.rs does the courtesy pre-check for ATLASSIAN_EMAIL + ATLASSIAN_API_KEY so the user sees a CLI error rather than an opaque fuse-daemon exit; the fuse binary itself ALSO re-checks (defense in depth)."
  - "CI job gated on 4 secrets (including REPOSIX_CONFLUENCE_SPACE) not 3; the contract test in 11-C needs a specific space key to probe. Matches 00-CREDENTIAL-STATUS.md convention."
metrics:
  duration: "~8 minutes"
  completed: 2026-04-14
  tasks_completed: 3
  tests_added: 3
  workspace_tests_before: 186
  workspace_tests_after: 189
  commits: 3
---

# Phase 11 Plan B: CLI dispatch Summary

Wired `--backend confluence` into `reposix list`, `reposix mount`, and
the `reposix-fuse` binary, plus added a CI job
`integration-contract-confluence` gated on all four Atlassian secrets.
Live verification against the real `reuben-john.atlassian.net` tenant
returned 4 pages (space home + 3 seeded demo pages) in both `--format
table` and `--format json` paths.

## Tasks

| Task | Name                                                        | Commit    | Files                                                                                                  |
| ---- | ----------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------ |
| 1    | Extend `ListBackend` + `reposix list` dispatch (TDD)        | `aa611c6` | `crates/reposix-cli/{Cargo.toml,src/list.rs,src/mount.rs}`, `Cargo.lock`                                |
| 2    | `MountProcess::spawn` + `reposix-fuse` binary arms          | `92e2e91` | `crates/reposix-cli/src/mount.rs`, `crates/reposix-fuse/{Cargo.toml,src/main.rs}`, `Cargo.lock`          |
| 3    | `integration-contract-confluence` CI job                    | `88c3b0c` | `.github/workflows/ci.yml`                                                                              |

## CLI Help-text Changes (verbatim, for 11-E README work)

`reposix list --help` now exposes:

```
--backend <BACKEND>
    Which backend to query

    Possible values:
    - sim:        In-process simulator at the configured `--origin`
    - github:     Real GitHub Issues at `api.github.com`. `--project` is `owner/repo`
    - confluence: Real Atlassian Confluence Cloud REST v2. `--project` is the space key
                  (e.g. `REPOSIX`). Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
                  `REPOSIX_CONFLUENCE_TENANT` env vars plus `REPOSIX_ALLOWED_ORIGINS`
                  that includes the tenant origin
```

`reposix mount --help` exposes the same three values in its
`--backend` enum. `reposix-fuse --backend-kind` (invoked internally
but user-facing for advanced users) exposes `sim|github|confluence`
with matching doc strings.

## Workspace Verification

| Check                                                                | Result                            |
| -------------------------------------------------------------------- | --------------------------------- |
| `cargo fmt --all -- --check`                                         | clean                             |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`     | clean                             |
| `cargo build --workspace` (also `cargo build --release --workspace`) | green                             |
| `cargo test --workspace --locked` total                              | **189 passed, 0 failed** (+3 new) |
| `cargo test -p reposix-cli --locked confluence`                      | 3 passed, 0 failed                |
| `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` | exits 0                     |

## Env-var-missing error (T-11B-01 check)

```
$ env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT \
    reposix list --backend confluence --project REPOSIX
Error: confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars

Caused by:
    confluence backend requires these env vars; currently unset:
    ATLASSIAN_EMAIL, ATLASSIAN_API_KEY, REPOSIX_CONFLUENCE_TENANT.
    Required: ATLASSIAN_EMAIL (your Atlassian account email),
    ATLASSIAN_API_KEY (token from id.atlassian.com/manage-profile/security/api-tokens),
    REPOSIX_CONFLUENCE_TENANT (your `<tenant>.atlassian.net` subdomain).
```

Exit code: 1. All three names present. No values echoed (the error
string is derived solely from the names list + fixed boilerplate,
and the unit test `confluence_requires_all_three_env_vars` asserts
that partial-set values do NOT appear in the error).

## Demo regression — smoke.sh stayed 4/4

```
smoke suite: 4 passed, 0 failed (of 4)
```

`PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh` —
unchanged (Tier 1 demos were untouched by this plan).

## SKIP-path verification — 06-mount-real-confluence.sh

With all four Atlassian env vars unset (the production-safe state
for dev hosts and fork CI):

```
$ env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT \
      -u REPOSIX_CONFLUENCE_SPACE PATH="$PWD/target/release:$PATH" \
      bash scripts/demos/06-mount-real-confluence.sh
SKIP: env vars unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_CONFLUENCE_SPACE
      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to
      run this demo.
== DEMO COMPLETE ==
```

Exit code: 0. Demo still gracefully skips with the new release
binary in place (proving the binary was not the reason the SKIP
path exited 0 — the demo's own env check was).

## LIVE VERIFICATION (optional) — WORKS

With `.env` loaded:

```
$ set -a; source .env; set +a
$ export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
$ PATH="$PWD/target/release:$PATH" reposix list --backend confluence --project REPOSIX --format table
ID         STATUS       TITLE
---------- ------------ ----------------------------------------
65916      open         Architecture notes
131192     open         Welcome to reposix
360556     open         reposix demo space Home
425985     open         Demo plan
```

Exit 0. The tenant returns 4 pages (the prompt's `prerequisite_state`
expected 3 — the 4th is the space home page auto-provisioned when
the REPOSIX space was created; that's correct Confluence behaviour
and a useful data point for 11-C's contract test expectations).

The JSON form also works:

```
$ ... reposix list --backend confluence --project REPOSIX --format json | head -12
[
  {
    "id": 65916,
    "title": "Architecture notes",
    "status": "open",
    "assignee": "557058:dd5e2f19-5bf6-4c0a-be0b-258ab69f6976",
    "labels": [],
    "created_at": "2026-04-14T04:16:31.725Z",
    "updated_at": "2026-04-14T04:16:31.725Z",
    "version": 1,
    "body": ""
  },
```

Observations for 11-C / 11-E:

- `body: ""` on list results is expected — Confluence v2's list
  endpoint doesn't include body unless `body-format` is requested
  per-page; `get_issue` is what populates `body.storage.value`.
- `assignee` carries the Atlassian accountId (`ownerId` from the
  page doc) — consistent with the ADR-002 Option A mapping.
- The list is NOT sorted by id or title by default (Atlassian
  returns its own ordering; tests and parity demos should not
  assume id-ordering).

## Success criteria — results

| # | Criterion                                                                                       | Result |
|---|-------------------------------------------------------------------------------------------------|--------|
| 1 | `grep -q 'Confluence,' crates/reposix-cli/src/list.rs`                                          | PASS   |
| 2 | `grep -q 'Confluence,' crates/reposix-cli/src/mount.rs`                                         | **spec-literal FAIL** (see below) |
| 3 | `grep -q 'Confluence,' crates/reposix-fuse/src/main.rs`                                         | PASS   |
| 4 | `grep -q 'reposix-confluence = { path = "../reposix-confluence" }' crates/reposix-cli/Cargo.toml` | PASS   |
| 5 | `grep -q 'reposix-confluence = { path = "../reposix-confluence" }' crates/reposix-fuse/Cargo.toml`| PASS   |
| 6 | `grep -q 'integration-contract-confluence' .github/workflows/ci.yml`                            | PASS   |
| 7 | `grep -q 'cargo test -p reposix-confluence -- --ignored' .github/workflows/ci.yml`              | PASS   |
| 8 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` exits 0            | PASS   |
| 9 | `cargo build --workspace --locked` exits 0                                                      | PASS   |
| 10| `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0                        | PASS   |
| 11| `cargo test --workspace --locked` exits 0 with ≥180 tests passed                                | PASS (189) |
| 12| `cargo test -p reposix-cli --locked confluence` produces `test result: ok`                     | PASS   |
| 13| env-missing error lists all three vars in a single stderr line                                  | PASS   |
| 14| `bash scripts/demos/smoke.sh` exits 0                                                           | PASS (4/4) |

### SC-2 (spec-literal FAIL) analysis

`grep -q 'Confluence,' crates/reposix-cli/src/mount.rs` is false
because mount.rs only uses the enum via `ListBackend::Confluence =>`
patterns (match arms) and `ListBackend::Confluence` comparisons —
neither of which leaves a trailing comma immediately after
`Confluence`. The enum **definition** (where `Confluence,` would
appear) lives in `list.rs`, which mount.rs `use`s.

The plan's frontmatter `artifacts[mount.rs].contains:
"ListBackend::Confluence"` matches this file (3 occurrences, lines 62
and 96 — `match` arm + `backend_kind` arm). That's the substantive
requirement; SC-2's literal form is a plan-wording bug, not a
deliverable miss.

No code change made to chase the literal grep — that would have
required a comment-only insertion with `Confluence,` in the source,
which is grounding-artefact pollution. Flagging for 11-E author to
clean up SC-2 wording in any future similar plan.

## Deviations from Plan

### [Rule 3 - Blocking] Task 1 needed a mount.rs stub to compile

**Found during:** Task 1 `cargo test`.
**Issue:** Adding `ListBackend::Confluence` made mount.rs's
`match backend` non-exhaustive (`match_same_arms` or
`non_exhaustive_patterns` E0004). Without a stub, Task 1 couldn't
compile in isolation.
**Fix:** Added a one-line placeholder arm
`ListBackend::Confluence => "confluence"` with a comment pointing
to 11-B-2. Task 2 replaced the placeholder with the full allowlist
+ env-var guard.
**Files modified:** `crates/reposix-cli/src/mount.rs` (one line,
Task 1) → replaced in Task 2.
**Commit:** `aa611c6` (Task 1) / `92e2e91` (Task 2).
**Classification:** Rule 3 (blocking) — atomic Task 1 commit required
exhaustive match. No scope creep; placeholder replaced in Task 2 as
planned.

### [Spec wording] SC-2 literal grep is incorrect

See "SC-2 (spec-literal FAIL) analysis" above. The substantive
deliverable is met; the literal grep pattern in the plan's
success-criteria section doesn't match Rust match-arm syntax.
Flagged for the 11-E README author to not repeat.

## Auth Gates

**None.** `.env` was already populated with valid Atlassian
credentials from a previous session's `chore(11)` commit
(`1e58dd0`). The live verification (`reposix list --backend
confluence --project REPOSIX`) worked on the first attempt.

## Known Stubs

**None.** All three tasks completed with no placeholder behaviour in
the shipped code paths. The Task 1 temporary `ListBackend::Confluence
=> "confluence"` arm in mount.rs was replaced in Task 2 (commit
`92e2e91`) with the full allowlist + four-env-var guard before any
user-facing binary consumed it.

## Threat-model Verification

| Threat   | Status               | Evidence                                                                                                                                                          |
| -------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-11B-01 | **mitigated + tested** | Test `confluence_requires_all_three_env_vars` asserts `user@example.com` and `mytenant` values do NOT appear in the error string despite being passed to `get()`. |
| T-11B-02 | mitigated            | Tenant substring match uses `format!("{tenant}.atlassian.net")` for allowlist display only. URL construction remains inside `ConfluenceReadOnlyBackend::new` (11-A Task 2, validated). |
| T-11B-03 | mitigated            | CI job `if:` clause gates on all four secrets; `needs: [test]` prevents wasted Atlassian quota on a broken build. Fork PRs lack the secrets and are auto-skipped. |
| T-11B-04 | accepted             | Documented in plan; Atlassian redacts `secrets.*` names from logs; 11-A's Debug redaction covers in-process tracing spans. |
| T-11B-05 | mitigated            | `timeout-minutes: 5` caps runtime; contract test hits ≤3 endpoints per run (well under Atlassian's 1000 req/min soft cap). |

## Self-Check: PASSED

- `crates/reposix-cli/Cargo.toml` contains `reposix-confluence = { path = "../reposix-confluence" }` — FOUND
- `crates/reposix-cli/src/list.rs` contains `ListBackend::Confluence` — FOUND (4 hits)
- `crates/reposix-cli/src/mount.rs` contains `ListBackend::Confluence` — FOUND (3 hits)
- `crates/reposix-fuse/Cargo.toml` contains `reposix-confluence = { path = "../reposix-confluence" }` — FOUND
- `crates/reposix-fuse/src/main.rs` contains `BackendKind::Confluence` — FOUND
- `.github/workflows/ci.yml` contains `integration-contract-confluence` — FOUND (line 113)
- `.github/workflows/ci.yml` contains `cargo test -p reposix-confluence -- --ignored` — FOUND (line 139)
- Commit `aa611c6` (feat(11-B-1)) — FOUND in `git log`
- Commit `92e2e91` (feat(11-B-2)) — FOUND in `git log`
- Commit `88c3b0c` (ci(11-B-3)) — FOUND in `git log`
- `cargo test --workspace --locked` = 189 passed — VERIFIED
- `bash scripts/demos/smoke.sh` = 4/4 passed — VERIFIED
- Live verification against real Atlassian tenant — WORKS (4 pages returned)
