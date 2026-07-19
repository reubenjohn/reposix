# Phase 125: Real-backend cadence & mirror-drift resilience - Research

**Researched:** 2026-07-18
**Domain:** DVCS mirror-drift self-healing (bash test-harness + Rust helper teaching strings + docs cadence wiring)
**Confidence:** HIGH (code paths and manager rulings verified against reality; MEDIUM/LOW flagged inline for design-decision items the planner must resolve)

## Summary

This phase closes a real, previously-diagnosed gap: the milestone-close vision-litmus
(`quality/gates/agent-ux/milestone-close-vision-litmus.sh` + its sourced
`lib/litmus-flow.sh`) can false-negative on its own second run because its **own prior
successful push re-stales the GitHub mirror it reads on the next clone** (DRAIN-02),
and separately has **zero self-heal logic** for either a trashed backend fixture or a
stale mirror clone (DRAIN-12). Both gaps were already diagnosed and design-sketched by
the project (`GTH-V15-09`, `SURPRISES-INTAKE` part-02 2026-07-14 entry) — this phase is
the scheduled implementation of that sketch, not net-new design work.

A 2026-07-16 **owner-ruled decision already constrains the solution space**
(`.planning/CONSULT-DECISIONS.md:113-136`): the webhook + 30-minute cron GH Action is
BLESSED as the authoritative external-mirror convergence mechanism, and
`scripts/refresh-tokenworld-mirror.sh` is explicitly named as "the manual op-recovery
(incl. the documented litmus pre-step)." A deeper product-code fix — rewriting the bus
push's mirror fan-out to push a post-write, backend-materialized tree instead of the
pre-write client tree (`GTH-V15-38`, "Option C") — is **explicitly NOT sanctioned** for
this milestone; it is pull-forward-gated on "a real incident or recurring operational
friction from the litmus pre-step." **This phase must stay in the test-harness /
teaching-string layer** (bash + a few Rust string edits), not touch the bus push's
production write path.

Live verification during this research surfaces one important, previously-undocumented
fact: `gh run list -R reubenjohn/reposix-tokenworld-mirror` shows the mirror-sync
workflow's only recorded run **failed 2026-05-01** and has not run since — the
"authoritative" webhook+cron mechanism is not currently converging the mirror at all.
This makes the manual pre-step / self-reconcile path (this phase's actual deliverable)
load-bearing in practice, not merely defense-in-depth.

Root-cause tracing through `crates/reposix-remote/src/{bus_handler,write_loop,precheck}.rs`
establishes exactly which mechanism fires in the DRAIN-02 scenario (a fine-grained,
content-aware per-record conflict check in `precheck.rs::precheck_export_against_changed_set`,
NOT the coarser bus-level PRECHECK A/B), and why the existing bounded self-heal in
`litmus-flow.sh` (lines 89-111) already partially works around it but still fails on
content-level rebase conflicts. The corrected teaching string (SC3) has a **hard
constraint**: at least 12 doc-alignment/regression-test bindings pin the literal
substring `"git pull --rebase"` inside `crates/reposix-remote/src/{write_loop,bus_handler}.rs`
— any fix must ADD guidance, not remove/replace that substring, or it breaks ~12
committed tests.

**Primary recommendation:** Implement SC1+SC2 as one mechanism — fold
`refresh-tokenworld-mirror.sh`'s proven technique (`git fetch reposix main` +
wholesale `pages/` overlay from `FETCH_HEAD`, non-circular re-verify) directly into
`litmus-flow.sh`'s `_litmus_flow()`, running it BEFORE target selection/marker edit
(so the litmus's own edit always bases on a backend-current tree and the git-level
rebase conflict never occurs), plus a pre-flight `confluence_tokenworld.py
restore`/`reparent` sweep over the three known fixture ids for backend drift. Then
separately correct the write_loop.rs mirror-lag-ref hint (SC3) to name the bus remote
explicitly and warn that plain `git pull`/`git rebase` on a Pattern-C `attach` tree
reads the stale origin mirror, not the SoT — while preserving the literal
`"git pull --rebase"` substring for the ~12 pinned tests.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DRAIN-02 | Document a mandatory mirror-refresh pre-step for `pre-release-real-backend` (run `scripts/refresh-tokenworld-mirror.sh` first, OR make the litmus self-reconcile), so a second-run vision-litmus doesn't false-negative on its own prior push re-staling the GitHub mirror. | See "Architecture Patterns → Pattern 1" (root-cause trace of the second-run false-negative) and "Don't Hand-Roll" (why self-reconcile-in-litmus is the sanctioned path, not a pre-step doc alone). `docs/reference/testing-targets.md` currently has ZERO reference to the refresh script (verified by grep) — that is the concrete doc gap SC1 closes at minimum. |
| DRAIN-12 | Make the milestone-close vision-litmus fixture self-healing for BOTH backend drift (trashed protected pages) and GitHub mirror drift (`reposix sync --reconcile` does NOT push to the mirror) — reconcile the mirror to backend-current through the reposix bus remote before the marker push; fix the helper's misleading `git pull --rebase` teaching string for the mirror-drift case. | See "Code Examples" for exact file:line self-heal insertion points in `lib/litmus-flow.sh`, and "Common Pitfalls → Pitfall 3" for the precise mechanism by which `git pull --rebase` misleads a Pattern-C attach tree (reads `origin`=stale mirror by default, not the SoT-backed bus remote). |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

Directives this phase must not contradict (root `CLAUDE.md`, `.planning/CLAUDE.md`,
`crates/CLAUDE.md`, `quality/CLAUDE.md` — all auto-loaded and re-verified against
reality during this research):

- **Leaf isolation.** Any `reposix init`/`attach`/sim-seed/mutating `git commit`/`config`
  invocation MUST `cd` into a throwaway `/tmp` clone in the SAME Bash invocation — never
  the shared repo. `litmus-flow.sh` and `refresh-tokenworld-mirror.sh` already do this
  correctly (`mktemp -d`); any new self-heal code added must preserve the pattern.
- **One cargo invocation machine-wide** — if this phase's implementation needs a
  `cargo build`/`cargo test`, serialize it; prefer `-p reposix-remote` / `-p reposix-cli`
  over `--workspace`.
- **Tainted by default / credential redaction.** Any new stderr/diag string that echoes a
  URL or remote name MUST go through the correct redactor
  (`reposix_core::http::strip_url_userinfo` for a lone well-formed URL,
  `reposix_remote::backend_dispatch::redact_userinfo` for anything else — a
  `reposix::`-prefixed URL, a bus URL with `?mirror=`, or free-form git stderr). See
  `crates/CLAUDE.md` § "Credential redaction before ANY error/stderr echo."
- **Never add a `waiver` block** to `agent-ux/milestone-close-vision-litmus-real-backend`
  (blast_radius P0, `pre-release-real-backend` cadence) — catalog-load-time SystemExit
  enforces this (`quality/runners/_audit_field.py`).
