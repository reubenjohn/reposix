# Phase 84 Research — Runtime State, Environment, Validation Architecture, Security

← [back to index](./index.md)

## Runtime State Inventory

> P84 is a YAML + shell phase. Most categories are N/A.

| Category | Items Found | Action Required |
|---|---|---|
| Stored data | None — workflow does not introduce new persistent state. The cache built by `reposix init` lives in `/tmp/sot` for the workflow's lifetime; ephemeral. | none |
| Live service config | **GH repo secrets on `reubenjohn/reposix-tokenworld-mirror`** (ATLASSIAN_*); **GH variable** `MIRROR_SYNC_CRON` (optional); **Confluence webhook** in TokenWorld admin UI pointing at `repos/reubenjohn/reposix-tokenworld-mirror/dispatches`. Owner-managed, NOT in git. | Document in P85 setup guide; verifier asserts the workflow's `secrets.*` references match the named secrets (a static lint, not a live check). |
| OS-registered state | None | none |
| Secrets / env vars | `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` — referenced by name in workflow YAML; SET by owner via `gh secret set`. PAT with `repo` scope on Atlassian side for outbound webhook. | None in code; document in P85. |
| Build artifacts | `cargo binstall` downloads `reposix-cli` from crates.io to the runner's `~/.cargo/bin/` per workflow run. Ephemeral (runner is destroyed). | none |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| `gh` CLI | local testing of workflow + dispatch | ✓ | v2.x (auth confirmed per CARRY-FORWARD § DVCS-MIRROR-REPO-01: "repo + workflow scopes") | — |
| `cargo binstall` | workflow runtime | available on `ubuntu-latest` runners; we install via curl in step | latest | — |
| `actionlint` | YAML lint in shell verifier | NOT installed locally; install on demand | latest | skip lint, rely on GH's parser at deploy time |
| `git ≥ 2.34` | `--force-with-lease` semantics | ✓ on ubuntu-latest (currently 2.43+) | 2.43.x | — |
| Real `reubenjohn/reposix-tokenworld-mirror` repo | latency measurement (approach b) + integration smoke | ✓ provisioned per CARRY-FORWARD; auth confirmed | n/a | approach (a) synthetic-only — lower-bound latency only |
| Confluence TokenWorld webhook | end-to-end real webhook test | NOT YET CONFIGURED — owner must set up via Atlassian admin UI | n/a | use approach (a) `gh api .../dispatches` synthetic trigger |

**Missing dependencies with no fallback:** none — the synthetic harness covers all 6 catalog rows for CI repeatability; real measurement is the headline number but degrades gracefully.

**Missing dependencies with fallback:** the Confluence webhook itself is owner-config-dependent; if not set up by phase-close, the latency measurement uses synthetic dispatch only and we capture a lower-bound number with a note.

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | shell scripts under `quality/gates/agent-ux/` (precedent — see existing `bus-*` verifiers) + Rust `cargo test -p reposix-cli --test webhook_*` for any race-protection unit tests |
| Config file | `quality/catalogs/agent-ux.json` (mints catalog rows); `.github/workflows/reposix-mirror-sync.yml` (in mirror repo) |
| Quick run command | `bash quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` (or any single verifier) |
| Full suite command | `python3 quality/runners/run.py --dimension agent-ux --tag webhook` |
| Phase gate | All 6 webhook-* catalog rows GREEN before phase-close push |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| DVCS-WEBHOOK-01 | Workflow shape ships | mechanical (YAML lint + grep) | `bash quality/gates/agent-ux/webhook-trigger-dispatch.sh` | ❌ Wave 0 (mint stub in T1) |
| DVCS-WEBHOOK-02 | `--force-with-lease` race | mechanical (shell harness) | `bash quality/gates/agent-ux/webhook-force-with-lease-race.sh` | ❌ Wave 0 (mint stub in T1) |
| DVCS-WEBHOOK-03 | First-run empty mirror | mechanical (shell harness) | `bash quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` | ❌ Wave 0 (mint stub in T1) |
| DVCS-WEBHOOK-04 | Latency < 60s p95 | asset-exists (JSON p95 ≤ 120) | `bash quality/gates/agent-ux/webhook-latency-floor.sh` | ❌ Wave 0 (mint stub in T1) |

### Sampling Rate
- **Per task commit:** the affected verifier (e.g., T2 commits the YAML → run the 3 catalog rows it satisfies)
- **Per phase merge:** all 6 webhook-* verifiers + `python3 quality/runners/run.py --dimension agent-ux`
- **Phase gate:** full agent-ux dimension green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `quality/gates/agent-ux/webhook-trigger-dispatch.sh` — DVCS-WEBHOOK-01 verifier
- [ ] `quality/gates/agent-ux/webhook-cron-fallback.sh` — cron block parses
- [ ] `quality/gates/agent-ux/webhook-force-with-lease-race.sh` — DVCS-WEBHOOK-02 verifier (shell harness)
- [ ] `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` — DVCS-WEBHOOK-03 verifier (shell harness, 2 sub-fixtures)
- [ ] `quality/gates/agent-ux/webhook-latency-floor.sh` — DVCS-WEBHOOK-04 verifier (asset-exists asserts JSON p95 ≤ 120s)
- [ ] `quality/gates/agent-ux/webhook-backends-without-webhooks.sh` — Q4.2 trim path produces valid YAML
- [ ] `quality/reports/verifications/perf/webhook-latency.json` — measurement artifact (T5)
- [ ] `.github/workflows/reposix-mirror-sync.yml` IN MIRROR REPO — workflow file itself (T2)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | yes | GitHub PAT for webhook delivery (Confluence-side); `secrets.GITHUB_TOKEN` for workflow runtime; Atlassian API key for `reposix init` step |
| V3 Session Management | n/a | stateless workflow runs |
| V4 Access Control | yes | repo-scoped secrets; no cross-repo leakage; `permissions: contents: write` only |
| V5 Input Validation | partial | `client_payload` from `repository_dispatch` is untrusted input; workflow IGNORES the payload — derives all values from `secrets.*` and `vars.*` (controlled by repo owner) |
| V6 Cryptography | n/a | TLS to confluence + GitHub; no custom crypto |

### Known Threat Patterns for GH Actions + repository_dispatch

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Untrusted `client_payload` injection | Tampering | Don't interpolate `${{ github.event.client_payload.* }}` into shell commands. The reference workflow never reads client_payload — derives all values from `secrets.*` and `vars.*`. |
| Secret exfiltration via workflow logs | Information Disclosure | GH Actions auto-redacts secrets; `set -x` in YAML can leak — avoid in `run:` blocks. |
| Workflow-trigger amplification | DoS | Cron + dispatch combined could fire 2× per 30min if confluence webhook fires near a cron tick. `--force-with-lease` makes the second push a no-op. |
| Cross-repo dispatch by attacker with repo:write on the mirror | Tampering | Owner-controlled mirror repo; only owner has write access. Document in P85 that mirror repo permissions should be `Maintain` for owner only. |
| Allowlist evasion | Tampering | `REPOSIX_ALLOWED_ORIGINS` is set in env to confluence tenant ONLY (no GH Issues, no JIRA, no foreign hosts). |
