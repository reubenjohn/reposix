---
phase: 04-demo-recording-readme
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - scripts/demo.sh
  - crates/reposix-sim/fixtures/seed.json
  - docs/demo.md
  - docs/demo.typescript
  - docs/demo.transcript.txt
autonomous: true
requirements:
  - FC-09
  - SG-08
  - FC-01
  - FC-02
  - FC-03
  - FC-05
  - FC-06

must_haves:
  truths:
    - "A single command (`bash scripts/demo.sh`) drives the full 9-step narrative from a clean slate to cleanup and exits 0."
    - "The run is idempotent — re-running immediately after a prior run succeeds without manual cleanup."
    - "The recording (`docs/demo.typescript`) visibly shows SG-02 bulk-delete cap refusing a 6-file delete, then accepting it after `[allow-bulk-delete]` is added to the commit message."
    - "The recording visibly shows an outbound HTTP allowlist refusal (`REPOSIX_ALLOWED_ORIGINS` mismatch producing EIO / refusal stderr from the FUSE daemon)."
    - "The recording visibly shows a server-controlled-field strip: a client write containing `version: 999` does NOT update the server's authoritative `version` field."
    - "`docs/demo.md` has a `## Walkthrough` section whose commands can be pasted literally into a shell on a fresh clone."
    - "`docs/demo.transcript.txt` is a non-binary text excerpt of the typescript suitable for GitHub rendering (head of `docs/demo.typescript`)."
  artifacts:
    - path: "scripts/demo.sh"
      provides: "Idempotent 9-step demo driver, `set -euo pipefail`, trap-based cleanup."
      min_lines: 150
    - path: "docs/demo.md"
      provides: "Human-readable walkthrough + reproduce-in-5-min instructions + guardrails callout."
      contains: "## Walkthrough"
    - path: "docs/demo.typescript"
      provides: "Raw `script(1)` recording of a full demo run."
    - path: "docs/demo.transcript.txt"
      provides: "Plain-text head excerpt of the typescript for GitHub readability."
    - path: "crates/reposix-sim/fixtures/seed.json"
      provides: "≥6 issues so bulk-delete cap demo can actually delete 6 (current seed has 3)."
      contains: "\"id\": 6"
  key_links:
    - from: "scripts/demo.sh"
      to: "reposix sim / reposix mount / git-remote-reposix"
      via: "cargo run --release --quiet -p <binary>"
      pattern: "cargo run[^\\n]*--release"
    - from: "scripts/demo.sh"
      to: "REPOSIX_ALLOWED_ORIGINS enforcement"
      via: "env override in guardrail step"
      pattern: "REPOSIX_ALLOWED_ORIGINS"
    - from: "docs/demo.md"
      to: ".planning/research/threat-model-and-critique.md"
      via: "inline link in limitations section"
      pattern: "threat-model"
---

<objective>
Produce a single, idempotent shell script (`scripts/demo.sh`) that drives the full 9-step reposix demo end-to-end, extend the simulator seed to ≥6 issues so the SG-02 bulk-delete cap demo is exercisable, record a `script(1)` typescript of the run, and write a `docs/demo.md` walkthrough a third party can follow verbatim.

Purpose: Satisfies ROADMAP Phase-4 Success Criteria #1, #2, and #3 and the SG-08 "guardrails fired on camera" requirement. The recording is the demo-day deliverable.

Output: `scripts/demo.sh`, extended `crates/reposix-sim/fixtures/seed.json`, `docs/demo.md`, `docs/demo.typescript`, `docs/demo.transcript.txt`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/04-demo-recording-readme/04-CONTEXT.md
@.planning/phases/S-stretch-write-path-and-remote-helper/S-DONE.md
@CLAUDE.md
@crates/reposix-cli/src/demo.rs
@crates/reposix-sim/fixtures/seed.json

<interfaces>
<!-- Binary surfaces the script drives. Extracted from Phase S-DONE + demo.rs. -->

