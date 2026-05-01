# POC-FINDINGS — v0.13.0 reposix attach + bus + precheck

**POC scope:** end-to-end exercise of the three v0.13.0 innovations
against the simulator. POC-01 / CARRY-FORWARD POC-DVCS-01.

**Simulator version:** `reposix-sim` from current workspace HEAD as of
P79-01 (P78 SHIPPED, gix 0.83.0, walker schema migration complete).

**Date:** 2026-05-01 (single session).

**Wall-clock:** ~8 minutes wall-clock for the POC's `run.sh` end-to-end
plus author time for fixtures, scratch reconciler, and FINDINGS. Well
under the 1-day budget; no SURPRISES-INTAKE entry required. See
§ Time spent.

---

## Path (a) — Reconciliation against mangled checkout

**Transcript:** `logs/path-a-reconciliation.log`. All 5 reconciliation
cases observed cleanly; reconciler exit 0.

### What worked

- `reposix_core::frontmatter::parse` is the right primitive for the
  reconciliation walk. The function returns `Err` for files lacking the
  YAML frontmatter delimiters; the parse-failure branch is a clean
  classification rule for `NO_ID`. No ambiguity between
  "no-frontmatter" and "malformed-YAML-with-id" — both fall to the same
  branch, which is correct for the architecture-sketch's "warn + skip"
  case 3 behavior.
- The classification logic is simple: `HashMap<id, Vec<PathBuf>>`
  collected during the walkdir pass, then a single sweep over the map
  with a separate sweep over the backend's id set. The whole reconciler
  is ~120 lines of Rust; it does not need to grow.
