← [back to index](./index.md)

# Testing, Constraints, and Security

## Test fixture strategy

**Recommendation: option (a) — `git --bare init` local mirror for unit-style tests.**

| Option | Fits where |
|--------|-----------|
| (a) `git --bare init` local | Default for `crates/reposix-remote/tests/mirror_refs.rs` and the three verifier shells. Zero external deps; CI-stable; ~100ms per test. |
| (b) Real GH mirror `reubenjohn/reposix-tokenworld-mirror` | `#[ignore]`-tagged smoke at milestone-close. Already has DVCS-MIRROR-REPO-01 documenting the endpoint [VERIFIED: recent commit ca5164b]. |
| (c) gix in-memory mirror | REJECTED — annotated-tag round-tripping through gix-only mirrors is unproven; `git --bare init` is upstream truth. |

The single-backend push doesn't actually push to a remote git mirror in P80 — refs are written to the **cache's** bare repo. The "mirror" in the ref name is conceptual until P83 (bus push) wires the actual `git push <mirror>` step. P80 verifies refs propagate through the existing helper's `stateless-connect` advertisement; the actual mirror endpoint is not exercised until P83 + P84.

This is a **subtle point worth flagging in the plan**: the success criterion 3 ("vanilla `git fetch` brings refs along") is satisfied by fetching from the *cache via the helper*, not from a real GH mirror. The GH mirror smoke is P83/P84 territory.

## Pitfalls

1. **`PreviousValue::Any` overwrites silently.** Idempotent re-runs (good — same SHA writes a no-op edit) but a buggy caller could clobber a newer ref with an older SHA. Mitigation: callers in P80 are confined to `handle_export` success path, which only fires after a successful SoT write — the new SHA is by construction newer.
2. **Annotated tag vs. lightweight tag.** `synced-at` is annotated (per architecture sketch + ROADMAP success criterion 1); `head` is a direct ref (lightweight equivalent). Don't accidentally use the same code path for both — the message body is the whole point of `synced-at`.
3. **Helper's ref advertisement is currently scoped to `refs/heads/main` only.** Vanilla `git fetch` will NOT bring `refs/mirrors/*` along until the helper's `list` output is widened. This is the only externally-visible behavior change in P80 — needs a sentence in CLAUDE.md § Threat model (refs in this namespace are read-only metadata; no privacy implication beyond the SoT SHA, which is also visible in the working tree commit history).
4. **Cache vs. working-tree confusion.** `handle_export` writes to the cache's bare repo (where refs live); the working-tree clone receives them via the helper's read advertisement. A reviewer might expect refs to land in `<working-tree>/.git/refs/mirrors/...` directly — they do not. They land in `<cache>/refs/mirrors/...` and propagate. Document this in `mirror_refs.rs` module-doc.
5. **`sot_host` derivation.** ROADMAP says "for confluence == `confluence`" but the helper today knows `(backend_name, project)` — `state.backend_name` is `"sim" | "github" | "confluence" | "jira"`. Use `backend_name` directly for `<sot-host>` slug; rejection-message text uses the human-readable name interchangeably. Don't introduce a new "host" concept — `backend_name` is sufficient.
6. **Reflog noise.** Every push writes a reflog entry on both refs. For long-lived caches this grows unboundedly; gix doesn't auto-prune. Filed as a v0.14.0 concern (operational maturity scope per the v0.14.0 vision doc) — NOT in P80 scope. Add a one-line note in `mirror_refs.rs` so a future agent finds it.
7. **`first push` reject case.** If a reject fires before any successful push, `read_mirror_synced_at` returns `None` — the hint should omit the timestamp line cleanly, not print "synced at None ago". Test `reject_hint_cites_synced_at_with_age` covers the populated path; add a sibling test for the empty-case hint composition.
8. **Q2.2 doc-clarity contract.** ROADMAP success criterion 4 demands the verbatim Q2.2 clarification appear in user-facing docs (P85 territory). P80 carries the prose into CLAUDE.md only — full docs treatment is P85's job. Don't try to ship `dvcs-topology.md` in this phase.

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | `gix::Repository::tag(...)` API exists at the workspace pin (gix 0.83) and creates an annotated tag object + ref atomically | Architecture Patterns / Pattern 2 | If the API differs, the planner uses two RefEdits instead (one to write the tag object via `repo.write_object()`, one to point the ref at it). Either path works; the API name only changes the implementation by ~10 lines. [ASSUMED — sync_tag.rs uses `edit_reference` only, not `tag`; planner should grep gix 0.83 docs to confirm] |
| A2 | `parsed.commits.last()` (or analogous field of `ParsedExport`) yields the SoT SHA the working-tree commit landed at | Wiring `handle_export` | If the field name differs, the planner needs to read `crates/reposix-remote/src/export_parser.rs` (or wherever `ParsedExport` is defined) for the right accessor. Mechanical fix; doesn't change the design. [ASSUMED] |
| A3 | The helper's `list` advertisement is currently limited to `refs/heads/main` and adding `refs/mirrors/*` is a one-line widening | Pitfall 3 | If the advertisement uses a different mechanism (e.g., `gix-protocol` object-info filtering), the change might be more involved — but still bounded. [ASSUMED — `crates/reposix-remote/src/stateless_connect.rs` not read in this research session] |

