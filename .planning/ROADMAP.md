# Roadmap: reposix

## Overview

Seven-hour autonomous build of a git-backed FUSE filesystem that exposes a REST
issue tracker as a POSIX tree. Roadmap front-loads the simulator + shared core
types + security guardrails (the load-bearing Phase 1), then lets FUSE and the
CLI/audit layer build on that substrate in parallel, then wraps with a demo
that is required to show the guardrails firing. Write-path features
(`git-remote-reposix`, swarm harness, FUSE-in-CI) are STRETCH only — the hard
decision gate at T+3h (03:30 PDT) cuts them if the MVD isn't already green.

**Baseline already in tree (not re-planned):**
- 5-crate Cargo workspace (`reposix-core`, `-sim`, `-fuse`, `-remote`, `-cli`)
- `reposix-core` types: `Issue`, `IssueId`, `IssueStatus`, `ProjectSlug`,
  `Project`, `RemoteSpec`, `parse_remote_url`, `frontmatter::{render, parse,
  yaml_to_json_value}`, `Error` (7 unit tests passing)
- CI workflow at `.github/workflows/ci.yml` (fmt + clippy + test + integration
  + coverage jobs, green on paper)
- README, CLAUDE.md, LICENSE-MIT, first push to `reubenjohn/reposix`

**Granularity:** coarse (4 integer phases, 3 MVD + 1 stretch bucket)

**Requirement index:**

| Group | IDs | Source |
|-------|-----|--------|
| Functional core | FC-01 … FC-09 | PROJECT.md §Active → *Functional core* |
| Security guardrails | SG-01 … SG-08 | PROJECT.md §Active → *Security guardrails* |

Requirements come from PROJECT.md `### Active` directly (this project has no
separate REQUIREMENTS.md). Every `Active` bullet maps to exactly one phase
below.

## MVD vs STRETCH

- **MVD (Phases 1–3)** ≤4.5h wall clock with subagent parallelism. This is
  the credible minimum-viable demo the threat-model agent recommended:
  read-only mount + simulator GET + CLI + audit log + green CI + guardrails.
- **STRETCH (Phase S)** only runs if MVD lands ahead of schedule. The `git push`
  write-path, bulk-delete guard wired through a real push, swarm harness, and
  FUSE-in-CI all live here. At T+3h (03:30 PDT) the orchestrator MUST look at
  phase-1/2/3 status and commit to STRETCH or drop it.
- **Phase N (Demo, always runs)** is Phase 4. It demos whatever MVD shipped,
  and the guardrails it shipped. If STRETCH also shipped, Phase 4 extends the
  demo script to cover push + 409-merge-as-conflict and records those too.

## Parallelism map

Phase 1 is serial — it publishes the contracts the other phases link against.
After Phase 1 lands, Phase 2 (simulator + audit) and Phase 3 (FUSE + CLI) can
run in parallel on separate crates. Phase 4 (demo) is serial by nature — it
needs whatever shipped to demo against.

```
Phase 1 (serial, load-bearing)
   │
   ├──► Phase 2 (sim + audit)  ┐
   │                            ├──► Phase 4 (demo + README + recording)
   └──► Phase 3 (FUSE + CLI)   ┘
                    │
                    └──► [T+3h gate] ──► Phase S (STRETCH: push + swarm)
```

## Phases

- [ ] **Phase 1: Core contracts + security guardrails** — Shared types,
  tainted/untainted discipline, HTTP client factory with allowlist, filename
  rules, audit-log schema. [MVD, serial, ~1.0h]
- [ ] **Phase 2: Simulator + audit log** — axum sim serving GET/POST/PATCH/
  DELETE for issues, append-only SQLite audit table, seed data, rate-limit +
  409 + RBAC middleware. [MVD, parallel with Phase 3, ~1.5h]
- [ ] **Phase 3: Read-only FUSE mount + CLI orchestrator** — `reposix mount`
  lazy-fetches issues from the sim as `<id>.md` files; `reposix sim`,
  `reposix mount`, `reposix demo` subcommands. [MVD, parallel with Phase 2,
  ~1.5h]
- [ ] **Phase 4: Demo + recording + README polish** — End-to-end walkthrough
  script, asciinema/script(1) recording that visibly fires guardrails, README
  update with honest scope statement. [MVD, serial after 2+3, ~1.0h]
