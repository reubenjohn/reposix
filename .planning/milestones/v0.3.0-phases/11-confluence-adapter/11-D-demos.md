---
phase: 11-confluence-adapter
plan: D
type: execute
wave: 1
depends_on: []
files_modified:
  - scripts/demos/parity-confluence.sh
  - scripts/demos/06-mount-real-confluence.sh
autonomous: true
requirements:
  - FC-09
  - SG-08
user_setup: []

must_haves:
  truths:
    - "`bash scripts/demos/parity-confluence.sh` exits 0 with a SKIP banner when any of ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT is unset"
    - "`bash scripts/demos/06-mount-real-confluence.sh` exits 0 with a SKIP banner when any of the three env vars is unset"
    - "Both scripts set `REPOSIX_ALLOWED_ORIGINS` to include the tenant origin when they run the live path"
    - "Tier 5 demo mounts → ls → cats a page → unmounts, leaving no zombie FUSE mount behind (verified by `mountpoint -q /tmp/reposix-conf-demo-mnt` returning non-zero after the script)"
    - "Tier 3B demo runs `reposix list --backend sim` AND `reposix list --backend confluence --project $REPOSIX_CONFLUENCE_SPACE`, normalizes both to `{id, title, status}` via jq, and asserts key-set parity"
    - "Neither script is added to smoke.sh — they are Tier 3B / Tier 5, not Tier 1"
    - "`bash scripts/demos/smoke.sh` stays 4/4 green"
  artifacts:
    - path: "scripts/demos/parity-confluence.sh"
      provides: "Tier 3B parity demo: sim-vs-confluence shape diff"
      min_lines: 100
    - path: "scripts/demos/06-mount-real-confluence.sh"
      provides: "Tier 5 live mount demo for Confluence"
      min_lines: 100
  key_links:
    - from: "scripts/demos/parity-confluence.sh"
      to: "reposix list --backend confluence"
      via: "direct CLI invocation after env-var check"
      pattern: "reposix list --backend confluence"
    - from: "scripts/demos/06-mount-real-confluence.sh"
      to: "reposix mount --backend confluence"
      via: "direct CLI invocation with MOUNT_PATH + --project + background PID tracking"
      pattern: "reposix mount.*--backend confluence"
---

<objective>
Ship two demo scripts: `parity-confluence.sh` (Tier 3B — structural diff of sim vs real Confluence) and `06-mount-real-confluence.sh` (Tier 5 — mount → ls → cat → unmount against real Confluence). Both skip cleanly when env vars are unset so `scripts/demos/smoke.sh` compatibility is preserved (in case a future change adds either to Tier 1) and so a dev without Atlassian access can still `bash scripts/demos/06-mount-real-confluence.sh` and get a green exit.

Purpose: Without these demos, Phase 11 ships a library + CI contract test but no human-facing proof that `reposix mount --backend confluence` actually works end-to-end. The FUSE mount + `cat` of a real page is the demo-ability of v0.3. 11-D runs in Wave 1 (parallel to 11-A + 11-B) because the shell scripts don't require either crate to build — they invoke the release binary at runtime and exit 0 cleanly if binaries aren't present or env vars are missing.

Output: Two new executable shell scripts in `scripts/demos/`. Both structurally mirror existing patterns: `parity.sh` for 11-D.1 and `05-mount-real-github.sh` for 11-D.2.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/11-RESEARCH.md
@CLAUDE.md
@scripts/demos/parity.sh
@scripts/demos/05-mount-real-github.sh
@scripts/demos/smoke.sh

<interfaces>
From existing demos (structural templates to copy):

