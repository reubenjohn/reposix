← [back to index](./index.md) · phase 82 plan 01

# Must-haves & Canonical References

<must_haves>
**`bus_url.rs` module (T02)** — `crates/reposix-remote/src/bus_url.rs`
(new file, ~120 lines):
- Module doc-comment cites Q3.3 RATIFIED query-param form, the
  `+`-rejection rule, and the unknown-query-key rejection (D-03).
- `pub(crate) enum Route { Single(ParsedRemote), Bus { sot: ParsedRemote, mirror_url: String } }`.
- `pub(crate) fn parse(url: &str) -> anyhow::Result<Route>` per
  RESEARCH.md Pattern 1 algorithm:
  1. Strip optional `reposix::` prefix (re-uses
     `reposix_core::remote::strip_reposix_prefix` if available; if not,
     local strip — confirm during T02 read_first).
  2. If stripped contains `+` AND no `?`: return error citing
     `?mirror=` form (the rejection from Q3.3).
  3. Split on first `?`: `(base, query?)`.
  4. Call `backend_dispatch::parse_remote_url(base)` to produce
     `ParsedRemote`. (CRITICAL: pass the prefix-stripped form back to
     the existing parser — it accepts both `reposix::...` and bare
     forms per `crates/reposix-remote/src/backend_dispatch.rs:92`).
  5. If no query: return `Route::Single(parsed)`.
  6. Parse query into `Vec<(String, String)>` via manual
     `split('&')` + `split_once('=')` (the `url::form_urlencoded`
     fallback from RESEARCH.md A3 — manual is safer because the
     `reposix::` scheme is not a real URL).
  7. Look up `mirror=` value; if absent → error
     *"bus URL query string present but `mirror=` parameter missing;
     expected `reposix::<sot-spec>?mirror=<mirror-url>`"*.
  8. Iterate other keys; reject any key != `"mirror"` per D-03 / Q-C.
  9. Return `Route::Bus { sot: parsed, mirror_url }`.
- 4 unit tests inline (`#[cfg(test)] mod tests`):
  `parses_query_param_form_round_trip`, `rejects_plus_delimited_bus_url`,
  `rejects_unknown_query_param`, `route_single_for_bare_reposix_url`.
- `# Errors` doc on `parse`; `cargo clippy -p reposix-remote --
  -D warnings` clean.

**`precheck.rs` extension (T03)** — append to
`crates/reposix-remote/src/precheck.rs` (existing file, 302 lines
post-P81):
- New `pub(crate) enum SotDriftOutcome { Drifted { changed_count: usize }, Stable }`.
- New `pub(crate) fn precheck_sot_drift_any(cache: Option<&Cache>,
  backend: &dyn BackendConnector, project: &str, rt: &Runtime) ->
  anyhow::Result<SotDriftOutcome>` per RESEARCH.md Pattern 2:
  ```rust
  let Some(since) = cache.and_then(|c| c.read_last_fetched_at().ok().flatten()) else {
      return Ok(SotDriftOutcome::Stable);  // first-push policy
  };
  let changed = rt.block_on(backend.list_changed_since(project, since))
      .context("backend-unreachable: list_changed_since (PRECHECK B)")?;
  if changed.is_empty() {
      Ok(SotDriftOutcome::Stable)
  } else {
      Ok(SotDriftOutcome::Drifted { changed_count: changed.len() })
  }
  ```
- The function is ≤ 15 lines including docs. Module-doc gets a new
  paragraph naming this wrapper alongside
  `precheck_export_against_changed_set` and explaining the
  bus-vs-single-backend asymmetry: bus handler runs PRECHECK B
  BEFORE reading stdin (push set unknown), so the coarser wrapper
  is sufficient; single-backend `handle_export` runs the finer
  intersect-with-push-set check AFTER `parse_export_stream`. P83's
  bus_handler will call BOTH (the coarser one before stdin, the
  finer one after).
- 1 unit test in the existing `#[cfg(test)] mod tests` block:
  `precheck_sot_drift_any_returns_stable_when_no_cursor`. (The
  `Drifted`/`Stable`-with-cursor cases are exercised end-to-end in
  T05's `bus_precheck_b.rs` integration test against wiremock —
  inline-unit-testing them would require duplicating that fixture.)
