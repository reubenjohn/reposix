# Session 5 rationale — cluster pick

> Date: 2026-04-14 (overnight session 5). Author: Claude Opus 4.6 1M-ctx.
> Scratch doc per session-5 brief: pick one cluster, write why.

## Options considered

| Cluster / standalone | Scope sketch | ROI bucket | Cost bucket |
|---|---|---|---|
| **A — Confluence writes** | atlas_doc_format↔Markdown render, `create/update/delete_or_close` on ConfluenceBackend, labels/comments/attachments surface. | **User-visible**: highest | **Risky**: biggest surface, hardest to test, live-backend side effects. Multi-session scope. |
| **B — Decouple sim from FUSE + remote** | `crates/reposix-fuse/src/fs.rs::release` + `::create` routed through `IssueBackend::update_issue`/`create_issue`. `crates/reposix-remote/src/main.rs` swap `api::{list,patch,post,delete}_issue` for `IssueBackend::{list,update,create,delete_or_close}`. Delete sim-specific `fetch::{patch,post}_issue` + `remote/src/client.rs` modules. | **Architectural**: unblocks all future write-path backends for free | **Mechanical**: ~2-3h wall-clock; disjoint crates let subagents fan out. |
| **C — Swarm Confluence-direct workload** | New `--mode confluence-direct` variant of `SimDirectWorkload` with rate-limit awareness. | **Operational**: niche; closes OP-6 gap | Small (~200-300 LoC). Good warm-up. |
| OP-2 — per-dir `INDEX.md` | Synthesized sitemap files in `tree/` and `pages/` dirs. | Medium (agent UX polish, composes with tree/) | Small-medium (~300 LoC). |
| OP-3 — git-pull cache | `reposix refresh` + sqlite-in-mount commit helper. Mount-as-time-machine. | Huge but future | Multi-session. Needs new crate. |
| OP-7 — hardening probes | Concurrent-write contention, 500-page probe, chaos audit restart, SSRF widen, tenant-name leakage. | Low-but-real (not user-visible) | Each probe ~100-200 LoC. |
| OP-8 — count_tokens benchmarks | Swap `len/4` for real Anthropic `count_tokens` API. | Low (honest numbers, not new features) | Small. |

## Decision: **Cluster B** — `v0.4.1` bugfix tag.

### Why B wins

