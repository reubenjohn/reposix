# Phase 80 — Plan Check

> **Revision status (2026-05-01).** The planner revised
> `80-01-PLAN.md` in response to this check. **All 3 HIGH issues**
> (H1 sim-port routing, H2 gix 0.83 tag invocation, H3 vacuous
> first-push test) and **4 of 5 MEDIUM issues** (M1 dev-deps, M2
> cache.db path, M3 Q2.2 verbatim phrase carrier, M4 fixed verifier
> ports) are addressed. **M5** (verifier-shell line-count
> watch-list) is left as guidance for the executor (not a code
> change). **L1–L5** are intentionally not addressed (stylistic
> per § Recommendation). See `80-01-PLAN.md` lines 26–37 for the
> embedded revision note + per-issue summary; revised plan is
> 2,443 lines (+184 net delta).

---

**Reviewer:** plan-checker subagent (goal-backward verification, pre-execution)
**Date:** 2026-05-01
**Plans reviewed:** `80-PLAN-OVERVIEW.md` (365 lines), `80-01-PLAN.md` (2,259 lines)
**Reference:** `80-RESEARCH.md`, ROADMAP § Phase 80 (lines 83–101), REQUIREMENTS DVCS-MIRROR-REFS-01..03, decisions Q2.1/Q2.2/Q2.3, `crates/reposix-cache/src/sync_tag.rs`, `crates/reposix-remote/src/main.rs::handle_export`, `CLAUDE.md`.

---

## Verdict: YELLOW

**Summary.** The plan is structurally sound, comprehensively researched, and respects every load-bearing CLAUDE.md operating principle (catalog-first, OP-3 audit, per-phase push, per-crate cargo, threat model). It maps each ROADMAP success criterion to a concrete artifact, and the donor-pattern citation discipline (`sync_tag.rs`, `log_sync_tag_written`, `reposix-attach.sh`) is exemplary.

However, there are **THREE HIGH-severity issues** that will fail execution as written:

1. **`reposix init` does NOT honor `REPOSIX_SIM_ORIGIN`** — verifier shells and integration tests will dial port 7878 (default) instead of the per-test port the sim is bound to. The plan inherits the env-var-based override pattern from P79's `reposix attach` verifier without verifying that `init` supports it.
2. **`gix::Repository::tag(...)` argument order is wrong** in the plan's code sample — the plan passes `(name, target, PreviousValue::Any, Some(committer), &message, true)` but the gix 0.83 signature is `(name, target, target_kind: gix_object::Kind, tagger: Option<SignatureRef>, message, constraint: PreviousValue)`. The bool `true` and the misplaced `PreviousValue::Any` will not compile. RESEARCH.md A1 hedged this; the plan should NOT have shipped a concrete invocation without confirming the signature.
3. **`reject_hint_first_push_omits_synced_at_line` integration test asserts nothing meaningful** — the "weaker form" the plan ships drives a SUCCESSFUL push (which never enters the conflict-reject branch) and asserts no "minutes ago" string in stderr. The success branch never composes the synced-at hint anyway, so this assertion is vacuously true regardless of correctness. SC4 first-push None-case behavior is not behaviorally tested at the helper layer.

A handful of MEDIUM issues (catalog test path bug, missing dev-dep declarations, ambiguous Q2.2 verbatim treatment) are listed below.

These are tractable to fix without splitting the phase. Verdict is YELLOW — minor revisions recommended before T01 lands.

---

## Per-question findings

### 1. Will execution actually deliver the phase goal?

**Mostly YES, with caveats.**

The architectural understanding is correct: refs live in the cache's bare repo (`Cache::repo`), the helper's `stateless-connect` tunnels `git upload-pack --advertise-refs` against that bare repo, vanilla `git fetch` from the working tree pulls all advertised refs. I verified `crates/reposix-remote/src/stateless_connect.rs:196` (`send_advertisement(proto, cache.repo_path())`) — `git upload-pack` advertises every non-hidden ref, so `refs/mirrors/*` automatically propagates without a code change, since `transfer.hideRefs` is configured to hide ONLY `refs/reposix/sync/` (`crates/reposix-cache/src/cache.rs:102–113`). RESEARCH.md A3's "one-line edit" is correctly downgraded by the plan to "possibly zero-line; investigate first" (T03 step 3a).

