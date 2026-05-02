---
phase: 82
plan: 01
title: "DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 — Bus remote URL parser, prechecks, fetch dispatch"
wave: 1
depends_on: [81]
requirements: [DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01]
files_modified:
  - crates/reposix-remote/src/bus_url.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-remote/src/precheck.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/tests/bus_url.rs
  - crates/reposix-remote/tests/bus_capabilities.rs
  - crates/reposix-remote/tests/bus_precheck_a.rs
  - crates/reposix-remote/tests/bus_precheck_b.rs
  - crates/reposix-remote/tests/common.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/bus-url-parses-query-param-form.sh
  - quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh
  - quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh
  - quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh
  - quality/gates/agent-ux/bus-fetch-not-advertised.sh
  - quality/gates/agent-ux/bus-no-remote-configured-error.sh
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 82 Plan 01 — Bus remote: URL parser, prechecks, fetch dispatch (DVCS-BUS-URL-01 / DVCS-BUS-PRECHECK-01 / DVCS-BUS-PRECHECK-02 / DVCS-BUS-FETCH-01)

<objective>
Land the read/dispatch surface of the bus remote for v0.13.0's DVCS
topology — URL parser recognizes `reposix::<sot-spec>?mirror=<mirror-url>`
per Q3.3; bus PUSH-only per Q3.4 (no `stateless-connect` advertisement);
two cheap prechecks (mirror drift via `git ls-remote`; SoT drift via P81's
`list_changed_since` substrate) bail BEFORE reading stdin. The WRITE
fan-out (steps 4–9 of the bus algorithm) is explicitly DEFERRED to P83 —
P82 ends in a clean "P83 not yet shipped" error after prechecks pass
(Q-B in this plan).

This is a **single plan, six sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 6 rows in `quality/catalogs/agent-ux.json` +
  6 TINY verifier shells (status FAIL).
- **T02** — `bus_url.rs` parser module (new file) + 4 unit tests inline.
- **T03** — Coarser SoT-drift wrapper `precheck_sot_drift_any` appended
  to `precheck.rs` + 1 unit test.
- **T04** — `bus_handler.rs` module (new file) + `main.rs` Route
  dispatch + capabilities branching + State extension.
- **T05** — 4 integration tests under `crates/reposix-remote/tests/`
  (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b).
- **T06** — Catalog flip FAIL → PASS + CLAUDE.md update + per-phase
  push.

Sequential (T01 → T02 → T03 → T04 → T05 → T06). Per CLAUDE.md "Build
memory budget" the executor holds the cargo lock sequentially across
T03 → T05. T01 is doc-only (catalog rows + verifier shell scaffolding).

**Architecture (read BEFORE diving into tasks):**

The bus URL parser lives in a NEW SIBLING module `bus_url.rs` that
wraps the existing `backend_dispatch::parse_remote_url` (RESEARCH.md
Pattern 1). The single-backend `ParsedRemote` shape stays unchanged
— the new `Route { Single(ParsedRemote) | Bus { sot: ParsedRemote,
mirror_url: String } }` enum branches at `argv[2]` parse time INSIDE
`bus_url::parse`. Strip `?<query>` segment BEFORE delegating to
`backend_dispatch::parse_remote_url(base)` (RESEARCH.md Pitfall 2 —
the existing splitter rejects `?` in the project segment).

`State` (`crates/reposix-remote/src/main.rs:48`) gains ONE new field:
`mirror_url: Option<String>`. `Some(url)` = bus invocation; `None` =
single-backend (D-05). Capability branching reads
`state.mirror_url.is_none()` to gate `stateless-connect`. The
bus_handler dispatch reads `state.mirror_url.as_deref()` to extract
the URL for STEP 0 + PRECHECK A. NO new `BusState` type-state
explosion — single Option field, minimal blast radius.

`bus_handler::handle_bus_export` is a SIBLING of the existing
`handle_export` (NOT a wrapper). Order: capabilities → list →
`export` verb received → STEP 0 (resolve local mirror remote name
by URL match per Q-A) → PRECHECK A (mirror drift) → PRECHECK B
(SoT drift via `precheck::precheck_sot_drift_any`) → emit deferred-
shipped error per Q-B (P82 stops here; P83 takes over). NO stdin
read. NO `parse_export_stream` invocation.

The coarser SoT-drift wrapper `precheck::precheck_sot_drift_any`
(NEW in T03) is a 10-line sibling of P81's
`precheck_export_against_changed_set`. It returns
`SotDriftOutcome { Drifted { changed_count: usize } | Stable }`,
reusing `cache.read_last_fetched_at()` (P81) and
`backend.list_changed_since(project, since)`. First-push (no cursor)
returns `Stable` — same policy as P81's wrapper.

