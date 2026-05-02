← [back to index](./index.md) · phase 84 plan 01

## Task 84-01-T04 — `--force-with-lease` race shell harness (DVCS-WEBHOOK-02)

<read_first>
- `quality/gates/agent-ux/webhook-force-with-lease-race.sh` (the
  T01 stub — replaced in this task with the full ~80-line harness).
- `.planning/phases/84-webhook-mirror-sync/84-RESEARCH.md`
  § "Race-protection test fixture (sketch)" (lines 482-516) — the
  verbatim shell sketch.
- `.planning/phases/84-webhook-mirror-sync/84-RESEARCH.md`
  § "`--force-with-lease` Semantics" (lines 199-227) — the
  invariant + race walk-through.
- `docs/guides/dvcs-mirror-setup-template.yml` (T02 output) — the
  push step's lease syntax (`--force-with-lease=refs/heads/main:${LEASE_SHA}`)
  which T04's harness mirrors.
- `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` (T03
  donor pattern for shell harness shape).
</read_first>

<action>
Replace the T01 stub `webhook-force-with-lease-race.sh` with the
full race-protection harness. ~80 lines total. Mirrors RESEARCH.md
§ "Race-protection test fixture (sketch)" verbatim, with a final
mirror-state-untouched assertion added.

Replace the file body verbatim:

```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/webhook-force-with-lease-race.sh
# Race-protection harness for `git push --force-with-lease=...`
# (DVCS-WEBHOOK-02 / T-84-03).
#
# CATALOG ROW: agent-ux/webhook-force-with-lease-race
# CADENCE: pre-pr (~1s wall time)
# INVARIANT:
#   When the workflow's local mirror/main is at SHA-A, and a concurrent
#   push (e.g., the bus-remote winning the race) advances the mirror's
#   main from SHA-A to SHA-B between the workflow's `git fetch` and
#   its `git push --force-with-lease=refs/heads/main:SHA-A`, the push
#   is REJECTED. The mirror's main remains at SHA-B (untouched by the
#   workflow's failed push).
#
# Rejection wording varies by git version — assert the SET
# {stale info, rejected, non-fast-forward}.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# 1. Set up the bare-repo "mirror".
git init --bare "$TMPDIR/mirror.git" >/dev/null
git -C "$TMPDIR/mirror.git" symbolic-ref HEAD refs/heads/main

# 2. Seed mirror with SHA-A.
git init "$TMPDIR/wt-a" >/dev/null 2>&1
git -C "$TMPDIR/wt-a" -c init.defaultBranch=main checkout -B main >/dev/null 2>&1
git -C "$TMPDIR/wt-a" -c user.name=test -c user.email=test@test \
  commit --allow-empty -m "seed-A" >/dev/null
SHA_A=$(git -C "$TMPDIR/wt-a" rev-parse HEAD)
git -C "$TMPDIR/wt-a" remote add mirror "$TMPDIR/mirror.git"
git -C "$TMPDIR/wt-a" push mirror main >/dev/null 2>&1

# 3. Workflow's working tree fetches mirror — sees SHA-A.
git init "$TMPDIR/wt-workflow" >/dev/null 2>&1
git -C "$TMPDIR/wt-workflow" remote add mirror "$TMPDIR/mirror.git"
git -C "$TMPDIR/wt-workflow" fetch mirror main >/dev/null 2>&1
LEASE_SHA=$(git -C "$TMPDIR/wt-workflow" rev-parse refs/remotes/mirror/main)
[ "$LEASE_SHA" = "$SHA_A" ] \
  || { echo "FAIL: workflow tree saw mirror/main=${LEASE_SHA:0:7}, expected SHA-A=${SHA_A:0:7}"; exit 1; }

# 4. Bus push wins the race — pushes SHA-B to mirror.
git -C "$TMPDIR/wt-a" -c user.name=test -c user.email=test@test \
  commit --allow-empty -m "bus-B" >/dev/null
SHA_B=$(git -C "$TMPDIR/wt-a" rev-parse HEAD)
git -C "$TMPDIR/wt-a" push mirror main >/dev/null 2>&1
MIRROR_SHA_AFTER_BUS=$(git -C "$TMPDIR/mirror.git" rev-parse refs/heads/main)
[ "$MIRROR_SHA_AFTER_BUS" = "$SHA_B" ] \
  || { echo "FAIL: bus push didn't advance mirror to SHA-B"; exit 1; }

# 5. Workflow now tries `git push --force-with-lease=refs/heads/main:SHA-A`.
#    Should reject (lease check sees mirror/main = SHA-B != SHA-A).
git -C "$TMPDIR/wt-workflow" -c user.name=test -c user.email=test@test \
  commit --allow-empty -m "workflow-X" >/dev/null

PUSH_OUTPUT=$(mktemp); trap "rm -f $PUSH_OUTPUT $TMPDIR" EXIT
if git -C "$TMPDIR/wt-workflow" push mirror "refs/heads/main:refs/heads/main" \
     --force-with-lease="refs/heads/main:$SHA_A" >"$PUSH_OUTPUT" 2>&1; then
  echo "FAIL: lease should have been rejected but push succeeded"
  cat "$PUSH_OUTPUT"
  exit 1
fi

# 6. Assert the rejection wording is one of the expected variants.
if grep -q -E "stale info|rejected|non-fast-forward" "$PUSH_OUTPUT"; then
  REJECTION_REASON=$(grep -oE "stale info|rejected|non-fast-forward" "$PUSH_OUTPUT" | head -1)
  echo "PASS: lease rejected ('${REJECTION_REASON}') as expected on race"
else
  echo "FAIL: rejection occurred but wording doesn't match {stale info, rejected, non-fast-forward}"
  cat "$PUSH_OUTPUT"
  exit 1
fi

# 7. Assert mirror's main is STILL at SHA-B (untouched by the failed push).
MIRROR_SHA_FINAL=$(git -C "$TMPDIR/mirror.git" rev-parse refs/heads/main)
[ "$MIRROR_SHA_FINAL" = "$SHA_B" ] \
  || { echo "FAIL: mirror state corrupted — expected SHA-B=${SHA_B:0:7}, got ${MIRROR_SHA_FINAL:0:7}"; exit 1; }
echo "PASS: mirror/main still at ${MIRROR_SHA_FINAL:0:7} (untouched by failed lease push)"

exit 0
```