- `cargo run --release --quiet -p reposix-cli -- sim --bind 127.0.0.1:7878 --db /tmp/demo-sim.db --seed-file crates/reposix-sim/fixtures/seed.json`
  (starts simulator on :7878; `--healthz` available at `/healthz`.)
- `cargo run --release --quiet -p reposix-cli -- mount /tmp/demo-mnt --backend http://127.0.0.1:7878 --project demo`
  (mounts FUSE; honors `REPOSIX_ALLOWED_ORIGINS` env.)
- `git-remote-reposix` (already on `$PATH` after `cargo build --release`): used via
  `git remote add origin reposix::http://127.0.0.1:7878/projects/demo`.
- SG-02 refusal message (from Phase S verified run):
  `error: refusing to push (would delete 6 issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)`
- Audit DB: `sqlite3 /tmp/demo-sim.db 'SELECT method, path, status FROM audit_events ORDER BY id DESC LIMIT 5;'`
- Allowlist env: `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:9999` — mount starts but every backend fetch fails (backend is on :7878, allowlist is :9999).
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Extend seed fixture to ≥6 issues (commit 1)</name>
  <files>crates/reposix-sim/fixtures/seed.json</files>
  <action>
Extend `crates/reposix-sim/fixtures/seed.json` from 3 issues to 6. Keep the existing 3 issues verbatim (they exercise specific fixtures: adversarial `<script>` body on id 1, NO_COLOR docs on id 2, `version: 999` body on id 3). Add issues 4, 5, 6 with realistic titles (e.g. "flaky integration test on CI", "document audit-log schema", "investigate p99 latency spike"), varied statuses (`open`, `in_progress`, `open`), non-adversarial bodies (plain text, ~3–5 lines each — this is the happy-path bulk of the demo). Preserve JSON formatting (2-space indent) to match existing file.

Then run `cargo test -p reposix-sim --quiet` and verify all sim tests still pass (seed-loader tests parse the new issues). If any test asserts issue count, update the assertion to 6.

Commit: `feat(04-01): extend demo seed to 6 issues for SG-02 bulk-delete demo`
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; jq '.issues | length' crates/reposix-sim/fixtures/seed.json | grep -q '^6$' &amp;&amp; cargo test -p reposix-sim --quiet</automated>
  </verify>
  <done>seed.json contains exactly 6 issues (ids 1..6); `cargo test -p reposix-sim` green; commit landed.</done>
</task>

<task type="auto">
  <name>Task 2: Write scripts/demo.sh — 9-step idempotent driver (commit 2)</name>
  <files>scripts/demo.sh</files>
  <action>
Create `scripts/demo.sh`, `chmod +x`. Structure:

```bash
#!/usr/bin/env bash
set -euo pipefail

# --- CONFIG ---
SIM_BIND="127.0.0.1:7878"
SIM_DB="/tmp/demo-sim.db"
MNT="/tmp/demo-mnt"
REPO="/tmp/demo-repo"
SEED="crates/reposix-sim/fixtures/seed.json"

# --- CLEANUP TRAP (runs on EXIT no matter what) ---
cleanup() {
  fusermount3 -u "$MNT" 2>/dev/null || true
  pkill -f "reposix-cli.*sim.*$SIM_BIND" 2>/dev/null || true
  rm -rf "$MNT" "$REPO" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
}
trap cleanup EXIT

banner() { echo; echo "==[ $1 ]== $2"; sleep 0.4; }

wait_for_url() {
  local url=$1 deadline=$((SECONDS + 5))
  while (( SECONDS < deadline )); do
    curl -sf "$url" >/dev/null && return 0
    sleep 0.1
  done
  echo "timeout waiting for $url" >&2; return 1
}

# --- PRE-FLIGHT: build release binaries OUTSIDE recording window ---
cargo build --release --workspace --bins --quiet

# --- CLEAN START ---
cleanup
mkdir -p "$MNT" "$REPO"

# [1/9] Show what we have
banner "1/9" "workspace overview"
cargo --version
ls crates/

# [2/9] Run the test suite
banner "2/9" "test suite"
cargo test --workspace --quiet --no-fail-fast

# [3/9] Start simulator
banner "3/9" "start simulator on $SIM_BIND"
cargo run --release --quiet -p reposix-cli -- sim \
    --bind "$SIM_BIND" --db "$SIM_DB" --seed-file "$SEED" &
wait_for_url "http://$SIM_BIND/healthz"

# [4/9] Mount FUSE
banner "4/9" "mount FUSE at $MNT"
cargo run --release --quiet -p reposix-cli -- mount "$MNT" \
    --backend "http://$SIM_BIND" --project demo &
sleep 1
ls "$MNT"

# [5/9] Browse
banner "5/9" "browse with shell tools"
cat "$MNT/0001.md"
grep -ril database "$MNT" || true

# [6/9] Edit via FUSE
banner "6/9" "edit an issue through FUSE"
sed -i 's/^status: open$/status: in_progress/' "$MNT/0001.md"
cat "$MNT/0001.md" | head -10
curl -s "http://$SIM_BIND/projects/demo/issues/1" | jq -r '.status'

# [7/9] git push narrative
banner "7/9" "git push round-trip"
cd "$REPO"
git init -q -b main
git config user.email demo@reposix.local
git config user.name demo
git remote add origin "reposix::http://$SIM_BIND/projects/demo"
git pull origin main --allow-unrelated-histories -q
sed -i 's/^status: in_progress$/status: in_review/' 0001.md || true
git commit -am 'request review' -q
git push origin main
curl -s "http://$SIM_BIND/projects/demo/issues/1" | jq -r '.status'
cd - >/dev/null

# [8/9] GUARDRAILS
banner "8/9a" "allowlist refusal (REPOSIX_ALLOWED_ORIGINS mismatch)"
# Spawn a second mount with an allowlist that deliberately doesn't cover the backend.
ALLOW_MNT="/tmp/demo-allow-mnt"; mkdir -p "$ALLOW_MNT"
(REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:9999" \
    cargo run --release --quiet -p reposix-cli -- mount "$ALLOW_MNT" \
    --backend "http://$SIM_BIND" --project demo 2>&1 &) || true
sleep 1
ls "$ALLOW_MNT" 2>&1 || echo "allowlist refused backend — expected"
fusermount3 -u "$ALLOW_MNT" 2>/dev/null || true
rm -rf "$ALLOW_MNT"

banner "8/9b" "SG-02 bulk-delete cap"
cd "$REPO"
git rm -q 0001.md 0002.md 0003.md 0004.md 0005.md 0006.md
git commit -am cleanup -q
# First push MUST fail with SG-02 refusal.
if git push origin main 2>&1 | tee /tmp/sg02.log | grep -q allow-bulk-delete; then
  echo "SG-02 fired as expected"
else
  echo "SG-02 did NOT fire — failing demo" >&2; exit 1
fi
git commit --amend -q -m '[allow-bulk-delete] cleanup'
git push origin main
cd - >/dev/null

banner "8/9c" "audit log truth"
sqlite3 "$SIM_DB" \
  'SELECT method, path, status FROM audit_events ORDER BY id DESC LIMIT 5;'

# [9/9] Cleanup (trap handles it, but make it visible)
banner "9/9" "cleanup"
echo "(cleanup trap will fusermount3 -u, pkill, rm)"

echo; echo "== DEMO COMPLETE =="
```