PRECHECK A invokes `Command::new("git").args(["ls-remote", "--",
mirror_url, "refs/heads/main"])` — the project's existing shell-out
idiom (D-06; `crates/reposix-cli/src/doctor.rs:446-944`). The `--`
separator + reject-`-`-prefix mitigations defang argument-injection
via mirror_url (T-82-01). Local SHA read via
`git rev-parse refs/remotes/<name>/main` (handles packed-refs
correctly).

STEP 0's name lookup: `git config --get-regexp '^remote\..+\.url$'`,
parse stdout into `(config_key, value)` pairs, filter where value
byte-equals `mirror_url` (with trailing-slash normalization), sort
matched names alphabetically, pick first + WARN if multiple per
Pitfall 4. Zero matches → emit Q3.5 hint and exit BEFORE PRECHECK A
(D-01 / Q-A).

P82 emits the deferred-shipped error after prechecks pass:
- stderr: `"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83"`
- stdout: `error refs/heads/main bus-write-not-yet-shipped`
- `state.push_failed = true`; return `Ok(())`.

The unknown-query-key rejection (D-03 / Q-C) lives inside
`bus_url::parse`: after `parse_query`, iterate keys; if any key !=
`"mirror"`, return `Err(...)` with verbatim message *"unknown query
parameter `<key>` in bus URL; only `mirror=` is supported"*. The
`+`-delimited rejection lives BEFORE the query-string split — if
`stripped.contains('+')` and the URL has no `?`, return error citing
the canonical form.

**Best-effort vs hard-error semantics:**

- **STEP 0 zero-match:** hard error (refuses to push; emits Q3.5 hint).
- **STEP 0 multi-match:** WARN to stderr, proceed with first
  alphabetical (D-01).
- **PRECHECK A `git ls-remote` failure:** hard error → reject path
  (`fail_push(diag, "ls-remote-failed", ...)`); the user sees the
  failure reason, not a silent succeed.
- **PRECHECK A drift:** reject path (`fail_push(diag, "fetch first",
  "your GH mirror has new commits...")`).
- **PRECHECK A empty mirror:** treat as `Stable` (no drift possible);
  P84 webhook sync handles first-push-to-empty-mirror via separate
  code path.
- **PRECHECK B failure (REST unreachable):** hard error → reject path
  (`fail_push(diag, "backend-unreachable", ...)`).
- **PRECHECK B `Drifted`:** reject path (`fail_push(diag, "fetch first",
  "<sot> has changes since your last fetch...")`); cite mirror-lag
  refs hint when `read_mirror_synced_at` is populated (P80).
- **PRECHECK B `Stable`:** proceed to deferred-shipped error stub
  (Q-B).

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo check -p reposix-remote`,
`cargo nextest run -p reposix-remote`) used instead of workspace-wide.

This plan terminates with `git push origin main` (per CLAUDE.md push
cadence) with pre-push GREEN. The catalog rows' initial FAIL status
is acceptable through T01–T05 because the rows are `pre-pr` cadence
(NOT `pre-push`); the runner re-grades to PASS during T06 BEFORE the
push commits.
</objective>

## Chapters

- **[Must-haves & spec](./must-haves.md)** — `<must_haves>` acceptance criteria for all 6 tasks + `<canonical_refs>` substrate links.
- **[Threat model](./threat-model.md)** — Trust boundaries, STRIDE threat register (T-82-01..T-82-05).
- **[T01 — Catalog-first](./T01-catalog-first.md)** — Mint 6 catalog rows + 6 TINY verifier shells (status FAIL).
- **[T02 — `bus_url.rs` parser](./T02-bus-url-parser.md)** — New parser module + Route enum + 4 unit tests.
- **[T03 — SoT-drift wrapper](./T03-sot-drift-wrapper.md)** — `precheck_sot_drift_any` + `SotDriftOutcome` + 1 unit test.
- **[T04 step 1 — State + main.rs wiring](./T04-step-1.md)** — State extension, URL dispatch, capabilities branching, export arm (4a–4d).
- **[T04 step 2 — `bus_handler.rs` module](./T04-step-2.md)** — Full `bus_handler.rs` source (4e) + HARD-BLOCKs + commit + verify + done.
- **[T05 step 1 — common + bus_url + bus_capabilities tests](./T05-step-1.md)** — Copy common.rs (5a-prime) + `bus_url.rs` (5a) + `bus_capabilities.rs` (5b).
- **[T05 step 2 — bus_precheck_a tests](./T05-step-2.md)** — `bus_precheck_a.rs` fixture + 4 tests (5c).
- **[T05 step 3 — bus_precheck_b tests + build + commit](./T05-step-3.md)** — `bus_precheck_b.rs` (5d) + build sweep (5e) + commit (5f) + verify + done.
- **[T06 — Catalog flip, CLAUDE.md, push + plan close](./T06-catalog-flip-push.md)** — FAIL→PASS flip + CLAUDE.md update + per-phase push + plan-internal close protocol.
