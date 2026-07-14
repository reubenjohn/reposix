← [back to index](./index.md)

# Architecture & constraints

## Subtle architectural points (read before T03)

The two below are flagged because they are the most likely sources of
T02 → T03 review friction. Executor must internalize them before writing
the wiring code.

### S1 — Refs live in the CACHE's bare repo, not the working tree

The `refs/mirrors/<sot>-head` + `refs/mirrors/<sot>-synced-at` refs are
written to **the cache's bare repo** (`Cache::repo` — a `gix::Repository`
handle on `<cache-root>/reposix/<backend>-<project>.git/`), NOT to the
working tree's `.git/` directory. The working tree receives them via
the helper's `stateless-connect` `list` advertisement (T03 widens this
from `refs/heads/main` only to also include `refs/mirrors/*`).

**Why this matters for T03.** A reviewer skimming the wiring may expect
the helper to call something like `git update-ref` on the working tree's
`.git/` — that would be wrong. The helper has no working-tree handle in
`handle_export`. It has `state.cache: Option<Cache>`, and the cache owns
the bare repo. Mirror refs land in the cache; vanilla `git fetch` from
the working tree pulls them across via the helper.

This also resolves RESEARCH.md pitfall 4 ("Cache vs. working-tree
confusion"). Document the cache-as-source-of-truth point in the new
`mirror_refs.rs` module-doc (T02) and in CLAUDE.md (T04 epilogue).

### S2 — `sot_sha` is the cache's post-write synthesis-commit OID

The `<sot-host>-head` ref records "the SHA of the SoT's main at last
sync." For the single-backend push path (P80 scope), the helper does
NOT push to a real GH mirror — that wiring lands in P83. So the SHA
the ref records is the cache's **post-write synthesis-commit OID**:
the OID that `Cache::build_from()` returns AFTER the SoT REST writes
have landed.

The implementation in T03 calls `cache.build_from().await?` AFTER
`execute_action` succeeded for every action and BEFORE the
`proto.send_line("ok refs/heads/main")`. The returned `gix::ObjectId`
is what's written to `refs/mirrors/<sot>-head`. This is verified
against `crates/reposix-cache/src/builder.rs:56` (`pub async fn build_from(&self) -> Result<gix::ObjectId>`).

**Why not `parsed.commits.last()`?** RESEARCH.md Assumption A2 noted the
ParsedExport shape was uncertain. Verified at planning time: `ParsedExport`
has `commit_message`, `blobs`, `tree`, `deletes` — NO `commits` field.
The synthesis-commit OID is what's actually meaningful for the mirror-head
contract: it's the SHA the cache's bare repo presents to vanilla
`git fetch` after this push. Use `cache.build_from()` (already a
helper-internal API) — do NOT introduce a new accessor.

Trade-off this introduces: `build_from` is "tree sync = full" per
`crates/reposix-cache/src/lib.rs:7-10` — calling it a second time per
push touches the full tree-list cycle. For single-backend push at
P80 scale (one space, dozens to hundreds of issues) this is acceptable.
P81's L1 migration (`list_changed_since`) would let us avoid this
re-list, but P81 ships AFTER P80. Document the cost as a comment
adjacent to the `build_from` call in `handle_export`; cite the
`.planning/research/v0.13.0-dvcs/architecture-sketch.md § "Performance subtlety"`
deferral target.

## Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P80) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints THREE rows + verifier shells
   BEFORE T02-T04 implementation. Initial status `FAIL`. Hand-edit per
   documented gap (NOT Principle A) — annotated in commit message
   referencing GOOD-TO-HAVES-01.
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-cache`,
   `cargo check -p reposix-remote`, `cargo nextest run -p <crate>`.
   Pre-push hook runs the workspace-wide gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 — never parallel,
   even though T02 (cache crate) and T03 (helper crate) touch different
   crates. CLAUDE.md "Build memory budget" rule is "one cargo invocation
   at a time" — sequencing the tasks naturally honors this.
4. **OP-3 audit unconditional.** When `handle_export` writes the refs on
   success, the cache logs an `audit_events_cache` row with
   `op = 'mirror_sync_written'`. Ref writes are best-effort (log WARN +
   continue per RESEARCH.md pattern §"Wiring `handle_export` success path");
   the audit row is UNCONDITIONAL — even if both ref writes failed,
   the audit row records the event. T03's wiring uses the same shape as
   the existing `log_helper_push_accepted` precedent (line 471).
5. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
   codified 2026-04-30).** T04 ends with `git push origin main`; pre-push
   gate must pass; verifier subagent grades the three catalog rows
   AFTER push lands. Verifier dispatch is an orchestrator-level action
   AFTER this plan completes — NOT a plan task.
6. **CLAUDE.md update in same PR (QG-07).** T04 documents the
   `refs/mirrors/<sot-host>-{head,synced-at}` namespace + the Q2.2
   doc-clarity contract carrier (full docs treatment defers to P85).
7. **No new error variants.** Per RESEARCH.md "Wiring `handle_export`
   success path", ref-write failure is best-effort with a
   `tracing::warn!` line. The push still acks `ok` to git. This
   matches the existing `Cache::log_*` family pattern (let-else +
   `let _ = ...` drop) and introduces NO new `RemoteError` variant
   nor new `cache::Error` variant.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces NO new
trifecta surface beyond what the cache + helper already have:

| Existing surface              | What P80 changes                                                                                                     |
|-------------------------------|----------------------------------------------------------------------------------------------------------------------|
| Cache writes to bare repo     | NEW ref namespace `refs/mirrors/...` — gix-validated names; `sot_host` slug from `state.backend_name` (controlled enum), NOT user input. STRIDE category: Tampering — mitigated by `gix::refs::FullName::try_from` rejecting `..`, `:`, control bytes. |
| Helper outbound HTTP          | UNCHANGED — refs are local to the cache; no new HTTP construction site.                                              |
| Reject-message stderr         | NEW: stderr now cites `refs/mirrors/<sot>-synced-at` timestamp (when present). Bytes copied from cache's annotated-tag message body — these are written by the helper itself (no taint propagation from the SoT). RFC3339 string + arithmetic only. STRIDE category: Information Disclosure (misleading) — mitigated by Q2.2 doc clarity contract carrier. |
| Audit log                     | NEW op `mirror_sync_written` in `audit_events_cache`. Schema unchanged (`op` column is `TEXT`, no DDL change). UNCONDITIONAL per OP-3. |

No `<threat_model>` STRIDE register addendum required — the new
surfaces map to existing categories with existing mitigations. Plan
body's `<threat_model>` section enumerates the three threats per
CLAUDE.md template requirements.

Reflog growth on long-lived caches (RESEARCH.md pitfall 6) is filed as
a v0.14.0 operational concern. P80 adds a one-line note in
`mirror_refs.rs` so a future agent finds it.
