# Phase 56: Restore release artifacts — fix the broken installer URLs (v0.12.0)

**Goal:** Close the user-facing breakage that motivated this milestone. `release.yml` does not fire on release-plz's per-crate `reposix-cli-v*` tags; consequently the curl/PowerShell installer URLs return `Not Found`, the homebrew tap formula has not auto-updated, and `cargo binstall reposix-cli reposix-remote` falls back to source build because no GH binary asset exists. Pick the cleaner of two options diagnosed in `.planning/research/v0.12.0/install-regression-diagnosis.md` (extend `on.push.tags` glob to match `reposix-cli-v*` and key the dist version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` (or equivalent) release and verify all 5 install paths end-to-end against the freshly-published assets. This phase is the catalyst that proves the framework is needed; the framework itself lands in P57. Operating-principle hooks: **OP-1 close the feedback loop** — fetch each install URL from a fresh container or curl session, do not trust the workflow log; **OP-6 ground truth obsession** — the verifier subagent runs each install path verbatim from the docs and asserts the binary lands on PATH.

**Requirements:** RELEASE-01, RELEASE-02, RELEASE-03

**Depends on:** (nothing — entry-point phase; v0.11.x shipped state is the precondition)

**Success criteria:**
1. `release.yml` fires on the appropriate tag pattern (per the chosen option in the diagnosis doc); a fresh release tag triggers the workflow and produces non-empty `assets:[…]` on the GH Release.
2. **All 5 install paths verified end-to-end against the fresh release:** `curl … | sh` (Linux/macOS), `iwr … | iex` (PowerShell), `brew install reubenjohn/reposix/reposix-cli`, `cargo binstall reposix-cli reposix-remote` (resolves to prebuilt binary, not source fallback), `cargo install reposix-cli` (build-from-source). Each path's success is recorded as a row in `quality/reports/verifications/release/install-paths/<path>.json`.
3. The `upload-homebrew-formula` job in `release.yml` runs and bumps the tap formula to the new version.
4. Catalog rows for the install paths land in `quality/catalogs/install-paths.json` (or equivalent — the unified-schema name is finalized in P57; for P56 the rows may live in a temporary catalog that P57 migrates) BEFORE the release.yml fix commit.
5. CLAUDE.md updated to reflect this phase's contributions (release.yml tag-glob convention, install-path verification expectation, the new install-paths catalog reference) in the same PR.
6. Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p56/<ts>.md`. RELEASE-01..03 flip from `planning` → `shipped` only after the verifier verdict.

**Context anchor:** `.planning/REQUIREMENTS.md` `## v0.12.0 Requirements — Quality Gates` § "Release dimension — close the immediate breakage", `.planning/research/v0.12.0/install-regression-diagnosis.md` (root cause + two fix options + recommended choice), v0.11.x release-plz workflow at `.github/workflows/release-plz.yml`, the broken `release.yml` it stopped firing.
