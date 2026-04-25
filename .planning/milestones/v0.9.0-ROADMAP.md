# Roadmap: reposix — v0.9.0 Architecture Pivot (archived)

**Status:** SHIPPED 2026-04-24. Tag (`v0.9.0`) push gated on
`bash scripts/tag-v0.9.0.sh` (owner runs).

**Milestone goal:** Replace the FUSE virtual filesystem with git's built-in
partial clone mechanism. The `git-remote-reposix` helper becomes a promisor
remote tunnelling protocol-v2 traffic to a local bare-repo cache built from
REST responses. Agents interact with the project using only standard git
commands (`clone`, `fetch`, `cat`, `grep`, `commit`, `push`) — zero
reposix-specific CLI awareness required. FUSE is deleted entirely;
`crates/reposix-fuse/` is removed and the `fuser` dependency is purged.

**Source of truth:**
`.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`
(canonical design doc, 440 lines, ratified 2026-04-24).

**Audit:** `.planning/v0.9.0-MILESTONE-AUDIT.md` — `status: tech_debt` (one
carry-forward: helper hardcodes SimBackend; three real-backend CI jobs
`pending-secrets` per intentional owner gate).

---

## v0.9.0 Architecture Pivot — Git-Native Partial Clone

**Motivation:** The FUSE-based design is fundamentally slow (every `cat`/`ls`
triggers a live REST API call) and doesn't scale (10k Confluence pages = 10k
API calls on directory listing). FUSE also has operational pain:
fusermount3, /dev/fuse permissions, WSL2 quirks, pkg-config/libfuse-dev
build dependencies. Research confirmed that git's built-in partial clone +
the existing `git-remote-reposix` helper can replace FUSE entirely, giving
agents a standard git workflow with zero custom CLI awareness required.

