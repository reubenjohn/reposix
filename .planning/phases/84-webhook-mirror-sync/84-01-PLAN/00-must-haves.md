← [index](./index.md)

# Must-Haves

<must_haves>
**Workflow YAML (T02)** — `docs/guides/dvcs-mirror-setup-template.yml`
(canonical repo, new file) AND `.github/workflows/reposix-mirror-sync.yml`
(in `reubenjohn/reposix-tokenworld-mirror`, new file) — byte-equal
modulo whitespace per D-08:

- Header comment block: 5-line tldr per D-03 + forward-link to P85
  (`docs/guides/dvcs-mirror-setup.md`).
- `name: reposix-mirror-sync`.
- Top-level `on:` block:
  - `repository_dispatch:` with `types: [reposix-mirror-sync]`.
  - `schedule:` with `- cron: '*/30 * * * *'` (literal per D-06).
  - `workflow_dispatch:` for manual re-trigger.
- Top-level `concurrency:` block per D-01:
  ```yaml
  concurrency:
    group: reposix-mirror-sync
    cancel-in-progress: false
  ```
- Top-level `permissions:` block:
  ```yaml
  permissions:
    contents: write
  ```
- Top-level `env:` block:
  ```yaml
  env:
    REPOSIX_ALLOWED_ORIGINS: 'http://127.0.0.1:*,https://${{ secrets.REPOSIX_CONFLUENCE_TENANT }}.atlassian.net'
  ```
- `jobs.sync`:
  - `runs-on: ubuntu-latest`.
  - `timeout-minutes: 10`.
  - `steps:`:
    1. **Checkout mirror repo** — `uses: actions/checkout@v6`
       with `fetch-depth: 0` (D-04 / Pitfall 4).
    2. **Install reposix-cli** — `run:` block that curls the binstall
       installer + runs `cargo binstall --no-confirm reposix-cli`
       (D-05; NOT `reposix`).
    3. **Build SoT cache via `reposix init`** — `env:` block carries
       `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
       `REPOSIX_CONFLUENCE_TENANT` from `${{ secrets.* }}`; `run:`
       executes `reposix init "confluence::${SPACE}" /tmp/sot`
       where `SPACE` is `${{ vars.CONFLUENCE_SPACE || 'TokenWorld' }}`.
    4. **Configure mirror remote in `/tmp/sot`** — `run:` block
       does `git remote add mirror "$MIRROR_URL"` where
       `MIRROR_URL="${{ github.server_url }}/${{ github.repository }}.git"`,
       then `git fetch mirror main 2>/dev/null || echo "first-run:
       mirror has no main yet"` (Q4.3 graceful-fail).
    5. **Push to mirror with `--force-with-lease`** — `run:` block:
       ```bash
       set -euo pipefail
       cd /tmp/sot
       if git show-ref --verify --quiet refs/remotes/mirror/main; then
         LEASE_SHA=$(git rev-parse refs/remotes/mirror/main)
         git push mirror "refs/heads/main:refs/heads/main" \
           --force-with-lease="refs/heads/main:${LEASE_SHA}"
       else
         git push mirror "refs/heads/main:refs/heads/main"
       fi
       git push mirror "refs/mirrors/confluence-head" \
                       "refs/mirrors/confluence-synced-at" || \
         echo "warn: mirror-lag refs push failed (non-fatal); cron will retry"
       ```
- NO interpolation of `${{ github.event.client_payload.* }}`
  anywhere in the YAML (S2 / T-84-01 mitigation).
- NO `set -x` in any `run:` block (T-84-02 mitigation).
- The two copies are byte-equal modulo whitespace; verifier asserts
  via `diff -w`.

