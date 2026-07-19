# Phase 125: Real-backend cadence & mirror-drift resilience - Pattern Map

**Mapped:** 2026-07-18
**Files analyzed:** 9 (2 Rust src edits, 1 new/extended Rust test, 1 bash self-heal
edit, 2 doc edits, 3 read-only reused mechanisms)
**Analogs found:** 9 / 9 (every file has an in-repo analog — this phase is composition,
not new-mechanism design, matching RESEARCH.md's own framing)

No CONTEXT.md exists for this phase (it went straight from ROADMAP scope to research).
File list below is extracted from RESEARCH.md's "Code Examples", "Wave 0 Gaps", and
"Common Pitfalls" sections, cross-checked against the phase's two requirements
(DRAIN-02, DRAIN-12 — `.planning/REQUIREMENTS.md` lines 166-169, 242-246).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/reposix-remote/src/write_loop.rs` (MODIFY) | service (git-protocol handler) | request-response | `crates/reposix-remote/src/bus_handler.rs::precheck_mirror_drift` (redaction+teach pattern) | role-match (self-file; analog supplies the redaction/teach convention this edit must follow) |
| `crates/reposix-remote/tests/push_conflict.rs` (MODIFY — add test) | test (Rust integration) | request-response | `push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest` (same file, sibling test) | exact — same fn (`apply_writes`), same firing site, same wiremock+`assert_cmd` harness |
| `quality/gates/agent-ux/lib/litmus-flow.sh` (MODIFY) | test-harness (bash, sourced lib) | batch / event-driven (multi-step git+REST round-trip) | `scripts/refresh-tokenworld-mirror.sh` (mechanism to fold in) + `quality/gates/agent-ux/dark-factory/reconciliation-fixture.sh` (factoring-out precedent if size budget is exceeded) | exact — RESEARCH.md names this exact insertion |
| `docs/reference/testing-targets.md` (MODIFY) | docs (reference) | — | its own `## Pre-flight verification` section (lines 23-45) + `## Running real-backend tests` callout (lines 271-280) | exact — same file, existing section shape is the insertion-point template |
| `docs/guides/troubleshooting.md` (MODIFY, fix-twice) | docs (guide) | — | its own `### Bus-remote fetch first rejection` section (lines 233-281) | exact — same file, the section whose recovery command SC3 augments |
| `docs/concepts/dvcs-topology.md` (verify/MODIFY, fix-twice) | docs (concept) | — | its own lines 89-93, 152 | exact — same file |
| `scripts/refresh-tokenworld-mirror.sh` (READ-ONLY, reused) | utility (bash, ops tool) | file-I/O + REST | — (source of the mechanism, not itself modified) | n/a |
| `scripts/confluence_tokenworld.py` (READ-ONLY, reused) | utility (Python, REST CLI) | CRUD (REST) | — (source of `restore`/`reparent`, not itself modified) | n/a |
| `crates/reposix-remote/src/bus_handler.rs` (READ-ONLY reference) | service (git-protocol handler) | request-response | — (confirmed NOT the DRAIN-02 firing site; read for the redaction convention only) | n/a |
| `crates/reposix-remote/src/precheck.rs` (READ-ONLY reference) | service (precheck/L1 logic) | request-response | — (confirmed root-cause site; no edit expected) | n/a |
| `quality/gates/agent-ux/milestone-close-vision-litmus.sh` (READ-ONLY reference) | test-harness (bash, caller) | shell-subprocess | — (wrapper around `litmus-flow.sh`; unmodified per RESEARCH.md's Standard Stack table) | n/a |

## Pattern Assignments

### `crates/reposix-remote/src/write_loop.rs` (service, request-response) — SC3 / DRAIN-12

**What to change (exact target, already located by RESEARCH.md):** the mirror-lag-ref
hint block, lines 201-219 of the CURRENT file (reproduced verbatim below so the plan can
diff against it exactly).

**Current code — lines 189-225** (the block containing BOTH bugs Pitfall 2 and Pitfall 4
name):
```rust
        // P121 W3.5: the `[RPX-0505]` tag rides this human-facing STDERR diag
        // ONLY. The `error refs/heads/main fetch first` PROTOCOL line emitted
        // below (git's standard conflict status) stays verbatim — tagging it
        // would corrupt the push status git parses (registry cause for RPX-0505
        // documents this split).
        crate::diag(&format!(
            "issue {} modified on backend at {} since last fetch (local base version: {}, backend version: {}) [RPX-0505]. Run: git pull --rebase (run `reposix explain RPX-0505` for the full cause + recovery)",
            first_id.0, backend_ts, local_v, backend_v,
        ));
        if let Some(c) = cache {
            c.log_helper_push_rejected_conflict(&first_id.0.to_string(), *local_v, *backend_v);

            // Mirror-lag-ref reject hint (DVCS-MIRROR-REFS-03). When
            // refs are populated (post-first-push), name the staleness
            // gap; when absent (first-push case), omit the hint cleanly
            // per RESEARCH.md pitfall 7.
            if let Ok(Some(synced_at)) = c.read_mirror_synced_at(backend_name) {
                let ago = chrono::Utc::now().signed_duration_since(synced_at);
                let mins = ago.num_minutes().max(0);
                crate::diag(&format!(
                    "hint: your origin (GH mirror) was last synced from {sot} at {ts} ({mins} minutes ago); see refs/mirrors/{sot}-synced-at",
                    sot = backend_name,
                    ts = synced_at.to_rfc3339(),
                ));
                crate::diag(&format!(
                    "hint: run `reposix sync` to update local cache from {backend_name} directly, then `git rebase`",
                ));
            }
        }
        proto.send_line("error refs/heads/main fetch first")?;
```

**Two concrete edits, both inside the `if let Ok(Some(synced_at)) = ...` arm:**

1. **Pitfall 2 (one-line, high confidence):** the `"hint: run \`reposix sync\` ..."`
   diag is missing `--reconcile`. Compare against the ALREADY-CORRECT documented example
   at `docs/concepts/dvcs-topology.md:90`: `` "hint: run `reposix sync --reconcile` to
   refresh your cache against the SoT, then `git pull --rebase`" ``. The code drifted
   from the doc, not the other way around — bring the string in line with the doc's
   phrasing (same words), keeping `crate::diag(&format!(...))` call shape.

2. **Pitfall 4 (ADD, do not replace):** append a NEW `crate::diag(&format!(...))` call
   in the same arm, after the corrected sync hint, that names the Pattern-C
   bare-`git pull`-reads-stale-mirror risk. RESEARCH.md's own suggested wording (Common
   Pitfalls → Pitfall 3):
   ```rust
   crate::diag(&format!(
       "hint: if this tree was created via `reposix attach`, a bare `git pull`/`git rebase` \
        reads the ORIGIN MIRROR by default (which may itself be stale) — rebase against the \
        SoT-backed remote explicitly, e.g. `git pull --rebase <reposix-remote-name> main`",
   ));
   ```
   **HARD CONSTRAINT:** do NOT touch the literal substring `"Run: git pull --rebase"` at
   line ~198 (the RPX-0505 diag) — 19 `grep` hits across
   `quality/catalogs/doc-alignment.json` plus 3 Rust test files pin that exact substring
   (see "Shared Patterns → Pin: literal substring" below). The new line is ADDITIVE.

**Redaction convention to reuse if the new hint interpolates anything dynamic**
(Security Domain — this phase's single highest-relevance security concern). Copy from
`crates/reposix-remote/src/bus_handler.rs:453-458`:
```rust
let safe_url = crate::backend_dispatch::redact_userinfo(mirror_url);
let stderr = String::from_utf8_lossy(&out.stderr);
let safe_stderr = crate::backend_dispatch::redact_userinfo(stderr.trim());
```
The write_loop.rs hint as sketched above interpolates only `backend_name` (a static
config string, not a URL) and a literal remote-name placeholder — if the planner's final
wording instead threads an ACTUAL git remote name or URL (Open Question 1 below), route
it through `redact_userinfo` before interpolation, per `crates/CLAUDE.md` § "Credential
redaction before ANY error/stderr echo."

**Open design question the planner must resolve before finalizing wording (RESEARCH.md
Assumption A3):** `apply_writes` (this file) is shared between `handle_export`
(single-backend) and `bus_handler::handle_bus_export` (bus) and, per its own doc header
(lines 1-46), does not currently receive a remote-name parameter. `bus_handler.rs`
resolves `mirror_remote_name` at its STEP 0 (line 167) but that name is not threaded into
`apply_writes`'s call at line 287-295. Either (a) add an `Option<&str>` remote-name param
threaded from both call sites, or (b) keep the hint generic ("the SoT-backed remote you
configured via `reposix attach`, not `origin`") without naming it. `litmus-flow.sh`'s own
bounded self-heal already hardcodes the literal remote name `reposix` (its own
convention, see below) — a generic hint is the lower-risk default if the planner doesn't
want to touch the `apply_writes` signature.

---

### `crates/reposix-remote/tests/push_conflict.rs` (test, request-response) — Wave 0 gap

**Analog: same file, `stale_base_push_emits_fetch_first_and_writes_no_rest`** (lines
109-215 read in full). This is the EXACT existing test driving the EXACT function
(`apply_writes` conflict path) the new test must also drive — same file is the right
home for the new test (sibling `#[tokio::test]` fn), not a new file, because it reuses
`sample_issue`, `render_with_overrides`, `one_file_export` verbatim.

**What's missing (RESEARCH.md Wave 0 Gaps):** no existing test asserts the CONTENT of
the augmented mirror-lag hint — only the base `"git pull --rebase"` substring (asserted
at line 198 of this file). The existing test never populates
`refs/mirrors/<sot>-synced-at`, so the mirror-lag-ref-populated branch (write_loop.rs's
`if let Ok(Some(synced_at)) = c.read_mirror_synced_at(...)`) never fires in the current
suite.

**The one new setup step needed, not present in any existing test:** call
`Cache::write_mirror_synced_at` BEFORE driving the stale-base push, so
`read_mirror_synced_at` returns `Some`. Signature (verified,
`crates/reposix-cache/src/mirror_refs.rs:155`):
```rust
pub fn write_mirror_synced_at(&self, sot_host: &str, ts: DateTime<Utc>) -> Result<String>
```
Call-site precedent for how a caller invokes it post-write (`bus_handler.rs:321-326`):
```rust
if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, chrono::Utc::now()) {
    tracing::warn!("write_mirror_synced_at failed: {e:#}");
}
```
The new test needs a `Cache::open(...)` handle in the SAME `REPOSIX_CACHE_DIR` the
`git-remote-reposix` subprocess will use, call `write_mirror_synced_at("sim", <ts>)` on
it (this test's backend is `sim`, matching the URL scheme `reposix::{server.uri()}/...`
used elsewhere in the file — NOT `"confluence"`, which is only correct for the litmus's
real-backend scenario), `drop(cache)` to release the lock (mirrors the `drop(cache);`
pattern already used in `bus_precheck_b.rs:114-115` after `cache.sync().await`), THEN run
the existing stale-base drive (lines 154-181 of this file), THEN add new assertions:

```rust
assert!(
    stderr.contains("reposix sync --reconcile"),
    "stderr missing corrected --reconcile flag on the sync hint: {stderr}"
);
assert!(
    stderr.contains("git pull --rebase <reposix-remote-name> main")   // or the planner's final wording
        || stderr.contains("reposix attach"),
    "stderr missing the Pattern-C remote-explicit augmentation hint: {stderr}"
);
```
(exact substring TBD by the planner's final wording choice from the two-edit block
above — the pattern is `stderr.contains(...)` with a descriptive panic message, matching
every existing assertion in this file, e.g. lines 189-211.)

**Test-name-honesty marker convention** (must appear within 6 lines above the fn
signature — `quality/CLAUDE.md` § "Marker placement window"), copy the exact comment
shape from this file's own line 110:
```rust
#[tokio::test]
// test-name-honesty: ok — drives helper export via stdin against wiremock; genuine push-path coverage
async fn <new_fn_name>() {
```

---

### `quality/gates/agent-ux/lib/litmus-flow.sh` (test-harness, batch) — SC2 / DRAIN-12

**Current file is 9794 bytes; the 10k `.sh` ceiling
(`quality/gates/structure/file-size-limits.sh`) is CURRENTLY WAIVED (`--warn-only`) until
2026-08-08** — so exceeding it this phase WARNs, does not block. Still budget-check:
Pattern 1 + Pattern 2 insertions below are ~7 lines / ~330 bytes and ~7 lines / ~380
bytes respectively — pushing the file to ~10,500 bytes, ~5% over. **If the planner wants
a clean (non-WARN) result,** factor the two insertions into a NEW sourced lib file
(`quality/gates/agent-ux/lib/litmus-self-heal.sh`), mirroring the EXACT precedent
`quality/gates/agent-ux/dark-factory/reconciliation-fixture.sh` already establishes for
this exact reason. That file's header (lines 1-11) is the copy-paste template:
```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory/reconciliation-fixture.sh -- RBF-A-05
# (p86 F13) shared fixture-writer + assert helpers for dvcs-third-arm.sh.
#
# Sourced (like lib.sh), NOT invoked directly -- factored out of
# dvcs-third-arm.sh per the file-size-limits progressive-disclosure budget
# (quality/gates/structure/file-size-limits.sh, .sh files = 10k chars).
#
# Relies on `fail_with` + `ASSERT_LOG` already being in scope from lib.sh,
# which dvcs-third-arm.sh sources BEFORE this file.
```

**Pattern 2 insertion (fixture pre-flight — FIRST step, before `git clone`):** insert
immediately after the function-local `local` declarations, before line 26 (`echo "\$ git
clone ..."`). Exact code (RESEARCH.md Pattern 2, already using
`scripts/confluence_tokenworld.py`'s real CLI contract verified against the script's
`main()` dispatch, `confluence_tokenworld.py:162-179`):
```bash
for pid in 2818063 7766017 7798785; do
  python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" restore "$pid" >&2 || true
done
python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" inspect 7798785 | \
  grep -q '"parentId": null' && \
  python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" reparent 7798785 7766017
```
(`REPO_ROOT` is already exported by the caller per this file's own header contract —
"Caller MUST export before sourcing/invoking: BIN, SPACE, MIRROR_URL, PROTECTED_IDS,
STATE_FILE, REPO_ROOT" — line 13-14.)

**Pattern 1 insertion (mirror pre-reconcile — AFTER `reposix attach` config-check,
BEFORE GUARD A):** insert after line 39 (the `fail "attach did not set expected git
config..."` branch's closing `fi`), before line 41's `# GUARD A:` comment. Exact code
(RESEARCH.md Pattern 1, adapted from `scripts/refresh-tokenworld-mirror.sh:111-129`'s
proven fetch+overlay sequence — note the SAME `git rm -r --quiet --ignore-unmatch` +
`checkout FETCH_HEAD --` ordering, not a simpler `checkout -- pages/`, so backend-side
deletions propagate):
```bash
git -C "$tree" fetch --quiet reposix main
git -C "$tree" rm -r --quiet --ignore-unmatch -- pages/ >/dev/null 2>&1 || true
git -C "$tree" checkout FETCH_HEAD -- pages/
git -C "$tree" add -A pages/
if ! git -C "$tree" diff --cached --quiet; then
  git -C "$tree" commit --quiet -m "litmus pre-reconcile: sync mirror clone to backend-current"
fi
```
**Match this file's existing config conventions exactly** — `git -C "$tree" config
user.email "litmus@reposix.invalid"` / `user.name "reposix-litmus"` are set later at line
84-85 (before the marker commit); the pre-reconcile commit above ALSO needs
`user.email`/`user.name` set first (git will fail without them) — move those two `git
config` lines UP to run once before Pattern 1's commit, or duplicate them immediately
before this block. Reuse the exact values already used at lines 84-85, do not invent new
ones.

**`errexit`-off discipline (load-bearing, RESEARCH.md Project Constraints):** this file
runs under the caller's `set -uo pipefail` (NO `errexit`) — every new git subcommand
above already uses explicit `||`/`if` guards matching the existing style throughout the
function (e.g. line 27's `|| { fail ...; return 1; }`), not bare unguarded commands.

**Remote-explicit convention already correct in this file (the pattern SC3's Rust hint
should match):** lines 94-99's own comment explains WHY the existing bounded self-heal at
line 105 uses `git -C "$tree" pull --rebase reposix main` (remote-explicit) and NEVER
`origin` — this is the prose the write_loop.rs hint (above) should echo in spirit:
> "NEVER `origin` (the stale mirror; after attach `branch.<b>.remote` still points at
> origin for fetch, so the recovery is remote-explicit on purpose)."

---

### `docs/reference/testing-targets.md` (docs, reference) — SC1 / DRAIN-02

**Analog: this file's own `## Pre-flight verification` section (lines 23-45)** — same
doc, same "before you run a real-backend test, run this script" shape SC1 needs for the
mirror-refresh pre-step. Copy the section's structure (heading → one-sentence rationale →
fenced bash block → outcome table), NOT its content:
```markdown
## Pre-flight verification

Before running any real-backend test (the `cargo test … --ignored
dark_factory_real_*` invocations below, or any P91+ phase that gates on a
real backend), run:

```bash
bash scripts/preflight-real-backends.sh
```
...
```

**Second analog for the callout-box phrasing convention: the `> **Milestone-close
cadence self-sources .env.**` blockquote at lines 271-280** — this is the existing
precedent for "a mandatory pre-step that a prior phase (P123/DRAIN-03) added and
cross-referenced from this exact doc." Match its `>` blockquote shape and its closing
sentence pattern ("...still need their own `export`s or a pre-sourced shell — only the
`run.py` cadence self-sources.") when writing SC1's new callout.

**Concrete insertion:** a new subsection near the `## Confluence — TokenWorld space`
section (after "Sacrificial editable page" subsection, lines 110-125, which already
documents `2818063` and `python3 scripts/confluence_tokenworld.py restore 2818063`) OR a
new top-level section mirroring `## Pre-flight verification`'s position, naming
`scripts/refresh-tokenworld-mirror.sh` explicitly (confirmed by RESEARCH.md: this script
currently has ZERO mentions in this doc). Content must state: (1) when to run it (before
`pre-release-real-backend` if not relying on SC2's self-heal), (2) that SC2 makes it
usually unnecessary (cross-reference, per RESEARCH.md Open Question 2's recommendation),
(3) the exact command `bash scripts/refresh-tokenworld-mirror.sh`.

---

### `docs/guides/troubleshooting.md` + `docs/concepts/dvcs-topology.md` (docs, fix-twice)

**Do NOT treat these as needing wholesale rewrite.** `dvcs-topology.md:89-90` ALREADY
shows the CORRECT `reposix sync --reconcile` phrasing — this doc is the thing write_loop.rs's
code drifted AWAY from (Pitfall 2), not the other way around. No edit needed here unless
SC3's NEW Pitfall-4 augmentation line introduces wording these docs should mirror.

**One live inconsistency worth flagging to the planner (not yet a proven bug — verify
before editing):** `troubleshooting.md`'s `### Bus-remote fetch first rejection` section
(lines 233-281) documents recovery as a BARE `git pull --rebase` (line 249:
`` git pull --rebase                 # fetch reconciles your cache against the SoT... ``),
while `dvcs-topology.md:152` states "Fetch is untouched: `git fetch` / `git pull` keep
reading from the mirror" for a Pattern-C attach tree — the exact Pitfall-4 gap. If SC3's
Rust hint change makes the remote-explicit distinction concrete, add ONE clarifying
sentence to `troubleshooting.md`'s "Bus-remote fetch first rejection" section (not a
rewrite) cross-referencing `litmus-flow.sh:94-99`'s own reasoning (quoted above) as the
canonical explanation of why the bus remote must be named explicitly on a Pattern-C tree.

**Doc-alignment binding rebind requirement (P117 W3 precedent, load-bearing):** any
wording change to a line already bound in `quality/catalogs/doc-alignment.json` (19
`grep` hits for `"git pull --rebase"` confirmed in this session) requires re-running the
bind in the SAME commit — do not leave a rebind for a later commit (re-drifts the
binding). Use `reposix-quality doc-alignment bind` / the `/reposix-quality-refresh`
fan-out per `quality/CLAUDE.md` § "Catalog-first rule" — never hand-edit the JSON.

---

### `scripts/refresh-tokenworld-mirror.sh` (utility, reused — NOT modified)

Read in full (160 lines). This is the SOURCE mechanism `litmus-flow.sh`'s Pattern 1
insertion adapts (see above) — do not re-derive a second implementation. Its own header
(lines 26-40, "THE MECHANISM") is the authoritative explanation of WHY `git rm` +
`checkout FETCH_HEAD --` (not a plain `checkout -- pages/`) is required — copy that
reasoning into any new comment the planner adds around the litmus-flow.sh insertion, do
not silently drop the `git rm` step (RESEARCH.md's own "Don't Hand-Roll" table flags this
exact omission risk).

### `scripts/confluence_tokenworld.py` (utility, reused — NOT modified)

Read in full (184 lines). Argument/output contract used by Pattern 2 above:
- `restore <page_id>` — exit 0 if already current (idempotent no-op, `cmd_restore` line
  130-147), exit 0 on successful un-trash, exit 1 on HTTP failure. Prints
  `"{page_id} already current (no-op)"` or `"restored {page_id} -> status current
  (version N)"`.
- `inspect <page_id>` — prints a JSON object to stdout with keys `id`, `title`, `status`,
  `parentId`, `parentType`, `spaceId`, `version` (exact shape at `cmd_inspect` lines
  67-81) — this is what Pattern 2's `grep -q '"parentId": null'` parses.
- `reparent <child_id> <parent_id>` — PUTs a version-bumped body preserving the storage
  content; prints `"reparented {child} -> parent {id} (version N)"`.
- `delete <page_id>` — REFUSES protected ids (`{"7766017", "7798785"}`), exit 2 in that
  case. Not used by this phase (only `restore`/`reparent`/`inspect` are).

---

## Shared Patterns

### Pin: literal substring `"git pull --rebase"` must survive verbatim

**Source:** confirmed via `grep -c "git pull --rebase" quality/catalogs/doc-alignment.json`
→ **19** hits (RESEARCH.md's own count was "≥12"; this session's live grep confirms it's
higher). Additional Rust pins:
- `crates/reposix-remote/tests/bus_precheck_b.rs:191` — `stderr.contains("git pull --rebase")`
- `crates/reposix-remote/tests/push_conflict.rs:198` — `stderr.contains("git pull --rebase")`
- `crates/reposix-cli/tests/agent_flow.rs:188` — `src.contains("git pull --rebase")` (scans SOURCE, not runtime stderr)

**Apply to:** any edit touching `write_loop.rs` lines 189-225 or `bus_handler.rs` lines
192-199/237-253. ADD new lines; never replace/reword the existing phrase.

### Credential redaction before any new interpolated string

**Source:** `crates/reposix-remote/src/bus_handler.rs:453-458` (`redact_userinfo`) +
`crates/reposix-remote/src/bus_handler.rs:134` (`strip_url_userinfo` for a lone
well-formed URL). **Apply to:** any new hint in `write_loop.rs` that interpolates a
remote name resolved from a URL, or any new litmus-flow.sh echo of `$MIRROR_URL`/`$rurl`.
Full two-redactor decision rule: `crates/CLAUDE.md` § "Credential redaction before ANY
error/stderr echo."

### Remote-explicit git recovery (never bare `origin` on a Pattern-C tree)

**Source:** `quality/gates/agent-ux/lib/litmus-flow.sh:94-99` (comment + line 105's
`git -C "$tree" pull --rebase reposix main`). **Apply to:** `write_loop.rs`'s new
Pitfall-4 hint line, and any doc text describing Pattern-C recovery.

### `errexit`-off / explicit exit-code checking

**Source:** `quality/gates/agent-ux/lib/litmus-flow.sh` throughout (e.g. lines 27, 47,
59, 72, 101-110) — every mutating step is followed by `||`/`if` guards, never bare.
**Apply to:** any new bash inserted into `_litmus_flow()`.

### Leaf isolation (`/tmp`-only mutating git/reposix calls, same Bash invocation)

**Source:** `scripts/refresh-tokenworld-mirror.sh`'s `mktemp -d` pattern (line 100) +
`litmus-flow.sh`'s own `run="$(mktemp -d -t litmus-run.XXXXXX)"` (line 21). **Apply to:**
any new self-heal code — already satisfied by both Pattern 1 and Pattern 2 insertions
above since they operate on `"$tree"`, which is already inside the caller's `mktemp -d`.
Enforced mechanically by `.claude/hooks/leaf-isolation-guard.sh` for interactive agent
Bash calls; this harness's own git/reposix calls run inside a script, not the Bash tool,
so the hook doesn't fire on them — the prose discipline is what makes them safe.

## No Analog Found

None. Every file in scope has a same-file or same-crate/same-directory analog — this
phase is explicitly composition of existing, already-proven mechanisms (RESEARCH.md's
own "Don't Hand-Roll" framing), not new pattern design.

## Metadata

**Analog search scope:** `crates/reposix-remote/{src,tests}/`, `crates/reposix-cache/src/`,
`quality/gates/agent-ux/{,lib/,dark-factory/}`, `scripts/`, `docs/{reference,guides,concepts}/`.
**Files scanned:** 11 (all in RESEARCH.md's Sources list + `crates/reposix-cache/src/mirror_refs.rs`
grepped live this session for the `write_mirror_synced_at` signature).
**Pattern extraction date:** 2026-07-18
**New live facts this session surfaced beyond RESEARCH.md (useful for the planner):**
- `quality/gates/agent-ux/lib/litmus-flow.sh` is 9794 bytes today — 206 bytes of headroom
  before the 10k ceiling (currently WAIVED/`--warn-only` until 2026-08-08, so not a hard
  blocker, but tight enough that the planner should decide up front whether to factor a
  new `lib/litmus-self-heal.sh` (precedent: `dark-factory/reconciliation-fixture.sh`) or
  accept the WARN.
- `doc-alignment.json` has 19 literal-substring hits for `"git pull --rebase"`, not the
  research's estimated "≥12" — the pin is even more load-bearing than stated.
- `Cache::write_mirror_synced_at(sot_host: &str, ts: DateTime<Utc>) -> Result<String>`
  (`crates/reposix-cache/src/mirror_refs.rs:155`) is the exact call the new Rust test
  needs to pre-populate the mirror-lag-ref branch — not previously named at file:line
  precision in RESEARCH.md.
