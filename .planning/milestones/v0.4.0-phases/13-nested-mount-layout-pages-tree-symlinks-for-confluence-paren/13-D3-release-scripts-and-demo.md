---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D3
type: execute
wave: 4
depends_on: [C]
files_modified:
  - scripts/tag-v0.4.0.sh
  - scripts/demos/07-mount-real-confluence-tree.sh
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "`scripts/tag-v0.4.0.sh` exists, is executable (`chmod +x`), and is structurally a copy of `scripts/tag-v0.3.0.sh` with every `v0.3.0`-specific string swapped to `v0.4.0`"
    - "`bash -n scripts/tag-v0.4.0.sh` exits 0 (syntax check)"
    - "`shellcheck scripts/tag-v0.4.0.sh` exits 0 if shellcheck is available; otherwise grep for common bash gotchas (unquoted `$*`, missing `set -euo pipefail`)"
    - "`scripts/demos/07-mount-real-confluence-tree.sh` exists, is executable, and demonstrates the `tree/` overlay against real Confluence (REPOSIX space on `reuben-john.atlassian.net`)"
    - "The Confluence-tree demo `skip-if-no-creds` cleanly: if `ATLASSIAN_API_KEY` unset, it prints a skip message and exits 0 (matching the pattern of `scripts/demos/06-mount-real-confluence.sh`)"
    - "The Confluence-tree demo uses the Wave-A+B helpers: `ls mount`, `cat mount/pages/00000360556.md`, `cd mount/tree/<slug>`, `readlink mount/tree/<slug>/welcome-to-reposix.md`, `cat mount/.gitignore`"
    - "The Confluence-tree demo cleans up: `fusermount3 -u` + `rmdir` on exit (trap EXIT ERR INT)"
    - "The Confluence-tree demo is NOT added to smoke.sh (smoke remains sim-only-4/4); it IS appropriate to mention in full.sh or leave as tier-5 standalone"
  artifacts:
    - path: "scripts/tag-v0.4.0.sh"
      provides: "Release-tagging script for v0.4.0"
      min_lines: 20
    - path: "scripts/demos/07-mount-real-confluence-tree.sh"
      provides: "Tier-5 live Confluence-tree demo"
      min_lines: 60
  key_links:
    - from: "scripts/tag-v0.4.0.sh"
      to: "scripts/tag-v0.3.0.sh"
      via: "structural copy"
      pattern: "v0\\.4\\.0"
    - from: "scripts/demos/07-mount-real-confluence-tree.sh"
      to: "scripts/demos/06-mount-real-confluence.sh"
      via: "skip-if-no-creds pattern + _lib.sh helpers"
      pattern: "ATLASSIAN_API_KEY"
---

<objective>
Wave-D3. Produce the release script + the new Tier-5 demo. Parallel-safe with D1 and D2 (disjoint file sets). Both scripts model after existing siblings — `tag-v0.3.0.sh` and `06-mount-real-confluence.sh` — keeping style and skip-gating identical.

Purpose: `scripts/tag-v0.4.0.sh` is what the user (or CI) runs to tag and trigger the release.yml binary upload. `07-mount-real-confluence-tree.sh` is the runnable asciinema-ready demo that captures the "hero.png" moment for the v0.4.0 blog post / social post.

Output: Two new executable shell scripts. No edits to existing scripts (if _lib.sh needs helpers, add them as additive new functions in the demo script itself — keep _lib.sh touches out of D3 to avoid D1/D3 conflicts).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@scripts/tag-v0.3.0.sh
@scripts/demos/06-mount-real-confluence.sh
@scripts/demos/_lib.sh
@scripts/demos/_record.sh

<interfaces>
<!-- The demo target (from CONTEXT.md §specifics): -->

