# Phase 21 CONTEXT — Hardening bundle: contention swarm, 500-page truncation, credential hygiene, macOS parity (OP-7)

> Status: scoped in session 5, 2026-04-14.
> Author: planning agent, session 6 prep.
> Can run in parallel with other v0.7.0 phases (disjoint files).

## Phase identity

**Name:** Hardening bundle — contention swarm, 500-page truncation, credential hygiene, SSRF regression, audit-log chaos, macOS parity (OP-7).

**Scope tag:** v0.7.0 (hardening + CI — no user-visible API changes).

**Addresses:** OP-7 from HANDOFF.md. Some items were partially completed in session-4 drive-bys (commits `ea5e548`, `f357c92`). This phase audits what is already done and completes what remains.

## Goal (one paragraph)

The v0.3–v0.5 read path is green against real GitHub and Confluence, but it has not been pressure-tested under load, contention, credential-hygiene adversarial input, or macOS. This phase runs the full hardening probe suite: concurrent-write contention swarm (If-Match 409 deterministic proof), FUSE under real-backend load with 500-issue/page repos (p99 vs SG-07 15s ceiling), silent truncation guard for the 500-page Confluence cap, credential hygiene fuzz, SSRF regression completeness audit, tenant-name redaction in log URLs, audit-log WAL consistency under kill-9, and macOS + macFUSE parity in CI. The phase starts with an audit of what session-4 drive-bys already shipped before writing any new code.

## Source design context

From HANDOFF.md §OP-7 (verbatim bullet list):

- **Concurrent writes against the sim.** Repeat Phase 9's swarm harness but with contention: N agents editing the same `0001.md`. Target: prove `If-Match: "<version>"` returns 409 deterministically; every winning write appears exactly once in `audit_events`; no torn writes. Extend `reposix-swarm` with a `--contention` mode (50 clients, same issue, 30s loop).
- **FUSE under real-backend load.** Phase 9 measured sim-direct + fuse-over-sim. Repeat over `--backend github` and `--backend confluence` against a 500-issue repo / 500-page space. Expected finding: rate-limit gate works, but p99 blows past SG-07's 15s list ceiling on cold cache — may need to split `list_issues` into a paginated-returns-progressively iterator instead of a fat single call.
- **Long-path / large-space limits.** `reposix-confluence` caps `list_issues` at 500 pages. Verify: what happens page 501 through ∞? A silent truncation is an SG-05 taint escape (the agent thinks it has the whole space when it doesn't). Ship a WARN log + a `--no-truncate` CLI flag that errors instead of silently capping.
- **Credential hygiene fuzz.** Grep every committed file + `tracing::` span + panic message for the characters `ATATT3` (the canonical Atlassian token prefix). Add a pre-push hook that rejects a commit if any `.rs` file contains a literal `Bearer ATATT3` or similar. One-day work; very cheap insurance.
- **SSRF regression.** WR-02 validated space_id server-side. What about `webui_link` or `_links.base` returned by Confluence? Malicious server could put `https://attacker.com` there — our adapter ignores those fields today, but a future "follow the webui_link for screenshots" feature would reopen the door. Write a wiremock test now that feeds adversarial `_links.base` + asserts no outbound call.
- **Tenant-name leakage.** `tracing::warn!` on 429 includes the full URL — which contains the tenant. If tracing is shipped to a third-party observability backend, tenant inference is possible. Consider: redact tenant in log URLs, or make the HttpClient wrapper do it.
- **Audit log under restart.** The sim's audit DB uses WAL mode. If the sim crashes mid-PATCH, is there a consistency path? Kill -9 the sim during a swarm run and check for dangling rows. Swarm harness could add a `--chaos` mode that kill-9s the sim every 10s.
- **macOS + macFUSE parity.** Today Linux-only. macFUSE support is a ~2-day CI matrix + conditional `fusermount3` → `umount -f` swap. Worth a Phase 14.

## Already partially done (session-4 drive-bys — audit before coding)

- **Credential-hygiene pre-push hook** (`f357c92` + `5361fd5`): `scripts/hooks/pre-push` and `scripts/install-hooks.sh` shipped in session 4. `scripts/hooks/test-pre-push.sh` 6/6 green. Verify this is still complete or identify gaps.
- **SSRF regression tests** (`ea5e548`): Three wiremock tests in `crates/reposix-confluence/tests/contract.rs` covering `_links.base`, `webui_link`, `_links.webui`, `_links.tinyui`, `_links.self`, `_links.edit`. Verify coverage is complete; identify any remaining adversarial surfaces.

Start this phase by running `cargo test --workspace --locked` and grepping for `ssrf` / `pre-push` test coverage before writing anything new.

## Canonical refs

- `crates/reposix-swarm/` — `SimDirectWorkload`, Phase 9 swarm harness; `--contention` mode is an extension here.
- `crates/reposix-confluence/tests/contract.rs` — SSRF tests `ea5e548`; extend for remaining surfaces.
- `scripts/hooks/pre-push` + `scripts/hooks/test-pre-push.sh` — credential hygiene hook `f357c92`.
- `crates/reposix-confluence/src/lib.rs` — 500-page cap; add WARN + `--no-truncate`.
- `.github/workflows/` — add macOS runner matrix with macFUSE conditional.
- `HANDOFF.md §OP-7` — original design capture.
- `research/threat-model-and-critique.md` — SG-05 (silent truncation taint escape), SG-07 (15s list ceiling).