- `# Errors` doc on `precheck_sot_drift_any`.
- L1-strict delete trade-off comment is NOT repeated in the new
  function (it's already in the module-doc; the wrapper inherits the
  same trust contract).

**`bus_handler.rs` module (T04)** — `crates/reposix-remote/src/bus_handler.rs`
(new file, ~180 lines):
- Module doc-comment naming the bus algorithm steps 1–3 (per Q3.3 +
  Q3.4 + Q3.5), the deferred-shipped error contract (Q-B / D-02), and
  the security mitigation for shell-out (T-82-01: `--` + reject-`-`).
- `pub(crate) fn handle_bus_export<R, W>(state: &mut State, proto:
  &mut Protocol<R, W>) -> anyhow::Result<()>` orchestrates STEP 0,
  PRECHECK A, PRECHECK B, deferred-shipped error.
- `fn resolve_mirror_remote_name(mirror_url: &str) -> Result<Option<String>>`
  — STEP 0 helper. Shells out `git config --get-regexp
  '^remote\..+\.url$'`, parses stdout, normalizes URLs (strip
  trailing `/`), filters byte-equal matches, returns first
  alphabetical or `None`. Logs WARNING to stderr if multiple matches.
- `fn precheck_mirror_drift(mirror_url: &str, mirror_remote_name: &str)
  -> Result<MirrorDriftOutcome>` — PRECHECK A. Rejects
  `-`-prefixed mirror URLs (T-82-01) BEFORE shell-out; passes `--`
  separator. Reads local SHA via `git rev-parse refs/remotes/<name>/main`.
  Empty `git ls-remote` output → `Stable`.
  ```rust
  enum MirrorDriftOutcome {
      Stable,
      Drifted { local: String, remote: String },
  }
  ```
- `fn emit_deferred_shipped_error<R, W>(proto: &mut Protocol<R, W>,
  state: &mut State) -> Result<()>` — Q-B / D-02 stub. Sets
  `state.push_failed = true`; emits stderr diagnostic AND stdout
  protocol error; returns `Ok(())`.
- All errors are `anyhow::Result<...>`. NO new typed Error variant.
- All five functions in the module are `pub(crate)` or private to
  the module; only `handle_bus_export` is exported to `main.rs`.

**`main.rs` dispatch wiring (T04)** — `crates/reposix-remote/src/main.rs`:
- New `mod bus_url;` declaration alphabetical with existing modules
  (between `mod backend_dispatch;` line 24 and the next mod).
- New `mod bus_handler;` declaration likewise alphabetical.
- `State` extended with `pub(crate) mirror_url: Option<String>` field.
  Initialized to `None` by default; set to `Some(url)` when
  `Route::Bus` is dispatched in `real_main`.
- `real_main`'s URL parsing replaced: instead of
  `let parsed = parse_dispatch_url(url).context(...)?;`, call
  `let route = bus_url::parse(url).context("parse remote url")?;`,
  match on `Route::Single(parsed) | Route::Bus { sot, mirror_url }`.
  For `Route::Single` the existing `instantiate(&parsed)` +
  `backend_name` + `project_for_cache` plumbing stays UNCHANGED.
  For `Route::Bus` the same `instantiate(&sot)` runs against the
  SoT side, AND `state.mirror_url = Some(mirror_url)` is set.
- Capabilities branching at lines 150-172 (S1 + Pitfall 6): wrap the
  existing `proto.send_line("stateless-connect")?;` line in
  `if state.mirror_url.is_none() { ... }`. The other four capability
  lines (`import`, `export`, `refspec`, `object-format=sha1`) stay
  unchanged.
- `"export"` verb arm at line 186-188: dispatch on
  `state.mirror_url.is_some()` to `bus_handler::handle_bus_export`
  vs the existing `handle_export`. ONE-LINE diff:
  ```rust
  "export" => {
      if state.mirror_url.is_some() {
          bus_handler::handle_bus_export(&mut state, &mut proto)?;
      } else {
          handle_export(&mut state, &mut proto)?;
      }
  }
  ```
