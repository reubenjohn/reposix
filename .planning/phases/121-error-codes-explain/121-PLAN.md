---
phase: 121-error-codes-explain
plan: 121
type: execute
wave: 0                    # Entry wave. SEQUENTIAL 5-wave phase (W0..W4); see <waves>.
depends_on: [120-cli-helper-error-hardening]
requirements: [UX-02]
autonomous: true          # No human checkpoints — every code allocation is BAKED (see <code_allocation>), not gated.
user_setup: []

# Sequential single-tree-writer phase: ONE executor lane per wave, waves run in order,
# never two cargo builds concurrently (crates/CLAUDE.md build-memory budget: one cargo
# invocation machine-wide, FOREGROUND-only, prefer `-p <crate>` jobs=2). Files may recur
# across sequential waves (strict ordering guarantees no concurrent writer); files in the
# SAME wave never overlap.
waves:
  W0: "Catalog-first — mint 1 agent-ux row (rpx-codes-registry) + author the gate script + the registry-integrity checker (rpx_registry_check.py) + scaffold the explain Rust test file. FIRST COMMIT, before any impl."
  W1: "Registry + render — reposix-core::codes (ExplainEntry + REGISTRY + explain(code)) + errmsg.rs Display renders the `[RPX-xxxx]` tag + `Explain:` nudge + REPLACE the code_slot_renders_nothing pinning test with a code_renders_tag_and_nudge test + add teach_coded() convenience fn."
  W2: "Explain subcommand — clap `Explain { code: Option<String> }` variant + dispatch arm + reposix_cli::explain module (run) reading the core registry; no-arg/--list enumerates all codes; unknown-code teaches + lists valid codes; rustc --explain parity test."
  W3: "Mint CLI codes — .code(RPX-xxxx) on the 4 errors.rs shape-helpers + the bespoke CLI user-facing sites (init/attach/list/spaces/gc/cost/log/worktree_helpers) + populate their REGISTRY entries + extend errors_teach_recovery assertions to the new tag."
  W4: "Mint HELPER codes — .code() on the reposix-remote user-facing sites (malformed_bus_url_error, missing_env_error, main.rs usage, stateless_connect upload-pack/EOF, blob-limit consts, write_loop backend-unreachable/conflict DIAG) + REGISTRY entries + rpx-codes gate GREEN + docs surface (cli.md explain section + RPX code index) + crates/CLAUDE.md fix-twice."

files_modified:
  # W0
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/rpx-codes-registry.sh
  - quality/gates/agent-ux/rpx_registry_check.py
  - crates/reposix-cli/tests/explain.rs
  # W1
  - crates/reposix-core/src/codes.rs
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/errmsg.rs
  # W2
  - crates/reposix-cli/src/explain.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/main.rs
  # W3
  - crates/reposix-cli/src/errors.rs
  - crates/reposix-cli/src/init.rs
  - crates/reposix-cli/src/attach.rs
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/spaces.rs
  - crates/reposix-cli/src/gc.rs
  - crates/reposix-cli/src/cost.rs
  - crates/reposix-cli/src/worktree_helpers.rs
  - crates/reposix-cli/tests/errors_teach_recovery.rs
  # W4
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/bus_url.rs
  - crates/reposix-remote/src/backend_dispatch.rs
  - crates/reposix-remote/src/stateless_connect.rs
  - crates/reposix-remote/src/write_loop.rs
  - crates/reposix-remote/tests/errors_teach_recovery.rs
  - docs/reference/cli.md
  - docs/reference/error-codes.md
  - mkdocs.yml
  - crates/CLAUDE.md