- **Catalog-first rule.** If this phase adds/modifies catalog rows (e.g. wiring the
  pre-step into `pre-release-real-backend`, or a new self-heal-specific row), the FIRST
  commit must write the GREEN-contract row before implementation lands.
- **Error-message 3-part bar** (teach the fix / suggest the alternative / give a
  copy-paste recovery command) applies to any corrected helper teaching string — model
  after `reposix-cli/src/init.rs::refuse_existing_repo_root`.
- **Fix-twice meta-rule.** Any CLAUDE.md/doc text this phase corrects (the mirror-vs-cache
  conflation warning already partially fixed 2026-07-16, the "git pull --rebase" recovery
  examples in `docs/concepts/dvcs-topology.md` and `docs/guides/troubleshooting.md`) must
  be updated in the SAME commit as the code fix, not left to drift.
- **Docs-alignment binding rebind.** Any wording change to a `doc-alignment.json`-bound
  line (see "Common Pitfalls → Pitfall 3" for the ~12 rows pinned to the literal
  `"git pull --rebase"` substring) requires rebinding in the SAME commit — a later
  separate reword re-drifts the binding (P117 W3 precedent).
- **`quality/gates/agent-ux/milestone-close-vision-litmus.sh` is `set -uo pipefail`**
  (no `errexit`) by design — any new self-heal code sourced/added must not assume
  `set -e` semantics; check exit codes explicitly (existing convention throughout
  `litmus-flow.sh`).
- **10k `.sh` file-size budget** (`structure/file-size-limits.sh`, ceiling 10000 bytes for
  `.sh`/`.bash`) — `litmus-flow.sh` is currently well under budget; adding the
  refresh-mirror overlay logic must be checked against this ceiling (see "Environment
  Availability" for current byte count).

## Architectural Responsibility Map

reposix's tiers differ from a generic web-app stack; mapped to the DVCS topology's own
three-role model (`docs/concepts/dvcs-topology.md`) plus the quality-harness layer this
phase actually touches:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Mirror-drift detection before marker edit | Quality/test-harness (`quality/gates/agent-ux/lib/litmus-flow.sh`) | Git Remote Helper (bus push prechecks, read-only consulted) | The fix must live in the TEST SETUP, not the product write path — Option C (product-level post-write fan-out) is explicitly not sanctioned this milestone. |
| Backend-drift detection (trashed fixture) | Quality/test-harness (`litmus-flow.sh` + `scripts/confluence_tokenworld.py`) | External Backend (Confluence TokenWorld, source of truth for `status`/`parentId`) | `confluence_tokenworld.py` already owns REST-level inspect/restore/reparent; the harness orchestrates it, never re-implements REST calls inline. |
| Teaching-string correctness (SC3) | Git Remote Helper (`crates/reposix-remote/src/{write_loop,bus_handler}.rs`) | Docs (`docs/guides/troubleshooting.md`, `docs/concepts/dvcs-topology.md`) | The helper is the single source of the stderr hint; docs mirror it for humans — both must move together (fix-twice). |
| Pre-release-real-backend cadence wiring | Docs (`docs/reference/testing-targets.md`) + Quality runner (`quality/runners/run.py --cadence pre-release-real-backend`) | Quality catalog (`quality/dispatch/milestone-close-verdict.md` probe-9 template) | SC1's "documented mandatory pre-step" is a docs+cadence-invocation concern, not a code concern, UNLESS the self-reconcile path (SC2) makes the pre-step unnecessary. |
| External mirror convergence (steady-state) | External Mirror (webhook + 30-min cron GH Action on `reposix-tokenworld-mirror`) | — | Owner-ruled authoritative mechanism (2026-07-16); this phase does NOT change it, only compensates for its current non-running state within the litmus. |

## Standard Stack

No new libraries or frameworks. This phase edits existing, already-standard project
tooling:

### Core (existing, reused)
| Component | Location | Purpose | Why reused, not rebuilt |
|-----------|---------|---------|--------------------------|
| `scripts/confluence_tokenworld.py` | stdlib-only Python 3 | REST inspect/list/restore/reparent/delete against TokenWorld, refuses protected ids | Already the named recovery tool per root CLAUDE.md; has `restore` (status flip) and `reparent` (parent-link fix) exactly matching the two backend-drift failure shapes. |
| `scripts/refresh-tokenworld-mirror.sh` | bash, `set -euo pipefail` | Wholesale, non-circular mirror-vs-backend reconciliation (fetch backend-materialized tree, `git rm`+`checkout FETCH_HEAD -- pages/`, fast-forward push) | Already builds/verifies the EXACT mechanism SC1/SC2 need; do not re-derive a second implementation — factor/reuse. |
| `crates/reposix-remote` (Rust, `errmsg`/`teach` primitives) | `crates/reposix-core/src/errmsg.rs` | 3-part teaching-error construction | The shared builder every corrected stderr string should route through if converted to a coded error; current sites use ad-hoc `crate::diag(&format!(...))` (see Code Examples). |
| `quality/gates/agent-ux/lib/transcript.sh` | bash | Wraps the litmus flow, emits the RBF-FW-02 transcript artifact | Unmodified by this phase — self-heal steps become new `pass`/`fail` lines inside `_litmus_flow`, not a new wrapper. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Fold refresh logic into `litmus-flow.sh` (SC2, sanctioned) | Rewrite the bus push's mirror fan-out to always push post-write backend-materialized tree (`GTH-V15-38` Option C) | REJECTED for this milestone by 2026-07-16 manager ruling — pull-forward gated on a real incident; out of scope here. |
| Documented manual pre-step (SC1, sanctioned) | Fully automate mirror convergence via a fixed cron (tighten the GH Action interval) | Doesn't fix the CURRENT finding that the workflow isn't running at all (last run failed 2026-05-01, zero runs since) — a tighter cron on a broken/unscheduled workflow changes nothing; also out of this phase's scope (external mirror repo's own `.github/workflows/`, not this repo). |

**Installation:** N/A — no new dependencies. `git --version` (2.50.1) and
`cargo --version` (1.96.1) already exceed the project's stated minimums (git 2.34+,
`rust-toolchain.toml`'s 1.82+).

**Version verification:** N/A — no package versions to pin.

## Architecture Patterns

### System flow — why the second-run false-negative happens (DRAIN-02 root cause)