- `fn diag` (line 80 of `crates/reposix-remote/src/main.rs`) and
  `fn ensure_cache` (line 219) widened from `fn`-private to
  `pub(crate)` so the sibling `bus_handler` module can call them.
  `fail_push` (line 246) stays private — `bus_handler` defines a
  local `bus_fail_push` wrapper instead. The widening is purely
  additive: no behavior change for `handle_export` /
  `handle_stateless_connect` which already invoke `diag` and
  `ensure_cache` from the same crate. Without this widening, T04's
  `cargo check -p reposix-remote` fails with "function `diag` is
  private" / "function `ensure_cache` is private" because
  `bus_handler::handle_bus_export` calls `crate::diag(...)` and
  `crate::ensure_cache(state)`. The bullet exists at the
  `<must_haves>` level (not buried in T04 HARD-BLOCK paragraphs)
  so an executor authoring `bus_handler.rs` first does NOT
  backtrack on the visibility error.
- `cargo check -p reposix-remote` exits 0 after the wiring.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.

**Integration tests (T05)**:
- `crates/reposix-remote/tests/bus_url.rs::*` — at least 4 tests
  exercising the parser via the public surface (cargo bin invocation
  is overkill; the parser is pure `fn parse(url: &str) -> Result<Route>`,
  testable as a unit but the Cargo test scope expects a tests/
  file). Instead, `tests/bus_url.rs` re-runs the parser through
  `assert_cmd::Command::cargo_bin("git-remote-reposix")` with a bus
  URL and asserts the helper exits non-zero on rejection paths +
  exits with the deferred-shipped error on the success path.
- `crates/reposix-remote/tests/bus_capabilities.rs::bus_url_omits_stateless_connect`
  (1 test) — drives the helper with `argv[2] = "reposix::sim::demo?mirror=file:///tmp/mock.git"`,
  sends `capabilities\n` on stdin, reads stdout, asserts the line
  set contains `import`, `export`, `refspec refs/heads/*:refs/reposix/*`,
  `object-format=sha1` AND does NOT contain `stateless-connect`.
- `crates/reposix-remote/tests/bus_precheck_a.rs::bus_precheck_a_emits_fetch_first_on_drift`
  (1 test) — file:// fixture: `tempfile::tempdir()` + two bare repos
  (one "mirror" with N+1 commits, one "local" with N commits); set
  up a working-tree-mock via `git config remote.mirror.url
  file:///tmp/mirror.git` + `git update-ref refs/remotes/mirror/main
  <local-sha>`; bus URL points at `file:///tmp/mirror.git`; helper
  exits non-zero with `error refs/heads/main fetch first` on stdout
  AND `your GH mirror has new commits` on stderr.
  PLUS sibling `bus_precheck_a_passes_when_mirror_in_sync` (single-mirror,
  refs match) asserts the helper proceeds past PRECHECK A (does NOT
  emit `fetch first`; emits the deferred-shipped error from Q-B
  instead).
- `crates/reposix-remote/tests/bus_precheck_b.rs::bus_precheck_b_emits_fetch_first_on_sot_drift`
  (1 test) — wiremock-backed sim per P81's `tests/perf_l1.rs`
  pattern; seed cache with `last_fetched_at` cursor at T0; mock
  `list_changed_since` to return `[id 5]`; mirror is in sync (PRECHECK
  A passes); bus URL points at the wiremock + a file:// mirror;
  helper exits non-zero with `error refs/heads/main fetch first`
  on stdout AND hint citing `refs/mirrors/<sot>-synced-at` (when
  populated by a prior P80 push) on stderr; assert ZERO PATCH/PUT
  calls hit wiremock (`Mock::expect(0)`).
  PLUS sibling `bus_precheck_b_passes_when_sot_stable` (mock
  `list_changed_since` returns `[]`) asserts the helper proceeds
  past PRECHECK B and emits the deferred-shipped error.

**Catalog rows + verifiers (T01)** — 6 rows + 6 TINY shells:
- All 6 rows live in `quality/catalogs/agent-ux.json` (D-04).
- Initial status `FAIL` (verifiers exist but tests don't yet pass
  until T05). Hand-edited per documented gap (NOT Principle A) —
  same shape as P81's `agent-ux/sync-reconcile-subcommand` row,
  citing GOOD-TO-HAVES-01.
