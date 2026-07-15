# GOOD-TO-HAVES

> **Purpose.** OP-8 nice-to-have intake — improvements (clarity, perf, consistency,
> grounding) observed but not folded in. Sized XS / S / M; XS items always close; M
> items default-defer to the next milestone with a named carry-forward target. Drained
> by a milestone's Slot-2 phase.

_Append below this line as polish opportunities surface._

## GOOD-TO-HAVES-01 — `kcov-nested-attribution-flake` — kcov nested-attribution flake loses coverage for 3 fleet-safety gates

**Discovered during:** v0.14 shell-coverage hotfix (commit `15bbe88`)

**Size:** S (rough effort estimate)

**Severity:** LOW — nice-to-have, non-blocking (the floor clears today on stably-traced targets).

**Source:** during the v0.14 shell-coverage hotfix (harness
`quality/gates/code/shell-coverage-tests/11-fleet-safety-guards.sh`, commit `15bbe88`),
3 of 4 fleet-safety gates — `fleet-safety-tat-identity-reject.sh`,
`fleet-safety-leaf-isolation-enforce.sh`, `fleet-safety-shared-config-write-guard.sh` —
trace 0% under kcov when run *nested* (harness→gate→helper), despite executing correctly
(exit 0). Run directly under kcov, `tat-identity` traces 50/52 lines. Common trait of the
3 flaky gates: they delegate real logic to a `scenario` function invoked via the sourced
`quality/gates/agent-ux/lib/transcript.sh` helper; `fleet-safety-evasion-probe.sh` (all
logic in its own body) traces reliably at 98.4%.

**Impact:** the passing aggregate (14.78%, floor 13) is PESSIMISTIC — it already counts
those 3 at 0%. Banking their lines would add ~several points of margin. Non-blocking; the
floor clears today on stably-traced targets (leaf-isolation-guard.sh direct drives +
evasion-probe).

**Acceptance:** the 3 flaky fleet-safety gates trace their real coverage under nested
kcov execution, so the shell-coverage aggregate reflects their lines. Sketch: either
(a) inline the `scenario` logic into each gate body, or (b) investigate kcov's
`--include-path` / subprocess attribution under nested shebang execution so sourced-helper
delegation is traced.

**Default disposition:** S closes-or-defers; safe to defer (non-blocking margin gain, not
a live failure). Close early if the inline-`scenario` pass proves < 1h.

**STATUS:** OPEN

## GOOD-TO-HAVES-09 — `slug-to-id-durable-create-model` — model create as a durable slug→id translation (interrupted-create duplicate elimination)

**Discovered during:** v0.14.0 P108 (paperwork-closure filing of the ADR-010 slug→id waiver
as a first-class intake remainder — originally filed only in
`.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md`; mirrored here 2026-07-12 per owner
deferral decision below).

**Size:** M (design-level, multi-crate reconciliation redesign)

**Severity:** MEDIUM-HIGH — data-integrity hazard confined to id-reassigning real backends
(GitHub Issues / JIRA / Confluence), recoverable by hand-deleting one duplicate; sim +
client-id backends are unaffected.

**One-line hazard:** a `create` against an id-assigning real backend that is cut off
mid-push can, on retry, leave one duplicate record — ADR-010's convergence contract
("already-landed writes are diffed away against the recomputed base") holds for UPDATEs
(stable ids) but is FALSE for CREATEs, whose backend-assigned id is unknown until the
interrupted call completes.

**Fix sketch:** redesign reconciliation to model a create as a durable slug→id translation
— mint a stable local slug before the push, model the create as "slug X → (pending) →
backend id N", so an interrupted create leaves a well-defined resumable state instead of
blindly re-creating.

**Pointer:** ADR-010 §3 (`docs/decisions/010-l2-l3-cache-coherence.md`); full detail and
prior discussion at `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` GOOD-TO-HAVES-09;
`.planning/milestones/v0.14.0-phases/ROADMAP.md` Phase 108 headline note.

**Default disposition:** M default-defers to the next milestone with a named
carry-forward target.

**TAG:** v0.15.0

**STATUS:** DEFERRED — owner scope call, 2026-07-12 (explicit deferral past v0.14.0
milestone-close, not a silent slip).

## GOOD-TO-HAVES-10 — `cargo-nextest-not-installed` — cargo nextest absent on the VM; plan/CLAUDE verify commands fail as written

**Discovered during:** 114-01 (Confluence oid-drift render-parity)

**Size:** XS

**Source:** `cargo nextest run -p reposix-confluence …` returns `error: no such command:
nextest` on this VM. `crates/CLAUDE.md`, `114-01-PLAN.md`, and `114-RESEARCH.md` all name
`cargo nextest` as the canonical per-crate / full-workspace test runner, but it is not
installed — every nextest verify command a plan copies verbatim fails, and the executor
must silently substitute `cargo test`.

**Acceptance:** either install `cargo-nextest` in the VM image (preferred — restores the
memory-friendlier one-binary-at-a-time test linking `crates/CLAUDE.md` relies on for
full-workspace runs), OR soften the docs/plans that hard-name `cargo nextest` to
"`cargo nextest` if available, else `cargo test`" so the copy-paste command always works.

**Default disposition:** XS always closes.

**STATUS:** OPEN

## GOOD-TO-HAVES-11 — `cursor-body-format-guard-substring-false-skip` — cursor re-append guard uses a loose `contains("body-format=")` substring check

**Discovered during:** 114-01 follow-up (code-review nit #3)

**Size:** XS

**Source:** the FIX-01 defensive re-append in `crates/reposix-confluence/src/client.rs`
(`list_issues_impl`'s `next_url` closure, ~L293) skips re-appending
`body-format=atlas_doc_format` whenever the followed cursor url already `contains("body-format=")`.
If a `_links.next` cursor ever carried a *different* representation (e.g.
`body-format=storage`), the guard would false-skip and leave the wrong representation on
the follow — reintroducing render-drift on page 2+. Negligible in practice: Confluence
echoes back the `body-format` we sent (`atlas_doc_format`), so a cursor never carries a
foreign value; the render-parity fix only ever sends `atlas_doc_format`.

**Acceptance:** tighten the guard to check for the specific value
(`contains("body-format=atlas_doc_format")`), or parse the query and assert the param
equals `atlas_doc_format`, so a foreign `body-format` on a cursor forces a corrective
re-append rather than a false-skip.

**Default disposition:** XS always closes.

**STATUS:** OPEN

## Entry format

```markdown
## GOOD-TO-HAVES-NN — `<short-id>` — one-line title

**Discovered during:** P<N>

**Size:** XS|S|M (rough effort estimate)

**Source:** where this was noticed.

**Acceptance:** what "done" looks like.

**Default disposition:** XS always closes; S closes-or-defers; M default-defers to the
next milestone with a named carry-forward target.

**STATUS:** OPEN
```
