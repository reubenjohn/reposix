---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: E
type: execute
wave: 5
depends_on: [D1, D2, D3]
files_modified:
  - .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md
  - Cargo.toml
  - CHANGELOG.md
autonomous: false
requirements:
  - OP-1
user_setup:
  - service: atlassian-confluence
    why: "Phase gate requires a live-Confluence verification run against the REPOSIX space on reuben-john.atlassian.net"
    env_vars:
      - name: ATLASSIAN_API_KEY
        source: "Atlassian account → Security → API tokens"
      - name: ATLASSIAN_EMAIL
        source: "Atlassian account email (the one tied to the API token)"
      - name: ATLASSIAN_TENANT
        source: "Subdomain of atlassian.net (e.g. reuben-john)"
      - name: REPOSIX_ALLOWED_ORIGINS
        source: "Must include https://<ATLASSIAN_TENANT>.atlassian.net"

must_haves:
  truths:
    - "Workspace `Cargo.toml` version bumped to `0.4.0` (if not already bumped)"
    - "`cargo fmt --all --check` exits 0"
    - "`cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0"
    - "`cargo test --workspace --locked` exits 0"
    - "`cargo test --workspace --release --locked -- --ignored --test-threads=1` exits 0 (includes FUSE integration + live Confluence contract tests)"
    - "`bash scripts/demos/smoke.sh` exits 0 (sim 4/4 green under new layout)"
    - "`bash scripts/demos/full.sh` exits 0"
    - "`bash scripts/demos/07-mount-real-confluence-tree.sh` exits 0 against live Atlassian (REPOSIX space on reuben-john.atlassian.net) with credentials set"
    - "`mkdocs build --strict` exits 0"
    - "Manual live-verify produces expected `ls` / `cat` / `readlink` output captured verbatim into 13-SUMMARY.md"
    - "`mount | grep -c reposix` returns 0 after E runs (no leaked mounts)"
    - "CHANGELOG `[Unreleased]` promoted to `[v0.4.0]` if D2 left it as [Unreleased]; date stamped 2026-04-14"
    - "13-SUMMARY.md lists all 9 plans' outcomes, the live-verify transcript, and confirms every CONTEXT.md locked decision is landed"
  artifacts:
    - path: ".planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md"
      provides: "Phase-level summary with live-verify transcript + coverage table + threat-model closeout"
      min_lines: 100
  key_links:
    - from: "13-SUMMARY.md"
      to: "every 13-*-SUMMARY.md from plans A..D3"
      via: "cross-references in the 'Plans landed' table"
      pattern: "13-[A-E][0-9]*-SUMMARY"
---

<objective>
Wave-E gauntlet. The phase-level verification gate. Run every quality check against the integrated v0.4.0 code: fmt, clippy, workspace test, `--ignored` (including the Confluence live-contract test), smoke.sh, full.sh, the new tree-demo against live Atlassian, mkdocs strict. Capture live-verify transcript (`ls` / `cat` / `readlink`) byte-for-byte into 13-SUMMARY.md. Promote CHANGELOG if still [Unreleased]. Bump Cargo.toml version. Includes one `checkpoint:human-verify` gate for the live Confluence demo, because the live run touches a real tenant and the user must confirm the output matches the hero walkthrough.

Purpose: This is the last gate before `scripts/tag-v0.4.0.sh` is safe to run. Every prior wave shipped green in isolation; E proves they stack green as an integrated whole against the real world.

Output: Workspace version bump in Cargo.toml, CHANGELOG promotion if needed, phase-level SUMMARY document with the live-verify transcript.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-WAVES.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-core-foundations.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B1-confluence-parent-id.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B2-fuse-tree-module.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B3-frontmatter-parent-id.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-C-fuse-wiring.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D1-breaking-migration-sweep.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D2-docs-and-adr.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D3-release-scripts-and-demo.md
@Cargo.toml
@CHANGELOG.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Version bump + CHANGELOG promotion</name>
  <files>
    Cargo.toml,
    CHANGELOG.md
  </files>
  <action>
    Cargo.toml — if the workspace `[workspace.package] version = "0.3.0"` (or similar), bump to `"0.4.0"`. Run `cargo check --workspace --locked` to confirm Cargo.lock updates cleanly. If the lockfile diff is ONLY the version field of the reposix-* crates, that is expected — commit it.

    CHANGELOG.md — if D2 wrote the v0.4.0 block under `## [Unreleased]`, promote: change `## [Unreleased]` to `## [v0.4.0] — 2026-04-14` (D2 may have already done this; confirm). Ensure today's date (2026-04-14) is in the heading per CONTEXT.md.

    Commit: `chore(13-E-1): bump workspace to 0.4.0 + CHANGELOG promotion`.
  </action>
  <verify>
    <automated>grep -qE '^version = "0\.4\.0"' Cargo.toml &amp;&amp; grep -qE '## \[v0\.4\.0\] — 2026-04-14' CHANGELOG.md &amp;&amp; cargo check --workspace --locked</automated>
  </verify>
  <done>
    Workspace on 0.4.0. CHANGELOG promoted. Cargo.lock consistent.
  </done>