**Research:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`
(canonical design document), `partial-clone-remote-helper-findings.md`
(transport layer POC), `push-path-stateless-connect-findings.md` (write
path POC), `sync-conflict-design.md` (sync model). POC code in `poc/`
subdir.

**Key design decisions:**
- DELETE `crates/reposix-fuse` entirely; drop `fuser` dependency
- ADD `stateless-connect` capability to `git-remote-reposix` for partial-clone reads
- KEEP `export` capability for push (hybrid confirmed working in POC)
- ADD `reposix-cache` crate: backing bare-repo cache built from REST responses
- Agent UX is pure git: `git clone`, `cat`, `git push` — zero reposix CLI awareness
- Push-time conflict detection: helper checks backend state at push time, rejects with standard git error
- Blob limit guardrail: helper refuses to serve >N blobs, error message teaches agent to use sparse-checkout
- Tree sync always full (cheap metadata); blob materialization is the only limited/lazy operation
- Delta sync via `since` queries (all backends support this natively)

**Phases (31–36) — all SHIPPED 2026-04-24:**

### Phase 31: `reposix-cache` crate — backing bare-repo cache from REST responses

**Goal:** Land the foundation crate that materializes REST API responses into
a real on-disk bare git repo. The cache is the substrate every later phase
builds on.

**Requirements:** ARCH-01, ARCH-02, ARCH-03

**Verification:** `passed` (7/7) — `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-VERIFICATION.md`

**Plans:** 3/3 SHIPPED
- [x] 31-01-PLAN.md — Wave 1: reposix-cache crate scaffold + gix 0.82 smoke + Cache::build_from with lazy tree (ARCH-01)
- [x] 31-02-PLAN.md — Wave 2: cache_schema.sql + audit/db/meta modules + Cache::read_blob (Tainted + egress-denial audit) + lift cache_db.rs from reposix-cli (ARCH-02, ARCH-03)
- [x] 31-03-PLAN.md — Wave 3: trybuild compile-fail fixtures — Tainted→Untainted + Untainted::new pub(crate) locks (ARCH-02)

**Commits (14 atomic commits):** `ee48a46..0fa960c`

### Phase 32: `stateless-connect` capability in `git-remote-reposix` (read path)

**Goal:** Port the Python POC's `stateless-connect` handler to Rust inside
`crates/reposix-remote/`. Tunnel protocol-v2 traffic to the Phase 31 cache
so `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` works
end-to-end with lazy blob loading. The existing `export` capability for
push must keep working in the same binary (hybrid).

**Requirements:** ARCH-04, ARCH-05

**Depends on:** Phase 31

**Verification:** `passed_with_gated_deferrals` (6/6 — 3 gated behind
`integration-git` feature for the alpine-git-2.52 CI runner; 3 fully
automated on the dev host) — `.planning/phases/32-stateless-connect-capability-in-git-remote-reposix-read-path/32-VERIFICATION.md`

**Tech debt carry-forward:** Helper backend dispatch hardcodes SimBackend
(documented in 32-SUMMARY.md §"Backend name hardcoded" + Phase 35 research
note). Resolution scheduled before v0.11.0 benchmark commits.

**Commits (5 on main):** `860de02..ca0c575`

### Phase 33: Delta sync — `list_changed_since` on `BackendConnector` + cache integration

**Goal:** Add incremental backend queries so `git fetch` after a backend
mutation transfers only the changed issue's tree+blob, not the whole
project.

**Requirements:** ARCH-06, ARCH-07

**Depends on:** Phase 31, Phase 32

**Verification:** `passed` (6/6) — `.planning/phases/33-delta-sync-list-changed-since-backendconnector-cache-integratio/33-VERIFICATION.md`

**Plans:** 2/2 SHIPPED
- [x] 33-01: trait method + 4 backend impls (sim `?since=`, GitHub `?since=`, Jira JQL `updated >=`, Confluence CQL `lastModified >`)
- [x] 33-02: Cache::sync atomic delta materialization + helper integration

**Commits (10 + 2 docs):** `5512124..c5e00ca`

### Phase 34: Push path — conflict detection + blob limit guardrail

**Goal:** Make the `export` handler conflict-aware and the
`stateless-connect` handler scope-bounded.

**Requirements:** ARCH-08, ARCH-09, ARCH-10

**Depends on:** Phase 32

**Verification:** `passed` (8/8) — `.planning/phases/34-push-path-conflict-detection-blob-limit-guardrail/34-VERIFICATION.md`

**Plans:** 2/2 SHIPPED
- [x] 34-01: blob-limit guardrail + helper_push audit ops + cache audit extension
- [x] 34-02: push-time conflict detection + frontmatter sanitize + integration tests

**Commits:** `2b79978..33a7527`

### Phase 35: CLI pivot — `reposix init` replacing `reposix mount` + agent UX validation

**Goal:** Replace the `reposix mount` command with `reposix init <backend>::<project> <path>`.
Run the dark-factory acceptance test. Capture latency for each step of the
golden path.

**Requirements:** ARCH-11, ARCH-12, ARCH-16, ARCH-17 (capture)

**Depends on:** Phase 31, Phase 32, Phase 33, Phase 34

**Verification:** `human_needed` — sim-backed gates green; real-backend
exercise `pending-secrets` for all three OP-6 targets (TokenWorld,
`reubenjohn/reposix`, JIRA TEST). Test infrastructure complete; CI secret
packs are owner gate. — `.planning/phases/35-cli-pivot-reposix-init-agent-ux-real-backend-validation/35-VERIFICATION.md`

**Plans:** 4/4 SHIPPED (recovered from VM-crash)
- [x] 35-01: `reposix init` subcommand + `mount` migration stub + CHANGELOG/README docs
- [x] 35-02: dark-factory regression test (script + integration tests)
- [x] 35-03: real-backend integration tests with skip_if_no_env! gating
- [x] 35-04: latency benchmark + testing-targets doc + CHANGELOG cross-links

**Commits:** `f32ed3c..262bedb`

### Phase 36: FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + final integration tests + release

**Goal:** Demolish FUSE entirely and ship v0.9.0. Per OP-4 self-improving
infrastructure: this phase updates project CLAUDE.md and adds the
`reposix-agent-flow` skill — agent grounding must ship in lockstep with
code.

**Requirements:** ARCH-13, ARCH-14, ARCH-15, ARCH-17 (artifact), ARCH-18, ARCH-19

**Depends on:** Phase 35

**Verification:** `passed` (15/15) — every acceptance gate from
36-CONTEXT's explicit DELETE / UPDATE / CREATE lists is verifiable in the
committed tree on `main`. Real-backend CI gates
(`integration-contract-{confluence,github,jira}-v09`) remain
`pending-secrets` until secret packs decrypt. — `.planning/phases/36-fuse-deletion-claudemd-update-reposix-agent-flow-skill-release/36-VERIFICATION.md`

**Plans:** 2/2 SHIPPED
- [x] 36-01: delete reposix-fuse crate + FUSE infrastructure
- [x] 36-02: rewrite CLAUDE.md, ship reposix-agent-flow skill, finalize v0.9.0 release artifacts

**Commits:** `1535cb0..058c297`

---

## v0.9.0 commit ledger

- 60 commits total since `f17119c docs(planning): scaffold v0.9.0
  architecture-pivot phases 31-36`
- Net workspace test delta: ~+49 across the milestone (clean
  `cargo test --workspace` at Phase 36 close, zero failures, 49
  `test result: ok` lines)
- New crate: `crates/reposix-cache/`
- Deleted crate: `crates/reposix-fuse/` (12 files, commit `1535cb0`)
- Workspace size: 9 members (was 10)
- Workspace version: 0.8.0 → 0.9.0
- New documents: `docs/benchmarks/v0.9.0-latency.md`,
  `docs/reference/testing-targets.md`,
  `.claude/skills/reposix-agent-flow/SKILL.md`,
  `scripts/dark-factory-test.sh`, `scripts/v0.9.0-latency.sh`,
  `scripts/tag-v0.9.0.sh`

## Audit & UAT

- **Milestone audit:** `.planning/v0.9.0-MILESTONE-AUDIT.md` —
  `status: tech_debt` (helper hardcodes SimBackend → tracked for v0.11.0
  prereq; real-backend CI jobs `pending-secrets`).
- **UAT:** sim path proven via dark-factory shell-script harness +
  `agent_flow.rs` integration test. Real-backend UAT gated on owner
  decrypting secret packs in CI; once decrypted, the same test suite
  exercises live tenants.

## Tag gate

`bash scripts/tag-v0.9.0.sh` — 8 numbered safety guards (clean tree, on
`main`, version match in `Cargo.toml`, CHANGELOG `[v0.9.0]` exists, tests
green, signed tag, plus a v0.9-specific guard for
`docs/reference/testing-targets.md` existence per ARCH-18). Owner runs.
