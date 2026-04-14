# reposix demo suite

reposix demos come in three tiers. Pick the one that matches your
question:

- **Tier 1** — four ~60-second demos, one per audience. If you have
  one minute and want to know whether this thing is worth the next
  ten, start here.
- **Tier 2** — the full 9-step walkthrough. The monolith demo that
  ships as `scripts/demo.sh` (which is now a shim to
  [`scripts/demos/full.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/full.sh)). Watch
  this if Tier 1 convinced you to spend more than a minute.
- **Tier 3** — sim-vs-real-backend parity demo. Lists issues
  from the simulator and from real `octocat/Hello-World`, and diffs
  their normalized `{id, title, status}` shape. The diff IS the
  story: same schema, different content.

All demos are `set -euo pipefail`, self-cleaning, and bounded by a
90-second `timeout`. The smoke suite in
[`scripts/demos/smoke.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/smoke.sh) runs the
four Tier 1 demos through [`scripts/demos/assert.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/assert.sh)
and is what the `demos-smoke` CI job invokes on every push.

## Tier 1 — audience-specific 60s demos

| # | Demo                                                                       | Audience    | Runtime | What it proves                                                                                      | Recording                                                                                                                                                                         |
|---|----------------------------------------------------------------------------|-------------|--------:|-----------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | [`01-edit-and-push.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/01-edit-and-push.sh)           | developer   |    ~60s | FUSE mount + `cat`/`sed` edit + `git push` round-trip changes server state end-to-end.              | [typescript](recordings/01-edit-and-push.typescript) · [transcript](recordings/01-edit-and-push.transcript.txt)                                                                   |
| 2 | [`02-guardrails.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/02-guardrails.sh)                 | security    |    ~60s | SG-01 allowlist refusal + SG-02 bulk-delete cap + SG-03 sanitize-on-egress all fire on camera.      | [typescript](recordings/02-guardrails.typescript) · [transcript](recordings/02-guardrails.transcript.txt)                                                                         |
| 3 | [`03-conflict-resolution.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/03-conflict-resolution.sh) | skeptic     |    ~60s | `If-Match` + 409 `version_mismatch` is what git turns into a native merge conflict on push.         | [typescript](recordings/03-conflict-resolution.typescript) · [transcript](recordings/03-conflict-resolution.transcript.txt)                                                       |
| 4 | [`04-token-economy.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/04-token-economy.sh)           | buyer       |    ~10s | 92.3% fewer tokens vs MCP-mediated baseline for the same task.                                      | [typescript](recordings/04-token-economy.typescript) · [transcript](recordings/04-token-economy.transcript.txt)                                                                   |

### Audience guide

- **developer** — will I want to use this? (Demo 1 shows the shell
  ergonomics are real.)
- **security** — can I hand this to an autonomous agent? (Demo 2
  shows the lethal-trifecta cuts are mechanical.)
- **skeptic** — what happens when two agents race on the same
  issue? (Demo 3 shows conflicts surface as first-class git
  conflicts, not a bespoke protocol.)
- **buyer** — is the ROI claim real? (Demo 4 is a 10-second
  benchmark against an MCP baseline.)

## Tier 2 — full 9-step walkthrough

The unabridged walkthrough (same script pre-Phase-8-A, moved into
the suite but substantively unchanged):

| Demo                                          | Runtime | Recording                                                                                                                |
|-----------------------------------------------|--------:|--------------------------------------------------------------------------------------------------------------------------|
| [`demos/full.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/full.sh) | ~90s    | [docs/demo.typescript](../demo.typescript) · [docs/demo.transcript.txt](../demo.transcript.txt) · [walkthrough prose](../demo.md) |

`scripts/demo.sh` remains as a backwards-compat shim that execs
`scripts/demos/full.sh`, so existing users and docs that reference
`bash scripts/demo.sh` keep working.

## Tier 3 — sim vs real-backend parity

| Demo                                                                                                                  | Audience | Runtime | What it proves                                                                                                                                                                                                 | Recording                                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------- | -------- | ------: | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| [`parity.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/parity.sh)                                 | skeptic  |    ~30s | `reposix list` against the sim and `gh api` against `octocat/Hello-World` produce the same `{id, title, status}` JSON shape. Diff = content only.                                                              | [typescript](recordings/parity.typescript) · [transcript](recordings/parity.transcript.txt)                               |
| [`parity-confluence.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/parity-confluence.sh)           | skeptic  |    ~45s | Same claim for Confluence: `reposix list --backend sim` and `reposix list --backend confluence` produce the same `{id, title, status}` JSON shape. Key-set parity asserted via jq; content differs by design.  | —                                                                                                                         |

**Skip behavior** for `parity-confluence.sh`: requires
`ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`,
and `REPOSIX_CONFLUENCE_SPACE`. Exits 0 with a `SKIP:` banner if any
are unset, so CI runners and dev hosts without Atlassian credentials
still complete cleanly.

The library-level proof of the same claim is
[`crates/reposix-github/tests/contract.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-github/tests/contract.rs):
the five invariants in `assert_contract` hold for both `SimBackend`
(in every CI run) and `GithubReadOnlyBackend` (opt-in via `cargo test
-p reposix-github -- --ignored`).

## Tier 4 — adversarial swarm harness

| Demo                                                                                       | Audience         | Runtime | What it proves                                                                                                                                                                                 | Recording                                                                                 |
| ------------------------------------------------------------------------------------------ | ---------------- | ------: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| [`swarm.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/swarm.sh)       | developer, ops   |    ~40s | 50 concurrent simulated agents hammer the simulator for 30s (≈130k ops). Emits p50/p95/p99 per op type + an audit-row invariant check that proves SG-06 (append-only) holds under real load.   | [typescript](recordings/swarm.typescript) · [transcript](recordings/swarm.transcript.txt) |

**Not in smoke.** The swarm demo is deliberately excluded from
`scripts/demos/smoke.sh` (and therefore from the `demos-smoke` CI
job) because a 30s-per-run load test is too long for per-push CI. Run
it locally or as part of a release-gate pipeline. `SWARM_CLIENTS` and
`SWARM_DURATION` env vars tune it without editing the script.

The binary itself is in the new `reposix-swarm` crate; see
[`crates/reposix-swarm`](https://github.com/reubenjohn/reposix/tree/main/crates/reposix-swarm)
for the HDR-histogram bookkeeping, per-client agent-id plumbing, and
the `sim-direct` / `fuse` mode split. `fuse` mode performs real
`std::fs` syscalls against a pre-mounted FUSE tree — use it when you
want end-to-end kernel-path coverage under load, not just the sim's
HTTP surface.

## Tier 5 — FUSE mount real backend end-to-end

The Phase-10 wire-up generalized in Phase-11: the FUSE daemon speaks
the `reposix_core::IssueBackend` trait directly, so `reposix mount
--backend <backend> --project <target>` mounts a real remote tracker
as a POSIX directory. No simulator involved.

| Demo                                                                                                                        | Audience  | Runtime | What it proves                                                                                                                                                                                                                                           | Recording |
| --------------------------------------------------------------------------------------------------------------------------- | --------- | ------: | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| [`05-mount-real-github.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/05-mount-real-github.sh)           | developer |    ~30s | `reposix mount --backend github` exposes `octocat/Hello-World` issues under `issues/<padded-id>.md` (Phase-13 per-backend bucket); `cat issues/00000000001.md` renders the real issue's frontmatter+body. Honors `REPOSIX_ALLOWED_ORIGINS` and `GITHUB_TOKEN`.  | —         |
| [`06-mount-real-confluence.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/06-mount-real-confluence.sh)   | developer |    ~45s | `reposix mount --backend confluence` exposes an Atlassian Confluence space as a tree of Markdown files; `cat` on the first page renders the real frontmatter+body. Honors `REPOSIX_ALLOWED_ORIGINS` + the tenant-hostname allowlist entry.                | —         |

**Not in smoke.** `05-mount-real-github.sh` requires `gh auth token`
to be present and skips cleanly with `SKIP:` if not. Run it locally
after `gh auth login`:

```bash
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
bash scripts/demos/05-mount-real-github.sh
```

`06-mount-real-confluence.sh` requires `ATLASSIAN_API_KEY`,
`ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, and
`REPOSIX_CONFLUENCE_SPACE`. Exits 0 with a `SKIP:` banner if any are
unset. Token and email are never echoed — only the tenant host +
space key + allowlist appear on stdout. Run it locally with:

```bash
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
# Populate the four env vars from your Atlassian tenant, e.g. via
# `source .env` after copying `.env.example` and filling it in.
bash scripts/demos/06-mount-real-confluence.sh
```

## Running the suite yourself

```bash
# Build release binaries once.
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"

# One demo.
bash scripts/demos/01-edit-and-push.sh

# One demo, with marker-assertion enforcement.
bash scripts/demos/assert.sh scripts/demos/01-edit-and-push.sh

# Full Tier 1 smoke suite (what CI runs).
bash scripts/demos/smoke.sh
```

Each demo self-installs an EXIT trap that tears down its FUSE mount,
kills the simulator, and removes its tmp scratch directory. Re-runs
are idempotent. A stuck demo is bounded by an internal `timeout 90`
so the smoke suite stays within its CI budget.
