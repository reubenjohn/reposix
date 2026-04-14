---
phase: 11-confluence-adapter
plan: F
type: execute
wave: 3
depends_on:
  - A
  - B
  - C
  - D
  - E
files_modified:
  - MORNING-BRIEF-v0.3.md
  - CHANGELOG.md
  - scripts/tag-v0.3.0.sh
autonomous: false
requirements:
  - FC-09
  - SG-08
user_setup:
  - service: git-tag-push
    why: "Tag v0.3.0 and push to GitHub — triggers release discovery and CHANGELOG visibility"
    dashboard_config:
      - task: "Optionally create a GitHub Release from the v0.3.0 tag with copy-pasted CHANGELOG v0.3.0 section"
        location: "https://github.com/reubenjohn/reposix/releases/new?tag=v0.3.0"

must_haves:
  truths:
    - "`MORNING-BRIEF-v0.3.md` at repo root tells the user exactly what to set (ATLASSIAN_EMAIL correction) and exactly what to run to prove live-mount works"
    - "CHANGELOG.md has a new `## [v0.3.0] — 2026-04-14` header promoted from Unreleased"
    - "`## [Unreleased]` section is restored to `— Nothing yet.` (or equivalent) below the new v0.3.0 header"
    - "`scripts/tag-v0.3.0.sh` is an executable script that creates an annotated tag `v0.3.0` with a release-notes message and pushes it (only when user explicitly runs the script — it does NOT auto-run)"
    - "The final human-verification checkpoint confirms workspace green + smoke.sh 4/4 + docs reviewed before the tag is actually pushed"
    - "The tag is NOT pushed autonomously by this plan — `git tag ... && git push origin v0.3.0` runs ONLY behind a checkpoint gate"
  artifacts:
    - path: "MORNING-BRIEF-v0.3.md"
      provides: "Human-facing morning brief: credential fix, re-run commands, summary of what shipped and what's still open"
      min_lines: 100
    - path: "scripts/tag-v0.3.0.sh"
      provides: "Release script — annotated tag, docs block, CHANGELOG excerpt embedded, single-purpose and idempotent-safe"
      min_lines: 40
    - path: "CHANGELOG.md"
      provides: "Promoted [v0.3.0] section with release date + restored [Unreleased] placeholder"
  key_links:
    - from: "MORNING-BRIEF-v0.3.md"
      to: "scripts/tag-v0.3.0.sh"
      via: "Instructions tell the user when/how to run it"
      pattern: "scripts/tag-v0.3.0.sh"
    - from: "scripts/tag-v0.3.0.sh"
      to: "CHANGELOG.md v0.3.0 section"
      via: "Annotated tag message references the CHANGELOG"
      pattern: "CHANGELOG.md"
---

<objective>
Close out Phase 11 with the release artifacts: a morning-brief document so the sleeping user wakes up to exact run commands (including the credential fix per 00-CREDENTIAL-STATUS.md), a CHANGELOG promotion from [Unreleased] to [v0.3.0], and a `scripts/tag-v0.3.0.sh` release helper that ONLY creates + pushes the tag when the user explicitly runs it. The actual tag push happens behind a `checkpoint:human-verify` gate so the orchestrator cannot ship a broken tag autonomously.

Purpose: v0.3.0 is a visible release. The user needs a one-screen orientation document to resume, and the tag itself is a one-way action that must be gated on human verification. 11-F runs last (Wave 3) because everything else must be landed and green before we declare a release.

Output: Two new files (`MORNING-BRIEF-v0.3.md`, `scripts/tag-v0.3.0.sh`) + one CHANGELOG edit. The tag itself is optionally pushed at the end (only if checkpoint approves).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md
@CLAUDE.md
@HANDOFF.md
@MORNING-BRIEF.md
@CHANGELOG.md
@.planning/phases/11-confluence-adapter/11-A-confluence-crate-core.md
@.planning/phases/11-confluence-adapter/11-B-cli-dispatch.md
@.planning/phases/11-confluence-adapter/11-C-contract-test.md
@.planning/phases/11-confluence-adapter/11-D-demos.md
@.planning/phases/11-confluence-adapter/11-E-docs-and-env.md