The cache-as-source-of-truth point is well documented in S1 of the overview and the `mirror_refs.rs` module-doc.

**Caveat (HIGH):** the goal is "vanilla `git fetch` brings these refs along" — but the verifier shells that prove this run `reposix init sim::demo` against a sim bound to a non-default port and CANNOT reach it (see Issue H1 below). The integration tests in Rust have the same flaw. The plan's machinery for connecting the working tree to the helper is broken at the harness layer, which means SC3 cannot pass without a fix.

### 2. All 6 ROADMAP success criteria covered?

| SC | Coverage | Notes |
|----|----------|-------|
| SC1 | YES | `mirror_refs.rs` writers + readers; constants + name formatters re-exported from `lib.rs`; namespace `refs/mirrors/<sot-host>-{head,synced-at}`. |
| SC2 | YES | `handle_export` success branch (lines 470–489 wiring); audit row written via `log_mirror_sync_written`. |
| SC3 | YES (with H1 caveat) | Integration test `vanilla_fetch_brings_mirror_refs` + verifier `mirror-refs-readable-by-vanilla-fetch.sh`. Both blocked by H1. |
| SC4 | PARTIAL (M3) | Reject hint cites refs + RFC3339 + (N minutes ago). The Q2.2 verbatim clarification phrase ("synced-at is the timestamp the mirror last caught up to confluence — NOT a 'current SoT state' marker") is carried into CLAUDE.md as a doc-clarity contract carrier, NOT into the reject stderr. The ROADMAP wording is ambiguous about whether the verbatim phrase MUST appear in stderr; the plan's reading is defensible but worth flagging — see Issue M3. |
| SC5 | YES | T01 mints rows + verifiers FIRST (in the same atomic commit, BEFORE T02 implementation); CLAUDE.md update lands in T04 same PR. |
| SC6 | YES | T04 ends with `git push origin main`; verifier subagent dispatch is orchestrator-level after push. |

### 3. Catalog-first invariant respected?

**YES.** T01 mints 3 rows in `quality/catalogs/agent-ux.json` with `status: FAIL` BEFORE any Rust changes. The runner re-grades to PASS at T04 BEFORE the terminal push. The hand-edit pathway is correctly annotated as "NOT Principle A" (Principle A applies to docs-alignment dim only); each row's `_provenance_note` references GOOD-TO-HAVES-01. Atomic commit shape matches P79's precedent.

### 4. OP-3 audit row UNCONDITIONAL?

**FUNCTIONALLY YES, but defensively WEAK.** The plan's wiring at lines 1605–1619 calls `cache.write_mirror_head` and `cache.write_mirror_synced_at` (each best-effort with WARN-on-fail; no `?` propagation), then UNCONDITIONALLY calls `cache.log_mirror_sync_written`. Code flow guarantees the audit row writes whether or not the ref writes succeeded.

However: the audit-row write is AFTER the ref-write attempts. If a future refactor changes ref-writes to propagate errors (`?`), the audit row would silently be lost. The defensive form is to write the audit row FIRST (or wrap in panic-recovery via `std::panic::catch_unwind`). The current shape is consistent with the existing `log_helper_push_accepted` precedent (line 471), so this is not strictly out-of-pattern, but the plan should add an inline comment ("audit row MUST remain unconditional even if ref writes are upgraded to propagate errors") to prevent future regression.

### 5. Cargo discipline?

**YES.** Every cargo invocation in the plan is per-crate (`-p reposix-cache` / `-p reposix-remote`). T02 → T03 → T04 are strictly sequential. The plan explicitly cites CLAUDE.md "Build memory budget" multiple times. Pre-push hook handles the workspace-wide gate; phase tasks never duplicate it. No parallel cargo invocations.

### 6. Threat model integrity?

**YES.** The `<threat_model>` section enumerates 3 STRIDE threats:
- T-80-01 Tampering (ref name composition) → mitigated by `gix::refs::FullName::try_from` + controlled `state.backend_name` enum.
- T-80-02 Information Disclosure (misleading staleness window) → mitigated by Q2.2 doc-clarity contract.
- T-80-03 DoS (reflog growth) → accepted, deferred to v0.14.0.