- 6 TINY shells under `quality/gates/agent-ux/` (each ~30-50 lines,
  delegate to `cargo test -p reposix-remote --test <name>` per the
  P81 precedent).
- All 6 rows flip FAIL → PASS via the runner during T06 BEFORE the
  per-phase push commits.

Row IDs:
1. `agent-ux/bus-url-parses-query-param-form` →
   `quality/gates/agent-ux/bus-url-parses-query-param-form.sh`
2. `agent-ux/bus-url-rejects-plus-delimited` →
   `quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh`
3. `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` →
   `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh`
4. `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first` →
   `quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh`
5. `agent-ux/bus-fetch-not-advertised` →
   `quality/gates/agent-ux/bus-fetch-not-advertised.sh`
6. `agent-ux/bus-no-remote-configured-error` →
   `quality/gates/agent-ux/bus-no-remote-configured-error.sh`

**CLAUDE.md (T06; QG-07; one paragraph + one bullet):**
- § Architecture: new paragraph (3-5 sentences) introducing the
  bus URL form `reposix::<sot>?mirror=<mirror-url>`, naming Q3.3
  (form RATIFIED), Q3.4 (PUSH-only — fetch falls through to
  single-backend), and the percent-encoding requirement for
  mirror URLs containing `?` (Pitfall 7). Cite
  `architecture-sketch.md § 3` and `decisions.md § Q3.3-Q3.6`.
- § Commands → "Local dev loop" block: bullet for the bus push
  form (1 line):
  ```
  git push reposix main                                     # bus push (SoT-first, mirror-best-effort) — DVCS-BUS-* in v0.13.0
  ```
  (this is illustrative; P83 lands the actual write fan-out, but
  P82 already supports the URL-recognition + capability-advertisement
  side, so the bullet is honest about what works today).

**Phase-close contract:**
- Plan terminates with `git push origin main` in T06 (per CLAUDE.md
  push cadence) with pre-push GREEN. Verifier subagent dispatch is
  an orchestrator-level action AFTER push lands — NOT a plan task.
