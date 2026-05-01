---
phase: 82
title: "Bus remote — URL parser, prechecks, fetch dispatch"
milestone: v0.13.0
requirements: [DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01]
depends_on: [81]
plans:
  - 82-01-PLAN.md  # DVCS-BUS-URL-01..02-PRECHECK + FETCH-01 (catalog → URL parser → coarser SoT precheck wrapper → bus_handler with PRECHECK A+B + capability branching → P83-not-yet-shipped exit + tests + close)
waves:
  1: [82-01]
---

# Phase 82 — Bus remote: URL parser, prechecks, fetch dispatch (overview)

This is the FOURTH DVCS-substantive phase of milestone v0.13.0 — the
read/dispatch surface of the bus remote. Per `decisions.md` Q3.3
(RATIFIED query-param URL form), Q3.4 (RATIFIED bus PUSH-only;
fetch via single-backend), Q3.5 (RATIFIED no-auto-mutate of git
config), and the architecture-sketch's bus algorithm steps 1–3:
recognize `reposix::<sot-spec>?mirror=<mirror-url>`, run two cheap
prechecks (mirror drift via `git ls-remote`; SoT drift via the P81
substrate's `list_changed_since`-driven check), and refuse the
`stateless-connect` capability so fetch falls through to the
single-backend code path. The WRITE fan-out (steps 4–9 of the bus
algorithm) is explicitly DEFERRED to P83 — P82 ends in a clean
"P83 not yet shipped" error after prechecks pass (Q-B in this plan).

**Single plan, six sequential tasks** per RESEARCH.md § "Plan
Splitting" — only TWO are cargo-heavy (T03 + T05); the other four
are doc/JSON/shell only.

- **T01 — Catalog-first.** Six rows mint BEFORE any Rust edits in
  `quality/catalogs/agent-ux.json` (the existing dimension home
  alongside `agent-ux/dark-factory-sim`,
  `agent-ux/reposix-attach-against-vanilla-clone`, `agent-ux/mirror-refs-*`,
  `agent-ux/sync-reconcile-subcommand` from P80/P81). Five rows track
  the four phase requirements + the no-mirror-configured success
  criterion (SC5). The sixth row covers the new shape `?mirror=` URL
  parses correctly. Six TINY shell verifiers under
  `quality/gates/agent-ux/`. Initial status `FAIL`. Hand-edited per
  documented gap (NOT Principle A) — same shape as P81's
  `agent-ux/sync-reconcile-subcommand` row, per GOOD-TO-HAVES-01.

- **T02 — `bus_url.rs` parser module + unit tests.** New file
  `crates/reposix-remote/src/bus_url.rs` with `pub(crate) enum Route
  { Single(ParsedRemote), Bus { sot: ParsedRemote, mirror_url: String }
  }` and `pub(crate) fn parse(url: &str) -> Result<Route>`. Strips
  the optional `?<query>` segment BEFORE delegating to
  `backend_dispatch::parse_remote_url(base)` (RESEARCH.md Pitfall 2);
  rejects `+`-delimited form with the verbatim hint citing
  `?mirror=`; rejects unknown query keys per Q-C; allows mirror URLs
  with embedded `?` only when percent-encoded (RESEARCH.md Pitfall 7).
  4 unit tests inline + 1 RFC-fuzz-style negative test for the `+`
  rejection.

- **T03 — Coarser SoT-drift precheck wrapper + unit test.** Append a
  10-line `pub(crate) fn precheck_sot_drift_any(...)` to
  `crates/reposix-remote/src/precheck.rs` returning
  `SotDriftOutcome { Drifted { changed_count: usize } | Stable }`.
  Reuses `cache.read_last_fetched_at()` (P81), calls
  `backend.list_changed_since(project, since)`, returns `Stable` on
  empty or no-cursor (first-push policy mirrors
  `precheck_export_against_changed_set`'s no-cursor path). Adds 1 unit
  test inside `precheck.rs`'s existing `#[cfg(test)] mod tests` block.

- **T04 — `bus_handler.rs` + `main.rs` dispatch wiring +
  capabilities branching.** New file `crates/reposix-remote/src/bus_handler.rs`
  with: `STEP 0` (resolve local mirror remote name by URL match per
  Q-A — `git config --get-regexp '^remote\..+\.url$'`; multi-match
  alphabetical-first + WARN per RESEARCH.md Pitfall 4); PRECHECK A
  (mirror drift via `git ls-remote -- <mirror_url> refs/heads/main`
  shell-out, with `--` defang per RESEARCH.md § Security; compare
  against `git rev-parse refs/remotes/<name>/main`); PRECHECK B
  (calls `precheck::precheck_sot_drift_any` from T03); on success
  emit the verbatim "P83 not yet shipped" error per Q-B and exit
  cleanly. `main.rs::real_main` widens to dispatch on `bus_url::parse`'s
  `Route` enum: `Single` continues to `parse_dispatch_url` +
  `instantiate` + existing `handle_export`; `Bus` builds the SoT
  backend via the same `instantiate` then routes to
  `bus_handler::handle_bus_export`. Capabilities branching (5-line
  edit at lines 150-172): `if matches!(route, Route::Single(_))
  { proto.send_line("stateless-connect")?; }`. Bus URL never
  advertises `stateless-connect` (DVCS-BUS-FETCH-01 closure).

- **T05 — Integration tests.** Four new test files under
  `crates/reposix-remote/tests/` — `bus_url.rs` (parser positive +
  rejection golden URL fixtures), `bus_capabilities.rs` (asserts
  capability list omits `stateless-connect` for bus URL), `bus_precheck_a.rs`
  (file:// bare-repo fixture; drifted local mirror ref triggers
  `error refs/heads/main fetch first`), `bus_precheck_b.rs`
  (wiremock-backed sim with seeded `last_fetched_at` cursor;
  `list_changed_since` returns non-empty → `error refs/heads/main
  fetch first`). Each test file closes ONE catalog row.

- **T06 — Catalog flip + CLAUDE.md update + per-phase push.** Run
  `python3 quality/runners/run.py --cadence pre-pr` to flip the 6 rows
  FAIL → PASS. CLAUDE.md update lands in the same commit (one
  paragraph in § Architecture documenting the bus URL form
  `reposix::<sot>?mirror=<mirror>` + Q3.4 PUSH-only contract; one
  bullet in § Commands showing the new push form `git push reposix
  main`). `git push origin main` with pre-push GREEN. The
  orchestrator then dispatches the verifier subagent.

Sequential — never parallel. T01 → T02 → T03 → T04 → T05 → T06.
Even though T02 (bus_url) and T03 (precheck wrapper) touch different
files in the same crate, sequencing per CLAUDE.md "Build memory
budget" rule (one cargo invocation at a time) makes this strictly
sequential.

## Wave plan

Strictly sequential — one plan, six tasks. T01 → T02 → T03 → T04 →
T05 → T06 within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 82-01  | YES (T03+T05)    | none with prior phase | catalog + URL parser + SoT-drift wrapper + bus_handler + capabilities branching + 4 integration tests + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit; line numbers cited at planning time and require re-confirmation
during T04 read_first):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 82-01 | `crates/reposix-remote/src/bus_url.rs` (new), `crates/reposix-remote/src/bus_handler.rs` (new), `crates/reposix-remote/src/precheck.rs` (append `precheck_sot_drift_any` + `SotDriftOutcome`), `crates/reposix-remote/src/main.rs` (mod declarations + URL-route dispatch + capabilities branching), `crates/reposix-remote/tests/bus_url.rs` (new), `crates/reposix-remote/tests/bus_capabilities.rs` (new), `crates/reposix-remote/tests/bus_precheck_a.rs` (new), `crates/reposix-remote/tests/bus_precheck_b.rs` (new), `crates/reposix-remote/tests/common.rs` (new — P81 M3 gap copy from `crates/reposix-cache/tests/common/mod.rs`), `quality/catalogs/agent-ux.json` (6 new rows), `quality/gates/agent-ux/bus-url-parses-query-param-form.sh` (new), `quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh` (new), `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh` (new), `quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh` (new), `quality/gates/agent-ux/bus-fetch-not-advertised.sh` (new), `quality/gates/agent-ux/bus-no-remote-configured-error.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T03 → T05. T01 + T02 + T04 (source-file edits)
need only one `cargo check -p reposix-remote` between them. T06
runs the catalog runner (no cargo) + a final test sweep (`cargo
nextest run -p reposix-remote`). No parallel cargo invocations.

## Plan summary table

| Plan  | Goal                                                                                                          | Tasks | Cargo? | Catalog rows minted | Tests added                                                                                                           | Files modified (count) |
|-------|---------------------------------------------------------------------------------------------------------------|-------|--------|----------------------|-----------------------------------------------------------------------------------------------------------------------|------------------------|
| 82-01 | Bus URL parser + STEP 0 + PRECHECK A + PRECHECK B + capability branching + bus-write deferred-error stub      | 6     | YES (T03+T05) | 6 (status FAIL → PASS at T06) | 4 unit (parse positive, reject `+`, reject unknown key, no-cursor stable) + 1 unit precheck wrapper + 4 integration (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b) = 9 total | ~16 (2 new modules + 4 new test files + 6 new verifier shells + 1 catalog edit + main.rs + precheck.rs + CLAUDE.md) |

Total: 6 tasks across 1 plan. Wave plan: sequential.

Test count: 4 unit `bus_url::parse` + 1 unit `precheck_sot_drift_any`
(in `precheck.rs` `#[cfg(test)] mod tests`) + 4 integration tests
(`tests/bus_url.rs::*`, `tests/bus_capabilities.rs::*`,
`tests/bus_precheck_a.rs::*`, `tests/bus_precheck_b.rs::*`) = 9 total.

## Decisions ratified at plan time

The three open questions surfaced by RESEARCH.md § "Open Questions"
are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Each decision
references the source artifact and the rationale.

### D-01 — Q-A: by-URL-match for the no-remote-configured check (RATIFIED)

**Decision:** the bus handler resolves the local mirror remote NAME
by **scanning all configured remotes' URLs** and matching against
`mirror_url`. NOT by requiring a NAME in the bus URL. NOT by
auto-mutating git config.

**Implementation:** STEP 0 of `bus_handler::handle_bus_export` runs
`git config --get-regexp '^remote\..+\.url$'`, splits each line on
whitespace into `(config_key, value)`, reverse-looks-up the remotes
whose value byte-equals `mirror_url`. If zero matches → emit the
verbatim Q3.5 hint (`"configure the mirror remote first: git remote
add <name> <mirror-url>"`) and exit non-zero BEFORE PRECHECK A. If
one match → use that remote's name. If multiple matches → pick the
**first alphabetically**, emit a stderr WARNING naming the chosen
remote per RESEARCH.md Pitfall 4, proceed.

**Why URL-match (not require-name-in-URL):** the user has already
NAMED the remote (via `git remote add`). Making them also encode
the name into the bus URL is friction — a single `mirror=` carries
the same information, and the URL is the canonical UX. The `+`-form
that initially encoded `<sot>+<mirror_name>+<mirror_url>` was
explicitly dropped per Q3.3.

**Why first-alphabetical-with-WARN (not error) on multi-match:**
multi-match is a legitimate (rare) case — a user with a fork +
upstream both pointing at the same URL. Erroring would be
unfriendly; picking deterministically + naming the choice in stderr
gives the user a way to disambiguate (rename or remove the duplicate
remote).

**Source:** RESEARCH.md § "Open Questions" Q-A; Pitfall 4;
`.planning/research/v0.13.0-dvcs/decisions.md` Q3.5 (no-auto-mutate
ratified).

### D-02 — Q-B: P82 emits clean "P83 not yet shipped" error after prechecks pass (RATIFIED)

**Decision:** after PRECHECK A and PRECHECK B both succeed, the bus
handler does NOT proceed to read stdin. It emits a CLEAN diagnostic
to stderr (`"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet
shipped — lands in P83"`) AND a protocol error to stdout (`"error
refs/heads/main bus-write-not-yet-shipped"`), sets
`state.push_failed = true`, and returns cleanly. NO stdin read. NO
SoT writes. NO mirror writes.

**Why this shape (and not: silently fall through to `handle_export`):**
P82 is dispatch-only by ROADMAP definition. Falling through to
`handle_export` would route the bus URL through the single-backend
write path (because `handle_export` has no concept of a mirror —
SoT-only), corrupting the "SoT-first, mirror-best-effort" contract
P83 will land. Emitting a clear deferred-shipped error preserves
the "PRECHECK-pass means BOTH prechecks succeeded" contract for
P82's tests AND signals to a user who tries `git push reposix main`
in v0.13.0-dev that the write path is forthcoming.

**Why not: silently emit `ok refs/heads/main`:** that would lie to
git, which would conclude the push succeeded — confusing the user
when their mirror + SoT both remain unchanged.

**P83's planner inherits a clean seam:** the `bus_handler.rs` body's
deferred-error stub is a single `return Ok(())` site. P83 replaces
it with the SoT-write + mirror-write fan-out + audit + ref-update
logic. P83's test surface re-uses P82's PRECHECK A + PRECHECK B
fixtures (file:// mirror, wiremock SoT) without modification — only
the post-precheck behavior changes.

**Source:** RESEARCH.md § "Open Questions" Q-B; ROADMAP P82 SC1
("Bus URL parser is dispatch-only; WRITE fan-out is P83").

### D-03 — Q-C: reject unknown query parameters (RATIFIED)

**Decision:** `bus_url::parse` rejects bus URLs containing query
keys other than `mirror=`. Only `mirror=` is recognized. Unknown
keys produce an error: *"unknown query parameter `<key>` in bus
URL; only `mirror=` is supported"*.

**Why reject (not silently ignore):** silent-ignore is a footgun. A
typo `?mirorr=git@github.com:org/repo.git` becomes a no-op (the
parser sees no `mirror=`, falls through to `Route::Single`, and the
push hits the single-backend path with NO mirror integration) —
silently violating the user's intent. Rejection forces typos to
surface immediately.

**Forward compatibility:** rejecting today is cheaper than reclaiming
namespace later. If v0.14.0 adds `?priority=`, `?retry=`, etc., the
parser opts those keys in explicitly. No legacy-key compatibility
debt accrues.

**Empty-query-string boundary case:** `reposix::sim::demo?` (no
key=value pairs) produces an error citing missing `mirror=`.
`reposix::sim::demo` (no `?` at all) is `Route::Single` and
unaffected.

**Mirror URL with embedded `?`** (RESEARCH.md Pitfall 7): the mirror
URL must be percent-encoded in that case. The `url` crate's
`form_urlencoded::parse` handles the encoding cleanly. Document
the requirement in `bus_url.rs`'s module-doc and in CLAUDE.md
§ Architecture.

**Source:** RESEARCH.md § "Open Questions" Q-C; Pitfall 7;
`.planning/research/v0.13.0-dvcs/decisions.md` Q3.3 (`?mirror=` as
sole syntax).

### D-04 — `agent-ux.json` is the catalog home (NOT a new `bus-remote.json`)

**Decision:** add the 6 new rows to the existing
`quality/catalogs/agent-ux.json` (joining `agent-ux/dark-factory-sim`,
`agent-ux/reposix-attach-against-vanilla-clone`,
`agent-ux/mirror-refs-write-on-success`, `agent-ux/mirror-refs-read-by-vanilla-clone`,
`agent-ux/mirror-refs-reject-message-cites-refs`,
`agent-ux/sync-reconcile-subcommand`). NOT a new `bus-remote.json`.

**Why:** dimension catalogs are routed to `quality/gates/<dim>/`
runner discovery — `agent-ux` is the existing dimension. Splitting
it into two catalog files would force the runner to discover both
via tag, adding indirection for no benefit. The existing
`agent-ux.json` already has 6 rows; adding 6 more keeps the
single-file shape readable. P81's `agent-ux/sync-reconcile-subcommand`
is the immediate-prior precedent.

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P80 and P81 catalog-home precedents.

### D-05 — Bus path uses the SAME `BackendConnector` pipeline as the single-backend path for the SoT side

**Decision:** when `Route::Bus { sot, mirror_url }` is dispatched,
the SoT side (the `sot: ParsedRemote`) is consumed by the existing
`backend_dispatch::instantiate(&sot)` to produce the same
`Arc<dyn BackendConnector>` the single-backend path uses. The
`mirror_url` is held alongside in a NEW `BusState { sot_state:
State, mirror_url: String }` shape (or, more minimally, two extra
fields on `State`; see implementation note below).

**Why one BackendConnector pipeline:** Q3.4 RATIFIED bus is
PUSH-only; bus's SoT reads happen during PRECHECK B (calls
`backend.list_changed_since`), and in P83 SoT writes happen via
`backend.create_record / update_record / delete_record_or_close`.
All three paths go through `BackendConnector` — there's no separate
"bus backend" trait. The mirror is plain git (shell-out to `git
ls-remote` / `git push`); it's NOT a `BackendConnector`.

**Implementation note (T04):** the simplest shape is to extend
`State` with `Option<String> mirror_url`. `Some(url)` means "this
is a bus invocation"; `None` means single-backend. Capability
branching reads `state.mirror_url.is_none()` to decide whether to
advertise `stateless-connect`. The bus_handler dispatch reads
`state.mirror_url.as_deref()` to extract the URL for STEP 0 +
PRECHECK A. This avoids a `BusState` type-state explosion with
minimal blast radius (one `Option` field).

**Source:** RESEARCH.md § "Architecture Patterns" Pattern 1
(`Route::Bus { sot, mirror_url }` carries the SoT as a
`ParsedRemote`); decisions.md Q3.4 RATIFIED PUSH-only.

### D-06 — `git ls-remote` shell-out (NOT gix-native) for PRECHECK A

**Decision:** PRECHECK A invokes `Command::new("git").args(["ls-remote",
"--", mirror_url, "refs/heads/main"])` via `std::process::Command`.
NOT gix's native `Repository::find_remote(...).connect(direction)`.

**Why shell-out:** the project's existing idiom is shell-out for
porcelain calls (`crates/reposix-cli/src/doctor.rs:446` for `git
--version`; lines 859/909/935/944 for `git rev-parse / for-each-ref
/ rev-list`). gix-native ls-remote requires ~50 lines of
refspec/connection-state management for what shell-out does in 5
lines. The helper runs in a context where `git push` already
invoked `git`, so `git` is on PATH (Assumption A1).

**Security: `--` separator + reject `-`-prefixed mirror URLs.**
RESEARCH.md § Security flagged argument-injection via `mirror_url`
(e.g., `--upload-pack=evil`). Mitigation: BEFORE the shell-out,
reject any `mirror_url` that starts with `-`; ALWAYS pass `--` as
a positional separator before the URL. The reject error: *"mirror
URL cannot start with `-`: <mirror_url>"*. Documented in
`bus_handler.rs`'s module-doc.

**Local SHA read:** `git rev-parse refs/remotes/<name>/main`
(another shell-out, same idiom). Handles packed-refs correctly;
raw fs reads of `.git/refs/remotes/<name>/main` would miss them.

**Source:** RESEARCH.md § "Don't Hand-Roll", Pattern 3, Pitfall 3;
`crates/reposix-cli/src/doctor.rs` (donor pattern).

## Subtle architectural points (read before T04)

The two below are flagged because they are the most likely sources
of T04 review friction. Executor must internalize them before
writing the wiring code.

### S1 — Capability branching is a 5-line edit, not a refactor

The current `"capabilities"` arm in
`crates/reposix-remote/src/main.rs:150-172` emits five capability
lines unconditionally. The bus URL flips ONE of them off
(`stateless-connect`); the other four (`import`, `export`, `refspec`,
`object-format=sha1`) stay. Per Q3.4 ratified bus is PUSH-only —
fetch on a bus URL falls through to the single-backend path,
so the bus URL never advertises `stateless-connect`.

**Why this matters for T04.** A reviewer skimming the wiring may
expect the bus path to need its own capabilities arm, or a separate
`fn handle_bus_capabilities`. That would be wrong. Capabilities
are advertised once per helper invocation BEFORE any verb dispatch
— the difference between bus and single-backend is one `if`. The
existing arm becomes:

```rust
"capabilities" => {
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    if state.mirror_url.is_none() {
        proto.send_line("stateless-connect")?;
    }
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

DVCS-BUS-FETCH-01 is closed by this 5-line diff. The integration
test (T05's `tests/bus_capabilities.rs`) sends `capabilities\n` to
the helper with a bus URL on argv[2], reads stdout, asserts the
list contains `import`, `export`, `refspec`, `object-format=sha1`
AND does NOT contain `stateless-connect`.

### S2 — Stdin must NOT be read before either precheck fires

The whole point of the cheap-precheck design is that PRECHECK A and
PRECHECK B run BEFORE `parse_export_stream` consumes stdin. If
stdin is read first, the precheck cost-savings claim collapses
(the user has paid to upload their fast-import stream over the
pipe). For typical issue-tracker push sizes (a few KB) this is
irrelevant; for larger artifacts (image attachments, etc.) it
matters loudly.

**Why this matters for T04.** The natural place to insert prechecks
is "right where the existing one runs", but the existing
`handle_export` precheck (P81's `precheck_export_against_changed_set`)
runs AFTER `parse_export_stream`. The bus path is a SIBLING of
`handle_export`, not a wrapper — `bus_handler::handle_bus_export`
runs prechecks FIRST, then (in P82) emits the deferred-shipped
error WITHOUT reading stdin, then (in P83) buffers stdin and
proceeds.

**Test contract:** T05's `bus_precheck_a.rs` and `bus_precheck_b.rs`
each assert a fixture-state property: NO `helper_push_started` audit
row OR NO `parse_export_stream` invocation observable. The cleanest
assertion is "the bus_handler returns BEFORE `BufReader::new(ProtoReader::new(proto))`
constructs" — verifiable via test-side instrumentation OR by
observing that wiremock saw zero PATCH/PUT calls AND zero
`Cache::log_helper_push_started` rows. Use the wiremock + cache-
audit shape (lower coupling than test-side instrumentation).

## Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P82) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints SIX rows + SIX verifier shells
   BEFORE T02–T06 implementation. Initial status `FAIL`. Rows are
   hand-edited per documented gap (NOT Principle A) — annotated in
   commit message referencing GOOD-TO-HAVES-01. The agent-ux
   dimension's bind verb is not yet implemented; rows ship as
   hand-edits matching P81's precedent.
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-remote`,
   `cargo nextest run -p reposix-remote`. Pre-push hook runs the
   workspace-wide gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 → T05 → T06
   — never parallel, even though T02 (`bus_url.rs`) and T03 (precheck
   wrapper) touch different files. CLAUDE.md "Build memory budget"
   rule is "one cargo invocation at a time" — sequencing the tasks
   naturally honors this.
4. **`bus_url.rs` is a SIBLING module of `parse_remote_url`.** The
   single-backend `ParsedRemote` shape stays unchanged (per RESEARCH.md
   Pattern 1). New `Route { Single(ParsedRemote) | Bus(...) }` enum
   branches at `argv[2]` parse time INSIDE `bus_url::parse`. The
   `backend_dispatch::parse_remote_url` function is called by
   `bus_url::parse` AFTER stripping the optional query string —
   the existing parser sees only single-backend-shaped URLs.
5. **PRECHECK B uses a COARSER wrapper.** New `precheck_sot_drift_any(cache,
   backend, project, rt) -> SotDriftOutcome` returning `Drifted | Stable`.
   P81's `precheck_export_against_changed_set` is preserved verbatim
   for P83's write-time intersect-with-push-set check. Both functions
   live in `precheck.rs`; the coarser one is ~10 lines (D-01 in
   RESEARCH.md § Pattern 2).
6. **Capabilities branching is a 5-line edit at `main.rs:150-172`** (S1
   above). `if state.mirror_url.is_none() { proto.send_line("stateless-connect")?; }`
   gates DVCS-BUS-FETCH-01. The other four capability lines stay
   unchanged.
7. **PRECHECK A shells out to `git ls-remote`.** D-06 RATIFIED. Use
   `--` separator to defang argument-injection via mirror_url. Reject
   mirror URLs starting with `-` BEFORE the shell-out.
8. **No-remote lookup: by-URL-match (D-01 / Q-A).** Match against all
   configured remotes' URLs (`git config --get-regexp '^remote\..+\.url$'`),
   with multi-match alphabetical-first + WARN. Zero-match → emit
   verbatim Q3.5 hint and exit before PRECHECK A.
9. **Bus path emits clean "P83 not yet shipped" error after prechecks
   pass (D-02 / Q-B).** P82 is dispatch-only; no write fan-out. The
   error: stderr "bus write fan-out (DVCS-BUS-WRITE-01..06) is not
   yet shipped — lands in P83"; stdout `error refs/heads/main bus-write-not-yet-shipped`.
10. **Reject unknown query params (D-03 / Q-C).** Only `mirror=` is
    recognized. Typo-protection + forward compat. `bus_url::parse`
    rejects with: *"unknown query parameter `<key>` in bus URL; only
    `mirror=` is supported"*.
11. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
    codified 2026-04-30).** T06 ends with `git push origin main`;
    pre-push gate must pass; verifier subagent grades the six catalog
    rows AFTER push lands. Verifier dispatch is an orchestrator-level
    action AFTER this plan completes — NOT a plan task.
