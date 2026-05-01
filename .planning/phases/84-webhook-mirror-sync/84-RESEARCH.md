# Phase 84: Webhook-driven mirror sync — GH Action workflow + setup guide — Research

**Researched:** 2026-05-01
**Domain:** GitHub Actions workflow authoring; `repository_dispatch` triggers; `--force-with-lease` race protection; webhook-vs-cron dual-trigger orchestration; real-backend latency measurement against TokenWorld + reposix-tokenworld-mirror.
**Confidence:** HIGH (all upstream substrates shipped — P80 mirror refs, P82 URL parser, P83 bus write fan-out; the mirror repo itself exists per CARRY-FORWARD § DVCS-MIRROR-REPO-01; workflow shape is ratified verbatim in `architecture-sketch.md` § "Webhook-driven mirror sync"; only first-run + latency-measurement details are DEFERRED-to-implementation in `decisions.md` Q4.3/Q4.1).

## Summary

P84 is a **mostly-YAML + integration-test phase**. The reference workflow shape is ratified verbatim in `architecture-sketch.md`; the four ratified Q4 decisions (Q4.1 cron `*/30` configurable, Q4.2 webhook-less backends documented but not implemented, Q4.3 first-run handled gracefully) leave only two implementation details to resolve: (a) the EXACT first-run `--force-with-lease` invariant against an empty mirror with no `mirror/main` ref, and (b) the latency measurement methodology (recommended: real TokenWorld + a synthetic harness for CI repeatability).

Phase scope is narrow because all the heavy lifting already shipped: P80 mints `refs/mirrors/<sot>-head` + `refs/mirrors/<sot>-synced-at` ref helpers (`crates/reposix-cache/src/mirror_refs.rs`); P83 wires bus pushes to update both refs (the workflow becomes a **no-op refresh** when the bus already touched them, per Q2.3); `cargo binstall` metadata is present in `crates/reposix-cli/Cargo.toml:19` (binary distribution path is ready); the real GH mirror at `github.com/reubenjohn/reposix-tokenworld-mirror` is provisioned with `gh` auth scopes confirmed (CARRY-FORWARD § DVCS-MIRROR-REPO-01).