</task>

<task type="auto">
  <name>Task 2: Workspace gauntlet (fmt + clippy + test + --ignored + mkdocs + smoke)</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    Run each check in sequence, capture the log. If any fails: STOP, diagnose, do NOT proceed to live-verify. Fix the root cause in the appropriate plan's code (may require re-entering A-D waves) — never paper over a red check.

    ```bash
    set -e
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    cargo test --workspace --release --locked -- --ignored --test-threads=1
    mkdocs build --strict
    bash scripts/demos/smoke.sh
    bash scripts/demos/full.sh
    mount | grep -c reposix   # expect 0
    ```

    If `mount | grep reposix` returns non-zero, clean up and fail the task:
    ```bash
    for m in $(mount | awk '/reposix/ { print $3 }'); do
      fusermount3 -u "$m" 2>/dev/null || fusermount -u "$m" 2>/dev/null || true
    done
    ```

    Record the workspace test count:
    ```bash
    cargo test --workspace --locked 2>&1 | grep -oE 'test result: ok\. [0-9]+ passed' | awk '{ sum += $4 } END { print sum }'
    ```
    This goes into 13-SUMMARY.md as the "before/after test count".
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked &amp;&amp; cargo test --workspace --release --locked -- --ignored --test-threads=1 &amp;&amp; mkdocs build --strict &amp;&amp; bash scripts/demos/smoke.sh &amp;&amp; bash scripts/demos/full.sh &amp;&amp; [ $(mount | grep -c reposix) -eq 0 ]</automated>
  </verify>
  <done>
    Every check in the chain exits 0. Zero leaked FUSE mounts. Test count captured for SUMMARY.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Live Confluence verify against REPOSIX space</name>
  <what-built>
    Full Phase-13 stack: nested mount layout, tree/ overlay, readlink resolution, .gitignore emission, Confluence parent_id populated. Waves A-D shipped; D3's tier-5 demo script is ready to run against the live Atlassian tenant.
  </what-built>
  <how-to-verify>
    Prerequisites — ensure these env vars are exported in the current shell:
    ```bash
    export ATLASSIAN_EMAIL="your-email@example.com"
    export ATLASSIAN_API_KEY="atlassian_xxx"
    export ATLASSIAN_TENANT="reuben-john"
    export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://reuben-john.atlassian.net"
    ```

    Step 1 — Run the tier-5 demo script and capture output:
    ```bash
    bash scripts/demos/07-mount-real-confluence-tree.sh 2>&1 | tee /tmp/13-E-live-verify.log
    ```

    Step 2 — Visually inspect the output for these exact shapes:
    1. `ls $MNT` contains `pages`, `tree`, `.gitignore` (plus `.` / `..` if listed).
    2. `cat $MNT/.gitignore` prints exactly `/tree/` (plus trailing newline).
    3. `ls $MNT/pages` contains `00000360556.md`, `00000131192.md`, `00000065916.md`, `00000425985.md` (IDs from CONTEXT.md §specifics — homepage + 3 children).
    4. `cat $MNT/pages/00000360556.md | head -5` starts with `---` (YAML frontmatter fence); second line is `id:` or `title:` (frontmatter's first field).
    5. `ls $MNT/tree` contains exactly one directory named `reposix-demo-space-home` (slug of page 360556's title).
    6. Inside `$MNT/tree/reposix-demo-space-home/`: four entries — `_self.md`, `architecture-notes.md`, `demo-plan.md`, `welcome-to-reposix.md` (order may vary).
    7. `readlink $MNT/tree/reposix-demo-space-home/welcome-to-reposix.md` prints exactly `../../pages/00000131192.md`.
    8. `cat $MNT/tree/reposix-demo-space-home/welcome-to-reposix.md | head -5` prints the same content as `cat $MNT/pages/00000131192.md | head -5` (symlink resolved).
    9. The demo's success banner prints at the end.
    10. After the script completes, `mount | grep reposix` returns empty (mount was cleaned up).

    Step 3 — Paste the entire `/tmp/13-E-live-verify.log` into 13-SUMMARY.md under "Live-verify transcript" section (written in Task 4).

    Step 4 — Respond with one of:
    - `approved` — every check in step 2 matches. Proceed to Task 4.
    - `issue: <description>` — something was off. Describe what differed. Planning will re-enter (likely revising C or B1 and re-running C integration tests).
  </how-to-verify>
  <resume-signal>Type "approved" or describe issues.</resume-signal>
</task>

<task type="auto">
  <name>Task 4: Write 13-SUMMARY.md with live-verify transcript + threat closeout</name>
  <files>
    .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md
  </files>
  <action>
    Compose the phase-level SUMMARY. Required sections:

    1. **Header** — phase name, date shipped, version tag target (v0.4.0).
    2. **Plans landed** — table of A, B1, B2, B3, C, D1, D2, D3, E with commit SHAs, a one-line outcome, and a link to the per-plan SUMMARY.
    3. **Decision coverage matrix** — copy from 13-WAVES.md, annotate each row with `SHIPPED` + commit SHA.
    4. **Test counts** — workspace test count before/after (from Task 2's captured numbers). FUSE integration test count. Confluence wiremock test count.
    5. **Live-verify transcript** — paste full output from `/tmp/13-E-live-verify.log`. Truncate per line if noisy (keep first 500 lines).
    6. **Threat-model closeout** — table of T-13-01 through T-13-06, each with either "MITIGATED by <plan> / <test_name>" or "ACCEPTED (documented in ADR-003 §X)".
    7. **Known issues / follow-ups** — Unicode NFC gap (T-13-06), any discretion calls made, anything C-SUMMARY flagged for later.
    8. **Next steps** — user runs `bash scripts/tag-v0.4.0.sh` after reviewing this SUMMARY. CI release.yml auto-uploads prebuilt binaries on tag push.

    Minimum ~100 lines. Reference every 13-*-SUMMARY.md file. Do NOT fabricate outcomes — only record what actually happened. If any check in Task 2 or Task 3 revealed a surprise, document it honestly.

    Commit: `docs(13-E-4): phase-13 SUMMARY with live-verify transcript + threat closeout`.
  </action>
  <verify>
    <automated>test -f .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md &amp;&amp; wc -l .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md | awk '{ exit ($1 &gt;= 100 ? 0 : 1) }' &amp;&amp; grep -q "Live-verify transcript" .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md &amp;&amp; grep -q "T-13-01" .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md &amp;&amp; grep -q "T-13-05" .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md</automated>
  </verify>
  <done>
    13-SUMMARY.md committed with all 8 sections, ≥100 lines, cross-references to every per-plan SUMMARY, live-verify transcript pasted, threat-model table showing every T-13-0X closed. Commit ready; user can now run `scripts/tag-v0.4.0.sh` whenever they want.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Test runner → live Atlassian | Task 3 hits the real REPOSIX space with the user's API token. SG-01 allowlist enforcement is already baked into the adapter; no new risk. |
| Release script → remote git + GitHub | `scripts/tag-v0.4.0.sh` is authored by D3 but only run manually by the user after this SUMMARY is approved. Same posture as v0.3.0 tagging. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-E1 | Information disclosure | Live-verify transcript leaks API token | mitigate | `bash` does NOT echo env vars when capturing stdout of a subprocess. The demo script never prints `$ATLASSIAN_API_KEY`. Before pasting transcript into SUMMARY.md, the human verifier checks the log for any accidental credential leakage. |
| T-13-E2 | Repudiation | SUMMARY.md hides a failed check | mitigate | Task 2 + Task 3 must both be green before Task 4 composes the SUMMARY. If either is red, Task 4 does NOT run. No SUMMARY can be produced without verified green state. |

**Full phase-level threat-model closeout is in 13-SUMMARY.md itself.** This plan's role is to ensure the closeout exists and is accurate, not to re-derive it.
</threat_model>

<verification>
Nyquist coverage:
- **Workspace gauntlet:** fmt + clippy + test + --ignored + mkdocs + smoke + full — all exit 0.
- **Live-verify:** demo-07 against real Atlassian — human gate confirms the 10-point checklist.
- **Mount hygiene:** `mount | grep reposix` empty after every stage.
- **SUMMARY completeness:** ≥100 lines, all 8 required sections, cross-references.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -qE '^version = "0\.4\.0"' Cargo.toml` exits 0.
2. `grep -qE '## \[v0\.4\.0\] — 2026-04-14' CHANGELOG.md` exits 0.
3. `cargo fmt --all --check` exits 0.
4. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
5. `cargo test --workspace --locked` exits 0.
6. `cargo test --workspace --release --locked -- --ignored --test-threads=1` exits 0.
7. `mkdocs build --strict` exits 0.
8. `bash scripts/demos/smoke.sh` exits 0.
9. `bash scripts/demos/full.sh` exits 0.
10. `[ $(mount | grep -c reposix) -eq 0 ]` exits 0.
11. `test -f .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md` exits 0.
12. `wc -l .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md | awk '{ exit ($1 >= 100 ? 0 : 1) }'` exits 0.
13. `grep -q "Live-verify transcript" .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md` exits 0.
14. `for t in 01 02 03 04 05 06; do grep -q "T-13-$t" .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-SUMMARY.md || exit 1; done` exits 0.
</success_criteria>

<output>
This plan's output IS 13-SUMMARY.md. After Task 4 commits, Phase 13 is fully shipped. User may run `bash scripts/tag-v0.4.0.sh` whenever they are ready.
</output>