```bash
mkdir -p /tmp/reposix-tree-mnt
reposix mount /tmp/reposix-tree-mnt --backend confluence --project REPOSIX &
sleep 3

ls /tmp/reposix-tree-mnt
# -> .gitignore  pages/  tree/

cat /tmp/reposix-tree-mnt/pages/00000131192.md | head -5

cd /tmp/reposix-tree-mnt/tree/reposix-demo-space-home
ls
# -> _self.md  architecture-notes.md  demo-plan.md  welcome-to-reposix.md

cat welcome-to-reposix.md | head -5
readlink welcome-to-reposix.md
# -> ../../pages/00000131192.md

cd /tmp/reposix-tree-mnt
cat .gitignore
# -> /tree/

fusermount3 -u /tmp/reposix-tree-mnt
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: `scripts/tag-v0.4.0.sh`</name>
  <files>
    scripts/tag-v0.4.0.sh
  </files>
  <action>
    Read `scripts/tag-v0.3.0.sh` carefully. Copy it to `scripts/tag-v0.4.0.sh`. Replace EVERY occurrence of:
    - `v0.3.0` → `v0.4.0`
    - `0.3.0` (bare) → `0.4.0` (for Cargo version checks)
    - any `v0.3` tag-reference → `v0.4`
    - any user-visible message referring to v0.3 feature-set (e.g. "Confluence read-only adapter") → v0.4 feature-set (e.g. "nested mount layout + Confluence tree/ overlay")

    Preserve every `set -euo pipefail`, trap, or sanity check from v0.3.0.

    If v0.3.0's script does a `grep -q '^version = "0.3.0"' Cargo.toml` pre-flight, update to `"0.4.0"` AND BE AWARE: the workspace root `Cargo.toml` version field must have been bumped by some earlier wave or needs bumping here. If not yet bumped, add a preflight:
    ```bash
    if ! grep -qE '^version = "0\.4\.0"' Cargo.toml; then
      echo "ERROR: Cargo.toml workspace version is not 0.4.0. Bump it before tagging."
      exit 1
    fi
    ```
    Do NOT bump the Cargo.toml version from within this script — it's a preflight check. The actual bump is a pre-tag commit the user makes (or a separate small commit in Wave E).

    If v0.3.0's script pushes to git, keep that behavior — matching the v0.3 release posture.

    Make the script executable:
    ```bash
    chmod +x scripts/tag-v0.4.0.sh
    ```

    Run `bash -n scripts/tag-v0.4.0.sh` to check syntax without executing.
    If `shellcheck` is available, `shellcheck scripts/tag-v0.4.0.sh` and fix any SC2086/SC2046-level issues.

    Commit: `chore(13-D3-1): add scripts/tag-v0.4.0.sh release script`.
  </action>
  <verify>
    <automated>test -x scripts/tag-v0.4.0.sh &amp;&amp; bash -n scripts/tag-v0.4.0.sh &amp;&amp; grep -q 'v0\.4\.0' scripts/tag-v0.4.0.sh &amp;&amp; ! grep -q 'v0\.3\.0' scripts/tag-v0.4.0.sh</automated>
  </verify>
  <done>
    `scripts/tag-v0.4.0.sh` exists, is executable, syntactically valid, contains `v0.4.0` references, contains zero `v0.3.0` strings. Shape mirrors v0.3.0 script.
  </done>
</task>

<task type="auto">
  <name>Task 2: `scripts/demos/07-mount-real-confluence-tree.sh`</name>
  <files>
    scripts/demos/07-mount-real-confluence-tree.sh
  </files>
  <action>
    Read `scripts/demos/06-mount-real-confluence.sh` and `scripts/demos/_lib.sh` to understand the existing pattern (skip-if-no-creds, mount + sleep, fusermount3 -u on trap, color output helpers).

    Create `scripts/demos/07-mount-real-confluence-tree.sh` following the same structure:

    ```bash
    #!/usr/bin/env bash
    # Tier-5 demo: Confluence nested-mount tree/ overlay against the real REPOSIX space.
    # Matches the hero walk-through from CONTEXT.md §specifics.
    # Skip-cleanly when ATLASSIAN_API_KEY is unset.

    set -euo pipefail

    HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    # shellcheck source=./_lib.sh
    source "$HERE/_lib.sh"

    demo_header "07: Confluence nested tree mount (real REPOSIX space)"

    # Skip gate — matches 06-mount-real-confluence.sh verbatim.
    if [ -z "${ATLASSIAN_API_KEY:-}" ]; then
      demo_skip "ATLASSIAN_API_KEY unset — skipping live Confluence tree demo."
      exit 0
    fi
    : "${ATLASSIAN_EMAIL:?ATLASSIAN_EMAIL must be set alongside ATLASSIAN_API_KEY}"
    : "${ATLASSIAN_TENANT:?ATLASSIAN_TENANT must be set (e.g. reuben-john)}"
    : "${REPOSIX_ALLOWED_ORIGINS:?must include https://${ATLASSIAN_TENANT}.atlassian.net}"

    MNT="$(mktemp -d -t reposix-tree-XXXX)"
    trap 'fusermount3 -u "$MNT" 2>/dev/null || fusermount -u "$MNT" 2>/dev/null; rmdir "$MNT" 2>/dev/null || true' EXIT INT ERR

    demo_step "1. Mount the REPOSIX space with the Confluence backend"
    cargo run -q --release -p reposix-cli -- mount "$MNT" --backend confluence --project REPOSIX &
    MOUNT_PID=$!
    sleep 3

    demo_step "2. Root layout — expect .gitignore, pages/, tree/"
    ls "$MNT"

    demo_step "3. .gitignore content — expect /tree/"
    cat "$MNT/.gitignore"

    demo_step "4. Flat bucket view"
    ls "$MNT/pages" | head -6
    echo "..."
    demo_step "   (first 5 lines of the homepage body)"
    cat "$MNT/pages/00000360556.md" | head -5

    demo_step "5. Hierarchical tree view — the hero.png moment"
    ls "$MNT/tree"
    echo
    cd "$MNT/tree/reposix-demo-space-home"
    demo_step "   Contents of tree/reposix-demo-space-home/"
    ls
    echo
    demo_step "   readlink welcome-to-reposix.md"
    readlink welcome-to-reposix.md
    demo_step "   cat welcome-to-reposix.md | head -5  (symlink resolves into pages/...)"
    cat welcome-to-reposix.md | head -5

    demo_step "6. Unmount"
    cd /
    kill $MOUNT_PID 2>/dev/null || true
    fusermount3 -u "$MNT" 2>/dev/null || fusermount -u "$MNT"
    trap - EXIT INT ERR
    rmdir "$MNT"

    demo_success "07-mount-real-confluence-tree: OK"
    ```

    Adapt the variable names (`demo_header`, `demo_step`, `demo_skip`, `demo_success`) to whatever `_lib.sh` actually exports. If `_lib.sh` uses different names, map appropriately — read the file first.

    Make executable:
    ```bash
    chmod +x scripts/demos/07-mount-real-confluence-tree.sh
    ```

    Syntax-check:
    ```bash
    bash -n scripts/demos/07-mount-real-confluence-tree.sh
    ```

    Run without creds to verify the skip path:
    ```bash
    unset ATLASSIAN_API_KEY
    bash scripts/demos/07-mount-real-confluence-tree.sh
    # Expected: exit 0 with a skip message.
    ```

    Do NOT attempt to run with live creds from this task (that's Wave E's job). Just confirm structural correctness + skip-path works.

    Commit: `feat(13-D3-2): scripts/demos/07-mount-real-confluence-tree.sh tier-5 demo`.
  </action>
  <verify>
    <automated>test -x scripts/demos/07-mount-real-confluence-tree.sh &amp;&amp; bash -n scripts/demos/07-mount-real-confluence-tree.sh &amp;&amp; (unset ATLASSIAN_API_KEY; bash scripts/demos/07-mount-real-confluence-tree.sh)</automated>
  </verify>
  <done>
    Demo script exists, is executable, syntactically valid, exits 0 in skip mode (no ATLASSIAN_API_KEY). When creds are present, Wave E will run it against live REPOSIX.
  </done>
</task>

<task type="auto">
  <name>Task 3: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    No code changes to regress anything; just spot-check that smoke.sh still passes (D1 should have ensured this already, but we're running in parallel with D1 and D2 so belt-and-braces):
    ```bash
    bash scripts/demos/smoke.sh
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    ```
  </action>
  <verify>
    <automated>bash scripts/demos/smoke.sh</automated>
  </verify>
  <done>
    smoke.sh 4/4 green. No regressions.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Demo script → Atlassian tenant | The demo uses user-supplied `ATLASSIAN_API_KEY` + `REPOSIX_ALLOWED_ORIGINS`. SG-01 enforcement is baked into the adapter (re-checked on every outbound request); the demo cannot bypass it even if it tried. |
| Tag script → git + GitHub | `tag-v0.4.0.sh` creates a git tag and may push. User's git creds handle auth. No new attack surface vs. v0.3. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-D3-1 | Information disclosure | Demo script echoes `ATLASSIAN_API_KEY` | mitigate | The script only reads the env var; never `echo`s it. Mirrored from 06-mount-real-confluence.sh which already follows this rule. |
| T-13-D3-2 | Tampering | Tag script pushed to a fork / wrong remote | accept | User runs this manually; `git remote -v` is their check. Same posture as v0.3. |
</threat_model>

<verification>
Nyquist coverage:
- **Syntax:** `bash -n` on both scripts.
- **Skip-path:** Demo exits 0 with `ATLASSIAN_API_KEY` unset (confirms skip gate works).
- **No smoke regression:** smoke.sh still 4/4 green.
- **Live run:** deferred to Wave E.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -x scripts/tag-v0.4.0.sh` exits 0.
2. `bash -n scripts/tag-v0.4.0.sh` exits 0.
3. `grep -qE 'v0\.4\.0' scripts/tag-v0.4.0.sh` exits 0.
4. `! grep -qE 'v0\.3\.0' scripts/tag-v0.4.0.sh` exits 0 (no leftover v0.3 strings).
5. `test -x scripts/demos/07-mount-real-confluence-tree.sh` exits 0.
6. `bash -n scripts/demos/07-mount-real-confluence-tree.sh` exits 0.
7. `grep -q 'ATLASSIAN_API_KEY' scripts/demos/07-mount-real-confluence-tree.sh` exits 0 (skip-gate present).
8. `grep -q 'readlink' scripts/demos/07-mount-real-confluence-tree.sh` exits 0 (exercises the key new feature).
9. `(unset ATLASSIAN_API_KEY; bash scripts/demos/07-mount-real-confluence-tree.sh)` exits 0 (skip-path works).
10. `bash scripts/demos/smoke.sh` exits 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D3-SUMMARY.md` documenting:
- Paste the `tag-v0.4.0.sh` diff vs `tag-v0.3.0.sh` (git diff-style)
- List of the 6 steps in the demo script
- Skip-path output (copy the "skipping" message)
- Any divergence from the 06-mount-real-confluence.sh template (should be minimal)
</output>
