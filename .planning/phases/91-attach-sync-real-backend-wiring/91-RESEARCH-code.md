# P91 Code-Surface Research — Attach/Sync Real-Backend Wiring + QL-001 Push-Path Fix

**Researched:** 2026-07-04
**Domain:** Rust / git-remote-helper / REST backend dispatch / fast-import-export protocol
**Confidence:** HIGH (every claim below is file:line-verified against `main`@`32ba856`, clean tree, git 2.25.1)

## Summary

Both lanes are well-scoped and the ground truth mostly matches the SURPRISES-INTAKE/raise-list
prose, with a few corrections and one major discovery: **the LANE 1 regression harness already
exists and is fully wired** — `quality/gates/agent-ux/real-git-push-e2e.sh` + its catalog row
`agent-ux/real-git-push-e2e` (WAIVED until 2026-07-31) already drive a real `git commit`+`git push`
against a seeded sim and assert PATCH-count/POST-count/DELETE-count via `sqlite3` against
`audit_events`. P91 does not need to build this harness — it needs to make the existing script's
assertions pass, and retire the waiver. The one hard blocker: **this box's git is 2.25.1**, and the
script's own git-version gate short-circuits to exit 75 (NOT-VERIFIED) below that, meaning the fix
cannot be regression-tested end-to-end via `git checkout -B main refs/reposix/origin/main` on this
box — Ubuntu Focal's apt has no newer git candidate. The planner must decide between (a) building
git ≥2.34 from source/PPA on this box, or (b) a synthetic-tree-construction fallback test (as the
BLOCKER repro itself used) that drives `diff::plan`/`fast_import::parse_export_stream` directly
without the stateless-connect fetch, keeping `real-git-push-e2e.sh` as the aspirational/CI-only
full-stack check.

For LANE 2, the shared dispatch logic P91 should reuse **already exists** at
`crates/reposix-remote/src/backend_dispatch.rs` (private module of the `git-remote-reposix` binary
crate, which has no `[lib]` target). `attach.rs`/`sync.rs` currently duplicate a much thinner
sim-only version of the same match statement. Wiring real backends into attach/sync by copy-pasting
`backend_dispatch.rs`'s logic a third and fourth time is the exact anti-ROI outcome the mission
warns about — recommend exposing `reposix-remote` as a lib+bin crate (add `src/lib.rs` with
`pub mod backend_dispatch;`, thin `main.rs` down to call it) and having `reposix-cli` depend on it,
so `attach.rs`/`sync.rs` call `reposix_remote::backend_dispatch::{parse_remote_url, instantiate}`
directly — inheriting the OP-3 `with_audit` wiring for Confluence/JIRA for free.

**Primary recommendation:** Lane 1 — introduce `reposix_core::path::record_path(id)` /
`record_id_from_path(path)` as the single canonical `issues/<id>.md`-unpadded pair, route
`builder.rs`, `refresh.rs`, `fast_import.rs`, `diff.rs`, `main.rs`, `precheck.rs` through it, fix
the `parse_export_stream` trailing-LF peek and the M/D tree-removal, re-key every fixture the raise
list names, and then flip `agent-ux/real-git-push-e2e` from WAIVED to real (accepting the git-2.34
environment gap as a documented residual risk). Lane 2 — expose `backend_dispatch.rs` as a shared
lib module and delegate `attach.rs`/`sync.rs` to it instead of writing a third/fourth dispatch copy.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Push-path shape canonicalization (record_path helper) | API / Backend (reposix-core) | — | Shared pure function; every consumer (cache, cli, remote) is a backend-tier crate |
| Fast-export stream parsing (peek-LF, M/D reconciliation) | API / Backend (reposix-remote) | — | Protocol-layer parsing, not UI |
| Diff planner (`plan()`, bulk-delete cap) | API / Backend (reposix-remote) | — | Business logic over Records |
| `reposix attach` real-backend dispatch | API / Backend (reposix-cli + reposix-remote lib) | — | Credential plumbing + REST connector construction |
| `reposix sync --reconcile` real-backend dispatch | API / Backend (reposix-cli + reposix-remote lib) | — | Same tier as attach |
| Reconciliation walk (5-case matching) | API / Backend (reposix-cache) | Database / Storage (`cache_reconciliation` table) | Cache-internal state machine |
| OP-3 audit wiring (`with_audit`) | Database / Storage (SQLite `audit_events`) | API / Backend (connector construction) | Audit table lives in storage tier; wiring happens at connector-construction time in the backend tier |
| Real-git-push-e2e regression | CI / Test harness (shell + sqlite3, calls binary) | — | Drives the real binary as a black box |

## Lane 1 — QL-001 Push-Path Fix

### 1.1 Verified line-number inventory (drift-corrected against 32ba856)

