# Webhook-driven mirror sync (the pull side)

← [back to index](./index.md)

## Webhook-driven mirror sync (the pull side)

The vision doc establishes this as the v0.13.0 default for keeping the GH mirror current with confluence-side edits. Sketch:

**A reference GitHub Action workflow** ships in `docs/guides/dvcs-mirror-setup.md`. Repo-side `.github/workflows/reposix-mirror-sync.yml`:

```yaml
on:
  repository_dispatch:
    types: [reposix-mirror-sync]
  schedule:
    - cron: '*/30 * * * *'   # safety net if webhook drops

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - run: cargo binstall reposix
      - env:
          ATLASSIAN_API_KEY: ${{ secrets.ATLASSIAN_API_KEY }}
          ATLASSIAN_EMAIL: ${{ secrets.ATLASSIAN_EMAIL }}
          REPOSIX_CONFLUENCE_TENANT: ${{ secrets.REPOSIX_CONFLUENCE_TENANT }}
          REPOSIX_ALLOWED_ORIGINS: 'https://${{ secrets.REPOSIX_CONFLUENCE_TENANT }}.atlassian.net'
        run: |
          reposix init confluence::${{ vars.SPACE }} /tmp/sot
          cd /tmp/sot
          git remote add mirror ${{ github.server_url }}/${{ github.repository }}
          git fetch mirror main
          git push mirror main --force-with-lease=refs/heads/main:$(git rev-parse mirror/main)
          git push mirror refs/mirrors/confluence-head refs/mirrors/confluence-synced-at
```

**Confluence webhook setup** (via Atlassian admin console or REST API) targets `POST https://api.github.com/repos/<org>/<repo>/dispatches` with `event_type: reposix-mirror-sync`. Owner sets up once per space.

**Why `--force-with-lease`:** the webhook sync is the only writer that's allowed to clobber the mirror's `main`. `--force-with-lease` makes it safe against the race where a bus-remote push lands between the workflow's fetch and its push — the lease check will fail and the workflow exits cleanly, knowing the bus remote already did the work.

**Open questions:**

- `Q4.1` Cron fallback frequency. Every 30min is conservative for confluence (low edit rate); aggressive for high-churn projects. Probably make it configurable in the workflow `vars`.
- `Q4.2` What about backends that don't emit webhooks? JIRA does; GH Issues does; confluence does. If a future connector doesn't, the cron path becomes the only sync mechanism. Document this in the connector-author guide.
- `Q4.3` Does the workflow need to do anything different on first run (no existing `mirror/main` ref, no `refs/mirrors/...`)? Probably the workflow handles it gracefully because `git fetch mirror main` will succeed (creates the ref), and `git rev-parse mirror/main` returns the just-fetched SHA. First-run case might want explicit handling for empty mirror; leave for the implementing phase to confirm.