```
Run 1 (litmus succeeds):
  clone mirror (v_N) --attach--> cache synced to backend v_N --edit page--> commit
    --push via bus "reposix" remote-->
       PRECHECK A (mirror ls-remote) STABLE (nothing else touched mirror)
       PRECHECK B (list_changed_since) STABLE (nothing changed since attach)
       apply_writes -> precheck_export_against_changed_set: local base v_N == backend v_N -> Proceed
       SoT write succeeds -> backend now v_(N+1) [Confluence re-normalizes storage body server-side]
       mirror fan-out: `git push <mirror_remote> main` pushes the CLIENT'S PRE-WRITE tree
         (base v_N body + appended marker) -- NOT the v_(N+1) backend-materialized body
    => mirror `pages/<id>.md` now carries a v_N-based, non-server-normalized body,
       while the backend is truly at v_(N+1) with re-normalized storage format.
       MIRROR IS NOW STALE RELATIVE TO BACKEND (self-inflicted by run 1's own push).

Run 2 (litmus, same day):
  clone mirror (STILL v_N-based / stale) --attach--> cache synced FRESH to backend v_(N+1)
    --edit SAME page (append a NEW marker onto the v_N-based body)--> commit
    --push via bus "reposix" remote-->
       PRECHECK A STABLE (mirror hasn't moved since our own clone -- no 3rd party raced us)
       PRECHECK B STABLE (nothing changed backend-side since attach moments ago)
       apply_writes -> precheck_export_against_changed_set:
           local base version (from the just-committed file's frontmatter, v_N)
             != backend_now.version (v_(N+1))
           stale_base = true
           content_diverges() -> TRUE (server-normalized v_(N+1) body differs from the
             locally-held v_N-based body beyond the appended marker)
         => CONFLICT -> reject "error refs/heads/main fetch first"
       write_loop.rs:198 RPX-0505 diag: "Run: git pull --rebase ..."
       write_loop.rs:216-219 mirror-lag hint (fires because refs/mirrors/<sot>-synced-at
         IS populated from run 1): "hint: your origin (GH mirror) was last synced ...
         hint: run `reposix sync` to update local cache ... then `git rebase`"
    litmus-flow.sh's bounded self-heal (lines 89-111) retries:
       `git pull --rebase reposix main` (correctly remote-explicit, NOT origin)
         -> fetches backend-current v_(N+1) tree via the bus/SoT tunnel (this part works)
         -> attempts to REPLAY the local commit (diff of v_N-body+marker against v_N parent)
            onto the v_(N+1) base
         -> REBASE CONFLICTS at the git level: the context lines around the marker
            differ too much between v_N and v_(N+1) (server-side storage-format
            re-normalization), so git cannot cleanly apply the patch
       `git pull --rebase reposix main` EXITS NON-ZERO (conflicted, needs manual resolve)
       `&&` short-circuits -> retry push never attempted
    => litmus hard-FAILs: "a rejection persisting past the bounded self-heal is a REAL
       coherence bug, not mirror lag" -- FALSE, this IS mirror lag (self-inflicted),
       not a coherence bug. FALSE NEGATIVE.
```

[VERIFIED: crates/reposix-remote/src/precheck.rs (read lines 90-290), write_loop.rs
(lines 160-219), bus_handler.rs (lines 1-330), scripts/refresh-tokenworld-mirror.sh
header comment, lib/litmus-flow.sh lines 18-149, .planning/milestones/v0.15.0-phases/good-to-haves/part-01.md:46-51,
.planning/milestones/v0.15.0-phases/surprises-intake/part-02.md:5-17]