- `scripts/demos/_lib.sh` provides: `require`, `section`, `setup_sim`, `cleanup_trap`, `wait_for_mount`, `_REPOSIX_TMP_PATHS`, `_REPOSIX_MOUNT_PATHS`, `_REPOSIX_SIM_PIDS`. Source it via `source "${SCRIPT_DIR}/_lib.sh"`.
- All demos self-wrap in `timeout 90` via the `REPOSIX_DEMO_INNER` sentinel. Copy this block verbatim.
- All demos SKIP with `echo "SKIP: ..."; echo "== DEMO COMPLETE =="; exit 0` when prereqs are missing. The `== DEMO COMPLETE ==` marker is what smoke.sh's assert.sh looks for.
- All demos set `REPOSIX_ALLOWED_ORIGINS` appropriately before invoking reposix.

Env vars required (from CONTEXT):
- `ATLASSIAN_API_KEY`
- `ATLASSIAN_EMAIL`
- `REPOSIX_CONFLUENCE_TENANT`
- `REPOSIX_CONFLUENCE_SPACE` — the space key to mount/list (e.g. `REPOSIX`)

If any of these four are missing, skip cleanly.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: `scripts/demos/parity-confluence.sh` (Tier 3B)</name>
  <files>
    scripts/demos/parity-confluence.sh
  </files>
  <action>
    Create `scripts/demos/parity-confluence.sh` modeled on `scripts/demos/parity.sh` but with these changes:

    Header block:
    ```bash
    #!/usr/bin/env bash
    # scripts/demos/parity-confluence.sh — Tier 3B sim-vs-Confluence parity demo.
    #
    # AUDIENCE: skeptic
    # RUNTIME_SEC: 45
    # REQUIRES: cargo, jq, reposix (release binary)
    # ASSERTS: "shape parity" "DEMO COMPLETE"
    #
    # Narrative: same story as parity.sh, but for Confluence Cloud. Lists
    # pages from a real Atlassian space AND issues from the reposix sim,
    # normalizes both to {id, title, status}, and diffs the key sets.
    # Identical keys = structural parity; only content differs.
    #
    # Skip behavior: exits 0 with SKIP banner if any of the four required
    # env vars are unset. This is the documented v0.3 SKIP contract.
    ```

    Body structure (adapt parity.sh):
    1. `timeout 90` self-wrap via `REPOSIX_DEMO_INNER` sentinel — verbatim from parity.sh.
    2. Source `_lib.sh`.
    3. `require` the binaries: `reposix-sim`, `reposix`, `jq`.
    4. **SKIP check** — immediately after `require`:
       ```bash
       MISSING=()
       [[ -z "${ATLASSIAN_API_KEY:-}"          ]] && MISSING+=("ATLASSIAN_API_KEY")
       [[ -z "${ATLASSIAN_EMAIL:-}"            ]] && MISSING+=("ATLASSIAN_EMAIL")
       [[ -z "${REPOSIX_CONFLUENCE_TENANT:-}"  ]] && MISSING+=("REPOSIX_CONFLUENCE_TENANT")
       [[ -z "${REPOSIX_CONFLUENCE_SPACE:-}"   ]] && MISSING+=("REPOSIX_CONFLUENCE_SPACE")
       if (( ${#MISSING[@]} > 0 )); then
           echo "SKIP: env vars unset: ${MISSING[*]}"
           echo "      set them to run the Confluence half of parity."
           echo "== DEMO COMPLETE =="
           exit 0
       fi
       ```
    5. Set `REPOSIX_ALLOWED_ORIGINS`:
       ```bash
       export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
       ```
    6. Set local paths:
       ```bash
       SIM_BIND="127.0.0.1:7805"   # different port from parity.sh's 7804 to avoid collision
       SIM_URL="http://${SIM_BIND}"
       SIM_DB="/tmp/reposix-demo-parity-conf-sim.db"
       SIM_JSON="/tmp/parity-conf-sim.json"
       CONF_JSON="/tmp/parity-conf-confluence.json"
       DIFF_OUT="/tmp/parity-conf-diff.txt"
       _REPOSIX_TMP_PATHS+=("$SIM_JSON" "$CONF_JSON" "$DIFF_OUT")
       cleanup_trap
       rm -f "$SIM_JSON" "$CONF_JSON" "$DIFF_OUT" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
       pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
       sleep 0.2
       ```
    7. **[1/4] boot sim**: same `setup_sim "$SIM_DB"` pattern as parity.sh.
    8. **[2/4] list sim**:
       ```bash
       reposix list --origin "${SIM_URL}" --project demo --format json \
           | jq '[.[] | {id, title, status}] | sort_by(.id)' > "$SIM_JSON"
       ```
    9. **[3/4] list Confluence**:
       ```bash
       section "[3/4] list pages via real Confluence (space=${REPOSIX_CONFLUENCE_SPACE})"
       reposix list \
           --backend confluence \
           --project "$REPOSIX_CONFLUENCE_SPACE" \
           --format json \
           | jq '[.[] | {id, title, status}] | sort_by(.id)' > "$CONF_JSON"
       echo "wrote $(jq length < "$CONF_JSON") confluence pages -> $CONF_JSON"
       head -20 "$CONF_JSON"
       ```
       (No `gh api` fallback — we have a real adapter now, unlike when parity.sh was written.)
    10. **[4/4] diff**: same structure as parity.sh's §4 — `diff -u`, show head, assert key-set parity via jq.
    11. Final `echo "== DEMO COMPLETE =="`.

    Make it executable:
    ```bash
    chmod +x scripts/demos/parity-confluence.sh
    ```

    Manual smoke test (no env vars):
    ```bash
    bash scripts/demos/parity-confluence.sh
    # Expected: SKIP banner + "== DEMO COMPLETE ==" + exit 0
    ```

    Manual smoke test (with env vars, if available): defer to Phase 11-F verification run. The script's correctness on the happy path is verified by the reviewer + user in the morning; tonight we only need SKIP-path correctness.
  </action>
  <verify>
    <automated>env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE bash scripts/demos/parity-confluence.sh 2>&amp;1 | tee /tmp/parity-conf-out.txt | grep -q 'SKIP:' &amp;&amp; tail -1 /tmp/parity-conf-out.txt | grep -q '== DEMO COMPLETE ==' &amp;&amp; test -x scripts/demos/parity-confluence.sh</automated>
  </verify>
  <done>
    Script exists, is executable, SKIPs cleanly when env unset, emits the `== DEMO COMPLETE ==` marker on the SKIP path. Commit: `feat(11-D-1): parity-confluence.sh Tier 3B demo with skip path`.
  </done>
