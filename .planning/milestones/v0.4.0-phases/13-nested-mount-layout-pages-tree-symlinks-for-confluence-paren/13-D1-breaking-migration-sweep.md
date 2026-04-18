---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D1
type: execute
wave: 4
depends_on: [C]
files_modified:
  - scripts/demos/01-edit-and-push.sh
  - scripts/demos/02-guardrails.sh
  - scripts/demos/03-conflict-resolution.sh
  - scripts/demos/04-token-economy.sh
  - scripts/demos/05-mount-real-github.sh
  - scripts/demos/06-mount-real-confluence.sh
  - scripts/demos/full.sh
  - scripts/demos/smoke.sh
  - scripts/demos/parity.sh
  - scripts/demos/parity-confluence.sh
  - scripts/demo.sh
  - README.md
  - docs/demo.md
  - docs/social/assets/_build_demo_gif.py
  - docs/social/assets/_build_hero_filebrowser.py
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "Every reference to the flat `mount/<id>.md` path in demo scripts, README, docs, docstrings, and social-asset generators now uses `mount/issues/<id>.md` (sim + GitHub) or `mount/pages/<id>.md` (Confluence) as appropriate"
    - "`bash scripts/demos/smoke.sh` exits 0 (4/4 green) against the new layout"
    - "`bash scripts/demos/full.sh` exits 0 against the new layout"
    - "`grep -rInE 'mount[^/]*/0+[0-9]+\\.md|reposix-mnt/0+[0-9]+\\.md' scripts/ docs/ README.md` returns zero matches EXCEPT paths containing `/pages/` or `/issues/` (verified with a negative-match pattern)"
    - "The existing `crates/reposix-fuse/tests/sim_death_no_hang.rs` is updated to the new layout (if it reads a path at mount root)"
    - "No demo or doc example asserts a path that would now return ENOENT under the new layout"
  artifacts:
    - path: "scripts/demos/*.sh"
      provides: "All demo scripts patched for new layout; smoke.sh + full.sh still pass"
    - path: "README.md"
      provides: "Quickstart / usage snippets reflect new layout"
    - path: "docs/demo.md"
      provides: "Walkthrough updated"
  key_links:
    - from: "scripts/demos/*.sh"
      to: "mount/$BUCKET/<id>.md"
      via: "Shell variable for bucket name OR hard-coded per-backend path"
      pattern: "mount.*(pages|issues)/"
---

<objective>
Wave-D1. The BREAKING change sweep. Every demo script, doc example, README snippet, social-asset generator, and test file that referenced the old flat `mount/<id>.md` path is patched to use `mount/issues/<id>.md` (sim + GitHub backends) or `mount/pages/<id>.md` (Confluence backend). `smoke.sh` and `full.sh` continue to exit 0.

Purpose: Phase 13 flips the mount layout. Without this sweep, every demo breaks. D1 catalogs every occurrence (via grep), classifies by backend, patches in one atomic wave. Parallel-safe with D2 (docs+ADR) and D3 (new scripts) because their file sets are disjoint — but the `README.md` is shared between D1 (inline path examples inside code blocks) and D2 (new top-level section). D1 patches code-block contents only; D2 appends new sections.

Output: Line-level edits to 10-15 files. No new files, no code architecture changes.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-C-fuse-wiring.md
@CLAUDE.md
@scripts/demos/_lib.sh
@scripts/demos/smoke.sh
@scripts/demos/full.sh
@scripts/demos/05-mount-real-github.sh
@scripts/demos/06-mount-real-confluence.sh
@README.md
@docs/demo.md

<interfaces>
<!-- Backend → bucket mapping (from Wave A + B1): -->

| Backend | `root_collection_name()` | Bucket path in mount |
|---|---|---|
| SimBackend | `"issues"` | `mount/issues/<padded>.md` |
| GithubReadOnlyBackend | `"issues"` | `mount/issues/<padded>.md` |
| ConfluenceReadOnlyBackend | `"pages"` | `mount/pages/<padded>.md` |

<!-- Grep patterns for the sweep: -->