**Key insight the planner needs:** the failure is NOT in bus_handler.rs's PRECHECK A
(mirror `git ls-remote` drift — a third-party-race guard that correctly stays STABLE
here) and NOT in bus_handler.rs's PRECHECK B (`precheck_sot_drift_any` — a coarse
"did anything change since last_fetched_at" gate that also stays STABLE, since nothing
touched the backend between `attach` and `push`). It fires in the FINER, per-record,
content-aware check in `precheck.rs::precheck_export_against_changed_set` (called from
`write_loop.rs::apply_writes`), which is deliberately content-aware precisely to avoid
false-rejecting no-op pulls (see precheck.rs:150-278's own comment on "stale base version
ALONE is not a conflict"). That same content-awareness is what makes THIS scenario a
genuine git-level rebase conflict rather than a clean auto-heal.

### Pattern 1: Reconcile-before-edit (the SC2 self-heal shape)

**What:** Instead of reactively retrying after a push rejection (current bounded
self-heal, which only fixes clean fast-forward cases), fetch the backend-materialized
tree and overlay it onto the WORKING TREE **before** picking a target and making the
marker edit — the same technique `refresh-tokenworld-mirror.sh` already proves works
(non-circular verify: re-clone + compare against a pristine `FETCH_HEAD` extract).

**When to use:** Any test harness that clones a plain-git mirror of an SoT-backed
project and then edits+pushes through the SoT — i.e., exactly the litmus's Pattern-C
round-tripper shape.

**Example (conceptual insertion point — see Code Examples for exact target):**
```bash
# Source: adapted from scripts/refresh-tokenworld-mirror.sh's proven mechanism
# Insert into lib/litmus-flow.sh, AFTER the `reposix attach` config-check (line ~39)
# and BEFORE the GUARD A reconciliation-safety check (line ~41), so the pages/
# directory is backend-current before GUARD A even evaluates matched/backend_deleted.
git -C "$tree" fetch --quiet reposix main
git -C "$tree" rm -r --quiet --ignore-unmatch -- pages/ >/dev/null 2>&1 || true
git -C "$tree" checkout FETCH_HEAD -- pages/
git -C "$tree" add -A pages/
if ! git -C "$tree" diff --cached --quiet; then
  git -C "$tree" commit --quiet -m "litmus pre-reconcile: sync mirror clone to backend-current"
fi
```

**Why before, not after:** overlaying AFTER a rejection (the current behavior) has
already committed the marker edit on top of the stale base, so the rebase must
reconcile a DIVERGED commit. Overlaying BEFORE means the marker edit is authored on
top of an already-backend-current tree — the push then has no stale base at all.

### Pattern 2: Idempotent fixture pre-flight (the backend-drift half of SC2)

**What:** Before target selection, `inspect` each of the three fixture ids TokenWorld
depends on (`2818063` sacrificial-editable, `7766017`/`7798785` protected pair) and
`restore`+`reparent` any found trashed/orphaned — using the EXISTING
`scripts/confluence_tokenworld.py` tool (already idempotent: `cmd_restore` no-ops if
`status == "current"`).

**When to use:** As the very first step of `_litmus_flow`, before the mirror `git
clone` even — a trashed fixture affects `reposix attach`'s reconciliation walk
(GUARD A's `backend_deleted` count) regardless of mirror state.

**Example:**
```bash
# Source: scripts/confluence_tokenworld.py's existing restore/reparent commands
# (root CLAUDE.md § "Commands you'll actually use" already names this as the
# recovery tool for "a trashed page whose parentId went null")
for pid in 2818063 7766017 7798785; do
  python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" restore "$pid" >&2 || true
done
# 7798785 is the CHILD in the durable parent/child pair — its parentId can go null
# even after a status restore (Confluence does not restore the parent link).
python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" inspect 7798785 | \
  grep -q '"parentId": null' && \
  python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" reparent 7798785 7766017
```

### Anti-Patterns to Avoid

- **Do NOT rewrite the bus push's mirror fan-out mechanism** (changing
  `bus_handler.rs`'s `push_mirror` to push a post-write/backend-materialized tree
  instead of the client's pre-write tree). This is exactly `GTH-V15-38` Option C,
  explicitly NOT sanctioned this milestone by the 2026-07-16 manager ruling. If the
  planner is tempted toward a "fix it at the source" product-code change, stop —
  that decision has already been made and reserved for a future pull-forward trigger.
- **Do NOT rely on `reposix sync --reconcile` to fix mirror drift.** It rebuilds ONLY
  the local reposix cache; it has zero effect on the external GitHub mirror repository
  (proven empirically in the v0.14.0 B1 evidence: "mirror HEAD `3be8390` byte-identical
  before→after reconcile" — `.planning/milestones/v0.15.0-phases/good-to-haves/part-01.md:50`).
  Any teaching-string fix that recommends `reposix sync --reconcile` as a mirror-drift
  remedy is WRONG and must not be introduced (see also the code/doc drift noted in
  Common Pitfalls — the CURRENT code already omits `--reconcile`, which is a distinct
  but related bug).
- **Do NOT remove or replace the literal substring `"git pull --rebase"`** from
  `write_loop.rs` / `bus_handler.rs`. See Common Pitfalls → Pitfall 3 for the ~12
  regression tests / doc-alignment rows this would break.
- **Do NOT loop the bounded self-heal retry.** `litmus-flow.sh`'s existing comment is
  explicit: "BOUNDED: exactly one fetch-rebase-retry; a rejection persisting past it is
  a REAL coherence bug ... we never loop or swallow it." Any Pattern-1 fix should make
  the retry unnecessary in the common case, not add a second/looping retry layer.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fetching the backend-materialized tree and overlaying it non-circularly | A new script or inline logic that re-derives the fetch+overlay+verify sequence | `scripts/refresh-tokenworld-mirror.sh`'s proven mechanism (already builds a `BACKEND_REF` pristine extract, does `git rm` + `checkout FETCH_HEAD --` for correct deletion propagation, verifies via independent re-clone) | This exact mechanism is already written, commented in detail, and has passed the `orphan-scripts-audit` structural check. Re-deriving it inside `litmus-flow.sh` risks a SECOND, subtly different implementation (e.g. forgetting the `git rm` step, which would silently retain a backend-deleted page — a documented pitfall in the script's own header). |
| Trashed-fixture detection/restore | Inline `curl`/REST calls against Confluence's page API | `scripts/confluence_tokenworld.py {inspect,restore,reparent}` | Already handles the `status` v2-vs-v1-API quirk (`cmd_restore` uses the v1 `?status=trashed` PUT endpoint because v2 DELETE only trashes, doesn't offer a v2 restore), the protected-id refusal, and is the ALREADY-named recovery tool in root CLAUDE.md. |
| Real-backend mirror-drift end-to-end proof | A synthetic/mocked git-remote-helper test simulating mirror staleness | The real `pre-release-real-backend` cadence (`bash quality/gates/agent-ux/milestone-close-vision-litmus.sh` against TokenWorld) | The precise git-level rebase-conflict mechanism (server-side storage re-normalization creating divergent context lines) is a property of the REAL Confluence backend's storage format; a mock cannot faithfully reproduce it. Unit-testable pieces (teaching-string substring presence, remote-name resolution logic) already have Rust regression tests (`bus_precheck_a.rs`, `bus_precheck_b.rs`, `push_conflict.rs`); the end-to-end self-heal proof stays real-backend-only by design (NOT-VERIFIED at pre-push, matching the P124 `example-05-blob-limit-recovery` precedent). |

**Key insight:** every primitive this phase needs (fetch+overlay+verify, REST
inspect/restore/reparent, bounded-retry-on-conflict) already exists as a proven, tested
building block somewhere in the repo. The work is **composition inside
`lib/litmus-flow.sh`** (ordering: fixture pre-flight → mirror pre-reconcile → target
selection → edit → push → existing bounded self-heal as a backstop) plus a handful of
**Rust string edits**, not new mechanism design.

## Common Pitfalls

### Pitfall 1: Conflating "PRECHECK A/B" with the actual failing mechanism
**What goes wrong:** Assuming the DRAIN-02 false-negative is caught by bus_handler.rs's
PRECHECK A (mirror `git ls-remote` drift) or PRECHECK B (`precheck_sot_drift_any`, coarse
SoT-changed-at-all gate), and fixing/tracing the WRONG teaching string.
**Why it happens:** The catalog row comment for `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`
and the phrase "PRECHECK B" both sound like the obvious candidates, and the
`additional_context` breadcrumb even frames it that way.
**How to avoid:** Trace the actual scenario (both PRECHECK A and B are STABLE in the
DRAIN-02 replay — see "Architecture Patterns" flow above). The real reject path is
`precheck.rs::precheck_export_against_changed_set` → `write_loop.rs:189-219`'s
RPX-0505 diag + mirror-lag-ref hint. Fix THOSE two teaching strings; bus_handler.rs's
two PRECHECK hints (lines 197-199, 253) are correct as-is for THEIR scenarios (a
literal third-party mirror race, and a coarse SoT-changed gate) and do not need
DRAIN-12's correction — though the planner should verify this against a live run if
budget allows, since this analysis is static-trace-derived (HIGH confidence from
reading the code paths, not from an executed reproduction — real Confluence creds
would be needed to reproduce live).
**Warning signs:** If a fix only touches `bus_handler.rs`, it likely missed the actual
firing site.

### Pitfall 2: Believing `reposix sync` (bare) does anything
**What goes wrong:** `write_loop.rs:216-218`'s existing mirror-lag hint reads: `"hint:
run \`reposix sync\` to update local cache from {backend_name} directly, then \`git
rebase\`"` — this is missing the `--reconcile` flag. Per `sync.rs`'s own doc comment,
bare `reposix sync` (no `--reconcile`) "prints a hint pointing at `--reconcile`; exits
0" and does **nothing else**. The documented example in
`docs/concepts/dvcs-topology.md` (line 90) correctly shows `reposix sync --reconcile`
— the ACTUAL CODE has drifted from the documented example.
**Why it happens:** Likely an accidental omission when the hint was last edited; not
malicious, just a real code/doc drift caught by this research.
**How to avoid:** Any SC3 fix touching this hint must add `--reconcile`. This is a
small, concrete, high-confidence bug fix independent of the larger mirror-drift
redesign — the planner can treat it as a one-line correction.
**Warning signs:** grep `write_loop.rs` for `"reposix sync\`"` without `--reconcile`.
[VERIFIED: crates/reposix-remote/src/write_loop.rs:216-218 vs crates/reposix-cli/src/sync.rs:71-74 vs docs/concepts/dvcs-topology.md:90]

### Pitfall 3: Breaking ~12 pinned regression tests by touching "git pull --rebase"
**What goes wrong:** A well-intentioned SC3 fix that REPLACES the phrase "git pull
--rebase" (e.g., with a remote-explicit form that doesn't contain the exact substring,
or removes it in favor of a different verb sequence) silently breaks a large set of
already-committed tests and doc-alignment bindings that grep the SOURCE FILES for the
literal substring.
**Why it happens:** The substring is pinned in at least these committed assertions
(non-exhaustive; found via `grep -n "git pull --rebase" doc-alignment.json` and the
Rust test tree):
- `crates/reposix-remote/tests/bus_precheck_b.rs:191` — `stderr.contains("git pull --rebase")`
- `crates/reposix-remote/tests/push_conflict.rs:198` — `stderr.contains("git pull --rebase")`
- `crates/reposix-cli/tests/agent_flow.rs:188` — `src.contains("git pull --rebase")` (scans SOURCE, not runtime stderr)
- `quality/catalogs/doc-alignment.json` rows (≥12 found): `dark_factory_conflict_teaching_string_present`-backed claims across README.md, `docs/reference/exit-codes.md`, and multiple `agent_flow*.rs`/`dark_factory*` claims, all asserting the literal substring is present in one of `crates/reposix-remote/src/{main,write_loop,bus_handler}.rs`.
**How to avoid:** ADD new guidance (a NEW hint line, or appended clarifying text) rather
than replacing the existing phrase. E.g., keep `"Run: git pull --rebase ..."` verbatim
and ADD a follow-on diag line specifically for the mirror-lag-ref-populated branch
(write_loop.rs:210-219) that names the risk: something like `"hint: if this tree was
created via \`reposix attach\`, a bare \`git pull\`/\`git rebase\` reads the ORIGIN
MIRROR by default (which may itself be stale) — rebase against the SoT-backed remote
explicitly, e.g. \`git pull --rebase <reposix-remote-name> main\`"`.
**Warning signs:** `cargo test -p reposix-remote` or `-p reposix-cli` failing on a
teaching-string assertion after the edit; `quality/gates/docs-alignment/walk.sh` BLOCK.
[VERIFIED: grep across crates/reposix-remote/tests/, crates/reposix-cli/tests/, quality/catalogs/doc-alignment.json]