- All cargo invocations SERIAL (one at a time per CLAUDE.md "Build
  memory budget"). Per-crate (`-p reposix-remote`) only. NO
  `cargo --workspace`.
- NO new error variants on `RemoteError` or `cache::Error`. Bus
  handler errors propagate via `anyhow::Result<...>`; `fail_push`
  reject paths reuse the existing `(diag, kind, detail)` shape.
- Q-A / Q-B / Q-C RATIFIED in three places: plan body, inline doc
  comment in `bus_url.rs` + `bus_handler.rs`, CLAUDE.md (T06).
</must_haves>

<canonical_refs>
**Spec sources:**
- `.planning/REQUIREMENTS.md` DVCS-BUS-URL-01 / DVCS-BUS-PRECHECK-01 /
  DVCS-BUS-PRECHECK-02 / DVCS-BUS-FETCH-01 (lines 71-80) — verbatim
  acceptance.
- `.planning/ROADMAP.md` § Phase 82 (lines 124-143) — phase goal +
  7 success criteria.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "3. Bus
  remote with cheap-precheck + SoT-first-write" (lines 83-146) — bus
  algorithm steps 1–3 (P82 scope) + steps 4–9 (P83 deferred).
- `.planning/research/v0.13.0-dvcs/decisions.md` Q3.3 (URL form),
  Q3.4 (PUSH-only), Q3.5 (no auto-mutate), Q3.6 (no helper retry).
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md` — full
  research bundle (especially § Architecture Patterns Pattern 1, §
  Common Pitfalls 1-7, § Catalog Row Design, § Test Fixture Strategy).
- `.planning/phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW.md`
  § "Decisions ratified at plan time" (D-01..D-06).

**Bus URL parser substrate (T02):**
- `crates/reposix-remote/src/backend_dispatch.rs:74-200`
  (`ParsedRemote`, `parse_remote_url`, `BackendKind::slug`,
  `instantiate`).
- `crates/reposix-core/src/remote.rs:43` (`split_reposix_url`) — the
  canonical splitter `parse_remote_url` delegates to.

**Coarser SoT precheck wrapper (T03):**
- `crates/reposix-remote/src/precheck.rs` (entire file, 302 lines
  post-P81) — donor pattern; the new wrapper is a 10-line sibling
  of `precheck_export_against_changed_set` (lines 80-302).
- `crates/reposix-cache/src/cache.rs::read_last_fetched_at` (P81)
  — read cursor; first-push fallback semantics.
- `crates/reposix-core/src/backend.rs:253` —
  `BackendConnector::list_changed_since` signature.

**Bus handler substrate (T04):**
- `crates/reposix-remote/src/main.rs::handle_export` (currently lines
  280-549 post-P81) — sibling pattern; bus_handler shares the
  diag()/fail_push()/Protocol idiom but does NOT call
  parse_export_stream.
- `crates/reposix-remote/src/main.rs:48` (`State` struct definition)
  — extension site (`mirror_url: Option<String>`).
- `crates/reposix-remote/src/main.rs:103-136` (`real_main` body) —
  URL dispatch site.
- `crates/reposix-remote/src/main.rs:150-172` (`"capabilities"` arm)
  — capabilities branching site (S1).
- `crates/reposix-remote/src/main.rs:186-188` (`"export"` arm) —
  bus dispatch insertion site.
- `crates/reposix-remote/src/main.rs:246-258` (`fail_push`) — reject-
  path helper bus_handler reuses verbatim.
- `crates/reposix-cli/src/doctor.rs:446-944` — donor pattern for
  `Command::new("git").args(...).output()` shell-outs (D-06).
- `crates/reposix-cache/src/mirror_refs.rs:227` —
  `Cache::read_mirror_synced_at` (P80) for PRECHECK B's hint
  composition.

**Test fixtures (T05):**
- `crates/reposix-remote/tests/perf_l1.rs` (P81) — wiremock fixture
  donor pattern for `bus_precheck_b.rs`.
- `crates/reposix-remote/tests/mirror_refs.rs` (P80) —
  helper-driver donor pattern (`drive_helper_export`,
  `render_with_overrides`, etc.) for `bus_precheck_a.rs` and
  `bus_capabilities.rs`.
- `scripts/dark-factory-test.sh` — file:// bare-repo fixture donor
  pattern (RESEARCH.md Test Fixture Strategy option (a)).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` —
  `wiremock`, `assert_cmd`, `tempfile` already present (verified
  during P81).

**Quality Gates:**
- `quality/catalogs/agent-ux.json` — existing file with 6 rows
  (P79/P80/P81 precedents); the 6 new rows join.
- `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (P81 TINY
  verifier precedent — 30-line shape).
- `quality/gates/agent-ux/mirror-refs-write-on-success.sh` (P80 TINY
  verifier precedent).
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" + §
  "Principle A".

**Operating principles:**
- `CLAUDE.md` § "Build memory budget" — strict serial cargo,
  per-crate fallback.
- `CLAUDE.md` § "Push cadence — per-phase" — terminal push protocol.
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first), OP-2
  (Tainted-by-default — no new tainted byte source in P82), OP-3
  (audit log unchanged in P82 — bus_handler does NOT write audit
  rows for the deferred-shipped path; P83 wires audit), OP-7
  (verifier subagent), OP-8 (+2 reservation).
- `CLAUDE.md` § "Threat model" — `<threat_model>` section below
  enumerates the new shell-out boundary's STRIDE register.
- `CLAUDE.md` § Quality Gates — 9 dimensions / 6 cadences / 5 kinds.

This plan introduces TWO new shell-out construction sites
(`git ls-remote` for PRECHECK A; `git config --get-regexp` for
STEP 0; `git rev-parse` for local SHA read). The
`BackendConnector::list_changed_since` call in PRECHECK B goes
through the existing `client()` factory + `REPOSIX_ALLOWED_ORIGINS`
allowlist (no new HTTP construction site). See `<threat_model>`
below for the STRIDE register.
</canonical_refs>
