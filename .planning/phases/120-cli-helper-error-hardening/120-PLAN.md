---
phase: 120-cli-helper-error-hardening
plan: 120
type: execute
wave: 0                    # Entry wave. This is a SEQUENTIAL 7-wave phase (W0..W6); see <waves>.
depends_on: []
requirements: [UX-01]
autonomous: true          # No human checkpoints — every disposition (RETROFIT vs EXEMPT) is BAKED (see <disposition_table>), not gated.
user_setup: []

# Sequential single-tree-writer phase: ONE executor lane per wave, waves run in order,
# never two cargo builds concurrently (crates/CLAUDE.md build-memory budget). Files may
# recur across sequential waves (strict ordering guarantees no concurrent writer); files
# in the SAME wave never overlap.
waves:
  W0: "Catalog-first — mint 2 agent-ux rows + author 2 gate scripts + the shared multi-line-aware teach_scan.py + scaffold 2 Rust test files. FIRST COMMIT, before any impl."
  W1: "Shared teaching-error builder (reposix-core::errmsg) + init.rs/attach.rs adoption + shared spec-parse & env-var & cache-build helpers."
  W2: "list.rs / refresh.rs / spaces.rs / sync.rs — adopt the helper; dedupe the 3 identical spaces bails; cache-build wrapper; cache_db.rs exempt markers."
  W3: "gc.rs / history.rs / tokens.rs / cost.rs — adopt the helper; SHARED missing_cache_db_error + worktree_helpers.rs no-reposix-tree teaching (fixes history/tokens/cost/gc at the source); cli/main.rs `reposix log` gate."
  W4: "reposix-remote helper ENTRY/URL/CRED surface — main.rs (usage, bus-url wrapper, per-action context) + bus_url.rs (6 malformed-bus-URL sites) + backend_dispatch.rs (missing_env_error retrofit + machine-URL exempt markers)."
  W5: "reposix-remote helper TRANSPORT/FAN-OUT surface — stateless_connect.rs (upload-pack subprocess, EOF) + write_loop.rs (backend-unreachable reject retrofit + already-teaching exempt markers) + bus_handler.rs + precheck.rs triage/markers."
  W6: "crates/CLAUDE.md convention fix-twice (shared builder + doctor.rs exception + teach-exempt marker convention) + final gate-green + verifier handoff."

files_modified:
  # W0
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/cli-errors-teach-recovery.sh
  - quality/gates/agent-ux/helper-errors-teach-recovery.sh
  - quality/gates/agent-ux/teach_scan.py
  - crates/reposix-cli/tests/errors_teach_recovery.rs
  - crates/reposix-remote/tests/errors_teach_recovery.rs
  # W1
  - crates/reposix-core/src/errmsg.rs
  - crates/reposix-core/src/lib.rs
  - crates/reposix-cli/src/init.rs
  - crates/reposix-cli/src/attach.rs
  # W2
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/refresh.rs
  - crates/reposix-cli/src/spaces.rs
  - crates/reposix-cli/src/sync.rs
  - crates/reposix-cli/src/cache_db.rs
  # W3
  - crates/reposix-cli/src/gc.rs
  - crates/reposix-cli/src/history.rs
  - crates/reposix-cli/src/tokens.rs
  - crates/reposix-cli/src/cost.rs
  - crates/reposix-cli/src/worktree_helpers.rs
  - crates/reposix-cli/src/main.rs
  # W4
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/bus_url.rs
  - crates/reposix-remote/src/backend_dispatch.rs
  # W5
  - crates/reposix-remote/src/stateless_connect.rs
  - crates/reposix-remote/src/write_loop.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-remote/src/precheck.rs
  # W6
  - crates/CLAUDE.md

must_haves:
  truths:
    - "A user who runs `reposix init` at an existing repo root, `reposix list` against an unreachable sim, a `confluence::`/`jira::` spec with the tenant env var unset, `reposix tokens`/`cost` before any fetch, `reposix log` without `--time-travel`, or any other enumerated CLI error path sees a message that (1) teaches the fix, (2) names the alternative command (omitted only when there is no genuine alternative), (3) gives a copy-paste recovery line."
    - "A user (or git) that trips a `reposix-remote` helper error — malformed bus URL (all 6 bus_url.rs arms), missing real-backend creds, upload-pack subprocess failure, unexpected EOF mid-request, backend-unreachable push reject — sees the same 3-part teaching message on stderr; git-protocol status strings (`fetch first`, `backend-unreachable`) stay verbatim as their accompanying stderr diag carries the teaching."
    - "EVERY user-facing-adjacent error site on the enumerated CLI + helper surface is DISPOSITIONED: it either routes through the shared builder/helpers (RETROFIT) or carries an explicit `// teach-exempt: ok — <reason>` marker (EXEMPT). No un-dispositioned user-facing error remains (see <disposition_table>)."
    - "The bespoke already-3-part sites (init.rs refuse_existing_repo_root / fetch-sync-failed, list.rs sim_unreachable_message, helper BLOB_LIMIT/UNFILTERED_FETCH_HINT consts, write_loop.rs conflict reject) still emit their teaching strings unchanged AND carry an explicit teach-exempt marker — no regression, no reliance on incidental single-line-scan skipping."
    - "The agent-ux contract rows + gate scripts + the multi-line-aware teach_scan.py for this surface are committed in the phase's FIRST commit, before any implementation commit (git log shows the catalog SHA predates every impl SHA)."
    - "A source-scan gate with an EXPLICIT documented file scope that MATCHES the retrofit set rejects any NEW teaching-less error site — single-line OR multi-line `bail!`/`anyhow!` block — added to the enumerated CLI/helper surface, so the bar cannot silently rot; residual scanner limitations are documented, not overclaimed."
  artifacts:
    - path: "crates/reposix-core/src/errmsg.rs"
      provides: "Shared teaching-error builder (Teach) + teach() convenience fn; empty-alternative suppresses its line; optional P121 error-code slot wired"
      contains: "pub fn teach"
      min_lines: 40
    - path: "quality/catalogs/agent-ux.json"
      provides: "Two contract rows: agent-ux/cli-errors-teach-recovery, agent-ux/helper-errors-teach-recovery"
      contains: "cli-errors-teach-recovery"
    - path: "quality/gates/agent-ux/cli-errors-teach-recovery.sh"
      provides: "Reality-exercising gate: runs the nextest suite + drives real CLI error paths + invokes teach_scan.py over the CLI scope"
    - path: "quality/gates/agent-ux/helper-errors-teach-recovery.sh"
      provides: "Reality-exercising gate for the reposix-remote helper surface + teach_scan.py over the helper scope"
    - path: "quality/gates/agent-ux/teach_scan.py"
      provides: "Multi-line-aware source scanner: parses each bail!/anyhow!/return Err(anyhow! block (balanced parens) over an EXPLICIT file allowlist; RAISEs any block lacking teach()/a shared helper AND a `// teach-exempt: ok` marker"
      contains: "teach-exempt"
    - path: "crates/reposix-cli/tests/errors_teach_recovery.rs"
      provides: "Per-error integration assertions (exact substrings) for the CLI surface — runs in normal nextest cadence"
      contains: "assert_cmd"
    - path: "crates/reposix-remote/tests/errors_teach_recovery.rs"
      provides: "Per-error integration assertions for the helper surface (malformed bus URL, missing creds, EOF/subprocess)"
    - path: "crates/CLAUDE.md"
      provides: "Updated error-message-convention section: shared builder location + doctor.rs structured-report exception + teach-exempt marker convention"
      contains: "errmsg"
  key_links:
    - from: "crates/reposix-cli/src/*.rs error sites"
      to: "reposix_core::errmsg::{teach, Teach}"
      via: "bail!(\"{}\", teach(headline, fix, alternative, &[recovery...]))"
      pattern: "errmsg::teach|teach\\(|Teach::new"
    - from: "crates/reposix-remote/src/{main,bus_url,backend_dispatch,stateless_connect,write_loop}.rs error sites"
      to: "reposix_core::errmsg::{teach, Teach}"
      via: "anyhow::bail!(\"{}\", teach(...)) / Teach::new(hd)...to_string()"
      pattern: "errmsg::teach|teach\\(|Teach::new"
    - from: "quality/gates/agent-ux/{cli,helper}-errors-teach-recovery.sh"
      to: "quality/gates/agent-ux/teach_scan.py + crates/**/tests/errors_teach_recovery.rs + real binaries"
      via: "python3 teach_scan.py --scope <files> + cargo test --test errors_teach_recovery + assert_cmd grep stderr for Fix:/Recovery:"
      pattern: "teach_scan|errors_teach_recovery"