### Pitfall 4: Assuming `git pull --rebase` (bare) reads the SoT for a Pattern-C tree
**What goes wrong:** The documented recovery in `docs/guides/troubleshooting.md` and
`docs/concepts/dvcs-topology.md` shows bare `git pull --rebase` (no remote argument) as
"the" recovery. For a Pattern-B (`reposix init` direct) tree this is correct — `origin`
IS the SoT-backed remote. For a **Pattern-C** (`reposix attach`) tree, `dvcs-topology.md`
itself already documents: "Fetch is untouched: `git fetch` / `git pull` keep reading
from the mirror" (line 152) — meaning bare `git pull --rebase` on an attach tree reads
the plain-git MIRROR (`origin`), completely bypassing reposix's cache/backend
reconciliation logic. If the mirror is stale (the DRAIN-02 scenario, or simply within
the webhook/cron's up-to-30-minute lag window), bare `git pull --rebase` is a
near-no-op and the retried push fails identically.
**Why it happens:** The generic teaching string was written assuming Pattern B/the
common case; it doesn't distinguish Pattern C's dual-remote shape.
**How to avoid:** This is the CORE of DRAIN-12's "helper's misleading git pull --rebase
teaching string for the mirror-drift case" — the fix should make the hint
remote-explicit (name the bus remote) specifically in the mirror-lag-ref-populated
branch (`write_loop.rs:210-219`), which is exactly the branch that ONLY fires for a
tree that has already completed at least one mirror sync — i.e., exactly the Pattern-C
attach shape this pitfall describes. `litmus-flow.sh`'s own bounded self-heal already
gets this right (`git pull --rebase reposix main`, remote-explicit) — its own comment
even says why: "NEVER `origin` (the stale mirror; after attach `branch.<b>.remote`
still points at origin for fetch, so the recovery is remote-explicit on purpose)."
**Warning signs:** A user/agent on a Pattern-C tree reports the SAME conflict
recurring after following the documented `git pull --rebase` recovery verbatim.
[VERIFIED: docs/concepts/dvcs-topology.md:152, quality/gates/agent-ux/lib/litmus-flow.sh:94-99]

### Pitfall 5: Assuming the webhook+cron mirror-sync workflow is currently converging
**What goes wrong:** Treating the 2026-07-16 manager ruling's "webhook + 30-min cron
[is] AUTHORITATIVE" as evidence the mirror self-heals over time, and therefore treating
SC1's pre-step as low-priority/belt-and-suspenders.
**Why it happens:** The ruling is real and current, but a LIVE check
(`gh run list -R reubenjohn/reposix-tokenworld-mirror`) during this research shows the
workflow's only recorded run **failed on 2026-05-01** (12s runtime) and there are
**zero runs since** — the cron is not ticking (or ticking and being filtered/disabled)
and the webhook has not fired successfully. `refresh-tokenworld-mirror.sh`'s own header
independently calls this "the KNOWN-BROKEN webhook mirror-sync Action."
**How to avoid:** Do not treat SC1/SC2 as optional hardening. Treat the manual
pre-step / self-reconcile as load-bearing until/unless the mirror-sync workflow itself
is separately repaired (that repair — living in the SEPARATE
`reposix-tokenworld-mirror` repo, not this one — is out of scope for this phase).
**Warning signs:** None from this repo alone — this is exactly why it was invisible
until checked live.
[VERIFIED: `gh run list -R reubenjohn/reposix-tokenworld-mirror -L 5` — single row,
status "completed", conclusion "failure", 2026-05-01T16:42:51Z, run 25223195636]

### Pitfall 6: Ambiguity in SC2's "trashed protected pages" wording
**What goes wrong:** SC2 says "backend drift (trashed protected pages)" but the
litmus's target-selection loop explicitly SKIPS the truly-protected pair
(`7766017`/`7798785`) and only ever edits a non-protected page — currently `2818063`
(the documented "sacrificial editable page"). `GTH-V15-09`'s own worked failure example
used `2818063` trashed, not the protected pair.
**Why it happens:** Loose terminology — `2818063` is also effectively "protected from
going missing" in practice (testing-targets.md documents it as fixture-critical) even
though it is NOT in the `PROTECTED_IDS` denylist (which exists to prevent DELETION, not
to guarantee presence/current-status).
**How to avoid:** Flagged in Assumptions Log below — treat all three known fixture ids
(`2818063`, `7766017`, `7798785`) as in-scope for the pre-flight restore/reparent sweep
(Pattern 2 above); this is the safe superset interpretation and costs nothing extra
(the restore call is idempotent).
**Warning signs:** A plan that pre-flights ONLY the protected pair and leaves 2818063
unchecked would still be vulnerable to GTH-V15-09's ORIGINAL documented incident.

## Code Examples

### Exact file:line targets for SC3 (teaching-string correction)

