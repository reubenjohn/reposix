# v0.11.0 real-backend latency benchmark plan

## Current state

**Script structure** (`scripts/v0.9.0-latency.sh`, 185 lines):
1. `cargo build --workspace --bins` to ensure binaries are fresh.
2. Spawn `reposix-sim --ephemeral` on `127.0.0.1:7780` with the seeded
   fixture (`crates/reposix-sim/fixtures/seed.json`).
3. Time five steps via `date +%s%N` millisecond differences:
   - `reposix init sim::demo $REPO` cold (the six-step partial-clone
     bootstrap, including best-effort `git fetch --filter=blob:none`).
   - `curl GET /projects/demo/issues` (list).
   - `curl GET /projects/demo/issues/1` (get one).
   - `curl PATCH /projects/demo/issues/1` (write).
   - Helper `capabilities` probe via stdin pipe to `git-remote-reposix`.
4. Soft thresholds (`>500ms` warns to stderr, never blocks).
5. Heredoc-emit a fully-formatted Markdown table to
   `docs/benchmarks/v0.9.0-latency.md` with `${INIT_MS}` etc. substituted in.

**Sim columns populated via** the live ephemeral sim spawned in step 2;
the seed fixture is the only on-disk state. Run is `~15s` end-to-end.

**Real-backend columns empty because**:
- The script never branches on `GITHUB_TOKEN` / `ATLASSIAN_API_KEY` /
  `JIRA_API_TOKEN`; it only emits the sim cells, leaving the other three
  columns as raw whitespace in the heredoc.
- No CI job invokes `scripts/v0.9.0-latency.sh`. The three v09 contract
  jobs in `.github/workflows/ci.yml` (`integration-contract-{confluence,
  github,jira}-v09`, lines 128-192) run `cargo test ... agent_flow_real`
  to assert the dark-factory regression, not to capture latency.
- No record-count probing. The script reports milliseconds with no
  context — "list took 9 ms" is meaningless without "of how many issues".