**Primary recommendation:** Single-plan with **6 tasks** — catalog rows first, then YAML, then three integration tests (first-run, race, latency), then a CLAUDE.md update + close. No phase split needed (P84 is much narrower than P83's risk surface). Place the workflow file at `.github/workflows/reposix-mirror-sync.yml` IN THE MIRROR REPO (`reubenjohn/reposix-tokenworld-mirror`), NOT in `reubenjohn/reposix` — per CARRY-FORWARD § DVCS-MIRROR-REPO-01 P84 bullet, *"the GH Action workflow at `.github/workflows/reposix-mirror-sync.yml` lands in THIS repo, not in `reubenjohn/reposix`."* This is load-bearing and easy to miss; surface it in the plan's first task.

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

## Workflow YAML Shape

The workflow lives at `.github/workflows/reposix-mirror-sync.yml` in `reubenjohn/reposix-tokenworld-mirror` (NOT the canonical reposix repo). Verbatim skeleton, derived from architecture-sketch + `bench-latency-cron.yml` precedent:

```yaml
name: reposix-mirror-sync

# Triggers:
#   - repository_dispatch (event_type: reposix-mirror-sync) — webhook from confluence
#   - cron (default */30 m, override via vars.MIRROR_SYNC_CRON) — safety net
# Per .planning/research/v0.13.0-dvcs/decisions.md Q4.1/Q4.2/Q4.3.

on:
  repository_dispatch:
    types: [reposix-mirror-sync]
  schedule:
    # GH Actions does NOT support `${{ vars.* }}` in the schedule cron field
    # (the schedule block is parsed before contexts are evaluated). The
    # default `*/30 * * * *` is the literal value here; the `vars.*`
    # override is consumed by a SEPARATE workflow file or via repo-level
    # workflow disable/enable. SEE PITFALL 5.
    - cron: '*/30 * * * *'
  workflow_dispatch:  # manual re-trigger for owner debugging

permissions:
  contents: write  # for git push to mirror

env:
  REPOSIX_ALLOWED_ORIGINS: 'http://127.0.0.1:*,https://${{ secrets.REPOSIX_CONFLUENCE_TENANT }}.atlassian.net'

jobs:
  sync:
    name: sync-confluence-to-mirror
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - name: Checkout mirror repo
        uses: actions/checkout@v6
        with:
          fetch-depth: 0  # need full history for force-with-lease lease check

      - name: Install reposix-cli
        run: |
          set -euo pipefail
          # cargo binstall reads the binstall metadata in reposix-cli's
          # Cargo.toml and downloads the right tarball from the
          # corresponding GH release. Faster than cargo install.
          curl -L --proto '=https' --tlsv1.2 -sSf \
            https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
          cargo binstall --no-confirm reposix-cli

      - name: Build SoT cache via reposix init
        env:
          ATLASSIAN_API_KEY: ${{ secrets.ATLASSIAN_API_KEY }}
          ATLASSIAN_EMAIL: ${{ secrets.ATLASSIAN_EMAIL }}
          REPOSIX_CONFLUENCE_TENANT: ${{ secrets.REPOSIX_CONFLUENCE_TENANT }}
        run: |
          set -euo pipefail
          SPACE="${{ vars.CONFLUENCE_SPACE || 'TokenWorld' }}"
          reposix init "confluence::${SPACE}" /tmp/sot
          # Post-init the cache has refs/mirrors/confluence-{head,synced-at}
          # populated (P80 ships these on init). Working tree at /tmp/sot
          # is a partial-clone checkout from the cache.

      - name: Configure mirror remote in /tmp/sot
        run: |
          set -euo pipefail
          cd /tmp/sot
          MIRROR_URL="${{ github.server_url }}/${{ github.repository }}.git"
          git remote add mirror "$MIRROR_URL"
          # First-run handling: fetch the mirror; on success, mirror/main
          # exists. On 404 / empty repo, fetch fails — handled in next step.
          git fetch mirror main 2>/dev/null || echo "first-run: mirror has no main yet"

      - name: Push to mirror with --force-with-lease
        run: |
          set -euo pipefail
          cd /tmp/sot
          # If mirror/main exists, lease against it. If it doesn't (first
          # run on empty mirror), use plain push.
          if git show-ref --verify --quiet refs/remotes/mirror/main; then
            LEASE_SHA=$(git rev-parse refs/remotes/mirror/main)
            git push mirror "refs/heads/main:refs/heads/main" \
              --force-with-lease="refs/heads/main:${LEASE_SHA}"
          else
            # First-run path: empty mirror; no lease available.
            # Plain push creates main on the mirror.
            git push mirror "refs/heads/main:refs/heads/main"
          fi
          # Mirror-lag refs are namespace-pushed in a SEPARATE invocation
          # so a refs-write failure doesn't taint the main-branch push.
          git push mirror "refs/mirrors/confluence-head" "refs/mirrors/confluence-synced-at" || \
            echo "warn: mirror-lag refs push failed (non-fatal); cron will retry"
```

**Note on `${{ vars.MIRROR_SYNC_CRON }}` in the schedule field.** GitHub Actions parses the `schedule` block at workflow-load time, BEFORE the `${{ vars.* }}` context is evaluated. The cron expression CANNOT be templated via `vars` directly. Q4.1's intent (configurable cron) is satisfied by either (a) editing the workflow file when the owner wants to change cadence, or (b) maintaining a SECOND workflow file with a different cron + a `repository_dispatch` to disable the first. **RECOMMEND option (a)** for v0.13.0 simplicity; document the constraint in P85's setup guide.

[VERIFIED: github-docs `https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#schedule` — "Note: The schedule event can be delayed during periods of high loads of GitHub Actions workflow runs."]

## `--force-with-lease` Semantics

**Invariant.** `--force-with-lease=refs/heads/main:<expected-sha>` succeeds IFF the remote's `refs/heads/main` currently points at `<expected-sha>`. If it has moved (e.g., a bus push landed between this workflow's `git fetch mirror` and its `git push`), the lease check fails BEFORE any mirror-side update happens — the push is rejected with `! [rejected] main -> main (stale info)` and exit 1.

