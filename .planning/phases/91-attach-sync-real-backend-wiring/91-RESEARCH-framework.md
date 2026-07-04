# P91 Research — Framework / Litmus-Surface Lanes

**Scope:** catalog rows (A) + dark-factory harness / T2 litmus (B). Sibling doc
`91-RESEARCH-code.md` (not read/touched by this researcher) owns the Rust
push-planner / attach-wiring fix.

---

## A — Catalog rows

### A(a) — Exact row schema a P91 row must satisfy

Unified schema fields (`quality/catalogs/README.md:20-49`) a new/edited P91
row MUST carry:

- `id`, `dimension: agent-ux`, `cadences` (non-empty list), `kind`,
  `expected.asserts`, `verifier.script` + `timeout_s`, `artifact`, `status`,
  `last_verified`, `blast_radius` — baseline unified fields
  (`quality/catalogs/README.md:22-44`).
- **`minted_at`** — required, write-once, for any row whose `last_verified`
  lands on/after `2026-07-05T00:00:00Z` (`_audit_field.py:42`, the
  `P90_MINT_CUTOFF` constant). Any row a P91 task re-verifies with a
  timestamp on/after that cutoff MUST carry `minted_at` or catalog load hard-
  `SystemExit`s (`_audit_field.py:236-249`). Today is 2026-07-04, so a P91
  session that runs late (crossing UTC midnight) will trip this — safest is
  to add `minted_at` proactively on every row P91 touches.
- **`coverage_kind`** — required for any row P90's `is_transport_or_perf_row`
  classifies as a transport/perf claim (`_audit_field.py:81-97`): explicit
  `transport_claim: true`, OR (absent) a regex match over `comment`+`id`
  against `\b(push|fetch|round-?trip|clone|p50|p95|latency|throughput|real[- ]backend|transport|list_records|list_changed_since)\b`,
  gated to dimensions `{perf, agent-ux, security}` (`_audit_field.py:48-53`).
  **agent-ux is in that gated set**, so almost every P91 row (attach, sync,
  push, real-backend litmus) will be transport-classified by id/comment
  alone. Once `minted_at` is present, such a row MUST declare
  `coverage_kind: real-backend` or carry a compliant waiver
  (`_audit_field.py:259-277`) — `sim-only`/`mechanical`/`manual` values are
  valid enum members (`_audit_field.py:54`) but do **NOT** satisfy this
  specific load-time gate; only `real-backend` or a waiver does. **Critical
  nuance for `agent-ux/real-git-push-e2e`:** its verifier
  (`quality/gates/agent-ux/real-git-push-e2e.sh:1-30`) drives a real `git
  push` against a `reposix init`'d tree talking to the **local in-process
  sim**, not an external backend. If P91 re-mints this row with `minted_at`
  set, declaring `coverage_kind: real-backend` would be a **false claim**
  (F-K4a/D90-05 exists precisely to forbid this). The honest move is
  `transport_claim: false` (explicit opt-out of the transport
  classification) UNLESS the row is rewritten to actually drive a sanctioned
  real backend.
- **`claim_vs_assertion_audit`** — required (≥50 chars) for rows whose
  `last_verified`/`minted_at` is on/after `2026-05-08T00:00:00Z`
  (`_audit_field.py:37,250-258`) — effectively all P91 rows. Must state how
  `expected.asserts` would falsify the row's claim if the claim were false
  (existing rows in `quality/catalogs/agent-ux.json` model the required
  shape, e.g. lines 750-751 for `bus-write-audit-completeness` region, and
  the `real-git-push-e2e` row's own field, agent-ux.json's `real-git-push-e2e`
  entry).
- **`kind: shell-subprocess`** contract — if P91 mints a shell-subprocess row
  (recommended for the real-backend litmus; see B(c) below), it MUST declare
  a transcript contract at load time (`expected.artifact.transcript_path`,
  row-level `transcript_path`, or a "transcript" mention in `expected.asserts`
  — `_audit_field.py:65-78`), and at grade time the transcript file must
  actually exist and contain an `argv:` line
  (`_audit_field.py:100-122`, wired into `apply_pass_gates` at
  `_audit_field.py:182-220`) — otherwise a would-be PASS is demoted to FAIL
  (`_audit_field.py:201-208`). Transcripts write to
  `quality/reports/transcripts/<row-slug>-<RFC3339>.txt` with argv + sorted
  env var **names only** (never values) + cwd + exit_code + stdout/stderr
  (`quality/catalogs/README.md:47,49`); use the shared helper
  `quality/gates/agent-ux/lib/transcript.sh` for the write
  (`quality/catalogs/README.md:49`).