<interfaces>
Git tagging convention (from prior releases):
```bash
git tag -a v0.3.0 -m "<one-line summary>
<multi-line body referencing CHANGELOG>"
git push origin v0.3.0
```

Prior morning-brief structure (`MORNING-BRIEF.md` at repo root) — copy its shape:
- one-screen summary of what shipped
- any user-required follow-ups
- suggested next actions
- tone: direct, no marketing
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write MORNING-BRIEF-v0.3.md</name>
  <files>
    MORNING-BRIEF-v0.3.md
  </files>
  <action>
    Create `MORNING-BRIEF-v0.3.md` at repo root. Required sections in this order:

    **`# MORNING-BRIEF — v0.3.0`** with date and "from overnight session 3" footer.

    **`## tl;dr`** — three or four sentences. Phase 11 shipped read-only Atlassian Confluence support. Live-wire verification could NOT run tonight because the API token and email don't pair to the same Atlassian account (see §credentials below). The library-level proof (wiremock + contract test + CI gate) passes. The 30-second user unblock is §credentials.

    **`## What shipped`** — short bullet list pointing to 11-E's CHANGELOG section. Link to CHANGELOG.md at the `[v0.3.0]` header.

    **`## 30-second fix: credentials`** — copy-paraphrase from `00-CREDENTIAL-STATUS.md`:
    1. Visit <https://id.atlassian.com/manage-profile/security/api-tokens>
    2. Note the email at the top right. This is the value for `ATLASSIAN_EMAIL`.
    3. Decide a tenant name (subdomain of `*.atlassian.net`). Confirm with `curl -s https://YOUR_TENANT.atlassian.net/wiki/rest/api/space | head -c 200` returns something non-empty.
    4. Pick a space key (e.g. from the URL of any page in your Confluence: `https://<tenant>.atlassian.net/wiki/spaces/<KEY>/...`).

    **`## Prove it works (commands to copy-paste)`** — end-to-end:
    ```bash
    # From the repo root
    cd /path/to/reposix

    # Update/confirm .env contains:
    cat > .env <<'EOF'
    ATLASSIAN_API_KEY=<your token>
    ATLASSIAN_EMAIL=<the email from step 2 above>
    REPOSIX_CONFLUENCE_TENANT=<your tenant subdomain>
    REPOSIX_CONFLUENCE_SPACE=<any space key you can read>
    EOF

    # Export them
    set -a; source .env; set +a
    export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

    # Build release binaries (you may have them from last night)
    cargo build --release --workspace --bins
    export PATH="$PWD/target/release:$PATH"

    # (A) list a real Confluence space from CLI
    reposix list --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" --format table

    # (B) FUSE-mount a real Confluence space
    mkdir -p /tmp/reposix-conf-mnt
    reposix mount /tmp/reposix-conf-mnt --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" &
    sleep 3
    ls /tmp/reposix-conf-mnt | head -20
    cat /tmp/reposix-conf-mnt/*.md | head -50
    fusermount3 -u /tmp/reposix-conf-mnt

    # (C) the two new demos
    bash scripts/demos/parity-confluence.sh
    bash scripts/demos/06-mount-real-confluence.sh

    # (D) full workspace tests (including live contract test)
    cargo test --workspace --locked
    cargo test -p reposix-confluence -- --ignored    # live half
    bash scripts/demos/smoke.sh                      # Tier 1 regression check
    ```

    **`## CI secrets (one-shot)`** — the `gh secret set` commands the user can run to enable `integration-contract-confluence` in CI:
    ```bash
    gh secret set ATLASSIAN_API_KEY        --body "$ATLASSIAN_API_KEY"
    gh secret set ATLASSIAN_EMAIL          --body "$ATLASSIAN_EMAIL"
    gh secret set REPOSIX_CONFLUENCE_TENANT --body "$REPOSIX_CONFLUENCE_TENANT"
    gh secret set REPOSIX_CONFLUENCE_SPACE --body "$REPOSIX_CONFLUENCE_SPACE"
    ```
    Note that these secrets are required for the `integration-contract-confluence` CI job to actually run; without them the `if:` clause skips the job (not a failure, but also no proof).

    **`## Cutting the tag`** — instruct the user to:
    1. Eyeball `CHANGELOG.md` [v0.3.0] section.
    2. Confirm `bash scripts/demos/smoke.sh` green on the dev host.
    3. Run `bash scripts/tag-v0.3.0.sh` to create + push the tag.
    4. (Optional) Create a GitHub release from <https://github.com/reubenjohn/reposix/releases/new?tag=v0.3.0>, paste the CHANGELOG v0.3.0 section as the body.

    **`## Known open gaps`** (be loud, per CLAUDE.md OP #6 "ground truth obsession"):
    - Live-wire empirical proof is deferred to the user; autonomous session could not exercise it due to the credential mismatch from 00-CREDENTIAL-STATUS.md.
    - PageBackend trait (Option B) is deferred to v0.4 per ADR-002.
    - Write path (`create_issue`, `update_issue`, `delete_or_close`) returns `NotSupported` — v0.4.
    - `atlas_doc_format` → Markdown rendering — v0.4 (body is raw storage XHTML in v0.3).
    - Labels, attachments, comments, page hierarchy — v0.4+.

    **`## Handoff`** — point the next overnight agent (if applicable) at this brief, `HANDOFF.md`, `.planning/phases/11-confluence-adapter/`, and `CHANGELOG.md` v0.3.0.

    **`## Sign-off`** — signed by Claude Opus 4.6 1M context, date 2026-04-13/14 (overnight).

    Target length: 100-200 lines. Prioritize CTRL-F-ability.
  </action>
  <verify>
    <automated>test -f MORNING-BRIEF-v0.3.md &amp;&amp; [ "$(wc -l &lt; MORNING-BRIEF-v0.3.md)" -ge 100 ] &amp;&amp; grep -q 'ATLASSIAN_EMAIL' MORNING-BRIEF-v0.3.md &amp;&amp; grep -q 'REPOSIX_CONFLUENCE_TENANT' MORNING-BRIEF-v0.3.md &amp;&amp; grep -q 'REPOSIX_CONFLUENCE_SPACE' MORNING-BRIEF-v0.3.md &amp;&amp; grep -q 'scripts/tag-v0.3.0.sh' MORNING-BRIEF-v0.3.md &amp;&amp; grep -qE 'Known open gaps|Known gaps|open gaps' MORNING-BRIEF-v0.3.md &amp;&amp; grep -q 'id.atlassian.com/manage-profile/security/api-tokens' MORNING-BRIEF-v0.3.md</automated>
  </verify>
  <done>
    MORNING-BRIEF-v0.3.md exists, ≥100 lines, covers credential fix + run commands + CI secrets + tag instructions + gaps. Commit: `docs(11-F-1): MORNING-BRIEF-v0.3.md for post-overnight handoff`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Promote CHANGELOG [Unreleased] → [v0.3.0]</name>
  <files>
    CHANGELOG.md
  </files>
  <action>
    After 11-E's Unreleased edits landed, re-open `CHANGELOG.md` and:

    1. Rename the `## [Unreleased]` header to `## [v0.3.0] — 2026-04-14`.
    2. Below the new `## [v0.3.0]` line, insert a new empty `## [Unreleased]` section with `— Nothing yet.`:
       ```markdown
       ## [Unreleased]

       — Nothing yet.

       ## [v0.3.0] — 2026-04-14

       The "real Confluence" cut. v0.2 shipped read-only GitHub; this release adds read-only Atlassian Confluence Cloud as a sibling backend via the same `IssueBackend` seam.

       ### Added
       ...  (content from 11-E)
       ### Changed
       ...
       ```
    3. Do NOT rewrite the content from 11-E — just move/retitle it.
    4. If 11-E's Changed section contains a breaking-change note (env-var rename), add a `### BREAKING` callout above Added:
       ```markdown
       ### BREAKING
       - `.env` variable `TEAMWORK_GRAPH_API` renamed to `ATLASSIAN_API_KEY`. Users with an existing `.env` must rename the variable OR re-source after updating `.env.example`.
       ```

    Validate the CHANGELOG still parses (no tool available, but visual check for balanced headers):
    ```bash
    grep -cE '^## \[' CHANGELOG.md
    # expect: 1 (Unreleased) + 1 (v0.3.0) + N (prior) — at least 3
    ```
  </action>
  <verify>
    <automated>grep -qE '^## \[v0\.3\.0\] — 2026-04-14' CHANGELOG.md &amp;&amp; grep -qE '^## \[Unreleased\]$' CHANGELOG.md &amp;&amp; [ "$(grep -cE '^## \[' CHANGELOG.md)" -ge 3 ] &amp;&amp; sed -n '/## \[v0.3.0\]/,/## \[v0.2.0-alpha\]/p' CHANGELOG.md | grep -q 'reposix-confluence'</automated>
  </verify>
  <done>
    v0.3.0 header promoted; new empty [Unreleased] above it; BREAKING callout for the env-var rename. Commit: `docs(11-F-2): promote CHANGELOG v0.3.0 + restore empty [Unreleased]`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Write `scripts/tag-v0.3.0.sh`</name>
  <files>
    scripts/tag-v0.3.0.sh
  </files>
  <action>
    Create `scripts/tag-v0.3.0.sh`. This script is the user-facing "push the tag" helper. It DOES run the tag-creation + push, but only when the user explicitly invokes it. It contains guardrails so running it accidentally (e.g. on a dirty tree) does NOT ship a broken tag.

    Required structure:
    ```bash
    #!/usr/bin/env bash
    # scripts/tag-v0.3.0.sh — create and push the v0.3.0 annotated tag.
    #
    # Only run this after:
    #   1. cargo test --workspace --locked green
    #   2. cargo clippy --workspace --all-targets -- -D warnings green
    #   3. bash scripts/demos/smoke.sh 4/4 green
    #   4. You've eyeballed the CHANGELOG [v0.3.0] section
    #   5. You've at least read MORNING-BRIEF-v0.3.md
    #
    # Safety guards (the script enforces):
    #   - Current branch must be `main`
    #   - Working tree must be clean (no uncommitted or untracked changes)
    #   - v0.3.0 tag must NOT already exist locally OR on origin
    #   - CHANGELOG.md must contain a `## [v0.3.0]` header
    #
    # Usage:  bash scripts/tag-v0.3.0.sh

    set -euo pipefail

    TAG="v0.3.0"

    # 1. Branch guard
    CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$CURRENT_BRANCH" != "main" ]]; then
        echo "ERROR: not on main (current branch: $CURRENT_BRANCH)" >&2
        exit 1
    fi

    # 2. Clean tree guard
    if ! git diff --quiet HEAD 2>/dev/null || [[ -n "$(git status --porcelain)" ]]; then
        echo "ERROR: working tree is not clean" >&2
        git status --short >&2
        exit 1
    fi

    # 3. Tag-doesn't-exist guard (local)
    if git rev-parse --verify "refs/tags/$TAG" >/dev/null 2>&1; then
        echo "ERROR: local tag $TAG already exists" >&2
        exit 1
    fi

    # 4. Tag-doesn't-exist guard (remote)
    if git ls-remote --tags origin "$TAG" | grep -q "refs/tags/$TAG"; then
        echo "ERROR: remote tag $TAG already exists on origin" >&2
        exit 1
    fi

    # 5. CHANGELOG section present
    if ! grep -qE '^## \[v0\.3\.0\]' CHANGELOG.md; then
        echo "ERROR: CHANGELOG.md has no '## [v0.3.0]' section" >&2
        exit 1
    fi

    # 6. All tests green (belt-and-suspenders; fast fail)
    echo "==> verifying workspace is green..."
    cargo test --workspace --locked
    bash scripts/demos/smoke.sh

    # 7. Extract v0.3.0 body from CHANGELOG for the tag message
    CHANGELOG_BODY="$(sed -n '/^## \[v0.3.0\]/,/^## \[/p' CHANGELOG.md | sed '$d')"

    echo "==> creating annotated tag $TAG"
    git tag -a "$TAG" -m "reposix $TAG — read-only Confluence Cloud adapter

See CHANGELOG.md for the full release notes.

$CHANGELOG_BODY
"

    echo "==> pushing $TAG to origin"
    git push origin "$TAG"

    echo
    echo "== TAG $TAG PUSHED =="
    echo "Optional: create a GitHub release at"
    echo "  https://github.com/reubenjohn/reposix/releases/new?tag=$TAG"
    echo "and paste the CHANGELOG v0.3.0 section as the body."
    ```

    `chmod +x scripts/tag-v0.3.0.sh`.

    Manual dry-run — just parse the script to catch bash errors:
    ```bash
    bash -n scripts/tag-v0.3.0.sh
    ```
  </action>
  <verify>
    <automated>test -x scripts/tag-v0.3.0.sh &amp;&amp; bash -n scripts/tag-v0.3.0.sh &amp;&amp; grep -q 'set -euo pipefail' scripts/tag-v0.3.0.sh &amp;&amp; grep -q 'CURRENT_BRANCH' scripts/tag-v0.3.0.sh &amp;&amp; grep -q 'git tag -a' scripts/tag-v0.3.0.sh &amp;&amp; grep -q 'git push origin' scripts/tag-v0.3.0.sh</automated>
  </verify>
  <done>
    Release script exists, is executable, parses cleanly, has all five safety guards. Commit: `chore(11-F-3): scripts/tag-v0.3.0.sh release helper with safety guards`.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: Human verification gate — review + run the tag script</name>
  <what-built>
    After 11-A through 11-F Task 3:
      - `reposix-confluence` crate with 10+ wiremock unit tests
      - CLI dispatch for `--backend confluence` in `reposix list` / `reposix mount` / `reposix-fuse`
      - CI `integration-contract-confluence` job gated on Atlassian secrets
      - Contract test `tests/contract.rs` parameterized over sim + wiremock + live
      - Two demos: `parity-confluence.sh` (Tier 3B) + `06-mount-real-confluence.sh` (Tier 5)
      - ADR-002 + `docs/reference/confluence.md` + README + architecture + CHANGELOG updates + .env.example rename
      - `MORNING-BRIEF-v0.3.md` + `scripts/tag-v0.3.0.sh`
      - CHANGELOG promoted to v0.3.0 with BREAKING callout for the env-var rename
  </what-built>
  <how-to-verify>
    From the repo root, confirm the full gate is green:

    ```bash
    # 1. Fast build + lint + test
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked

    # 2. Tier 1 demos untouched
    cargo build --release --workspace --bins --locked
    PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh
    # expect: 4 passed, 0 failed

    # 3. New demos SKIP cleanly with no env
    env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE \
      bash scripts/demos/parity-confluence.sh
    env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE \
      bash scripts/demos/06-mount-real-confluence.sh
    # expect: both show "SKIP:" and end with "== DEMO COMPLETE =="

    # 4. Contract test SKIPs live half cleanly
    env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE \
      cargo test -p reposix-confluence --locked -- --ignored --nocapture 2>&1 | grep 'SKIP:'

    # 5. CHANGELOG promoted
    grep -E '^## \[(v0\.3\.0|Unreleased)\]' CHANGELOG.md
    # expect: two lines, [Unreleased] above [v0.3.0] — 2026-04-14

    # 6. Release script dry-run (don't push yet)
    bash -n scripts/tag-v0.3.0.sh  # parse check
    cat scripts/tag-v0.3.0.sh | head -30  # eyeball the guards

    # 7. Morning-brief readable
    head -40 MORNING-BRIEF-v0.3.md
    ```

    If ALL of 1-7 are green AND you (the user) have reviewed the CHANGELOG + ADR-002, you may type `approved` to unblock Task 5 (actual tag push). If ANY step fails, describe the failure and DO NOT type approved — the orchestrator should route the fix to a gap-closure plan.
  </how-to-verify>
  <resume-signal>Type "approved" to proceed to the tag push, or describe issues.</resume-signal>
</task>

<task type="auto">
  <name>Task 5: Execute `scripts/tag-v0.3.0.sh` (only after approval)</name>
  <files>
    (runtime only — the script was created in Task 3; this task is its invocation)
  </files>
  <action>
    ONLY runs after Task 4's `approved` signal.

    Run:
    ```bash
    bash scripts/tag-v0.3.0.sh
    ```

    The script itself enforces the six safety guards (branch=main, clean tree, tag doesn't already exist locally or remotely, CHANGELOG has v0.3.0 section, cargo test green, smoke demos green). If ANY guard fails, the script exits non-zero WITHOUT tagging — surface the error to the user, do NOT bypass.

    On success, confirm:
    ```bash
    git tag -l v0.3.0                              # local tag exists
    git ls-remote --tags origin v0.3.0             # remote tag exists
    ```

    Per ~/.claude/CLAUDE.md OP #1 ("close the feedback loop"), after the push check that CI runs:
    ```bash
    sleep 10  # let GitHub register the new tag
    gh run list --limit 3
    ```
    Eyeball that the push triggered CI (or if the repo only runs CI on branch push, not tag push, note that the tag itself doesn't run CI but the commit it points to already has green CI from earlier pushes).

    Commit: none needed — the tag IS the artifact.
  </action>
  <verify>
    <automated>git ls-remote --tags origin v0.3.0 | grep -q 'refs/tags/v0.3.0' &amp;&amp; git tag -l v0.3.0 | grep -q '^v0.3.0$'</automated>
  </verify>
  <done>
    v0.3.0 tag exists locally and on origin. CI status visible in `gh run list`. Phase 11 complete.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `MORNING-BRIEF-v0.3.md` → user action | The brief tells the user what commands to run. If it contains a wrong command, the user runs wrong commands — a documentation-integrity issue, not an exploit. |
| `scripts/tag-v0.3.0.sh` → repository state | The script pushes a tag, which is a permanent and widely-visible artifact. The six safety guards prevent a bad tag. |
| Git tag → downstream consumers | Anyone with the clone URL sees the v0.3.0 tag; force-deleting it later is hard (GitHub preserves some references). Getting it right the first time matters. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11F-01 | Tampering | Tag created on a dirty tree → ships unintended code | mitigate | Guard 2 (working tree must be clean) in `tag-v0.3.0.sh`. Verified in Task 3's script structure. |
| T-11F-02 | Tampering | Tag created on wrong branch | mitigate | Guard 1 (must be on `main`). |
| T-11F-03 | Tampering / repudiation | Tag created despite failing tests | mitigate | Guard 6 (`cargo test --workspace --locked && bash scripts/demos/smoke.sh` inside the script). Belt-and-suspenders alongside the Task 4 human-verify checkpoint. |
| T-11F-04 | DoS | Tag `v0.3.0` already exists; re-pushing creates confusion | mitigate | Guards 3+4 (tag must not exist locally OR remotely). Script exits non-zero without clobbering. |
| T-11F-05 | Information disclosure | MORNING-BRIEF-v0.3.md containing real credential values | mitigate | The brief uses `<your token>` / `<your tenant>` placeholders, never real values. Pre-commit visual check: `git diff MORNING-BRIEF-v0.3.md | grep -E 'ATLASSIAN_API_KEY=[A-Za-z0-9]+'` should return empty. |
| T-11F-06 | Elevation of privilege | Autonomous tag push without human review | mitigate | Task 4 is `checkpoint:human-verify gate="blocking"`. Task 5 runs ONLY after `approved`. The script itself enforces Guard 6 (tests green) as a second line of defense. |

Block-on-high: T-11F-06 — autonomous tag push is the single biggest risk in this plan. The checkpoint gate IS the mitigation; the executor MUST NOT skip Task 4 in yolo mode. The plan's `autonomous: false` frontmatter signals this to the orchestrator.
</threat_model>

<verification>
Nyquist coverage:
- **Artifact existence + content:** each of Tasks 1-3 has a `<verify>` with a Bash command.
- **Bash safety:** Task 3's `bash -n scripts/tag-v0.3.0.sh` proves the script parses.
- **Script guards:** Task 4's manual review + Task 5's actual run exercise all six guards.
- **Regression:** Task 4's verification runs `cargo test --workspace --locked` + `smoke.sh` before the tag push.
- **Close-the-loop:** Task 5 checks `gh run list` per OP #1.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f MORNING-BRIEF-v0.3.md` returns 0.
2. `[ "$(wc -l < MORNING-BRIEF-v0.3.md)" -ge 100 ]` returns 0.
3. `grep -q 'id.atlassian.com/manage-profile/security/api-tokens' MORNING-BRIEF-v0.3.md` returns 0.
4. `grep -q 'scripts/tag-v0.3.0.sh' MORNING-BRIEF-v0.3.md` returns 0.
5. `grep -qE '(Known open gaps|Known gaps)' MORNING-BRIEF-v0.3.md` returns 0.
6. `test -x scripts/tag-v0.3.0.sh` returns 0.
7. `bash -n scripts/tag-v0.3.0.sh` returns 0.
8. `grep -q 'CURRENT_BRANCH' scripts/tag-v0.3.0.sh && grep -q 'git diff --quiet' scripts/tag-v0.3.0.sh && grep -q 'git ls-remote --tags origin' scripts/tag-v0.3.0.sh && grep -q 'cargo test --workspace --locked' scripts/tag-v0.3.0.sh && grep -q 'bash scripts/demos/smoke.sh' scripts/tag-v0.3.0.sh` returns 0 (all six guards present).
9. `grep -qE '^## \[v0\.3\.0\] — 2026-04-14' CHANGELOG.md` returns 0.
10. `grep -qE '^## \[Unreleased\]$' CHANGELOG.md` returns 0.
11. `sed -n '/## \[v0.3.0\]/,/## \[v0.2.0-alpha\]/p' CHANGELOG.md | grep -qE '(BREAKING|breaking change)'` returns 0 (env-var rename callout).
12. After Task 5 runs: `git tag -l v0.3.0 | grep -q '^v0.3.0$'` returns 0.
13. After Task 5 runs: `git ls-remote --tags origin v0.3.0 | grep -q 'refs/tags/v0.3.0'` returns 0.
14. After Task 5 runs: `gh run list --limit 5` shows no freshly-failed jobs triggered by the tag push (tag-triggered CI is informational, not load-bearing).

Criteria 12-14 only apply after Task 4's human-verify gate approved.
</success_criteria>

<rollback_plan>
If Task 4's checkpoint uncovers a regression (tests fail, smoke.sh fails, CHANGELOG wrong):
1. Do NOT type "approved".
2. Run `/gsd-plan-phase 11 --gaps` to generate a gap-closure plan for the specific failures.
3. After the gap plan ships and tests return green, re-invoke Task 4.

If Task 5's `scripts/tag-v0.3.0.sh` fails a guard mid-run:
1. Fix the root cause — dirty tree? commit or stash. Tag already exists? Investigate how (someone else pushed? A previous run got as far as the push?). Tests red? Revert to a known-good commit and re-run Task 4.
2. Do NOT override the guard. The guard exists because a bad tag is visible and permanent.

If the tag gets pushed and then you realize a fix is needed:
1. DO NOT `git push --delete origin v0.3.0 && git tag -d v0.3.0` silently. That's destructive per ~/.claude/CLAUDE.md OP — requires user approval.
2. Instead, prepare a `v0.3.1` with the fix. Document in CHANGELOG.
3. Only re-use the `v0.3.0` tag name if the user explicitly authorizes it. Force-push a tag only when approved.

If `MORNING-BRIEF-v0.3.md` accidentally contained a real credential (T-11F-05):
1. The file is in git history. Use `git log -p MORNING-BRIEF-v0.3.md` to confirm.
2. If the leaked credential is the Atlassian API token, revoke it at id.atlassian.com IMMEDIATELY.
3. Rewrite history is out of scope without explicit user approval; prefer "revoke + rotate" over "rewrite history".
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-F-SUMMARY.md` with:
- Confirmation of each artifact (MORNING-BRIEF, tag script, CHANGELOG promotion).
- Whether Task 4 checkpoint passed (yes / no + reason if no).
- If Task 5 ran: the commit hash the tag points to, plus the `gh run list` output's first three rows.
- Any deferred items surfaced during human verification that should seed Phase 12's backlog.
</output>