```bash
# Find all occurrences of old flat path.
# Match: "mount/" or "reposix-mnt/" or similar followed by a padded numeric ID + ".md",
# excluding paths that already have /issues/ or /pages/ between them.
grep -rInE 'mount[^/]*/0+[0-9]+\.md|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md

# Also check for variable expansions like $MNT/$ID.md or ${MNT}/$ID.md
grep -rInE '\$\{?MNT\}?/\$?\{?[A-Z_]*ID\}?\.md' scripts/ docs/
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Audit all callsites — build the full patch list</name>
  <files>
    (no edits; audit only)
  </files>
  <action>
    Produce an exhaustive list of every file + line needing a path update. Use these greps (save output to `/tmp/13-D1-audit.txt`):

    ```bash
    # 1. Direct padded-numeric references under a mount path
    grep -rInE 'mount[^/]*/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null | grep -vE '/(issues|pages)/' > /tmp/13-D1-audit.txt

    # 2. Shell variable expansions likely referring to a mount path
    grep -rInE '\$\{?MNT\}?/\$?\{?[A-Z_]*ID\}?\.md|\$\{?MOUNT\}?/\$?\{?[A-Z_]*ID\}?\.md' scripts/ docs/ 2>/dev/null >> /tmp/13-D1-audit.txt

    # 3. Explicit demo-path patterns
    grep -rInE 'reposix-mnt/0+[0-9]+\.md|\$MNT/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null >> /tmp/13-D1-audit.txt

    # 4. Docs/demo-specific patterns
    grep -rInE '(cat|ls|grep -r)\s+(\$MNT|/tmp/reposix-mnt|mount)[^/]*?/[A-Za-z0-9_.-]+\.md' docs/ README.md 2>/dev/null >> /tmp/13-D1-audit.txt

    # Dedupe + sort.
    sort -u /tmp/13-D1-audit.txt > /tmp/13-D1-audit-sorted.txt
    ```

    Produce a classification table (write it into the commit message for this task):

    | File | Line | Context | Backend (inferred) | New path |
    |---|---|---|---|---|
    | scripts/demos/01-edit-and-push.sh | 42 | `cat $MNT/0001.md` | sim | `cat $MNT/issues/0001.md` |
    | scripts/demos/06-mount-real-confluence.sh | 77 | `cat $MNT/00000360556.md` | confluence | `cat $MNT/pages/00000360556.md` |
    | ... | ... | ... | ... | ... |

    Backend-inference rules:
    - Any file containing `--backend confluence` or `/wiki/api/v2/` or `REPOSIX space` in context → Confluence → `pages/`.
    - Any file containing `--backend github` or `gh api repos/` or `Issue state_reason` → GitHub → `issues/`.
    - Any file using `reposix sim` or `127.0.0.1:7878` or `SimBackend` → sim → `issues/`.
    - If ambiguous: run the script OR read surrounding context carefully. Don't guess.

    **Expected audit size:** ~15-25 hits across 10-15 files based on the file list in `files_modified`. If audit returns < 5 hits, re-check grep patterns — likely missed something. If > 50 hits, there's probably a template loop being spuriously matched; narrow the pattern.
  </action>
  <verify>
    <automated>test -s /tmp/13-D1-audit-sorted.txt &amp;&amp; wc -l /tmp/13-D1-audit-sorted.txt</automated>
  </verify>
  <done>
    Complete audit list saved to `/tmp/13-D1-audit-sorted.txt`. Classification table produced. Every hit has a clear backend → new path mapping.
  </done>
</task>

<task type="auto">
  <name>Task 2: Apply patches file-by-file + validate smoke.sh/full.sh</name>
  <files>
    scripts/demos/01-edit-and-push.sh,
    scripts/demos/02-guardrails.sh,
    scripts/demos/03-conflict-resolution.sh,
    scripts/demos/04-token-economy.sh,
    scripts/demos/05-mount-real-github.sh,
    scripts/demos/06-mount-real-confluence.sh,
    scripts/demos/full.sh,
    scripts/demos/smoke.sh,
    scripts/demos/parity.sh,
    scripts/demos/parity-confluence.sh,
    scripts/demo.sh,
    README.md,
    docs/demo.md,
    docs/social/assets/_build_demo_gif.py,
    docs/social/assets/_build_hero_filebrowser.py,
    crates/reposix-fuse/tests/sim_death_no_hang.rs
  </files>
  <action>
    For each entry in `/tmp/13-D1-audit-sorted.txt`, apply the patch using the Edit tool (not sed — preserves readability, makes the diff reviewable).

    **General rules:**
    - If a script uses `$MNT` or `$MOUNT` as the mount dir variable, introduce a new `$BUCKET` variable near the top:
      ```bash
      BUCKET="${BUCKET:-issues}"   # override via env for Confluence demos: BUCKET=pages bash script.sh
      ```
      OR (cleaner) hard-code per-script since each script targets one backend. Pick the approach that minimizes churn.
    - For demos that are backend-specific (e.g. `06-mount-real-confluence.sh`), hard-code `pages` — no variable needed.
    - For demos that mix backends (unlikely but possible in `parity.sh`), use `$BUCKET`.
    - For README code blocks, pick the backend that the surrounding narrative is about. If the narrative is generic, use `issues` (sim is the reference backend).
    - For Python asset generators (`_build_demo_gif.py`, `_build_hero_filebrowser.py`), the mount path appears as a string literal in a recorded frame — update the string literal and regenerate the asset ONLY if the asset is committed. If the asset is git-ignored, skip regeneration.

    **Validation after patching:**

    Re-run the audit grep to confirm zero leftover hits:
    ```bash
    LEFT=$(grep -rInE 'mount[^/]*/0+[0-9]+\.md|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null | grep -vE '/(issues|pages)/' | wc -l)
    [ "$LEFT" -eq 0 ] || { echo "MISSED: "; grep -rInE 'mount[^/]*/0+[0-9]+\.md|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null | grep -vE '/(issues|pages)/'; exit 1; }
    ```

    Run smoke.sh (needs the sim + mount pipeline working end-to-end):
    ```bash
    bash scripts/demos/smoke.sh
    ```
    Assert exit 0. If it fails, inspect the exact first failing command — likely a path that's still wrong.

    Run full.sh (the larger demo suite):
    ```bash
    bash scripts/demos/full.sh
    ```
    Assert exit 0. (full.sh runs longer; budget 2-3 min.)

    If `crates/reposix-fuse/tests/sim_death_no_hang.rs` references a path at mount root, patch it — but note that Wave C already handled `tests/readdir.rs`. Only touch the sim_death test if grep showed it has an old-style path.

    **Commit message template:**
    ```
    refactor(13-D1): migrate all demo/doc paths to new nested layout

    - scripts/demos/*.sh: mount/<id>.md -> mount/{issues,pages}/<id>.md per backend
    - README.md, docs/demo.md: quickstart snippets updated
    - docs/social/assets/_build_*.py: asset-generator path strings updated
    - No behavioral code changes; only paths.

    smoke.sh: 4/4 green after migration.
    full.sh: all demos green after migration.
    ```
  </action>
  <verify>
    <automated>[ $(grep -rInE 'mount[^/]*/0+[0-9]+\.md|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null | grep -vE '/(issues|pages)/' | wc -l) -eq 0 ] &amp;&amp; bash scripts/demos/smoke.sh</automated>
  </verify>
  <done>
    Zero leftover flat-path references. smoke.sh exits 0. full.sh exits 0. The single atomic commit documents the BREAKING transition.
  </done>