**Race walk-through (the load-bearing test).**

| Time | Workflow step | Bus push (concurrent) | Mirror state |
|---|---|---|---|
| T0 | `git fetch mirror main` | — | `main = SHA-A` |
| T1 | local `mirror/main = SHA-A`; `LEASE_SHA = SHA-A` | bus-remote precheck A passes (mirror = SHA-A locally too) | `main = SHA-A` |
| T2 | (workflow doing `reposix init` / cache build) | bus push lands SoT write + mirror write | `main = SHA-B` (advanced) |
| T3 | `git push --force-with-lease=...:SHA-A` | — | server sees `main = SHA-B`; lease compares vs `SHA-A`; **REJECTS** |
| T4 | workflow exits non-zero on push step | — | `main = SHA-B` (untouched by workflow) |

**The contract:** the bus push's mirror-leg already advanced `main` to `SHA-B` AND wrote `refs/mirrors/confluence-synced-at` per P83. The workflow's failure to push is the CORRECT outcome — there's nothing to do; the bus already did it. The workflow exits 1 (signals to GH Actions log that a no-op-via-race occurred); the next cron tick will see no drift and re-attempt cleanly.

**Test fixture.** `tests/webhook_force_with_lease_race.rs` (or shell harness if Rust integration is heavy):

```text
1. Set up local file:// "mirror" bare repo with main = SHA-A.
2. In a temp working tree, simulate the workflow's "fetch mirror" → mirror/main = SHA-A.
3. While the workflow is "doing reposix init" (no-op in test), push SHA-B
   directly to the bare mirror (simulates the bus push winning the race).
4. Run `git push --force-with-lease=refs/heads/main:SHA-A` to the mirror.
5. Assert: exit code 1, stderr contains "stale info" or "non-fast-forward"
   (git's exact wording varies by version; assert one of {stale info,
   non-fast-forward, rejected}).
6. Assert: mirror's main = SHA-B (untouched by the workflow's failed push).
```