12. **CLAUDE.md update in same PR (QG-07).** T06 documents the bus URL
    scheme `reposix::<sot>?mirror=<mirror>` (§ Architecture) + the new
    push form `git push reposix main` (§ Commands). The mirror-URL
    percent-encoding requirement (RESEARCH.md Pitfall 7) is named.
13. **No new error variants.** The remote crate uses `anyhow` throughout
    (per `crates/reposix-remote/src/main.rs:18`). Bus_url + bus_handler
    + precheck wrapper all return `anyhow::Result<...>`. Reject-path
    stderr strings are passed via `.context("bus-precheck: ...")`
    annotations and emitted by the existing `fail_push(diag, ...)` shape.
14. **State extension is one Option field (D-05 / S1).** `State` gains
    `mirror_url: Option<String>` field. `Some(url)` = bus invocation;
    `None` = single-backend. Capability branching + bus-handler dispatch
    both read this single field. NO new `BusState` type.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces ONE new
trifecta surface (the `git ls-remote` shell-out's argument boundary)
and reuses three existing surfaces unchanged:

| Existing surface              | What P82 changes                                                                                                                                                                                                                                                                |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — PRECHECK B's `list_changed_since` call is the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist used since v0.9.0. No new HTTP construction site introduced. |
| Cache prior-blob parse (Tainted bytes) | UNCHANGED — P82 does not touch the cache prior parse. The bus path uses the coarser `precheck_sot_drift_any` (P82's NEW wrapper) which only reads the `last_fetched_at` cursor and counts changed records — does NOT parse blobs. |
| `Tainted<T>` propagation      | UNCHANGED — no new tainted-byte sources or sinks in P82. |
| **Shell-out boundary (NEW)**  | NEW: `git ls-remote -- <mirror_url> refs/heads/main`. The `mirror_url` is user-controlled (from the bus URL in argv[2]); the threat is argument-injection. Mitigation: (a) reject mirror URLs starting with `-` BEFORE the shell-out; (b) pass `--` separator unconditionally before the URL argument; (c) `mirror_url` is byte-passed (no template expansion). STRIDE category: Tampering — mitigated by D-06 + the rejection at parse time. |
| **`git config --get-regexp` shell-out (NEW)** | NEW: STEP 0 invokes `git config --get-regexp '^remote\..+\.url$'`. The regex is helper-controlled (no user input); the helper parses stdout via whitespace-split + byte-equal comparison. STRIDE category: Tampering — mitigated by helper-controlled regex (no string concatenation with user input) + read-only config invocation. |
| **`git rev-parse` shell-out (NEW)** | NEW: PRECHECK A reads the local mirror SHA via `git rev-parse refs/remotes/<name>/main`. The `<name>` is bounded by the result of STEP 0's `git config` lookup (so it matches `^remote\..+\.url$` — the regex itself is helper-controlled). STRIDE category: Tampering — mitigated by the bounded source of `<name>`. |

`<threat_model>` STRIDE register addendum below the per-task threat
register in the plan body:

- **T-82-01 (Tampering — argument injection via `mirror_url` shell-out):**
  reject `-`-prefix + `--` separator before the URL.
- **T-82-02 (Information Disclosure — Tainted SoT bytes leaking via
  bus_handler logs):** UNCHANGED from P81 — `precheck_sot_drift_any`
  only counts records, never logs body bytes; deferred-error stub
  emits no tainted bytes.
- **T-82-03 (Denial of Service — `git ls-remote` against private
  mirrors hangs on SSH-agent prompt):** documented in CLAUDE.md
  `git ls-remote` requires SSH agent set up; tests use `file://`
  fixture exclusively per RESEARCH.md Pitfall 3.

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin main`
   in T06 (per CLAUDE.md "Push cadence — per-phase", codified
   2026-04-30, closes backlog 999.4). Pre-push gate-passing is part of
   the plan's close criterion.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 82-01 pushes (i.e., after
   T06 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the six P82
   catalog rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p82/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P81 SHIPPED ... next P82" → "P82 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated in T06.** T06's CLAUDE.md edit lands in the
   terminal commit (one § Architecture paragraph + one § Commands
   bullet per QG-07).
7. **REQUIREMENTS.md DVCS-BUS-URL-01 / DVCS-BUS-PRECHECK-01 / DVCS-BUS-PRECHECK-02 /
   DVCS-BUS-FETCH-01 checkboxes flipped.** Orchestrator (top-level)
   flips `[ ]` → `[x]` after verifier GREEN. NOT a plan task.

## Risks + mitigations

| Risk                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                |
|-------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`url` crate's `form_urlencoded::parse` doesn't preserve raw `mirror=` value with embedded `:` and `@`** (RESEARCH.md Assumption A3) | MEDIUM     | T02's unit tests include a parser case for `?mirror=git@github.com:org/repo.git` (verbatim, NOT percent-encoded). If `form_urlencoded::parse` mangles the value, fall back to manual `split_once('=')` after `split_once('?')` per RESEARCH.md fallback path. T02's done criteria includes the round-trip assertion. |
| **`git ls-remote -- <mirror_url>` against `file://` fixture hangs on macOS** (RESEARCH.md Pitfall 3 sibling) | LOW        | T05's `bus_precheck_a.rs` uses `tempfile::tempdir()` + `git init --bare` + `file://` URL. Linux + macOS both handle file:// without blocking. If a CI environment has unusual git config (e.g., `protocol.file.allow=user`), the test sets `GIT_CONFIG_NOSYSTEM=1` + `GIT_CONFIG_GLOBAL=/dev/null` per CLAUDE.md test isolation precedent. |
| **`git config --get-regexp` returns multiple matches** (RESEARCH.md Pitfall 4)                       | MEDIUM     | D-01 ratified: pick first alphabetically + emit stderr WARNING naming the chosen remote. T05's `bus_precheck_a.rs` includes a multi-match fixture asserting the WARNING appears + the chosen remote is the alphabetically-first. |
| **Mirror SHA from `git ls-remote` is empty** (empty mirror; first push) | LOW        | P82 treats empty `git ls-remote` output as `MirrorDriftOutcome::Stable` (no drift possible). P84 (webhook sync) handles the first-push-to-empty-mirror case via a separate code path. T05's `bus_precheck_a.rs` includes an empty-mirror fixture asserting `Stable`. |
| **PRECHECK B firing on the same-record self-edit case** (RESEARCH.md Pitfall 5) | LOW        | P81's RATIFIED `>`-strict semantics on `list_changed_since(since)` mean a same-second self-write is filtered cleanly. T05's `bus_precheck_b.rs` includes a self-edit case asserting `Stable`. |
| **`reposix::<sot>?mirror=<mirror>` URL with query in the mirror value** (RESEARCH.md Pitfall 7) | LOW-MED    | Document the percent-encoding requirement in `bus_url.rs` module-doc + CLAUDE.md § Architecture (D-03). T02 has a test case asserting the percent-encoded form parses correctly. The non-encoded form errors with a clear message per Q-C (extra `?` introduces an unknown key). |
| **Capability branching breaks existing single-backend tests** (S1; D-05)                              | LOW        | The capability arm change is a SINGLE-LINE addition wrapping the existing `proto.send_line("stateless-connect")?;`. Single-backend invocations have `state.mirror_url.is_none() == true`, so the `stateless-connect` line still fires. T05's `bus_capabilities.rs` covers the bus case; the existing `crates/reposix-remote/tests/stateless_connect.rs` covers single-backend. |
| **`State` extension breaks compilation of `handle_export` or `handle_stateless_connect`**             | LOW        | Adding ONE `Option<String>` field with a default `None` initializer in `real_main` is purely additive. `handle_export` doesn't read `state.mirror_url`; `handle_stateless_connect` doesn't read it. T04 confirms via `cargo check -p reposix-remote` after the State edit lands. |
| **Cargo memory pressure** (load-bearing CLAUDE.md rule)                                              | LOW        | Strict serial cargo across all six tasks. Per-crate (`cargo check -p reposix-remote`, `cargo nextest run -p reposix-remote`) only. T01 + T02 + T04 + T06 doc-or-source-edit cargo-checks; T03 + T05 are the cargo-test-bearing tasks (sequential). |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P82**                                    | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`. |
| **No-remote-configured detection: false-negative when remote URL has trailing slash variant** (RESEARCH.md A5) | LOW        | T04 normalizes `mirror_url` (strip trailing `/`) BEFORE the byte-equal compare against config values. T05's `bus_no_remote_configured` test includes a fixture where one remote URL has trailing `/` and the bus URL doesn't — assertion: still matches. |

## +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P82 surfaces
candidates only when they materialize during execution — none pre-filed
at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — `url::form_urlencoded::parse` requires the input be a URL
  (full scheme + host); the bus URL form starts with `reposix::` which
  is NOT a real URL scheme. Eager-resolve in T02 by parsing manually
  via `split_once('?')` + `split('&')` + `split_once('=')` (the
  resulting parser is ~20 lines and avoids the URL crate's strictness).
  RESEARCH.md flagged this as Assumption A3 (MEDIUM risk).
- **LOW** — `git config --get-regexp` outputs values that contain
  whitespace (rare — URLs don't usually have whitespace). Eager-resolve
  in T04 by using `splitn(2, char::is_whitespace)` for the
  `(key, value)` split, NOT the simpler `split_whitespace`.
- **LOW** — A new `bus_handler.rs` test file's wiremock fixture races
  with the `bus_precheck_b.rs`'s wiremock fixture if both run in the
  same nextest process (port conflicts). Eager-resolve via wiremock's
  per-test `MockServer::start()` returning a unique port — same idiom
  as `crates/reposix-remote/tests/perf_l1.rs` (P81 precedent). NOT a
  candidate unless ports clash in CI.
- **LOW-MED** — `state.cache.as_ref()` returns `None` during
  PRECHECK B because `ensure_cache` hasn't fired yet (the bus_handler
  runs BEFORE `handle_export`'s `ensure_cache` line). Eager-resolve
  in T04 by calling `ensure_cache(state)?` at the top of
  `handle_bus_export` (best-effort like in `handle_export`); if cache
  unavailable, PRECHECK B's `precheck_sot_drift_any(None, ...)` returns
  `Stable` (matching the no-cursor first-push policy). NOT a candidate.

Items NOT in scope for P82 (deferred per the v0.13.0 ROADMAP):

- Bus write fan-out (P83). The `bus_handler.rs` body's deferred-error
  stub is a single `return Ok(())` site that P83 replaces with the
  SoT-write + mirror-write logic.
- 30s TTL cache for the `git ls-remote` precheck (Q3.2 DEFERRED).
  Measure first; add only if push latency is hot. Filed as v0.13.0
  GOOD-TO-HAVE candidate.
- Webhook-driven mirror sync (P84). Out of scope.
- DVCS docs (P85). Out of scope; T06 only updates CLAUDE.md.
- Real-backend tests (TokenWorld + reubenjohn/reposix issues). Out of
  scope per OP-1 — milestone-close gates them, not phase-close.
- Multi-SoT bus URL form (`reposix::sot1+sot2?mirror=...`). Out of
  scope per Q3.3 (1+1 bus only in v0.13.0).

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                                  |
|------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 82-01 T01 (6 catalog rows + 6 verifier shells)                  | `gsd-executor` — catalog-first commit; **hand-edits agent-ux.json per documented gap (NOT Principle A)**.                                                                                                                  |
| 82-01 T02 (`bus_url.rs` parser + 4 unit tests)                  | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T03 (`precheck_sot_drift_any` wrapper + 1 unit test)      | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T04 (`bus_handler.rs` + main.rs dispatch + capabilities)  | Same 82-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                            |
| 82-01 T05 (4 integration tests: bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b) | Same 82-01 executor (cargo-heavy task). Cargo lock for `reposix-remote` integration test run. Per-crate cargo only.                                                                                                            |
| 82-01 T06 (catalog flip + CLAUDE.md + push)                      | Same 82-01 executor (terminal task).                                                                                                                                                                                        |
| Phase verifier (P82 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 82-01 T06 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the six catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P82):

- **DVCS-BUS-URL-01:** `crates/reposix-remote/src/bus_url.rs` exists;
  `pub(crate) enum Route { Single(ParsedRemote), Bus { sot: ParsedRemote,
  mirror_url: String } }`; `pub(crate) fn parse(url) -> Result<Route>`
  parses `reposix::sim::demo?mirror=file:///tmp/m.git` to `Route::Bus`;
  rejects `+`-delimited form with verbatim "use `?mirror=` instead"
  hint; rejects unknown query keys per Q-C; unit tests
  (`cargo test -p reposix-remote --test bus_url`) pass.
- **DVCS-BUS-PRECHECK-01:** `bus_handler::handle_bus_export` runs
  PRECHECK A via `git ls-remote -- <mirror_url> refs/heads/main`;
  on drift emits `error refs/heads/main fetch first` to stdout +
  hint to stderr (*"your GH mirror has new commits..."*); bails
  BEFORE PRECHECK B; integration test
  (`cargo test -p reposix-remote --test bus_precheck_a`) passes;
  `--` separator + `-`-prefix reject ARE in code (grep-able).
- **DVCS-BUS-PRECHECK-02:** `bus_handler::handle_bus_export` runs
  PRECHECK B via `precheck::precheck_sot_drift_any(...)`; on `Drifted`
  emits `error refs/heads/main fetch first` + hint citing
  `refs/mirrors/<sot>-synced-at` (when populated, via
  `read_mirror_synced_at`); bails BEFORE stdin read; integration
  test (`cargo test -p reposix-remote --test bus_precheck_b`) passes.
- **DVCS-BUS-FETCH-01:** `crates/reposix-remote/src/main.rs:150-172`
  capabilities arm gates `proto.send_line("stateless-connect")?;`
  on `state.mirror_url.is_none()`; integration test
  (`cargo test -p reposix-remote --test bus_capabilities`) asserts
  bus URL omits `stateless-connect` from capability list.
- **No-remote-configured (SC5 + 6th catalog row):** `bus_handler` STEP
  0's URL-match lookup fires BEFORE PRECHECK A; zero matches → emit
  verbatim Q3.5 hint; integration test asserts the hint string
  appears in stderr verbatim.
- New catalog rows in `quality/catalogs/agent-ux.json` (6); each
  verifier exits 0; status PASS after T06.
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02–T06 implementation); per-phase push completed;
  verdict file at `quality/reports/verdicts/p82/VERDICT.md`; CLAUDE.md
  updated in T06.

## Verification approach (developer-facing)

After T06 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/bus-url-parses-query-param-form.sh
bash quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh
bash quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh
bash quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh
bash quality/gates/agent-ux/bus-fetch-not-advertised.sh
bash quality/gates/agent-ux/bus-no-remote-configured-error.sh
python3 quality/runners/run.py --cadence pre-pr  # re-grade catalog rows
cargo nextest run -p reposix-remote --test bus_url           # parser unit tests
cargo nextest run -p reposix-remote --test bus_capabilities  # capability list omits stateless-connect for bus URL
cargo nextest run -p reposix-remote --test bus_precheck_a    # mirror drift fixture
cargo nextest run -p reposix-remote --test bus_precheck_b    # SoT drift wiremock fixture
cargo nextest run -p reposix-remote                          # full crate test sweep
```

The fixtures for PRECHECK A use **two local bare repos**
(`tempfile::tempdir()` + `git init --bare` + `file://` URL) per
RESEARCH.md § "Test Fixture Strategy". Same approach as
`scripts/dark-factory-test.sh`. The PRECHECK B fixture uses
**wiremock** mirroring P81's `tests/perf_l1.rs` setup pattern.

This is a **subtle point worth flagging**: success criteria 2-3 (the
prechecks) are satisfied by two contracts simultaneously: (a) the
helper exits non-zero AND emits the expected stdout/stderr lines, AND
(b) the helper makes ZERO REST writes to wiremock AND ZERO stdin
reads. The integration tests assert BOTH.