</task>

<task type="auto">
  <name>Task 2: `scripts/demos/06-mount-real-confluence.sh` (Tier 5)</name>
  <files>
    scripts/demos/06-mount-real-confluence.sh
  </files>
  <action>
    Create `scripts/demos/06-mount-real-confluence.sh` modeled on `scripts/demos/05-mount-real-github.sh` but adapted for Confluence:

    Header block:
    ```bash
    #!/usr/bin/env bash
    # scripts/demos/06-mount-real-confluence.sh — Tier 5 FUSE-mount-real-Confluence demo.
    #
    # AUDIENCE: developer
    # RUNTIME_SEC: 45
    # REQUIRES: cargo, fusermount3, reposix (release binary)
    # ASSERTS: "DEMO COMPLETE" ".md"
    #
    # Narrative: `reposix mount --backend confluence --project <SPACE_KEY>`
    # mounts a real Atlassian Confluence space as a tree of Markdown files.
    # This demo mounts, lists, cats the first page, and unmounts.
    #
    # Not in smoke — requires real Atlassian credentials.
    # Skips cleanly (exit 0) if the four required env vars are unset.
    ```

    Body structure (adapt 05-mount-real-github.sh):
    1. `timeout 90` self-wrap.
    2. Source `_lib.sh`.
    3. `require reposix`, `require fusermount3`.
    4. **SKIP check** — identical four-var check as parity-confluence.sh's:
       ```bash
       MISSING=()
       [[ -z "${ATLASSIAN_API_KEY:-}"         ]] && MISSING+=("ATLASSIAN_API_KEY")
       [[ -z "${ATLASSIAN_EMAIL:-}"           ]] && MISSING+=("ATLASSIAN_EMAIL")
       [[ -z "${REPOSIX_CONFLUENCE_TENANT:-}" ]] && MISSING+=("REPOSIX_CONFLUENCE_TENANT")
       [[ -z "${REPOSIX_CONFLUENCE_SPACE:-}"  ]] && MISSING+=("REPOSIX_CONFLUENCE_SPACE")
       if (( ${#MISSING[@]} > 0 )); then
           echo "SKIP: env vars unset: ${MISSING[*]}"
           echo "      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to run this demo."
           echo "== DEMO COMPLETE =="
           exit 0
       fi
       ```
    5. Config:
       ```bash
       MOUNT_PATH="/tmp/reposix-conf-demo-mnt"
       export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
       fusermount3 -u "$MOUNT_PATH" 2>/dev/null || true
       rm -rf "$MOUNT_PATH"
       mkdir -p "$MOUNT_PATH"
       _REPOSIX_MOUNT_PATHS+=("$MOUNT_PATH")
       _REPOSIX_TMP_PATHS+=("$MOUNT_PATH")
       cleanup_trap
       ```
    6. **[1/4] mount**:
       ```bash
       section "[1/4] mount real Confluence at ${MOUNT_PATH}"
       echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
       echo "space:  ${REPOSIX_CONFLUENCE_SPACE}"
       echo "allowlist: ${REPOSIX_ALLOWED_ORIGINS}"
       reposix mount "$MOUNT_PATH" \
           --backend confluence \
           --project "$REPOSIX_CONFLUENCE_SPACE" \
           >/tmp/reposix-conf-demo-mnt.log 2>&1 &
       MOUNT_PID=$!
       _REPOSIX_SIM_PIDS+=("$MOUNT_PID")
       if ! wait_for_mount "$MOUNT_PATH" 30; then
           echo "----- mount log -----"
           cat /tmp/reposix-conf-demo-mnt.log || true
           exit 1
       fi
       ```
    7. **[2/4] ls**:
       ```bash
       section "[2/4] ls the mount — every page is a Markdown file"
       LISTING="$(ls "$MOUNT_PATH")"
       COUNT=$(echo "$LISTING" | wc -l)
       echo "page count: $COUNT"
       echo "$LISTING" | head -5
       if [[ "$COUNT" -lt 1 ]]; then
           echo "FAIL: mount exposed 0 files"
           exit 1
       fi
       ```
    8. **[3/4] cat**: cat the first .md file in the listing (not a hardcoded `0001.md` — Confluence IDs are page IDs, not issue numbers, and space-dependent):
       ```bash
       section "[3/4] cat the first page — frontmatter renders from real Confluence"
       FIRST_PAGE="$(echo "$LISTING" | head -1)"
       echo "+ cat ${MOUNT_PATH}/${FIRST_PAGE}"
       if ! cat "${MOUNT_PATH}/${FIRST_PAGE}"; then
           echo "FAIL: cat ${FIRST_PAGE} did not succeed"
           exit 1
       fi
       ```
    9. **[4/4] unmount**:
       ```bash
       section "[4/4] unmount"
       fusermount3 -u "$MOUNT_PATH" || true
       for _ in $(seq 1 30); do
           mountpoint -q "$MOUNT_PATH" 2>/dev/null || break
           sleep 0.1
       done
       if mountpoint -q "$MOUNT_PATH" 2>/dev/null; then
           echo "FAIL: mount still active after 3s"
           exit 1
       fi
       echo "unmounted cleanly"
       echo
       echo "== DEMO COMPLETE =="
       ```

    Make it executable: `chmod +x scripts/demos/06-mount-real-confluence.sh`.

    Manual smoke: `env -u ATLASSIAN_API_KEY … bash scripts/demos/06-mount-real-confluence.sh` → expect SKIP path + exit 0.
  </action>
  <verify>
    <automated>env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE bash scripts/demos/06-mount-real-confluence.sh 2>&amp;1 | tee /tmp/06-mnt-conf-out.txt | grep -q 'SKIP:' &amp;&amp; tail -1 /tmp/06-mnt-conf-out.txt | grep -q '== DEMO COMPLETE ==' &amp;&amp; test -x scripts/demos/06-mount-real-confluence.sh</automated>
  </verify>
  <done>
    Script exists, is executable, SKIPs cleanly when env unset. Commit: `feat(11-D-2): 06-mount-real-confluence.sh Tier 5 demo with skip path`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Regression check — smoke.sh still 4/4 green</name>
  <files>
    (validation only — confirms smoke.sh untouched)
  </files>
  <action>
    Verify that adding the two new scripts did NOT add them to smoke.sh and that smoke.sh still passes:
    ```bash
    # 1. Confirm smoke.sh does NOT reference the new scripts (they are Tier 3B + Tier 5, not Tier 1)
    ! grep -qE 'parity-confluence|06-mount-real-confluence' scripts/demos/smoke.sh
    # 2. Confirm smoke.sh still runs 4/4 green
    # (Requires release binaries built; PATH must include target/release.)
    cargo build --release --workspace --bins --locked
    PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh
    ```
    Expected output: `smoke suite: 4 passed, 0 failed (of 4)`.
  </action>
  <verify>
    <automated>! grep -qE 'parity-confluence|06-mount-real-confluence' scripts/demos/smoke.sh &amp;&amp; cargo build --release --workspace --bins --locked &amp;&amp; PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh</automated>
  </verify>
  <done>
    smoke.sh stays 4/4 green. New scripts are not accidentally included in Tier 1. No commit unless a fix was needed.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| env vars → subprocess argv | Demo reads env vars and passes them as process-level env (not CLI args) to `reposix`, which is the safe path. No secrets are ever written to stdout, files on disk, or shell history. |
