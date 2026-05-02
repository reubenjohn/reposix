# Phase 84 Research — State of the Art, Assumptions, Open Questions, Constraints, Sources

← [back to index](./index.md)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| Manual `reposix sync` from a workstation | Cron-driven workflow on the mirror repo itself | v0.13.0 P84 | mirror stays current without owner intervention |
| `git push --force` (unsafe) | `git push --force-with-lease` (race-safe) | always — but explicit in v0.13.0 | bus-vs-webhook race protection |
| Webhook-only sync (no fallback) | Webhook + cron safety net | v0.13.0 (this phase) | webhook drops don't strand the mirror |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `gh repo create --add-readme` produces a README on `main` (sub-case 4.3.a) — not 4.3.b. The CARRY-FORWARD says the mirror is "empty except auto-generated README." | First-run handling | If wrong, the YAML's `if git show-ref` branch is the wrong path; either way the YAML handles BOTH sub-cases via the if/else. Risk = low (defensive code handles either reality). [ASSUMED based on `gh repo create` documented behavior] |
| A2 | GH Actions does NOT support `${{ vars.* }}` in cron expressions. | Workflow YAML shape | If wrong, we could template the cron and avoid Pitfall 3. [VERIFIED: github-docs explicitly note that contexts are not available at parse time for `schedule:`; tested empirically by the `bench-latency-cron.yml` precedent which hardcodes `'0 13 * * 1'`] |
| A3 | Confluence webhook to GitHub `dispatches` requires a PAT, not the runner `GITHUB_TOKEN`. | Pitfall 7 | If wrong, owner setup is simpler. [CITED: docs.github.com/en/rest/repos/repos#create-a-repository-dispatch-event] |
| A4 | The 60s p95 target is achievable. | Latency measurement | If real measurement shows p95 > 120s, P85 docs document the constraint per ROADMAP SC4. The phase still ships; the catalog row's `expected.p95_seconds_max` is the falsifiable claim. [ASSUMED — to be validated in T5] |
| A5 | `cargo binstall reposix-cli` works against the latest published version on crates.io. | Standard Stack | If the latest published version lacks the `reposix init` confluence path (e.g., release lag), workflow fails. Mitigation: pin the binstall version in YAML. [ASSUMED — verified the binstall metadata exists; did NOT verify a specific version installs cleanly. T2 should test this empirically before declaring the row green.] |
| A6 | The Confluence TokenWorld webhook can be configured to POST to `api.github.com/repos/.../dispatches`. Atlassian Cloud's outbound-webhook surface supports arbitrary URLs + custom headers (for the PAT). | Secrets convention | If wrong, the setup guide needs a different mechanism (e.g., a relay service). [ASSUMED — Atlassian Cloud's webhook docs claim arbitrary URL + headers; not personally verified for this account.] |

## Open Questions

1. **Should the latency measurement be a recurring CI check or one-shot artifact?**
   - What we know: ROADMAP SC4 says "measured in sandbox during this phase"; doesn't specify recurrence.
   - What's unclear: does the catalog row need `freshness_ttl: 30d` (forces re-measurement) or is it a one-shot phase artifact?
   - Recommendation: start as one-shot (cadence: pre-release), file as v0.14.0 GOOD-TO-HAVE if recurring measurement becomes valuable.

2. **Does the workflow need `concurrency:` to prevent overlapping runs?**
   - What we know: cron + dispatch could fire 2× near a 30-min boundary.
   - What's unclear: does the `--force-with-lease` no-op-on-race property fully eliminate the need for `concurrency:`, or do we still want `concurrency: { group: sync, cancel-in-progress: false }` for runtime efficiency?
   - Recommendation: ADD `concurrency: { group: reposix-mirror-sync, cancel-in-progress: false }` — defends against duplicate runs, idiomatic for GH Actions, costs nothing. Cite the precedent in `ci.yml:16-18`.

3. **Where does the workflow log persist for post-phase audit?**
   - What we know: GH Actions retains run logs for 90 days by default.
   - What's unclear: do we need to mirror the workflow's logs into `audit_events_cache` for OP-3 dual-table compliance?
   - Recommendation: NO — the workflow's `reposix init` step writes its own audit rows to the cache (which is ephemeral and discarded post-run anyway); the workflow run itself is GH-side, not reposix-side. Audit lives where it has always lived.

