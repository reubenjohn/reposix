← [index](./index.md)

## Task 84-01-T02 — Workflow YAML (template + live), pushed to BOTH repos

<read_first>
- `.github/workflows/bench-latency-cron.yml` (entire file ~108 lines)
  — donor pattern: `actions/checkout@v6` + `dtolnay/rust-toolchain@stable`
  + secrets-into-env shape; literal cron expression (line 18); job
  shape.
- `.github/workflows/ci.yml:1-30` — donor pattern: top-level env
  block, concurrency block (different cancellation semantics — D-01
  chooses `cancel-in-progress: false`), permissions block.
- `.github/workflows/release.yml:200-270` (or wherever the env-block
  pattern is — re-confirm during read_first via `grep -n
  REPOSIX_ALLOWED_ORIGINS .github/workflows/*.yml`) — donor pattern
  for `REPOSIX_ALLOWED_ORIGINS` env composition.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md:204-240` —
  verbatim YAML skeleton (the source of truth for the workflow
  shape).
- `.planning/phases/84-webhook-mirror-sync/84-RESEARCH.md` § "Workflow
  YAML Shape" (lines 100-198) — the verbatim YAML with comments
  about the cron-vars constraint (Pitfall 3) + first-run handling
  (Q4.3) + the binstall step (Pitfall 2).
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`
  § "Decisions ratified at plan time" — D-01 (concurrency), D-03
  (header tldr), D-05 (binstall reposix-cli), D-06 (literal cron),
  D-07 (first-run predicate), D-08 (template + live, byte-equal mod
  whitespace).
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:152-155` —
  the workflow lands in the MIRROR repo, NOT canonical (load-bearing).
- `crates/reposix-cli/Cargo.toml` `[package.metadata.binstall]`
  block (re-confirm presence + URL template; binstall metadata
  shape determines whether the workflow's `cargo binstall reposix-cli`
  succeeds against the latest published version per Assumption A5).
</read_first>

<action>
Two concerns: write the YAML in the canonical repo (template) →
push the live copy to the mirror repo → atomic commit in canonical
repo.

### 2a. Template copy in canonical repo

Author `docs/guides/dvcs-mirror-setup-template.yml` per the verbatim
shape from RESEARCH.md § "Workflow YAML Shape" with the D-01..D-08
ratifications applied. The full file (~80 lines):

```yaml
# reposix-mirror-sync — v0.13.0 webhook-driven mirror sync
#
# What: sync the GH mirror with confluence-side edits (the pull
# direction; bus-remote handles the push direction).
# Triggers: repository_dispatch (event_type=reposix-mirror-sync) +
# cron */30min safety net + workflow_dispatch (manual).
# Secrets needed: ATLASSIAN_API_KEY, ATLASSIAN_EMAIL,
# REPOSIX_CONFLUENCE_TENANT (set via `gh secret set` on this repo).
# Cron override: edit this YAML directly (the schedule field cannot
# read ${{ vars.* }} — see Pitfall 3 in P84 RESEARCH).
# Full setup walk-through: docs/guides/dvcs-mirror-setup.md (P85).

name: reposix-mirror-sync

on:
  repository_dispatch:
    types: [reposix-mirror-sync]
  schedule:
    # Literal '*/30 * * * *' — Actions parses schedule BEFORE evaluating
    # ${{ vars.* }} contexts; cron CANNOT be templated. To change cadence,
    # edit this line directly. See P84 RESEARCH § Pitfall 3.
    - cron: '*/30 * * * *'
  workflow_dispatch:

concurrency:
  group: reposix-mirror-sync
  cancel-in-progress: false

permissions:
  contents: write

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
          fetch-depth: 0  # full history needed for --force-with-lease

      - name: Install reposix-cli
        run: |
          set -euo pipefail
          # cargo binstall reads the binstall metadata in
          # crates/reposix-cli/Cargo.toml [package.metadata.binstall]
          # and downloads the right tarball from the matching GH
          # release. Faster than cargo install (~10s vs ~3min).
          curl -L --proto '=https' --tlsv1.2 -sSf \
            https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
            | bash
          # NOTE: published crate name is `reposix-cli`, NOT `reposix`
          # (the workspace root). See P84 RESEARCH § Pitfall 2.
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
          # First-run handling (Q4.3): on a truly-empty mirror, the
          # fetch fails with "couldn't find remote ref main"; downstream
          # `git show-ref --verify --quiet refs/remotes/mirror/main`
          # detects the absence and routes to the plain-push branch.
          git fetch mirror main 2>/dev/null \
            || echo "first-run: mirror has no main yet"

      - name: Push to mirror with --force-with-lease
        run: |
          set -euo pipefail
          cd /tmp/sot
          # Branch on local tracking ref presence (D-07).
          if git show-ref --verify --quiet refs/remotes/mirror/main; then
            LEASE_SHA=$(git rev-parse refs/remotes/mirror/main)
            git push mirror "refs/heads/main:refs/heads/main" \
              --force-with-lease="refs/heads/main:${LEASE_SHA}"
          else
            # First-run path: empty mirror; no lease available.
            git push mirror "refs/heads/main:refs/heads/main"
          fi
          # Mirror-lag refs in a SEPARATE invocation so a refs-write
          # failure doesn't taint the main-branch push outcome. Cron
          # tick re-attempts on transient failure.
          git push mirror "refs/mirrors/confluence-head" \
                          "refs/mirrors/confluence-synced-at" \
            || echo "warn: mirror-lag refs push failed (non-fatal); cron will retry"