must_haves:
  truths:
    - "Running any enumerated CLI error path (init bad-spec / init at existing repo root / confluence-tenant-unset / tokens|cost before any fetch / history in a non-reposix tree / `reposix log` without `--time-travel` / spaces non-confluence / gc bad-strategy / cost bad --since) prints a `[RPX-xxxx]` code tag in its output AND an `Explain: reposix explain RPX-xxxx` nudge line."
    - "Running any enumerated `reposix-remote` helper error path (malformed bus URL, missing real-backend creds, upload-pack subprocess failure, unexpected EOF mid-request, blob-limit exceeded, backend-unreachable push reject) surfaces its `[RPX-xxxx]` code on git stderr (protocol-contract status strings stay verbatim; the code rides the accompanying diag line)."
    - "`reposix explain RPX-xxxx` exists and, for EVERY code in the registry, prints a non-empty title + cause + `Fix:` + copy-paste `Recovery:` (the codified half of the north-star). Two-tier BY DESIGN (rustc-faithful terse-error + verbose-explain split): the code-id and its EXTENDED cause/fix/recovery live ONCE in `reposix-core::codes` (the sole home of the extended explanation); the terse inline `Fix:`/`Alternative:`/`Recovery:` limbs come from the P120 `teach()` call-site args — a deliberately shorter surface, NOT a copy of the registry prose. Only the CODE-ID is single-sourced across the inline render and explain; no explanation prose is duplicated (the extended text lives once in the registry, the terse text once at the call-site) — so there is NO single-render coherence gate, the two tiers are supposed to differ."
    - "`reposix explain` (no arg) and `reposix explain --list` enumerate every registered code + title, one per line."
    - "`reposix explain RPX-9999` (unknown code) fails with a teaching message that names how to list valid codes — dogfooding the 3-part bar (it does NOT panic or print a bare 'not found')."
    - "A committed gate (`agent-ux/rpx-codes-registry`) asserts: (a) every `.code(\"RPX-xxxx\")` literal referenced anywhere in `crates/**` has a REGISTRY entry; (b) every REGISTRY entry has a non-empty cause AND fix AND recovery; (c) codes are unique + 4-digit zero-padded `RPX-\\d{4}`; (d) `reposix explain <unknown>` exits non-zero with a teaching message."
    - "`reposix explain` output SHAPE matches `rustc --explain E0308` (a code-header line + an extended prose body + a fix section), verified side-by-side by a committed test (the rustc leg is best-effort — skipped when `rustc --explain` is unavailable — but the shape invariants on reposix's OWN output are the hard gate)."
    - "The registry + every explain/code-slot render is entirely `&'static str`; NO tainted remote byte is interpolated into the code slot or the explain output (OP-2 — confirmed by construction: `.code()` takes a static code id, `explain` reads only the static REGISTRY)."
    - "The agent-ux contract row + gate script + registry-integrity checker are committed in the phase's FIRST commit, before any implementation commit (git log shows the catalog SHA predates every impl SHA)."
  artifacts:
    - path: "crates/reposix-core/src/codes.rs"
      provides: "Single-source RPX registry: pub struct ExplainEntry (code/title/cause/fix/alternative/recovery, all &'static str) + pub const REGISTRY: &[ExplainEntry] + pub fn explain(code: &str) -> Option<&'static ExplainEntry>"
      contains: "pub fn explain"
      min_lines: 40
    - path: "crates/reposix-cli/src/explain.rs"
      provides: "reposix explain <code> subcommand: run(code: Option<String>) -> Result<()>; --list/no-arg enumerates; unknown-code teaches via errmsg + lists"
      contains: "reposix_core::codes"
    - path: "crates/reposix-cli/tests/explain.rs"
      provides: "Per-code explain assertions (every REGISTRY code prints non-empty cause+fix+recovery) + list + unknown-code-teaches + rustc-parity-of-shape"
      contains: "assert_cmd"
    - path: "quality/catalogs/agent-ux.json"
      provides: "Contract row agent-ux/rpx-codes-registry (kind mechanical, coverage_kind mechanical, cadences on-demand, status NOT-VERIFIED until W4 green + verifier)"
      contains: "rpx-codes-registry"
    - path: "quality/gates/agent-ux/rpx-codes-registry.sh"
      provides: "Reality-exercising gate: runs the explain nextest suite + drives a real `reposix explain <code>` + `reposix explain RPX-9999` + invokes rpx_registry_check.py"
    - path: "quality/gates/agent-ux/rpx_registry_check.py"
      provides: "Registry-integrity checker: cross-checks every `.code(\"RPX-xxxx\")` literal in crates/** against REGISTRY; asserts entries non-empty + codes unique + format RPX-\\d{4}"
      contains: "RPX"
    - path: "docs/reference/error-codes.md"
      provides: "User-facing RPX code index + `reposix explain` usage; registered in mkdocs nav"
      contains: "reposix explain"
    - path: "crates/CLAUDE.md"
      provides: "Error-message-convention section updated: RPX registry location + .code() usage + explain subcommand + the rpx-codes gate"
      contains: "RPX"
  key_links:
    - from: "crates/reposix-core/src/errmsg.rs Display impl"
      to: "self.code (the P121 slot)"
      via: "renders `<headline> [RPX-xxxx]` + a trailing `Explain: reposix explain RPX-xxxx` limb when code is set"
      pattern: "self\\.code|Explain:"
    - from: "crates/reposix-cli/src/explain.rs"
      to: "reposix_core::codes::explain"
      via: "explain(code) registry lookup -> print title/cause/Fix/Recovery"
      pattern: "codes::explain|reposix_core::codes"
    - from: "quality/gates/agent-ux/rpx-codes-registry.sh"
      to: "rpx_registry_check.py + crates/reposix-cli/tests/explain.rs + the real reposix binary"
      via: "python3 rpx_registry_check.py + cargo test -p reposix-cli --test explain + reposix explain <code>|<unknown>"
      pattern: "rpx_registry_check|--test explain"
---

<objective>
Give every user-facing `reposix` error a **stable, structured `RPX-xxxx` code** and ship
**`reposix explain <code>`** — a subcommand mirroring `rustc --explain E0308` that prints the
extended cause + fix + alternative + copy-paste recovery for that code. This is the codified,
queryable half of the Rust-compiler-grade UX north star (P120 shipped the prose bar; P121 adds the
stable identifier + lookup).

P120 already routed ~40 CLI + helper error sites through ~5 shared teaching helpers (`spec_parse_error`,
`missing_env_var_error`, `cache_build_error`, `missing_cache_db_error`, `malformed_bus_url_error`) plus
~6 bespoke already-3-part sites, and wired a forward-compat `Teach::code()` slot that renders NOTHING
today (byte-pinned by `errmsg.rs::tests::code_slot_renders_nothing`). **This is exactly why P121 does
NOT balloon:** minting a code is `.code(RPX-xxxx)` on each SHARED helper + each bespoke site — ~22
codes, not 40 edits — plus one registry + one subcommand.

