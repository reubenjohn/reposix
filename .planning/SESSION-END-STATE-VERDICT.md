# SESSION-END-STATE-VERDICT — RED

- Session ID: `5468ec00-693b-45ce-bfbd-b145d6382b9b`
- Generated at: `2026-04-27T05:30:05Z`
- Workspace version: `0.11.1`

| status | count |
|---|---|
| PASS | 16 |
| FAIL | 4 |
| PARTIAL | 0 |
| NOT-VERIFIED | 0 |

## FAIL

- `freshness/no-version-pinned-filenames` — No version-pinned filenames (vN.N.N) outside CHANGELOG and .planning/milestones/v*-phases/. Catches §0.3-class drift.
  - log: `.planning/verifications/_logs/freshness__no-version-pinned-filenames.txt`
- `freshness/install-leads-with-pkg-mgr/docs-index` — docs/index.md hero must show a package-manager install command (brew/binstall/curl) BEFORE any 'git clone'. Catches §0.2 drift.
  - log: `.planning/verifications/_logs/freshness__install-leads-with-pkg-mgr__docs-index.txt`
- `freshness/install-leads-with-pkg-mgr/README` — README.md must show a package-manager install command BEFORE any 'git clone' / 'cargo build --release'. Catches §0.2 drift.
  - log: `.planning/verifications/_logs/freshness__install-leads-with-pkg-mgr__README.txt`
- `freshness/no-loose-roadmap-or-requirements` — No loose v*ROADMAP*.md / v*REQUIREMENTS*.md outside *phases/ or .planning/archive/. §0.5.
  - log: `.planning/verifications/_logs/freshness__no-loose-roadmap-or-requirements.txt`

## PASS

- `freshness/benchmarks-in-mkdocs-nav`
- `freshness/no-orphan-docs`
- `mermaid-renders/how-it-works/filesystem-layer`
- `mermaid-renders/how-it-works/git-layer`
- `mermaid-renders/how-it-works/trust-model`
- `mermaid-renders/index`
- `mermaid-renders/reference/git-remote`
- `ci/main-green-on-latest-completed`
- `crates-io/reposix-core-at-workspace-version`
- `crates-io/reposix-cache-at-workspace-version`
- `crates-io/reposix-sim-at-workspace-version`
- `crates-io/reposix-github-at-workspace-version`
- `crates-io/reposix-confluence-at-workspace-version`
- `crates-io/reposix-jira-at-workspace-version`
- `crates-io/reposix-remote-at-workspace-version`
- `crates-io/reposix-cli-at-workspace-version`

---

_Verdict above is computed from the last `verify` run per claim. To refresh, run `python3 scripts/end-state.py verify` then re-run `verdict`._