No new HTTP origin, no new `Tainted<T>` propagation path, no new shell-out. Refs are written using attacker-uninfluenced data: timestamps from `chrono::Utc::now()` (local clock), `sot_sha` from `cache.build_from()` (helper-derived synthesis-commit OID, not attacker-influenced — see plan S2), `sot_host` slug from controlled `state.backend_name` enum. The annotated tag's message body is RFC3339 + a fixed prefix — no shell escape risk. All correct.

The plan correctly resolves the RESEARCH A2 ambiguity (`parsed.commits.last()` does not exist; `ParsedExport` has `commit_message`, `blobs`, `tree`, `deletes`) by using `cache.build_from()` post-write — this is verified against `crates/reposix-remote/src/fast_import.rs:72-81`.

### 7. Plan size + executor context budget

**LARGE BUT NAVIGABLE.** 2,259 lines is at the high end. The plan is a single phase with 4 sequential tasks; the executor reads it linearly. The size is dominated by:

- Three full verifier-shell scripts inlined verbatim (~225 lines).
- The full `mirror_refs.rs` module body inlined (~280 lines).
- The full `handle_export` wiring deltas inlined twice (success + reject; ~150 lines).
- Four full integration-test bodies inlined (~250 lines).
- A long `<canonical_refs>` block (~110 lines) duplicated against 80-RESEARCH.md.

**Trim candidates (~400 lines, no executor signal lost):**

- `<canonical_refs>` block (lines 356–469) duplicates 80-RESEARCH.md's "Sources" section; the plan could replace ~70% of these with a single "see RESEARCH.md § Sources for the full citation map" referral.
- The Architecture preamble in `<objective>` (lines 53–137) duplicates 80-PLAN-OVERVIEW.md § "Subtle architectural points (S1, S2)" almost verbatim; one of the two is sufficient — keep the OVERVIEW version, trim the PLAN version to a 5-line summary.
- Risks table in 80-PLAN-OVERVIEW.md (lines 239–249) duplicates content in the 80-01 plan's `<must_haves>` and `<threat_model>` — the OVERVIEW could narrow to a 3-row table covering only the LOW-likelihood items not already addressed in tasks.

That said: the plan is for a SINGLE executor reading linearly through 4 sequential tasks, with no parallelism, no separate plan files. The redundancy is friction, not a blocker. Estimate: trimming 400 lines saves ~5% executor context; not worth re-revising for that alone unless other revisions are required.

### 8. Open questions Q1–Q6 — deferrable to executor verification?

| Q | Topic | Acceptable to defer? | Notes |
|---|-------|---------------------|-------|
| Q1 | gix `Repository::tag` API name at gix 0.83 | NO (HIGH H2) | Plan writes a concrete invocation (lines 1132–1142) with WRONG argument order. Verified gix 0.83 signature; the plan's call won't compile. The "executor verifies via cargo check" hedge is too soft — at minimum the plan should cite the correct signature OR commit to the fallback path (`write_object` + `tag_reference`) up front. |
| Q2 | `cache.repo.committer()` accessor name | YES | Confirmed at gix 0.83 `repository::identity::committer() -> Option<Result<SignatureRef<'_>, _>>`. Borrows from `&self`. The plan's `.expect(...).map(|c| c.into())` chain may need adjustment but is bounded; executor diagnoses on `cargo check`. |
| Q3 | `stateless_connect.rs` advertisement widening | YES (zero-line) | Confirmed: `send_advertisement` calls `git upload-pack --advertise-refs` against `cache.repo_path()`. `transfer.hideRefs` only hides `refs/reposix/sync/`. Mirror refs propagate automatically. Plan's "investigate first" stance is correct. |
| Q4 | cache.db path layout | NO (M2) | Integration test at line 1967 has `let db_path = cache_bare.join("..").join("cache.db");` — but `db.rs:35–37` puts cache.db INSIDE cache_dir (which IS the bare repo dir). The `..` is wrong. Easy fix; should be `cache_bare.join("cache.db")`. Listed as M-severity below. |
| Q5 | reject-hint integration test shape (weaker form) | NO (HIGH H3) | The "weaker form" asserts nothing meaningful (see issue H3). |
| Q6 | verifier shell two-WORK-tree scenario | YES | Brittleness is real; the plan flags it and proposes a relocation if needed. The hint-text contract (stderr cites refs/mirrors/sim-synced-at + RFC3339 + N minutes ago) is the load-bearing assertion and is testable in either shape. |