These assumptions are LOW-impact: each has a known fallback path (slightly different gix API; slightly different parser field name; one extra line of advertisement). None blocks the design.

## Open Questions

1. **Should `mirror_refs.rs` be a sibling of `sync_tag.rs` or extend `sync_tag.rs`?**
   - What we know: both write refs to the cache's bare repo; both audit; both use `RefEdit`.
   - What's unclear: the architecture sketch treats them as conceptually distinct (sync tags = "what did we observe at time T"; mirror refs = "what did we last successfully publish to the mirror"). The audit ops are different (`sync_tag_written` vs. `mirror_sync_written`).
   - **Recommendation:** sibling file. Keeps each concern legible; the copy-paste cost is ~30 lines.

2. **Does the helper's reject path read `synced_at` from the *cache* or from the *working tree's* `refs/mirrors/<sot>-synced-at`?**
   - The helper has the cache handle (it writes audit rows there). Reading from the cache is one-line.
   - **Recommendation:** cache. Matches the cache-as-source-of-truth contract; working tree is the consumer.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `git` | Verifier shells (vanilla-fetch test); `git --bare init` fixtures | ✓ | system (CI requires `>= 2.34` per CLAUDE.md) | — |
| `cargo` | `cargo check -p reposix-cache` / `-p reposix-remote`; `cargo nextest` for integration | ✓ | 1.82+ via rust-toolchain.toml | — |
| `reposix-sim` | Each verifier starts a sim on a unique port (mirrors `reposix-attach.sh:34`) | ✓ | local crate | — |

**Missing dependencies with no fallback:** none. **Missing dependencies with fallback:** none.

## Validation Architecture

### Test framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` / `cargo nextest run` (workspace-default) |
| Config file | `.cargo/config.toml` (workspace-pinned) |
| Quick run | `cargo nextest run -p reposix-remote --test mirror_refs` |
| Full suite | `cargo nextest run --workspace` (gated by pre-push) |

### Phase requirements → test map
| Req ID | Behavior | Test type | Automated command | File exists? |
|--------|----------|-----------|-------------------|-------------|
| DVCS-MIRROR-REFS-01 | Helper APIs in `crates/reposix-cache/` | unit | `cargo nextest run -p reposix-cache mirror_refs` | ❌ Plan creates |
| DVCS-MIRROR-REFS-02 | `handle_export` updates both refs on success | integration | `cargo nextest run -p reposix-remote --test mirror_refs::write_on_success_updates_both_refs` | ❌ Plan creates |
| DVCS-MIRROR-REFS-03 | Reject messages cite both refs | integration | `cargo nextest run -p reposix-remote --test mirror_refs::reject_hint_cites_synced_at_with_age` | ❌ Plan creates |

### Sampling rate
- **Per task commit:** `cargo check -p <crate>` (per-crate, per CLAUDE.md memory budget)
- **Per task verification:** `cargo nextest run -p <crate>` scoped
- **Phase gate:** pre-push hook runs full workspace; verifier-subagent grades the three catalog rows from `quality/gates/agent-ux/mirror-refs-*.sh` artifacts

### Wave 0 gaps
- [ ] `crates/reposix-cache/src/mirror_refs.rs` — new module
- [ ] `crates/reposix-remote/tests/mirror_refs.rs` — new test file
- [ ] `quality/gates/agent-ux/mirror-refs-*.sh` — three verifier shells

## Security Domain

| ASVS category | Applies | Standard control |
|---------------|---------|-----------------|
| V2 Authentication | no | Refs are written by the helper which already holds backend creds; no new auth surface. |
| V5 Input Validation | yes | `gix::refs::FullName::try_from` validates ref names; `sot_host` slug is from the controlled `state.backend_name` enum, not user input. |
| V6 Cryptography | no | No new crypto — refs are git plumbing. |
| V8 Data Protection (`Tainted<T>`) | yes | The SoT SHA written into the head ref is itself derived from the working-tree commit (already-tainted territory). No new tainted-byte path; ref content is the SHA only, which is metadata, not body bytes. |