Test the harness locally + commit:

```bash
chmod +x quality/gates/agent-ux/webhook-force-with-lease-race.sh
bash quality/gates/agent-ux/webhook-force-with-lease-race.sh \
  || { echo "harness failed locally; fix before committing"; exit 1; }
git add quality/gates/agent-ux/webhook-force-with-lease-race.sh
git commit -m "$(cat <<'EOF'
test(P84): --force-with-lease race harness (DVCS-WEBHOOK-02)

Replace T01 stub with full race-protection walk-through (~80 lines).

Walk-through:
  1. bare-repo mirror seeded with SHA-A
  2. workflow's working tree fetches; sees mirror/main = SHA-A
  3. concurrent "bus push" pushes SHA-B to the mirror
  4. workflow attempts `git push --force-with-lease=refs/heads/main:SHA-A`
  5. lease check rejects (server sees mirror/main = SHA-B, not SHA-A)
  6. assert rejection wording in {stale info, rejected, non-fast-forward}
  7. assert mirror/main is STILL at SHA-B (untouched by failed push)

Wall time <1s on file:// bare repos.

Phase 84 / Plan 01 / Task 04 / DVCS-WEBHOOK-02.
EOF
)"
```
</action>

<verify>
  <automated>bash quality/gates/agent-ux/webhook-force-with-lease-race.sh</automated>
</verify>

<done>
- `quality/gates/agent-ux/webhook-force-with-lease-race.sh` is the
  full ~80-line harness, executable, runs in <1s.
- Running the harness exits 0 with stdout containing both
  "PASS: lease rejected" and "PASS: mirror/main still at" lines.
- The rejection-wording assertion matches one of `{stale info,
  rejected, non-fast-forward}` (git's variants across versions).
- The mirror-untouched assertion holds: `git rev-parse
  refs/heads/main` returns `SHA-B` exactly after the failed lease
  push.
- Catalog row `agent-ux/webhook-force-with-lease-race` would flip
  PASS if T06 ran the catalog runner now (deferred to T06).
- Commit message annotates "P84 / Plan 01 / Task 04".
</done>

---