### 9. Per-phase push cadence

**YES.** T04 ends with `git push origin main` BEFORE the verifier-subagent dispatch. The dispatch is correctly identified as orchestrator-level (top-level coordinator action AFTER 80-01 T04 pushes), NOT a plan task. Pre-push GREEN is part of phase-close criterion. `--no-verify` and `--amend` correctly forbidden.

### 10. Phase-shape + dependency chain

**SOUND.** T01 → T02 → T03 → T04 strictly sequential.

- T01 (catalog-first): doc-only; produces verifier shells that FAIL until T02–T04 land. The shells reference test files not yet created — acceptable per QG-06 catalog-first contract.
- T02 (cache impl): depends on nothing (T01 doc-only). Cargo-locked on `reposix-cache`.
- T03 (helper wiring): requires T02's `Cache::write_mirror_head` / `write_mirror_synced_at` / `read_mirror_synced_at` / `refresh_for_mirror_head` / `log_mirror_sync_written` APIs. Cargo-locked on `reposix-remote`. T02 must be on disk first — correctly serialized.
- T04 (integration tests + flip + close): requires T03 wiring + T02 unit tests pass. Cargo-locked on `reposix-remote --tests`. Sequential after T03.

No circular dependencies, no skipped-prerequisite relations. Wave plan correctly identifies a single wave.

---

## Issues by severity

### HIGH

#### H1 — `reposix init` does NOT honor `REPOSIX_SIM_ORIGIN`; verifiers + integration tests cannot reach a non-default-port sim

**Where:** All 3 verifier shells (T01) at the `${CLI_BIN} init "sim::demo" "${WORK}"` line + the integration-test `drive_init_edit_push` helper (T04) at `Command::new(env!("CARGO_BIN_EXE_reposix")).args(["init", "sim::demo", &work_dir.to_string_lossy()]).env("REPOSIX_SIM_ORIGIN", ...)`.

**Root cause:** `crates/reposix-cli/src/init.rs:23–55` hardcodes `DEFAULT_SIM_ORIGIN = "http://127.0.0.1:7878"` and DOES NOT read `REPOSIX_SIM_ORIGIN`. The translated remote URL stored in `git config remote.origin.url` is `reposix::http://127.0.0.1:7878/projects/demo`. When the verifier then runs `git push origin main`, the helper extracts the origin from the URL (port 7878) — the env var has zero effect.

The plan ASSUMED `REPOSIX_SIM_ORIGIN` worked symmetrically because P79's `reposix attach` verifier honors it. But `attach.rs:155` reads the env var explicitly, and `init.rs` does not. This is a real divergence in CLI surface.

**Evidence:** `crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path` explicitly patches around the same issue with a comment: *"The trailing `git fetch` against the default sim port (7878) will fail because we ran the sim on a different port — that's fine. … We re-point the URL to our test sim below for any subsequent commands."* — followed by `git config remote.origin.url <our-test-port-url>`.

**Impact:** All 3 verifier shells will hang/fail at the `git push` step. All 4 integration tests will fail. SC1, SC2, SC3 catalog rows cannot flip to PASS.

**Fix (recommended):** Insert a `git config remote.origin.url` re-pointing step in each verifier shell after `reposix init`, AND in the `drive_init_edit_push` helper. Pattern:

```bash
git -C "${WORK}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
```

Then the env-var passthrough on `git push` is unnecessary (URL carries the port).

**Alternative (not recommended for v0.13.0 P80):** add a Wave 0 task that extends `init.rs` to honor `REPOSIX_SIM_ORIGIN`. Out of scope for this phase; file as GOOD-TO-HAVE.

#### H2 — `gix::Repository::tag(...)` invocation has wrong argument order in plan code sample

**Where:** Plan lines 1132–1142 (T02 `write_mirror_synced_at` body):

```rust
self.repo.tag(
    format!("{sot_host}-synced-at"),
    target,
    gix::refs::transaction::PreviousValue::Any,   // slot 3
    Some(self.repo.committer().expect(...).map(|c| c.into())),  // slot 4
    &message,
    true,  // slot 6 — bool
)
```