1. **Unblocks everything downstream for free.** The moment `fs.rs::release` routes through `backend.update_issue(...)`, any backend's `IssueBackend::update_issue` impl works through the FUSE mount. That's what the trait seam was designed for and it's currently violated — the FUSE write path hardcodes the sim's REST shape. Cluster A (Confluence writes) is roughly 2× more valuable _after_ B than without, because without B the only way to exercise Confluence writes is CLI `reposix update` — not `vi /mount/pages/00000131192.md`.
2. **Minimal diff; maximum leverage.** Two call sites in `fs.rs` (`release` → PATCH, `create` → POST). Four call sites in `reposix-remote/src/main.rs` (`list_issues`, `patch_issue`, `post_issue`, `delete_issue`). Roughly 100-300 lines of real change plus test churn, plus two modules we can delete entirely (`crates/reposix-fuse/src/fetch.rs::{patch_issue, post_issue}` and `crates/reposix-remote/src/client.rs`).
3. **Load-bearing hidden.** Two v0.3-era HANDOFF entries have flagged this as an open gap — `fetch.rs` still PATCHes `/projects/{p}/issues/{id}` directly, `reposix-remote` still hardcodes the sim REST shape. Session 3 and session 4 both punted. The user called it out again in session 5's brief as load-bearing. If we do not pay this down now, every future write feature (Cluster A, OP-9 comments, OP-9 attachments) continues to hardcode the sim shape — or has to refactor it retroactively with bigger blast radius.
4. **Clean tag story.** Bugfix scope = `v0.4.1`. No new user-visible features, no CHANGELOG surprises. A confidence-building tag that proves the session's internal refactor is durable.
5. **Parallelism-friendly.** Sub-goal B1 (FUSE) touches `crates/reposix-fuse/**`. Sub-goal B2 (remote) touches `crates/reposix-remote/**`. Disjoint filesets; two subagents in parallel per the session-4 playbook.
6. **Safety rails stay intact.** The `Untainted<Issue>` sanitizer + allowlist-gated `HttpClient` continue to protect egress — nothing about the threat model changes. Audit log continues to work (the sim-backed path still writes audit rows; it's just reached through a trait).

### Why not A (yet)

- Biggest user-visible value, but _after_ B. Without B, Confluence writes live only in CLI, not FUSE mount. The user's mental model of the project is "cat + sed + git on real wikis"; that needs the FUSE write path to be backend-agnostic first.
- atlas_doc_format ↔ Markdown round-trip is a **lot** of XHTML nuance. A dedicated renderer + inverse is easily its own phase. Rushing it in one session is how half-ships happen.
- Side effects on a live Atlassian tenant require a cleanup story for test pollution. Need ephemeral space creation, not just the REPOSIX demo space.

### Why not standalones first

- OP-2 (INDEX.md) doesn't depend on B, but the real payoff is when it's also a write-capable view (`INDEX.md` as a directory summary under B's writeable mount). Ship after B.
- OP-7 hardening probes are defence-in-depth. Cheap individually, but none of them are blocking v0.5 features. Good "fill the gap" work post-B if time remains.
- OP-8 is a measurement honesty fix — nice to have, not blocking.

## Stretch goal for this session

If Cluster B lands with time to spare (e.g., CI green by 2PM PST), fold in one of:

1. **Cluster C** (swarm `--mode confluence-direct`). Small, composes with the refactor — the swarm already routes through `IssueBackend` for sim-direct, so adding a confluence-direct variant uses the same trait that just got proven solid.
2. **OP-2 partial** — `pages/INDEX.md` only (not `tree/INDEX.md`). Demonstrates the pattern, doesn't commit the full scope.
3. **OP-7 SSRF + contention probe**. Extends the existing `ea5e548` hardening work.

If stretch lands, tag becomes `v0.5.0` (feature scope). Otherwise `v0.4.1`.

## Non-goals (explicit)

- **Do NOT start OP-10** (eject 3rd-party adapter crates) — user-gated.
- **Do NOT start OP-11** (repo-root reorg of `InitialReport.md` etc.) — user-gated.
- **Do NOT start Phase 12** (subprocess/JSON-RPC ABI) — user-gated, design question open.
- **Do NOT extend `parse_remote_url` to carry backend scheme** as part of Cluster B. That's a feature (v0.5+), not a refactor. Cluster B preserves the current sim-only remote-helper URL semantics — only the plumbing moves to the trait. Real-backend writes via `git push` stay out-of-scope until a follow-up phase adds URL-syntax selection.

## Plan

1. `/gsd-add-phase` — Phase 14, "Decouple sim REST from FUSE write-path + git-remote helper via IssueBackend trait."
2. `/gsd-plan-phase` — research + planner. Expected waves:
   - Wave A (serial): any `IssueBackend` trait-surface additions or error-mapping helpers needed in `reposix-core`.
   - Wave B1 (parallel): FUSE `fs.rs` refactor + `fetch.rs` write-helper deletion. Tests re-home onto `SimBackend` + wiremock.
   - Wave B2 (parallel): `reposix-remote` refactor. Delete `client.rs`. Tests via `SimBackend` pointed at wiremock.
   - Wave C (serial): integration verification — smoke, green-gauntlet, live-demo-12-push-via-git-remote.
3. `/gsd-execute-phase 14` with subagent parallelism for Wave B1+B2.
4. `/gsd-code-review` + `/gsd-verify-work`.
5. If stretch fits: Cluster C as a sibling phase 14.1 or 15; otherwise tag `v0.4.1` via `scripts/tag-v0.4.1.sh`.
6. Write HANDOFF session-5 augmentation.

## Bar

- `cargo test --workspace --locked` green (baseline tonight: clean, smoke 4/4).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` green.
- `bash scripts/green-gauntlet.sh --full` green, including `--ignored` FUSE integration tests.
- Atomic commits per wave with phase-prefix messages (e.g. `refactor(14-B1): fs.rs write path through IssueBackend`).
- `.planning/phases/14-<slug>/` carries CONTEXT + RESEARCH + PLAN + SUMMARY + REVIEW + VERIFICATION.
- Delete `crates/reposix-fuse/src/fetch.rs::{patch_issue, post_issue, EgressPayload, ConflictBody}` and `crates/reposix-remote/src/client.rs` entirely (don't just leave them as dead code).
- Grep proof: `git grep -n 'EgressPayload\|patch_issue\|post_issue' crates/reposix-fuse/src/` returns only read-path helpers (if any survive) and/or tests-against-trait.