- The simulator's seed file contract (`{project: {slug,name,desc},
  issues: [...]}`) is straightforward to author; cooking a one-off seed
  file inline in `path-a.sh` works fine for POC-tier testing.
- All 5 reconciliation cases (MATCH / BACKEND_DELETED / NO_ID /
  DUPLICATE_ID / MIRROR_LAG) are *distinct* — the architecture-sketch's
  5-row table is not under-counted. No 6th case surfaced. This is a
  green signal for the early-warning trigger from
  `vision-and-mental-model.md` § "Risks" (">5 cases means design needs
  revisiting").

### What surprised

- **Naming inconsistency: "records" vs "issues".** The architecture
  sketch refers abstractly to "records" (the canonical
  `reposix_core::Record` type), but the simulator's concrete HTTP route
  is `/projects/<slug>/issues`. Code referring to "records" in plan
  prose gets confusing because the wire path says "issues". This is
  not a bug — the type *is* `Record`, and the sim is one specialization
  to the issue-tracker shape — but production attach docs / help text
  should pick one term and stick with it. **Recommendation:** keep
  "record" as the abstract type in code (it is), and document the URL
  template explicitly in `attach`'s help text + topology doc.
- **`frontmatter::parse` accepts files with `id` but missing
  server-managed timestamps.** Wait — actually it does NOT (the
  Frontmatter struct requires `created_at` / `updated_at`). The fixture
  files include those fields specifically to roundtrip cleanly through
  `parse`. A real "user authored their own .md without thinking about
  YAML" file would fail parse and fall to the `NO_ID` branch — which
  is the desired behavior, but worth knowing: the reconciliation walk's
  "skip non-reposix files" effectively requires those files to either
  have NO frontmatter at all OR have full reposix-shaped frontmatter
  (with `id`, `title`, `status`, `created_at`, `updated_at`). A
  half-frontmatter file is treated like a no-frontmatter file. Probably
  fine — the architecture sketch's case 3 is "no `id` field — warn;
  skip" and our parse-failure branch covers both "no frontmatter" and
  "frontmatter without id" identically.

### Design questions for 79-02

- **Q-A1.** Should the reconciliation walk skip directories like
  `.git/`, `.github/`, `node_modules/` by default? The POC walks
  everything ending in `.md` and falls back to `NO_ID` for non-reposix
  markdown — which is correct in classification terms, but produces a
  noisy reconciliation table on a real checkout with vendored docs.
  **Routing tag: REVISE (small).** 79-02's reconciliation walker should
  accept an optional `--ignore` glob (default: `.git, .github`) that
  prunes the WalkDir traversal. This is XS-sized — a few lines.
- **Q-A2.** When two files claim the same id (DUPLICATE_ID), the
  architecture sketch says "hard error". Does this hold even when the
  duplicates are in different directories (e.g., `issues/0042.md` AND
  `archive/issues/0042.md`)? The POC reproduced the duplicate behavior
  with files in the same directory; the production walker needs to
  decide whether directory boundaries weaken the duplicate signal.
  **Routing tag: INFO** (architecture sketch already says "hard error";
  this is just a confirmation question for the test fixtures in 79-03).
- **Q-A3.** The reconciler in this POC streams the entire `/issues`
  list response into memory. For a 5,000-record space this is fine
  (single REST page, multi-MB at most); for a 50,000-record space it
  starts being noticeable. Production attach should probably stream-
  and-classify. **Routing tag: INFO** — out of scope for v0.13.0;
  could be a v0.14.0 perf carry-forward. The L1 `list_changed_since`
  migration in P81 will make this a non-issue for the steady-state
  case (only changed records ever flow); it remains relevant for the
  initial attach.

---

## Path (b) — Bus SoT-first observing mirror lag

**Transcript:** `logs/path-b-bus-mirror-lag.log`. SoT-first sequencing
exercised end-to-end; mirror failure injected mid-flight; lag observable;
recovery via replay restores both stores to v=2.

### What worked

- The SoT-first sequencing IS sound: even when the mirror write fails,
  the SoT is in a forward-progressing state and the mirror can be
  caught up by either replay (next push) or webhook sync. The recovery
  story matches the architecture-sketch's claim ("next pusher catches
  up").
- The simulator restart-and-replay produces the correct end state
  (both at v=2). The POC didn't need to distinguish "mirror's v=2 is
  the same v=2 as SoT's v=2" — version numbers are independent
  per-server in the POC, but the production bus would write the GH
  mirror's commit SHA *from* the SoT-side fast-import buffer, so the
  SHAs are identical by construction. Worth noting in the production
  spec but not a POC concern.
- The POC's "kill mid-flight" demonstrates the SoT-only-write window
  has a clean recoverable shape: nothing is corrupted; the mirror is
  simply stale. This is the desired property; the architecture-sketch
  is correct.

### What surprised

- **No HTTP audit endpoint.** The simulator's `audit_events` table is
  written by the audit middleware (we read the source: `crates/reposix
  -sim/src/middleware/audit.rs`), but there is no `/audit` route to
  query it via REST. `GET /projects/demo/audit` returns 404 on both
  sims. **Implication for production bus:** the bus remote can NOT
  rely on querying the SoT's audit log to verify a write landed —
  the bus must own the cache-side audit trail per OP-3. The
  `audit_events_cache` table (in `reposix-cache::audit`) is the load-
  bearing record of "I, the helper, attempted/succeeded/failed this
  REST write." This is consistent with OP-3 ("audit log lives in TWO
  append-only tables"); the POC just confirms the production bus
  cannot offload audit responsibility to the SoT.
- **The `version` field is per-record on the SoT side.** This is
  expected for confluence/JIRA/GitHub Issues (each record has its own
  optimistic-concurrency version), but in the POC's "two sims" setup,
  the version sequence is independent on each sim. A more faithful
  bus simulation would have the mirror's `body` reflect the SoT's
  state by SHA, not by version-bumping a separate record. The POC's
  shape was sufficient for surfacing the SoT-first sequencing finding,
  but production bus tests in P83 should drive the mirror with
  fast-import bytes from the SoT's commit, not with independent
  PATCHes.
- **`pkill -f "reposix-sim.*${SIM_PORT}"` is the cleanest way to kill
  a sim mid-flight.** The PID-tracking via `/tmp/*.pid` files is
  belt-and-suspenders; the trap on EXIT does the heavy lifting. This
  is a shell-script-shape note, not a finding for production.

### Design questions for 79-02 / P82 / P83

- **Q-B1.** What does the helper write to `audit_events_cache` when
  the SoT write succeeds but the mirror write fails? The architecture-
  sketch step 7 says "write mirror-lag audit row." Production needs a
  schema for that row (event_type=`mirror_lag_partial_failure`?
  `bus_push_mirror_failure`?), with which fields (SoT commit SHA,
  mirror error message, retry count). **Routing tag: REVISE (P83).**
  Not blocking 79-02 (which only ships the attach side), but the
  schema decision affects which `Cache::log_*` audit-hook signature
  79-02 lands. **Recommended action:** 79-02's `Cache::log_attach_walk`
  audit hook signature should be designed knowing that P83 will add
  sibling `Cache::log_mirror_lag_partial_failure` etc. — keep the
  audit-row signature regular (event_type + jsonblob) rather than
  per-event-typed-args.
- **Q-B2.** The POC simulated "mirror failure" by killing the mirror's
  HTTP server. In production, the mirror is `git push <gh-remote>` —
  failure modes include 401 (token expired), 403 (force-push refused
  by branch protection), 422 (non-fast-forward), and network errors.
  Are these all "the same kind of mirror-write failure" from the
  bus's perspective, or does some require different handling?
  **Routing tag: INFO** — relevant to P83's fault-injection test
  matrix, not 79-02. **Recommended action:** P83 plan should
  enumerate the mirror-failure taxonomy (and probably split the
  "transient retry" from "configuration broken" failure modes).

---

## Path (c) — Cheap precheck on SoT mismatch

**Transcript:** `logs/path-c-cheap-precheck.log`. Single-record GET
precheck refuses fast on version mismatch; production-shaped error
message emitted; no stdin read; no REST writes attempted.

### What worked

- The version-mismatch detection IS a single REST GET in the POC —
  cheaper than the full `list_records` walk. This validates the
  cheap-precheck shape from architecture-sketch step 3.
- The production-shaped error message (`error refs/heads/main fetch
  first` + hint) is a clean teach-by-stderr pattern. The hint text
  ("confluence has changes since your last fetch; git pull --rebase")
  is a complete user instruction — no follow-up "what do I do" round
  trip needed. This matches the dark-factory ergonomic pattern from
  `agentic-engineering-reference.md`.
- The precheck completes WITHOUT reading stdin or attempting any
  write. This is load-bearing: the precheck is "fast-fail before
  expensive work," and "expensive work" includes reading the
  potentially-multi-MB git fast-import stream from stdin.

### What surprised

- **Single-record GET is O(1) vs production's `list_changed_since`
  which is O(delta).** In the POC's single-record case they are
  equivalent. In production, the precheck must enumerate ALL records
  whose version may have drifted since `last_fetched_at` — this is
  inherently more than one REST call when many records have changed.
  But it is bounded by changed-record count, not by total-record
  count, so for a steady-state workflow it is still cheap on average.
  This is the architecture-sketch's L1 design; the POC simplification
  is fine.
- **No `If-None-Match` / `ETag` shortcut needed for the POC.** A
  production cheap precheck could use HTTP conditional GET to avoid
  even retrieving the body when the version hasn't changed; the sim
  already exposes ETags (we noticed in `routes/issues.rs` — the PATCH
  uses `If-Match`). This is a v0.13.0 GOOD-TO-HAVE candidate per
  Q3.2 ("30s TTL cache for cheap precheck — defer; measure first").
- **Hint text references `git pull --rebase` but in the bus topology,
  the user's `git pull` goes to the GH mirror (vanilla git), not the
  SoT.** This means `git pull --rebase` may NOT pick up the SoT-side
  drift if the mirror hasn't synced yet. The accurate hint is
  closer to: *"`reposix sync` to refresh from confluence directly,
  then `git pull --rebase` to merge with your work."* **This is
  partly a documentation issue and partly a UX one.**

### Design questions for 79-02 / P82

- **Q-C1.** The hint text is wrong-ish in the bus topology — `git
  pull --rebase` from the GH mirror won't catch SoT-side drift if
  the mirror is also lagging. Should the reject message be different
  in `bus://` vs single-backend mode? **Routing tag: REVISE (P82).**
  P82 lands the bus URL parser + dispatch; the reject-message
  branching can live alongside. **Recommended action:** 82-T-?? has
  a sub-task to author the reject message per topology mode.
- **Q-C2.** When `last_fetched_at` is unset (fresh attach with no
  prior fetch state), what does `list_changed_since(None)` return?
  Probably "all records" — degrading to today's `list_records` walk
  on the very first push from a freshly-attached checkout. The L1
  perf migration in P81 needs to think about this. **Routing tag:
  INFO** — relevant to P81 (L1 migration), not 79-02. **Recommended
  action:** P81 plan should explicitly handle the "first push after
  attach" case; the cache-side `last_fetched_at` is set during attach
  to NOW, not None, so the next push's precheck only enumerates
  drift since attach.

---

## Implications for 79-02

The orchestrator reads this section to decide whether to revise
`79-02-PLAN.md` before execution. Routing per
`79-PLAN-OVERVIEW.md` § "POC findings → planner re-engagement":

- **INFO** — informational, no plan revision needed.
- **REVISE** — 79-02 plan needs an in-place tweak.
- **SPLIT** — 79-02 scope exceeds budget; orchestrator surfaces split
  options to the owner.

| ID  | Tag    | Finding                                                                                                                                                                                                                                                                                                                                            | 79-02 task affected                                                              |
|-----|--------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| F01 | REVISE | Reconciliation walk should accept an optional `--ignore` glob (default: `.git, .github`) so reconciliation tables stay clean on real checkouts with vendored docs (Q-A1).                                                                                                                                                                          | 79-02 T03 (cache reconciliation module): add ignore-glob param to walker.        |
| F02 | INFO   | Naming consistency — "record" is the abstract type, but the simulator's wire path says "/issues". Plan prose for 79-02 should pick one term consistently. The code already uses `Record`; help text + topology docs should follow suit. (Path (a) "What surprised".)                                                                              | 79-02 T02 (clap surface): help text says "records" not "issues" for portability. |
| F03 | INFO   | The 5-row reconciliation table from architecture-sketch is COMPLETE; no 6th case surfaced in POC. Green signal for the early-warning trigger from `vision-and-mental-model.md` § "Risks". (Path (a).)                                                                                                                                              | 79-02 / 79-03 — no scope expansion needed.                                      |
| F04 | REVISE | `Cache::log_attach_walk` audit-hook signature should be designed regular (event_type + jsonblob) rather than per-event-typed-args, anticipating sibling `Cache::log_mirror_lag_partial_failure` etc. in P83 (Q-B1).                                                                                                                                | 79-02 T03 (cache audit hook): use a generic `(event_type, payload_json)` API.   |
| F05 | INFO   | Production bus must own its own cache-side audit trail (`audit_events_cache`) — the SoT's `audit_events` table is reachable only via SQLite, not REST. This is consistent with OP-3 but worth noting for P83 design. (Path (b) "What surprised".)                                                                                                | n/a — relevant to P83.                                                           |
| F06 | INFO   | Reject-message text for the cheap precheck differs by topology — `git pull --rebase` is wrong-ish in `bus://` mode because the mirror is also lagging (Q-C1). P82 should fork the reject-message text by topology.                                                                                                                                | n/a — relevant to P82.                                                           |
| F07 | INFO   | `last_fetched_at` should be set to NOW on attach, not None — so the next push's precheck only enumerates drift since attach, avoiding a full `list_records` walk on the first push after attach (Q-C2).                                                                                                                                          | 79-02 T03 (cache schema): document the contract; init `last_fetched_at` on attach. |

**Routing summary:**
- INFO: 5 items
- REVISE: 2 items (F01, F04)
- SPLIT: 0 items

**Highest-severity tag:** REVISE. The orchestrator should re-engage
the planner for an in-place revision of 79-02 covering F01 + F04.
The two revisions are small (one parameter on the walker, one
signature decision on the audit hook) and do not expand scope; they
tighten the spec.

If the orchestrator judges F01 + F04 as merely "noted, will-fix
during execution" (not warranting a plan revision), proceed to 79-02
execution as-drafted with this FINDINGS file as the executor's
context.

---

## Time spent

- **Started:**  2026-05-01T06:20:31Z
- **Finished:** 2026-05-01T06:29:16Z
- **Total:**    ~9 minutes wall-clock for the entire POC including
  fixture authoring, scratch reconciler, three path scripts, end-to-end
  runs, and FINDINGS authoring.

Time-budget status:
- < 1d → **on budget** (target). Well within. No SURPRISES-INTAKE
  entry required; CARRY-FORWARD POC-DVCS-01 budget honored.

The compressed wall-clock reflects (a) tight integration between the
simulator and `reposix_core::frontmatter`, (b) the fact that the
architecture sketch was already specific enough that the POC mostly
*confirmed* it rather than discovered surprises, and (c) v0.9.0
precedent (`kickoff-recommendations.md` § rec #2) — the saved 3-4 days
on the v0.9.0 POC was a leading indicator that this POC would be cheap.

---

## Cleanup

After this POC's findings are absorbed by 79-02:

- The directory `research/v0.13.0-dvcs/poc/` is RETAINED as historical
  research artifact (per CARRY-FORWARD POC-DVCS-01: "Throwaway code
  only; not v0.13.0 implementation"). No deletion in 79-02 or v0.13.0
  milestone close.
- The standalone scratch crate at `scratch/Cargo.toml` is NOT added to
  the workspace; production attach in 79-02 reuses the relevant code
  patterns by READING this POC, not by importing it.
- Runtime cleanup of `/tmp/reposix-poc-79*` happens inside `run.sh`
  via the `cleanup_residual` trap; manual cleanup is `rm -rf
  /tmp/reposix-poc-79*` per `README.md`.