Key correctness notes:
1. `set -euo pipefail` + trap on EXIT guarantees cleanup on any failure path.
2. Release build happens OUTSIDE the timed demo so the recording isn't dominated by `cargo` compile noise.
3. The allowlist demo uses a *second* mount on `$ALLOW_MNT`; the primary mount stays healthy for the rest of the demo.
4. The SG-02 step uses `grep -q allow-bulk-delete` against the stderr output because that exact phrase appears in the refusal message (verified in S-DONE.md).
5. `git pull origin main --allow-unrelated-histories` handles the case where reposix serves an empty tree initially — harmless if the tree is non-empty too.
6. All sleeps are ≤ 1s — total demo wall-clock target ≤ 60 s after release build.

Test the script by running `bash scripts/demo.sh` twice in a row. Both must exit 0. If the second run fails with leftover state, the cleanup trap is buggy — fix it.

Commit: `feat(04-01): add scripts/demo.sh — 9-step idempotent demo driver`
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; bash scripts/demo.sh &amp;&amp; bash scripts/demo.sh</automated>
  </verify>
  <done>`bash scripts/demo.sh` exits 0 on two consecutive runs; script is executable; trap cleans up mount + sim + tmp files; all 9 banners print; SG-02 refusal grep-assertion fires; allowlist refusal produces EIO/ENOENT on `ls`; commit landed.</done>
</task>

<task type="auto">
  <name>Task 3: Record the typescript + produce transcript excerpt (commit 3)</name>
  <files>docs/demo.typescript, docs/demo.transcript.txt</files>
  <action>
Prereq: Task 2 is committed and `bash scripts/demo.sh` exits 0.

Record the demo under `script(1)`:

```bash
mkdir -p docs
script -q -c 'bash scripts/demo.sh' docs/demo.typescript
```