```

Validate it parses:

```bash
python3 -c "import yaml; yaml.safe_load(open('docs/guides/dvcs-mirror-setup-template.yml'))"
```

### 2b. Push live copy to mirror repo

Use a temp-clone flow to avoid mutating the canonical repo's working
tree's `.github/workflows/`:

```bash
# Verify gh auth has the right scopes BEFORE attempting the push.
gh auth status 2>&1 | grep -q "repo" \
  || { echo "gh auth missing repo scope; cannot push to mirror"; exit 1; }

# Clone the mirror repo to a tempdir.
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT
git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git \
  "$TMPDIR/mirror"

# Stage the workflow file.
mkdir -p "$TMPDIR/mirror/.github/workflows"
cp docs/guides/dvcs-mirror-setup-template.yml \
   "$TMPDIR/mirror/.github/workflows/reposix-mirror-sync.yml"

# Commit + push.
cd "$TMPDIR/mirror"
git add .github/workflows/reposix-mirror-sync.yml
git commit -m "feat(workflow): add reposix-mirror-sync.yml (P84 / DVCS-WEBHOOK-01)

Mirrors docs/guides/dvcs-mirror-setup-template.yml in reubenjohn/reposix
P84 plan 01. Byte-equal modulo whitespace per D-08.

Triggers: repository_dispatch (event_type=reposix-mirror-sync) + cron
*/30min + workflow_dispatch.

Phase 84 / Plan 01 / Task 02 / live-copy."
git push origin main

# Confirm the push landed.
gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml \
  -q .name >/dev/null \
  || { echo "live copy not reachable post-push"; exit 1; }
echo "live copy landed in mirror repo"

# Return to the canonical repo working tree.
cd "${OLDPWD}"
```

If the cron-clutter concern (RESEARCH.md Pitfall 6) materializes —
i.e., the executor wants to defer the cron's first fire until P84
fully closes — the workflow can be DISABLED on the mirror repo
post-push:

```bash
# OPTIONAL: disable the workflow until phase close to avoid
# cron-driven Actions tab clutter while T03/T04/T05 land.
gh workflow disable reposix-mirror-sync \
  --repo reubenjohn/reposix-tokenworld-mirror

# After T06 phase-close, re-enable:
gh workflow enable reposix-mirror-sync \
  --repo reubenjohn/reposix-tokenworld-mirror
```

This is OPTIONAL; document the choice in T02's commit message
either way.

### 2c. Atomic commit in canonical repo

```bash
git add docs/guides/dvcs-mirror-setup-template.yml
git commit -m "$(cat <<'EOF'
docs(P84): add reposix-mirror-sync workflow template (DVCS-WEBHOOK-01)

Template copy at docs/guides/dvcs-mirror-setup-template.yml; live
copy lives in reubenjohn/reposix-tokenworld-mirror's
.github/workflows/reposix-mirror-sync.yml (pushed in same task).

Triggers: repository_dispatch (event_type=reposix-mirror-sync) +
cron */30 * * * * (literal per D-06; not vars.*-templated due to
GH Actions schedule-block parse-time limitation, see RESEARCH.md
Pitfall 3) + workflow_dispatch.

Concurrency: { group: reposix-mirror-sync, cancel-in-progress:
false } per D-01 — queues duplicate runs (cron-vs-dispatch
near-boundary case) instead of cancelling.

Steps: actions/checkout@v6 (fetch-depth: 0 for --force-with-lease)
→ install reposix-cli via binstall (NOT bare 'reposix' per D-05)
→ reposix init confluence::TokenWorld /tmp/sot → git remote add
mirror + fetch (Q4.3 graceful-fail on truly-empty mirror) →
branch on `git show-ref --verify --quiet refs/remotes/mirror/main`
to choose between --force-with-lease push (D-07 a) and plain
push (D-07 b) → mirror-lag refs push in separate invocation.

NO `${{ github.event.client_payload.* }}` references (T-84-01
mitigation per S2 of OVERVIEW). NO `set -x` (T-84-02 mitigation).
REPOSIX_ALLOWED_ORIGINS set to confluence-tenant-only (T-84-05
allowlist).

Phase 84 / Plan 01 / Task 02 / DVCS-WEBHOOK-01 / template-copy.
EOF
)"
```

NO push to canonical repo yet — that's T06.
</action>

<verify>
  <automated>test -f docs/guides/dvcs-mirror-setup-template.yml && python3 -c "import yaml; yaml.safe_load(open('docs/guides/dvcs-mirror-setup-template.yml'))" && bash quality/gates/agent-ux/webhook-trigger-dispatch.sh && bash quality/gates/agent-ux/webhook-cron-fallback.sh && bash quality/gates/agent-ux/webhook-backends-without-webhooks.sh</automated>
</verify>

<done>
- `docs/guides/dvcs-mirror-setup-template.yml` exists in canonical
  repo working tree, committed.
- `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml`
  returns 200 (live copy reachable in mirror repo).
- `diff -w docs/guides/dvcs-mirror-setup-template.yml <(gh api ... | base64 -d)`
  returns zero (byte-equal modulo whitespace per D-08).
- `webhook-trigger-dispatch.sh` exits 0 (the YAML structural greps
  pass, both copies present, diff -w zero).
- `webhook-cron-fallback.sh` exits 0 (literal cron, fetch-depth 0,
  concurrency block all present).
- `webhook-backends-without-webhooks.sh` exits 0 (the trim-simulation
  produces still-valid YAML).
- The mirror-repo push commit message annotates "P84 / Plan 01 /
  Task 02 / live-copy."
- The canonical-repo commit message annotates "P84 / Plan 01 / Task
  02 / template-copy."
- `git log -1 --oneline` (in canonical repo) shows the template
  commit.
- T03/T04/T05 verifiers (force-with-lease-race, first-run-empty-mirror,
  latency-floor) STILL fail cleanly because their underlying harnesses
  / artifacts haven't shipped yet.
</done>
