---
title: DVCS mirror setup
---

# DVCS mirror setup

This is the owner's walk-through for standing up a DVCS deployment against a Confluence space. You run it once per space. After it ships, anyone on your team can `git clone` the mirror with vanilla git (the [mirror-only consumer pattern](../concepts/dvcs-topology.md#pattern-a-vanilla-mirror-only-mirror-only-consumer)), and writers can `reposix attach` to round-trip back to the SoT (the [round-tripper pattern](../concepts/dvcs-topology.md#pattern-c-vanilla-clone-then-reposix-attach-round-tripper)).

The mechanism: a GitHub Action workflow living in the **mirror repository** runs on `repository_dispatch` (fired by a Confluence webhook) plus a 30-minute cron safety net. Each run does `reposix init confluence::SPACE`, then `git push --force-with-lease` to the mirror.

## What you need before you start

- An Atlassian Cloud account that owns (or can edit) the Confluence space you want to mirror.
- An Atlassian API token. Generate at <https://id.atlassian.com/manage-profile/security/api-tokens>.
- A GitHub account with permission to create a repository under your org or username.
- The `gh` CLI installed and authenticated (`gh auth login`).
- About 15 minutes for the first walk-through; 5 minutes per subsequent space.

If you also want the [agent-flow real-backend tests](../reference/testing-targets.md) against the mirrored space, set `REPOSIX_ALLOWED_ORIGINS` and the credentials in your shell after this setup completes.

## Step 1 — Create the mirror repository

The mirror repo holds the rendered Markdown of the SoT plus the `refs/mirrors/...` annotation refs. Make it **public** so vanilla-git mirror-only consumers can clone without auth, or **private** if your tracker is internal.

```bash
gh repo create <org>/<space>-mirror --public --description "DVCS mirror of confluence::<space>"
```

Pick the name carefully — by convention it ends in `-mirror`. Examples: `acme/engineering-mirror`, `reubenjohn/reposix-tokenworld-mirror`.

> **Tip:** start with one space. Adding a second space is a second walk-through; do not try to mirror multiple spaces into a single GH repository.

## Step 2 — Drop the workflow into the mirror repo

The workflow template lives at [`docs/guides/dvcs-mirror-setup-template.yml`](dvcs-mirror-setup-template.yml) in the reposix repo. Copy it into your new mirror repo at `.github/workflows/reposix-mirror-sync.yml`.

```bash
git clone git@github.com:<org>/<space>-mirror.git /tmp/<space>-mirror
cd /tmp/<space>-mirror
mkdir -p .github/workflows
curl -sSfL https://raw.githubusercontent.com/reubenjohn/reposix/main/docs/guides/dvcs-mirror-setup-template.yml \
  -o .github/workflows/reposix-mirror-sync.yml
git add .github/workflows/reposix-mirror-sync.yml
git commit -m 'ci: add reposix-mirror-sync workflow'
git push origin main
```

Open the file. The two pieces you might want to edit:

- **Cron cadence** — line 22 is the literal `'*/30 * * * *'` schedule. GitHub Actions evaluates `schedule:` before any `${{ vars.* }}` substitution, so the cron interval **cannot be templated**. To change the cadence, edit this line directly. Every 30 minutes is conservative for low-edit Confluence spaces; aggressive for high-churn projects. Tune to taste.
- **Space name** — `${{ vars.CONFLUENCE_SPACE || 'TokenWorld' }}` lets you set the space via repo variables (Step 3 below) or fall back to a default. If you only ever mirror one space, hard-code it for clarity.

## Step 3 — Configure secrets and variables on the mirror repo

The workflow reads three secrets and one variable. Set them with `gh`:

```bash
cd /tmp/<space>-mirror

# Secrets — encrypted, only visible to the workflow.
gh secret set ATLASSIAN_API_KEY            # paste the API token from prerequisites
gh secret set ATLASSIAN_EMAIL              # your Atlassian account email
gh secret set REPOSIX_CONFLUENCE_TENANT    # e.g. 'acme' for acme.atlassian.net

# Variables — readable, can be templated into env: blocks.
gh variable set CONFLUENCE_SPACE --body '<SPACE-KEY>'
```

Verify:

```bash
gh secret list
gh variable list
```

The workflow's `env:` block builds `REPOSIX_ALLOWED_ORIGINS` automatically from the tenant secret — you do not set it directly. The egress allowlist is fail-closed by default; the workflow opens exactly one origin (`https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net`) and keeps loopback open for any sidecar testing.

## Step 4 — Smoke-test with a manual run

Before you wire up the webhook, confirm the workflow works by triggering it manually:

```bash
gh workflow run reposix-mirror-sync.yml
gh run watch                                        # follow the live log
```

Expected outcome:

- The job installs `reposix-cli` via `cargo binstall` (~10 seconds; faster than `cargo install`).
- It runs `reposix init confluence::<SPACE> /tmp/sot` against your space.
- It pushes the rendered Markdown to `refs/heads/main` of the mirror repo with `--force-with-lease`.
- It pushes `refs/mirrors/<sot-host>-head` and `refs/mirrors/<sot-host>-synced-at`.
- The job exits 0.