| Site | Shape produced/consumed | Verified location |
|------|--------------------------|--------------------|
| Cache tree builder (production read path, stateless-connect serves this) | `issues/<id>.md` unpadded, `issues/` subtree entry | `crates/reposix-cache/src/builder.rs:90` (`format!("{}.md", issue.id.0)`), tree nesting at `:131-138` (outer tree entry named `"issues"` wraps inner tree — the mission's `:135` citation for the outer-entry line is a 3-4-line drift; content and intent match) |
| `reposix refresh` CLI write path | `issues/00000000042.md` (11-zero-pad) | `crates/reposix-cli/src/refresh.rs:120` (`format!("{:011}.md", issue.id.0)`); bucket dir resolution at `:102-105` (`"issues"` for Sim/Github/Jira, `"pages"` for Confluence) |
| `fast_import` emit (deprecated `import` fallback, git<2.34) | `0042.md` bare 4-pad, no prefix | `crates/reposix-remote/src/fast_import.rs:63` (`M 100644 :{} {:04}.md`) |
| Push diff planner prior-key | `0042.md` bare 4-pad | `crates/reposix-remote/src/diff.rs:106` (`format!("{:04}.md", i.id.0)`) — cited line matches exactly |
| `issue_id_from_path` (diff.rs) | parses bare `<digits>.md` only, `strip_suffix` then `parse::<u64>` | `crates/reposix-remote/src/diff.rs:74-77` — matches cited lines exactly |
| `issue_id_from_path` duplicate (main.rs, QL-157) | identical body | `crates/reposix-remote/src/main.rs:433-436` (mission cited `:432-435`; actual is 433-436, one-line drift — same body verbatim) |
| Conflict precheck silent skip on real paths | `let Some(id_num) = issue_id_from_path(path) else { continue; }` | `crates/reposix-remote/src/precheck.rs:151-152` — matches cited lines exactly |
| `parse_export_stream` "data N" handler swallowing a line unconditionally | `let _ = r.read_line(&mut maybe_nl);` | `crates/reposix-remote/src/fast_import.rs:156-157` — matches cited lines exactly (comment at :153-155 says "consume one [LF] if present" but the code always reads a full line) |
| `parse_export_stream` M-insert never removed on later D | `out.tree.insert(path.to_owned(), mark)` at `:181`; `out.deletes.push(rest.to_owned())` at `:185` — no code anywhere removes a path from `out.tree` when a `D` line for the same path is later seen | `crates/reposix-remote/src/fast_import.rs:173-186` |
| `plan()` — no `issues/*.md` path filter | `frontmatter::parse(&text)` runs on every path in `parsed.tree` unconditionally | `crates/reposix-remote/src/diff.rs:114-123` |

**Additional record-path production/consumption sites found by full-workspace grep (not enumerated
in the BLOCKER entry) that a canonical-path refactor must also touch:**

- `crates/reposix-core/src/path.rs:57-72` — `validate_record_filename(name)` **already exists** and
  already accepts arbitrary zero-padding for a bare `<digits>.md` filename (no `issues/` prefix
  handling — operates on the final path component only). **This is the correct foundation to build
  the canonical helper on** — extend it (or wrap it) with an `issues/` prefix strip/join rather than
  writing a fifth parser from scratch. Today NOTHING in `reposix-remote` calls into
  `reposix_core::path` at all — `diff.rs`'s and `main.rs`'s `issue_id_from_path` are two
  independent hand-rolled copies that ignore this existing, tested, core-crate function.
- `crates/reposix-core/src/path.rs:139-149` — `slug_or_fallback` documents "the 11-digit padding
  matches the existing `<padded-id>.md` convention in `pages/` and `issues/`" — this comment is
  itself evidence of the padding-convention confusion; if `refresh.rs` moves off 11-pad, this
  comment (and the `page-{:011}` fallback format used for Confluence pages) needs re-examination —
  though note Confluence pages use `pages/` bucket + slug-based names, NOT bare `<id>.md`, so this
  is lower urgency but should be double-checked by whoever lands the fix (searched: no other file
  reads `page-<11-digit>` format back out, so it's likely cosmetic-only, but verify before ruling
  it out of scope).
- `crates/reposix-cli/tests/attach.rs:353,393,495,500,612,757` — write `issues/0001.md`,
  `issues/0099.md`, `issues/dup-a.md`, `issues/dup-b.md` (via `write_record_md` helper) — these
  ALREADY use the `issues/<id>.md` unpadded shape (attach's reconciliation walker reads real
  working-tree files directly via `walkdir`, not through the planner) — these are NOT in the buggy
  shape and do not need re-keying; they're independent evidence the `issues/<id>.md` convention is
  already the de facto real-tree shape everywhere except the push planner.
- `crates/reposix-cache/tests/gix_api_smoke.rs:46` — writes `"issues/1.md"` directly via
  `gix::Tree` API — same as above, already-correct shape, no fix needed.
- `crates/reposix-remote/tests/refresh_integration.rs:125` (actually
  `crates/reposix-cli/tests/refresh_integration.rs:125`) references literal `"issues/"` as the
  bucket-dir string, not a full path — unaffected by the padding fix.

### 1.2 Test/fixture re-key inventory (confirms raise-list §3, with exact citations)

All of the following hand-build `parsed.tree`/fixtures in the buggy bare-4-pad shape and MUST be
re-keyed to `issues/<id>.md` (unpadded) so they go **honestly RED** until the fix lands, then green
after:

| File:line | Fixture | Note |
|-----------|---------|------|
| `crates/reposix-remote/src/diff.rs:284` | `tree.insert(format!("{:04}.md", issue.id.0), mark)` inside `unchanged_push_emits_no_patches` | masks BUG-1; test currently can never observe the real-path miss |
| `crates/reposix-remote/src/diff.rs:311` | `tree.insert("0001.md".to_owned(), 1)` inside `extra_trailing_newline_is_a_noop` | same masking |
| `crates/reposix-remote/src/diff.rs:233-264` | `five_deletes_passes_cap`/`six_deletes_fires_cap`/`six_deletes_with_allow_tag_passes` | these use `ParsedExport::default()` (empty new tree) so they don't touch path-shape directly, but the SG-02 cap is validated purely against `prior_by_path`'s bare-4-pad keys (`plan()` line 106) — cap logic itself is path-shape-agnostic, so these three specifically do NOT need re-keying (correction to raise-list: the cap tests build an empty `parsed.tree`, there's no path string in them to re-key) — but they DO indirectly depend on `prior_by_path`'s `format!("{:04}.md", …)` internal call, so once that internal format string changes to the canonical helper, these tests keep passing unmodified (good regression signal that the refactor didn't change cap semantics) |
| `crates/reposix-remote/tests/push_conflict.rs:154` | `one_file_export("0002.md", &blob, "edit issue 2\n")` | ARCH-08 stale-base regression — currently only fires because bare shape parses; must become `"issues/2.md"` |
| `crates/reposix-remote/tests/protocol.rs:67,85` | doc comment "mapped to path `0001.md`" + `writeln!(&mut out, "M 100644 :1 0001.md")` | core export-protocol fixture |
| `crates/reposix-remote/tests/bus_write_happy.rs:251` | `vec![("0001.md", blob1), ("0002.md", blob2), ("0003.md", blob3)]` | |
| `crates/reposix-remote/tests/bus_write_sot_fail.rs:245` | same shape | |
| `crates/reposix-remote/tests/bus_write_mirror_fail.rs:213` | same shape | |
| `crates/reposix-remote/tests/bus_write_post_precheck_409.rs:228` | same shape (also `:275` greps stderr for `"issue 1"` OR `"issues/1"` — already tolerant of both spellings) | |
| `crates/reposix-remote/tests/bus_write_audit_completeness.rs:214` | same shape | |
| `crates/reposix-remote/tests/perf_l1.rs:311` (comment) / the `no_op_tree_export(n, …)` helper it calls | comment says "0001.md in parsed.tree matching prior" — need to locate `no_op_tree_export`'s definition (likely in `common.rs` or inline in perf_l1.rs) and confirm its internal path-format string, since the comment references the shape but the actual `format!` call wasn't independently re-verified in this pass — **flag for the executor to grep `no_op_tree_export` before assuming it's fixed by re-keying only the cited line** | 

**Not previously enumerated, found this pass:** `crates/reposix-remote/tests/bulk_delete_cap.rs`
exists as its OWN file (separate from `diff.rs`'s in-module `five/six_deletes_*` tests) — the
executor should grep it for the same bare-shape pattern before considering the re-key inventory
closed; this pass did not have budget to open every line of every remote test file individually
beyond grepping the exact `"000N.md"` literal (which returned no hits in `bulk_delete_cap.rs`,
suggesting it may already use `ParsedExport::default()`-only fixtures like the cap tests above —
verify, don't assume).

### 1.3 `parse_export_stream` semantics + correct fix

Current logic (`fast_import.rs:90-192`):
- `in_commit` flag set true on `commit ` directive (`:170`), false on `blob` directive (`:132`).
- `data N` handler (`:147-164`): reads exactly N bytes, then **unconditionally** calls
  `r.read_line(&mut maybe_nl)` to "consume one LF if present" — this is wrong: `read_line` reads
  until the NEXT newline or EOF, i.e. it consumes an entire line, not "up to one byte". For a blob
  payload (which fast-export always follows with an actual blank line before the next directive),
  this happens to be harmless because the "line" being over-consumed IS just the LF. But for the
  **commit message** payload, git fast-export emits the message bytes immediately followed by the
  first `M` directive with NO separating blank line — so `read_line` swallows the entire first
  `M 100644 :N <path>` line as if it were the "optional trailing newline." This is BUG-3, and it is
  deterministic: it always eats the lowest-mark M-line right after the commit-message `data N`
  block, because that's structurally always the first M-line emitted.
- **Correct fix:** peek exactly one byte (not a line). If it's `\n` (0x0A), consume it; otherwise
  push it back / don't consume. Rust's `BufRead` supports this via `fill_buf()` + `consume(1)`
  without an extra allocation, avoiding the `read_line` over-consumption entirely. This must apply
  uniformly to both the blob-data path and the commit-message-data path since both share the same
  `data N` handler code (no branch differentiates them at the point of the bug).
- **M/D reconciliation fix:** `out.tree.insert` (`:181`) needs a paired removal path — when a later
  `D <path>` line is seen for a path already in `out.tree` (e.g., add-then-remove across the same
  stream, which real `git fast-export` CAN legitimately emit for certain rename/rewrite shapes),
  `out.tree.remove(path)` should fire, OR (simpler, matches BUG-2's fix) apply an explicit filter at
  `plan()`-time: only consider `issues/*.md` paths, and treat the final `deletes` vec as
  authoritative over `tree` for any path appearing in both (deletes win). The mission's `(d)`
  ask specifically flags this — recommend the "deletes win, and filter to `issues/*.md` before
  parsing frontmatter" fix at the `plan()` layer rather than mutating `out.tree` during parse,
  since `plan()` already has a from-scratch reconciliation loop (`diff.rs:169-191`) that can absorb
  this cheaply.

### 1.4 Migration considerations

- **`refresh.rs` 11-pad → unpadded is a producer-only change, not a reader migration hazard,
  IF `issue_id_from_path`/`validate_record_filename` already tolerate any padding.** Confirmed:
  `reposix_core::path::validate_record_filename` (`path.rs:57-72`) parses `<digits>.md` via
  `parse::<u64>()`, which is padding-agnostic (`"00042".parse::<u64>()` == `42` == `"42".parse()`).
  So an EXISTING checkout that still has `issues/00000000042.md` on disk from a prior `reposix
  refresh` run will continue to be readable by anything that goes through the canonical parser —
  no data migration needed, only a producer-side change (new `refresh` calls emit the unpadded
  name). The OLD file simply sits there unless the user re-runs `refresh` or edits it; nothing
  breaks by its mere presence, though a stale-padded file and a fresh-unpadded file for the same id
  would coexist as two different git blobs if a refresh ever writes both — **this is a real hazard
  the plan should call out**: if `refresh.rs` changes format strings but a working tree already has
  `issues/00000000042.md` from before, the NEXT refresh must either (a) detect and rename/delete
  the stale file, or (b) `refresh` always fully regenerates the bucket dir from scratch (need to
  verify: does `run_refresh_inner` ever delete pre-existing files in `bucket_dir`, or does it only
  write new ones? Reading `refresh.rs:112-124`: it does `create_dir_all` then loops writing new
  files by name — **it never deletes stale files**, so a schema change WILL leave orphaned
  differently-padded duplicates in any pre-existing `refresh`'d tree). Planner must add either a
  bucket-dir wipe-and-rewrite step or an explicit migration note in the plan.
- **`builder.rs` (cache tree, stateless-connect production path) ALREADY agrees with the
  recommended target shape** (`issues/<id>.md` unpadded, confirmed `builder.rs:90,131-138`) — no
  change needed there. This is the strongest argument for canonicalizing on this shape: it's zero
  extra work for the crate that's actually exercised by every real `git fetch`/`git init`.
  Confirmed by `real-git-push-e2e.sh:120` itself: `find "$REPO/issues" -maxdepth 1 -name '*.md'`
  after a real `reposix init` + `git checkout -B main refs/reposix/origin/main` — the harness
  author already assumed the unpadded shape is what a real checkout produces.
- **`stateless-connect` serves the cache's tree directly** (gix-based bare-repo object serving);
  it does not re-derive filenames, so whatever `builder.rs` writes is exactly what a `git fetch`
  materializes. This corroborates the "canonical = builder.rs's shape" recommendation.

### 1.5 Regression-test harness — ALREADY EXISTS

`quality/gates/agent-ux/real-git-push-e2e.sh` (161 lines, fully built) already:
1. Checks git version, exits 75/NOT-VERIFIED with a `git_too_old` reason + an artifact JSON if
   git < 2.34 (this box: git 2.25.1 — confirmed via `git --version`, and confirmed via
   `apt-cache policy git` that Ubuntu Focal has NO newer candidate in its configured repos, only
   `1:2.25.1-1ubuntu3.14`).
2. Sources `quality/gates/agent-ux/dark-factory/lib.sh` for `build_and_resolve_bins` (runs
   `cargo build --workspace --bins -q` — **this means running this verifier itself triggers a
   cargo build; do not run it from two subagents concurrently, per the project's memory budget
   rule**), `spawn_sim seeded`, `fail_with`, and the `cleanup`/artifact-writing EXIT trap.
3. Runs `reposix init sim::demo $REPO`, `git checkout -B main refs/reposix/origin/main`, appends
   an edit to the lowest-id `issues/*.md` file, commits, `git push origin main`.
4. Asserts via `sqlite3` against the sim's own db: exactly 1 PATCH, 0 POST, 0 DELETE.
5. Does a second no-op `pull`+`push` and asserts zero additional mutating requests.
6. Catalog row: `quality/catalogs/agent-ux.json:1338-1379`, id `agent-ux/real-git-push-e2e`,
   `kind: mechanical`, `status: WAIVED`, waiver `until: 2026-07-31T00:00:00Z`,
   `tracked_in: "v0.13.0 SURPRISES-INTAKE.md 2026-07-04 QL-001 entry"`, cadences
   `["pre-release", "on-demand"]` (deliberately NOT `pre-pr` — D-CONV-1 stripped it to avoid a
   WAIVED row failing loud on every PR). **The row's own `owner_hint` says P90/P91 must
   re-add `pre-pr` once the fix lands and the waiver is retired.**
7. **Planner-critical: this script CANNOT reach GREEN on this dev box** regardless of whether the
   code fix is correct, because of the git 2.34 precondition gate. The plan MUST decide: (a) treat
   a git-2.34 upgrade (build-from-source or a PPA) as an in-scope task so the fix is actually
   provable end-to-end here, or (b) accept that this verifier stays NOT-VERIFIED locally and rely
   on CI (which likely runs a newer Ubuntu image with git ≥2.34 — **not independently confirmed in
   this research pass; check `.github/workflows/ci.yml`'s runner OS + any explicit git-version
   pin before assuming CI is fine**), or (c) additionally write a lower-level (non-git-version-
   gated) regression that drives `diff::plan`/`parse_export_stream` directly against real
   `issues/<id>.md`-shaped fixtures (satisfying acceptance criteria 1-4 and 6 from the BLOCKER
   entry) as the box-independent proof, keeping `real-git-push-e2e.sh` as the aspirational
   full-stack check that goes green whenever the environment allows.

## Lane 2 — Attach + Sync Real-Backend Wiring

### 2.1 Current stub state

**`crates/reposix-cli/src/attach.rs:143-165`** (mission cited `:147-166`; actual match arm spans
147-165, one-line drift): `connector` match has one live arm `"sim"` (constructs `SimBackend` from
`REPOSIX_SIM_ORIGIN` or `http://127.0.0.1:7878` default) and a catch-all:
```rust
other => bail!(
    "attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03" // banned-words: ok — P91 RBF-A-03 will remove this string
),
```
This line already carries a `// banned-words: ok — P91 RBF-A-03` marker — confirms the project's
own banned-token linter already expects P91 to remove this exact string.

**`crates/reposix-cli/src/sync.rs:83-95`** (mission cited `:79-92`; actual match arm is 83-95,
~4-line drift): identical shape, sim-only:
```rust
other => bail!(
    "sync --reconcile: backend `{other}` not yet wired in v0.13.0 (sim only); \
     github/confluence/jira land alongside the bus-remote work in P82+"
),
```
This one has **no** `// banned-words: ok` marker and no `-NN` suffix on `P82+`, so per CLAUDE.md's
documented banned-token regex scope (`\bP\d{2,3}-\d+\b` requires a hyphen-digit suffix), it is
**deliberately NOT caught** by `banned-production-tokens.sh` — confirmed by grep: this string
appears in the full-workspace `P[0-9]{2,3}[-+]` grep sweep done this pass, alongside similar
un-suffixed `P82+`/`P83-01`-with-marker hits in `reconciliation.rs`, `main.rs`, `precheck.rs`,
`bus_handler.rs`. None of these are hard-blocked by the pre-push gate today; P91 should still clean
the two in `sync.rs`/`attach.rs` (and the `reconciliation.rs:60,182` mentions below) as part of
retiring the stub, per the project's own "CLAUDE.md stays current" + banned-token forward
convention.

**Full-workspace phase-ID grep in production (non-test) code** (this pass, `P[0-9]{2,3}[-+]`):
`reposix-cache/src/reconciliation.rs:60,182`, `reposix-cli/src/sync.rs:44,94`,
`reposix-cli/src/attach.rs:163`, `reposix-remote/src/main.rs:126,190,440`,
`reposix-remote/src/precheck.rs:13,26`, `reposix-remote/src/bus_handler.rs:25,112,245`,
`reposix-quality/src/catalog.rs:226,484`, `reposix-quality/src/commands/doc_alignment.rs:366`,
`reposix-cache/src/db.rs:97`. Most already carry `// banned-words: ok` markers (historical-refactor
class per the earlier-cited intake entry at line 190); the ones directly relevant to P91's stub
removal are `attach.rs:163` and `sync.rs:44,94` and `reconciliation.rs:60,182`.

### 2.2 The pattern to mirror — and the consolidation ROI call

**`crates/reposix-remote/src/backend_dispatch.rs`** (full file read this pass, 724 lines incl.
tests) is the mature, already-correct version of exactly what attach/sync need:

- `BackendKind` enum (Sim/GitHub/Confluence/Jira) + `.slug()`.
- `parse_remote_url(url) -> ParsedRemote { kind, origin, project }` — parses `reposix::<scheme>://
  <host>/[<marker>/]projects/<project>` including the `/confluence/`/`/jira/` disambiguator
  markers (`:105-179`).
- `instantiate(parsed) -> Result<Arc<dyn BackendConnector>>` (`:222-229`) dispatching to
  `instantiate_sim`/`instantiate_github`/`build_confluence`/`build_jira`.
- `build_confluence`/`build_jira` (`:284-324`) each: (1) check required env vars via
  `collect_missing`, (2) construct creds, (3) call `open_connector_audit(kind, project)` — which
  resolves the cache path via `resolve_cache_path`, derives a **sibling** `<...>.audit.db` file
  (deliberately NOT inside the bare-repo `.git` dir, since `gix::init_bare` refuses non-empty
  dirs), opens it via `reposix_core::audit::open_audit_db`, and wraps it in
  `Arc<Mutex<Connection>>`, (4) constructs the backend and calls `.with_audit(audit)` — this is the
  **exact OP-3 wiring** the mission asks P91 to "not bypass."
- `instantiate_github` requires only `GITHUB_TOKEN` and does NOT wire `with_audit` (comment at
  `:236-239` explains: GitHub is read-only in this milestone, writes are `NotSupported`, so no
  client-side mutation-audit connection is needed for it — only Confluence/JIRA get one).
- Tests already assert `connector_audit_wired_confluence`/`connector_audit_wired_jira` (`:674-722`)
  by checking `backend.has_audit()` AND that the sibling `.audit.db` file's `audit_events` table
  exists — this is a ready-made template for equivalent `attach_real_*`/`sync_real_*` assertions.

**Crate-dependency reality check (verified via Cargo.toml reads):** `reposix-remote` has
**no `[lib]` target** (only `[[bin]] name = "git-remote-reposix" path = "src/main.rs"`), so
`backend_dispatch` is presently unreachable from `reposix-cli` even in principle. `reposix-core`
(the only crate all of confluence/github/jira/cache/cli/remote depend on) **cannot** host this
logic without creating a reverse dependency (confluence/github/jira all depend ON reposix-core;
reposix-core depends on none of them — confirmed via each backend crate's Cargo.toml). So the two
realistic consolidation shapes are:
1. **(Recommended)** Add `crates/reposix-remote/src/lib.rs` exposing `pub mod backend_dispatch;`,
   thin `main.rs` down to `use git_remote_reposix::backend_dispatch;` (or whatever the lib crate
   name resolves to — note the package is named `reposix-remote`, binary is `git-remote-reposix`;
   the lib target name would default to `reposix_remote`), add `reposix-remote = { path =
   "../reposix-remote", version = "0.12.0" }` to `reposix-cli/Cargo.toml`, and have `attach.rs`/
   `sync.rs` call `reposix_remote::backend_dispatch::{parse_remote_url, instantiate}` instead of
   their own bail-only match arms. Currently exposed items (`parse_remote_url`, `instantiate`,
   `BackendKind`, `ParsedRemote`, `sanitize_project_for_cache`) are all `pub(crate)` today and would
   need `pub` visibility bumps.
2. A brand-new tiny shared crate (e.g. `reposix-backend-dispatch`) that both `reposix-remote` and
   `reposix-cli` depend on, avoiding coupling `reposix-cli`'s build to `reposix-remote`'s unrelated
   modules (`diff`, `fast_import`, `bus_handler`, `pktline`, etc.). This is cleaner architecturally
   but is a bigger diff (new crate registration in the workspace `Cargo.toml`, new crate boilerplate
   satisfying `#![forbid(unsafe_code)]`/`#![warn(clippy::pedantic)]`/missing-docs conventions).

Either way, **do not** write attach/sync's real-backend dispatch as fresh code — the credential
error messages, the `docs/reference/testing-targets.md` pointer convention
(`missing_env_error`, `:363-372`), and the audit wiring are all already correct and tested in
`backend_dispatch.rs`; re-deriving them a third/fourth time is the ROI anti-pattern the mission
flags. Note also **`refresh.rs`'s `fetch_issues`** (`:174-240`, already read in full this pass) is
a **third**, read-only-only copy of the same env-var-check pattern (confirmed: no `with_audit` call
anywhere in `refresh.rs` — grep for `with_audit`/`open_audit_db` in that file returns zero hits,
consistent with refresh being list-only/no mutation). If P91's consolidation lands, a natural
follow-on (flag as GOOD-TO-HAVE, not P91-mandatory) is retargeting `refresh.rs` at the same shared
helper too, closing all three/four copies at once.

### 2.3 `crates/reposix-cache/src/reconciliation.rs` — the 5 cases + the `ForkAsNew` stub

Five cases (module doc `:4-15`, code `:94-229`):
1. **Match** — local id found in backend set → `cache_reconciliation` row written (`:203-227`).
2. **Backend-deleted** — local id absent from backend → handled per `OrphanPolicy` (`:166-189`).
3. **No-id** — file fails `frontmatter::parse` → warned, skipped (`:117-127`, `:198-201`).
4. **Duplicate-id** — two local files share an id → hard abort, **zero rows written** (atomicity,
   `:136-144`).
5. **Mirror-lag** — backend has id, no local file → counted only, no row written (`:191-196`).

`OrphanPolicy::ForkAsNew` stub (`:180-186`):
```rust
OrphanPolicy::ForkAsNew => {
    eprintln!(
        "BACKEND_DELETED id={} local_file={} action=FORK_AS_NEW (TODO P82+)",
        id.0,
        path.display(),
    );
}
```
This is a **pure eprintln, no state written anywhere** — the "fork as new record to be created on
next push" promise (doc comment `:57-60`, CLI help text in `attach.rs:77`) is not backed by any
actual mechanism today; there is no row, flag, or marker file that a subsequent `git push` could
read to know "treat this orphan as a Create." **Recommend: either implement it for real this
phase** (the natural mechanism: since the push planner's `plan()` already treats any path present
in the new tree but absent from `prior_by_path` as a `Create` — see `diff.rs:163-165` — `ForkAsNew`
at attach-time may not need ANY additional state at all, because the very next `git push` from this
tree would already classify the file as a Create for free, once BUG-1's path-shape fix lands. In
that case `ForkAsNew`'s job is *only* to NOT delete the file and NOT abort — which the current code
already does correctly! The "(TODO P82+)" comment may be describing already-satisfied behavior once
Lane 1 lands.) **or make the TODO an explicit, cited error** if there's a real gap the researcher
didn't surface (e.g., what happens if the orphan's frontmatter has a *stale* id that collides with
a DIFFERENT backend id assigned to someone else's concurrent create between attach and push? Out of
this research's depth — flag as an open question for the planner/executor to resolve with a
targeted read of `diff.rs`'s Create path once Lane 1 lands).

**Doc/behavior mismatch worth flagging (see NOTICED):** `OrphanPolicy::Abort` — despite its name and
doc comment ("Abort attach (default)") — does **not** abort. Reading `:166-189`: all three policy
arms just choose a different `eprintln!` message (and `DeleteLocal` additionally calls
`std::fs::remove_file`); none of them returns an `Err` or sets any abort flag, and the caller
(`attach.rs::run`) only bails on `duplicate_id_files` non-empty, never on `backend_deleted_count`.
This is confirmed intentional-but-misleadingly-named by the test at
`crates/reposix-cli/tests/attach.rs:403`, whose own comment says "attach should succeed under
--orphan-policy=abort (default just warns)". The enum variant name and its doc comment should be
reconciled in this phase (rename to e.g. `WarnOnly`/`Report`, or fix the doc to stop saying
"Abort attach") — small, but a real doc-vs-code mismatch a cold reader would trip on.

### 2.4 `crates/reposix-cli/tests/agent_flow_real.rs` — current content + stale claim

Full file read this pass (233 lines). Structure:
- `skip_if_no_env!` macro (`:50-63`), copied-verbatim convention from
  `reposix-confluence/tests/contract.rs` (confirmed: that file's macro is indeed at its own
  `:61-74`, matching the module doc's citation).
- `run_init_and_assert(spec, expected_url_prefix)` helper (`:94-123`): runs `reposix init <spec>
  <path>`, asserts exit 0, greps `git config remote.origin.url` for the prefix. **No fetch, no
  push, no list_records call is exercised** — purely a config-string assertion, matching raise-list
  §2's "THIN-but-exempt" classification.
- Three `#[ignore = "real-backend; requires <ENV_VARS>"]`-gated tests: `dark_factory_real_github`
  (`:126-138`), `dark_factory_real_confluence` (`:144-165`), `dark_factory_real_jira`
  (`:168-184`) — **this is the exact gating idiom to mirror** for new `attach_real_*`/`sync_real_*`
  tests: `#[ignore = "real-backend; requires <VARS>"]` + `skip_if_no_env!(...)` as the first line of
  the test body, so `cargo test` (no `--ignored`) is always safe on a fresh clone with no secrets,
  and CI opts in via `cargo test ... -- --ignored <test_name>` with secrets injected as env
  (confirmed against `ci.yml:265,290` which does exactly this for github/jira).
- `skip_pattern_compiles_and_runs_without_creds` (`:191-232`) — a non-ignored defensive test
  proving the skip macro itself works; good template if new tests want an analogous
  compiles-and-skips-cleanly smoke test.

**STALE CLAIM (flag for fix, not P92's job — this is Lane-2-adjacent doc hygiene):** module doc
`:25-27` states *"the helper still hardcodes `SimBackend` (Phase 32 limitation — see
32-SUMMARY.md), so the 'real-backend exercise' verified here is bounded to..."* — this is **false
today**. `backend_dispatch.rs`'s own module doc (`:1-7`) says it *"Closes the v0.9.0 Phase 32
carry-forward tech debt where the helper hardcoded `SimBackend`... The helper now parses the URL,
identifies the target backend (sim, github, confluence, jira), and instantiates the corresponding
`BackendConnector`."* Two module docs in the same workspace directly contradict each other about
whether this debt is closed — `backend_dispatch.rs` is right (it demonstrably dispatches to all
four backends, confirmed above), `agent_flow_real.rs`'s doc is stale prose that predates
`backend_dispatch.rs` landing and was never updated. **P91 should fix this doc comment** while it's
touching this file to add attach/sync real-backend tests — leaving it would compound the confusion
for the next reader trying to understand why "real backend" tests only check a config string.

### 2.5 OP-3 audit obligations for attach/sync

- `attach.rs` already writes ONE `audit_events_cache` row per successful attach via
  `cache.log_attach_walk("attach_walk", payload)` (`attach.rs:225-227`, backing fn at
  `reposix-cache/src/cache.rs:580`+) — this covers the cache-internal audit table regardless of
  backend. **Confirmed sufficient for attach's own REST traffic**, because `attach.rs` only calls
  `cache.build_from()` (a `list_records` REST GET, no mutation) — the `audit_events` (mutation)
  table is not applicable here; only Confluence/JIRA *mutation* calls need `with_audit`.
- `sync.rs --reconcile` calls `Cache::sync()` → `Cache::build_from()` under the hood, which itself
  writes `audit::log_tree_sync(...)` (confirmed at `builder.rs:118`) — same cache-internal audit
  coverage, same "no mutation, so no `audit_events` requirement" reasoning applies.
- **Conclusion: if P91 adopts the `backend_dispatch.rs`-derived shared helper for constructing real
  connectors, OP-3 wiring for any FUTURE mutation path is inherited for free** (Confluence/JIRA
  connectors built via `build_confluence`/`build_jira` always get `with_audit`); if P91 instead
  hand-rolls a THIRD dispatch copy in `attach.rs`/`sync.rs` without calling
  `open_connector_audit`, that would be a silent OP-3 regression risk for whichever future phase
  (P92 per the mission) adds write paths through attach/sync's connector. **This is the strongest
  argument for the consolidation recommendation in §2.2** — it's not just DRY, it's the difference
  between "OP-3 compliant by construction" and "OP-3 compliant only if the next phase remembers."

### 2.6 Confluence live-hierarchy fragile test + self-seeding option

`crates/reposix-confluence/tests/contract.rs:750-797` (`contract_confluence_live_hierarchy`, exact
line range matches the mission's citation): `#[ignore]` + `skip_if_no_env!` gated, lists records
from a live Confluence space (default via `REPOSIX_CONFLUENCE_SPACE`), asserts non-empty AND
asserts `≥1` record has `parent_id.is_some()`. This is READ-ONLY against live state it doesn't
own — SURPRISES-INTAKE 2026-07-03 21:00 entry (line 198-206) documents it broke CI run
`28692818500` and was mitigated by seeding durable fixture pages in the TokenWorld tenant's
`REPOSIX` space (id 360450): parent page `7766017` → child `7798785`, labeled
`reposix-durable-fixture` (deliberately not the sweepable `kind=test` label), **do not delete**.
`create_record` already supports `parentId` — confirmed at
`crates/reposix-confluence/src/lib.rs:280-290` (`create_record` builds the REST payload including
`"parentId": issue.inner_ref().parent_id.map(|id| id.0.to_string())`). A self-seeding version of
this test would: at test start, `create_record` a parent + child pair (or verify the durable
fixture pair still exists by id, e.g. re-`get_record(7766017)`/`get_record(7798785)`) rather than
depending on whatever the live space happens to contain that day. Recommend the "verify the durable
fixture pair still exists, re-seed if missing" hybrid over pure self-seeding-every-run (avoids
Confluence space clutter across repeated CI runs) — but this is a genuine judgment call the planner
should make explicitly, not something this research settles unilaterally.

### 2.7 CI env-var gap (intake entry, confirmed)

`.github/workflows/ci.yml:267-290` (`integration-contract-jira-v09` job) sets `JIRA_EMAIL`,
`JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` but **never forwards `JIRA_TEST_PROJECT` or
`REPOSIX_JIRA_PROJECT`** — confirmed by direct read; the job's `run:` block (`:283-290`) has no
reference to either var. `agent_flow_real.rs::jira_test_project()` (`:79-83`) falls back to the
literal `"TEST"` when both are unset, but the owner's actual live JIRA project key is `KAN` (per
the intake entry, sourced from the owner's `.env.example` note) — so CI silently runs against
whatever project key `"TEST"` resolves to on the configured JIRA instance, NOT the owner's real
project, unless a `TEST` project also happens to exist. Same gap likely applies to the
`bench-latency-v09` job (`ci.yml:292-349`) if it also drives JIRA — confirmed that job's env block
(`:317-327`) sets the same three JIRA vars and also omits `JIRA_TEST_PROJECT`/
`REPOSIX_JIRA_PROJECT`. Fix: add `JIRA_TEST_PROJECT: ${{ secrets.JIRA_TEST_PROJECT }}` (or a repo
variable, owner's choice) to both job env blocks and forward it into the test process env (already
automatic for `agent_flow_real` since `env:` on the GH Actions step sets process env directly for
the `run:` shell, which then invokes `cargo test` — no extra plumbing needed beyond adding the line
to `env:`).

## NOTICED (ownership-charter §2 — deliverable, not optional)

1. **Two module docs directly contradict each other** about whether backend-dispatch tech debt is
   closed: `agent_flow_real.rs:25` ("the helper still hardcodes SimBackend") vs
   `backend_dispatch.rs:1-7` ("Closes the v0.9.0 Phase 32... hardcoded SimBackend"). The former is
   stale and should be corrected in the same commit that touches this file for P91's new tests.
2. **`OrphanPolicy::Abort` never aborts** — misleading enum-variant name + doc comment
   ("Abort attach (default)") for behavior that is, per the codebase's own test comment
   (`attach.rs:403`), "default just warns." Small naming/doc fix, real cold-reader trap.
3. **`OrphanPolicy::ForkAsNew`'s "TODO P82+" may already be a no-op fix** once Lane 1's path-shape
   canonicalization lands — worth a deliberate check rather than assuming it needs new code (see
   §2.3). Don't build machinery that turns out to be unnecessary.
4. **`reposix_core::path::validate_record_filename` already exists, is tested, and is
   padding-agnostic** — but is used by NEITHER `diff.rs`'s nor `main.rs`'s `issue_id_from_path`,
   which are two independent hand-rolled duplicates (one of which — the `main.rs` copy — is
   explicitly tracked as tech debt under "QL-157" per the raise-list). The canonical-path fix should
   build on this existing function rather than writing a sixth parser.
5. **`refresh.rs`'s bucket-dir write loop never deletes stale files** (`refresh.rs:112-124`) — if
   the padding convention changes, any working tree that was previously `refresh`'d under the old
   11-pad scheme will accumulate BOTH the old-padded and new-unpadded file for the same id on the
   next refresh, unless the plan explicitly adds a wipe-and-rewrite (or detect-and-rename) step.
   This is a real migration hazard that neither the SURPRISES-INTAKE entry nor the raise-list
   explicitly calls out.
6. **The real-git-push-e2e regression harness (script + catalog row) is fully built already** —
   this was NOT explicitly stated as "already exists, just needs the waiver retired" in the mission
   brief, and a planner working from the SURPRISES-INTAKE prose alone might reasonably assume this
   harness still needs to be written from scratch (mission deliverable (f) asks "where should the
   regression test live and what harness exists" as if it's an open question). It is not open — the
   harness is fully built (`quality/gates/agent-ux/real-git-push-e2e.sh`, catalog row at
   `quality/catalogs/agent-ux.json:1338-1379`), only currently WAIVED because the bug it tests for
   is real and expected. **This materially shrinks Lane 1's scope**: no new e2e-harness authoring
   task should appear in the plan, only "make the existing assertions pass + flip WAIVED→PASS +
   restore `pre-pr` cadence" — but the git≥2.34 environment gap (below) limits what "pass" can mean
   on this box specifically.
7. **This dev box cannot locally prove the git≥2.34 path is fixed** — Ubuntu Focal's apt has no
   git≥2.34 candidate (`apt-cache policy git` confirmed only `1:2.25.1-1ubuntu3.14` available). The
   `integration-git` cargo feature (`reposix-remote/Cargo.toml:36-41`) already documents this exact
   constraint for a DIFFERENT test suite ("Off by default because the dev host's git is 2.25.1").
   This is a pre-existing, known, structural environment limit — not new information, but worth
   restating because it directly bears on how P91's acceptance criteria can be verified locally.
8. **`bulk_delete_cap.rs` exists as a standalone test file** separate from `diff.rs`'s in-module cap
   tests, and this research pass did not have budget to fully audit it beyond a negative grep for
   the bare-padded literal (`"000N.md"`, zero hits) — flagging as an unclosed loop for the executor
   rather than asserting it's clean.
9. **`no_op_tree_export`'s exact path-format string** (used by `perf_l1.rs`'s no-op-push economy
   test) was not independently traced to its definition in this pass (only the comment referencing
   it at `perf_l1.rs:311` was read) — flag for the executor to verify before assuming the cited
   line alone captures the fixture.

## Planner Warnings

1. **Do not scope Lane 1 as "write a real-git-push regression test."** It already exists and is
   catalog-registered. Scope it as: fix the four path-shape sites + the stream-parser bug + the
   `issues/*.md` filter, re-key the ~10 fixture files enumerated in §1.2, then flip the existing
   waiver. Writing a *second* e2e harness would be pure duplication.
2. **The git≥2.34 precondition cannot be satisfied by this box's package manager.** If the plan
   wants a GREEN, locally-verifiable acceptance signal, it needs either an explicit git-upgrade
   task (build from source / third-party PPA — not yet researched, would need its own research
   spike) or a lower-level test that bypasses stateless-connect fetch entirely (drives `diff::plan`
   / `parse_export_stream` against hand-constructed-but-canonically-shaped fixtures, matching what
   the BLOCKER entry's own reproduction did). Don't assume CI's runner has a newer git without
   checking `ci.yml`'s runner image/git-setup step first (not verified in this pass).
3. **`reposix-cli` has ZERO existing dependency on `reposix-remote`.** If the plan adopts the
   consolidation recommendation (§2.2), it must include a Cargo.toml dependency-graph change
   (`reposix-remote` gains a `[lib]` target, `reposix-cli` adds a path dependency) as an explicit
   task — this is not a trivial one-line import, it changes the crate's public API surface
   (several `pub(crate)` items need `pub`) and its build shape (a `[lib]` alongside the existing
   `[[bin]]`), which affects `cargo check -p reposix-cli` transitively pulling in `reposix-remote`'s
   full dependency tree (gix, wiremock-adjacent dev-deps stay dev-only, so production build weight
   growth should be limited to what `reposix-cli` already links via confluence/github/jira/cache —
   verify no NEW production dependency is pulled in, e.g. `parking_lot`/`rusqlite[bundled]`, which
   `reposix-remote` already has and `reposix-cli` may or may not already carry — quick check: `grep
   parking_lot crates/reposix-cli/Cargo.toml` returned nothing in a corollary grep during this
   pass, so this WOULD be a new prod dependency for reposix-cli; note it in the plan's dependency
   diff).
4. **Do not build new `OrphanPolicy::ForkAsNew` machinery before checking whether Lane 1's fix
   already satisfies it for free** — see NOTICED #3. This could turn a planned M-sized subtask into
   a no-op verification task, or reveal it genuinely needs new state; either way, investigate before
   scoping the task size.
5. **Attach/sync real-backend wiring must preserve the OP-3 with_audit wiring pattern** — if the
   plan's tasks hand-roll a fresh match arm in `attach.rs`/`sync.rs` instead of delegating to
   (a refactored) `backend_dispatch.rs`, explicitly add "wire `with_audit` for Confluence/JIRA
   connectors, mirroring `backend_dispatch.rs:284-324`" as an acceptance criterion — it will not
   happen automatically from copy-pasting only the `SimBackend` arm's shape.
6. **Both cited env-var CI gaps (JIRA_TEST_PROJECT) are `.github/workflows/ci.yml` edits, not Rust
   code** — cheap, but easy to forget since the mission's other six deliverables are all Rust-code
   focused; make sure the plan has an explicit task for the two `ci.yml` job env blocks
   (`:267-290` and `:292-349`).
7. **`agent_flow_real.rs`'s stale module doc (§2.4) should be corrected in the same commit that adds
   new attach/sync real-backend tests to that file** (or a sibling file) — don't let it silently
   persist alongside newly-accurate test bodies; that's exactly the "doc claims that lie" class the
   ownership charter asks every phase to notice AND fix.

## Sources

### Primary (HIGH confidence — direct file:line reads this session)
- `crates/reposix-cache/src/builder.rs` (lines 70-150)
- `crates/reposix-cli/src/refresh.rs` (lines 80-230)
- `crates/reposix-remote/src/fast_import.rs` (full file)
- `crates/reposix-remote/src/diff.rs` (full file)
- `crates/reposix-remote/src/precheck.rs` (lines 120-180)
- `crates/reposix-remote/src/main.rs` (issue_id_from_path grep context)
- `crates/reposix-core/src/path.rs` (full file)
- `crates/reposix-remote/src/backend_dispatch.rs` (full file)
- `crates/reposix-cli/src/attach.rs` (full file)
- `crates/reposix-cli/src/sync.rs` (full file)
- `crates/reposix-cache/src/reconciliation.rs` (full file)
- `crates/reposix-cli/tests/agent_flow_real.rs` (full file)
- `crates/reposix-cli/tests/agent_flow.rs` (lines 90-225)
- `crates/reposix-confluence/tests/contract.rs` (lines 740-797)
- `quality/gates/agent-ux/real-git-push-e2e.sh` (full file)
- `quality/gates/agent-ux/dark-factory/lib.sh` (full file)
- `quality/catalogs/agent-ux.json` (lines 1325-1379)
- `.github/workflows/ci.yml` (lines 260-349)
- `crates/reposix-remote/Cargo.toml`, `crates/reposix-cli/Cargo.toml`, `crates/reposix-core/Cargo.toml`,
  `crates/reposix-confluence/Cargo.toml`, `crates/reposix-jira/Cargo.toml`, `crates/reposix-github/Cargo.toml`
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (lines 300-425, esp. 2026-07-04 06:20
  QL-001 BLOCKER entry, and the two 2026-07-03 21:00 entries re: confluence live test + JIRA CI gap)
- `quality/reports/raise-list-p90.md` (full file)
- Shell: `git --version`, `apt-cache policy git`, `git log -1`, `git status` (this box)
- Grep sweeps: `format!("{:04}`/`{:011}`, `"issues/`, `strip_suffix(".md")`, `P[0-9]{2,3}[-+]` across
  `crates/**/*.rs` (excluding `target/`)

### Secondary (MEDIUM — cross-referenced against catalog/intake prose, not independently re-derived)
- Confluence durable-fixture page ids (7766017/7798785) and the 2026-07-03 CI-break narrative —
  sourced from the SURPRISES-INTAKE entry, not independently verified against live Confluence
  (would require `ATLASSIAN_*` creds; out of scope for a read-only research pass).

### Not verified this pass (flag for executor)
- Whether CI's runner image ships git ≥2.34 (would resolve Planner Warning #2's "can CI prove this
  green" question) — `ci.yml`'s `runs-on: ubuntu-latest` + `actions/checkout@v7` steps were seen in
  context but no explicit git-version-setup step was inspected.
- `crates/reposix-remote/tests/bulk_delete_cap.rs` full contents.
- `no_op_tree_export`'s definition site (likely `perf_l1.rs` itself or `common.rs`).

## Metadata

**Confidence breakdown:**
- Lane 1 line citations: HIGH — every cited line independently re-read this session; several
  1-4-line drifts from the mission's citations corrected above (root cause: file edits between when
  the mission brief was written and this research pass, or the brief's own line numbers were
  approximate — not a discrepancy that changes any conclusion).
- Lane 1 harness-already-exists finding: HIGH — full file + catalog row read directly.
- Lane 2 consolidation recommendation: HIGH on the facts (no `[lib]` target, no `reposix-cli` →
  `reposix-remote` dependency today, `backend_dispatch.rs` is `pub(crate)`-only) — MEDIUM on the
  specific recommended shape (lib-in-remote vs. new shared crate), which is a legitimate design
  choice the planner should make explicitly rather than treating as settled by this research.
- git≥2.34 environment constraint: HIGH (directly probed via `git --version` + `apt-cache policy`).

**Research date:** 2026-07-04
**Valid until:** ~14 days (fast-moving phase; code lines will shift as P91 lands edits — re-verify
line numbers before citing them in a PR description or plan-check gate).