---

<objective>
Bring EVERY user-facing `reposix` CLI subcommand error AND every `reposix-remote` git-helper
error up to the 3-part Rust-compiler-grade bar the project already claims in `crates/CLAUDE.md`
and demonstrates in `init.rs::refuse_existing_repo_root`: (1) **teach the fix**, (2) **suggest
the alternative**, (3) **give a copy-paste recovery command**.

The inventory below found ~40 teaching-less or partial error sites across 15 CLI subcommands and
the FULL helper surface (single-backend + bus-URL + transport + fan-out). Rather than hand-edit
40 prose strings blind, this phase introduces ONE shared teaching-error builder
(`reposix_core::errmsg::{Teach, teach}`) plus shared teaching helpers for the repeated failure
shapes (spec-parse, env-var-setup, cache-build, missing-cache-db, malformed-bus-URL), then
retrofits or EXEMPT-marks each surface. A multi-line-aware source-scan gate over an EXPLICIT file
scope makes the bar mechanically un-rottable going forward.

Every user-facing-adjacent site on the enumerated surface receives a disposition — RETROFIT (route
through the builder) or EXEMPT (`// teach-exempt: ok — <reason>` marker). The <disposition_table>
is the coverage contract: it leaves NO user-facing error un-dispositioned.

Purpose: the phase's literal goal IS the project north star (OD-3 §5 / Rust-compiler-grade UX) —
would a skeptical dev hitting this error for the first time come away impressed?
Output: a shared builder, ~40 upgraded error sites + a complete exempt-marker set, 2
reality-exercising agent-ux gates + a multi-line-aware scanner, 2 Rust integration suites
(continuous regression), and a fix-twice `crates/CLAUDE.md` update.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@crates/CLAUDE.md
@quality/CLAUDE.md

<interfaces>
<!-- The BAR (already-3-part exemplar — DO NOT MODIFY its emitted strings; ADD a teach-exempt marker): -->
crates/reposix-cli/src/init.rs::refuse_existing_repo_root (~L142-153, call site L258): a bail! that
names the corruption shape, points at `reposix attach` as the alternative, and prints indented
runnable recovery lines. list.rs:172-180 `sim_unreachable_message()` is a reusable shared-pattern
precedent. init.rs:365-373 (fetch sync failed) already has a `Fix:` block.

<!-- NEW shared contract this phase creates (reposix-core::errmsg). Both crates depend on
     reposix-core, so this is the single shared home. A BUILDER (not just a free fn) so P121 can
     add RPX-xxxx codes to individual sites without churning the other ~40 call sites (FLAG 2), and
     so an empty alternative omits its line (FLAG 1). -->
```rust
/// Builder for a Rust-compiler-grade, 3-part teaching-error body.
/// Rendered SHAPE is contract (the gate + tests grep for these anchors):
///   <headline>
///   Fix: <fix>
///   Alternative: <alternative>      // OMITTED when alternative is unset/empty (FLAG 1)
///   Recovery:                       // OMITTED when the recovery slice is empty
///     <recovery[0]>
///     <recovery[1]>
/// Forward-compat (FLAG 2 / P121): `.code("RPX-xxxx")` slot is wired NOW and renders NOTHING while
/// unset. P121 owns the code render format and adds `.code(...)` to the individual sites that need
/// a code — the other ~40 call sites are untouched by that change.
pub struct Teach<'a> { /* headline; fix; alternative: Option<&str>; recovery: &[&str]; code: Option<&str> */ }
impl<'a> Teach<'a> {
    pub fn new(headline: &'a str) -> Self;
    #[must_use] pub fn fix(self, fix: &'a str) -> Self;
    #[must_use] pub fn alternative(self, alt: &'a str) -> Self;   // "" == unset == omit the line
    #[must_use] pub fn recovery(self, recovery: &'a [&'a str]) -> Self;
    #[must_use] pub fn code(self, code: &'a str) -> Self;          // P121 slot; unset renders nothing
}
impl std::fmt::Display for Teach<'_> { /* renders the contract shape */ }

/// Convenience free-fn for the common (headline, fix, alternative, recovery) shape. An EMPTY
/// `alternative` ("") omits the Alternative line (FLAG 1). Delegates to `Teach`; no code slot —
/// use the builder directly when a site needs `.code(...)`. Most of the ~40 call sites use this fn.
pub fn teach(headline: &str, fix: &str, alternative: &str, recovery: &[&str]) -> String;
```
Call shapes: `bail!("{}", reposix_core::errmsg::teach(hd, fix, alt, &[rec]))` (common), or
`bail!("{}", Teach::new(hd).fix(f).recovery(&[rec]))` when the alternative is genuinely absent
(e.g. stateless_connect.rs:331 "unexpected EOF mid-request" — a protocol desync with no alternative).

<!-- The shared FAILURE-SHAPE helpers (W1 creates the first three; W3 adds missing_cache_db_error;
     W4 adds malformed_bus_url_error): -->
- `spec_parse_error(spec, cause) -> anyhow::Error` — the `<backend>::<project>` parse failure shared
  by init/attach/sync/refresh (root: init.rs::translate_spec_to_url).
- `missing_env_var_error(var, backend, example) -> anyhow::Error` — Confluence/JIRA tenant/instance
  unset (CLI side); MUST emit `export <VAR>=<value>` + a retry line + a `sim::` alternative.
- `cache_build_error(backend, path, source) -> anyhow::Error` — the `.context("build cache from
  backend")` wrapper: surfaces the connector's own message AND names the likely root cause
  (sim not running / missing creds) with a runnable recovery.
- `missing_cache_db_error(cache_path) -> anyhow::Error` (W3) — the SHARED "no cache / no cache.db"
  teaching for tokens/cost (and any subcommand reading `<cache>/cache.db`). Teaches "run `git fetch`
  / `reposix refresh` to populate the ledger" + recovery. See <disposition_table> note on why this
  — NOT `cache_db.rs` — is the real shared source for tokens/cost.
- `malformed_bus_url_error(reason, got) -> anyhow::Error` (W4) — the SHARED teaching for all 6
  bus_url.rs reject arms: names the canonical `reposix::<sot-spec>?mirror=<mirror-url>` form + a
  corrected example built from `got`.
</interfaces>
</context>

<ownership_charter>
Embedded verbatim per OD-3 — every executor dispatched against a wave of this plan owns what it touches:
1. You own what you touch. Acceptance criteria are the FLOOR, not the ceiling — done means "I'd
   defend this in review as excellent," not "plan executed."
2. Noticing is a deliverable — report lying doc claims, tests that don't assert what their names
   promise, teaching-free errors, dead code, stale comments, missing edge cases. An empty noticing
   section from code-touching work is a red flag.
