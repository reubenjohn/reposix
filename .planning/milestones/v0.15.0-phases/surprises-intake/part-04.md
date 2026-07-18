# v0.15.0 Surprises Intake — Part 4 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-16 | discovered-by: manager (w1:p7) finding to L0 #47, verified live | severity: MEDIUM (HIGH-visibility — uncatalogued hero-number surfaces)

**What:** Three hero surfaces present the NEW ~94.3%/~74.9% four-axis token-economy
figures as current with **NO doc-alignment catalog binding** — so once the owner's
pending 11-row confirm-retire batch lands, these headline claims become **entirely
uncatalogued** (no gate watches them for drift):
  - `docs/index.md:17` hero bullet — the new figure, restated at L42/L156/L168; only the
    **mermaid sub-numbers at L31-39 are bound**, the hero bullet itself is not.
  - `README.md:27` — new figure, unbound. **Verified: the only README token-economy
    catalog row is the RETIRING `README-md/token-89-percent`** — so post-retire, README's
    live number has zero binding.
  - `docs/concepts/reposix-vs-mcp-and-sdks.md:29-31` — new figure, unbound.
This is the same lying-hero-claim failure class the P115 destaling lanes purged from the
OLD numbers, re-appearing one layer up: the numbers are now CORRECT but UNWATCHED, so
the next drift (e.g. a P116/P117 re-measurement) could silently re-stale them with no
gate firing. Manager-verified live before routing to #47.