If the run fails, jump to [Troubleshooting → DVCS push/pull issues](troubleshooting.md#dvcs-pushpull-issues) for the common causes (egress allowlist, wrong tenant slug, missing API token scope).

After the first successful run, `git fetch origin && git log --oneline -5` from a fresh clone will show the rendered issues; `git for-each-ref refs/mirrors/` will show the two annotation refs.

## Step 5 — Configure the Confluence webhook

Webhooks fire `repository_dispatch` on the mirror repo, which the workflow's `on:` block listens for. With this in place, an edit in Confluence triggers the mirror sync within ~30 seconds (rather than waiting up to 30 minutes for the cron).

1. Mint a fine-grained GitHub PAT with `repo` scope on the mirror repo. (Atlassian's webhook needs to call `POST https://api.github.com/repos/<org>/<space>-mirror/dispatches`.)
2. In Atlassian admin, navigate to **Settings → System → Webhooks → Create webhook**.
3. Configure:
    - **URL:** `https://api.github.com/repos/<org>/<space>-mirror/dispatches`
    - **Method:** `POST`
    - **Headers:**
      - `Authorization: token <your-PAT>`
      - `Accept: application/vnd.github+json`
    - **Body** (JSON):
      ```json
      {"event_type": "reposix-mirror-sync"}
      ```
    - **Events:** subscribe to `Page created`, `Page updated`, `Page removed` for your space.
4. Save.

Atlassian's webhook docs: <https://confluence.atlassian.com/doc/manage-webhooks-1021225606.html>.

Edit a page in Confluence; within ~30 seconds, `gh run list -R <org>/<space>-mirror -L 1` should show a fresh run kicked off by `repository_dispatch`.

## Backends without webhooks (cron-only mode)

Some backends do not emit webhooks (or emit them on plans you do not have). For those, the cron schedule is the only sync mechanism. The workflow already supports this — just remove the `repository_dispatch:` trigger from `on:`:

```yaml
# .github/workflows/reposix-mirror-sync.yml — cron-only variant.
on:
  schedule:
    - cron: '*/30 * * * *'
  workflow_dispatch:
```

Trade-off:

- **Latency floor** is the cron interval (every 30 minutes by default). A Confluence edit at 14:31:00 lands in the mirror at 15:00 cron tick — up to 29 minutes of staleness in the worst case.
- **`refs/mirrors/<sot-host>-synced-at`** still tells the truth — it advances each cron tick when there are SoT-side changes to apply.

For Confluence specifically, webhooks are available on Standard / Premium / Enterprise plans; if your space is on Free, cron-only is your only option.

## Updating the cron cadence

To change `*/30` to a different frequency:

```bash
cd /tmp/<space>-mirror
sed -i "s|cron: '\\*/30 \\* \\* \\* \\*'|cron: '\\*/15 \\* \\* \\* \\*'|" \
  .github/workflows/reposix-mirror-sync.yml
git add .github/workflows/reposix-mirror-sync.yml
git commit -m 'ci(mirror-sync): tighten cron to every 15 minutes'
git push origin main
```

GitHub Actions has a documented quirk: scheduled workflows in low-traffic repos may skip ticks. If precise interval matters, configure the webhook (Step 5) and treat the cron as a safety net.

## Cleanup procedure (tear-down)

When you no longer need the mirror — the space was archived, the team moved off the tracker, or you are migrating to a different SoT — tear it down cleanly so you do not leave orphan refs or zombie cron jobs.

```bash
# 1. Disable the workflow so no further runs fire.
gh workflow disable reposix-mirror-sync.yml -R <org>/<space>-mirror

# 2. Remove the Confluence webhook.
#    Atlassian admin → Settings → System → Webhooks → delete the entry from Step 5.

# 3. Delete the mirror repo (DESTRUCTIVE — confirm you want this).
gh repo delete <org>/<space>-mirror --yes

# 4. (Optional) Revoke the PAT used by the webhook at
#    https://github.com/settings/tokens.

# 5. (Optional) Locally, delete the cache directory for the SoT.
rm -rf ~/.cache/reposix/confluence-<SPACE>.git
```

If you only want to **pause** rather than tear down, do steps 1 and 2 (disable workflow + remove webhook). The mirror repo and its refs stay intact; re-enabling later just runs `gh workflow enable` and re-creates the webhook.

## Troubleshooting

If a run fails, the most common causes are:

| Symptom | Fix |
|---|---|
| `Error::InvalidOrigin` in the run log | `REPOSIX_ALLOWED_ORIGINS` does not include the tenant; check `REPOSIX_CONFLUENCE_TENANT` secret. |
| `couldn't find remote ref main` on first run | Truly-empty mirror — workflow handles it; subsequent run should pass. |
| `--force-with-lease` rejected | Race with a concurrent bus push; cron will retry the next tick. |
| `cargo binstall` cannot find a release | Pinned to `reposix-cli`; check the published version on crates.io. |
| Webhook fires but no run starts | PAT lacks `repo` scope on the mirror; rotate at GitHub settings. |

The full troubleshooting matrix — including bus-remote `fetch first` rejections, attach reconciliation warnings, and cache-desync recovery — lives at [Troubleshooting → DVCS push/pull issues](troubleshooting.md#dvcs-pushpull-issues).

## See also

- [DVCS topology — three roles](../concepts/dvcs-topology.md) — the mental model this guide implements.
- [`dvcs-mirror-setup-template.yml`](dvcs-mirror-setup-template.yml) — the workflow template you copied in Step 2.
- [Testing targets](../reference/testing-targets.md) — TokenWorld is the reference Confluence space; the same setup applies to your space.
- [Troubleshooting](troubleshooting.md) — diagnosis catalog when something goes wrong.