| tenant HTML → terminal (`cat`) | The demo `cat`s a real Confluence page body; content is attacker-influenced XHTML. Terminal rendering is out-of-surface unless the user has a malicious terminal emulator (in which case they have bigger problems). |
| `$REPOSIX_CONFLUENCE_SPACE` → `reposix` CLI `--project` | String is passed verbatim; reposix-cli does not shell-expand it. Still, the demo uses `"$REPOSIX_CONFLUENCE_SPACE"` (double-quoted) so even adversarial characters like `;` or `$()` are inert at the bash level. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11D-01 | Information disclosure | Demo output containing the API token | mitigate | Demo NEVER echoes env vars; references `${REPOSIX_CONFLUENCE_TENANT}` + `${REPOSIX_CONFLUENCE_SPACE}` (non-secret identifiers) only. The ALLOWED_ORIGINS line echoes the tenant-hostname (non-secret). Token + email never appear in any `echo`. |
| T-11D-02 | Tampering | Shell injection via `$REPOSIX_CONFLUENCE_SPACE` | mitigate | Variable is always double-quoted: `"$REPOSIX_CONFLUENCE_SPACE"`. Even if the user set it to `"; rm -rf /"`, bash would pass that as a single argv element, and `reposix list --project` treats it as a literal space key. |
| T-11D-03 | DoS | Demo hanging on a slow/dead tenant | mitigate | `timeout 90` outer wrap + `wait_for_mount 30` inner + fuse daemon's 5-second SG-07 ceiling. Max runtime: 90 seconds. |
| T-11D-04 | Repudiation | Stale FUSE mount left behind on script failure | mitigate | `cleanup_trap` installs EXIT/ERR/TERM handlers that call `fusermount3 -u` for every entry in `_REPOSIX_MOUNT_PATHS`. The Task 2 script pushes `$MOUNT_PATH` there. Final `mountpoint -q` check at the end confirms clean unmount (fails the demo if not). |