- **F-K4b congruence** — once `minted_at` is set, every `expected.asserts`
  entry must token-match ≥1 `asserts_passed` entry at grade time
  (`_audit_field.py:151-179`, `209-219`) — a P91 verifier that writes a
  terse `asserts_passed` list not covering every expected assert will FAIL
  even on exit 0.
- **OD-2 waiver prohibition** — any row carrying `cadences: [...,
  "pre-release-real-backend", ...]` MUST NOT carry a `waiver` block; catalog
  load hard-`SystemExit`s if it does (`_audit_field.py:292-300`). This is the
  mechanical enforcement behind D-03c / OD-2 (anti-C7 self-licensing-
  deferral-loop guard) and directly governs the
  `milestone-close-vision-litmus-real-backend` row (already `waiver: null`,
  `quality/catalogs/agent-ux.json` — confirmed by direct read of that row).

### A(b) — Recommended row set for P91

**Existing rows to flip/repair (not new mints unless noted):**

1. **`agent-ux/real-git-push-e2e`** (`quality/catalogs/agent-ux.json`,
   currently `status: WAIVED`, `waiver.until: 2026-07-31T00:00:00Z`,
   `waiver.tracked_in: "v0.13.0 SURPRISES-INTAKE.md 2026-07-04 QL-001 entry"`).
   Once the code lane (91-RESEARCH-code.md's owner) fixes the QL-001
   path-shape bug (`diff.rs:104-107`) + the fast-import stream-parser bug
   (`fast_import.rs:156-157`), P91's catalog-first commit sequence for this
   row is:
   - Remove the `waiver` block entirely (comment already documents the
     intent: "this row anchors the P90/P91 fix contract").
   - Re-run `real-git-push-e2e.sh`; expect `status: PASS` (git ≥ 2.34
     required — confirm the box's git version first; the script itself
     self-detects and exits 75/NOT-VERIFIED on git < 2.34, per its own
     header comment).
   - Add `minted_at` (this flip crosses the cutoff).
   - Set `transport_claim: false` UNLESS this task also upgrades the
     verifier to drive a real sanctioned backend — see the false-claim
     nuance in A(a) above. If a real-backend variant is added instead,
     use `coverage_kind: real-backend` honestly.
   - Add `cadences: ["pre-pr", ...]` back — the row's own `owner_hint`
     explicitly says D-CONV-1 stripped it from `pre-pr` "at which point that
     phase MUST re-add pre-pr per the SURPRISES.md contract" — that phase is
     P91.
   - `claim_vs_assertion_audit` already present and adequate; re-verify it
     still describes the fixed behavior accurately (no edit needed if the
     bug fix doesn't change the claim shape).

2. **`agent-ux/dvcs-third-arm`** (currently PASS, `last_verified:
   2026-07-04T09:00:04Z`). Per B(a)/B(f) below, its harness builds an EMPTY
   work tree + empty bare mirror (p86 F13 / `REMEDIATION-PLAN.md:31` row
   `H-A7`, `RBF-A-05`). If P91 (or a sibling task) implements `RBF-A-05`
   (populate the tree + mirror to exercise the 5 reconciliation cases), the
   row's `expected.asserts` MUST be rewritten to require non-vacuous
   `matched>0`/`no_id>0` etc. counts (not just the shape-only regex at
   `dvcs-third-arm.sh:169`), and `claim_vs_assertion_audit` must be updated
   to describe why a buggy reconciliation could no longer hide behind an
   all-zeros report.

3. **`agent-ux/milestone-close-vision-litmus-real-backend`** (currently
   `status: NOT-VERIFIED`, `cadence: pre-release-real-backend`, `blast_radius:
   P0`, `waiver: null`, verifier is a stub that unconditionally exits 75 —
   see B(a)/B(b)/B(c) below). This is the row P91 exists to make
   executable. Do NOT touch its `waiver` field (must stay absent per OD-2 —
   `_audit_field.py:292-300`). DO rewrite
   `quality/gates/agent-ux/milestone-close-vision-litmus.sh` to actually run
   the litmus scenario against a sanctioned target (Confluence TokenWorld
   default per D90-06) instead of the current unconditional-exit-75 stub —
   see B(c) for the concrete procedure. Recommend converting `kind` to
   `shell-subprocess` (currently already `shell-subprocess` in the catalog —
   confirmed direct read) so the transcript-evidence PASS gate applies
   honestly to a P0 row.

4. **New rows to consider minting** (if P91's code lane adds real-backend
   wiring to `attach`/`sync` beyond what dvcs-third-arm covers): a
   `agent-ux/reposix-attach-against-real-confluence` row mirroring
   `agent-ux/reposix-attach-against-vanilla-clone`'s shape
   (`quality/catalogs/agent-ux.json:43-77`) but driving `confluence::TokenWorld`
   with `cadence: pre-release-real-backend` and `coverage_kind: real-backend`
   set honestly from the start (avoids the retrofit problem row #1 above
   has). Follow the `_provenance_note` hand-edit convention already used by
   every agent-ux row (bind verb doesn't cover this dimension yet —
   `quality/catalogs/agent-ux.json:46` et al.).

### A(c) — `cadence: pre-release-real-backend` row behavior

- **Env gate** (`_realbackend.py:57-64`): a row tagged this cadence is
  skipped (forced to `NOT-VERIFIED`) unless `REPOSIX_ALLOWED_ORIGINS`
  contains a non-loopback `http(s)` origin (loopback regex covers
  `127.0.0.1`, `localhost`, `0.0.0.0`, `[::1]` in any spelling —
  `_realbackend.py:31-34`) AND at least one full credential set is present
  (Confluence: `ATLASSIAN_API_KEY`+`ATLASSIAN_EMAIL`+`REPOSIX_CONFLUENCE_TENANT`;
  GitHub: `GITHUB_TOKEN`; JIRA: `JIRA_EMAIL`+`JIRA_API_TOKEN`+
  `REPOSIX_JIRA_INSTANCE` — `_realbackend.py:21-25`).
- **Sanctioned-target membership is NOT checked by `_realbackend.py`** —
  that module only checks non-loopback + credential-completeness, never
  whether the resolved target is one of the three sanctioned targets
  (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`). Per D90-06
  (`.planning/phases/90-framework-fixes-honesty-rules/90-DECISIONS.md:103-111`),
  that proof obligation belongs in **the litmus verifier body P91 writes** —
  see B(c).
- **Exit-code mapping** (`_realbackend.py:81-98`, wired into `run.py:337`):
  `0`→PASS, `2`→PARTIAL, `75`→NOT-VERIFIED (sysexits.h `EX_TEMPFAIL`
  repurposed), anything else→FAIL. A P91 verifier that fails a real-backend
  precondition it cannot recover from (e.g., target unreachable) should exit
  75, NOT 1 — exiting 1 would let the runner overwrite an honest deferral
  signal with a hard FAIL.
- **Skip is fail-closed, not preserve-status** (`run.py:182-209`,
  RBF-FW-07b/D90-04): a cred-less run on a `pre-release-real-backend` row
  ALWAYS flips (and persists) the row to `NOT-VERIFIED`, writing
  `skip_reason: "env-missing"` + human `skip_detail` into the artifact
  (`run.py:194-195`). The prior real grade is preserved once in
  `last_real_grade`/`last_real_verified` (`run.py:203-205`,
  `quality/catalogs/README.md:41`) — never overwritten by a second skip.
- **OD-2 hard-RED distinction** (`quality/PROTOCOL.md:137-148`): NOT-VERIFIED
  (env absent) and creds-missing-at-milestone-close are DIFFERENT states.
  Once P91-P95 land the substrate, failure to EXECUTE this cadence at
  milestone-close is hard RED — no waiver, no `until_date`, no
  PASS-with-comment, enforced both by the OD-2 prose and mechanically by the
  waiver-prohibition SystemExit at `_audit_field.py:292-300`.

### A(d) — What `verdict.py` / pre-push currently require (so P91 flips don't break pre-push)

- `quality/runners/verdict.py` composes `compute_color`/`compute_exit_code`
  then applies a **darken-only** milestone-adversarial gate
  (`quality/PROTOCOL.md:199-206`) — irrelevant to per-phase pre-push, but
  relevant if P91's changes get graded at milestone-close.
- Pre-push cadence currently runs (full enumeration via direct catalog
  query across every `quality/catalogs/*.json` file, `cadences` field):
  - `agent-ux`: `kind-shell-subprocess-worked-example`,
    `test-name-vs-asserts`, `absorption-honesty-template-present`,
    `milestone-adversarial-pass`.
  - `code`: `clippy-lint-loaded`, `fixtures-valid`, `cargo-fmt-check`,
    `cargo-clippy-warnings`.
  - `docs-build`: `mkdocs-strict`, `mermaid-renders`, `link-resolution`,
    `badges-resolve`.
  - `docs-repro`: `snippet-coverage`.
  - `structure` (`freshness-invariants.json`, 22 rows incl.
    `banned-production-tokens`, `deferral-pointer-linter`,
    `coverage-kind-required`, `minted-at-write-once`,
    `verifier-missing-demotes`, `skip-fail-closed-with-history`,
    `shell-subprocess-transcript-runtime`, `asserts-congruence-grade-time`
    — these last 6 are P90's own framework-honesty regression rows and will
    re-validate any P91 catalog edit against the exact rules in A(a)) plus
    `docs-alignment/walk`.
  - `orphan-scripts.json`: 8 rows including `preflight-real-backends-sh`.
  - **None of P91's target agent-ux rows (`real-git-push-e2e`,
    `dvcs-third-arm`, `milestone-close-vision-litmus-real-backend`) are
    currently pre-push-tagged** — `real-git-push-e2e` is
    `["pre-release", "on-demand"]`, `dvcs-third-arm` is `["pre-pr"]`
    (not pre-push), and the litmus row is `["pre-release-real-backend"]`.
    So flipping their status will not, by itself, change pre-push's exit
    code — but re-adding `pre-pr` to `real-git-push-e2e` per A(b)#1 means
    the `pre-pr` CI job (not local pre-push) will exercise it. Catalog edits
    still run through `load_catalog`'s `_audit_field.validate_row` at
    **every** invocation, pre-push included, since that validation is
    unconditional at load time regardless of cadence scope
    (`run.py:394-400`, `load_catalog` called once per catalog file
    `run.py:394` before `is_in_scope` filtering) — so a malformed P91 edit
    (missing `minted_at`, missing `coverage_kind`, forbidden waiver) WILL
    break every cadence's runner invocation, including pre-push, even if the
    row itself isn't pre-push-scoped.

---

## B — Dark-factory harness + T2 litmus

### B(a) — Current dvcs-third-arm.sh work-tree shape

Confirmed empty-tree, per p86 F13
(`.../02-phase-audits-may08/phase-audit-p86.md:193-200`, also catalogued as
`H-A7` in `REMEDIATION-PLAN.md:31` and its fix as `RBF-A-05` at line 210):

- `git init --quiet "$WORK_REPO"` (`dvcs-third-arm.sh:131`) — no content.
- `git init --bare --quiet "$MIRROR_BARE"` (`dvcs-third-arm.sh:134`) — no
  content.
- Reconciliation report assert is shape-only:
  `grep -qE 'matched=[0-9]+ no_id=[0-9]+ backend_deleted=[0-9]+
  mirror_lag=[0-9]+'` (`dvcs-third-arm.sh:169`) — accepts `matched=0
  no_id=0 backend_deleted=0 mirror_lag=0` (and no `orphan=` field is
  asserted at all — duplicate-id/orphan case is entirely unexercised).

**What `RBF-A-05` requires** (per `REMEDIATION-PLAN.md:210`, "the fix"): the
harness must create a **NON-EMPTY** work tree + populated bare mirror before
`reposix attach` runs, with content shaped to exercise each of the 5
reconciliation cases from the architecture-sketch (matched OID alignment,
no-id file, backend-deleted record, mirror-lag drift, duplicate-id/orphan
hard-error) — then assert non-zero, case-specific counts (e.g. `matched=1
no_id=1 backend_deleted=1 mirror_lag=1`), not just the regex shape. This
requires: (1) seeding the sim (`spawn_sim seeded` already called at
`dvcs-third-arm.sh:55` — check what `seeded` populates and align mirror
content to match those records' `id` frontmatter), (2) writing matching
markdown files with frontmatter `id` fields into `$WORK_REPO` before `git
init`/`commit`, pushed into `$MIRROR_BARE` as the "vanilla clone" baseline,
(3) deliberately omitting an `id` field from one file (no-id case), (4)
deleting/closing one record backend-side after seeding the mirror
(backend-deleted case), (5) advancing the sim's cursor without updating the
mirror (mirror-lag case).

### B(b) — D90-06 sanctioned-target litmus-body criterion + current state

D90-06 (`90-DECISIONS.md:103-111`, quoted): *"The env-gate stays a skip
heuristic. The proof obligation (litmus verifier asserts the resolved
target is one of the sanctioned three and fails loud) belongs in the litmus
body P91 writes — exactly as the intake sketch proposes. P90 records this
as a named P91 acceptance criterion in the ROADMAP amendment... rather than
adding a second, weaker allowlist check in `_realbackend` that would
duplicate the real assertion."*

**Current state of `quality/gates/agent-ux/milestone-close-vision-litmus.sh`:**
it does **NOT** yet assert anything about the target. The entire script
(24 executable lines) unconditionally writes a `NOT-VERIFIED` artifact and
`exit 75` regardless of environment — see the script body: it writes
`{"reason":"substrate_not_landed","blocked_on":["P91","P92","P93","P94","P95"]}`
and exits 75 every time it is invoked (`milestone-close-vision-litmus.sh:36-50`).
This is the honest SLOT stub P89 shipped; P91 is the phase chartered to
replace this stub body with real logic, per the script's own header comment
(`milestone-close-vision-litmus.sh:2-5`: *"Substrate dependency: P91 (real-
backend attach)..."*).

### B(c) — Concrete executable T2-vs-TokenWorld litmus procedure for the REOPEN gate

Recommended body for the rewritten
`quality/gates/agent-ux/milestone-close-vision-litmus.sh`, informed by
T2-attach.md's real run and `docs/reference/testing-targets.md`:

1. **Resolve + assert sanctioned target.** Read
   `REPOSIX_CONFLUENCE_TENANT` (or GitHub/JIRA equivalents); assert the
   resolved host/space matches one of: Confluence tenant `reuben-john` space
   `TokenWorld`, GitHub repo `reubenjohn/reposix`, or JIRA project `TEST`
   (default) / `$JIRA_TEST_PROJECT` / `$REPOSIX_JIRA_PROJECT`
   (`docs/reference/testing-targets.md:48-155`). Fail loud (non-75, i.e. a
   real FAIL) if the env resolves to something else — this is the D90-06
   proof obligation; it must not be satisfied by prose alone.
2. **Preflight.** `bash scripts/preflight-real-backends.sh`
   (`docs/reference/testing-targets.md:23-44`) — exit 0 required before
   proceeding; exit 1/2 maps to this litmus row exiting 75 (NOT-VERIFIED —
   target/creds present per env-gate but unreachable is a different failure
   class than creds-absent, but per OD-2 this must be evaluated carefully:
   at milestone-close, unreachable-when-creds-are-set is likely closer to
   the hard-RED "substrate exists, cannot execute" state, NOT a legitimate
   75 — flag this distinction explicitly in the rewritten script's
   comments).
3. **Commands a fresh agent runs** (mirrors T2's Step 2 in
   `T2-attach.md:32-46`, but this time against a POPULATED mirror per
   RBF-A-05 groundwork, and using `confluence::TokenWorld` which per T2's
   Step 3 finding (F6) must actually be wired by this point):
   ```
   git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git /tmp/litmus-run
   cd /tmp/litmus-run
   reposix attach confluence::TokenWorld --remote-name reposix
   $EDITOR pages/<some-page>.md   # or reuse an existing seeded fixture
   git commit -am 'litmus edit'
   git push reposix main
   ```
4. **The 5 pass boxes** (mirrors T2-attach.md's "Goal outcome" checklist,
   `T2-attach.md:117-125`):
   - [ ] Vanilla clone obtained
   - [ ] `reposix attach` ran and modified git config
     (`extensions.partialClone=reposix`, `remote.reposix.url` present)
   - [ ] Edit + commit succeeded
   - [ ] `git push` succeeded (helper round-trip, not a synthetic stream)
   - [ ] Server-side change confirmed via REST (`reposix list --backend
     confluence --project TokenWorld` shows the edit, or equivalent REST
     read)
5. **Dual-table audit assertion** — query both `audit_events_cache` (cache
   crate) and `audit_events` (core crate, written by the confluence adapter)
   for rows matching this run, per OP-3 (CLAUDE.md "Audit log is non-
   optional") and the existing catalog row's own `expected.asserts`
   (`agent-ux/milestone-close-vision-litmus-real-backend`'s asserts already
   require this — confirmed direct read of the row).
6. **HIGH-friction severity rubric** (per T2-attach.md's finding severities
   `T2-attach.md:105-115`): a HIGH finding is any point where the documented
   happy path (README / `docs/concepts/dvcs-topology.md` Pattern C /
   `--help`) DISAGREES with what the binary actually does — T2's own F6/F7
   are the canonical HIGH shape ("docs say it works, binary says not yet
   wired," compounded by leaking internal phase IDs to stderr with no
   recovery hint). If the rewritten litmus hits an equivalent disagreement,
   it should FAIL loud (not silently downgrade to NOT-VERIFIED) — this is
   exactly the OD-2 "substrate exists, cannot execute" hard-RED case, not a
   deferral.
7. **Transcript artifact** — per `kind: shell-subprocess`'s contract (A(a)
   above), write the transcript to
   `quality/reports/transcripts/milestone-close-vision-litmus-real-backend-<RFC3339>.txt`
   using `quality/gates/agent-ux/lib/transcript.sh`, recording every command
   above as real subprocess invocations (argv + env var NAMES only + cwd +
   exit_code + stdout/stderr).
8. **Cleanup convention** — tag any created Confluence page with a
   `kind=test` label per `docs/reference/testing-targets.md:75-79` sweep
   convention. **NEVER delete pages 7766017 or 7798785** (project-owner
   fixture pages the mission explicitly flags as protected — this
   constraint is NOT found verbatim in `testing-targets.md`, which only
   describes a generic manual-delete-leftover-pages cleanup step
   [`docs/reference/testing-targets.md:73-79`]; treat the two page IDs as an
   external constraint from the mission brief that the litmus script must
   still honor even though the doc doesn't enumerate them — flag this doc
   gap in NOTICED below).

### B(d) — REQUIREMENTS.md status + architecture-sketch "shipped" overstatement

`.planning/milestones/v0.13.0-phases/REQUIREMENTS.md` DVCS-ATTACH rows
(quoted verbatim, lines 45-48):

> - [x] **DVCS-ATTACH-01**: `reposix attach <backend>::<project>` subcommand
>   in `crates/reposix-cli/`. In CWD: builds fresh cache directory derived
>   from `<backend>::<project>` (NOT `remote.origin.url`, per Q1.1);
>   REST-lists backend; populates cache OIDs lazily; reconciles by walking
>   current HEAD tree matching files to backend records by frontmatter `id`;
>   adds remote `reposix::<sot-spec>?mirror=<existing-origin-url>` (or
>   `reposix::<sot-spec>` if `--no-bus`); sets
>   `extensions.partialClone=<remote-name>`. Existing `origin` keeps
>   plain-git semantics.
> - [x] **DVCS-ATTACH-02**: Reconciliation cases per `architecture-sketch.md`
>   § "Reconciliation cases": match (OID alignment), backend-deleted
>   (warn+skip+`--orphan-policy={delete-local,fork-as-new,abort}`), no-id
>   (warn+skip), duplicate `id` (hard error), mirror-lag (cache marks for
>   next fetch). Each row has a corresponding test case.
> - [x] **DVCS-ATTACH-03**: Re-attach with different SoT spec is REJECTED
>   with clear error per Q1.2... Re-attach with same SoT is IDEMPOTENT per
>   Q1.3...
> - [x] **DVCS-ATTACH-04**: `Cache::read_blob`... returns `Tainted<Vec<u8>>`
>   per OP-2. Verified by static type-system assertion + runtime integration
>   test in `crates/reposix-cli/tests/attach.rs`.

And the lookup/traceability table (`REQUIREMENTS.md:124-133`):

> | REQ-ID | Phase | Status |
> |--------|-------|--------|
> | DVCS-ATTACH-01 | P79 | shipped |
> | DVCS-ATTACH-02 | P79 | shipped |
> | DVCS-ATTACH-03 | P79 | shipped |
> | DVCS-ATTACH-04 | P79 | shipped |

**Correction to the mission brief:** this "shipped" lookup table lives in
`.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:124-139`, NOT in
`.planning/research/v0.13.0-dvcs/architecture-sketch/index.md` (that file
was searched directly; its only "Status" line is a single "Pre-roadmap
research" banner at `index.md:5` — no phase-status lookup table exists
there). Flagging this so the planner doesn't waste a cycle looking for a
table that isn't in the file the mission named — see NOTICED.

**The overstatement itself:** all four DVCS-ATTACH rows are marked `[x]`
"shipped" against P79, but the real-run finding T2-attach.md (dated
2026-05-02, i.e. AFTER P79 closed) reproduces `reposix attach
confluence::REPOSIX` failing with `Error: attach: backend "confluence" not
yet wired in P79-02 scaffold (sim only); github/confluence/jira land
alongside the integration tests in P79-03` (`T2-attach.md:56-61`, finding
F6, rated HIGH). This is the exact `H-A3`/`H-A4` pattern in
`REMEDIATION-PLAN.md:23-24`: *"p79 F2 — 79-03 PLAN silently dropped real-
backend wiring 79-02 promised"* and *"p79 F3 — verifier mistook 'tests pass'
for 'feature ships'; DVCS-ATTACH-01..04 graded sim-only."* The `[x] shipped`
label is true only for the `sim::*` backend path; it silently omits that
`github`/`confluence`/`jira` backends were never wired for `attach` — this
is precisely what P91's code lane must close and the mission brief already
anticipates.

### B(e) — CLAUDE.md attach example: current incoherence + coherent replacement

Verbatim current block (`CLAUDE.md:108-112`, "Commands you'll actually use"):

```
# Attach an existing checkout (vanilla GH-mirror clone or hand-edited tree, v0.13.0+)
git clone git@github.com:org/issues-repo.git /tmp/issues  # vanilla mirror clone (no reposix needed)
cd /tmp/issues
reposix attach sim::demo --remote-name reposix            # build cache from REST; reconcile by frontmatter id; add reposix remote
git push reposix main                                     # push via reposix remote (single-SoT shape; bus URL form requires P82+)
```

**Incoherence:** line 1 clones a placeholder GitHub-shaped mirror
(`org/issues-repo`), but line 3 attaches the `sim::demo` backend — a
simulator SoT bound to a tree that was cloned from a GitHub-shaped remote.
Nothing about `sim::demo`'s records has anything to do with the cloned
mirror's content; this is the p79-class F8 pattern (`--help`/docs promise
one flow, but the concrete example wires unrelated pieces together) applied
to CLAUDE.md itself. It also carries a bare `P82+` phase-ID reference with
no recovery meaning to a reader (see B(f) for why no gate catches this).

**Proposed coherent replacement** (mirrors the real bus/mirror shape this
project actually ships, matching T2-attach.md's Pattern C and the mission's
own "sanctioned target" framing):

```
# Attach an existing checkout to a real SoT with a GH mirror (v0.13.0+ bus shape)
git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git /tmp/issues  # vanilla clone of the GH MIRROR (not the SoT)
cd /tmp/issues
reposix attach confluence::TokenWorld --remote-name reposix --mirror-name origin  # SoT=confluence; folds existing `origin` into ?mirror=
git push reposix main                                     # bus push: confluence-first, GH mirror best-effort (see § Load-bearing behaviors)
```

This keeps `git clone`'s target (the mirror) and `attach`'s SoT argument
(the backend the mirror mirrors) coherent, uses a real sanctioned target
instead of a placeholder `org/issues-repo`, and drops the bare `P82+`
reference in favor of the actual cross-reference already used two lines
above it (`§ Load-bearing behaviors`). If P91 does not want to commit to
naming TokenWorld in CLAUDE.md before the confluence-attach wiring lands,
a minimal fix is simply to make the clone/attach spec pair coherent
(`git clone .../reposix.git` + `reposix attach github::reubenjohn/reposix`)
without introducing the bus/mirror form at all.

### B(f) — Pre-push gate surface + P82+/banned-token confirmation

**Full pre-push gate surface** — see A(d) above for the enumerated list (24
distinct catalog IDs discovered by direct query across all 8 catalog
files' `cadences` arrays).

**Does `banned-production-tokens.sh` catch `P82+`?** No. Confirmed by direct
read: the script's pattern is `PATTERN='\bP[0-9]{2,3}-[0-9]+\b'`
(`banned-production-tokens.sh:35`) — it requires a literal
`-<digits>` suffix (e.g. `P82-01`). `P82+` (trailing plus, no dash-digit) does
**not** match this regex. This is intentional per the script's own header
(`banned-production-tokens.sh:9-25`: "CATCHES: v0.13+ phase numbers —
`\bP\d{2,3}-\d+\b`... INTENTIONALLY MISSES:..." — though the header's
miss-list only names the P0-2/P1-1 audit-ID class, not the bare `P82+`
shape; this is a second, undocumented miss).

**Does `deferral-pointer-linter.sh` own the `P82+` shape, as CLAUDE.md
claims?** Partially, and with a gap. Confirmed by direct read of
`sync.rs:14` (doc comment) — *"the bus remote (P82–P83) names this command
in its reject-path stderr hints"* — does not match any of the linter's 3
patterns (`not yet wired in P[0-9]+` / `lands? (alongside|in) P[0-9]+` /
`substrate-gap-deferred`, `deferral-pointer-linter.sh:33-37`) either, since
"P82–P83" isn't preceded by "lands"/"land" adjacent to "in"/"alongside".
Separately, `sync.rs:44` (`"...real-backend wiring lands in P82+..."`) DOES
match `lands? (alongside|in) P[0-9]+` (extracts `P82`, resolves against
`.planning/phases/82-*/` which exists — PASS). But `sync.rs:94` (the
**actual user-facing `bail!` error text**: `"...github/confluence/jira land
alongside the bus-remote work in P82+"`) does **NOT** match any pattern —
the words "alongside" and "P82" are not adjacent (separated by "the
bus-remote work in"), so the regex's contiguous 3-token requirement fails.
**This means the single most user-visible `P82+` string in
`crates/reposix-cli/src/sync.rs` (the one an agent actually sees in stderr
when `sync --reconcile` rejects a non-sim backend) is caught by NEITHER
structure gate.** If P91's code lane removes/rewords this string when
wiring real backends, no automated gate will flag a stale leftover if it's
missed — this must be caught by manual code review or the verifier
subagent reading `sync.rs` directly, not by `git push`. Additionally, both
structure scripts only scan `crates/**/*.rs`
(`banned-production-tokens.sh:47`, `deferral-pointer-linter.sh` `grep -rnHE
"$pat" crates/`) — **neither scans CLAUDE.md, docs/, or
architecture-sketch files**, so the `P82+` reference inside CLAUDE.md
itself (B(e) above) is entirely outside both gates' reach by design.

---

## NOTICED (per ownership charter clause 2)

1. **T2-attach.md file path mismatch in the mission brief.** The mission
   named `.../01-dark-factory-may02/T2.md`; the actual file is
   `T2-attach.md` in that same directory. Filed here rather than silently
   working around it, since a future agent hitting the same wrong path
   should not have to re-discover the rename.
2. **Cleanup page-ID protection (7766017/7798785) is not documented in
   `docs/reference/testing-targets.md`.** The mission brief names two page
   IDs that must never be deleted during Confluence litmus cleanup, but
   `testing-targets.md`'s Confluence cleanup section only says "manually
   delete leftover pages... Do not leave junk pages lying around"
   (`docs/reference/testing-targets.md:73-79`) with no allowlist/blocklist
   of protected page IDs. Either the mission brief has out-of-band owner
   knowledge not yet written down, or this doc is missing a load-bearing
   safety constraint — recommend P91 add an explicit "NEVER delete these
   page IDs" note to `testing-targets.md` in the same PR that writes the
   real litmus body, so the constraint lives in a committed artifact
   instead of only in agent-to-agent oral tradition.
3. **`agent-ux/dvcs-third-arm`'s own `claim_vs_assertion_audit` and
   `expected.asserts` already over-claim relative to what the harness
   verifies.** The row's asserts include "cache materialization... audit_events_cache
   contains an attach_walk row" (true, exercised) alongside "reconciliation
   walk emits `matched=N no_id=N backend_deleted=N mirror_lag=N` line" (true
   but vacuously — see B(a)); the row is `status: PASS` today. This is not a
   P91 blocker, but P91 inherits an already-PASS row whose PASS partly rests
   on an assert that doesn't distinguish "reconciliation ran" from
   "reconciliation is correct" — worth folding into the RBF-A-05 rewrite in
   the same commit rather than leaving the row honest-but-weak.
4. **`scripts/dark-factory-test.sh` shim vs canonical path** (p86 F14,
   `phase-audit-p86.md:203-215`) — confirmed the shim still exists
   (`exec bash .../quality/gates/agent-ux/dark-factory.sh "$@"`) and CLAUDE.md
   still references both the shim (line 189 region, sim arm) and the
   canonical path (line 191 region, dvcs-third-arm) for what is
   conceptually the same dispatcher. Not a P91 blocker but a small
   consistency debt the CLAUDE.md-update step could close cheaply while
   already editing the "Commands" section for B(e).
5. **`reposix-agent-flow` skill's GitHub cleanup claim disagrees with
   `testing-targets.md`'s own GitHub cleanup section.** The skill says
   "Cleanup is automatic via `gh issue close`" (`SKILL.md` "Canonical test
   targets" section), but `docs/reference/testing-targets.md:106-111` says
   cleanup is manual today ("The Phase 36 cleanup automation will handle
   this; for now manual cleanup..."). Minor doc-drift; flagging since P91's
   real-backend litmus will exercise this exact cleanup path and an agent
   trusting the skill's claim would be surprised.

## Planner warnings

- **Do not let the code lane (91-RESEARCH-code.md) rename/remove the
  `sync.rs:94` `P82+` string without a corresponding catalog-row or manual
  review step** — no structure gate will catch a stale or malformed
  replacement (B(f)).
- **`agent-ux/real-git-push-e2e`'s waiver retirement is coupled to git
  version on the execution box.** The verifier itself gates on git ≥ 2.34
  and exits 75 (NOT-VERIFIED) below that — confirm the box's `git
  --version` before assuming the fix flips this row to PASS; it may
  legitimately land at NOT-VERIFIED instead if the environment gap (not the
  code bug) is what's active on this machine.
- **`coverage_kind: real-backend` is a hard, checked claim, not decoration**
  — any P91 row edit that adds `minted_at` to an agent-ux-dimension row
  whose id/comment matches the transport regex (very likely for
  attach/sync/push rows) MUST get `coverage_kind` or `transport_claim:
  false` right the first time, or catalog load `SystemExit`s and breaks
  every cadence's runner invocation, including unrelated ones (A(d)).
- **The milestone-close litmus script rewrite is the single highest-
  leverage deliverable of this phase** — until it stops unconditionally
  exiting 75, no version of P91-P95's substrate can ever make the P0
  `milestone-close-vision-litmus-real-backend` row anything but
  NOT-VERIFIED, which blocks every future milestone close per OD-2.