</task>

<task type="auto">
  <name>Task 3: Workspace-wide green check + CI smoke</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    Run the workspace quality gate to confirm no test still asserts the old layout:
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    cargo test --workspace --release --locked -- --ignored --test-threads=1
    bash scripts/demos/smoke.sh
    ```

    If any test reveals a missed path, patch + amend Task 2's commit.
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked &amp;&amp; bash scripts/demos/smoke.sh</automated>
  </verify>
  <done>
    Full workspace + smoke suite green. The migration is complete.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

No new trust boundaries — this is a pure path-rename sweep. No network, no new surface.

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-D1-1 | Information disclosure | Stale paths in docs confuse users into thinking they can bypass the new layout | accept | The audit grep catches all current references. Future references (post-ship) are covered by CI (smoke.sh runs on every push). |
| T-13-D1-2 | Repudiation | Amending existing demo history might erase recorded asciinema proofs | accept | Demo recordings in `docs/` are committed artifacts; they'll show the OLD layout. That's historically accurate. A new `scripts/demos/07-mount-real-confluence-tree.sh` (ships in D3) records the NEW layout. Keep both. |
</threat_model>

<verification>
Nyquist coverage:
- **Audit grep:** zero hits for the old pattern after the patch.
- **smoke.sh:** 4/4 green — proves sim+github path references work end-to-end.
- **full.sh:** full suite green — proves all demos still work.
- **Workspace tests:** no test asserts an old path (C already handled `tests/readdir.rs`; this task catches anything C missed).
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -rInE 'mount[^/]*/0+[0-9]+\.md|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md crates/ 2>/dev/null | grep -vE '/(issues|pages)/' | wc -l` returns `0`.
2. `bash scripts/demos/smoke.sh` exits 0.
3. `bash scripts/demos/full.sh` exits 0.
4. `cargo test --workspace --locked` exits 0.
5. `cargo fmt --all --check` exits 0.
6. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
7. `git diff HEAD~1 --name-only | wc -l` ≥ 10 (confirms the sweep touched many files in one commit).
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D1-SUMMARY.md` documenting:
- Final file list touched (from `git diff HEAD~1 --name-only` for the sweep commit)
- Paste the classification table from the commit message
- Confirmation that smoke.sh 4/4 green (paste the last line of output)
- Any files flagged by the audit but deliberately SKIPPED (and why — e.g. historical asciinema recordings)
</output>