- [ ] **Phase S: STRETCH — write path + swarm + FUSE-in-CI** — Only executed
  if Phase 2+3 land before T+3h. Adds FUSE write ops, `git-remote-reposix`
  push with bulk-delete guard, adversarial swarm harness, FUSE mount inside
  `ubuntu-latest` CI job. [STRETCH, conditional, budget: whatever's left]

## Phase Details

### Phase 1: Core contracts + security guardrails
**Goal**: Every downstream crate imports a single, locked-down contract layer
that makes the easy path the safe path. `reposix_core::http::client()` is the
only legal way to build an HTTP client; `Tainted<T>` wraps every byte that
will come from the network; filenames are `<id>.md` and nothing else; the
audit-log schema with its append-only triggers exists as a `.sql` fixture.
**Depends on**: Nothing (first phase — baseline workspace already in tree)
**Requirements**: SG-01, SG-03, SG-04, SG-05, SG-06, SG-07 (types + schema
contracts; enforcement at use-sites lands in Phases 2–3), FC-02 (frontmatter
round-trip already in `reposix-core` — this phase adds the server-authoritative
field stripper)
**Success Criteria** (each is a Bash assertion runnable against the repo):
  1. `cargo test -p reposix-core --all-features` is green and includes tests
     named `egress_to_non_allowlisted_host_is_rejected`,
     `server_controlled_frontmatter_fields_are_stripped`,
     `filename_is_id_derived_not_title_derived`,
     `path_with_dotdot_or_nul_is_rejected`,
     `tainted_cannot_be_used_where_untainted_required` (the last via
     `trybuild` compile-fail).
  2. `grep -RIn 'reqwest::Client::new\|Client::builder' crates/ --include='*.rs'
     | grep -v 'crates/reposix-core/src/http.rs' | wc -l` prints `0` AND
     `cat clippy.toml` contains `reqwest::Client::new` in
     `disallowed-methods`.
  3. `cargo run -q -p reposix-core --example show_audit_schema` prints DDL
     that contains `CREATE TRIGGER audit_no_update BEFORE UPDATE` and
     `CREATE TRIGGER audit_no_delete BEFORE DELETE` on the `audit_events`
     table (the schema is a committed `.sql` fixture consumed by Phase 2).
  4. `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:* cargo test -p reposix-core
     allowlist_default_and_env -- --nocapture` passes and the same test with
     `REPOSIX_ALLOWED_ORIGINS=http://evil.example` rejects
     `http://127.0.0.1:7878` (proves env-var override is honored, default
     is safe).
  5. `cargo clippy -p reposix-core --all-targets -- -D warnings` is clean.
**Plans**: TBD

Plans:
- [ ] 01-01: `reposix-core::http::client()` factory + allowlist + redirects
  disabled + clippy `disallowed-methods` lint + tests for all four origin
  classes (loopback default-allow, non-loopback default-deny, env override,
  redirect refusal).
- [ ] 01-02: `Tainted<T>` / `Untainted<T>` newtype pair + `sanitize()` that
  strips `id`/`created_at`/`version`/`updated_at` from inbound frontmatter +
  `trybuild` compile-fail test that `Tainted<Url>` cannot be passed where
  `Untainted<Url>` is required + filename/path validator
  (`validate_issue_filename(&str) -> Result<IssueId, Error>`).
- [ ] 01-03: Audit-log DDL fixture (`crates/reposix-core/fixtures/audit.sql`)
  with append-only triggers + schema-loader helper + `examples/show_audit_schema.rs`
  + test that `pragma table_info(audit_events)` matches the fixture.

### Phase 2: Simulator + audit log
**Goal**: A standalone `reposix-sim` binary that serves a REST issue tracker
on `127.0.0.1:7878` with real HTTP semantics — list/get/create/update/delete
issues as JSON, rate-limit middleware, ETag-based 409 on stale updates,
append-only SQLite audit log wired through `reposix-core`'s schema fixture.
This is the backend Phase 3 talks to, and the first place the guardrails get
exercised end-to-end.
**Depends on**: Phase 1
**Requirements**: FC-01, FC-06, SG-06 (enforced at the sim's SQLite boundary)
**Success Criteria** (each is a Bash assertion):
  1. `cargo run -p reposix-sim -- --addr 127.0.0.1:7878 --seed-file
     crates/reposix-sim/fixtures/seed.json &` starts in <2s and
     `curl -sf http://127.0.0.1:7878/projects/demo/issues | jq 'length'`
     prints an integer ≥ 3.
  2. `curl -s -o /dev/null -w '%{http_code}'
     http://127.0.0.1:7878/projects/demo/issues/0001` prints `200`, and
     `curl -s http://127.0.0.1:7878/projects/demo/issues/0001 | jq
     '.frontmatter.id, .frontmatter.version'` returns the issue's id and a
     non-null version integer.
  3. `curl -s -X PATCH -H 'If-Match: "bogus"' -d '{"status":"done"}'
     http://127.0.0.1:7878/projects/demo/issues/0001` returns HTTP 409 (the
     staleness path the remote helper will one day turn into a git merge
     conflict).
  4. `sqlite3 runtime/sim-audit.db 'SELECT COUNT(*) FROM audit_events WHERE
     method IN ("GET","PATCH");'` returns ≥ 2 after the above curls, AND
     `sqlite3 runtime/sim-audit.db 'UPDATE audit_events SET path="x" WHERE
     id=1;'` fails with a trigger-raised error.
  5. `cargo test -p reposix-sim` is green and includes an integration test
     that boots the sim on an ephemeral port, issues a GET, and asserts the
     audit row was written.
**Plans**: TBD

Plans:
- [ ] 02-01: axum app with `/projects/:slug/issues` list+get+create+patch+
  delete handlers reading/writing SQLite through a `parking_lot::Mutex<Connection>`
  behind an `AppState`; committed `seed.json` fixture.
- [ ] 02-02: Audit middleware (`axum::middleware::from_fn`) that writes a row
  per request with the Phase-1 schema + ETag/`If-Match` → 409 path + a
  rate-limit tower layer using `governor`.

### Phase 3: Read-only FUSE mount + CLI orchestrator
**Goal**: `reposix mount /tmp/mnt --backend http://127.0.0.1:7878` presents a
directory containing `<id>.md` files for every issue in the simulator; `ls`,
`cat`, and `grep -r` work; the FUSE daemon never blocks the kernel longer than
5s even if the sim dies. The `reposix` CLI orchestrates all three subcommands
(`sim`, `mount`, `demo`). No write path yet — writes are the STRETCH gate.
**Depends on**: Phase 1 (can start in parallel with Phase 2 against a stubbed
backend; wiring happens late)
**Requirements**: FC-03 (read-path portion only for MVD; write path is Phase S),
FC-05, SG-04 (filename enforcement at FUSE boundary), SG-07
**Success Criteria** (each is a Bash assertion):
  1. `cargo run -p reposix-sim & sleep 1; cargo run -p reposix-cli -- mount
     /tmp/reposix-mnt --backend http://127.0.0.1:7878 & sleep 2;
     ls /tmp/reposix-mnt | sort` prints at least `0001.md 0002.md
     0003.md`.
  2. `cat /tmp/reposix-mnt/0001.md | head -1` prints `---` (frontmatter
     fence) and `grep -q '^id: 1$' /tmp/reposix-mnt/0001.md` exits 0.
  3. `grep -rIl database /tmp/reposix-mnt | wc -l` returns ≥ 1 (agent-style
     grep works end-to-end).
  4. Kill the sim, then `timeout 7 stat /tmp/reposix-mnt/0001.md; echo $?`
     returns in <7 s and exits non-zero (no kernel hang); `fusermount -u
     /tmp/reposix-mnt` then completes within 3 s.
  5. `cargo run -p reposix-cli -- --help` lists subcommands `sim`, `mount`,
     `demo`, and `cargo test -p reposix-fuse -p reposix-cli` is green
     (includes a `readdir` test against a mock backend and a
     simulator-death-does-not-hang test gated behind `--ignored`).
**Plans**: TBD
**UI hint**: no

Plans:
- [ ] 03-01: `reposix-fuse` — `Filesystem` impl with `init`, `getattr`,
  `lookup`, `readdir`, `read` (per research §3); inode registry
  (monotonic counter + `DashMap<IssueId, u64>`); Tokio-runtime async bridge
  owned by the FS struct (per research §5.2); `with_timeout(5s)` wrapper on
  every HTTP call; filename validator from Phase 1 at every path-bearing op.
- [ ] 03-02: `reposix-cli` — clap-derive CLI with `sim`, `mount`, `demo`
  subcommands; `demo` spawns sim + mounts + runs scripted `ls/cat/grep`
  sequence + tails the audit log; shared HTTP client built via
  `reposix_core::http::client()` only.

### Phase 4: Demo + recording + README polish
**Goal**: A third party can `git clone reubenjohn/reposix`, follow README,
and reproduce the demo in <5 minutes. The recorded asciinema (or `script(1)`
typescript) shows the central thesis working AND shows the guardrails firing
— read paths, an allowlist refusal, and a server-authoritative-field-strip
moment. README's scope section is honest about whatever shipped vs. what
deferred to v0.2.
**Depends on**: Phase 2 AND Phase 3 (plus Phase S if that shipped)
**Requirements**: FC-09, SG-08, and a pass-through verification that
FC-01/02/03/05/06/08 (the CI green-on-push requirement) all still hold end-to-end
**Success Criteria** (each is a Bash assertion):
  1. `test -f docs/demo.md && test -f docs/demo.cast` (or `docs/demo.typescript`)
     and `docs/demo.md` contains a `## Walkthrough` heading whose commands can
     be pasted literally.
  2. `bash scripts/demo.sh` (idempotent demo driver committed in this phase)
     exits 0 and leaves `runtime/demo-audit.db` populated; the same script is
     what the recording captures, so the recording is not hand-edited.
  3. `grep -E 'ALLOWED_ORIGINS|allowlist' docs/demo.md | wc -l` returns ≥ 1
     AND the recording (as rendered by `asciinema cat docs/demo.cast | grep
     -c EPERM`) contains ≥ 1 EPERM/refusal event — the "guardrails fired on
     camera" requirement.
  4. `grep -E '### Security|Threat model|v0\.2' README.md | wc -l` returns
     ≥ 2 — README has an honest scope/security section linking to
     `.planning/research/threat-model-and-critique.md` and naming what's
     deferred to v0.2.
  5. `gh run list --workflow ci.yml --limit 1 --json conclusion -q
     '.[0].conclusion'` prints `success` for the latest commit on `main`
     (green CI on the commit the demo was recorded against — FC-08).
**Plans**: TBD
**UI hint**: no

Plans:
- [ ] 04-01: `scripts/demo.sh` + `docs/demo.md` walkthrough + asciinema (or
  `script(1)`) recording of the script run against a fresh clone + a
  deliberate guardrail-triggering segment (attempt egress to
  `evil.example`, attempt a frontmatter `version: 999999` override). README
  scope/security section update citing threat-model.

### Phase S: STRETCH — write path + swarm + FUSE-in-CI (INSERTED, conditional)
**Goal**: If and only if Phases 2 + 3 are green before the T+3h (03:30 PDT)
decision gate, extend reposix from read-only to full round-trip: FUSE `write`,
`create`, `unlink`, `setattr`; `git-remote-reposix` helper that translates
pushes into API PATCH/POST/DELETE with the bulk-delete guard and
server-field stripper; adversarial swarm harness; and FUSE mounted inside the
`ubuntu-latest` CI job. The orchestrator commits to this phase at T+3h or
drops it — no sunk-cost grinding past 03:30.
**Depends on**: Phase 2 and Phase 3 both green; triggered by orchestrator
decision at T+3h
**Requirements**: FC-03 (write-path portion), FC-04, FC-07, FC-08 (the
"integration test that actually mounts FUSE in the runner" half of the CI
requirement; the `cargo test`/clippy/coverage half ships with MVD), SG-02
**Success Criteria** (each is a Bash assertion; phase is considered skipped
rather than failed if orchestrator picks the read-only fallback at T+3h):
  1. `echo "---\nstatus: done\n---\nbody" > /tmp/reposix-mnt/0001.md;
     cat /tmp/reposix-mnt/0001.md | grep '^status: done$'` succeeds AND
     `curl -s http://127.0.0.1:7878/projects/demo/issues/0001 | jq -r
     .frontmatter.status` returns `done` (write round-trips through the sim).
  2. `cd /tmp/reposix-mnt && git init && git add . && git commit -m init &&
     git remote add origin reposix::http://127.0.0.1:7878/projects/demo &&
     git push origin main` exits 0, and the audit DB shows one PATCH row per
     changed issue.
  3. A contrived commit that deletes 6 files then `git push` exits non-zero
     with stderr containing `--force-bulk-delete` (SG-02 fires on a real
     push, not just a unit test).
  4. `gh run view --log` for the latest `main` commit shows the
     `integration-fuse` job mounting, running `ls /tmp/mnt`, and unmounting,
     all green.
  5. `cargo run -p reposix-swarm -- --clients 50 --duration 30s` completes
     without a daemon panic; audit DB shows ≥ 100 rows; no row has a body
     field (SG-06 redaction holds under load).
**Plans**: TBD
**UI hint**: no

Plans:
- [ ] S-01: FUSE write path — `write`, `create`, `unlink`, `setattr` per
  research §3.7–3.10, with per-fh write buffer and timeout wrapper.
- [ ] S-02: `git-remote-reposix` helper — `capabilities`/`list`/`fetch`/
  `push` per research §2–§7, server-field stripper on push, bulk-delete
  guard.
- [ ] S-03: FUSE-in-CI job (`apt install fuse3` + `/dev/fuse` check + integration
  test) + `reposix-swarm` harness (tokio tasks hammering the mount).

## Coverage

| Req | Phase | Notes |
|-----|-------|-------|
| FC-01 Simulator-first architecture | Phase 2 | sim binary + seed + rate-limit + 409 |
| FC-02 Issues as Markdown + YAML | Phase 1 | render/parse already in core; server-field stripper lands here |
| FC-03 FUSE mount with read+write | Phase 3 (read) + Phase S (write) | read-path is MVD, write is STRETCH |
| FC-04 `git-remote-reposix` helper | Phase S | STRETCH; dropped if T+3h gate goes read-only |
| FC-05 Working CLI orchestrator | Phase 3 | `sim`, `mount`, `demo` subcommands |
| FC-06 Audit log (SQLite, queryable) | Phase 1 (schema) + Phase 2 (writes) | DDL in core, writes in sim middleware |
| FC-07 Adversarial swarm harness | Phase S | STRETCH |
| FC-08 Working CI on GitHub Actions | baseline + Phase 4 (green-on-demo-commit) + Phase S (FUSE mount in CI) | MVD owns fmt/clippy/test/coverage (already green); FUSE-in-CI is STRETCH |
| FC-09 Demo-ready by morning | Phase 4 | README + recording + walkthrough |
| SG-01 Outbound HTTP allowlist | Phase 1 | single `http::client()` factory + clippy lint |
| SG-02 Bulk-delete cap on push | Phase S | lives on the push path; no MVD surface for it |
| SG-03 Server-controlled frontmatter immutable | Phase 1 | `sanitize()` strips `id`/`created_at`/`version`/`updated_at` |
| SG-04 Filename = `<id>.md`; path validation | Phase 1 (validator) + Phase 3 (FUSE enforcement) | |
| SG-05 Tainted-content typing | Phase 1 | `Tainted<T>`/`Untainted<T>` + trybuild test |
| SG-06 Audit log append-only | Phase 1 (schema + triggers) + Phase 2 (enforcement test) | |
| SG-07 FUSE never blocks kernel forever | Phase 3 | `with_timeout(5s)` wrapper + EIO path |
| SG-08 Demo shows guardrails firing | Phase 4 | on-camera allowlist refusal + field-strip |

**Coverage:** 17/17 requirements mapped ✓  (no orphans, no duplicates —
shared requirements are split by sub-deliverable: schema/enforcement, or
read/write.)

## Decision gates

- **T+3h (03:30 PDT) — STRETCH commit gate.** If Phase 1 is not green and
  Phases 2+3 are not both at or near success-criteria, CUT Phase S entirely
  and reallocate the remaining budget to Phase 4 polish. No litigation of this
  decision at T+5h.
- **T+5h (05:30 PDT) — recording cutoff.** Start the asciinema/script(1)
  recording no later than 05:30 regardless of STRETCH status. Re-records eat
  ~30 min each; budgeting for exactly one re-record.
- **T+7h (07:30 PDT) — push and walk away.** Final commit pushed, CI green,
  README rendered on github.com verified via playwright screenshot (per user's
  global OP #1).

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 ∥ 3 → [T+3h gate] → (S?) → 4.
Phases 2 and 3 run in parallel. Phase S is conditional.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core contracts + security guardrails | 0/3 | Not started | - |
| 2. Simulator + audit log | 0/2 | Not started | - |
| 3. Read-only FUSE mount + CLI | 0/2 | Not started | - |
| 4. Demo + recording + README | 0/1 | Not started | - |
| S. STRETCH: write + swarm + FUSE-in-CI | 2/3 | Complete (swarm+CI deferred) | 2026-04-13 |
| 8. Demo suite + real-backend seam | 0/4 | Post-ship value add | - |

## Phase 8: Demo suite + real-backend seam (post-ship)
**Goal**: Split the demo monolith into a maintainable suite AND carve the IssueBackend seam so a real GitHub adapter can land without reshaping the FUSE/remote layers.
**Added**: 2026-04-13 09:05 PDT (post-v0.1.0-ship; user-requested value add)
**Deadline**: 12:15 PDT (~3h 10min)
**Depends on**: Phase 4 demo, Phase 7 robustness
**Requirements**: introduces maintainability + real-backend parity (v0.2 prep)

**Plans**:
- [ ] 08-A: Demo restructure (`scripts/demos/_lib.sh` + 4 Tier-1 one-liners + assert.sh + smoke.sh + full.sh rename + old demo.sh shim + docs/demos/index.md + CI smoke job).
- [ ] 08-B: `IssueBackend` trait in `reposix-core` + `SimBackend` impl + `reposix list` CLI subcommand.
- [ ] 08-C: `reposix-github` crate with `GithubReadOnlyBackend` + state-mapping ADR + `scripts/demos/parity.sh`.
- [ ] 08-D: Contract test suite parameterized over both backends + Tier-1 demo recordings + docs updates.

### Phase 11: Confluence Cloud read-only adapter (v0.3)
**Goal**: Ship a `reposix-confluence` crate implementing `IssueBackend` against Atlassian Confluence Cloud REST v2 (`https://<tenant>.atlassian.net/wiki/api/v2/`). Adopt **Option A** from HANDOFF §3 — flatten page hierarchy; encode `parent_id` + `space_key` in frontmatter — so existing FUSE + CLI machinery works unchanged. Basic auth (`email:ATLASSIAN_API_KEY`). CLI dispatch for `list --backend confluence` and `mount --backend confluence --project <SPACE_KEY>`. Wiremock unit tests ≥5. Contract test parameterized like GitHub's. Tier 3B parity demo + Tier 5 live-mount demo. ADR-002 for page→issue mapping. Docs update. Rename `TEAMWORK_GRAPH_API` → `ATLASSIAN_API_KEY` in `.env.example`. CHANGELOG + `v0.3.0` tag.

(Note: gsd-tools auto-allocated "Phase 9" above, but Phases 9-swarm and 10-FUSE-GitHub already shipped from the previous session as committed git history without formal ROADMAP.md entries. Skipping numerically to Phase 11 keeps the numbering honest.)

**Added**: 2026-04-13 ~20:55 PDT (overnight session 3)
**Depends on**: Phase 10 (FUSE mount via `IssueBackend` trait already shipped)
**Deadline**: 08:00 PDT 2026-04-14

**Success Criteria** (each is a Bash assertion):
  1. `cargo test --workspace --locked` returns ≥180 pass / 0 fail.
  2. `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
  3. `bash scripts/demos/smoke.sh` still 4/4 green — Tier 1 demos untouched.
  4. `reposix list --backend confluence --project <SPACE_KEY>` (with `ATLASSIAN_API_KEY` + `REPOSIX_ALLOWED_ORIGINS=...,https://<tenant>.atlassian.net` set) against real Atlassian prints ≥1 issue row.
  5. `reposix mount /tmp/reposix-conf-mnt --backend confluence --project <SPACE_KEY>` + `ls` + `cat *.md` returns real page frontmatter + body; `fusermount3 -u` succeeds; subsequent mount works again (re-entrant).
  6. `bash scripts/demos/06-mount-real-confluence.sh` exits 0 when `ATLASSIAN_API_KEY` is set; skips cleanly (exit 0) when unset.
  7. `docs/decisions/002-confluence-page-mapping.md` exists and documents the field mapping; `.env.example` reflects the renamed var.

**Plans**: TBD (run `/gsd-plan-phase 11` to break down)