Block-on-high: T-11D-01 mitigation is trivially verifiable: `grep -E 'echo.*ATLASSIAN_API_KEY|echo.*ATLASSIAN_EMAIL' scripts/demos/*-confluence*.sh` should return no matches.
</threat_model>

<verification>
Nyquist coverage:
- **Skip path (both scripts):** run with env unset, assert SKIP banner + `== DEMO COMPLETE ==` + exit 0. Covered by Task 1 + Task 2 verify commands.
- **Happy path (both scripts):** deferred to Phase 11-F morning verification, since we have no live creds tonight. This is the known gap documented in 00-CREDENTIAL-STATUS.md.
- **Regression:** smoke.sh stays 4/4 green. Covered by Task 3 verify.
- **Permissions:** `test -x` for both scripts — Task 1 and Task 2 verify commands.
- **No-token-leak:** `grep -E 'echo.*ATLASSIAN' scripts/demos/*-confluence*.sh | grep -v TENANT` returns empty — can be added as a separate assertion:
  ```bash
  ! grep -E 'echo[^|]*\$(ATLASSIAN_API_KEY|ATLASSIAN_EMAIL)' scripts/demos/parity-confluence.sh scripts/demos/06-mount-real-confluence.sh
  ```
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -x scripts/demos/parity-confluence.sh` returns 0.
2. `test -x scripts/demos/06-mount-real-confluence.sh` returns 0.
3. `grep -q 'ASSERTS: "shape parity" "DEMO COMPLETE"' scripts/demos/parity-confluence.sh` returns 0.
4. `grep -q 'ASSERTS: "DEMO COMPLETE"' scripts/demos/06-mount-real-confluence.sh` returns 0.
5. `env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE bash scripts/demos/parity-confluence.sh >/dev/null 2>&1` exits 0.
6. `env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE bash scripts/demos/06-mount-real-confluence.sh >/dev/null 2>&1` exits 0.
7. `env -u ATLASSIAN_API_KEY bash scripts/demos/parity-confluence.sh 2>&1 | grep -qE 'SKIP:.*ATLASSIAN_API_KEY'` returns 0.
8. `env -u ATLASSIAN_API_KEY bash scripts/demos/06-mount-real-confluence.sh 2>&1 | grep -qE 'SKIP:.*ATLASSIAN_API_KEY'` returns 0.
9. `! grep -qE 'parity-confluence|06-mount-real-confluence' scripts/demos/smoke.sh` (new scripts NOT added to Tier 1).
10. `! grep -qE 'echo[^|]*\$\{?ATLASSIAN_API_KEY\}?' scripts/demos/parity-confluence.sh scripts/demos/06-mount-real-confluence.sh` — no token leakage.
11. `PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh` (after release build) reports `smoke suite: 4 passed, 0 failed`.
</success_criteria>

<rollback_plan>
If either demo's SKIP path fails because `_lib.sh` has diverged from the GitHub template:
1. Inspect `scripts/demos/_lib.sh` for `cleanup_trap`, `_REPOSIX_TMP_PATHS`, `wait_for_mount` — all exist as of the last commit.
2. If `_lib.sh` has been renamed or functions removed, inline the essentials (cleanup trap with `fusermount3 -u`) rather than reworking `_lib.sh` in this plan.

If the `timeout 90` wrapper misbehaves (e.g. doesn't propagate SIGTERM on the happy path):
1. Copy the working pattern from `05-mount-real-github.sh` character-for-character.
2. Do NOT reduce the timeout below 60s — Atlassian's first round-trip on a cold connection comfortably consumes 10-15s.

If the Tier 5 demo's cleanup leaves a zombie mount (rare — mitigated by `cleanup_trap`):
1. Run `fusermount3 -u /tmp/reposix-conf-demo-mnt` manually.
2. Fix `cleanup_trap` in `_lib.sh` only if the bug is reproducible across all Tier 5 demos.
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-D-SUMMARY.md` with:
- Confirmation both scripts exist and are executable.
- Paste of the SKIP-path run for each.
- Confirmation smoke.sh stayed 4/4 green.
- Any choice made about port numbers or temp paths that 11-F's MORNING-BRIEF should surface to the user.
</output>
