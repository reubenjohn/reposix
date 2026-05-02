# Issues by severity

← [back to index](./index.md)

## HIGH

### H1 — `reposix init` does NOT honor `REPOSIX_SIM_ORIGIN`; verifiers + integration tests cannot reach a non-default-port sim

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

### H2 — `gix::Repository::tag(...)` invocation has wrong argument order in plan code sample

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

### H3 — `reject_hint_first_push_omits_synced_at_line` integration test asserts nothing meaningful

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

## MEDIUM

### M1 — `walkdir` and `regex` are missing from `crates/reposix-remote/Cargo.toml [dev-dependencies]`

**Where:** Integration test (T04) uses `walkdir::WalkDir` (line 1928) and `regex::Regex` (lines 2022, 2024). Confirmed at `crates/reposix-remote/Cargo.toml`: dev-dependencies are `wiremock`, `assert_cmd`, `tempfile`, `tokio`, `chrono`, `reposix-sim` — no `walkdir`, no `regex`.

**Plan flags it ("may need to be added") but does not include a concrete `cargo add` step** in T04 action. The executor will discover this on first `cargo check -p reposix-remote --tests`.

**Impact:** Trivial fix-on-first-build, but adds an unnecessary diagnostic round-trip. Worth pinning explicitly.

**Fix:** Add a concrete sub-step to T04: `cargo add --dev --package reposix-remote walkdir regex` BEFORE the integration test commit. Pin versions to workspace defaults if a `[workspace.dependencies]` entry exists.

### M2 — Integration test `cache.db` path is wrong

**Where:** Plan line 1967: `let db_path = cache_bare.join("..").join("cache.db");` with comment "adjust per actual layout".

**Root cause:** `crates/reposix-cache/src/db.rs:37` puts `cache.db` INSIDE `cache_dir`, and `cache_dir` IS the bare-repo dir (per `cache.rs:115–117` "cache.db lives inside the bare repo dir so a single path scheme covers both git state and cache state"). The correct path is `cache_bare.join("cache.db")` — no `..`.

**Impact:** Test `write_on_success_updates_both_refs` will fail at `rusqlite::Connection::open` with "file not found"; executor diagnoses on first run.

**Fix:** Change line 1967 to `let db_path = cache_bare.join("cache.db");`.

### M3 — Q2.2 verbatim phrase carrier is ambiguous re: ROADMAP SC4

**Where:** ROADMAP SC4 reads: *"reject messages cite the refs in hints — verbatim form per Q2.2 doc clarity contract: 'refs/mirrors/confluence-synced-at is the timestamp the mirror last caught up to confluence — it is NOT a current SoT state marker'"*.

The plan reads this as a doc-clarity contract (CLAUDE.md gets the verbatim text in T04 at lines 2167–2171). The reject hint stderr cites the ref by name + "(N minutes ago)" but does NOT include the verbatim Q2.2 phrase.

**Plan-internal evidence for the doc-clarity reading:** decisions.md Q2.2 says *"Document loudly in `docs/concepts/dvcs-topology.md` and `docs/guides/dvcs-mirror-setup.md`: '...'"* — i.e., the clarification is a docs target, not a stderr target. The plan's CLAUDE.md update is the v0.13.0 P80 carrier; full docs treatment defers to P85.

**Counter-evidence:** ROADMAP SC4's wording reads as if the clarification phrase is the verbatim form for the reject message itself. A strict-reading verifier subagent might grade this RED.

**Impact:** Verifier risk. The plan's reading is defensible but should be made explicit so the verifier doesn't ambiguity-grade it.

**Fix (recommended):** Add a one-line note to `<canonical_refs>` or `<must_haves>` in 80-01-PLAN.md: *"Q2.2 verbatim phrase ('refs/mirrors/...synced-at is the timestamp the mirror last caught up to ... — NOT a current SoT state marker') lives in CLAUDE.md (T04 epilogue) per the doc-clarity contract; reject stderr cites the ref name + age rendering only. Decision rationale: decisions.md Q2.2 names docs/concepts and docs/guides as targets, not stderr."*

Alternatively, broaden the reject-hint stderr to include the Q2.2 verbatim phrase as a third `diag(...)` line. ~5 additional lines; bounded; eliminates the verifier-grading risk.

### M4 — Verifier shells use fixed ports (7900–7902) → parallel test-runner collision risk

**Where:** Plan lines 563, 636, 699 (3 verifier shells with `PORT=7900/7901/7902`).

**Root cause:** Fixed ports collide if any other test or system service uses them. The Rust integration tests use `pick_free_port()` (good); verifier shells should too.

**Impact:** Flaky CI when port-7900 is occupied (e.g. another test, dev server). The catalog runner runs verifiers sequentially during `--cadence pre-pr`, so within-runner collision is avoided — but cross-runner (parallel CI workers) collision is not.

**Fix:** Replace fixed PORT assignments with random-port logic mirroring `quality/gates/agent-ux/reposix-attach.sh`'s pattern (if it uses `pick_free_port`-equivalent). Cheaper alternative: pick a high port range less likely to collide (e.g. 49152–65535) and document.

### M5 — Plan-OVERVIEW says `mirror-refs-cited-in-reject-hint.sh` is "TINY ~70-line", commit message also says ~70-line — actual plan body is ~75 lines + a substantive scenario design — borderline TINY, OK

**Where:** Plan-OVERVIEW says verifier is TINY ~30-50 lines; T01 commit message claims 70 lines; actual scenario for the reject hint requires two work trees + sequential pushes.

**Impact:** Minor — verifier stays TINY by intent ("≤ 60 lines" budget). Watch for scope creep during implementation; if the scenario logic exceeds 80 lines, escalate as SURPRISES-INTAKE candidate.

**Fix:** Cap at 80 lines in the verifier file. If exceeded, prefer relocating logic to a Rust integration test and keeping the shell as a thin shim.

## LOW

### L1 — Plan duplicates content between `<objective>` (80-01-PLAN.md) and § "Subtle architectural points S1, S2" (80-PLAN-OVERVIEW.md)

Trim the OVERVIEW S1/S2 sections to a 2-line summary or trim the PLAN's objective preamble; either path saves ~80 lines of executor-context overhead.

### L2 — `cache.refresh_for_mirror_head()` introduces a thin wrapper around `Cache::build_from()` whose only purpose is naming

The plan justifies this as "names the call site for the helper's mirror-head wiring; makes the call site grep-discoverable and the cost commentary targeted." Defensible but adds API surface for stylistic reasons. Consider whether a helper-internal `// SoT SHA via build_from()` comment achieves the same goal at zero API cost. Not blocking.

### L3 — `handle_export` success branch wiring uses `state.rt.block_on(cache.refresh_for_mirror_head())` — second `block_on` in the same handler

Cargo lock + tokio runtime allocation: each `block_on` on the same `state.rt` is fine. Not a concern in practice. Worth noting in case the runtime is single-threaded and a future refactor reuses one `block_on` site.

### L4 — Reflog pruning deferred to v0.14.0 — accept; documented in module doc

OK. The "one-line note in `mirror_refs.rs` module-doc citing the deferral target" is in the plan body lines 974–981; correctly delegated.

### L5 — Pre-existing `transfer.hideRefs` only hides `refs/reposix/sync/` — mirror refs propagate by default. Plan correctly identifies this.

OK. No fix needed.