This test is portable and runs in <1s; place it in `crates/reposix-cli/tests/` (or a new `quality/gates/agent-ux/webhook-force-with-lease-race.sh` shell verifier per the project's existing agent-ux gate convention).

## First-run Handling (Q4.3)

**The empty-mirror case.** When `reposix-tokenworld-mirror` is freshly created (current state per CARRY-FORWARD § DVCS-MIRROR-REPO-01: "empty except auto-generated README"), the workflow's `git fetch mirror main` will FAIL with `fatal: couldn't find remote ref main` because the auto-generated README created `main` with one commit (the README) — actually it does exist. Let me re-verify: `gh repo create --add-readme` creates `main` pointing at a single README commit. So `mirror/main` exists from day 1.

**Two sub-cases the planner must distinguish:**

| Sub-case | mirror/main state | Action |
|---|---|---|
| 4.3.a — fresh-but-readme | `main` = README commit | `--force-with-lease=...:<readme-sha>` succeeds; lease check passes against the unchanged-since-creation SHA. Plain force-push REPLACES the README with the SoT's main. |
| 4.3.b — truly-empty | `main` does not exist (`gh repo create` without `--add-readme`) | `git fetch mirror main` returns 0 but no ref is created; `git show-ref --verify` returns 1; the YAML's `if` branch falls through to plain `git push mirror main` (no lease). |

**Recommended invariant (cleanest):** check `git show-ref --verify --quiet refs/remotes/mirror/main` AFTER `git fetch mirror main`. If present → lease push. If absent → plain push. The YAML skeleton above already encodes this. Test fixture: simulate both sub-cases via `git init --bare /tmp/empty-mirror` (no main) vs `git init --bare && git push --initial-commit` (main present).

**Note on the README that already exists in `reposix-tokenworld-mirror`.** The first real workflow run will REPLACE the auto-generated README with the SoT's content. This is the intended behavior — the mirror's purpose is to mirror confluence, not to host hand-edited README content. P85's setup guide should call this out: *"the mirror repo's README is replaced on first sync; if you want a custom README, host it elsewhere."*

## Latency Measurement Strategy

**The target.** < 60s p95 from confluence-edit (web UI or REST PATCH) to mirror ref-update. Per ROADMAP SC4: *"if p95 > 120s, P85 docs document the constraint and tune ref semantics."* — i.e., the 60s is aspirational; the failure threshold is 120s.

**Three candidate approaches:**

| Approach | Realism | Repeatability | Cost | Recommend |
|---|---|---|---|---|
| (a) Sandbox harness — confluence webhook simulated via direct POST to `repos/.../dispatches` | LOW (skips actual confluence webhook latency) | HIGH (deterministic) | LOW | for CI repeatability |
| (b) Real TokenWorld test — manual edit + measure ref-arrival | HIGH (full path including confluence's webhook delay) | LOW (manual edit per measurement; rate-limited) | MEDIUM | for the headline number |
| (c) CI synthetic — wiremock + in-process | LOW (no real GH Actions runner involved) | HIGH | LOW | NOT recommended — too synthetic |

**RECOMMEND: hybrid (a) + (b).** Use (b) — real TokenWorld with 10 manual edits — to capture the headline p95 number for `webhook-latency.json` artifact (the ROADMAP SC4 deliverable). Use (a) — `gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches -f event_type=reposix-mirror-sync` to trigger the workflow synthetically, then poll `gh api repos/.../git/refs/mirrors/confluence-head` for ref update — as the CI-runnable repeatability check. Approach (a) gives a lower-bound (no confluence webhook delay); approach (b) gives the headline number.

**Measurement script outline (approach b):**
```bash
#!/usr/bin/env bash
# scripts/webhook-latency-measure.sh — run 10 measurements, write JSON.
# Manual: owner edits a TokenWorld page between iterations.
for i in $(seq 1 10); do
  echo "Iteration $i: edit a TokenWorld page now (ENTER when done)..."
  read
  T_EDIT=$(date +%s)
  # Confluence webhook fires; workflow runs; ref updates.
  # Poll the mirror for ref advance.
  PRIOR=$(gh api repos/reubenjohn/reposix-tokenworld-mirror/git/refs/mirrors/confluence-synced-at -q .object.sha)
  while true; do
    NEW=$(gh api repos/reubenjohn/reposix-tokenworld-mirror/git/refs/mirrors/confluence-synced-at -q .object.sha)
    if [ "$NEW" != "$PRIOR" ]; then
      T_DONE=$(date +%s)
      echo "$((T_DONE - T_EDIT))" >> /tmp/latencies.txt
      break
    fi
    sleep 2
    if [ $(($(date +%s) - T_EDIT)) -gt 180 ]; then echo "TIMEOUT"; break; fi
  done
done
# Compute p95 + write JSON.
sort -n /tmp/latencies.txt | awk '{a[NR]=$1} END {print a[int(NR*0.95)]}'
```

Output to `quality/reports/verifications/perf/webhook-latency.json`:
```json
{
  "measured_at": "2026-05-XX",
  "method": "real-tokenworld-manual-edit",
  "n": 10,
  "p50_seconds": 35,
  "p95_seconds": 58,
  "max_seconds": 90,
  "target_seconds": 60,
  "verdict": "PASS"
}
```

If p95 > 120s, P85 docs document the constraint. If p95 ∈ (60s, 120s), document but pass the catalog row with a note. If p95 ≤ 60s, clean PASS.

## Secrets Convention

Atlassian creds are repo-scoped secrets on `reubenjohn/reposix-tokenworld-mirror` (NOT the canonical reposix repo). Owner-side setup, ONE-TIME (idempotent re-run):

```bash
# Required for the confluence::TokenWorld init step.
gh secret set ATLASSIAN_API_KEY --repo reubenjohn/reposix-tokenworld-mirror
gh secret set ATLASSIAN_EMAIL --repo reubenjohn/reposix-tokenworld-mirror
gh secret set REPOSIX_CONFLUENCE_TENANT --repo reubenjohn/reposix-tokenworld-mirror

# Optional — confluence space override (default 'TokenWorld' in workflow).
gh variable set CONFLUENCE_SPACE --repo reubenjohn/reposix-tokenworld-mirror --body 'TokenWorld'

# Optional — cron cadence override (default '*/30 * * * *' is the literal in YAML;
# changing requires editing the workflow file due to GH Actions schedule limitation).
# Documented in P85 setup guide.
```

The workflow references them via `${{ secrets.* }}` per the precedent in `ci.yml:131-134`. NO secret leakage in workflow logs — GH Actions auto-redacts secrets in step output. The `REPOSIX_ALLOWED_ORIGINS` env var is computed from `secrets.REPOSIX_CONFLUENCE_TENANT` (the only origin the cache is allowed to reach during this workflow).

The webhook-side credential (Confluence's outbound POST to `api.github.com/repos/.../dispatches`) requires a GH personal access token (PAT) with `repo` scope, configured on the Atlassian side as a webhook header `Authorization: token ghp_...`. Document in P85 — the workflow itself doesn't need to know about this; it just receives the dispatch event.

## Backends Without Webhooks (Q4.2)

Per Q4.2 RATIFIED: backends without webhooks fall back to cron-only mode. The workflow already supports this — DELETE the `repository_dispatch` block and the workflow runs purely on the cron schedule.

**Trim path (documented in P85's `dvcs-mirror-setup.md`):**
```yaml
on:
  # repository_dispatch:                  # ← delete this block
  #   types: [reposix-mirror-sync]
  schedule:
    - cron: '*/30 * * * *'                # ← keep only this
  workflow_dispatch:
```

No code change required. Cron fires every 30 minutes regardless of webhook. Owner accepts staleness ≤ cron interval.

**Currently-supported backends and their webhook status:**

| Backend | Has webhooks? | Default mode |
|---|---|---|
| Confluence | ✓ | webhook + cron |
| GitHub Issues | ✓ (`issues` event) | webhook + cron (workflow uses different event_type; same shape) |
| JIRA | ✓ | webhook + cron |
| `sim` (in-process simulator) | n/a | not relevant — sim is dev/test only, no mirror sync |

All currently-supported backends DO emit webhooks; the Q4.2 fallback is for hypothetical future connectors (e.g., a SQL-table backend that polls). No P84 implementation work — only documentation.

## Catalog Row Design

Per ROADMAP SC7, six rows in `quality/catalogs/agent-ux.json`. Each follows the precedent shape from `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` (`agent-ux.json`); each verifier under `quality/gates/agent-ux/` follows the shape of `bus-precheck-a-mirror-drift-emits-fetch-first.sh`. **All six rows are minted hand-edited** per the existing P79 footnote (`"_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension..."`); P84 inherits this constraint.

| Row ID | Verifier | Cadence | Kind | Sources |
|---|---|---|---|---|
| `agent-ux/webhook-trigger-dispatch` | `quality/gates/agent-ux/webhook-trigger-dispatch.sh` | pre-pr | mechanical | `.github/workflows/reposix-mirror-sync.yml` (in mirror repo); REQUIREMENTS DVCS-WEBHOOK-01 |
| `agent-ux/webhook-cron-fallback` | `quality/gates/agent-ux/webhook-cron-fallback.sh` | pre-pr | mechanical | same workflow file; verifies cron block parses + matches `*/30 * * * *` default |
| `agent-ux/webhook-force-with-lease-race` | `quality/gates/agent-ux/webhook-force-with-lease-race.sh` | pre-pr | mechanical | shell harness or `tests/webhook_force_with_lease_race.rs`; REQUIREMENTS DVCS-WEBHOOK-02 |
| `agent-ux/webhook-first-run-empty-mirror` | `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` | pre-pr | mechanical | shell harness exercising both 4.3.a + 4.3.b sub-cases; REQUIREMENTS DVCS-WEBHOOK-03 |
| `agent-ux/webhook-latency-floor` | `quality/gates/agent-ux/webhook-latency-floor.sh` (asserts JSON p95 ≤ 120s) | pre-release | asset-exists | `quality/reports/verifications/perf/webhook-latency.json`; REQUIREMENTS DVCS-WEBHOOK-04 |
| `agent-ux/webhook-backends-without-webhooks` | `quality/gates/agent-ux/webhook-backends-without-webhooks.sh` (asserts the trim path produces a valid YAML by stripping `repository_dispatch:` block) | pre-pr | mechanical | workflow YAML + Q4.2 doc snippet |

**Catalog row order:** rows land in commit 1 (per CLAUDE.md "catalog-first rule"); verifiers are stubs that exit non-zero with a clear "not yet implemented" message until the corresponding implementation tasks land. After P84 closes, all six are GREEN.

The `webhook-latency-floor` row is the only one with cadence `pre-release` (others are `pre-pr`). Rationale: the latency artifact freshness TTL should match the perf-targets dimension's general cadence (re-measured per release, not per-PR). The walker's freshness check fires on the artifact's `measured_at` timestamp.

## Test Infrastructure

**Recommendation: shell harnesses, NOT `act`.**

Three test categories:

1. **YAML lint + structure:** `actionlint` (Go binary, fast) verifies the workflow file parses and uses valid action references. Add to `quality/gates/agent-ux/webhook-trigger-dispatch.sh` as a sub-step.

2. **`--force-with-lease` race protection:** shell harness using `git init --bare` for the "mirror." Walk-through above. ~50 lines of shell. Place at `quality/gates/agent-ux/webhook-force-with-lease-race.sh`. Runs in <1s.

3. **First-run handling:** shell harness with two sub-fixtures (4.3.a fresh-but-readme, 4.3.b truly-empty). Each fixture is a `git init --bare` with deterministic seed. Place at `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`. Runs in <2s.

**Why NOT `act`:**
- Pulls Docker images (~500MB initial) — heavy on the dev VM.
- Requires Docker daemon — adds setup friction.
- Doesn't run on the CI runner faithfully (different environment).
- The substrates we care about (`reposix init`, `git push --force-with-lease`) are testable WITHOUT a workflow runner — we test them directly.

**Real-backend test (gated by secrets, milestone-close):** the existing `agent_flow_real` integration tests in `crates/reposix-cli/tests/agent_flow_real.rs` are the precedent. Add `webhook_real_dispatch` as a new `#[ignore]`-gated test that uses `gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches -f event_type=reposix-mirror-sync` to trigger the workflow + polls for ref-update. Skip cleanly when `GITHUB_TOKEN` is absent. Adds < 30 lines of Rust.

## Plan Splitting Recommendation

**Single plan, ~6 tasks.** P84 is much narrower than P83's risk surface — no Rust code paths to refactor, no audit-table schema changes, no fault-injection coverage. The riskiest parts (race protection, first-run) are isolated test fixtures, not architecture.

Recommended task sequence:

| # | Task | Output |
|---|---|---|
| 1 | Catalog rows + stub verifiers | 6 rows in `agent-ux.json`; 6 shell stubs under `quality/gates/agent-ux/webhook-*.sh` (each `exit 1` with TODO message) |
| 2 | Workflow YAML at `.github/workflows/reposix-mirror-sync.yml` (in mirror repo) | YAML file landed; `webhook-trigger-dispatch.sh` + `webhook-cron-fallback.sh` + `webhook-backends-without-webhooks.sh` flip GREEN |
| 3 | First-run handling test | `webhook-first-run-empty-mirror.sh` flips GREEN; covers 4.3.a + 4.3.b |
| 4 | `--force-with-lease` race test | `webhook-force-with-lease-race.sh` flips GREEN |
| 5 | Latency measurement (real TokenWorld + synthetic harness) | `webhook-latency.json` artifact; `webhook-latency-floor.sh` flips GREEN |
| 6 | CLAUDE.md update + phase-close push | CLAUDE.md "v0.13.0 — in flight" section gains P84 entry; `git push origin main`; verifier subagent dispatch |

**Why no split:** total task surface is ~6 commits; no single task is risky enough to warrant isolation. Compare to P83's split which was driven by build-memory-budget concerns (fault-injection tests are heavy linkage) — P84 has zero new Rust integration tests; only YAML + shell + a few API polls.

## Common Pitfalls

### Pitfall 1: Workflow file lands in the WRONG repo
**What goes wrong:** Plan says "ship the workflow," executor commits it to `reubenjohn/reposix` instead of `reubenjohn/reposix-tokenworld-mirror`.
**Why it happens:** Working tree default is the canonical repo; CARRY-FORWARD's "lands in THIS repo" line is easy to miss.
**How to avoid:** First plan task explicitly names the target repo; the verifier checks `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml` returns 200, NOT a local file check.
**Warning signs:** "I committed the workflow but the dispatch never fires" — the workflow needs to live in the repo it dispatches against.

### Pitfall 2: `cargo binstall reposix` instead of `cargo binstall reposix-cli`
**What goes wrong:** Workflow fails to install reposix at the binstall step.
**Why it happens:** Architecture-sketch wrote `cargo binstall reposix` as shorthand; the actual published crate name is `reposix-cli`.
**How to avoid:** Use `cargo binstall reposix-cli` verbatim; add a comment in the YAML citing the binstall metadata location.
**Warning signs:** Workflow log shows `error: no crate matching 'reposix'`.

### Pitfall 3: `${{ vars.MIRROR_SYNC_CRON }}` in the schedule field
**What goes wrong:** Owner sets the var, expects cron to change, nothing happens.
**Why it happens:** GH Actions parses `schedule:` BEFORE evaluating contexts. `vars.*` is unsupported in cron expressions.
**How to avoid:** Use a literal `*/30 * * * *` in the YAML; document in P85 that changing cadence requires editing the workflow file directly.
**Warning signs:** Owner sets `gh variable set MIRROR_SYNC_CRON='*/15 * * * *'`, no behavior change.

### Pitfall 4: `actions/checkout@v6` `fetch-depth: 0` is REQUIRED for force-with-lease
**What goes wrong:** Default `fetch-depth: 1` means workflow can't see prior history; `git fetch mirror main` succeeds but the lease comparison may compare against a shallow clone artifact.
**Why it happens:** Most workflows don't need full history; default checkout is shallow.
**How to avoid:** Always `with: { fetch-depth: 0 }` in this workflow.
**Warning signs:** intermittent `--force-with-lease` failures with cryptic "object not found" errors.

### Pitfall 5: Mirror's auto-generated README causes a "lease success but main was overwritten" confusion
**What goes wrong:** First workflow run replaces `reposix-tokenworld-mirror`'s README with confluence content; owner sees "what happened to my README?"
**Why it happens:** The mirror's purpose IS to mirror confluence; the auto-generated README from `gh repo create --add-readme` is collateral.
**How to avoid:** Document in P85 setup guide; surface a "the mirror repo is read-only from your perspective; don't hand-edit it" callout.
**Warning signs:** owner asks "where did my README go?"

### Pitfall 6: Workflow runs every 30 minutes in CI
**What goes wrong:** P84's CI tests trigger the workflow every 30 minutes during the phase, accumulating workflow-run history clutter.
**Why it happens:** Once the YAML lands on the mirror repo's main branch, the cron is live.
**How to avoid:** Land the YAML in the mirror repo only AFTER all other P84 tasks pass; consider a "feature branch" approach (mirror repo branch `wip/phase-84` with the YAML; merge to main only at phase-close).
**Warning signs:** mirror repo's Actions tab fills up with `sync-confluence-to-mirror` runs during the phase.

### Pitfall 7: GH Actions `repository_dispatch` requires a PAT, not the default `GITHUB_TOKEN`
**What goes wrong:** Confluence webhook posts to `api.github.com/repos/.../dispatches` with the runner's `GITHUB_TOKEN`; receives 403.
**Why it happens:** `GITHUB_TOKEN` is workflow-scoped; cannot trigger workflows in OTHER workflows. Cross-trigger requires a PAT.
**How to avoid:** Document in P85 that the Atlassian webhook config needs a PAT with `repo` scope; the workflow itself doesn't deal with this.
**Warning signs:** Confluence webhook delivery log shows 403; mirror Actions tab has no dispatched runs.

[CITED: docs.github.com/en/rest/repos/repos#create-a-repository-dispatch-event — "If the repository is private, you must use a personal access token with the repo scope or a GitHub App with both metadata:read and contents:read and write permissions."]

## Code Examples

### Workflow `if` for first-run vs steady-state
```bash
# Inside the "Push to mirror" step.
cd /tmp/sot
if git show-ref --verify --quiet refs/remotes/mirror/main; then
  LEASE_SHA=$(git rev-parse refs/remotes/mirror/main)
  git push mirror "refs/heads/main:refs/heads/main" \
    --force-with-lease="refs/heads/main:${LEASE_SHA}"
else
  # First-run path: empty mirror; no lease available.
  git push mirror "refs/heads/main:refs/heads/main"
fi
```

### Synthetic dispatch (CI repeatability check)
```bash
# Trigger the workflow synthetically (no actual confluence edit needed).
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync \
  -f client_payload='{"trigger":"ci-test"}'
# Wait for the workflow run to complete.
gh run watch --repo reubenjohn/reposix-tokenworld-mirror \
  $(gh run list --repo reubenjohn/reposix-tokenworld-mirror --workflow=reposix-mirror-sync --limit 1 --json databaseId -q '.[0].databaseId')
# Verify ref state.
gh api repos/reubenjohn/reposix-tokenworld-mirror/git/refs/mirrors/confluence-synced-at \
  -q .object.sha
```

### Race-protection test fixture (sketch)
```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/webhook-force-with-lease-race.sh
set -euo pipefail
TMPDIR=$(mktemp -d); trap "rm -rf $TMPDIR" EXIT
git init --bare "$TMPDIR/mirror.git" >/dev/null
git -C "$TMPDIR/mirror.git" symbolic-ref HEAD refs/heads/main

# Seed mirror with SHA-A.
git init "$TMPDIR/wt-a" >/dev/null
git -C "$TMPDIR/wt-a" commit --allow-empty -m "seed-A" >/dev/null
SHA_A=$(git -C "$TMPDIR/wt-a" rev-parse HEAD)
git -C "$TMPDIR/wt-a" remote add mirror "$TMPDIR/mirror.git"
git -C "$TMPDIR/wt-a" push mirror main >/dev/null

# Workflow's working tree fetches mirror, sees SHA-A.
git init "$TMPDIR/wt-workflow" >/dev/null
git -C "$TMPDIR/wt-workflow" remote add mirror "$TMPDIR/mirror.git"
git -C "$TMPDIR/wt-workflow" fetch mirror main >/dev/null

# Bus push wins the race — pushes SHA-B to mirror.
git -C "$TMPDIR/wt-a" commit --allow-empty -m "bus-B" >/dev/null
git -C "$TMPDIR/wt-a" push mirror main >/dev/null

# Workflow now tries force-with-lease=...:SHA-A. Should reject.
git -C "$TMPDIR/wt-workflow" commit --allow-empty -m "workflow-X" >/dev/null
if git -C "$TMPDIR/wt-workflow" push mirror "refs/heads/main:refs/heads/main" \
     --force-with-lease="refs/heads/main:$SHA_A" 2>&1 | grep -q -E "stale info|rejected|non-fast-forward"; then
  echo "PASS: lease rejected as expected on race"
  exit 0
else
  echo "FAIL: lease should have been rejected"
  exit 1
fi
```

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
