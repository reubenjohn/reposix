# Release.yml fire evidence — P56 RELEASE-01..03

## Trigger that fired

- **Run ID:** `25011639541`
- **Trigger event:** `workflow_dispatch` (stop-gap; see "Tag-push trigger gap" below)
- **Trigger ref:** `refs/tags/reposix-cli-v0.11.3`
- **Option chosen (A or B):** **Option A** — `release.yml` `on.push.tags` glob extended to `['v*', 'reposix-cli-v*']` in commit `d3f0dce` (Wave B / 56-02). The plan-job version derivation strips the `reposix-cli-` prefix and computes `version=0.11.3`, `version_tag=v0.11.3` regardless of trigger.
- **Run URL:** https://github.com/reubenjohn/reposix/actions/runs/25011639541
- **Started at:** `2026-04-27T18:09:37Z`
- **Completed at:** `2026-04-27T18:16:01Z`
- **Duration:** ~6m24s
- **Conclusion:** success
- **Head SHA built:** `f9fd21c5c0494950fc5cbe6f558c9ff3ca1b639f` (the `chore: release v0.11.3 (#24)` merge commit)

## Tag-push trigger gap (the surprise)

The natural path was: merge release-plz PR #24 → release-plz workflow runs → release-plz pushes per-crate tags including `reposix-cli-v0.11.3` → `release.yml` fires automatically on `on.push.tags: 'reposix-cli-v*'`.

What actually happened: PR #24 merged at `f9fd21c` (2026-04-27T18:04:59Z); release-plz workflow `25011441978` completed success and pushed all 8 v0.11.3 tags (verified via `git fetch origin --tags`); release-plz also created the 8 GH Release objects (with empty asset arrays). **`release.yml` did NOT auto-fire** despite the tag push and the matching glob.

Root cause: per the GitHub Actions documented behaviour, **tags pushed by the default `GITHUB_TOKEN` do not trigger downstream `on.push` workflows** (this is GH's loop-prevention rule). The `release-plz.yml` workflow uses `GITHUB_TOKEN` to push tags (see `env: GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}` on the `MarcoIeni/release-plz-action@v0.5` step). Therefore the trigger semantics for release-plz-pushed tags are: tag exists, GH Release object exists, no `release.yml` run.

**Stop-gap used in this wave:** dispatched `release.yml` manually via `gh workflow run release.yml --repo reubenjohn/reposix --ref reposix-cli-v0.11.3`. Because `--ref` was a tag, `GITHUB_REF` resolved to `refs/tags/reposix-cli-v0.11.3` inside the workflow; the plan-job's `if [[ "$GITHUB_REF" == refs/tags/* ]]` branch fired correctly; the workflow fanned out across 5 platforms, populated `dist/`, ran the homebrew formula upload, and `gh release edit reposix-cli-v0.11.3 --notes-file body.md` updated the existing release object (already created by release-plz) with notes + 8 assets.

**Recovery path (long-term — Wave D / SURPRISES.md / MIGRATE-03 carry-forward):** for `release-plz`-driven tags to auto-trigger `release.yml`, the release-plz workflow must use a fine-grained PAT (separate from `GITHUB_TOKEN`) when pushing tags. Two paths:

1. Add a `RELEASE_PLZ_TOKEN` repo secret (fine-grained PAT with `contents:write`) and pass it as `token:` to release-plz.
2. Or: keep release-plz on `GITHUB_TOKEN` and add a one-line `gh workflow run release.yml --ref reposix-cli-v$VERSION` step at the end of `release-plz.yml`. Mechanical, no PAT-management cost, preserves the human-merges-the-PR security shape.

Either fix is a 5-line YAML change; flagged for v0.12.1 carry-forward (MIGRATE-03 supplement).

## Assets attached to release `reposix-cli-v0.11.3`

| Asset | Bytes | sha256 (first 12) |
|---|--:|---|
| `reposix-installer.sh` | 1,264 | `a3b21cbd3320` |
| `reposix-installer.ps1` | 1,075 | `77a2520464e9` |
| `reposix-v0.11.3-x86_64-unknown-linux-musl.tar.gz` | 9,249,858 | `19955da60ce1` |
| `reposix-v0.11.3-aarch64-unknown-linux-musl.tar.gz` | 8,268,238 | `a337b792ce7a` |
| `reposix-v0.11.3-x86_64-apple-darwin.tar.gz` | 8,709,847 | `4c0d86ce4cf8` |
| `reposix-v0.11.3-aarch64-apple-darwin.tar.gz` | 8,193,025 | `a570023829907` |
| `reposix-v0.11.3-x86_64-pc-windows-msvc.zip` | 8,546,286 | `2db874584816` |
| `SHA256SUMS` | 559 | `78eb216c1767` |

Asset count: **8** (≥7 required by the plan's automated `<verify>`).

## Latest pointer caveat

The `releases/latest/download/...` URL pattern follows GitHub's release-recency pointer. release-plz cuts per-crate releases; `reposix-cli-v0.11.3` is currently `Latest` because it was the most-recently-published per-crate release in the v0.11.3 cycle (cli is alphabetically last among the 8 crates and release-plz publishes in dependency order). If a future per-crate release ships AFTER the cli release (e.g. a `reposix-jira-v0.11.4` patch), `releases/latest/download/reposix-installer.sh` will return 404 again because the non-cli releases don't have installer assets.

This is a known release-plz behaviour and a v0.12.0 known-issue. Recovery options for v0.12.1:

- Switch docs to use pinned-tag URLs (`releases/download/reposix-cli-v0.11.3/...`).
- Make `release.yml`'s `gh release create/edit` step pass `--latest` to override the natural pointer.
- Make release-plz publish `reposix-cli` last-of-cycle (it already does this, by alphabetical chance — but we shouldn't rely on it).

Tracked under **MIGRATE-03** (v0.12.1 carry-forward) — Wave D appends a SURPRISES.md entry.

## Next: install-path verification

Tasks 56-03-B through 56-03-F run each install path and persist per-path evidence under `.planning/verifications/p56/install-paths/`:

- `curl-installer-sh.json` — ubuntu:24.04 container rehearsal
- `powershell-installer-ps1.json` — asset-existence (windows runner deferred)
- `cargo-binstall.json` — rust:1.82-slim container rehearsal
- `homebrew.json` — tap formula version + sha256 (macos runner deferred)
- `build-from-source.json` — pointer to ci.yml `test` job