3. Eager-fix or file — never silently skip. <1h + no new dependency → fix in place; else → file to
   `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with severity + sketch.
4. Verify against reality — build it, EXERCISE the error path, grep the real stderr. A claim without
   an artifact is not done.
5. North star: Rust-compiler-grade UX — would a skeptical first-time dev hitting this error come away
   impressed? This IS the phase's literal goal.

Standing NOTICING carried from planning (verify + fold into your wave's report):
- **cache_db.rs is NOT the tokens/cost path** (plan-check claimed it was). `tokens.rs`/`cost.rs` open
  `<cache>/cache.db` directly (tokens.rs:81/84, cost.rs:113/116); `cache_db.rs` is the separate
  `reposix refresh` `refresh_meta` store at `<worktree>/.reposix/cache.db`. The corrected dispositions
  are in <disposition_table>.
- **bus_handler.rs:442** (`git ls-remote {mirror_url} failed: ...`) echoes `mirror_url` RAW while the
  sibling error paths at L134/172/191 apply `strip_url_userinfo`. Potential credential leak on stderr.
  W5 executor: eager-fix (apply `strip_url_userinfo` before echo) if <1h, else file to SURPRISES-INTAKE.
</ownership_charter>

<shared_design>
## The shared builder + leverage decisions (BAKED — no per-wave re-litigation)

**Builder (`reposix-core::errmsg::{Teach, teach}`).** Pure string formatting, no new deps,
`forbid(unsafe_code)` clean, no anyhow in core (callers wrap with `bail!`/`anyhow!` at the binary
boundary per crates/CLAUDE.md). The rendered shape (`Fix:` / optional `Alternative:` / `Recovery:` +
2-space-indented command lines) is the mechanical contract the gate greps. Two FLAG resolutions are
baked into the builder: empty `alternative` omits the line (FLAG 1); an optional `.code()` slot is
wired now and renders nothing until P121 populates it (FLAG 2).

**Leverage #1 — spec-parse:** `translate_spec_to_url` (init.rs) is shared by init/attach/sync/refresh.
Upgrade its error arms ONCE via `spec_parse_error`; all four subcommands inherit the teaching message.

**Leverage #2 — env-var setup (CLI + helper):** CLI Confluence/JIRA errors → `missing_env_var_error`
(emits `export …=<value>` + retry + `sim::` alternative). The HELPER has its own equivalent —
`backend_dispatch::missing_env_error` (backend_dispatch.rs:362) — retrofit that ONCE and every
GitHub/Confluence/JIRA helper instantiate inherits it (W4).

**Leverage #3 — the `.context("build cache from backend")` epidemic (DECISION):** SURFACE the
connector's own message AND wrap it in a teaching `cache_build_error` that names the likely cause +
recovery — do NOT swallow the source (`(underlying: <source>)` appended).

**Leverage #4 — the tokens/cost "no cache.db" source (CORRECTED):** `tokens.rs`/`cost.rs` share the
IDENTICAL "no cache / no cache.db" bail shape and both read `<cache>/cache.db` directly. Retrofit a
SHARED `missing_cache_db_error(cache_path)` (W3) consumed by both. SEPARATELY, the upstream "not a
reposix tree / no reposix remote" teaching for history/tokens/cost/gc lives in ONE place —
`worktree_helpers.rs::cache_path_from_worktree` (worktree_helpers.rs:186) — retrofit THAT once and all
four subcommands inherit it. (This is the real "fix at the source" the plan-check reached for; it is
NOT `cache_db.rs`, which serves `reposix refresh` only.)

**Leverage #5 — malformed bus URL:** all 6 bus_url.rs reject arms share the "malformed
`reposix::<sot>?mirror=<url>`" shape. Retrofit ONE `malformed_bus_url_error` (W4) consumed by all six;
fix main.rs:129's `.context("parse remote url")` wrapper so the teach() body surfaces cleanly instead
of being buried behind the terse context prefix.

**doctor.rs is a documented EXCEPTION.** `doctor` returns a structured `DoctorReport` (not exceptions).
Do NOT force `bail!`s into it. W6 documents this exception in crates/CLAUDE.md.

**Git-protocol status strings are a documented EXEMPT class.** The helper emits contract strings git
parses — `error refs/heads/main fetch first`, `... backend-unreachable`, `... bulk-delete`,
`... invalid-blob:<path>`. These CANNOT be arbitrary teaching text (git renders them in its own
`! [remote rejected]` format). The teaching belongs in the ACCOMPANYING `crate::diag` stderr line;
many already teach (PRECHECK A/B hints, conflict "Run: git pull --rebase"). Where the accompanying
diag does NOT teach (write_loop.rs backend-unreachable at L172), W5 retrofits the DIAG, not the
protocol string. Every such site carries a teach-exempt marker on the protocol-string line naming
"git-protocol contract; teaching in the diag above/below."

## Gate design (reality-first, un-rottable — MATCHED SCOPE, NO GREEN-WASHING)
- **Continuous regression = Rust integration tests** in `crates/reposix-cli/tests/errors_teach_recovery.rs`
  and `crates/reposix-remote/tests/errors_teach_recovery.rs` (normal `cargo nextest` cadence). Each
  per-error test drives the REAL binary via `assert_cmd` (CLI) or a spawned helper / malformed-URL
  invocation (remote) and asserts the EXACT fix/recovery substrings. This is where "verify against
  reality" lives.
- **Phase-close gate = 2 on-demand agent-ux scripts** (`cadences: ["on-demand"]`, dark-factory
  precedent, so the catalog-first commit does NOT self-block intermediate pushes). Each script (a)
  runs its nextest suite, (b) EXERCISES ≥1 real error path end-to-end and greps stderr for `Fix:` +
  an indented `Recovery:` line, (c) invokes the shared **`teach_scan.py`** over its EXPLICIT scope.
- **`teach_scan.py` — multi-line-aware, explicit-scope, honest about its limits:**
  - **Explicit file scope (matches the retrofit set, NOT a broad glob):** CLI scope = the enumerated
    `crates/reposix-cli/src/{init,attach,list,refresh,spaces,sync,gc,history,tokens,cost,worktree_helpers,cache_db,main}.rs`;
    helper scope = `crates/reposix-remote/src/{main,bus_url,backend_dispatch,stateless_connect,write_loop,bus_handler,precheck}.rs`.
    The scope list lives at the top of the script as a documented constant; adding a new subcommand
    file to the surface = adding it to the scope list (a reviewable one-line change, not a silent gap).
  - **Multi-line detection (closes the REVERSE HOLE):** the scanner does NOT scan single lines. It
    tokenizes each file, finds every `bail!(` / `anyhow!(` / `return Err(anyhow!(` invocation, and
    accumulates lines until the parens balance (the FULL macro block, single- or multi-line). A block
    RAISEs unless it EITHER (i) calls `teach(` / `Teach::new` / a named shared helper
    (`spec_parse_error|missing_env_var_error|cache_build_error|missing_cache_db_error|malformed_bus_url_error|missing_env_error`),
    OR (ii) contains the literal anchors `Fix:` AND `Recovery:` in its own string literals, OR (iii)
    is immediately preceded (within 2 lines, comments only) by a `// teach-exempt: ok` marker. A NEW
    teaching-less MULTI-LINE `bail!` therefore CANNOT evade the scan — the reverse hole is closed by
    construction, not by luck.
  - **Documented residual limitations (HONEST, in the script header + this plan):** the scanner keys
    on lexical presence. It does NOT resolve a `bail!(some_var)` whose string was built elsewhere
    without teaching (indirection escapes), nor `format!`-into-`bail!` where the anchors live in a
    helper the scanner doesn't know by name. These are LOW-likelihood (the codebase's convention is
    inline literals or the named helpers) and are stated as known gaps — NOT papered over with an
    "un-rottable" absolute. The claim is "un-rottable for the direct inline `bail!`/`anyhow!` pattern
    over the enumerated scope, single- or multi-line," which is exactly what the scanner enforces.
  - Exit 0 = PASS; exit 1 with a teaching stderr naming the offending file:line + the fix (dogfood
    the bar) on any un-dispositioned block.

## Leaf isolation (HARD — any test/gate that runs `reposix init`/sim-seed)
Rust tests use `tempfile::TempDir` (existing tests/*.rs precedent) — never the shared repo. Shell
gates that run `reposix init` MUST `cd` into a `mktemp -d`-under-`/tmp` dir in the SAME invocation
(dark-factory.sh precedent; `.claude/hooks/leaf-isolation-guard.sh` fails closed otherwise). Most
teaching paths here fail BEFORE any network/seed (existing-repo-root, malformed spec/bus-URL,
unreachable sim) so they are hermetic.
</shared_design>

<disposition_table>
## COMPLETE helper + shared-CLI error-site disposition (coverage contract)

RETROFIT = routes through the builder/a shared helper. EXEMPT = carries `// teach-exempt: ok — <reason>`.
Every user-facing-adjacent site below is dispositioned. Executors VERIFY each against the source and
add the marker or the retrofit; an un-dispositioned site is a wave failure.

### Item 1 — bus_url.rs (malformed bus URL) + its wrapper  [W4]
| Site | Disposition | Notes |
|------|-------------|-------|
| bus_url.rs:88 (`+`-form dropped) | RETROFIT | via `malformed_bus_url_error` |
| bus_url.rs:98 (base form rejected) | RETROFIT | via `malformed_bus_url_error` (wraps parse_remote_url) |
| bus_url.rs:106 (empty query) | RETROFIT | via `malformed_bus_url_error` |
| bus_url.rs:132 (unknown query param) | RETROFIT | via `malformed_bus_url_error` |
| bus_url.rs:141 (`mirror=` missing) | RETROFIT | via `malformed_bus_url_error` |
| bus_url.rs:148 (`mirror=` empty) | RETROFIT | via `malformed_bus_url_error` |
| main.rs:129 (`.context("parse remote url")` wrapper) | RETROFIT | drop the burying context so the teach() body surfaces cleanly in git stderr |

### Item 2 — bus_handler.rs / backend_dispatch.rs / precheck.rs
| Site | Disposition | Notes | Wave |
|------|-------------|-------|------|
| bus_handler.rs:139 (bad-mirror-url `-` prefix) | RETROFIT | user's bus URL; teach the `?mirror=<url>` form | W5 |
| bus_handler.rs:156-161 (egress-denied) | EXEMPT | `diag(denied.teaching_message())` at L156 already emits full teaching | W5 |
| bus_handler.rs:173-177 (no-mirror-remote, Q3.5) | EXEMPT | already teaches `git remote add <name> <url>` (fix+recovery) | W5 |
| bus_handler.rs:184-206 (PRECHECK A drift) | EXEMPT | diag hints already teach `git fetch <name>`; `fetch first` = git-protocol contract | W5 |
| bus_handler.rs:235-259 (PRECHECK B drift) | EXEMPT | diag hints already teach `git pull --rebase`; protocol contract | W5 |
| bus_handler.rs:273-278 (parse-error) | EXEMPT | git's own malformed fast-import stream; internal protocol error | W5 |
| bus_handler.rs:344-348 (mirror-fail WARN) | EXEMPT | already teaches retry/webhook recovery; WARN+ok ack, not a failure | W5 |
| bus_handler.rs:380 (`git config` subprocess exit) | EXEMPT | internal subprocess failure; not user-actionable teaching | W5 |
| bus_handler.rs:442 (`git ls-remote` failed) | EXEMPT | surfaces git's own descriptive stderr; internal transport wrap. NOTICING: raw mirror_url echo — see ownership_charter | W5 |
| bus_handler.rs:522 (push_mirror `-` reject) | EXEMPT | defense-in-depth, unreachable (name from git's remote-name validation) | W5 |
| bus_handler.rs:529 (push_mirror spawn) | EXEMPT | git invoked the helper ⇒ git on PATH; pathological | W5 |
| backend_dispatch.rs:112/114 (splitter/traversal) | EXEMPT | machine-generated URL from `reposix init`/`attach` | W4 |
| backend_dispatch.rs:167/170/176 (atlassian marker / unknown origin) | EXEMPT | machine-generated URL; already lists expected forms | W4 |
| backend_dispatch.rs:362 (missing_env_error) | RETROFIT | user-facing real-backend creds; add `export`+`sim::` alt+retry (leverage #2 helper side) | W4 |
| backend_dispatch.rs:298/317/324/332 (constructor) | EXEMPT | internal; origin machine-generated | W4 |
| backend_dispatch.rs:265/273 (audit DB open) | EXEMPT | documented OP-3 hard error; filesystem-level | W4 |
| precheck.rs:113/123 (`backend-unreachable` context) | EXEMPT | `.context()` annotation consumed by write_loop reject diag; not terminal | W5 |
| precheck.rs:230/251/325/331/348/377 (cache/REST context) | EXEMPT | internal annotations; terminal teaching at write_loop reject | W5 |

### Item 3 — CLI shared paths
| Site | Disposition | Notes | Wave |
|------|-------------|-------|------|
| cache_db.rs:101-105 (busy: "another refresh in progress") | EXEMPT | already teaches (unmount/wait). NOTE: this is the `reposix refresh` refresh_meta store, NOT tokens/cost | W2 |
| cache_db.rs:108 (map_busy generic fallback) | EXEMPT | internal SQLite open failure (disk/corrupt); not a teaching target | W2 |
| cache_db.rs:132 (update_metadata context) | EXEMPT | internal SQLite write failure | W2 |
| cli/main.rs:415 (`reposix log` requires `--time-travel`) | RETROFIT | user-facing; teach + name `reposix history` alt + recovery | W3 |
| worktree_helpers.rs:186 (no reposix remote) | RETROFIT | the REAL shared leverage: route through teach(); history/tokens/cost/gc inherit | W3 |
| tokens.rs:64 (no cache at path) + tokens.rs:81 (no cache.db) | RETROFIT | via shared `missing_cache_db_error` | W3 |
| cost.rs:113 (no cache.db) + cost.rs:260 | RETROFIT | via shared `missing_cache_db_error` | W3 |

### Already-3-part bespoke sites — EXEMPT WITH EXPLICIT MARKER (do NOT rely on incidental scan-skip)
| Site | Wave to add marker |
|------|--------------------|
| init.rs refuse_existing_repo_root (~L142-153) | W1 |
| init.rs:365-373 (fetch sync failed) | W1 |
| list.rs:172-180 (sim_unreachable_message) | W2 |
| stateless_connect.rs:54-62 (BLOB_LIMIT_EXCEEDED_FMT / UNFILTERED_FETCH_HINT consts) | W5 |
| write_loop.rs:207 (conflict `fetch first`; teaches via diag L183-186) | W5 |
</disposition_table>

<inventory>
## Grounded error-site inventory (file:line — DO NOT re-discover; verify then edit or mark)

ALREADY 3-PART — keep verbatim + add teach-exempt marker: init.rs refuse_existing_repo_root,
init.rs:365-373; list.rs:172-180; helper: stateless_connect.rs:54-62 consts, write_loop.rs:207
conflict (teaches via diag L183-186 above the protocol line).

W1 sites — init.rs / attach.rs (+ spec_parse/env-var/cache-build helpers):
- init.rs: 48/51/104-106 (→ spec_parse_error), 75-79 & 92-96 (→ missing_env_var_error), 376-379 (git
  not on PATH — partial, upgrade), 505-507 (no cache at path — partial), 273 (path UTF-8), 514-517 &
  559-562 (sync tag / update-ref). Add markers to refuse_existing_repo_root + 365-373.
- attach.rs: 384-395 (multi_sot_conflict_error — partial), 120 (not a git tree), 177 & 198
  (.context backend/reconcile → cache_build_error), 202 (duplicate ids).

W2 sites — list/refresh/spaces/sync (+ cache_db exempt markers):
- list.rs: 77-119 (.context → cache_build_error), 271 & 321 (format/scheme). Marker: 172-180.
- refresh.rs: 69-72 (--offline unimplemented), 206-244 (.context → cache_build_error), 308/321/352.
- spaces.rs: 26/29/32 (THREE identical "only supported for confluence" bails — dedupe to ONE helper).
- sync.rs: 91-95 (no reposix remote — partial), 99/116/120/130 (.context → cache_build_error).
- cache_db.rs: 101-105/108/132 → EXEMPT markers (refresh-meta store; see <disposition_table>).

W3 sites — gc/history/tokens/cost (+ shared missing_cache_db_error + worktree_helpers + cli/main.rs):
- gc.rs: 74 (strategy unimplemented), 103/110/287-313 (.map_err/.context).
- history.rs: 26 (not a reposix tree).
- tokens.rs: 64 (no cache at path) + 81 (no cache.db) → missing_cache_db_error.
- cost.rs: 67 (parse --since — partial), 113 (no cache.db) + 260 → missing_cache_db_error / teach.
- worktree_helpers.rs: 186 (no reposix remote) → RETROFIT (shared upstream for all four).
- cli/main.rs: 415 (`reposix log` requires --time-travel) → RETROFIT.
- doctor.rs — EXCEPTION (structured DoctorReport). No bail! retrofit; documented in W6.

W4 sites — reposix-remote ENTRY/URL/CRED:
- main.rs: 115 (usage), 129 (parse remote url wrapper → surface teach body), 367 (fail_push import
  "cannot list issues" — backend-unreachable, retrofit the DIAG detail), 458 (parse-error — internal,
  exempt marker), 546/571/593 (per-action .context → EXEMPT markers; terminal teaching at write_loop
  reject).
- bus_url.rs: 88/98/106/132/141/148 → malformed_bus_url_error.
- backend_dispatch.rs: 362 (missing_env_error) → RETROFIT; 112/114/167/170/176/298/317/324/332/265/273
  → EXEMPT markers.

W5 sites — reposix-remote TRANSPORT/FAN-OUT:
- stateless_connect.rs: 280 & 479 (git upload-pack subprocess exit → teach on top of raw stderr), 331
  (unexpected EOF mid-request → Teach::new, NO alternative). Markers: 54-62 consts.
- write_loop.rs: 172 (backend-unreachable reject DIAG → retrofit teach: sim not running / creds), 207
  (conflict — marker), 218/227/236 (bulk-delete / invalid-blob / duplicate-id — already teach, markers).
- bus_handler.rs / precheck.rs: markers per <disposition_table>; bus_handler.rs:139 RETROFIT.
</inventory>

<tasks>

## Wave 0 — Catalog-first (FIRST COMMIT — before ANY implementation commit) [reposix-cli + reposix-remote + quality]

<task type="auto">
  <name>W0.1: Mint the two agent-ux contract rows</name>
  <files>quality/catalogs/agent-ux.json</files>
  <action>
Add two rows (agent-ux rows are hand-edited per the catalog's own `_provenance_note`; mint the full
OP-3 shape or the loader rejects at load time — `_audit_field.py::validate_row`). Required for a NEW row:
`minted_at` (RFC3339, write-once), `coverage_kind`, `claim_vs_assertion_audit` (≥50 chars). Both rows:
`dimension:"agent-ux"`, `kind:"mechanical"`, `transport_claim:false`, `coverage_kind:"mechanical"`
(local sim / local binary error paths, NOT a real external backend — same honest rationale as
agent-ux/dark-factory-sim), `cadences:["on-demand"]`, `blast_radius:"P1"`, `waiver:null`,
`status:"NOT-VERIFIED"` (honest — the gate has not run yet; W6 flips to PASS after a green run + the
unbiased verifier confirms — never mint a PASS before the gate passes).

Row 1 `agent-ux/cli-errors-teach-recovery`: command `bash quality/gates/agent-ux/cli-errors-teach-recovery.sh`;
verifier {script: same, args:[], timeout_s:180, container:null}; artifact
`quality/reports/verifications/agent-ux/cli-errors-teach-recovery.json`; sources = the gate script + the
Rust test + `teach_scan.py` + `crates/CLAUDE.md` (error-message-convention). expected.asserts:
  - "cargo test -p reposix-cli --test errors_teach_recovery passes (per-error exact-substring assertions)"
  - "a real `reposix init` at an existing repo root emits Fix:/Alternative:/Recovery: on stderr"
  - "a real `reposix list` against an unreachable sim emits the 3-part teaching message"
  - "teach_scan.py over the CLI scope finds no un-dispositioned bail!/anyhow! block (single- or multi-line)"
Row 2 `agent-ux/helper-errors-teach-recovery`: mirror for `crates/reposix-remote` (command/script
`helper-errors-teach-recovery.sh`; test `cargo test -p reposix-remote --test errors_teach_recovery`;
real path = malformed bus URL OR missing creds; teach_scan.py over the helper scope). Write a truthful
`claim_vs_assertion_audit` (≥50 chars) stating these gates exercise the LOCAL binary/sim error paths,
not a real external backend, hence coverage_kind:mechanical.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && jq -e '[.rows[]|select(.id|endswith("teach-recovery"))] | (length==2) and (map(.minted_at!=null and .coverage_kind!=null and (.claim_vs_assertion_audit|length>=50))|all)' quality/catalogs/agent-ux.json</automated>
  </verify>
  <done>Both rows load-valid (minted_at RFC3339, coverage_kind, audit ≥50 chars), status NOT-VERIFIED, cadences on-demand.</done>
</task>

<task type="auto">
  <name>W0.2: Author teach_scan.py + the two gate scripts + scaffold the two Rust test files</name>
  <files>quality/gates/agent-ux/teach_scan.py, quality/gates/agent-ux/cli-errors-teach-recovery.sh, quality/gates/agent-ux/helper-errors-teach-recovery.sh, crates/reposix-cli/tests/errors_teach_recovery.rs, crates/reposix-remote/tests/errors_teach_recovery.rs</files>
  <action>
**teach_scan.py (the un-rottable core — build this FIRST, it is the reverse-hole fix).** Python3, no
non-stdlib deps. Two documented SCOPE constants at the top (CLI_SCOPE, HELPER_SCOPE) listing the exact
files from <shared_design> gate-design. `--scope cli|helper` selects. For each file: read the whole
text, walk it finding every `bail!(`, `anyhow!(`, `return Err(anyhow!(` start; from each start,
accumulate characters until the parens balance (respecting Rust string/char literals + line comments
so a `)` inside a string doesn't miscount) — this is the FULL macro block, single- OR multi-line. A
block PASSES iff it (i) mentions `teach(` / `Teach::new` / one of the named shared helpers
(`spec_parse_error|missing_env_var_error|cache_build_error|missing_cache_db_error|malformed_bus_url_error|missing_env_error`),
OR (ii) contains BOTH literal anchors `Fix:` and `Recovery:` in its own text, OR (iii) is preceded
within 2 lines (comments/blank only) by `// teach-exempt: ok`. Otherwise RAISE: print
`file:line: un-dispositioned error block — route through teach()/a shared helper or add '// teach-exempt: ok — <reason>'`
and exit 1. Header comment documents the residual limitations verbatim from <shared_design> (indirection
/ format!-into-bail! escapes) — HONEST, not "un-rottable" absolute. Add a `--self-test` mode with 3
inline fixtures (a teaching multi-line bail! PASSES, a teaching-less multi-line bail! RAISEs, a marked
bail! PASSES) so the scanner's own logic is regression-covered.

**Gate scripts** (model on bus-no-remote-configured-error.sh + dark-factory.sh; `set -euo pipefail`,
resolve REPO_ROOT, cd there): (a) run the crate's nextest suite for `errors_teach_recovery`; (b)
exercise ≥1 real error path end-to-end in a `/tmp` TMP=$(mktemp -d) with `cd "$TMP"` in the SAME
invocation, grep stderr for `Fix:` + an indented `Recovery:` command line; (c) run
`python3 "$REPO_ROOT/quality/gates/agent-ux/teach_scan.py" --scope cli|helper`. Exit 0 = PASS, exit 1
with a teaching stderr on any failure (dogfood the bar).

**Rust test scaffolds:** create both files with the workspace_root() helper (copy from cli.rs) and ONE
anchor test each that asserts the ALREADY-3-PART exemplar still teaches (CLI: `reposix init
<existing-repo-root>` stderr contains "reposix attach" + a recovery line; helper: a malformed bus URL
`reposix::sim::demo?priority=high` — via the built `git-remote-reposix` binary — emits the canonical
`?mirror=` form on stderr; use TempDir, never the shared repo). Later waves APPEND per-error cases.
This gives W0 a real green baseline, not an empty pass.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && python3 quality/gates/agent-ux/teach_scan.py --self-test && test -x quality/gates/agent-ux/cli-errors-teach-recovery.sh && test -x quality/gates/agent-ux/helper-errors-teach-recovery.sh && bash -n quality/gates/agent-ux/cli-errors-teach-recovery.sh && bash -n quality/gates/agent-ux/helper-errors-teach-recovery.sh && test -f crates/reposix-cli/tests/errors_teach_recovery.rs && test -f crates/reposix-remote/tests/errors_teach_recovery.rs && echo PASS</automated>
  </verify>
  <done>teach_scan.py `--self-test` green (multi-line detection + marker exemption proven); both gates executable + `bash -n` clean; both test files exist with a passing anchor test. COMMIT W0.1+W0.2 as the phase's FIRST commit (catalog + gates + scanner + scaffold) — SC3 ordering gate. No impl yet.</done>
</task>

## Wave 1 — Shared builder + init.rs/attach.rs [reposix-core + reposix-cli]

<task type="auto" tdd="true">
  <name>W1.1: Create the shared teaching-error builder + 3 failure-shape helpers</name>
  <files>crates/reposix-core/src/errmsg.rs, crates/reposix-core/src/lib.rs, crates/reposix-cli/tests/errors_teach_recovery.rs</files>
  <behavior>
    - `teach(hd, fix, alt, &[r1,r2])` renders: headline; "Fix: <fix>"; "Alternative: <alt>"; "Recovery:"; each recovery 2-space-indented.
    - `teach(hd, fix, "", &[r])` OMITS the "Alternative:" line entirely (FLAG 1 — empty alternative suppressible).
    - `teach(hd, fix, alt, &[])` still emits headline + Fix + Alternative but NO dangling "Recovery:" command (empty recovery edge).
    - `Teach::new(hd).fix(f).recovery(&[r])` (no `.alternative`) omits the Alternative line — same as empty alt.
    - `Teach::new(hd).fix(f).code("RPX-0001")` renders identically to no-code TODAY (code slot wired, renders nothing this phase — FLAG 2 forward-compat); a `#[test]` pins "unset code ⇒ byte-identical output".
    - Rendered string contains the greppable anchors "Fix:" and "Recovery:" (and "Alternative:" only when set).
  </behavior>
  <action>
Create `reposix-core::errmsg` with `pub struct Teach<'a>` (builder: fix/alternative/recovery/code) +
`impl Display` + the `pub fn teach(...) -> String` convenience wrapper, per the <interfaces> contract
(pure formatting, no anyhow, pedantic-clean). The `alternative` field is `Option<&str>` set by
`.alternative()` where `""` maps to `None`; `code` is `Option<&str>` rendering nothing while `None`
(the P121 slot). Export via lib.rs. Add `#[cfg(test)] mod tests` covering ALL <behavior> cases —
especially the empty-alternative suppression and the code-slot byte-identity. Unit-test the formatter
shape only here (CLI call sites come in W1.2).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-core 2>&1 | tail -3 && cargo nextest run -p reposix-core errmsg 2>&1 | tail -8</automated>
  </verify>
  <done>errmsg::{Teach, teach} exist, exported, unit-tested for shape + empty-alternative suppression + empty-recovery + code-slot byte-identity; reposix-core builds clean.</done>
</task>

<task type="auto" tdd="true">
  <name>W1.2: Retrofit init.rs + attach.rs + the 3 shared helpers + already-3-part markers</name>
  <files>crates/reposix-cli/src/init.rs, crates/reposix-cli/src/attach.rs, crates/reposix-cli/tests/errors_teach_recovery.rs</files>
  <behavior>
    - `reposix init foo` (no `::`) → stderr teaches `<backend>::<project>` form + lists backends + a runnable example.
    - `reposix init confluence::X` with REPOSIX_CONFLUENCE_TENANT unset → stderr contains `export REPOSIX_CONFLUENCE_TENANT=` + retry + `sim::` alternative.
    - `reposix init sim::demo <path-with-no-cache>` / git-not-on-PATH → 3-part teaching.
    - `reposix attach` on a non-git dir / duplicate-id / multi-SoT conflict → 3-part teaching.
    - init.rs refuse_existing_repo_root + init.rs:365-373 still emit their exact strings AND carry `// teach-exempt: ok` markers (anchor tests pass; teach_scan.py does not RAISE on them).
  </behavior>
  <action>
Introduce `spec_parse_error`, `missing_env_var_error`, `cache_build_error` (leverage #1/#2/#3) — place
them shared (a small `errors` mod in reposix-cli, or inline pub(crate) fns; MUST be shared not
duplicated). Retrofit init.rs sites 48/51/104-106 (→spec_parse), 75-79/92-96 (→missing_env_var),
376-379/505-507/273/514-517/559-562 (→teach) and attach.rs 384-395/120/177/198/202. Route
.context("build cache…") through cache_build_error (surface the source). ADD `// teach-exempt: ok —
bespoke multi-line 3-part` markers to refuse_existing_repo_root + init.rs:365-373. Add per-error cases
to the CLI test file (assert_cmd, exact substrings, TempDir).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-cli 2>&1 | tail -3 && cargo nextest run -p reposix-cli --test errors_teach_recovery 2>&1 | tail -8</automated>
  </verify>
  <done>init.rs + attach.rs enumerated sites route through the shared helpers; already-3-part sites unchanged + marked; per-error tests pass; nextest green. Commit.</done>
</task>

## Wave 2 — list / refresh / spaces / sync [reposix-cli]

<task type="auto" tdd="true">
  <name>W2.1: Retrofit list.rs + refresh.rs + spaces.rs + sync.rs + cache_db.rs markers</name>
  <files>crates/reposix-cli/src/list.rs, crates/reposix-cli/src/refresh.rs, crates/reposix-cli/src/spaces.rs, crates/reposix-cli/src/sync.rs, crates/reposix-cli/src/cache_db.rs, crates/reposix-cli/tests/errors_teach_recovery.rs</files>
  <behavior>
    - `reposix spaces` against a non-confluence backend → ONE deduped error naming which `<backend>` was requested + how to target confluence (was 3 identical bails at spaces.rs 26/29/32).
    - `reposix sync` with no reposix remote → teaches `git remote add` + recovery (upgrade partial at 91-95).
    - list/refresh cache-build failures → cache_build_error (surfaces connector source + teaching layer).
    - `reposix refresh --offline` (unimplemented) → teaches it's not yet supported + the supported alternative.
    - cache_db.rs busy/open/update sites carry `// teach-exempt: ok` markers (refresh-meta store; teach_scan.py does not RAISE).
  </behavior>
  <action>
Consume the W1 shared helpers (import, do not re-create). Dedupe spaces.rs 26/29/32 into one helper.
Route list.rs 77-119, refresh.rs 206-244, sync.rs 99/116/120/130 through cache_build_error. Upgrade
list.rs 271/321, refresh.rs 69-72/308/321/352, sync.rs 91-95 via teach. Keep list.rs:172-180
(sim_unreachable) verbatim + add `// teach-exempt: ok` marker. Add `// teach-exempt: ok — reposix
refresh refresh_meta store; busy path (L101-105) already teaches; L108/L132 internal SQLite` markers to
cache_db.rs:101-105/108/132. Append per-error test cases.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-cli 2>&1 | tail -3 && cargo nextest run -p reposix-cli --test errors_teach_recovery 2>&1 | tail -8</automated>
  </verify>
  <done>list/refresh/spaces/sync enumerated sites teaching-grade; spaces 3-way dupe collapsed; list.rs:172-180 + cache_db.rs sites marked; tests pass; nextest green. Commit.</done>
</task>

## Wave 3 — gc / history / tokens / cost + shared cache-db teaching + cli/main.rs [reposix-cli]

<task type="auto" tdd="true">
  <name>W3.1: Retrofit gc/history/tokens/cost + shared missing_cache_db_error + worktree_helpers + cli/main.rs log</name>
  <files>crates/reposix-cli/src/gc.rs, crates/reposix-cli/src/history.rs, crates/reposix-cli/src/tokens.rs, crates/reposix-cli/src/cost.rs, crates/reposix-cli/src/worktree_helpers.rs, crates/reposix-cli/src/main.rs, crates/reposix-cli/tests/errors_teach_recovery.rs</files>
  <behavior>
    - `reposix tokens` / `reposix cost` with no cache.db → teaches "run `git fetch` / `reposix refresh` first" + recovery (via the SHARED missing_cache_db_error; both subcommands identical).
    - `reposix history`/`tokens`/`cost`/`gc` outside a reposix tree → the SHARED worktree_helpers teaching (route the "no reposix remote" bail through teach(); all four inherit).
    - `reposix log` (no `--time-travel`) → teaches + names `reposix history` alternative + recovery (cli/main.rs:415).
    - `reposix gc --strategy <bad>` (unimplemented) → teaches supported strategies.
    - `reposix cost --since <bad>` → teaches the accepted date format + example (upgrade partial at 67).
  </behavior>
  <action>
Add `missing_cache_db_error(cache_path)` to the shared `errors` mod (leverage #4); consume it at
tokens.rs:64/81 and cost.rs:113/260. RETROFIT worktree_helpers.rs:186 (`no reposix remote`) through
teach() — the shared upstream so history/tokens/cost/gc all inherit uniform teaching. RETROFIT
cli/main.rs:415 (`reposix log` requires `--time-travel`) via teach(). Retrofit gc.rs 74/103/110/287-313,
history.rs 26, cost.rs 67 via teach(). doctor.rs is the documented EXCEPTION — leave it. Append
per-error test cases (assert `reposix tokens` in a fresh tree emits the shared teaching; assert
`reposix log` teaches).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-cli 2>&1 | tail -3 && cargo nextest run -p reposix-cli --test errors_teach_recovery 2>&1 | tail -8</automated>
  </verify>
  <done>gc/history/tokens/cost + worktree_helpers + cli/main.rs log enumerated sites teaching-grade; missing_cache_db_error shared by tokens+cost; doctor.rs untouched-by-design; tests pass; nextest green. Commit.</done>
</task>

## Wave 4 — reposix-remote helper ENTRY/URL/CRED surface [reposix-remote]

<task type="auto" tdd="true">
  <name>W4.1: Retrofit main.rs + bus_url.rs + backend_dispatch.rs (entry/url/cred)</name>
  <files>crates/reposix-remote/src/main.rs, crates/reposix-remote/src/bus_url.rs, crates/reposix-remote/src/backend_dispatch.rs, crates/reposix-remote/tests/errors_teach_recovery.rs</files>
  <behavior>
    - A malformed bus URL — EACH of the 6 arms (`+`-form, base-form reject, empty query, unknown param, missing `mirror=`, empty `mirror=`) → stderr teaches the `reposix::<sot-spec>?mirror=<mirror-url>` form + a corrected example (git invokes the helper, so the message must be legible in git's stderr).
    - `git-remote-reposix` with wrong/missing args (main.rs:115 usage) → teaches correct invocation.
    - A `github::`/`confluence::`/`jira::` push with creds unset (backend_dispatch.rs:362) → teaches `export <VAR>=…` + `sim::` alternative + retry (helper-side leverage #2).
    - backend_dispatch machine-URL parse/constructor/audit sites carry `// teach-exempt: ok` markers (teach_scan.py clean).
  </behavior>
  <action>
Consume `reposix_core::errmsg::teach`/`Teach` (helper depends on core). Add `malformed_bus_url_error`
(leverage #5) and route bus_url.rs 88/98/106/132/141/148 through it. Fix main.rs:129 — drop the
`.context("parse remote url")` so the teach() body surfaces cleanly in git stderr (do NOT double-wrap).
Retrofit main.rs:115 (usage) and main.rs:367 (fail_push import "cannot list issues" DIAG detail →
teach the backend-unreachable recovery: sim not running / creds). RETROFIT backend_dispatch.rs:362
(missing_env_error → add `export`+`sim::`+retry). ADD `// teach-exempt: ok — <reason>` markers to
backend_dispatch.rs 112/114/167/170/176 (machine-generated URL), 298/317/324/332 (internal constructor),
265/273 (OP-3 audit hard error), and main.rs 458 (git's own malformed stream) + 546/571/593 (per-action
REST context; terminal teaching at write_loop reject). Add per-error tests (spawn the built
`git-remote-reposix` with each malformed URL; assert the canonical form + example on stderr; TempDir).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-remote 2>&1 | tail -3 && cargo nextest run -p reposix-remote --test errors_teach_recovery 2>&1 | tail -8</automated>
  </verify>
  <done>main.rs entry sites + all 6 bus_url.rs arms + backend_dispatch missing_env_error teaching-grade; machine-URL/constructor/audit/per-action sites marked; tests pass; `cargo check -p reposix-remote` + nextest green. Commit.</done>
</task>

## Wave 5 — reposix-remote helper TRANSPORT/FAN-OUT surface [reposix-remote]

<task type="auto" tdd="true">
  <name>W5.1: Retrofit stateless_connect.rs + write_loop.rs + bus_handler.rs + precheck.rs markers</name>
  <files>crates/reposix-remote/src/stateless_connect.rs, crates/reposix-remote/src/write_loop.rs, crates/reposix-remote/src/bus_handler.rs, crates/reposix-remote/src/precheck.rs, crates/reposix-remote/tests/errors_teach_recovery.rs</files>
  <behavior>
    - upload-pack subprocess exit (stateless_connect.rs 280/479) → names the likely cause (git version / partial-clone extension) + recovery, ON TOP of the raw git stderr (do not discard the git stderr).
    - unexpected EOF mid-request (stateless_connect.rs:331) → teaches it's a protocol desync + the recovery move, using `Teach::new(...).fix(...).recovery(...)` with NO Alternative line (there is no genuine alternative — the FLAG-1 motivating case).
    - backend-unreachable push reject (write_loop.rs:172) → the accompanying stderr DIAG teaches "sim not running / creds unset" + recovery; the `error refs/heads/main backend-unreachable` PROTOCOL line stays verbatim (git contract).
    - bad-mirror-url `-` prefix (bus_handler.rs:139) → teaches the `?mirror=<url>` form.
    - All already-teaching / protocol-contract / internal sites (BLOB_LIMIT/UNFILTERED consts, write_loop conflict + bulk-delete + invalid-blob + duplicate-id, all EXEMPT bus_handler + precheck sites) carry `// teach-exempt: ok — <reason>` markers; teach_scan.py --scope helper is clean.
  </behavior>
  <action>
Retrofit stateless_connect.rs 280/479 (teach on top of raw stderr) and 331 (`Teach::new`, no
alternative). Retrofit write_loop.rs:172 DIAG via teach() (backend-unreachable recovery). Retrofit
bus_handler.rs:139 via teach(). ADD `// teach-exempt: ok — <reason>` markers to: stateless_connect.rs
54-62 consts; write_loop.rs 207/218/227/236; bus_handler.rs 156-161/173-177/184-206/235-259/273-278/
344-348/380/442/522/529; precheck.rs 113/123/230/251/325/331/348/377 — each reason per
<disposition_table>. For bus_handler.rs:442 (raw mirror_url echo) apply the ownership_charter NOTICING:
eager-fix `strip_url_userinfo` if <1h else file to SURPRISES-INTAKE + note in the report. KEEP the
already-3-part BLOB_LIMIT + conflict `fetch first` behavior verbatim. Add per-error tests (drive a
malformed/EOF path; assert teaching on stderr; TempDir for any init/seed, cd in same invocation).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-remote 2>&1 | tail -3 && cargo nextest run -p reposix-remote --test errors_teach_recovery 2>&1 | tail -8 && python3 quality/gates/agent-ux/teach_scan.py --scope helper 2>&1 | tail -5</automated>
  </verify>
  <done>stateless_connect/write_loop/bus_handler transport sites teaching-grade; every EXEMPT site marked; teach_scan.py --scope helper clean; already-3-part helper sites unchanged; tests pass; nextest green. Commit.</done>
</task>

## Wave 6 — Convention fix-twice + final gate-green + verifier handoff [crates + quality]

<task type="auto">
  <name>W6.1: crates/CLAUDE.md fix-twice + flip catalog rows + run both gates green</name>
  <files>crates/CLAUDE.md, quality/catalogs/agent-ux.json</files>
  <action>
FIX-TWICE (meta-rule): update the crates/CLAUDE.md `## Error-message convention` section — it already
CLAIMS a "three-part bar" but doesn't name the mechanism. Add: (1) the shared builder home
(`reposix_core::errmsg::{Teach, teach}`) + the shared failure-shape helpers as the pattern to REUSE
(not hand-roll), including the empty-alternative suppression + the P121 `.code()` slot; (2) the
`doctor.rs` structured-`DoctorReport` EXCEPTION AND the git-protocol-status-string EXEMPT class
(teaching lives in the accompanying diag, not the contract string); (3) the two agent-ux gate rows +
`teach_scan.py` (multi-line-aware, explicit scope) as the enforcement + the `// teach-exempt: ok —
<reason>` marker convention + its documented residual limitations. Then run BOTH gates green in a
no-cargo-contention window (one at a time — build-memory budget):
`bash quality/gates/agent-ux/cli-errors-teach-recovery.sh` then `.../helper-errors-teach-recovery.sh`.
On green, flip both rows' `status` NOT-VERIFIED→PASS + set `last_verified` (RFC3339). Run
`python3 quality/runners/run.py --cadence pre-push` (structure/banned-words + file-size +
banned-production-tokens) exit 0 before the phase-close push.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && grep -q 'errmsg' crates/CLAUDE.md && grep -q 'doctor' crates/CLAUDE.md && grep -q 'teach-exempt' crates/CLAUDE.md && bash quality/gates/agent-ux/cli-errors-teach-recovery.sh 2>&1 | tail -3 && bash quality/gates/agent-ux/helper-errors-teach-recovery.sh 2>&1 | tail -3</automated>
  </verify>
  <done>crates/CLAUDE.md names the shared builder + doctor exception + protocol-string EXEMPT class + teach_scan enforcement + marker convention; both gates exit 0; both rows PASS with fresh last_verified; pre-push exit 0. Push origin main BEFORE the verifier dispatch (push cadence), confirm post-push code/ci-green-on-main P0, then dispatch the unbiased gsd-verifier.</done>
</task>

</tasks>

<threat_model>
## Trust boundaries
This phase edits error-message strings + a formatter + tests + gates. No NEW remote-byte flow, no new
`Tainted<T>` sink. The live concern: a teaching error must not leak tainted content into an
exfiltration path — these messages are static teaching text + already-surfaced local errors +
user-supplied argv (spec/path/bus-URL), not attacker-controlled record bodies routed outward.

## STRIDE register
| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|------------|
| T-120-01 | Information disclosure | a teaching error interpolating a tainted backend response (issue title) into stderr | mitigate | interpolate only user-supplied argv (spec/path/bus-URL) + the connector's OWN error string; never render a tainted record body into a teaching message |
| T-120-02 | Information disclosure | credential leak — bus_handler.rs:442 echoes raw `mirror_url` (userinfo) to stderr where sibling paths strip it | mitigate | W5 eager-fix `strip_url_userinfo` before echo (or file to SURPRISES-INTAKE if >1h) — carried as an ownership_charter NOTICING |
| T-120-03 | Tampering | regressing an already-3-part site while retrofitting neighbors | mitigate | anchor tests in W0 assert the exemplar strings still emit; those sites LEFT untouched + carry explicit teach-exempt markers (no reliance on incidental single-line-scan skip) |
| T-120-04 | Denial (gate) | teach_scan.py false-positiving on helper-internal/non-user-facing errors and blocking legit code | accept | scan scoped to the enumerated files + `// teach-exempt: ok` escape hatch; on-demand cadence (not pre-push) so a false positive never blocks an unrelated push |
| T-120-05 | Repudiation (catalog-first) | an impl commit landing before the catalog commit, defeating SC3 | mitigate | W0 is the FIRST commit (catalog+gates+scanner+scaffold); phase-close verification greps `git log` to confirm the catalog SHA predates every impl SHA |
| T-120-06 | Tampering (gate rot) | a NEW multi-line teaching-less `bail!` evading a single-line scan | mitigate | teach_scan.py parses balanced-paren macro BLOCKS, single- OR multi-line; residual indirection escapes documented, not overclaimed |
</threat_model>

<verification>
- **SC3 commit-ordering (the gate):** `git log --oneline` for this phase must show the W0 catalog commit
  (touching quality/catalogs/agent-ux.json + the two gate scripts + teach_scan.py) BEFORE any commit
  touching `crates/**/src/*.rs`. Explicit check at phase close: `git log --format='%H %s' <phase-range>`.
- Both nextest suites green: `cargo nextest run -p reposix-cli --test errors_teach_recovery` and
  `-p reposix-remote --test errors_teach_recovery` (continuous regression, normal cadence).
- Both agent-ux gates exit 0 (on-demand, at phase close), each INCLUDING a green `teach_scan.py` run
  over its scope (CLI + helper) — no un-dispositioned block, single- or multi-line.
- `teach_scan.py --self-test` green (the scanner's own multi-line/marker logic is regression-covered).
- `python3 quality/runners/run.py --cadence pre-push` exit 0 before the phase-close push.
- Push landed on main + post-push `code/ci-green-on-main` P0 GREEN before the unbiased verifier dispatch.
- Every executor's report carries a non-empty Noticing section (OD-3 §2).

## SC → wave map
- **SC1 (every CLI subcommand error 3-part):** W1 (init/attach + shared helpers) + W2 (list/refresh/
  spaces/sync) + W3 (gc/history/tokens/cost + worktree_helpers + cli/main.rs log). Verified by row
  `agent-ux/cli-errors-teach-recovery` + the CLI nextest suite + teach_scan.py --scope cli. doctor.rs is
  the documented structured-report exception (W6).
- **SC2 (EVERY helper error 3-part):** W4 (main.rs entry + bus_url 6 arms + backend_dispatch creds) +
  W5 (stateless_connect + write_loop + bus_handler + precheck). Verified by row
  `agent-ux/helper-errors-teach-recovery` + the helper nextest suite + teach_scan.py --scope helper. The
  <disposition_table> is the coverage contract: every helper site is RETROFIT or EXEMPT-marked.
- **SC3 (catalog-first commit ordering):** W0 is the FIRST commit; verified by the git-log ordering check.
</verification>

<success_criteria>
1. Every enumerated `reposix-cli` subcommand error routes through the shared teaching builder/helper (or a
   documented exception) and emits Fix:/Recovery: — proven by the CLI nextest suite + gate + teach_scan.py --scope cli.
2. EVERY enumerated `reposix-remote` helper error is dispositioned (RETROFIT or EXEMPT-marked) per the
   <disposition_table> and meets the bar where retrofit — proven by the helper nextest suite + gate + teach_scan.py --scope helper.
3. The 2 agent-ux rows + 2 gate scripts + teach_scan.py + 2 test scaffolds are the phase's FIRST commit (git-log ordering verified).
4. The already-3-part exemplar sites still emit their teaching strings AND carry explicit teach-exempt markers (anchor tests pass; no incidental-skip reliance) — zero regression.
5. crates/CLAUDE.md's error-message-convention section names the shared builder + doctor.rs exception + git-protocol EXEMPT class + teach_scan enforcement + marker convention (fix-twice).
6. teach_scan.py detects a NEW teaching-less MULTI-LINE bail! (proven by --self-test); residual limitations documented, not overclaimed.
7. Both gates exit 0; both rows PASS with fresh last_verified; pre-push exit 0; main CI green post-push.
</success_criteria>

<output>
After completion, create `.planning/phases/120-cli-helper-error-hardening/120-SUMMARY.md` recording:
per-wave what-landed (which sites RETROFIT, which EXEMPT-marked, which shared helpers created), the
final <disposition_table> reconciled against the shipped code (every helper + shared-CLI site → its
actual disposition), the git-log evidence that the catalog commit preceded all impl commits (SC3),
final gate exit codes + both rows' PASS SHA, the teach_scan.py residual-limitation note, the doctor.rs +
git-protocol-string EXEMPT classes as documented, the bus_handler.rs:442 credential-leak resolution
(eager-fixed or filed), and a RAISE-LIST for anything deferred (e.g. migrating already-3-part sites to
the shared builder for consistency, promoting teach_scan.py to pre-push, P121 RPX-code population of the
wired `.code()` slot).
</output>