**Root cause:** gix 0.83 signature (verified at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gix-0.83.0/src/repository/object.rs:338–346`):

```rust
pub fn tag(
    &self,
    name: impl AsRef<str>,
    target: impl AsRef<gix_hash::oid>,
    target_kind: gix_object::Kind,                          // slot 3 — NOT PreviousValue
    tagger: Option<gix_actor::SignatureRef<'_>>,            // slot 4 — SignatureRef, not Option<Result<>>
    message: impl AsRef<str>,
    constraint: PreviousValue,                              // slot 6 — PreviousValue, NOT bool
) -> Result<Reference<'_>, tag::Error>
```

The plan's call won't compile: slot 3 needs `gix_object::Kind::Commit`, slot 6 needs `PreviousValue::Any` (instead of `true`), and slot 4 has a `Some(Result<...>.map(...))` chain that doesn't unwrap to a plain `Option<SignatureRef<'_>>`.

**Impact:** T02's `cargo check -p reposix-cache` will fail. The plan's "API verification step" (lines 1331–1352) anticipates an API-name divergence and prescribes a fallback path — but the failure here is shape/argument-count, not name. The fallback writes-tag-object-then-edit-ref path is more involved than the plan estimates and benefits from being prescribed up front.

**Fix (recommended):** Replace the plan's lines 1132–1142 with the correct invocation:

```rust
let tagger_owned: Option<gix_actor::Signature> = self.repo.committer()
    .and_then(|r| r.ok())
    .and_then(|sig_ref| gix_actor::Signature::try_from(sig_ref).ok());
let tagger_ref: Option<gix_actor::SignatureRef<'_>> = tagger_owned.as_ref().map(|s| s.to_ref());
let _ref = self.repo.tag(
    format!("{sot_host}-synced-at"),
    target,
    gix::object::Kind::Commit,
    tagger_ref,
    &message,
    PreviousValue::Any,
)
.map_err(|e| Error::Git(format!("write annotated tag {ref_name}: {e}")))?;
```

(Lifetime-management on `tagger_ref` may need adjustment — `committer()` returns a borrow tied to `&self`. If the borrow shape is awkward, fall through to the two-step path: `repo.write_object(&Tag { ... })` + `repo.tag_reference(name, tag_id, PreviousValue::Any)`.)

#### H3 — `reject_hint_first_push_omits_synced_at_line` integration test asserts nothing meaningful

**Where:** Plan lines 2028–2063 (T04 integration test).

**Root cause:** The test drives a successful single-backend push (no conflict), then asserts `!stderr.contains("minutes ago")`. The "minutes ago" hint is composed ONLY in the conflict-reject branch (`handle_export` lines 384–407). A successful push never enters that branch, so the assertion is vacuously true — it would pass even if `read_mirror_synced_at` returned garbage or the helper wrote a bogus hint string. SC4's first-push None-case behavior at the helper layer is not tested.

The plan acknowledges this ("This is a weaker assertion than the first-push case, but the strong assertion (None case) is covered by … `read_mirror_synced_at_returns_none_when_absent` unit test"). The unit test covers the cache-layer return value but NOT the helper-layer omission of the hint stderr line.

**Impact:** A regression where the helper crashes or writes "synced at None ago" on the first-push conflict path would not be caught by P80's tests. SC4 coverage is partial.

**Fix (recommended):** Engineer a real first-push conflict by seeding the sim with a record at `version: 2` BEFORE the working tree's first push (which uses `version: 1` as the prior). The conflict-reject path fires; the test asserts:

```rust
let stderr = String::from_utf8_lossy(&push_out.stderr);
assert!(stderr.contains("fetch first") || stderr.contains("modified on backend"),
    "expected conflict-reject stderr; got: {stderr}");
assert!(!stderr.contains("synced at"), "first-push conflict stderr should not contain synced-at hint: {stderr}");
assert!(!stderr.contains("minutes ago"), "first-push conflict stderr should not contain ago rendering: {stderr}");
```

If sim seeding is too brittle within T04 scope, an alternative is to write a unit test in `crates/reposix-remote/src/main.rs` that exercises the conflict-reject branch directly with a stubbed cache returning `Ok(None)` from `read_mirror_synced_at` — bypassing the sim entirely. This is the path the plan flagged ("if the sim-mutation path proves brittle, move this assertion to a unit test in `crates/reposix-remote/src/main.rs`"). Either path is acceptable; the current "weak form" is not.

### MEDIUM

#### M1 — `walkdir` and `regex` are missing from `crates/reposix-remote/Cargo.toml [dev-dependencies]`

**Where:** Integration test (T04) uses `walkdir::WalkDir` (line 1928) and `regex::Regex` (lines 2022, 2024). Confirmed at `crates/reposix-remote/Cargo.toml`: dev-dependencies are `wiremock`, `assert_cmd`, `tempfile`, `tokio`, `chrono`, `reposix-sim` — no `walkdir`, no `regex`.

**Plan flags it ("may need to be added") but does not include a concrete `cargo add` step** in T04 action. The executor will discover this on first `cargo check -p reposix-remote --tests`.

**Impact:** Trivial fix-on-first-build, but adds an unnecessary diagnostic round-trip. Worth pinning explicitly.

**Fix:** Add a concrete sub-step to T04: `cargo add --dev --package reposix-remote walkdir regex` BEFORE the integration test commit. Pin versions to workspace defaults if a `[workspace.dependencies]` entry exists.

#### M2 — Integration test `cache.db` path is wrong

**Where:** Plan line 1967: `let db_path = cache_bare.join("..").join("cache.db");` with comment "adjust per actual layout".

**Root cause:** `crates/reposix-cache/src/db.rs:37` puts `cache.db` INSIDE `cache_dir`, and `cache_dir` IS the bare-repo dir (per `cache.rs:115–117` "cache.db lives inside the bare repo dir so a single path scheme covers both git state and cache state"). The correct path is `cache_bare.join("cache.db")` — no `..`.

**Impact:** Test `write_on_success_updates_both_refs` will fail at `rusqlite::Connection::open` with "file not found"; executor diagnoses on first run.

**Fix:** Change line 1967 to `let db_path = cache_bare.join("cache.db");`.

#### M3 — Q2.2 verbatim phrase carrier is ambiguous re: ROADMAP SC4

**Where:** ROADMAP SC4 reads: *"reject messages cite the refs in hints — verbatim form per Q2.2 doc clarity contract: 'refs/mirrors/confluence-synced-at is the timestamp the mirror last caught up to confluence — it is NOT a current SoT state marker'"*.

The plan reads this as a doc-clarity contract (CLAUDE.md gets the verbatim text in T04 at lines 2167–2171). The reject hint stderr cites the ref by name + "(N minutes ago)" but does NOT include the verbatim Q2.2 phrase.

**Plan-internal evidence for the doc-clarity reading:** decisions.md Q2.2 says *"Document loudly in `docs/concepts/dvcs-topology.md` and `docs/guides/dvcs-mirror-setup.md`: '...'"* — i.e., the clarification is a docs target, not a stderr target. The plan's CLAUDE.md update is the v0.13.0 P80 carrier; full docs treatment defers to P85.

**Counter-evidence:** ROADMAP SC4's wording reads as if the clarification phrase is the verbatim form for the reject message itself. A strict-reading verifier subagent might grade this RED.

**Impact:** Verifier risk. The plan's reading is defensible but should be made explicit so the verifier doesn't ambiguity-grade it.

**Fix (recommended):** Add a one-line note to `<canonical_refs>` or `<must_haves>` in 80-01-PLAN.md: *"Q2.2 verbatim phrase ('refs/mirrors/...synced-at is the timestamp the mirror last caught up to ... — NOT a current SoT state marker') lives in CLAUDE.md (T04 epilogue) per the doc-clarity contract; reject stderr cites the ref name + age rendering only. Decision rationale: decisions.md Q2.2 names docs/concepts and docs/guides as targets, not stderr."*

Alternatively, broaden the reject-hint stderr to include the Q2.2 verbatim phrase as a third `diag(...)` line. ~5 additional lines; bounded; eliminates the verifier-grading risk.

#### M4 — Verifier shells use fixed ports (7900–7902) → parallel test-runner collision risk

**Where:** Plan lines 563, 636, 699 (3 verifier shells with `PORT=7900/7901/7902`).

**Root cause:** Fixed ports collide if any other test or system service uses them. The Rust integration tests use `pick_free_port()` (good); verifier shells should too.

**Impact:** Flaky CI when port-7900 is occupied (e.g. another test, dev server). The catalog runner runs verifiers sequentially during `--cadence pre-pr`, so within-runner collision is avoided — but cross-runner (parallel CI workers) collision is not.

**Fix:** Replace fixed PORT assignments with random-port logic mirroring `quality/gates/agent-ux/reposix-attach.sh`'s pattern (if it uses `pick_free_port`-equivalent). Cheaper alternative: pick a high port range less likely to collide (e.g. 49152–65535) and document.

#### M5 — Plan-OVERVIEW says `mirror-refs-cited-in-reject-hint.sh` is "TINY ~70-line", commit message also says ~70-line — actual plan body is ~75 lines + a substantive scenario design — borderline TINY, OK

**Where:** Plan-OVERVIEW says verifier is TINY ~30-50 lines; T01 commit message claims 70 lines; actual scenario for the reject hint requires two work trees + sequential pushes.

**Impact:** Minor — verifier stays TINY by intent ("≤ 60 lines" budget). Watch for scope creep during implementation; if the scenario logic exceeds 80 lines, escalate as SURPRISES-INTAKE candidate.

**Fix:** Cap at 80 lines in the verifier file. If exceeded, prefer relocating logic to a Rust integration test and keeping the shell as a thin shim.

### LOW

#### L1 — Plan duplicates content between `<objective>` (80-01-PLAN.md) and § "Subtle architectural points S1, S2" (80-PLAN-OVERVIEW.md)

Trim the OVERVIEW S1/S2 sections to a 2-line summary or trim the PLAN's objective preamble; either path saves ~80 lines of executor-context overhead.

#### L2 — `cache.refresh_for_mirror_head()` introduces a thin wrapper around `Cache::build_from()` whose only purpose is naming

The plan justifies this as "names the call site for the helper's mirror-head wiring; makes the call site grep-discoverable and the cost commentary targeted." Defensible but adds API surface for stylistic reasons. Consider whether a helper-internal `// SoT SHA via build_from()` comment achieves the same goal at zero API cost. Not blocking.

#### L3 — `handle_export` success branch wiring uses `state.rt.block_on(cache.refresh_for_mirror_head())` — second `block_on` in the same handler

Cargo lock + tokio runtime allocation: each `block_on` on the same `state.rt` is fine. Not a concern in practice. Worth noting in case the runtime is single-threaded and a future refactor reuses one `block_on` site.

#### L4 — Reflog pruning deferred to v0.14.0 — accept; documented in module doc

OK. The "one-line note in `mirror_refs.rs` module-doc citing the deferral target" is in the plan body lines 974–981; correctly delegated.

#### L5 — Pre-existing `transfer.hideRefs` only hides `refs/reposix/sync/` — mirror refs propagate by default. Plan correctly identifies this.

OK. No fix needed.

---

## Recommendation

**Verdict: YELLOW. Minor revisions recommended before T01 lands.**

Three HIGH issues are blocking but each has a bounded fix:

- **H1 (sim-port routing)** — add `git config remote.origin.url` re-pointing in 3 verifier shells + 1 Rust helper. Affects ~6 lines of plan body.
- **H2 (gix tag API shape)** — replace lines 1132–1142 with the correct invocation OR commit to the two-step `write_object` + `tag_reference` path up front. Affects ~15 lines.
- **H3 (vacuous integration assertion)** — engineer a real first-push conflict via sim seeding, OR move the assertion to a unit test in `main.rs` with a stub cache. Affects ~30 lines.

Combined revision footprint: ~50 plan lines. Estimated one revision loop (≤ 30 min planner time) before execution.

Medium issues (M1–M5) can be fixed inline during execution as eager-resolution items per OP-8 (each is < 1 hour, no new dep introduced) — they don't block T01.

Low issues (L1–L5) are stylistic / observational — no action required.

If the planner declines to revise H1–H3 before execution, the executor will (a) hit H2 on first `cargo check -p reposix-cache` and fix-forward, (b) hit H1 on first verifier-shell run and fix-forward, (c) miss H3 entirely (the test passes but proves nothing) — surfaced only at verifier-subagent grading time, where it may be downgraded to PASS-with-caveat or graded YELLOW.

---

**End of PLAN-CHECK.md**