**Why out-of-scope for the discovering session (#47, filing turn):** Minting bindings is
a top-level `/reposix-quality-refresh` fan-out per doc (subagents propose claim→test
bindings with file:line citations, the `reposix-quality` binary validates + mints catalog
state) — a scoped multi-agent lane, not an inline edit. Filed at session-start per manager
directive; the refresh lanes execute at the first clean boundary (see below), not mid the
first-act verify turn.

**Sketched resolution:** Run the doc-alignment refresh flow (`/reposix-quality-refresh`
per doc, TOP-LEVEL) for each of the three surfaces, binding the live figure to
`bench_token_economy.py` tests / `headline-numbers-cross-check.py` — **the same test
targets the existing `output-reduction-94-percent` rows already bind to**, so this
extends an established binding pattern rather than inventing one. **Timing (manager
directive):** execute at the first clean boundary — before or right after P116 planning
locks — **do NOT let it slip past P117**. Once the 11-row retire batch lands, the README
surface in particular has no token-economy binding at all until this runs.

**STATUS:** **RESOLVED (2026-07-16, L0 #48 doc-alignment bind executor)** — all three
hero surfaces now BOUND in `quality/catalogs/doc-alignment.json`:
- `docs/index/hero-token-economy-94-75` (docs/index.md:17) → commit `c35f993`
- `README/hero-token-economy-94-75` (README.md:27) → commit `7553c36`
- `docs/concepts/reposix-vs-mcp-and-sdks/token-economy-output-cost` (concepts:29-31) → commit `aa75e96`

Each binds to `quality/gates/perf/headline-numbers-cross-check.py`
(`run_cross_check` asserts the exact hero substrings on the page vs canonical
`token-economy.md`) + `quality/gates/perf/bench_token_economy.py`
(`_assert_headline_reduction` pins the 94.3% median-of-3 output reduction from
committed captures) — the same test surface the existing
`output-reduction-94-percent` / `cost-reduction-75-percent` BOUND rows use.
`bash quality/gates/docs-alignment/walk.sh` exits 0 (no blocking states).
**NOTE:** the concepts-page row is conservatively scoped to the OUTPUT + COST axes
only; the cache-creation/input-context axes were NOT false-bound and are filed
separately below.

---

## 2026-07-16 | discovered-by: L0 #48 doc-alignment bind executor (OD-3 no-false-bound noticing) | severity: LOW

**What:** The concepts page `docs/concepts/reposix-vs-mcp-and-sdks.md:29-31` states a
FOUR-axis token-economy comparison: `~94.3% fewer output tokens`, `~66.0% fewer
cache-creation tokens`, `~55.6% smaller total input-context`, `~74.9% cheaper per
session`. Only the OUTPUT (94.3%) and COST (74.9%) axes are pinned by a test:
`headline-numbers-cross-check.py::run_cross_check`'s `TOKEN_CLAIMS` registry (L232-233)
cross-checks ONLY those two substrings on this file against canonical `token-economy.md`,
and `bench_token_economy.py::_assert_headline_reduction` asserts ONLY the real-capture
OUTPUT reduction (94.3% ±1.0%). The `~66.0% cache-creation` and `~55.6% input-context`
figures are **stated-but-untested**: no test asserts the committed captures yield those
specific real values. (`test_bench_token_economy.py::test_compute_arm_medians_and_reductions`
iterates all four axes — but only over SYNTHETIC 90%-by-construction seeds, proving the
reduction FORMULA is applied per-axis, NOT the real 66.0%/55.6% values.) The bind row
`docs/concepts/reposix-vs-mcp-and-sdks/token-economy-output-cost` was deliberately
NARROWED to the two tested axes rather than false-bound to all four (a FALSE BOUND is the
worst catalog-integrity failure).

**Why out-of-scope for the discovering session:** Closing this means either (a) adding two
`TOKEN_CLAIMS`-style substring cross-checks (`~66.0% fewer cache-creation`, `~55.6%
smaller total input-context`) for `HERO_CONCEPTS` to `headline-numbers-cross-check.py`,
plus reduction-axis extraction in `parse_token_canonical`, then a doc-alignment refresh to
widen the row's claim; or (b) a `bench_token_economy.py` assertion pinning the real-capture
cache_create/input_context reductions to a band (analogous to `_assert_headline_reduction`).
Both touch the perf-gate honesty surface and warrant their own catalog-row review — out of
scope for an additive bind pass.

**Sketched resolution:** Preferred = extend `TOKEN_CLAIMS` in
`quality/gates/perf/headline-numbers-cross-check.py` with cache-creation + input-context
entries for `HERO_CONCEPTS` (`compute_arm_medians` already yields both medians per arm — add
the reduction axes to `parse_token_canonical` / `compute_reductions` surfacing), then
`/reposix-quality-refresh docs/concepts/reposix-vs-mcp-and-sdks.md` to widen the bound
row's claim to all four axes. Until then the two extra axes remain correct-but-unwatched
(same class as the parent row above, one layer finer).

**STATUS:** OPEN

## 2026-07-16 | discovered-by: L0 #48 doc-alignment bind executor | severity: LOW

**What:** `reposix-quality doc-alignment bind --help` advertises the `--test` flag
generically as accepting `<file>::<fn>` citations ("Test fn citation (`<file>::<fn>`).
Repeatable -- one per binding"), but the validator (`parse_test` in
`crates/reposix-quality/src/commands/doc_alignment.rs`) only resolves the `::fn` form to
a hashable function when the file part ends in `.rs` (`TestRef::RustFn`); for any other
extension — Python included — the split never happens: the WHOLE `<file>::<fn>` string
is treated as a literal file path and existence-checked as `TestRef::File`, producing an
opaque `bind: --test #<i> \`<file>::<fn>\`: test file \`<file>::<fn>\` does not exist`
error (confirmed live against `headline-numbers-cross-check.py::run_cross_check`). Every
existing BOUND row pointing at a Python verifier — including the 3 minted this rotation —
stores the BARE path with no `::fn` suffix, matching the form the validator actually
accepts, so this is a help-text/validator mismatch, not a live bug blocking any row; it's
a sharp edge for the next agent who reads `--help` literally and tries to pin a specific
Python function.

**Why out-of-scope for the discovering session:** Discovered incidentally while minting
the 3 hero-number rows (task was to bind, not audit CLI help text). Fixing either side —
extending the validator to hash a specific Python function the way `.rs::fn` hashes a
Rust fn body, or narrowing the help text to say `::fn` is Rust-only — touches a shared
CLI/hash-computation code path used by every existing bound row, and warrants its own
review rather than a drive-by edit mid-mint.

**Sketched resolution:** Either (a) extend `parse_test`/`TestRef` to recognize
`<file>.py::<fn>` (needs a Python-side fn-body hash, e.g. AST-based, analogous to
`test_body_hash`'s syn-based Rust parse) so Python bindings can pin one function instead
of the whole file; or (b) the cheaper fix — reword the `--test` help string in
`crates/reposix-quality/src/commands/doc_alignment.rs` to state the `::fn` form is
Rust-only (`<file>.rs::<fn>`) and non-Rust verifiers always bind to the bare file path.
(b) matches the code comment already at `doc_alignment.rs:1550-1560`, which documents the
intended two-form contract — only the `--help` string is stale.

**STATUS:** OPEN

## 2026-07-16 23:50 | discovered-by: L0 #53 (P116 phase-close relief writer) | severity: HIGH

**What:** A CONCURRENT sibling worktree lane `gth-hook-curb-capture` (pid 2222573, live)
contaminated the SHARED `.git/config` at ~2026-07-16 17:29 with `core.bare = true` +
fixture identity (`user.email = t@t` / `user.name = t`). This blocked ALL work-tree git
operations on `main` (`git status`/`add`/`commit` all fail on a repo that believes it is
bare; the `.githooks/pre-commit` fixture-identity check would additionally reject any
commit attempted under `t@t`) — including P116's own phase-close verifier dispatch and
this rotation's handover commit. `origin/main` and all refs were intact throughout;
nothing durable was lost. #53 repaired it via a direct edit (`core.bare` → `false`,
removed the injected `[user]` block); the work-tree and the real committer identity were
restored, verified live (`cat .git/config`, `git status`, `git log -1`) before resuming
any write. This is a recurrence of the class first named in `S-260707-pr-08`.

**Why out-of-scope for the discovering session:** the coverage boundary is documented,
not novel — root `CLAUDE.md` § Non-negotiables "Leaf test setup" already states the
`.claude/hooks/leaf-isolation-guard.sh` PreToolUse hook fires only on the Claude Code
Bash *tool*: a sibling lane's leaf-test setup writing `.git/config` via a subprocess or
script bypasses it entirely, and the `.githooks/pre-commit` backstop catches fixture
*commits* on that path but not raw `git config`/`reposix init` config writes. Closing
that coverage boundary is a hardening-tool change (a new enforcement surface, or a
process/ownership fix for the sibling lane) — out of scope for a phase-close relief
handover, which is scoped to certifying P116 and advancing cursor state, not authoring
new guard tooling.

**Sketched resolution (candidates):** (a) extend the leaf-isolation guard to the
subprocess path — e.g. a git `config`-change hook, or a `.git/config` integrity check
wired into `pre-commit`/`pre-push` that fails loud on `core.bare = true` or a fixture
identity (`t@t`-shaped `[user]` block) appearing in the SHARED repo's config; (b) enforce
that leaf-setup lanes run in fully-isolated `/tmp` clones with pinned `GIT_CONFIG_*` env
vars so they structurally cannot write the shared `.git/config` at all; (c) notify the
sibling-lane owner — the manager (w1:p7 owns `gth-hook-curb-capture`) — so that lane is
fixed or paused before it corrupts the shared tree again. Route the owner-notification to
the manager; this SURPRISES row + the #53→#54 `SESSION-HANDOVER.md` are the durable
surfacing mechanism in the meantime. Cross-ref: root `CLAUDE.md` § Non-negotiables "Leaf
test setup" coverage-boundary note; `.planning/ORCHESTRATION.md` § Leaf isolation.

**STATUS:** OPEN

## 2026-07-16 23:59 | discovered-by: P117 W2 push-blocker Step A executor (catalog-corruption unblock) | severity: BLOCKER

**What:** `quality/catalogs/doc-alignment.json` row
`benchmarks/README-md/session-provenance` (minted `2026-07-16T23:05:16Z`, rationale
credits "Catalog-first mint (P117-03 SC5a, W2)") reached the tree with **three stacked
hand-edit corruptions**, none catchable by JSON-syntax validation alone because the file
is syntactically valid JSON throughout — only `reposix-quality`'s serde/validation layer
catches them, and it does so ONE AT A TIME (fix one, the next surfaces): (1) `next_action`
carried the HAND-WRITTEN token `"BIND"`, not a real `NextAction` enum variant (valid set:
`WRITE_TEST` / `FIX_IMPL_THEN_BIND` / `UPDATE_DOC` / `RETIRE_FEATURE` / `BIND_GREEN`) —
serde failed with `unknown variant 'BIND'` and EVERY `reposix-quality` invocation loads
this catalog, so the whole binary was unusable, blocking the P117 W2 push; (2) the row was
missing the required `last_verdict: RowState` field entirely (`missing field
'last_verdict'`) — confirmed via a full-catalog scan that this is the ONLY row (of 401) in
this state, i.e. isolated to this one hand-mint, not a systemic gap; (3) the row's `tests`
array had one entry (`quality/gates/perf/bench_token_economy.py`) while
`test_body_hashes` was empty, violating the `Row::validate_parallel_arrays` invariant
(`tests.len() != test_body_hashes.len()`) — the row's own rationale says
`test_body_hashes` was "deliberately omitted" pending a W6 refresh, so a non-empty
`tests` alongside an empty `test_body_hashes` was internally inconsistent with its own
stated intent. **Root cause:** the row was hand-typed directly into the JSON file instead
of minted through `reposix-quality doc-alignment <verb>` (`bind` / `mark-missing-test` /
`propose-retire`) — the only sanctioned catalog-mutation paths per
`quality/gates/docs-alignment/README.md` § Conventions ("Subagents NEVER write
`quality/catalogs/doc-alignment.json` directly. All state mutation flows through
`reposix-quality doc-alignment <subcmd>`"). Every verb that constructs a `Row` sets
`last_verdict` + keeps `tests`/`test_body_hashes` parallel by construction (see
`crates/reposix-quality/src/commands/doc_alignment.rs:703-726` `mark-missing-test`); a
hand-typed row structurally cannot inherit those invariants for free.

**Why out-of-scope for the discovering session:** this filing IS the discovering
session's own fix-twice obligation (not deferred elsewhere) — the code-level fix (enum
token → `WRITE_TEST`, add `last_verdict: MISSING_TEST`, empty the `tests` array) landed
in the SAME commit as this intake row per the router's Step-A charter. Filed here per OP-8
doctrine because the ROOT CAUSE (a catalog-mutation path that let a hand-typed row reach
the committed tree at all) is a process/tooling gap, not a one-row content bug — closing
it durably (a pre-commit/pre-push structural check that fails loud on any doc-alignment
row NOT traceable to a `reposix-quality doc-alignment <verb>` invocation) is new guard
tooling, out of scope for a bounded 3-blocker unblock step.

**Sketched resolution:** Add a structure-dimension gate (parallel to the existing
`structure/doc-alignment-catalog-present` / `-summary-block-valid` / `-floor-not-decreased`
rows in `quality/catalogs/freshness-invariants.json`) that fails loud when a
`doc-alignment.json` row change in a commit's diff was NOT produced by one of the
sanctioned `reposix-quality doc-alignment` verbs — e.g. require every row-touching commit
to also touch a verb-emitted marker (a `last_run`/`last_extracted_by` stamp already exists
per-row; a git-hook check that a hand-diff introduces a row with an unrecognized
`last_extracted_by` value, or omits it while adding new content, could catch this class
pre-commit rather than post-hoc at `reposix-quality` invocation time). Secondary,
cheaper mitigation: extend `quality/CLAUDE.md`'s mint-only reminder (this same commit) to
explicitly name the THREE invariants a hand-edit is likely to break (enum token validity,
required `last_verdict`, parallel-array lengths), so a future hand-edit temptation is met
with a documented reason to route through the binary instead.

**STATUS:** OPEN

## 2026-07-16 23:59 | discovered-by: P117 W3 sub-lane 117-06 (docs/social freshness gate + dead-code + CLAUDE.md sweep) | severity: MEDIUM

**What:** the leaf-isolation guard has a false-positive: a plain grep for a setup-verb
string as literal search text can false-positive-BLOCK. Specifically, `.claude/hooks/
leaf-isolation-guard.sh` guard B (`guard_leaf_setup_location`) and guard C
(`guard_shared_config_write`) false-positive-BLOCK a Bash command that merely GREPS FOR a
setup-verb string as literal search text (never invokes it), when that string sits inside
a regex ALTERNATION whose `|` reads as a shell command-position separator to
`at_command_position`'s scan. Live-reproduced (not hypothetical): a command as ordinary as
`grep -E "foo|reposix init" file.sh` — legitimately scanning a file for either of two
plain strings — trips guard B and BLOCKs with "leaf setup in shared tree", even though
`reposix init` never executes; it is dead fixed text inside a quoted `-E` pattern. Root
cause: `at_command_position()` (leaf-isolation-guard.sh:193-195) matches the verb regex
against the RAW, un-tokenized `$cmd` string via `grep -Eq`, so it cannot distinguish a
literal `|` used as a shell pipe (a real command-position separator) from a literal `|`
that is itself PART OF a quoted argument (regex alternation, awk/sed scripts, etc.) — the
guard has no shell-lexing/quote-awareness, only a flat string scan. Verified live: `grep
-rn "reposix init" quality/ crates/` (a plain substring search, this very lane's own
Task-2 dead-code-confirmation command) correctly ALLOWS (the string isn't adjacent to a
real separator), but `grep -E "foo|reposix init" <file>` BLOCKs on the identical effective
intent (search-for-text-only) merely because the alternation's `|` sits where the
CMD_POS_PREFIX regex expects a pipe. The same class applies to guard C's `git config`
verb inside an alternation.

**Why out-of-scope for the discovering session:** fixing `at_command_position` to be
quote-aware (skip matches inside single/double-quoted spans) is a nontrivial parsing
change to a security-load-bearing hook shared by every concurrent agent — the exact kind
of "new guard tooling" change OP-8 reserves for a dedicated hardening lane, not a
docs/social-gate + dead-code lane already carrying its own ~1h budget. It is also LOW
day-to-day impact today: it never actually blocked a real verify command in this lane
(both `render_results_markdown`-confirmation greps and the new gate's own asserts avoid
alternation over these specific verb strings), so it is a latent correctness gap, not a
live blocker.

**Sketched resolution:** teach `at_command_position` (or a small pre-pass) to strip
matched-quote spans (`'...'` / `"..."`) from `$cmd` before applying `CMD_POS_PREFIX` +
the verb regex — a plain `sed -E "s/'[^']*'//g; s/\"[^\"]*\"//g"` pre-pass would suffice
for the common case (no nested-quote escaping in practice for these verbs) and would turn
the `grep -E "foo|reposix init"` repro into a non-match without weakening the guard's real
catches (a genuine `reposix init` invocation is never itself inside quotes). Add a
selftest case mirroring `social-freshness.selftest.sh`'s structure once the fix lands.
Natural home: the same hardening lane that owns `.claude/hooks/leaf-isolation-guard.sh`
(P102-adjacent) or a dedicated guard-precision follow-up.

**STATUS:** OPEN