`script(1)` writes a raw terminal transcript (may contain ANSI escapes; that's fine — the recording is the primary artifact; the transcript is the human-readable excerpt).

Verify the recording captured the required guardrails firing. These must all appear in `docs/demo.typescript`:

```bash
grep -q "SG-02 fired as expected" docs/demo.typescript  # SG-02
grep -qE "allowlist refused|EIO|ENOENT" docs/demo.typescript  # allowlist
grep -q "== DEMO COMPLETE ==" docs/demo.typescript  # happy-path end
```

If any of the three greps fails, re-run the script from scratch, re-record, and re-verify. Do not hand-edit the typescript.

Then produce the plain-text transcript excerpt (first 200 lines, ANSI stripped for GitHub readability):

```bash
cat docs/demo.typescript | head -200 | sed -r 's/\x1B\[[0-9;]*[mK]//g' > docs/demo.transcript.txt
```

(The `sed` pattern strips CSI color/cursor escapes. If the typescript is very short — fewer than 200 lines — the excerpt is the whole thing; that's fine.)

Verify the transcript is ASCII-ish:

```bash
file docs/demo.transcript.txt | grep -qE 'ASCII|UTF-8'
```

Commit: `docs(04-01): record script(1) typescript of demo.sh + plain-text excerpt`

Commit message body should include the 3 grep-assertions that verified the recording, so the commit itself documents the guardrails-on-camera verification.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; test -s docs/demo.typescript &amp;&amp; test -s docs/demo.transcript.txt &amp;&amp; grep -q "SG-02 fired as expected" docs/demo.typescript &amp;&amp; grep -qE "allowlist refused|EIO|ENOENT" docs/demo.typescript &amp;&amp; grep -q "== DEMO COMPLETE ==" docs/demo.typescript</automated>
  </verify>
  <done>`docs/demo.typescript` exists, non-empty, contains SG-02 refusal + allowlist refusal + DEMO COMPLETE markers; `docs/demo.transcript.txt` exists, non-empty, plain-text; commit landed.</done>
</task>

<task type="auto">
  <name>Task 4: Write docs/demo.md walkthrough (commit 4)</name>
  <files>docs/demo.md</files>
  <action>
Write `docs/demo.md`. Structure (use these exact headings — downstream grep assertions rely on them):

```markdown
# reposix — end-to-end demo

One-liner: reposix mounts a REST-based issue tracker as a POSIX directory tree and translates `git push` into HTTP PATCH/POST/DELETE. This walkthrough runs the full flow against the in-process simulator.

## Reproduce in 5 minutes

Prereqs (Linux only for v0.1):
- Rust stable 1.82+
- `fusermount3` (Ubuntu: `sudo apt install fuse3`)
- `jq`, `sqlite3`, `curl` on `$PATH`

Then:

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix
bash scripts/demo.sh
```

Expect ~60 s total after the release build completes.

## Walkthrough

Each of the 9 steps below corresponds to a banner in `scripts/demo.sh`. Commands are paste-ready.

### 1/9 — What we have
(cargo --version, ls crates/) — shows the 5-crate workspace.

### 2/9 — Test suite
(cargo test --workspace --quiet) — ~133 tests, all green.

### 3/9 — Start simulator
(reposix sim --bind 127.0.0.1:7878 --db /tmp/demo-sim.db --seed-file ...) — starts the in-process HTTP fake and waits for /healthz.

### 4/9 — Mount FUSE
(reposix mount /tmp/demo-mnt --backend http://127.0.0.1:7878 --project demo) — kernel sees a new VFS, `ls /tmp/demo-mnt` lists 6 issues.

### 5/9 — Browse
(cat, grep) — `0001.md` renders with YAML frontmatter + markdown body.

### 6/9 — Edit through FUSE
(sed -i) — FUSE write path sanitizes, PATCHes the sim, `curl ... | jq .status` confirms server-side state changed.

### 7/9 — git push round-trip
(git init, git remote add origin reposix::..., git pull, git commit, git push) — diff is translated to PATCH; sim state reflects the push.

### 8/9 — Guardrails on camera
This is the section that matters. Three guardrails fire visibly:

**a) Outbound HTTP allowlist (SG-01).** Setting `REPOSIX_ALLOWED_ORIGINS` to a URL that doesn't match the configured backend causes every fetch to refuse. Proven on-camera: `ls` returns empty / errors.

**b) Bulk-delete cap (SG-02).** Deleting 6 issues in one commit is refused with:

```
error: refusing to push (would delete 6 issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)
```

Adding `[allow-bulk-delete]` to the commit message lets it through. Defends against a stray `rm -rf` on the mount point cascading into a DELETE storm.

**c) Server-controlled frontmatter strip (SG-03).** Issue 3 in the seed contains `version: 999` inside its body; the FUSE + remote-helper write paths sanitize tainted input through `Tainted<T> → sanitize()` before PATCH, so the server's authoritative version is never overwritten by client-supplied values. (This fires implicitly in step 6; the audit log in step 8c shows the PATCH happened without `version` in the payload.)

### 9/9 — Cleanup
fusermount3 -u + pkill + rm. Trap-driven; runs on any exit path.

## What the recording shows

The file `docs/demo.typescript` is the raw `script(1)` recording. `docs/demo.transcript.txt` is a plain-text excerpt (head + ANSI stripped) suitable for GitHub rendering inside a PR or issue. Both were recorded from the same `scripts/demo.sh` invocation — the recording is not hand-edited.

Three lines in the recording are the "guardrails on camera" proof:

- `SG-02 fired as expected` (bulk-delete cap)
- `allowlist refused backend — expected` (or EIO/ENOENT from `ls` on the allowlist-constrained mount)
- The step-6 `sed + curl | jq .status` pair that proves the server accepted the client's `status` change but rejected the adversarial `version: 999` override (audit log confirms).

## Limitations / honest scope

This is v0.1 alpha, built autonomously overnight. What's **not** in the demo:

