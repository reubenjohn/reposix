← [back to index](./index.md)

# Early Phases 1–S: Phase List + Phase Details

<details>
<summary>Shipped: Phases 1–15 (v0.1–v0.5)</summary>

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

### Phase 1: Core contracts + security guardrails (v0.1)
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

### Phase 2: Simulator + audit log (v0.1)
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

### Phase 3: Read-only FUSE mount + CLI orchestrator (v0.1)
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

### Phase 4: Demo + recording + README polish (v0.1)
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

### Phase S: STRETCH — write path + swarm + FUSE-in-CI (INSERTED, conditional) (v0.1)
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
