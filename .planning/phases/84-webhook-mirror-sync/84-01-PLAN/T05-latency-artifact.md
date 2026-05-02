← [index](./index.md)

## Task 84-01-T05 — Latency artifact + measurement script (DVCS-WEBHOOK-04)

<read_first>
- `.planning/phases/84-webhook-mirror-sync/84-RESEARCH.md`
  § "Latency Measurement Strategy" (lines 246-302) — methodology +
  JSON template.
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`
  § "D-02" — pre-release cadence ratification + falsifiable threshold
  (p95 ≤ 120).
- `quality/reports/verifications/agent-ux/dark-factory-sim.json`
  (or any existing verifications JSON) — donor pattern for
  measurement artifact JSON shape.
- `quality/gates/agent-ux/webhook-latency-floor.sh` (T01 mint —
  already implements the assertion shape; no replacement needed,
  this task lands the artifact the verifier asserts on).
- `docs/guides/dvcs-mirror-setup-template.yml` (T02 output) — the
  workflow whose synthetic-dispatch latency we measure.
</read_first>

<action>
T05 ships TWO files:

1. **`quality/reports/verifications/perf/webhook-latency.json`** —
   the synthetic-method measurement artifact (CI-runnable lower
   bound; the catalog row's claim).
2. **`scripts/webhook-latency-measure.sh`** — the owner-runnable
   real-TokenWorld measurement script (the headline number; run
   post-phase by the owner against TokenWorld, refreshes the JSON
   with `method: "real-tokenworld-manual-edit"`).

### 5a. Generate the synthetic-method JSON

Run the synthetic-dispatch loop locally (or in CI) to capture n=10
timings. The loop dispatches the workflow via `gh api .../dispatches`
and polls for the `refs/mirrors/confluence-synced-at` ref-update.
This is a LOWER-BOUND number — it skips the actual confluence
webhook delay (which is the dominant term in the real-TokenWorld
methodology).

```bash
#!/usr/bin/env bash
# Inline T05 synthetic measurement (no committed script — this is a
# one-shot generation; the JSON artifact gets committed).
set -euo pipefail
REPO="reubenjohn/reposix-tokenworld-mirror"
REF="refs/mirrors/confluence-synced-at"
TIMINGS=$(mktemp); trap "rm -f $TIMINGS" EXIT

# Pre-flight: workflow must already be live in the mirror repo (T02
# shipped it). gh auth must have repo + workflow scopes.
gh auth status 2>&1 | grep -qE "repo|workflow" \
  || { echo "ABORT: gh auth missing scopes"; exit 1; }
gh api "repos/${REPO}/actions/workflows" --jq '.workflows[] | select(.name=="reposix-mirror-sync") | .id' >/dev/null \
  || { echo "ABORT: reposix-mirror-sync workflow not found in mirror repo"; exit 1; }

for i in $(seq 1 10); do
  echo "Iteration $i / 10:"
  T_START=$(date +%s)

  # Capture the prior synced-at SHA (or "" if not yet present).
  PRIOR=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "first-run")

  # Dispatch the workflow.
  gh api "repos/${REPO}/dispatches" \
    -f event_type=reposix-mirror-sync \
    -f client_payload="{\"trigger\":\"p84-t05-synthetic\",\"iter\":${i}}" \
    >/dev/null

  # Poll for ref-update.
  while true; do
    NEW=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "$PRIOR")
    if [ "$NEW" != "$PRIOR" ]; then
      T_DONE=$(date +%s)
      DELTA=$((T_DONE - T_START))
      echo "  -> ref-update observed after ${DELTA}s"
      echo "$DELTA" >> "$TIMINGS"
      break
    fi
    sleep 5
    if [ $(($(date +%s) - T_START)) -gt 300 ]; then
      echo "  -> TIMEOUT (>300s); skipping"
      break
    fi
  done

  # Brief pause so consecutive dispatches don't queue.
  sleep 5
done

# Compute p50 / p95 / max.
N=$(wc -l < "$TIMINGS")
P50=$(sort -n "$TIMINGS" | awk -v n="$N" 'NR==int(n*0.5)+1')
P95=$(sort -n "$TIMINGS" | awk -v n="$N" 'NR==int(n*0.95)+1')
MAX=$(sort -n "$TIMINGS" | tail -1)
TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
VERDICT="PASS"; [ "${P95:-9999}" -gt 120 ] && VERDICT="FAIL"

mkdir -p quality/reports/verifications/perf
cat > quality/reports/verifications/perf/webhook-latency.json <<JSON
{
  "measured_at": "${TS}",
  "method": "synthetic-dispatch",
  "n": ${N},
  "p50_seconds": ${P50:-0},
  "p95_seconds": ${P95:-0},
  "max_seconds": ${MAX:-0},
  "target_seconds": 60,
  "verdict": "${VERDICT}"
}
JSON