```
crates/reposix-remote/src/write_loop.rs
  :189-200   RPX-0505 diag — generic per-record conflict hint.
             KEEP the literal "Run: git pull --rebase" substring (pinned by tests).
  :201-219   mirror-lag-ref-populated hint (only fires when
             refs/mirrors/<sot>-synced-at exists, i.e. post-first-sync — exactly the
             DRAIN-02 second-run shape). Two bugs to fix here:
               (a) line 217: "reposix sync" is MISSING "--reconcile" (Pitfall 2)
               (b) the hint does not warn that bare `git rebase` on this branch reads
                   a possibly-stale LOCAL tracking ref, not backend-current (Pitfall 4)

crates/reposix-remote/src/bus_handler.rs
  :192-199   PRECHECK A hint ("git fetch {mirror_remote_name}") — third-party mirror
             race guard; NOT the DRAIN-02 firing site per this research's trace, but
             worth a light pass for consistency if the planner's implementation
             threads a shared helper function through both sites.
  :237-253   PRECHECK B hint ("git pull --rebase" for coarse SoT-changed gate) — also
             not the DRAIN-02 firing site; verify against a live run before excluding
             it from scope, since this is a static trace (see Pitfall 1).
```

### Exact insertion point for SC2 (litmus self-heal)

```
quality/gates/agent-ux/lib/litmus-flow.sh
  _litmus_flow() body:
    line ~18-24  (setup: run dir, cache dir, tree dir) — add fixture pre-flight
                 (Pattern 2) as the FIRST executed step, before the `git clone`.
    line ~26-28  `git clone --quiet "$MIRROR_URL" "$tree"`
    line ~30-39  `reposix attach` + config-check
    <<< INSERT Pattern 1 (mirror pre-reconcile: fetch reposix + overlay pages/) HERE >>>
    line ~41-51  GUARD A (reconciliation safety) — now evaluates against a
                 backend-current pages/ tree, so `backend_deleted` reflects reality.
    line ~53-62  GUARD B (bucket-shape check)
    line ~64-73  target selection + marker edit
    line ~89-111 existing bounded self-heal retry — KEEP as a backstop (should now be
                 unreachable in the common DRAIN-02 case, but still correct for a
                 genuine peer-write race during the litmus's own run).
```

### Existing proven mechanism to adapt (reference, not to be duplicated verbatim)

