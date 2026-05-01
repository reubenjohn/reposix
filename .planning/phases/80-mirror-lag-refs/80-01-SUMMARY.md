# 80-01 Plan Summary — Mirror-lag refs (`refs/mirrors/<sot-host>-{head,synced-at}`)

Single-plan phase, 4 sequential tasks, 4 atomic commits, GREEN verdict at `quality/reports/verdicts/p80/VERDICT.md`.

## Tasks shipped (4/4)

**T01 — Catalog-first.** Three `agent-ux` catalog rows minted in `quality/catalogs/agent-ux.json` with `status: FAIL` BEFORE any Rust landed, plus three TINY verifier shells under `quality/gates/agent-ux/`:

- `agent-ux/mirror-refs-write-on-success` → `quality/gates/agent-ux/mirror-refs-write-on-success.sh`
- `agent-ux/mirror-refs-readable-by-vanilla-fetch` → `quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh`
- `agent-ux/mirror-refs-cited-in-reject-hint` → `quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh`

**T02 — Cache crate (`crates/reposix-cache/`).** New module `mirror_refs.rs` (~374 lines) modeled on the donor pattern `sync_tag.rs`. Five unit tests cover ref-name formatters, RFC3339 parse round-trip, and gix-validation surface. `audit::log_mirror_sync_written` extends the `audit_events_cache` schema CHECK list with `'mirror_sync_written'` (eager-resolution: discovered missing during T04 integration test; folded into T02 retroactively). `lib.rs` re-exports `write_mirror_head`, `write_mirror_synced_at`, `read_mirror_synced_at`, `refresh_for_mirror_head`, plus the audit helper.

**T03 — Helper wiring (`crates/reposix-remote/src/main.rs::handle_export`).** Success branch (lines ~470-489) writes both refs (best-effort `tracing::warn!`) AND unconditionally writes the `audit_events_cache` row with `op = 'mirror_sync_written'` (OP-3 invariant). Reject branch (lines ~384-407) reads the current `synced-at` timestamp via `read_mirror_synced_at` and composes a `(N minutes ago)` rendering when present; first-push case omits the hint.

**T04 — Integration tests + CLAUDE.md + close.** Four integration tests in `crates/reposix-remote/tests/mirror_refs.rs`:

- `write_on_success_updates_both_refs` — happy-path round-trip; asserts both refs land + `mirror_sync_written` audit row count == 1.
- `vanilla_fetch_brings_mirror_refs` — uses `git clone --mirror` (NOT `--bare`; corrected in-task) to verify dark-factory contract; vanilla `git fetch` propagates `refs/mirrors/*` via the helper's `stateless-connect` advertisement.
- `reject_hint_first_push_omits_synced_at_line` — non-vacuous H3 fix: wiremock seeded with `version=5` while inbound declares `prior_version=3` forces real conflict-reject; asserts (a) "fetch first" present, (b) "synced at" / "minutes ago" absent.
- `reject_hint_after_sync_cites_age` — second push after a first-push success; asserts stderr cites the synced-at age in human form.

CLAUDE.md updated: Architecture paragraph documents the namespace, the Q2.2 verbatim doc-clarity contract, and the OP-3 audit-table extension. The 3 catalog rows flip FAIL → PASS via the runner's catalog walk on pre-push (NOT hand-edit).

## Commits

| SHA | Subject |
|---|---|
| `6711e59` | quality(agent-ux): mint mirror-refs catalog rows + 3 TINY verifiers (P80-01 T01) |
| `a1c29e5` | feat(cache): mirror_refs module + log_mirror_sync_written audit helper (P80-01 T02) |
| `bf0fe95` | feat(remote): wire mirror-lag refs into handle_export success + conflict-reject paths (P80-01 T03) |
| `d50533d` | test(remote): integration tests + verifier flip + CLAUDE.md update + schema migration (P80-01 T04) |

## Q1–Q6 path resolutions (planner-flagged at plan time)

- **Q1 / H2 (gix `Repository::tag` API).** PRIMARY PATH UNUSABLE — `Repository::tag(name, ...)` and `tag_reference(...)` hardcode the `refs/tags/` prefix and cannot write to `refs/mirrors/...`. Used the **two-step pattern**: `gix::Repository::write_object(&gix::objs::Tag { ... })` → `gix::Repository::reference(name, tag_id, PreviousValue::Any, log_msg)`. Preserves the FullName + annotated-tag contract.
- **Q2 (committer accessor).** `committer()` returns `Option<Result<SignatureRef<'_>, _>>`; chain `.and_then(Result::ok).and_then(|sr| sr.to_owned().ok())` materializes the owned `gix::actor::Signature` for the Tag struct.
- **Q3 (stateless-connect advertisement widening).** ZERO LINES. `send_advertisement` invokes `git upload-pack --advertise-refs` against `cache.repo_path()`; non-hidden refs propagate by default; `transfer.hideRefs` only hides `refs/reposix/sync/`.
- **Q4 (cache.db path).** Inside `cache_dir` (NOT `..`); `cache_bare.join("cache.db")` resolves correctly per `db.rs:35-37` + `cache.rs:115-117`.
- **Q5 / H3 (reject-hint test path).** Path A (sim-seeded conflict via wiremock at version=5) succeeded — non-vacuous. Path B (unit-test stub) not exercised.
- **Q6 (verifier-shell brittleness).** Shape change applied: shells rewired as thin wrappers around the integration tests (`cargo test -p reposix-remote --test mirror_refs <name>`). Same artifact contract (exit 0 + named stdout asserts), more deterministic than the original `reposix init` + `git fetch` cycle. Filed a SURPRISES-INTAKE entry to journal the deviation per the verifier's advisory.

## In-phase deviations (eager-resolution per OP-8)

1. `audit_events_cache` CHECK constraint did not include `'mirror_sync_written'`. T04 discovery; folded back into T02's schema migration commit.
2. Plan unit-test signature `Cache::open(tmp.path(), "sim", "demo")` did not match the actual `Cache::open(backend: Arc<dyn BackendConnector>, ...)`. Pure unit tests in `mirror_refs.rs` cover what is testable without a backend; round-trips moved to integration tests where wiremock is available.
3. `git clone --bare` does NOT propagate `refs/mirrors/*` (only `refs/heads/*` + `refs/tags/*`); test 2 uses `--mirror` per the dark-factory contract.
4. `git log <annotated-tag> -1 --format=%B` peels the tag to its commit; tests use `git cat-file -p <tagref>` and parse the post-`\n\n` payload to read the tag-object body.

## Acceptance

- All 3 DVCS-MIRROR-REFS-* requirements shipped, observable test coverage, GREEN verdict by unbiased subagent.
- Catalog rows at PASS; `python3 quality/runners/run.py --cadence pre-push` shows 26 PASS / 0 FAIL / 0 WAIVED at phase close.
- CLAUDE.md updated in-phase (QG-07).
- One SURPRISES-INTAKE entry filed for the verifier-shape change (T04 advisory).
