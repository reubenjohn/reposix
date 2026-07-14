# Phase 84 Research — Latency Measurement, Secrets Convention, Backends Without Webhooks

← [back to index](./index.md)

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