```bash
# Source: scripts/refresh-tokenworld-mirror.sh:111-134 (the exact fetch+overlay+
# protected-fixture-guard sequence — the "THE MECHANISM" section of its header
# comment explains the git rm + checkout FETCH_HEAD ordering rationale)
"${BIN}" attach "confluence::${SPACE}" "${TREE}" --remote-name reposix --mirror-name origin
cd "${TREE}"
git fetch --quiet reposix main
BACKEND_REF="${RUN}/backend-ref"; mkdir -p "${BACKEND_REF}"
git archive FETCH_HEAD pages/ | tar -x -f - -C "${BACKEND_REF}"
git rm -r --quiet --ignore-unmatch -- pages/ > /dev/null 2>&1 || true
git checkout FETCH_HEAD -- pages/
for pid in 7766017 7798785; do
  [ -e "pages/${pid}.md" ] || { echo "ERROR: backend tree missing protected fixture ${pid}" >&2; exit 1; }
done
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|---------------|--------|
| Litmus relies on `origin` mirror always being fresh | Litmus must self-verify/self-reconcile mirror freshness before editing (this phase) | This phase (P125) | Removes the "second-run false-negative" failure class; makes `pre-release-real-backend` cadence reliable across repeated milestone-close attempts in the same day. |
| "reposix sync --reconcile fixes mirror drift" (informally believed pre-2026-07-16) | Explicitly corrected: `sync --reconcile` heals ONLY the local cache; webhook+cron (or the manual pre-step) heals the external mirror | 2026-07-16 manager ruling (`.planning/CONSULT-DECISIONS.md:113-136`) + this research | Root CLAUDE.md's "Mirror-head refresh promise" section already carries the corrected framing; `docs/concepts/dvcs-topology.md`'s L1 bullet (lines 182-190) already carries the corrected framing too — this phase should NOT need to re-litigate that doc fix, only build on it. |

**Deprecated/outdated:**
- The bare `"hint: run \`reposix sync\` ... then \`git rebase\`"` string
  (write_loop.rs:216-218) is a stale/buggy variant of the CORRECT documented example
  (`reposix sync --reconcile` — dvcs-topology.md:90); treat the code as the thing that
  drifted, not the doc.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The DRAIN-02 second-run false-negative fires through `precheck.rs::precheck_export_against_changed_set` (fine-grained, content-aware) rather than bus_handler.rs's PRECHECK A or PRECHECK B. This is a STATIC trace through the Rust source, not an executed live reproduction against real TokenWorld+GH mirror. | Architecture Patterns, Pitfall 1 | If wrong, the planner would need to correct bus_handler.rs's PRECHECK A/B hints instead of (or in addition to) write_loop.rs's — a live reproduction against real creds would definitively confirm which reject path fires. Recommend the planner's first implementation task include an early live-run checkpoint that captures the ACTUAL stderr output of a reproduced second-run failure, before finalizing which string(s) to edit. |
| A2 | SC2's "trashed protected pages" should be interpreted to include the sacrificial editable page `2818063`, not only the literal `PROTECTED_IDS` pair (`7766017`/`7798785`). | Pitfall 6, Pattern 2 | If the user/owner intended SC2 literally (protected pair only), the pre-flight sweep is a harmless superset (idempotent restore calls cost nothing) — low risk either way. |
| A3 | The corrected teaching string can safely thread the local bus-remote name into the hint at the `write_loop.rs:210-219` call site. This was NOT verified against the actual call signature of `apply_writes` (which is shared between single-backend and bus paths and, per its own doc header, does not currently receive a remote-name parameter). | Common Pitfalls → Pitfall 4, Code Examples | If `apply_writes` genuinely has no access to the local remote name at that call site, the fix may need to either (a) thread a new `Option<&str>` parameter through from both callers, or (b) use generic phrasing ("the SoT-backed remote you configured via `reposix attach`, not `origin`") instead of naming it explicitly. This is a real implementation-design decision for the planner, not fully resolved by this research. |
| A4 | This phase's scope excludes touching `.github/workflows/` in the SEPARATE `reposix-tokenworld-mirror` mirror repository (i.e., does not attempt to fix the webhook+cron workflow itself). | Pitfall 5, Standard Stack | If the phase's actual intent includes repairing the mirror-sync workflow, that is a distinct, out-of-this-repo change requiring `gh` access to `reubenjohn/reposix-tokenworld-mirror` and is not scoped by DRAIN-02/DRAIN-12's text (which is about the LITMUS and the HELPER, not the external Action). Recommend confirming scope explicitly with the user/owner if the planner is tempted to touch it. |

## Open Questions (RESOLVED)

1. **Does `apply_writes` have (or need) access to the local bus-remote name for the
   corrected mirror-lag hint?**
   - **RESOLVED (Plan 125-01 Task 2):** confirmed no remote name is threaded into
     `apply_writes` at that call site; Plan 01 chose the safe default — a generic
     `<reposix-remote-name>` placeholder in the corrected hint rather than a dynamic
     interpolation, sidestepping both the plumbing question and the credential-leak
     redaction requirement that a dynamic remote/URL string would otherwise trigger.
   - What we know: `bus_handler.rs` resolves `mirror_remote_name` at STEP 0 (the
     MIRROR's local name, e.g. `origin`), but the BUS remote's own local name (the name
     the user gave the `reposix::...` URL, e.g. `reposix`) is not obviously threaded
     into `apply_writes`'s signature per the code read during this research.
   - What's unclear: whether git invokes the remote helper with enough context to
     recover the bus remote's local name at that call site (`main.rs`'s dispatch may
     already have it — not read in this research pass).
   - Recommendation: planner's first task should grep `main.rs` for how
     `handle_export`/`handle_bus_export` are invoked and whether a remote name is
     available at the entry point, before committing to a specific hint-wording design.

2. **Should the pre-step be wired into `docs/reference/testing-targets.md`
   independently of the self-heal (belt-and-suspenders), or does a fully
   self-reconciling litmus make the doc pre-step redundant?**
   - **RESOLVED (Plan 125-03):** documented regardless, per this section's own
     recommendation below — Plan 03 wires `refresh-tokenworld-mirror.sh` into
     `testing-targets.md` as the documented pre-step (SC1) AND cross-references the
     Plan 02 self-heal as the "you shouldn't usually need this" escape-hatch caveat.
   - What we know: SC1 explicitly offers an either/or ("a documented mandatory
     mirror-refresh pre-step ... OR a self-reconciling litmus"); SC2 mandates the
     self-heal unconditionally. The 2026-07-16 manager ruling ALSO already blesses
     `refresh-tokenworld-mirror.sh` as "the documented litmus pre-step" independently
     of any self-heal work.
   - What's unclear: whether documenting the pre-step in `testing-targets.md` becomes
     pure redundancy once SC2 ships (since a self-reconciling litmus no longer NEEDS
     the pre-step to avoid false-negatives), or whether it should stay documented as a
     faster manual escape hatch for a human operator who wants to pre-warm/verify state
     without running the full litmus.
   - Recommendation: cheap either way (a short doc section costs little); recommend
     documenting it regardless, cross-referencing the self-heal as the "you shouldn't
     usually need this" caveat — mirrors how `reposix sync --reconcile` is documented
     as "safe to run any time" even though most flows shouldn't need it.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| git | all litmus/refresh-script git operations | ✓ | 2.50.1 | — |
| cargo/rustc | building `reposix`/`git-remote-reposix` binaries for the litmus | ✓ | 1.96.1 | — |
| `target/debug/reposix` binary | litmus's `reposix attach` step | ✓ (pre-built) | — | rebuild via `cargo build -p reposix-cli -p reposix-remote` |
| `.env` file | Confluence/GitHub creds for real-backend verification | ✓ (file present; contents not read by this research) | — | — |
| `gh` CLI | checking mirror-sync workflow run history (used live during this research) | ✓ | — | — |
| Real Confluence TokenWorld reachability | actually EXECUTING SC1-SC3's verification (`pre-release-real-backend` cadence) | NOT VERIFIED live in this research (no REST call made) | — | `bash scripts/preflight-real-backends.sh` before implementation work begins |
| `reposix-tokenworld-mirror` GH Action | steady-state mirror convergence (NOT this phase's target) | ✗ (last run FAILED 2026-05-01, zero runs since) | — | This phase's own deliverable (SC1/SC2) is the fallback |

**Missing dependencies with no fallback:** none — the phase's own deliverable IS the
fallback for the one broken dependency (the mirror-sync GH Action).

**Missing dependencies with fallback:** the external mirror-sync workflow is
effectively non-functional today; this phase's self-heal (SC2) and/or documented
pre-step (SC1) is the intended compensating control, not a blocker to implementing it.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` / `cargo nextest run`, per-crate (`reposix-remote`, `reposix-cli`) |
| Framework (bash gates) | Hand-rolled `pass`/`fail` shell harness + `quality/gates/agent-ux/lib/transcript.sh` wrapper; no bats/shunit2 in this repo |
| Config file | none dedicated — `Cargo.toml` per crate; shell gates are catalog-driven (`quality/catalogs/agent-ux.json`) |
| Quick run command | `cargo test -p reposix-remote` (covers `bus_precheck_a.rs`, `bus_precheck_b.rs`, `push_conflict.rs` — all bin-target-adjacent, run by the BARE per-crate invocation, not a `--test`-scoped one per `crates/CLAUDE.md`'s documented bin-vs-integration-target seam) |
| Full suite command | `python3 quality/runners/run.py --cadence pre-release-real-backend` (env-gated; needs real creds) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DRAIN-02 | `pre-release-real-backend` cadence documents/enforces a mirror-refresh pre-step OR the litmus self-reconciles, closing the second-run false-negative | shell-subprocess (real backend) | `bash quality/gates/agent-ux/milestone-close-vision-litmus.sh` (run TWICE back-to-back, second run must PASS not FAIL) | ✅ (script exists; the "run twice" proof procedure is new — Wave 0 gap) |
| DRAIN-02 | Doc references the pre-step (if SC1 goes the doc route) | docs-alignment / manual review | `bash quality/gates/docs-alignment/walk.sh` (verifies any new binding); manual read of `docs/reference/testing-targets.md` | ❌ Wave 0 — no existing binding to grep for this specific claim |
| DRAIN-12 | Litmus self-heals trashed fixture (backend drift) | shell-subprocess (real backend) | Deliberately trash `2818063` via `confluence_tokenworld.py delete`-equivalent-status-flip (there is no direct "trash" command in the tool — would need a raw DELETE, or wait for/simulate an out-of-band trash), then run the litmus and confirm PASS | ❌ Wave 0 — no scripted way to INDUCE the trashed state for a repeatable test; likely needs manual/one-off verification rather than an automated regression test |
| DRAIN-12 | Litmus self-heals mirror drift | shell-subprocess (real backend) | Run litmus twice back-to-back (same proof as DRAIN-02's second-run case) | ✅ once the self-heal lands — same "run twice" procedure |
| DRAIN-12 | Corrected `git pull --rebase` teaching string still present + augmented | unit (Rust) | `cargo test -p reposix-remote` (bus_precheck_b, push_conflict tests) + a NEW test asserting the augmented mirror-lag hint text | ❌ Wave 0 — no existing test asserts the AUGMENTED hint content (only the base substring) |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-remote` (fast, no real backend) for any
  Rust teaching-string edit; `bash -n quality/gates/agent-ux/lib/litmus-flow.sh` (syntax
  check) for any bash edit — a full litmus run needs real creds and is NOT a per-commit
  gate.
- **Per wave merge:** `bash scripts/preflight-real-backends.sh` (read-only reachability
  check, safe/cheap) before attempting a real litmus run; a real litmus run itself
  ("run twice back-to-back") only at a wave boundary where real-backend budget is
  available.
- **Phase gate:** `python3 quality/runners/run.py --cadence pre-release-real-backend`
  exit 0, run TWICE in immediate succession (the second run is the actual DRAIN-02
  regression proof) before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] No scripted way to reliably INDUCE a "trashed fixture" state for repeatable
  backend-drift testing — DRAIN-12's backend-drift half may need to stay a
  manual/documented verification step rather than a fully automated regression test.
  Flag this explicitly rather than pretending a synthetic test exists.
- [ ] No existing Rust test asserts the CONTENT of the augmented mirror-lag hint (only
  the base substring) — add one alongside the code change (`bus_precheck_b.rs` or a new
  sibling test file), following the existing `stderr.contains(...)` pattern.
- [ ] No "run the litmus twice and assert the second run PASSES" proof procedure exists
  yet — this is the core DRAIN-02 regression test and should be the first Wave 0 item
  the planner schedules, likely as a documented manual verification step for the
  verifier subagent to execute against real TokenWorld (given the cost/complexity of
  scripting a fully automated "run twice" gate on top of the existing `shell-subprocess`
  kind).

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | This phase reuses existing Confluence Basic-auth (`ATLASSIAN_EMAIL`/`ATLASSIAN_API_KEY`) via already-audited `confluence_tokenworld.py`; no new auth surface. |
| V3 Session Management | no | N/A — stateless REST calls, no sessions. |
| V4 Access Control | no | N/A — no new roles/permissions; `confluence_tokenworld.py`'s protected-id refusal is the only access-control-shaped logic touched, and it is unmodified by this phase (reused, not edited). |
| V5 Input Validation | minimal | Fixture ids (`2818063`, `7766017`, `7798785`) are hardcoded/static, not user input — low risk. If the corrected teaching string interpolates a git remote name, it must be treated as untrusted per the threat model (see below). |
| V6 Cryptography | no | N/A — no new crypto. |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Credential leak via an echoed URL/remote-name in a corrected teaching string | Information Disclosure | Route any new interpolated URL/remote string through `reposix_core::http::strip_url_userinfo` (lone well-formed URL) or `reposix_remote::backend_dispatch::redact_userinfo` (anything else, e.g. free-form git stderr or a `reposix::...?mirror=` URL) per `crates/CLAUDE.md`'s documented two-redactor convention. This is the single highest-relevance security concern for this phase's code change, since it is EXACTLY the kind of small stderr-string edit the credential-leak convention exists to guard. |
| Tainted remote byte reaching an outbound side-effect | Tampering / Elevation of Privilege | Not newly introduced — `confluence_tokenworld.py` and `refresh-tokenworld-mirror.sh` already gate all REST/git egress behind `REPOSIX_ALLOWED_ORIGINS` / explicit tenant env vars; this phase composes existing calls, does not add new egress paths. |

## Sources

### Primary (HIGH confidence — read directly, this session)
- `crates/reposix-remote/src/bus_handler.rs` (full module doc + lines 1-330)
- `crates/reposix-remote/src/write_loop.rs` (lines 1-219)
- `crates/reposix-remote/src/precheck.rs` (lines 1-290)
- `crates/reposix-remote/src/fast_import.rs` (lines 85-215)
- `quality/gates/agent-ux/lib/litmus-flow.sh` (full file)
- `quality/gates/agent-ux/milestone-close-vision-litmus.sh` (full file)
- `scripts/refresh-tokenworld-mirror.sh` (full file + header)
- `scripts/confluence_tokenworld.py` (full file)
- `crates/reposix-cli/src/sync.rs` (doc comments, lines 1-74)
- `crates/reposix-cli/src/attach.rs` (bus-URL construction, lines 14-370)
- `docs/concepts/dvcs-topology.md` (full file)
- `docs/guides/dvcs-mirror-setup.md` (full file)
- `docs/guides/troubleshooting.md` § "DVCS push/pull issues" (lines 227-421)
- `docs/reference/testing-targets.md` (full file — confirmed zero mention of `refresh-tokenworld-mirror.sh`)
- `quality/PROTOCOL.md` (pre-release-real-backend cadence contract, grep hits)
- `quality/catalogs/agent-ux.json` (rows: `bus-precheck-a-mirror-drift-emits-fetch-first`, `bus-precheck-b-sot-drift-emits-fetch-first`, `milestone-close-vision-litmus-real-backend`)
- `quality/catalogs/doc-alignment.json` (grep for `"git pull --rebase"` — 12 row hits)
- `quality/gates/structure/orphan-scripts-audit.py` (full file — confirms structural constraints on `refresh-tokenworld-mirror.sh`'s registration)
- `.planning/CONSULT-DECISIONS.md` (lines 113-136 — the 2026-07-16 manager ruling, verbatim)
- `.planning/milestones/v0.15.0-phases/good-to-haves/part-01.md` (lines 40-51 — GTH-V15-09 fix-sketch, verbatim)
- `.planning/milestones/v0.15.0-phases/good-to-haves/part-04.md` (lines 63-77 — GTH-V15-38, verbatim)
- `.planning/milestones/v0.15.0-phases/surprises-intake/part-02.md` (lines 5-17 — original DRAIN-02 discovery entry, verbatim)
- `.planning/REQUIREMENTS.md` (grep DRAIN-02/DRAIN-12, lines 166-169, 242-246)
- `.planning/STATE.md` (top 80 lines — current cursor, P124 CLOSED GREEN, next = P125)
- `crates/reposix-remote/tests/{bus_precheck_a,bus_precheck_b,push_conflict}.rs`, `crates/reposix-cli/tests/agent_flow.rs` (grep for pinned substring)
- Live `gh run list -R reubenjohn/reposix-tokenworld-mirror -L 5` (executed this session — confirms workflow's last-known-run state)
- Live `git --version` / `cargo --version` / `target/debug/reposix` presence checks (executed this session)

### Secondary (MEDIUM confidence)
- None — all findings in this research trace to primary sources listed above.

### Tertiary (LOW confidence)
- None flagged separately; all uncertain items are captured in the Assumptions Log
  above with explicit risk statements rather than as unlabeled LOW-confidence prose.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all reused tools read directly.
- Architecture (root-cause trace): HIGH for the code-path trace (all three precheck
  functions read in full); MEDIUM for the claim that PRECHECK A/B are definitively NOT
  the firing site (static trace only, not confirmed via a live reproduction — see
  Assumption A1).
- Pitfalls: HIGH — Pitfalls 2-5 are each backed by a direct grep/read/live-command
  verification, not inference.
- Manager-ruling scope constraint (Option C not sanctioned): HIGH — verbatim ruling
  quoted from `.planning/CONSULT-DECISIONS.md`.

**Research date:** 2026-07-18
**Valid until:** 30 days (2026-08-17) for the code-path analysis (stable unless the
bus push / precheck mechanism is refactored elsewhere first); re-verify the
`reposix-tokenworld-mirror` GH Action's live run status immediately before
implementation, since that is a live external fact that could change independently of
this repo.
