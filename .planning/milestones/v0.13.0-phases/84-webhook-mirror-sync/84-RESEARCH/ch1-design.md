# Phase 84 Research — Design: Responsibilities, Constraints, Requirements, Stack

← [back to index](./index.md)

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Workflow trigger dispatch (webhook + cron) | GitHub Actions runner (in mirror repo) | — | `on: repository_dispatch` + `on: schedule` — both upstream-supported; no custom infra. |
| Cron interval configuration | GitHub Actions `vars` context | Workflow YAML | Per Q4.1: `${{ vars.MIRROR_SYNC_CRON || '*/30 * * * *' }}` — owner sets the var via `gh variable set` on the mirror repo. |
| `reposix init` invocation (build SoT cache) | reposix-cli binary (cargo-binstall'd in workflow step) | reposix-cache, reposix-confluence | Workflow runs `reposix init confluence::TokenWorld /tmp/sot` — same binary path as local dev. |
| Mirror push (force-with-lease) | OS git binary (called by workflow `run:` step) | mirror repo's HTTPS endpoint | Plain `git push --force-with-lease=...` against the configured mirror remote; no reposix code involved on the push side. |
| Mirror-lag refs propagation | Workflow `git push` of `refs/mirrors/...` | reposix-cache (writes refs into local cache; workflow pushes them to mirror) | The `reposix init` step builds the cache + writes the refs; workflow then `git push mirror refs/mirrors/<sot>-head refs/mirrors/<sot>-synced-at`. |
| Latency observation | sandbox harness (TokenWorld + cron-skip variant) | webhook receiver (`repos/<owner>/<repo>/dispatches` POST) | Measure timestamp delta from confluence-edit (REST PATCH) to mirror ref-update (`gh ref view` polling). |
| Atlassian credential plumbing | GitHub repo secrets in mirror repo | Workflow `env:` block | `${{ secrets.ATLASSIAN_API_KEY }}` etc. — owner sets via `gh secret set` (one-time) per the precedent in `ci.yml:117-119`. |

## User Constraints (from upstream)

No CONTEXT.md exists for P84 (no `/gsd-discuss-phase` invocation per `config.json` `skip_discuss: true`). Constraints flow from:

### Locked Decisions (RATIFIED in `decisions.md` 2026-04-30)
- **Q4.1 — Cron `*/30` default, configurable.** Workflow `vars.MIRROR_SYNC_CRON` overrides the default; falls back to `*/30 * * * *` if unset.
- **Q4.2 — Backends without webhooks → cron-only.** Workflow's `repository_dispatch` block is REMOVABLE without breaking; the cron path stands alone. Documented in P85's `dvcs-mirror-setup.md`; no implementation work in P84.
- **Q4.3 — First-run on empty mirror handled gracefully.** Workflow runs cleanly against an empty GH mirror (no `refs/heads/main`, no `refs/mirrors/...`); populates them on first run. Verified by sandbox test.
- **Q2.3 — Bus updates both refs (P83 shipped).** Webhook becomes a **no-op refresh** when the bus already touched the refs. Workflow does NOT need to detect this; idempotent ref-write semantics handle it.
- **Q2.2 — `synced-at` is the lag marker, NOT a "current SoT state" marker.** Workflow updates `synced-at` to NOW only after the mirror push lands (matching bus-remote semantics from P83).
- **OP-1 simulator-first.** Synthetic CI tests use `act` or shell stubs against the simulator; real-backend tests gated by secrets + `REPOSIX_ALLOWED_ORIGINS=https://reuben-john.atlassian.net`.
- **OP-3 dual-table audit.** The workflow's `reposix init` step writes audit rows to `audit_events_cache` for the cache build + each blob materialization; that's free. The workflow itself does NOT need to write extra audit rows — the underlying `reposix init` already audits.
- **CARRY-FORWARD § DVCS-MIRROR-REPO-01.** Workflow YAML lives in `reubenjohn/reposix-tokenworld-mirror`, NOT in `reubenjohn/reposix`. The dispatch target is `https://api.github.com/repos/reubenjohn/reposix-tokenworld-mirror/dispatches`.

### Claude's Discretion
- Whether to ship the workflow as a **template** in `docs/guides/dvcs-mirror-setup-template.yml` (referenced by P85 docs) AND a **live copy** in the mirror repo, or only one of the two. **RECOMMEND: both.** Live copy in the mirror repo is what the catalog asserts exists; template copy in the canonical repo's `docs/guides/` is what the setup guide walks through. Avoid drift by having the live copy `git diff`-comparable to the template (catalog row asserts byte-identical or whitespace-only diff).
- Whether to use `cargo binstall reposix-cli` or `cargo binstall reposix` in the workflow. **RECOMMEND: `reposix-cli`** — that's the actual crate name with binstall metadata (`crates/reposix-cli/Cargo.toml:19`); `reposix` is not a published crate. The architecture-sketch wrote `cargo binstall reposix` as shorthand; correct it.
- Whether to push the two `refs/mirrors/...` refs via a separate `git push <mirror> refs/mirrors/...` invocation or fold into the main push refspec. **RECOMMEND: separate invocation** — clearer error attribution (a refs-write failure shouldn't fail the main-branch push), idiomatic for ref-namespace pushes.
- Whether to make the latency measurement a CI gate or a one-shot phase artifact. **RECOMMEND: one-shot artifact** captured to `quality/reports/verifications/perf/webhook-latency.json` per the ROADMAP SC4. Cron-driven recurring measurement is v0.14.0 territory.

### Deferred Ideas (OUT OF SCOPE)
- **Confluence webhook configuration automation.** Webhooks are set up via Atlassian admin UI (or REST API); P84 documents the manual setup but does NOT automate it. Per architecture-sketch § "Webhook-driven mirror sync": *"Owner sets up once per space."* Filed for v0.14.0 if owner load becomes painful.
- **Bidirectional sync.** Mirror is read-only from confluence's perspective. Out of scope per architecture-sketch § "What we're NOT building".
- **Multi-mirror fan-out.** Workflow targets a single mirror. Generalization deferred to v0.14.0 per same § "What we're NOT building".
- **Real-time webhook latency SLA.** Target is < 60s p95 *measurement*; if exceeded, P85 docs document the constraint — no in-phase tuning beyond surfacing the number.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DVCS-WEBHOOK-01 | Reference workflow ships at `.github/workflows/reposix-mirror-sync.yml` (in mirror repo) | § "Workflow YAML shape" — full skeleton derived from architecture-sketch + verified against existing `bench-latency-cron.yml` precedent. |
| DVCS-WEBHOOK-02 | `--force-with-lease` race protection | § "Force-with-lease semantics" — invariant + sandbox-test fixture. |
| DVCS-WEBHOOK-03 | First-run on empty mirror handled gracefully | § "First-run handling (Q4.3)" — recommended invariant: detect missing `mirror/main`, fall back to plain `git push mirror main` (no lease) on the first-ever run only. |
| DVCS-WEBHOOK-04 | Latency target < 60s p95 measured | § "Latency measurement strategy" — recommend hybrid: real TokenWorld for the headline number, synthetic harness for CI repeatability. |

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|---|---|---|---|
| GitHub Actions runner | ubuntu-latest | host the workflow | Project precedent — every workflow under `.github/workflows/` uses `ubuntu-latest` (verified: `ci.yml:23,33,44,75`, `release.yml:55,98,210`, `bench-latency-cron.yml:27`). |
| `actions/checkout@v6` | v6 | clone the mirror repo into the runner | Project precedent — `ci.yml:46` migrated to v6 from v4 in P78 hygiene. The bench-cron `release.yml` still uses v4 (mixed in repo); v6 is the current preferred. [VERIFIED: grepped both versions in `.github/workflows/`] |
| `dtolnay/rust-toolchain@stable` | stable | install Rust toolchain | Project precedent — every workflow that compiles uses this action. [VERIFIED: 7 occurrences across `.github/workflows/`] |
| `cargo binstall` | latest | install reposix-cli pre-built binary | Avoids a `cargo build --release` step inside the workflow (~2-3 min savings). Binstall metadata exists in `crates/reposix-cli/Cargo.toml:19`. [VERIFIED] |
| `git push --force-with-lease=<ref>:<sha>` | git ≥ 2.34 | safe force-push | Standard git race-protection idiom. The lease form (`<ref>:<sha>`) compares `<sha>` against the actual remote ref state at push time and fails cleanly on drift. [CITED: git-scm.com/docs/git-push#Documentation/git-push.txt---force-with-leaseltrefnamegt] |
| `repository_dispatch` event | GH Actions native | webhook trigger | Standard GH event for cross-repo / external triggering. POST `repos/<owner>/<repo>/dispatches` with `{event_type, client_payload}`. [CITED: docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#repository_dispatch] |

### Supporting
| Tool | Purpose | When to Use |
|---|---|---|
| `actions/upload-artifact@v4` | preserve workflow-run logs / latency JSON for post-run inspection | Reference: `ci.yml:259-265` `latency-table` artifact. Use for `quality/reports/verifications/perf/webhook-latency.json` upload in P84's measurement run. |
| `gh secret set` (CLI, run by owner) | install Atlassian creds as repo secrets | One-time owner action; documented in setup guide. Precedent: `ci.yml:114-120` `gh secret set ATLASSIAN_API_KEY` etc. |
| `gh variable set` (CLI, run by owner) | install `MIRROR_SYNC_CRON` override | Per Q4.1 — owner sets `MIRROR_SYNC_CRON='*/15 * * * *'` (or whatever) via `gh variable set MIRROR_SYNC_CRON --repo reubenjohn/reposix-tokenworld-mirror`. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|---|---|---|
| `cargo binstall reposix-cli` | `cargo install reposix-cli --locked` | binstall ~10s; `cargo install` ~2-3 min compile. Binstall is correct for a workflow that runs every 30 minutes. |
| `gh api repos/.../dispatches` to test webhook delivery | curl POST + manual `Authorization: token` header | `gh api` handles auth from the runner's `GITHUB_TOKEN` automatically. |
| `act` for local workflow testing | shell scripts that stub the workflow steps | `act` requires Docker + image pulls (heavyweight on dev VM); shell stubs are 10× faster and exercise the same `reposix init` + `git push` substrates. Recommend shell stubs (see § "Test infrastructure"). |

**Installation:**
```bash
# In the workflow YAML — no install step in the canonical repo.
# In the mirror repo (one-time, by owner):
gh secret set ATLASSIAN_API_KEY --repo reubenjohn/reposix-tokenworld-mirror
gh secret set ATLASSIAN_EMAIL --repo reubenjohn/reposix-tokenworld-mirror
gh secret set REPOSIX_CONFLUENCE_TENANT --repo reubenjohn/reposix-tokenworld-mirror
gh variable set MIRROR_SYNC_CRON --repo reubenjohn/reposix-tokenworld-mirror --body '*/30 * * * *'  # optional override
```

**Version verification:** `cargo binstall` resolves the latest published `reposix-cli` from crates.io; binstall metadata at `crates/reposix-cli/Cargo.toml:19` ties versions to GH release archive filenames. Verified the metadata block exists; I did NOT verify the actual published version on crates.io is current (out of scope — that's release-pipeline territory).