**Catalog rows + verifiers (T01)** — 6 rows + 6 TINY shells:
- All 6 rows live in `quality/catalogs/agent-ux.json` (D-04).
- Initial status `FAIL` (verifiers exist but the substrate they grep
  for doesn't yet — flips at T06). Hand-edited per documented gap
  (NOT Principle A) — same shape as P79–P83 row annotations,
  citing GOOD-TO-HAVES-01.
- 6 TINY shells under `quality/gates/agent-ux/` (each ~30-50 lines,
  delegating to grep / `gh api` / shell-harness invocation per row
  type).
- All 6 rows flip FAIL → PASS via the runner during T06 BEFORE the
  per-phase push commits.

Row IDs:
1. `agent-ux/webhook-trigger-dispatch` →
   `quality/gates/agent-ux/webhook-trigger-dispatch.sh`
   (cadence: pre-pr; kind: mechanical; asserts BOTH copies exist +
   YAML structure invariants per D-05/D-08; verbatim asserts in T01.)
2. `agent-ux/webhook-cron-fallback` →
   `quality/gates/agent-ux/webhook-cron-fallback.sh`
   (cadence: pre-pr; kind: mechanical; asserts literal cron + the
   `fetch-depth: 0` checkout invariant + the `concurrency:` block
   shape.)
3. `agent-ux/webhook-force-with-lease-race` →
   `quality/gates/agent-ux/webhook-force-with-lease-race.sh`
   (cadence: pre-pr; kind: mechanical; runs the file:// bare-repo
   harness from T04 and asserts rejection-on-race.)
4. `agent-ux/webhook-first-run-empty-mirror` →
   `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`
   (cadence: pre-pr; kind: mechanical; runs the file:// bare-repo
   harness from T03 with both 4.3.a and 4.3.b sub-cases.)
5. `agent-ux/webhook-backends-without-webhooks` →
   `quality/gates/agent-ux/webhook-backends-without-webhooks.sh`
   (cadence: pre-pr; kind: mechanical; asserts the trim-path —
   removing the `repository_dispatch:` block from the YAML produces
   still-valid YAML — by simulating the trim and re-parsing.)
6. `agent-ux/webhook-latency-floor` →
   `quality/gates/agent-ux/webhook-latency-floor.sh`
   (cadence: **pre-release**; kind: asset-exists; asserts the JSON
   artifact `quality/reports/verifications/perf/webhook-latency.json`
   exists, parses, and `p95_seconds ≤ 120`.)

**T03 first-run shell harness** —
`quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` (~80
lines):
- Two sub-fixtures in one harness:
  - **4.3.a (fresh-but-readme):** `git init --bare /tmp/mirror-a.git
    + git push --initial-commit` (a working tree pushes one README
    commit). Workflow tree fetches mirror; `mirror/main` IS present.
    Lease-push branch fires; assert `git push --force-with-lease=...`
    succeeds; mirror's `main` advances from README SHA to workflow's
    SHA.
  - **4.3.b (truly-empty):** `git init --bare /tmp/mirror-b.git`
    only; no `main` ref pushed. Workflow tree fetches mirror; the
    fetch returns 0 but no ref is created. `git show-ref --verify
    --quiet refs/remotes/mirror/main` returns 1; plain-push branch
    fires; assert `git push mirror main` succeeds; mirror's `main`
    is created at workflow's SHA.
- Both sub-fixtures run in <2s wall-clock; trap-cleanup of temp
  dirs.
- Harness exits 0 ONLY if BOTH sub-cases pass; exits 1 with a
  diagnostic naming the failing sub-case otherwise.

**T04 race shell harness** —
`quality/gates/agent-ux/webhook-force-with-lease-race.sh` (~50
lines, verbatim from RESEARCH.md § "Code Examples" race-protection
test fixture sketch):
- `mktemp -d` for `$TMPDIR`; trap-cleanup on exit.
- `git init --bare "$TMPDIR/mirror.git"` + `symbolic-ref HEAD
  refs/heads/main`.
- Seed mirror with SHA-A: `git init "$TMPDIR/wt-a"`; `commit
  --allow-empty`; capture `SHA_A=$(git rev-parse HEAD)`; `git
  remote add mirror "$TMPDIR/mirror.git"`; `git push mirror main`.
- Workflow's tree fetches mirror: `git init "$TMPDIR/wt-workflow"`;
  `git remote add mirror "$TMPDIR/mirror.git"`; `git fetch mirror main`.
- Bus push wins race: in `wt-a`, `commit --allow-empty -m bus-B`;
  `git push mirror main`. Mirror is now at SHA-B.
- Workflow's tree attempts the lease push:
  ```bash
  git -C "$TMPDIR/wt-workflow" commit --allow-empty -m "workflow-X"
  if git -C "$TMPDIR/wt-workflow" push mirror "refs/heads/main:refs/heads/main" \
       --force-with-lease="refs/heads/main:$SHA_A" 2>&1 \
       | grep -q -E "stale info|rejected|non-fast-forward"; then
    echo "PASS: lease rejected as expected on race"
    exit 0
  else
    echo "FAIL: lease should have been rejected"
    exit 1
  fi
  ```
- Verifier exits 0 IFF the rejection fires AND grep matches one of
  the expected wording variants. Mirror's `main` is at SHA-B
  (untouched by the failed push); the harness can additionally
  assert this via `git -C "$TMPDIR/mirror.git" rev-parse
  refs/heads/main` returning `SHA-B` exactly.

**T05 latency artifact + measurement script:**

- `quality/reports/verifications/perf/webhook-latency.json` (new
  file), shape:
  ```json
  {
    "measured_at": "<T05-commit-date in ISO 8601 UTC>",
    "method": "synthetic-dispatch",
    "n": 10,
    "p50_seconds": <integer>,
    "p95_seconds": <integer ≤ 120>,
    "max_seconds": <integer>,
    "target_seconds": 60,
    "verdict": "PASS"
  }
  ```
  T05 produces the n=10 measurements via the synthetic dispatch
  harness (`gh api repos/.../dispatches` + poll for
  `refs/mirrors/confluence-synced-at` ref-update); writes the JSON
  with `verdict: PASS` if `p95_seconds ≤ 120`, otherwise `verdict:
  FAIL` (and T06 blocks).

- `scripts/webhook-latency-measure.sh` (new file, ~40 lines, owner-
  runnable for the real-TokenWorld n=10 manual-edit pass per
  RESEARCH.md § "Latency Measurement Strategy"):
  ```bash
  #!/usr/bin/env bash
  # scripts/webhook-latency-measure.sh — owner-runnable n=10
  # manual-edit pass against TokenWorld + reposix-tokenworld-mirror.
  # Shipped in P84 T05; produces the headline real-TokenWorld
  # number for the v0.13.0 latency artifact refresh.
  #
  # Prerequisites:
  #   - gh auth status confirms repo + workflow scopes.
  #   - Confluence webhook configured to dispatch reposix-mirror-sync.
  #   - Edit access to TokenWorld pages.
  #
  # Output: refreshed quality/reports/verifications/perf/webhook-latency.json
  # with method="real-tokenworld-manual-edit", n=10, real timings.
  set -euo pipefail
  REPO="reubenjohn/reposix-tokenworld-mirror"
  REF="refs/mirrors/confluence-synced-at"
  TIMINGS=$(mktemp); trap "rm -f $TIMINGS" EXIT
  for i in $(seq 1 10); do
    echo ""
    echo "Iteration $i / 10:"
    echo "  1. Edit a TokenWorld page in your browser."
    echo "  2. Save the edit."
    echo "  3. Press ENTER here when the save completes."
    read
    T_EDIT=$(date +%s)
    PRIOR=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "")
    while true; do
      NEW=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "")
      if [ -n "$NEW" ] && [ "$NEW" != "$PRIOR" ]; then
        T_DONE=$(date +%s)
        echo "  -> ref-update observed after $((T_DONE - T_EDIT))s"
        echo "$((T_DONE - T_EDIT))" >> "$TIMINGS"
        break
      fi
      sleep 2
      if [ $(($(date +%s) - T_EDIT)) -gt 180 ]; then
        echo "  -> TIMEOUT (>180s); skipping iteration"
        break
      fi
    done
  done
  P50=$(sort -n "$TIMINGS" | awk 'NR==int(NR_TOTAL*0.5)+1' NR_TOTAL=$(wc -l < "$TIMINGS"))
  P95=$(sort -n "$TIMINGS" | awk 'NR==int(NR_TOTAL*0.95)+1' NR_TOTAL=$(wc -l < "$TIMINGS"))
  MAX=$(sort -n "$TIMINGS" | tail -1)
  N=$(wc -l < "$TIMINGS")
  TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  VERDICT="PASS"; [ "$P95" -gt 120 ] && VERDICT="FAIL"
  cat > quality/reports/verifications/perf/webhook-latency.json <<JSON
  {
    "measured_at": "${TS}",
    "method": "real-tokenworld-manual-edit",
    "n": ${N},
    "p50_seconds": ${P50:-0},
    "p95_seconds": ${P95:-0},
    "max_seconds": ${MAX:-0},
    "target_seconds": 60,
    "verdict": "${VERDICT}"
  }
  JSON
    echo ""
    echo "Wrote quality/reports/verifications/perf/webhook-latency.json"
    echo "  p50=${P50}s p95=${P95}s max=${MAX}s verdict=${VERDICT}"
  ```
- Script is `chmod +x` and committed alongside the JSON artifact in
  T05.

**`webhook-latency-floor.sh` verifier (T01 mints; T05 closes):** —
asset-exists asserter, ~25 lines:
```bash
#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-latency-floor
# CADENCE: pre-release
# INVARIANT: quality/reports/verifications/perf/webhook-latency.json
#            exists, parses, has p95_seconds ≤ 120 (falsifiable threshold
#            per ROADMAP P84 SC4).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"
ARTIFACT="quality/reports/verifications/perf/webhook-latency.json"
test -f "$ARTIFACT" || { echo "FAIL: $ARTIFACT does not exist"; exit 1; }
P95=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['p95_seconds'])" "$ARTIFACT")
[ "$P95" -le 120 ] || { echo "FAIL: p95_seconds=$P95 > 120s threshold"; exit 1; }
echo "PASS: $ARTIFACT p95=${P95}s within threshold"
exit 0
```

**CLAUDE.md (T06; QG-07; one paragraph + one bullet):**

- § Architecture: new paragraph (4-6 sentences) introducing the
  webhook-driven mirror sync workflow. Names: (a) the workflow path
  in the mirror repo (`reubenjohn/reposix-tokenworld-mirror/.github/
  workflows/reposix-mirror-sync.yml`); (b) the secrets convention
  (`gh secret set ATLASSIAN_*` on the mirror repo); (c) the
  template copy at `docs/guides/dvcs-mirror-setup-template.yml`;
  (d) the cron-cadence-edit-the-YAML constraint (D-06 / Pitfall 3);
  (e) forward-link to P85's `docs/guides/dvcs-mirror-setup.md` for
  the full owner walk-through.
- § Commands → "Local dev loop" or new "Mirror sync" sub-section:
  bullet for the synthetic dispatch invocation:
  ```
  gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
    -f event_type=reposix-mirror-sync                    # manually trigger mirror sync (P84+)
  ```

**Phase-close contract:**

- Plan terminates with `git push origin main` against the canonical
  repo in T06 (per CLAUDE.md push cadence) with pre-push GREEN.
  Verifier subagent dispatch is an orchestrator-level action AFTER
  push lands — NOT a plan task.
- T02 separately pushes the live workflow YAML to the mirror repo
  (`reubenjohn/reposix-tokenworld-mirror`) — that's a SEPARATE git
  push (no pre-push hook on the mirror repo); idempotent on retry.
- NO cargo invocations across all tasks. Build memory budget rule
  trivially satisfied.
- The verifier subagent runs ALL 6 verifier shells independently +
  re-grades the catalog rows from artifacts.
</must_haves>