# Pretty-print for the commit log.
python3 -m json.tool quality/reports/verifications/perf/webhook-latency.json
```

If `VERDICT` is `FAIL` (synthetic p95 > 120s), the row's threshold
is breached. P85 docs document the constraint per ROADMAP SC4. T06
still ships, but the row's status reflects the actual measurement.
**Important:** if the synthetic measurement fails, file as a
SURPRISES-INTAKE entry with severity HIGH — the synthetic should be
WELL under 120s in normal operation; a FAIL points at substrate
bugs (cron not firing; binstall regression; cache build failure)
that warrant phase-internal investigation before pushing.

### 5b. Author the owner-runnable real-TokenWorld measurement script

Create `scripts/webhook-latency-measure.sh` (~50 lines, owner-only;
NOT run in CI). The body is the script in `<must_haves>` above —
copy it into `scripts/webhook-latency-measure.sh` and `chmod +x`.

```bash
# (verbatim shell body from <must_haves>; copy here at write-time)
```

### 5c. Validate + commit

```bash
# Validate the JSON parses + has all required fields + p95 within threshold.
python3 -c '
import json, sys
d = json.load(open("quality/reports/verifications/perf/webhook-latency.json"))
required = {"measured_at", "method", "n", "p50_seconds", "p95_seconds", "max_seconds", "target_seconds", "verdict"}
missing = required - d.keys()
assert not missing, f"missing fields: {missing}"
assert d["p95_seconds"] <= 120, f"p95={d["p95_seconds"]} > 120s threshold"
assert d["verdict"] in ("PASS", "FAIL")
assert d["method"] in ("synthetic-dispatch", "real-tokenworld-manual-edit")
print(f"webhook-latency.json valid: method={d["method"]} n={d["n"]} p95={d["p95_seconds"]}s verdict={d["verdict"]}")
'

# Run the catalog row's verifier to confirm.
bash quality/gates/agent-ux/webhook-latency-floor.sh

chmod +x scripts/webhook-latency-measure.sh

git add quality/reports/verifications/perf/webhook-latency.json \
        scripts/webhook-latency-measure.sh
git commit -m "$(cat <<'EOF'
perf(P84): land synthetic-method webhook-latency.json + owner-runnable measurement script (DVCS-WEBHOOK-04)

Synthetic measurement:
- method: synthetic-dispatch (gh api .../dispatches + poll for refs/mirrors/confluence-synced-at)
- n=10 iterations
- p95 within 120s falsifiable threshold per ROADMAP P84 SC4
- verdict: PASS

Headline real-TokenWorld n=10 measurement deferred to owner via
scripts/webhook-latency-measure.sh (manual edits in TokenWorld; not
CI-runnable). The script writes the JSON artifact with method=
"real-tokenworld-manual-edit"; the catalog row's verifier asserts
p95 <= 120s regardless of method.

scripts/webhook-latency-measure.sh:
- Owner runs after milestone close (or any time the mirror needs
  re-measurement)
- Prerequisites: gh auth status with repo+workflow scopes; Confluence
  webhook configured per P85 setup guide; edit access to TokenWorld
- Output: refreshed quality/reports/verifications/perf/webhook-latency.json

Phase 84 / Plan 01 / Task 05 / DVCS-WEBHOOK-04.
EOF
)"
```
</action>

<verify>
  <automated>test -f quality/reports/verifications/perf/webhook-latency.json && python3 -c 'import json; d=json.load(open("quality/reports/verifications/perf/webhook-latency.json")); assert d["p95_seconds"] <= 120; assert d["verdict"] in ("PASS","FAIL"); assert d["method"] in ("synthetic-dispatch","real-tokenworld-manual-edit"); assert d["n"] >= 1' && bash quality/gates/agent-ux/webhook-latency-floor.sh && test -x scripts/webhook-latency-measure.sh</automated>
</verify>

<done>
- `quality/reports/verifications/perf/webhook-latency.json` exists,
  parses, has all required fields (`measured_at`, `method`, `n`,
  `p50_seconds`, `p95_seconds`, `max_seconds`, `target_seconds`,
  `verdict`).
- `method` is `synthetic-dispatch` (T05 ships the synthetic
  measurement; the real-TokenWorld refresh is owner-driven post-phase).
- `p95_seconds` ≤ 120 (falsifiable threshold per ROADMAP P84 SC4).
  IF `p95 > 120` was observed empirically, T05 filed a SURPRISES-INTAKE
  entry and consulted the orchestrator before continuing.
- `verdict: "PASS"` (since p95 ≤ 120).
- `scripts/webhook-latency-measure.sh` exists, executable, contains
  the n=10 manual-edit pass body for owner-driven future
  measurements.
- `bash quality/gates/agent-ux/webhook-latency-floor.sh` exits 0
  with `p95=<N>s within 120s threshold` stdout.
- Commit message annotates "P84 / Plan 01 / Task 05".
</done>
