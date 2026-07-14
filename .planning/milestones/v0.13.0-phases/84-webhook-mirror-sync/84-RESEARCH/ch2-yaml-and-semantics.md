# Phase 84 Research — Workflow YAML, Force-with-Lease Semantics, First-run Handling

← [back to index](./index.md)

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