The doc itself acknowledges the gap (line 36-39: "Phase 36 wires the
`integration-contract-{confluence,github,jira}-v09` CI jobs that populate
them"), but Phase 36 wired the regression jobs and skipped the bench.

## Required changes

### Script (`scripts/v0.9.0-latency.sh`)
1. **Detect creds** — branch on env vars to decide which real-backend
   columns to populate. Each backend block is independent and skips
   cleanly when its bundle is absent.
2. **Per-backend `init` timing** — call `reposix init github::reubenjohn/reposix`,
   `confluence::TokenWorld`, `jira::TEST` into separate scratch dirs.
3. **Record-count probing** — for each backend, hit the same REST endpoint
   the cache uses for `list_records` and count the returned array length:
   - GitHub: `GET https://api.github.com/repos/reubenjohn/reposix/issues?state=all&per_page=100`
     (matches `crates/reposix-github/src/lib.rs:357`); count via `jq 'length'`.
   - Confluence: two-step — `GET /wiki/api/v2/spaces?keys=TokenWorld` to
     resolve the space ID, then `GET /wiki/api/v2/spaces/{id}/pages?limit=250`
     (matches `crates/reposix-confluence/src/lib.rs:816`); count `.results | length`.
   - JIRA: `POST /rest/api/3/search/jql` with `{"jql":"project=TEST"}`
     (matches `crates/reposix-jira/src/lib.rs:247`); count `.issues | length`.
4. **Get/PATCH against a known record** — for each backend, capture the
   first issue ID from the list step and reuse it for the get and PATCH
   timings. PATCH is a no-op write (set the title to the value it already
   has) so the bench leaves no permanent mutation.
5. **Blob-materialization counter** — query the cache's SQLite audit
   table (`SELECT COUNT(*) FROM audit_events WHERE event='blob_materialize'`)
   immediately after init to report "init materialized N blobs lazily".
6. **Emit the augmented table** — change the heredoc to include the
   record counts (see "Concrete table format proposal" below).

Estimated script delta: +120 LOC (currently 185, target ~305). All in
one file, no new crate code needed.

### Table format
Add a "N records" column reporting the canonical count for that backend
(issues for github/sim/jira, pages for confluence). Fold blob-count into
the init row's footnote. Mark `n/a` for steps where the count concept
doesn't apply (capabilities probe).

### CI (`.github/workflows/ci.yml`)
Add a new job `bench-latency-v09` that depends on `[test]` and runs
the script with all three credential bundles wired from secrets. Output
strategy:
- **Primary**: upload the regenerated Markdown as a workflow artifact
  (`actions/upload-artifact@v4`), so reviewers can download it from any
  run without polluting `main`.
- **Secondary** (weekly cron only): on the scheduled run, open a PR via
  `peter-evans/create-pull-request` titled "chore(bench): refresh v0.9.0
  latency". This avoids commit-storms on every push but keeps the
  committed table fresh.

Avoid pushing directly to `main` from CI — the doc lives in `docs/` which
mkdocs publishes, so a bad row would ship to docs.

### Re-run cadence
- **On-demand**: workflow_dispatch trigger so the owner can refresh
  manually before a release.
- **Weekly cron** (`0 13 * * 1`, Monday 1pm UTC): catches API drift
  (e.g. GitHub deprecating an endpoint, Atlassian rotating quotas)
  without spamming on every PR.
- **Release tags**: add a step to `tag-v*.sh` scripts that checks the
  doc was regenerated within 7 days; warn (don't block) if stale.

## Concrete table format proposal

```markdown
| Step                          | sim    | github          | confluence       | jira             |
|-------------------------------|--------|-----------------|------------------|------------------|
| `reposix init` cold [^blob]   | 24 ms  | 1247 ms         | 1893 ms          | 1102 ms          |
| List records [^N]             | 9 ms   | 217 ms (N=47)   | 312 ms (N=14)    | 184 ms (N=8)     |
| Get one record                | 8 ms   | 142 ms          | 198 ms           | 156 ms           |
| PATCH record (no-op)          | 8 ms   | 198 ms          | 287 ms           | 211 ms           |
| Helper `capabilities` probe   | 5 ms   | 5 ms            | 5 ms             | 5 ms             |

[^blob]: init materializes blobs lazily — counts in init footnote per backend
         (sim=N, github=N, confluence=N, jira=N).
[^N]: `N` = records returned by the canonical list endpoint:
       sim/github/jira `issues`, confluence `pages` in the configured space.
```

Footnote line below the table:
> **N values reflect live backend state at run time** (commit `<sha>`,
> generated `<timestamp>`). The TokenWorld space and `reubenjohn/reposix`
> issue count drift over time; expect ±20% wobble between runs.

The "Helper `capabilities` probe" row is identical across columns
because it's local-only (no network); keep it as a control for runner
variance.

## CI integration

**Files to add/edit**:
- `.github/workflows/ci.yml`: add `bench-latency-v09` job (~50 LOC).
  Mirror the structure of `integration-contract-confluence-v09` for the
  secret-skip-clean pattern. Add `workflow_dispatch:` to the existing
  `on:` block if not already present, plus a `schedule:` entry.
- `scripts/v0.9.0-latency.sh`: see "Required changes" above.

**Secrets required** (all already in repo settings per the existing
v09 jobs):
- `GITHUB_TOKEN` (auto-injected; only needs `issues:read` for list/get
  and `issues:write` for the no-op PATCH — already scoped via the
  default Actions token).
- `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`.
- `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`.

**Permissions**: the bench is **strictly more privileged** than the
existing contract jobs because it now invokes a real `PATCH`. Recommended
mitigation: make the PATCH a self-no-op (read current title, PATCH it
back to itself) so even on success the backend state is unchanged. The
existing dark-factory tests already create-and-delete real issues, so
this is no regression.

`REPOSIX_ALLOWED_ORIGINS` per backend follows the established pattern
in the existing v09 jobs (lines 125, 165, 191).

## Estimated effort

- **LOC**: ~+120 in `v0.9.0-latency.sh`, ~+60 in `ci.yml`, 0 in Rust crates.
- **Files touched**: 2 (script + workflow). Doc regenerates from script.
- **Time**: ~3 hours for a focused executor — 1h script, 30min CI wiring,
  30min iterating on PR-create vs artifact-upload, 1h soaking the cron
  through one cycle to confirm stability.

**Risk areas**:
- **GitHub rate limits**: the bench hits `~3` requests per run; with the
  Actions-injected token's 5000/hr ceiling, a weekly cron is safe even
  with co-tenanted CI.
- **Confluence/JIRA rate limits**: undocumented per-tenant; a 100ms
  warm-up `GET` before timing avoids cold-TLS skew on the metric of
  interest.
- **JIRA TEST project state**: if the project gets archived or renamed,
  the bench fails — surface as a clear error, not a silent zero. Use
  `JIRA_TEST_PROJECT` env override consistently with the agent-flow tests.
- **Flakiness from network jitter**: take 3 samples per step and report
  the median; current single-shot timing is an inherent flake source
  even on sim. This is the highest-leverage script change.
- **TokenWorld page count drift**: don't bake an expected `N` into a CI
  assertion; the bench is informational, not a regression gate. Keep the
  soft-threshold WARN-only philosophy from the existing script.