4. **Should P84's plan reference forward to P85's docs or vice-versa?**
   - What we know: P85 docs the setup; P84 implements the workflow.
   - What's unclear: does the workflow file's comments link to `dvcs-mirror-setup.md` (which doesn't exist yet), or just inline the setup tldr?
   - Recommendation: inline a 5-line tldr in the workflow header comment + say "see `docs/guides/dvcs-mirror-setup.md` (P85) for full walk-through." Forward-references are fine; the docs land before milestone close.

## Project Constraints (from CLAUDE.md)

- **OP-3 dual-table audit non-optional.** Already satisfied — the `reposix init` step inside the workflow writes its own audit rows; P84 doesn't need to add new ones.
- **OP-1 simulator-first.** Synthetic tests use shell stubs against local file:// mirrors; real-backend test (TokenWorld + reposix-tokenworld-mirror) is the headline measurement, gated by secrets.
- **`REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Workflow sets it to `http://127.0.0.1:*,https://${tenant}.atlassian.net` — confluence-only. No GitHub API egress from the cache (the cache talks to confluence; the `git push` to mirror is OS-git, not cache).
- **Build memory budget — single cargo invocation.** P84 has zero new cargo workspace operations; the workflow uses `cargo binstall` (no compilation). Local tests are shell, not Rust. Constraint trivially satisfied.
- **Catalog-first rule.** T1 mints all 6 rows + stub verifiers BEFORE T2's YAML commit; subsequent commits cite the row id.
- **Per-phase push cadence.** T6 closes with `git push origin main` BEFORE verifier subagent dispatch.
- **CLAUDE.md stays current.** T6 updates the "v0.13.0 — in flight" section with a P84 entry summarizing the workflow shape + secrets convention + which rows landed.
- **Workflow runs in mirror repo, NOT canonical repo.** Per CARRY-FORWARD § DVCS-MIRROR-REPO-01. Surface in T1 + T2 explicitly.

## Sources

### Primary (HIGH confidence)
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-driven mirror sync" — full design with verbatim YAML skeleton.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+4 (webhook sync) decisions" — Q4.1, Q4.2, Q4.3 ratified.
- `.planning/ROADMAP.md` § "Phase 84" lines 167-188 — phase goal + 8 success criteria.
- `.planning/REQUIREMENTS.md` — DVCS-WEBHOOK-01..04 verbatim text.
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` § DVCS-MIRROR-REPO-01 — the real GH mirror's existence + scopes + workflow lands in mirror repo.
- `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `.github/workflows/bench-latency-cron.yml` — project's GH Actions idioms (`actions/checkout@v6`, `dtolnay/rust-toolchain@stable`, secrets shape, env wiring).
- `crates/reposix-cache/src/mirror_refs.rs:62-95` — ref name format functions (`format_mirror_head_ref_name`, `format_mirror_synced_at_ref_name`).
- `crates/reposix-cli/src/main.rs:62-75` — `reposix init <backend>::<project> <path>` invocation shape.
- `crates/reposix-cli/Cargo.toml:19-25` — `[package.metadata.binstall]` block (binstall metadata exists).
- `docs/reference/testing-targets.md` — TokenWorld is the sanctioned real-backend webhook sandbox.
- `quality/catalogs/agent-ux.json` — catalog row shape precedent (`bus-precheck-a-mirror-drift-emits-fetch-first` etc.).
- `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh` — verifier script shape precedent.

### Secondary (MEDIUM confidence)
- docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#repository_dispatch (cited for trigger semantics)
- docs.github.com/en/rest/repos/repos#create-a-repository-dispatch-event (cited for PAT requirement)
- git-scm.com/docs/git-push#--force-with-lease (cited for lease semantics)

### Tertiary (LOW confidence)
- Atlassian Confluence webhook configuration UI details — assumed but not personally verified for this account; the setup guide should walk through it as part of P85.

## Metadata

**Confidence breakdown:**
- Workflow YAML shape: HIGH — verbatim derivation from architecture-sketch + project YAML precedents.
- Catalog rows: HIGH — shape from existing `bus-*` row precedent.
- First-run handling: MEDIUM — depends on assumption A1 about `gh repo create --add-readme` behavior; YAML defensively handles both sub-cases regardless.
- Latency measurement: MEDIUM — methodology clear; the 60s number itself is a target, not a verified achievable.
- Force-with-lease: HIGH — git's behavior is well-specified; test fixture is ~50 lines of deterministic shell.
- Cron `vars` constraint (Pitfall 3): HIGH — verified empirically against the `bench-latency-cron.yml` precedent + GH docs.
- PAT requirement (Pitfall 7): MEDIUM — citation is GH docs; not empirically tested in this session.

**Research date:** 2026-05-01
**Valid until:** 2026-05-31 (30 days for stable substrates — workflow APIs change rarely; reposix-cli's binstall metadata could shift on the next release)