The registry (`reposix-core::codes`) is the SINGLE SOURCE OF TRUTH for the CODE-ID and its EXTENDED
explanation. Two-tier BY DESIGN (rustc-faithful: a terse inline error + a verbose `explain`, exactly
like rustc's compact `E0308` message vs its longer `--explain E0308`). The inline `.code()` render
prints ONLY a short `[RPX-xxxx]` tag + an `Explain: reposix explain RPX-xxxx` nudge (north-star touch)
— its terse `Fix:`/`Alternative:`/`Recovery:` limbs come from the P120 `teach()` call-site args, NOT
from the registry — while `reposix explain` prints the registry entry's full extended cause/fix/
recovery. The extended explanation lives ONCE (in the registry); the terse inline prose lives ONCE (at
the call-site); only the CODE-ID is shared across the two surfaces — no explanation prose is
duplicated. (This is a deliberate two-tier split, not a coherence gap; there is no single-render
coherence gate because the two tiers are SUPPOSED to differ.)

Purpose: the phase's literal goal IS the project north star (OD-3 §5 / Rust-compiler-grade UX) — a
skeptical dev who hits `RPX-0201` on stderr can run `reposix explain RPX-0201` and get a
compiler-grade extended explanation, exactly like `rustc --explain`.
Output: a static RPX registry, a `reposix explain` subcommand (+ `--list`), ~22 coded error sites
across the CLI + helper, 1 reality-exercising agent-ux gate + a registry-integrity checker, a Rust
integration suite, a user-facing error-code doc, and a fix-twice `crates/CLAUDE.md` update.
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
@.planning/phases/120-cli-helper-error-hardening/120-PLAN.md

<interfaces>
<!-- The P120 seam this phase builds on. DO NOT re-litigate; verify against source then extend. -->

reposix-core::errmsg::Teach (crates/reposix-core/src/errmsg.rs) — the shared 3-part teaching builder.
The `.code(&'a str)` slot (L118-125) is WIRED and renders NOTHING today; the Display impl (L128-151)
has the exact insertion point marked at L143-149. `tests::code_slot_renders_nothing` (L246) byte-pins
the inert state — P121 REPLACES that test with one asserting the NEW rendered format.

```rust
// EXISTING (P120): renders headline + Fix: + Alternative: (omitted if empty) + Recovery: + 2-space cmds.
// P121 ADDS (in Display, at the marked L143-149 slot):
impl std::fmt::Display for Teach<'_> {
    // ... existing headline/fix/alt/recovery limbs UNCHANGED ...
    // NEW: when code is Some, append a code tag to the FIRST line and an Explain: limb at the end.
    // The tag goes on the headline line: `<headline> [RPX-0001]`; the nudge is a trailing limb
    // mirroring Fix:/Alternative:/Recovery: — `Explain: reposix explain RPX-0001`.
}
```

reposix-cli::errors (crates/reposix-cli/src/errors.rs) — the 4 shape-helpers currently call the
free `teach(hd, fix, alt, &[rec])` fn (no code). W3 switches each to attach a code. Either build via
`Teach::new(hd).fix(f).alternative(a).recovery(r).code(RPX_x)` OR (recommended, minimal churn) add a
`teach_coded(code, hd, fix, alt, rec)` free-fn in W1 mirroring `teach`'s signature with a leading code.

reposix-cli/src/main.rs — clap `#[derive(Subcommand)] enum Cmd` (L39-348) + match dispatcher in
`async fn main()` (L362-477). Study the `Sim` variant (L43-62) + its `Cmd::Sim {..} => run_sim(..)`
arm (L367-374) as the template. Add an `Explain { code, list }` variant + arm + `use reposix_cli::explain`.

reposix-remote helper (crates/reposix-remote/src/) — depends on reposix-core, so it shares the SAME
`errmsg::Teach` + the SAME `codes` registry. The helper does NOT get its own `explain` subcommand;
its error strings CARRY codes, and the user resolves them via `reposix explain` (a CLI command). The
registry living in reposix-core is what makes one `explain` resolve BOTH binaries' codes. Helper
user-facing sites already teaching (from the P120 <disposition_table>): main.rs:115 usage,
bus_url.rs 6 arms (via `malformed_bus_url_error`), backend_dispatch.rs:362 `missing_env_error`,
stateless_connect.rs upload-pack (L84) + EOF (L331) + BLOB_LIMIT/UNFILTERED consts (L54-62),
write_loop.rs backend-unreachable diag (L169-173) + conflict diag (L183-207).
</interfaces>
</context>

<ownership_charter>
Embedded verbatim per OD-3 — every executor dispatched against a wave of this plan OWNS what it touches:
1. You own what you touch. Acceptance criteria are the FLOOR, not the ceiling — done means "I'd defend
   this in review as excellent," not "plan executed."
2. Noticing is a deliverable — report lying doc claims, tests that don't assert what their names promise,
   teaching-free errors, dead code, stale comments, missing edge cases. An empty noticing section from
   code-touching work is a red flag. (Standing candidate: does adding a code to a shape-helper break any
   existing `errors_teach_recovery` exact-substring assertion? EXTEND those assertions, don't just avoid
   breaking them.)
3. Eager-fix (<1h, no new dependency) or FILE — never silently skip. File to
   `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with severity + sketch.
4. Verify against reality — build it, RUN `reposix explain RPX-xxxx`, hit the real error path, grep the
   real `[RPX-xxxx]` tag on stderr. A claim without an artifact is not done.
5. North star — Rust-compiler-grade UX: every explain entry teaches the fix, suggests the alternative,
   gives a copy-paste recovery command. Exemplar: `crates/reposix-cli/src/init.rs::refuse_existing_repo_root`.
   Would a skeptical first-time dev running `reposix explain RPX-0201` come away impressed?
</ownership_charter>

<shared_design>
## BAKED decisions — no per-wave re-litigation

**Single source of truth for the CODE-ID + its EXTENDED explanation = `reposix-core::codes`.** A `pub
struct ExplainEntry { code, title, cause, fix, alternative, recovery }` — EVERY field `&'static str`
(recovery `&'static [&'static str]`). A `pub const REGISTRY: &[ExplainEntry]` array + `pub fn
explain(code: &str) -> Option<&'static ExplainEntry>` (linear scan; ~22 entries, O(n) is fine).
TWO-TIER, rustc-faithful (a terse inline error + a verbose `explain`): `reposix explain` consumes the
registry entry's EXTENDED cause/fix/recovery — the sole home of that extended prose; the inline
`.code()` render consumes ONLY the code-id (it prints the `[RPX-xxxx]` tag + the `Explain: reposix
explain RPX-xxxx` nudge, NOT the registry prose), and its terse `Fix:`/`Alternative:`/`Recovery:` limbs
are the P120 `teach()` call-site args. So no explanation prose is duplicated: the extended text lives
once in the registry, the terse text once at the call-site, and only the CODE-ID is shared across the
two surfaces — which is why there is NO single-render coherence gate (the two tiers are supposed to
differ, exactly like rustc's compact error vs `--explain`). Prefer named consts for the code
IDs (`pub mod ids { pub const SPEC_PARSE: &str = "RPX-0001"; ... }`) so call sites reference by name and
typos are impossible; the gate is the backstop for raw literals.

**Inline render format (errmsg.rs Display, FLAG-2 slot).** When `code` is set, append ` [RPX-xxxx]` to
the headline line, and append a trailing limb `Explain: reposix explain RPX-xxxx` AFTER the Recovery
block. The `Fix:`/`Alternative:`/`Recovery:` limbs are UNCHANGED (so P120's `teach_scan.py` anchors +
the `errors_teach_recovery` substring assertions still hold). When `code` is unset the output is
byte-identical to today (backward-compat for the ~40 sites that never get a code — though SC1 aims to
code every USER-FACING site). REPLACE `tests::code_slot_renders_nothing` with `code_renders_tag_and_nudge`
asserting the new shape + that an unset code still renders nothing.

**`reposix explain` output shape (SC3 — rustc parity).** Print:
```
RPX-0201: <title>

<cause — one or more prose lines>

Fix: <fix>
Alternative: <alternative>        # omitted when empty
Recovery:
  <cmd1>
  <cmd2>
```
This mirrors `rustc --explain E0308`'s "code header + extended body + how-to-fix" structure. SC3's
side-by-side parity is a committed test that captures both `rustc --explain E0308` and `reposix explain
RPX-0201` and asserts the SHARED shape invariants (a header line containing the code; a non-empty
multi-line body; a fix section). The rustc leg is BEST-EFFORT (skip if `rustc --explain` errors /
unavailable on the runner); the shape assertions on reposix's OWN output are the hard gate. Do NOT
byte-diff against rustc (brittle).

**`reposix explain` with no arg / `--list`.** Enumerate every REGISTRY code + title, one per line
(sorted by code). `reposix explain RPX-9999` (unknown) → a teach() 3-part error (its OWN code RPX-0900,
the explain-meta code) that names `reposix explain --list` as the recovery. Dogfood the bar — no bare
"not found", no panic.

**Helper codes, CLI explain.** The helper carries codes but has no `explain` verb; `reposix explain`
(CLI) resolves ALL codes because the registry is in reposix-core (shared). Git-protocol contract
strings (`error refs/heads/main fetch first`, `... backend-unreachable`) stay VERBATIM — git parses
them — so the code rides the ACCOMPANYING `diag()` stderr line (same ruling P120 used for teaching:
"teaching belongs in the diag"). W4 adds the code to the diag, never the protocol string.

**Coverage floor (HONEST).** SC1's "every user-facing error" = the complete set of P120-dispositioned
RETROFIT + bespoke-user-facing sites (enumerated in <code_allocation>). The P120 EXEMPT sites
(machine-generated-URL wraps, internal SQLite/subprocess wraps, git-protocol-contract strings whose
teaching already rides a coded diag) are NOT user-facing teaching errors and are OUT of scope BY THE
SAME reasoning P120 used to EXEMPT them — this is not a scope reduction, it is the same surface P120
defined. If an executor finds a genuinely user-facing site P120 mis-dispositioned as EXEMPT, eager-fix
(add a code) or file — do not silently skip.

**Threat (OP-2 — tainted-byte-into-code-slot).** The registry is 100% `&'static str`; `.code()` takes
a static code id; `explain` reads only the static REGISTRY. NO remote byte can reach the code slot or
explain output. The headline MAY carry a (P120-redacted) tainted spec/URL — that is pre-existing and
unchanged; the CODE + EXPLAIN paths are static-only. W1 executor: confirm by construction that the
Display code-limb interpolates ONLY `self.code` (a static id), never a headline-derived or remote byte.

## Leaf isolation (HARD — any test/gate that runs `reposix init`/sim-seed)
Rust tests use `tempfile::TempDir` (existing `tests/*.rs` precedent) — never the shared repo. `explain`
paths are HERMETIC (no tree, no network, no seed). Any error-path exercise that runs `reposix init`
MUST `cd` into a `mktemp -d`-under-`/tmp` dir in the SAME shell invocation (dark-factory.sh precedent;
`.claude/hooks/leaf-isolation-guard.sh` fails closed otherwise).
</shared_design>

<code_allocation>
## RPX-xxxx starter allocation (categorized — executors finalize exact numbers, fill the prose)

Every code below maps to a P120 RETROFIT helper OR a bespoke user-facing site. Executors VERIFY each
against source, allocate the code, and write the REGISTRY entry (title/cause/fix/alternative/recovery).

### RPX-00xx — spec / parse  [W3]
| Code | Site (P120 helper/site) | Covers |
|------|------------------------|--------|
| RPX-0001 | `spec_parse_error` (errors.rs) | init / attach / sync / refresh spec parse |

### RPX-01xx — env / credentials  [W3 CLI, W4 helper]
| RPX-0101 | `missing_env_var_error` (errors.rs) | CLI confluence/jira tenant/instance unset |
| RPX-0102 | `backend_dispatch::missing_env_error` (helper) | helper real-backend creds unset |

### RPX-02xx — cache / tree state  [W3]
| RPX-0201 | `cache_build_error` (errors.rs) | attach / list / refresh / sync cache build |
| RPX-0202 | `missing_cache_db_error` (errors.rs) | tokens / cost no cache.db |
| RPX-0203 | `worktree_helpers::cache_path_from_worktree` no-reposix-tree | history / tokens / cost / gc |

### RPX-03xx — CLI subcommand-specific  [W3]
| RPX-0301 | cli/main.rs:415 `reposix log` requires `--time-travel` | log |
| RPX-0302 | spaces.rs non-confluence backend | spaces |
| RPX-0303 | refresh.rs `--offline` unimplemented | refresh |
| RPX-0304 | gc.rs `--strategy` unimplemented | gc |
| RPX-0305 | cost.rs `--since` parse | cost |
| RPX-0306 | init.rs git-not-on-PATH | init |

### RPX-04xx — init/attach lifecycle (bespoke already-3-part)  [W3]
| RPX-0401 | init.rs `refuse_existing_repo_root` | init |
| RPX-0402 | init.rs fetch-sync-failed | init |
| RPX-0403 | attach.rs not-a-git-tree | attach |
| RPX-0404 | attach.rs duplicate-ids | attach |
| RPX-0405 | attach.rs multi-SoT conflict | attach |

### RPX-05xx — helper transport  [W4]
| RPX-0501 | stateless_connect.rs upload-pack subprocess (L84) | helper fetch |
| RPX-0502 | stateless_connect.rs unexpected EOF (L331) | helper protocol |
| RPX-0503 | stateless_connect.rs BLOB_LIMIT_EXCEEDED (L54) | helper fetch |
| RPX-0504 | write_loop.rs backend-unreachable DIAG (L169-173) | helper push |
| RPX-0505 | write_loop.rs push-conflict/fetch-first DIAG (L183-207) | helper push |

### RPX-06xx — helper URL / entry  [W4]
| RPX-0601 | `malformed_bus_url_error` (bus_url.rs 6 arms + bus_handler bad-mirror) | helper URL |
| RPX-0602 | main.rs:115 too-few-args usage | helper entry |

### RPX-09xx — explain-meta  [W2]
| RPX-0900 | explain.rs unknown-code | reposix explain |

~22 codes. Shared helpers collapse ~40 P120 sites onto ~10 codes; the rest are bespoke 1:1. Executors
MAY split/merge within a category (mechanical — your call) as long as the gate stays green.
</code_allocation>

<tasks>

## Wave 0 — Catalog-first (FIRST COMMIT — before ANY implementation commit) [quality + reposix-cli]

<task type="auto">
  <name>W0.1: Mint the rpx-codes-registry contract row + author the gate + registry checker + scaffold the explain test</name>
  <files>quality/catalogs/agent-ux.json, quality/gates/agent-ux/rpx-codes-registry.sh, quality/gates/agent-ux/rpx_registry_check.py, crates/reposix-cli/tests/explain.rs</files>
  <action>
**Catalog row** (agent-ux rows are hand-edited per the catalog's `_provenance_note`; mint the full OP-3
shape or `_audit_field.py::validate_row` rejects at load — model on the existing
`agent-ux/cli-errors-teach-recovery` row at agent-ux.json:2701). Row `agent-ux/rpx-codes-registry`:
`dimension:"agent-ux"`, `kind:"mechanical"`, `transport_claim:false`, `coverage_kind:"mechanical"`
(local binary/registry checks, NOT a real external backend — same honest rationale as the P120 rows),
`cadences:["on-demand"]`, `blast_radius:"P1"`, `waiver:null`, `status:"NOT-VERIFIED"` (honest — the gate
has not run; W4 flips to PASS after a green run + the unbiased verifier). `minted_at` RFC3339 write-once.
`claim_vs_assertion_audit` (≥50 chars) stating these legs exercise the LOCAL `reposix` binary + the
static registry, not a real backend. command `bash quality/gates/agent-ux/rpx-codes-registry.sh`;
verifier {script: same, args:[], timeout_s:180, container:null}; artifact
`quality/reports/verifications/agent-ux/rpx-codes-registry.json`; sources = the gate + the checker +
`crates/reposix-cli/tests/explain.rs` + `crates/reposix-core/src/codes.rs`. expected.asserts:
  - "cargo test -p reposix-cli --test explain passes (every REGISTRY code prints non-empty cause+fix+recovery; list; unknown-code teaches; rustc-parity-of-shape)"
  - "rpx_registry_check.py finds every `.code(\"RPX-xxxx\")` literal in crates/** has a REGISTRY entry, entries non-empty, codes unique + format RPX-\\d{4}"
  - "a real `reposix explain RPX-0201` prints a code header + Fix: + Recovery: on stdout"
  - "a real `reposix explain RPX-9999` (unknown) exits non-zero with a teaching message naming `reposix explain --list`"

**rpx_registry_check.py** (python3, stdlib only). (a) grep every `.code("RPX-\d{4}")` (and any
`ids::` const if used) literal across `crates/**/*.rs`, collect the code set. (b) Extract the REGISTRY
code set from `crates/reposix-core/src/codes.rs` (parse `code:` / const fields — a simple regex over
`"RPX-\d{4}"` string literals inside codes.rs is acceptable AND documented as the heuristic). (c) RAISE
if any call-site code is absent from REGISTRY, if any REGISTRY code is malformed (not `RPX-\d{4}`), or if
any code is duplicated. Print `file:line: RPX-xxxx referenced but not in REGISTRY — add an entry to
crates/reposix-core/src/codes.rs` on failure (dogfood the bar). Add a `--self-test` mode with inline
fixtures (a referenced-and-registered code PASSES; a referenced-but-unregistered code RAISEs; a dup
RAISEs) so the checker's own logic is regression-covered. Header comment documents the regex heuristic +
its residual limit (indirection escapes) HONESTLY — do not overclaim.

**Gate script** rpx-codes-registry.sh (`set -euo pipefail`, resolve REPO_ROOT, cd there; model on
`quality/gates/agent-ux/cli-errors-teach-recovery.sh`): (a) run `cargo nextest run -p reposix-cli
--test explain` (single FOREGROUND cargo invocation — never background, never `--workspace`); (b)
exercise a real `reposix explain <code>` + `reposix explain RPX-9999` from a `/tmp` `mktemp -d` (cd in
SAME invocation), grep stdout/stderr for the code header + `Fix:`/`Recovery:` and the unknown-code
teaching; (c) run `python3 "$REPO_ROOT/quality/gates/agent-ux/rpx_registry_check.py"`. Exit 0 = PASS,
exit 1 with a teaching stderr on any failure. NOTE: the explain binary + nextest need a built
`reposix` — the gate builds it once, foreground.

**Rust test scaffold** crates/reposix-cli/tests/explain.rs — the `workspace_root()`/assert_cmd helper
(copy from cli.rs or errors_teach_recovery.rs) + ONE anchor test: `reposix explain --list` exits 0 and
prints at least one `RPX-` line. Later waves APPEND per-code cases. Gives W0 a real green baseline.

COMMIT W0.1 as the phase's FIRST commit (catalog + gate + checker + scaffold). SC "catalog-first
ordering" gate. No impl yet.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && python3 quality/gates/agent-ux/rpx_registry_check.py --self-test && test -x quality/gates/agent-ux/rpx-codes-registry.sh && bash -n quality/gates/agent-ux/rpx-codes-registry.sh && test -f crates/reposix-cli/tests/explain.rs && jq -e '[.rows[]|select(.id=="agent-ux/rpx-codes-registry")] | (length==1) and (.[0].minted_at!=null) and (.[0].coverage_kind=="mechanical") and (.[0].status=="NOT-VERIFIED") and (.[0].claim_vs_assertion_audit|length>=50)' quality/catalogs/agent-ux.json && echo PASS</automated>
  </verify>
  <done>Row loads valid (minted_at RFC3339, coverage_kind mechanical, status NOT-VERIFIED, audit ≥50 chars); rpx_registry_check.py `--self-test` green; gate executable + `bash -n` clean; explain.rs scaffold exists. Catalog+gate+checker committed as the FIRST commit, before any impl SHA.</done>
</task>

## Wave 1 — Registry + render [reposix-core]

<task type="auto" tdd="true">
  <name>W1.1: Create reposix-core::codes registry + explain() + render the code tag/nudge + replace the pinning test</name>
  <files>crates/reposix-core/src/codes.rs, crates/reposix-core/src/lib.rs, crates/reposix-core/src/errmsg.rs</files>
  <behavior>
    - `codes::explain("RPX-0900")` returns Some(entry) with non-empty title+cause+fix+recovery; `codes::explain("RPX-9999")` returns None.
    - `codes::REGISTRY` codes are all unique and match `RPX-\d{4}`; a `#[test]` asserts uniqueness + format over the whole array.
    - `Teach::new(hd).fix(f).recovery(&[r]).code("RPX-0001")` renders `hd [RPX-0001]\nFix: f\nRecovery:\n  r\nExplain: reposix explain RPX-0001`.
    - `Teach::new(hd).fix(f).recovery(&[r])` (NO `.code`) renders byte-identical to today — no tag, no Explain limb (backward-compat).
    - The `Fix:`/`Alternative:`/`Recovery:` limbs are UNCHANGED in position + text (P120 teach_scan.py anchors + errors_teach_recovery substrings still hold).
    - `teach_coded(code, hd, fix, alt, rec)` free-fn renders identically to the builder with `.code(code)` (convenience for W3/W4 minimal churn).
  </behavior>
  <action>
Create `reposix-core::codes` with `pub struct ExplainEntry` (all `&'static str` fields; recovery
`&'static [&'static str]`) + `pub const REGISTRY: &[ExplainEntry]` (seed with RPX-0900 explain-meta this
wave; W3/W4 append the rest) + `pub fn explain(code: &str) -> Option<&'static ExplainEntry>` +
(recommended) `pub mod ids` named consts. Export via lib.rs. Add `#[test]`s: uniqueness + `RPX-\d{4}`
format over REGISTRY; `explain` hit/miss.

In `errmsg.rs` Display (the marked L143-149 slot): when `self.code` is `Some`, append ` [{code}]` to the
headline line AND append a trailing `\nExplain: reposix explain {code}` limb after the recovery block.
Interpolate ONLY `self.code` (static id) — confirm no headline/remote byte reaches this limb (OP-2). Add
`teach_coded(code, hd, fix, alt, rec) -> String` free-fn next to `teach`. REPLACE
`tests::code_slot_renders_nothing` (L246) with `code_renders_tag_and_nudge` asserting the new shape AND
that an UNSET code still renders byte-identical to no-code (keep that half of the invariant). Keep every
other errmsg test green.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-core 2>&1 | tail -3 && cargo nextest run -p reposix-core errmsg codes 2>&1 | tail -12</automated>
  </verify>
  <done>codes::{ExplainEntry, REGISTRY, explain, ids} exist + exported; errmsg Display renders the code tag + Explain nudge when set, byte-identical when unset; old pinning test replaced with a passing new-shape test; reposix-core builds + tests green. Commit.</done>
</task>

## Wave 2 — Explain subcommand [reposix-cli]

<task type="auto" tdd="true">
  <name>W2.1: Add the `reposix explain` subcommand + dispatch + --list + unknown-code teaching + rustc-parity test</name>
  <files>crates/reposix-cli/src/explain.rs, crates/reposix-cli/src/lib.rs, crates/reposix-cli/src/main.rs, crates/reposix-cli/tests/explain.rs</files>
  <behavior>
    - `reposix explain RPX-0900` prints `RPX-0900: <title>`, a blank line, the cause prose, then `Fix:` and `Recovery:` (Alternative: only when set); stdout, exit 0.
    - `reposix explain` (no code) and `reposix explain --list` print every REGISTRY code + title one per line (sorted); exit 0.
    - `reposix explain RPX-9999` (unknown) prints a teach() 3-part message (code RPX-0900) naming `reposix explain --list`; exits non-zero (anyhow bail → exit 1).
    - Output shape matches `rustc --explain E0308`: a code-header line + a non-empty multi-line body + a fix section (rustc leg best-effort/skippable).
  </behavior>
  <action>
Create `reposix_cli::explain` with `pub fn run(code: Option<String>, list: bool) -> Result<()>` reading
`reposix_core::codes`. No code / `--list` → enumerate. `Some(code)` → `codes::explain(&code)` → print the
entry in the SC3 shape; `None`-from-lookup → `anyhow::bail!("{}", teach_coded("RPX-0900", "no such error
code `{code}`.", "run `reposix explain --list` to see all defined RPX codes.", "", &["reposix explain
--list"]))`. Export in lib.rs. In main.rs add the clap variant `Explain { code: Option<String>, #[arg(long)]
list: bool }` (doc-comment it like the `Sim` variant) + the match arm `Cmd::Explain { code, list } =>
explain::run(code, list)`. Append test cases to tests/explain.rs: per-code non-empty-cause+fix+recovery
(iterate assertions over the seeded codes), list, unknown-code-teaches, and a `rustc_parity_of_shape` test
that captures `rustc --explain E0308` (skip via early-return if it errors) + `reposix explain RPX-0900`
and asserts the shared shape invariants on reposix's output. Use assert_cmd; hermetic (no tree/network).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-cli 2>&1 | tail -3 && cargo nextest run -p reposix-cli --test explain 2>&1 | tail -12</automated>
  </verify>
  <done>`reposix explain <code>` / `--list` / unknown-code all work against the real binary; rustc-parity-of-shape test green (or honestly skipped); nextest green. Commit.</done>
</task>

## Wave 3 — Mint CLI codes [reposix-cli]

<task type="auto" tdd="true">
  <name>W3.1: Attach RPX codes to the 4 shape-helpers + bespoke CLI sites + populate REGISTRY entries</name>
  <files>crates/reposix-cli/src/errors.rs, crates/reposix-cli/src/init.rs, crates/reposix-cli/src/attach.rs, crates/reposix-cli/src/list.rs, crates/reposix-cli/src/spaces.rs, crates/reposix-cli/src/gc.rs, crates/reposix-cli/src/cost.rs, crates/reposix-cli/src/worktree_helpers.rs, crates/reposix-cli/tests/explain.rs, crates/reposix-cli/tests/errors_teach_recovery.rs, crates/reposix-core/src/codes.rs</files>
  <behavior>
    - Every RPX-00xx..04xx code in <code_allocation> has a REGISTRY entry with non-empty title+cause+fix+recovery.
    - `reposix init foo <tmp>` stderr contains `[RPX-0001]` + `Explain: reposix explain RPX-0001`; `reposix init` at an existing repo root contains `[RPX-0401]`; `reposix log` (no --time-travel) contains `[RPX-0301]`; `reposix tokens` in a fresh tree contains `[RPX-0203]`.
    - `reposix explain RPX-0001` (and every other CLI code) prints its registry entry.
    - Existing `errors_teach_recovery` substring assertions still pass, AND each coded site's assertion is EXTENDED to also assert its `[RPX-xxxx]` tag.
    - rpx_registry_check.py passes over the CLI scope (every `.code` literal registered).
  </behavior>
  <action>
Switch the 4 errors.rs shape-helpers (`spec_parse_error`→RPX-0001, `missing_env_var_error`→RPX-0101,
`cache_build_error`→RPX-0201, `missing_cache_db_error`→RPX-0202) from the free `teach()` to
`teach_coded(RPX_x, ...)` (or the builder `.code()`). RETROFIT `worktree_helpers.rs`
no-reposix-tree→RPX-0203, `main.rs`-adjacent CLI sites already routed in P120 W2/W3 — add `.code()` to:
cli/main.rs log (already RPX-0301 target — main.rs is W2's file; do the `.code()` there in THIS wave via
the errors path or a coded teach), spaces→RPX-0302, refresh --offline→RPX-0303, gc --strategy→RPX-0304,
cost --since→RPX-0305, init git-not-on-PATH→RPX-0306, and the bespoke RPX-04xx (refuse_existing_repo_root,
fetch-sync-failed, attach not-a-git-tree/duplicate-ids/multi-sot). NOTE: `main.rs` is listed in W2's
files_modified; the `Cmd::Log` `.code()` addition rides W2's main.rs edit OR is done here — coordinator
picks one wave to own main.rs to avoid a same-wave writer conflict (sequential waves make either safe).
Populate ALL these REGISTRY entries in codes.rs (append to the W1 seed). Append per-code cases to
tests/explain.rs and EXTEND errors_teach_recovery.rs assertions to the tag. Run rpx_registry_check.py.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-cli 2>&1 | tail -3 && cargo nextest run -p reposix-cli --test explain --test errors_teach_recovery 2>&1 | tail -12 && python3 quality/gates/agent-ux/rpx_registry_check.py 2>&1 | tail -5</automated>
  </verify>
  <done>All CLI RPX-00xx..04xx codes minted + registered + rendered on real error paths + explain-able; errors_teach_recovery extended + green; rpx_registry_check clean for CLI scope. Commit.</done>
</task>

## Wave 4 — Mint helper codes + gate GREEN + docs [reposix-remote + docs + quality]

<task type="auto" tdd="true">
  <name>W4.1: Attach RPX codes to helper user-facing sites + REGISTRY entries + gate GREEN + docs + CLAUDE.md fix-twice</name>
  <files>crates/reposix-remote/src/main.rs, crates/reposix-remote/src/bus_url.rs, crates/reposix-remote/src/backend_dispatch.rs, crates/reposix-remote/src/stateless_connect.rs, crates/reposix-remote/src/write_loop.rs, crates/reposix-remote/tests/errors_teach_recovery.rs, crates/reposix-core/src/codes.rs, docs/reference/cli.md, docs/reference/error-codes.md, mkdocs.yml, crates/CLAUDE.md</files>
  <behavior>
    - Every RPX-05xx/06xx/0102 code has a REGISTRY entry with non-empty title+cause+fix+recovery.
    - A malformed bus URL through `git-remote-reposix` surfaces `[RPX-0601]` on stderr; a too-few-args invocation surfaces `[RPX-0602]`; a missing-creds helper path surfaces `[RPX-0102]`.
    - The blob-limit / EOF / backend-unreachable / conflict helper paths carry `[RPX-0503]`/`[RPX-0502]`/`[RPX-0504]`/`[RPX-0505]` on the DIAG line (protocol-contract status strings stay verbatim).
    - `reposix explain RPX-0601` (a HELPER code, from the CLI binary) prints its entry — proving one explain resolves both binaries.
    - `docs/reference/error-codes.md` documents `reposix explain` + the RPX namespace; registered in mkdocs nav; mkdocs-strict + mermaid + banned-words + link-resolution green.
    - The `agent-ux/rpx-codes-registry` gate exits 0; the catalog row flips NOT-VERIFIED→PASS with an honest claim_vs_assertion_audit of the green run.
  </behavior>
  <action>
Add `.code()` to the helper user-facing sites: `malformed_bus_url_error`→RPX-0601 (bus_url.rs helper +
bus_handler bad-mirror), main.rs:115 usage→RPX-0602, `backend_dispatch::missing_env_error`→RPX-0102,
stateless_connect upload-pack→RPX-0501 / EOF→RPX-0502 / BLOB_LIMIT const→RPX-0503, write_loop
backend-unreachable DIAG→RPX-0504 / conflict DIAG→RPX-0505 (code on the diag, NOT the protocol string).
Populate the RPX-05xx/06xx/0102 REGISTRY entries in codes.rs. EXTEND
reposix-remote/tests/errors_teach_recovery.rs assertions to the tag (bin-target vs integration-target
seam: helper regression tests in a `#[cfg(test)]` bin module are invisible to `--test <name>` — grade
with the bare `cargo test -p reposix-remote` per crates/CLAUDE.md).

**Docs surface:** author `docs/reference/error-codes.md` (lead with the 80% use case: "hit an
`RPX-xxxx`? run `reposix explain RPX-xxxx`"; then the code index by category; push internals later —
progressive disclosure). Add a `reposix explain` section to `docs/reference/cli.md`. Register
error-codes.md in `mkdocs.yml` nav (near Exit codes). Run BOTH `bash
quality/gates/docs-build/mkdocs-strict.sh` AND `bash quality/gates/structure/banned-words.sh`
(fix-twice: a docs-build-only sweep let a plumbing term leak in P117) + mermaid + link-resolution. If a
new doc line carries a doc-alignment binding, rebind in the SAME commit.

**Gate GREEN + fix-twice:** run `bash quality/gates/agent-ux/rpx-codes-registry.sh` → exit 0; flip the
catalog row NOT-VERIFIED→PASS with the honest green-run audit. Update `crates/CLAUDE.md` error-message
convention: add the RPX registry location (`reposix-core::codes`), the `.code()` usage, the `reposix
explain` subcommand, and the `agent-ux/rpx-codes-registry` gate (revise to reflect the new state — not
an appended narrative).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo check -p reposix-remote 2>&1 | tail -3 && cargo nextest run -p reposix-remote 2>&1 | tail -8 && python3 quality/gates/agent-ux/rpx_registry_check.py 2>&1 | tail -3 && bash quality/gates/agent-ux/rpx-codes-registry.sh 2>&1 | tail -5 && bash quality/gates/docs-build/mkdocs-strict.sh 2>&1 | tail -3 && bash quality/gates/structure/banned-words.sh 2>&1 | tail -3</automated>
  </verify>
  <done>All helper RPX codes minted + registered + rendered + explain-able from the CLI binary; rpx-codes-registry gate exits 0 + row flipped PASS with honest audit; error-codes.md authored + in nav + mkdocs/banned-words/mermaid/link gates green; crates/CLAUDE.md revised. Commit.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| remote backend → error rendering | every REST byte (sim included) is attacker-influenced (OP-2); error headlines may echo a spec/URL |
| error string → stderr/stdout | stderr is an exfiltration leg (root CLAUDE.md § Threat model) |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-121-01 | Information disclosure | `errmsg::Teach` code slot + `codes::explain` output | mitigate | Registry is 100% `&'static str`; `.code()` takes a static code id; the Display code-limb interpolates ONLY `self.code`; `explain` reads only the static REGISTRY. W1 confirms by construction that no headline-derived or remote byte reaches the code slot or explain output. |
| T-121-02 | Information disclosure | error HEADLINE echoing a tainted spec/bus-URL | accept (pre-existing) | Unchanged from P120 — headline redaction is P120's job (`strip_url_userinfo`/`redact_userinfo` per crates/CLAUDE.md). P121 adds no new headline interpolation; the code + explain paths are static-only (T-121-01). |
| T-121-03 | Tampering | `rpx_registry_check.py` regex heuristic missing an indirection-built code | accept | Documented residual limit in the checker header (same honesty stance as P120's teach_scan.py); the codebase convention is inline `.code("RPX-xxxx")` literals, which the checker catches. Not papered over as "un-rottable". |
</threat_model>

<verification>
- `cargo nextest run -p reposix-core` + `-p reposix-cli --test explain --test errors_teach_recovery` +
  bare `-p reposix-remote` all green (one FOREGROUND cargo invocation at a time).
- `bash quality/gates/agent-ux/rpx-codes-registry.sh` exits 0; `python3 rpx_registry_check.py` +
  `--self-test` clean.
- Real-binary spot checks (from a `/tmp` mktemp -d, cd same invocation): `reposix explain RPX-0201`
  prints a code header + Fix + Recovery; `reposix explain --list` enumerates; `reposix explain RPX-9999`
  teaches + exits non-zero; a real `reposix init foo <tmp>` stderr carries `[RPX-0001]` + the Explain nudge.
- `bash quality/gates/docs-build/mkdocs-strict.sh` + `mermaid-renders.sh` + `structure/banned-words.sh`
  + `docs-alignment/walk.sh` green after the docs surface lands.
- Catalog-first: `git log` shows the W0 catalog/gate/checker SHA precedes every impl SHA.
- Push cadence: `git push origin main` BEFORE the verifier subagent; then `quality/runners/run.py
  --cadence post-push --persist` — `code/ci-green-on-main` (P0) must show main's newest ci.yml run success.
</verification>

<success_criteria>
- **SC1 (ROADMAP):** every user-facing CLI + helper error (the complete P120-dispositioned RETROFIT +
  bespoke set) emits a `[RPX-xxxx]` code — proven by the extended `errors_teach_recovery` suites + a real
  error-path grep on both binaries.
- **SC2 (ROADMAP):** `reposix explain <code>` exists and prints a non-empty cause + fix + copy-paste
  recovery for EVERY registry code — proven by the per-code assertions in tests/explain.rs + the gate.
- **SC3 (ROADMAP):** `reposix explain` output shape matches `rustc --explain E0308` (code header +
  extended body + fix section) — proven by the `rustc_parity_of_shape` test (rustc leg best-effort).
- **SC4 (derived — catalog-first invariant gate):** the committed `agent-ux/rpx-codes-registry` gate
  asserts registry completeness + uniqueness + format + unknown-code-teaches; `--list`/no-arg enumerates.
- **SC5 (derived — docs + fix-twice):** `reposix explain` + the RPX namespace are documented
  (error-codes.md in nav, cli.md section), mkdocs-strict + banned-words green, crates/CLAUDE.md revised.
- OP-2 threat cut confirmed: registry + code slot + explain output are `&'static str`-only.
</success_criteria>

<output>
After completion, create `.planning/phases/121-error-codes-explain/121-SUMMARY.md` (use
@$HOME/.claude/get-shit-done/templates/summary.md). Record: codes minted (final allocation map), the
registry location, the explain output shape, the SC1–SC5 evidence (real-binary transcripts), any
eager-fixes/filings, and the ownership-charter noticing section (non-empty).
</output>
