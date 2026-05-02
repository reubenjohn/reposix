# Phase 84 Research — Common Pitfalls and Code Examples

← [back to index](./index.md)

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
