# v0.12.0 — Install Regression Diagnosis

> **Audience.** The agent planning P56 (RELEASE-01). This document is the COMPLETE root-cause analysis of the curl-installer regression that motivated v0.12.0. P56's plan should reference this doc as its `Context anchor`.

## The user observation

Owner ran:
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh
```
Result: `Not Found` (the URL returns ~9 bytes — the literal text "Not Found").

## The chain of cause

1. **release-plz cut over to per-crate tags.** Starting with v0.11.1, release-plz creates per-crate tags like `reposix-cli-v0.11.2`, `reposix-core-v0.11.2`, etc. — one tag per crate. There is NO workspace-wide `v0.11.2` tag.

2. **`.github/workflows/release.yml` only fires on tag glob `v*`.** Looking at the workflow:
   ```yaml
   on:
     push:
       tags:
         - 'v*'
   ```
   Glob `v*` matches tags starting with `v`. Per-crate tags start with `r` (e.g., `reposix-cli-v0.11.2`). They do NOT match.

3. **`release.yml` last ran on 2026-04-26 via `workflow_dispatch`** for the manual `v0.11.0` tag push. It has not fired automatically since the release-plz cutover.

4. **Every release after `v0.11.0` has `assets:[]`.** Verified:
   ```
   gh api repos/reubenjohn/reposix/releases/latest | jq .tag_name → "reposix-cli-v0.11.2"
   gh api repos/reubenjohn/reposix/releases/latest | jq '.assets[].name' → (empty)
   ```
   release-plz creates the GitHub Release object (because it tags), but no workflow runs to populate assets.

5. **GitHub's `releases/latest` pointer follows release recency.** The most recently published release is `reposix-cli-v0.11.2` → that's "latest" → `releases/latest/download/<asset>` 302s to that release → asset is missing → `Not Found`.

## What's actually broken (4 install paths)

| Path | Status | Why |
|---|---|---|
| `curl ... reposix-installer.sh \| sh` | **BROKEN** | The asset doesn't exist on the latest release. |
| `irm ... reposix-installer.ps1 \| iex` | **BROKEN** | Same root cause as the curl path. |
| `cargo binstall reposix-cli reposix-remote` | **PARTIAL** | crates.io has v0.11.2 (release-plz published it). binstall resolves to the crate version, then looks for a prebuilt GH binary; falls back to slow source build because no GH binary asset exists. Functionally works, just slow. |
| `brew install reubenjohn/reposix/reposix` | **STALE** | Tap repo (`reubenjohn/homebrew-reposix`) is reachable. Formula version likely pinned at v0.11.0 because `release.yml`'s `upload-homebrew-formula` job hasn't run. (Verify: `gh api repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb`.) |

## What still works

- `cargo install reposix-cli` (slow but functional — crates.io has the latest)
- Build from source (`git clone && cargo build --release`)

## Two fix options

### Option A — Extend the tag glob

```yaml
# .github/workflows/release.yml
on:
  push:
    tags:
      - 'v*'
      - 'reposix-cli-v*'
```

Then in the `plan` job, derive version from whichever tag matched:

```yaml
- name: Resolve tag + version
  id: plan
  run: |
    TAG="${GITHUB_REF#refs/tags/}"
    # Strip leading "reposix-cli-" if present
    VERSION="${TAG#reposix-cli-}"
    VERSION="${VERSION#v}"
    echo "tag=${TAG}" >> "$GITHUB_OUTPUT"
    echo "version=${VERSION}" >> "$GITHUB_OUTPUT"
```

**Pro:** minimal diff; fires automatically on every release-plz CLI tag push.
**Con:** if release-plz also tags `reposix-remote-v*` (which has the helper binary that the installer needs), we might want to fire on remote tags too. Need to check what the helper crate's release pattern is.
**Risk:** the `${{ github.ref }}` substitutions in downstream steps may need updating.

### Option B — release-plz post-publish mirror tag

Add a step to `.github/workflows/release-plz.yml` that, after `reposix-cli` ships, pushes a workspace-wide `vX.Y.Z` tag:

```yaml
- name: Mirror workspace tag for release.yml
  if: contains(steps.release-plz.outputs.releases_created, 'reposix-cli')
  run: |
    VERSION=$(jq -r '.[] | select(.package_name == "reposix-cli") | .version' \
              <<< '${{ steps.release-plz.outputs.releases }}')
    git tag "v${VERSION}"
    git push origin "v${VERSION}"
```

**Pro:** keeps `release.yml` unchanged — `v*` glob still works as designed.
**Con:** more YAML; depends on release-plz outputs which may have versioning quirks across release-plz versions.
**Risk:** double-tagging (per-crate + workspace) might confuse some tooling that watches GH releases.

## Recommendation

Option A is cleaner and has fewer moving parts. Option B is the fallback if Option A breaks the version derivation somewhere downstream.

P56's plan should:
1. Try Option A first (extend tag glob + version derivation).
2. Test by triggering `release.yml` via `workflow_dispatch` on a `reposix-cli-v0.11.3` tag.
3. If the build succeeds and assets appear: cut a real release-plz PR for v0.11.3 and verify it triggers `release.yml` automatically.
4. If Option A breaks: pivot to Option B (mirror tag in release-plz workflow).

## Validation steps (the catalog row's `expected.asserts` for RELEASE-01)

After the fix:

1. `gh api repos/reubenjohn/reposix/releases/latest | jq '.assets[].name'` lists 7 assets:
   - `reposix-installer.sh`
   - `reposix-installer.ps1`
   - `reposix-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`
   - `reposix-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` (or `aarch64-apple-darwin`)
   - `reposix-vX.Y.Z-x86_64-apple-darwin.tar.gz`
   - `reposix-vX.Y.Z-x86_64-pc-windows-msvc.zip`
   - `SHA256SUMS`
2. `curl -sLI https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh` returns HTTP 200, content-length > 1024, content-type contains `octet-stream` or `text`.
3. In a fresh `ubuntu:24.04` container:
   ```
   apt-get update && apt-get install -y curl
   curl --proto '=https' --tlsv1.2 -LsSf \
       https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh
   command -v reposix              # exits 0
   command -v git-remote-reposix   # exits 0
   reposix --version               # prints the latest version
   ```
4. `gh api repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb | jq -r .content | base64 -d | grep version` shows the latest version.
5. `cargo binstall --no-confirm reposix-cli` resolves to the prebuilt binary (NOT source build).

## Why this is P56 first, not woven into the framework first

Users are blocked right now. Every documented install path either fails or works wrong. The framework that prevents this regression class is high-leverage long-term but takes 8 phases to land. P56 ships the fix in 1-2 hours; P58 (release dimension) builds the gate that catches recurrence.

The catalog row for `release/installer-curl-sh` is in `.planning/docs_reproducible_catalog.json` (DRAFT) — that doc was the seed for v0.12.0. P58 ports it into `quality/catalogs/release-assets.json` with the unified schema. P56's verifier (the asserts above) becomes that catalog row's `verifier.script` once the framework lands.

## Cross-references

- `.planning/docs_reproducible_catalog.json` — DRAFT catalog with this regression as row #1
- `v0.12.0-vision-and-mental-model.md` — why a coherent system catches this class
- `v0.12.0-roadmap-and-rationale.md` §P56 — phase rationale + pivot rules
- `.github/workflows/release.yml` — file to edit
- `.github/workflows/release-plz.yml` — fallback file for Option B