- **No real backend.** Simulator-only. Real Jira/GitHub/Confluence integration is v0.2. See [PROJECT.md — Out of Scope](../.planning/PROJECT.md).
- **No man page, .deb, brew formula.** Clone-and-`cargo build`.
- **Linux only.** FUSE3/FUSE2. macOS-via-macFUSE is a follow-up.
- **Threat model is taken seriously but not exhaustively mitigated yet.** See [`threat-model-and-critique.md`](../.planning/research/threat-model-and-critique.md) — the SG-01/02/03 cuts demonstrated here close the most lethal-trifecta paths but do not cover every M-* finding in the red-team report. Those are deferred to v0.2.
```

Verify the two grep-assertions the ROADMAP demands succeed against this file:

```bash
grep -c '## Walkthrough' docs/demo.md        # must be ≥ 1
grep -cE 'ALLOWED_ORIGINS|allowlist' docs/demo.md  # must be ≥ 1
```

Commit: `docs(04-01): add docs/demo.md walkthrough with guardrails callout`
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; grep -q '## Walkthrough' docs/demo.md &amp;&amp; grep -qE 'ALLOWED_ORIGINS|allowlist' docs/demo.md &amp;&amp; grep -q 'threat-model-and-critique.md' docs/demo.md</automated>
  </verify>
  <done>`docs/demo.md` exists; `## Walkthrough` heading present; allowlist mentioned; threat-model link present; paste-ready reproduce section; commit landed.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| script → local shell | `scripts/demo.sh` runs real binaries as the invoking user; no elevation. All writes are under `/tmp/demo-*` or the repo workspace. |
| demo → GitHub `main` | The recording + script commit to `main` — the artifacts become part of the public repo. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-01 | I (Info disclosure) | `docs/demo.typescript` | mitigate | Record uses only simulator seed data; no real credentials, no personal paths beyond `/tmp/`. Verified by grepping the typescript for patterns like `HOME=`, `/home/`, `ssh`, `token` before commit. |
| T-04-02 | T (Tamper) | `scripts/demo.sh` left over state | mitigate | `trap cleanup EXIT` with `fusermount3 -u` + `pkill` + `rm -rf /tmp/demo-*`. Idempotency test (two consecutive runs) is the regression guard. |
| T-04-03 | S (Spoofing) | Recording authenticity | accept | `script(1)` timestamps are trusted-by-invocation; we do not claim cryptographic provenance on the recording. The recording is a demo artifact, not a signed attestation. |
| T-04-04 | E (Elevation) | `REPOSIX_ALLOWED_ORIGINS` misuse in demo step 8a | mitigate | The allowlist demo runs a *second* mount process scoped to `$ALLOW_MNT`; it cannot affect the primary mount or sim. Cleanup unmounts the allowlist mount before the next step. |
</threat_model>

<verification>
After all 4 tasks land:

1. `bash scripts/demo.sh` exits 0 (idempotent — two consecutive runs).
2. `docs/demo.typescript` contains the SG-02 + allowlist + DEMO COMPLETE markers.
3. `docs/demo.md` grep assertions (`## Walkthrough`, `ALLOWED_ORIGINS|allowlist`, `threat-model`) all pass.
4. `seed.json` parses to 6 issues.
5. `cargo test -p reposix-sim --quiet` green.
</verification>

<success_criteria>
- [ ] Seed extended to 6 issues, sim tests still green.
- [ ] `scripts/demo.sh` exits 0 on two consecutive runs from a clean working dir.
- [ ] Recording captures SG-02 bulk-delete refusal + allowlist refusal + DEMO COMPLETE.
- [ ] `docs/demo.transcript.txt` is plain-text, ≤200 lines.
- [ ] `docs/demo.md` has `## Walkthrough`, mentions allowlist, links to threat-model.
- [ ] Four atomic commits (one per task).
</success_criteria>

<output>
After completion, create `.planning/phases/04-demo-recording-readme/04-01-SUMMARY.md` with:
- The exact grep-assertions that verified the recording (timestamped).
- Second-run exit code and wall-clock time of `bash scripts/demo.sh`.
- The 4 commit SHAs.
- Any deviations from this plan.
</output>