### Threat patterns
| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Malicious ref name injection | Tampering | gix `FullName::try_from` rejects `..`, `:`, control bytes; `sot_host` derived from enum (`backend_name`), not user input. |
| Stale `synced-at` misread as "current SoT state" | Information Disclosure (misleading) | Q2.2 verbatim doc clarification carried into CLAUDE.md in same PR; full docs treatment in P85. |
| Reflog growth on long-lived caches | DoS (disk) | Filed as v0.14.0 operational concern; not P80 scope. |

## Project Constraints (from CLAUDE.md)

- **Build memory budget:** per-crate cargo only; no parallel cargo invocations across subagents.
- **Catalog-first:** the three `agent-ux.json` rows + verifier shells land in T01, BEFORE T02's Rust impl.
- **Per-phase push:** phase closes with `git push origin main` BEFORE verifier-subagent dispatch.
- **CLAUDE.md update in same PR:** `refs/mirrors/<sot>-{head,synced-at}` namespace + Q2.2 clarification get a paragraph (not a narrative) inserted into the Architecture or Threat-model section.
- **OP-1 simulator-first:** all verifier shells use `reposix-sim`; real-mirror smoke is `#[ignore]`-tagged.
- **OP-3 audit non-optional:** new `mirror_sync_written` audit op; best-effort WARN-on-fail consistent with sibling helpers.
- **OP-7 verifier-subagent on phase close:** dispatch reads catalog rows from artifacts only.
- **OP-8 +2 reservation:** items < 1hr / no new dep get fixed in-phase; else `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`.

## Sources

### Primary (HIGH confidence)
- `/home/reuben/workspace/reposix/.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "2. Mirror-lag observability via plain-git refs" (lines 52–82) and § "3. Bus remote ... step 8" (line 130–131).
- `/home/reuben/workspace/reposix/.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+1 (mirror-lag refs) decisions" (lines 20–30) — Q2.1/Q2.2/Q2.3 ratified.
- `/home/reuben/workspace/reposix/.planning/REQUIREMENTS.md` lines 65–68 — DVCS-MIRROR-REFS-01..03 verbatim.
- `/home/reuben/workspace/reposix/.planning/ROADMAP.md` Phase 80 (lines 83–101) — success criteria.
- `/home/reuben/workspace/reposix/crates/reposix-cache/src/sync_tag.rs` — donor pattern for ref writing.
- `/home/reuben/workspace/reposix/crates/reposix-cache/src/audit.rs` lines 340–363 (`log_sync_tag_written`) — donor pattern for new audit row.
- `/home/reuben/workspace/reposix/crates/reposix-remote/src/main.rs` lines 280–491 (`handle_export`) — wiring point.
- `/home/reuben/workspace/reposix/crates/reposix-cache/Cargo.toml` — dependency confirmation (gix, chrono, rusqlite all present).
- `/home/reuben/workspace/reposix/quality/gates/agent-ux/reposix-attach.sh` — verifier-shell shape.
- `/home/reuben/workspace/reposix/quality/catalogs/agent-ux.json` — `agent-ux/reposix-attach-against-vanilla-clone` row shape.

### Secondary (MEDIUM confidence)
- `crates/reposix-cache/src/db.rs` lines 1–120 — confirmed `audit_events_cache` table schema applies; new `op` value drops in without DDL changes.
- `crates/reposix-remote/src/main.rs` lines 309–314 (`ensure_cache` → `state.cache.as_ref()`) — confirmed cache handle is available in `handle_export`'s success branch.

### Tertiary (LOW confidence)
- `gix::Repository::tag(...)` API existence at gix 0.83 — assumed standard; planner verifies in T02.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dependency already in cache `Cargo.toml`.
- Architecture: HIGH — `sync_tag.rs` is a near-isomorphic donor.
- Wiring point: HIGH — line numbers verified by direct read of `handle_export`.
- Catalog row design: HIGH — `reposix-attach.sh` is a shipped precedent (P79).
- Annotated tag implementation: MEDIUM — `Repository::tag` API name assumed; falls back to two `RefEdit`s if needed.
- Test fixture choice: HIGH — `git --bare init` is the obvious cheap path; #[ignore] smoke matches v0.13.0 milestone-close pattern.

**Research date:** 2026-05-01
**Valid until:** 2026-05-31 (30 days — stable APIs; gix 0.83 just bumped in P78 so unlikely to drift mid-milestone).
