# Phase 4: Demo + recording + README polish — Context

**Gathered:** 2026-04-13 03:47 PDT
**Status:** Ready for execution. MVD + STRETCH all shipped.
**Source:** Auto-generated; discuss step skipped per user instruction.

<domain>
## Phase Boundary

**In scope:**
- `scripts/demo.sh` — idempotent end-to-end demo driver. Runs the full narrative top-to-bottom and exits 0. The recording captures THIS script verbatim.
- `docs/demo.md` — walkthrough page that links to the script + recording + screenshots.
- Recording — `script(1)` typescript (`docs/demo.typescript`) + a hand-typed transcript-of-record (`docs/demo.transcript.txt`) showing what the user would see. (`asciinema` is unavailable on the dev host, per pre-flight check.)
- `README.md` polish:
  - Replace `🚧` markers with `✓` for what shipped (which is everything).
  - Add a `## Demo` section linking to `docs/demo.md` + the typescript.
  - Add a `## Security` section that's brutally honest about lethal trifecta + links to the threat model + names what's deferred to v0.2.
  - Add a `## Status` block at the top showing the phase-by-phase ship status.
  - Honest scope statement: this was built autonomously overnight; treat as `alpha` per Simon's "proof of usage" rule.
- Final `gh run list` check — CI must be green at the commit the recording was made against.

**Out of scope:**
- A web UI / dashboard.
- A `man` page (defer to v0.2).
- An installable .deb / brew formula.
- Real-backend integration (Jira/GitHub) — explicitly out of scope per PROJECT.md.

</domain>

<decisions>
## Implementation Decisions

### Demo narrative (the recording must show ALL of this)

```
[1/9] Show what we have: cargo --version, ls crates/
[2/9] Run the test suite: cargo test --workspace --quiet (133 tests green)
[3/9] Start the simulator in background: reposix sim --bind 127.0.0.1:7878 --db /tmp/demo-sim.db --seed-file ...
[4/9] Mount the FUSE filesystem: reposix mount /tmp/demo-mnt --backend http://127.0.0.1:7878
[5/9] Browse the mount with shell tools:
        ls /tmp/demo-mnt
        cat /tmp/demo-mnt/0001.md
        grep -ril database /tmp/demo-mnt
[6/9] EDIT an issue: sed -i 's/^status: open$/status: in_progress/' /tmp/demo-mnt/0001.md
        diff via cat — show new state.
        Verify backend updated: curl http://127.0.0.1:7878/projects/demo/issues/1 | jq '.frontmatter.status'
[7/9] git push narrative:
        cd /tmp/demo-repo (a fresh git repo)
        git init && git remote add origin reposix::http://127.0.0.1:7878/projects/demo
        git pull origin main  # fast-import populates the working tree
        sed -i 's/^status: in_progress$/status: in_review/' 0001.md
        git commit -am 'request review'
        git push origin main  # PATCH
        Verify: curl ... | jq .frontmatter.status -> "in_review"
[8/9] GUARDRAILS FIRING:
        a) Allowlist refusal: REPOSIX_ALLOWED_ORIGINS=http://nope.example reposix sim --bind 127.0.0.1:7878 ... → no, run a curl that proves a non-allowlisted backend is rejected. Show the EPERM/error.
        b) SG-02 bulk-delete cap: cd /tmp/demo-repo, rm 0001.md 0002.md 0003.md ... (6 files), git commit -am cleanup, git push → REFUSED with the allow-bulk-delete hint. Then git commit --amend -m '[allow-bulk-delete] cleanup' && git push → succeeds.
        c) Audit log truth: sqlite3 /tmp/demo-sim.db 'SELECT method, path, status FROM audit_events ORDER BY id DESC LIMIT 5;'
[9/9] Cleanup: fusermount3 -u /tmp/demo-mnt; pkill reposix-sim
```

The recording is run under `script(1)`:
```
script -q -c 'bash scripts/demo.sh' docs/demo.typescript
```
Then `cat docs/demo.typescript | head -200 > docs/demo.transcript.txt` for a non-binary excerpt.

### `scripts/demo.sh` design
- Bash with `set -euo pipefail`.
- Top-level cleanup trap that always runs (`fusermount3 -u`, `kill %1`, `rm -rf /tmp/demo-*`).
- Each step prints a banner: `echo; echo "==[ $N/9 ] $description ==="`.
- Pauses (`sleep 0.4`) between steps so the recording is human-watchable.
- Idempotent: re-running it cleanly tears down prior state.
- Returns exit 0 on the happy path; returns non-zero on any failure (so CI can also run it).
- Uses `cargo run --quiet --release` for binaries (release because the demo should be fast — but build first OUTSIDE the recording so the recording isn't dominated by compile time).
- `wait_for_url` helper (5s deadline) before any curl.

### `docs/demo.md` design
- Markdown.
- Sections:
  - **Demo** (overview + 1-liner of what reposix is, link to script + typescript)
  - **Reproduce in 5 minutes** (numbered prereqs + `bash scripts/demo.sh`)
  - **Walkthrough** (each of the 9 steps with explanation)
  - **What the recording shows** (highlight the 3 guardrails)
  - **Limitations / honest scope** (link to PROJECT.md Out of Scope and threat-model)

### README polish design
- Status badges at top: CI ✓, version ✓, license ✓.
- `## Status` block summarizing what shipped this build.
- Replace 🚧 with ✓.
- Add `## Quickstart` (prereqs + 3-line install + `bash scripts/demo.sh`).
- `## Architecture` (small ASCII diagram already there — keep).
- `## Security` (lethal-trifecta paragraph + link to research/threat-model + table of what's enforced vs deferred).
- `## Honest scope` (built autonomously overnight; alpha; no real-backend yet; `proof of usage` paragraph from Simon's interview).
- `## Contributing` (small — pre-commit hooks, where to find docs).
- License footer.

### Tests in this phase
- `bash scripts/demo.sh` exits 0 (this becomes the goal-backward verification).
- `cargo test --workspace` still green at the recorded commit.
- `gh run list --limit 1 --json conclusion -q '.[0].conclusion'` returns `success`.

### Claude's discretion
- Exact prose in README.
- Number of seconds in `sleep` between demo steps.
- Whether to render the typescript as plain text or also embed in `docs/demo.md` as a `<pre>` block.
- Whether to add screenshots (probably not — typescript is sufficient).

</decisions>

<canonical_refs>
## Canonical References

- `.planning/PROJECT.md` — scope, requirements (mark as Validated after this phase).
- `.planning/research/threat-model-and-critique.md` — the Security section MUST link here.
- `.planning/phases/S-stretch-write-path-and-remote-helper/S-DONE.md` — what shipped.
- `crates/reposix-cli/src/demo.rs` — existing `reposix demo` subcommand. The shell script `scripts/demo.sh` extends it with the git push + guardrails segments. (We don't need to extend the Rust subcommand; the shell script can drive the whole thing.)
- [script(1) man page](https://man7.org/linux/man-pages/man1/script.1.html)
- [SG-02 message format] — exact text from the bulk-delete cap commit.

</canonical_refs>

<specifics>
## Specific Ideas

- **Run release builds before the recording.** `cargo build --release --workspace --bins` so the recording isn't 30s of compilation.
- **Pre-create `/tmp/demo-mnt` and `/tmp/demo-repo`** in the script setup so the recording skips noise.
- **The allowlist refusal demo** is the trickiest — we need a way to make the FUSE daemon try to talk to a non-allowlisted host without it actually being a real attack. Approach: run `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:9999 reposix mount /tmp/demo-mnt --backend http://127.0.0.1:7878 &` (allowlist constrains to a port that doesn't match the backend). The mount will start but every fetch fails with `EIO` (allowlist refusal). `ls /tmp/demo-mnt` returns empty + stderr shows the refusal. This is the on-camera proof the allowlist is doing its job.
- **The SG-02 cap demo** uses 6 deletes — the existing seed has only 3 issues, so we need to first POST 3 more (curl) then `git pull` to bring them into the working tree, then `rm` 6 of them. Or simpler: extend the seed to have 6+ issues. (Easier — seed extension.)

</specifics>

<deferred>
## Deferred Ideas

- A v0.2 hardening pass with M-* findings from REVIEW.md.
- An asciinema upload (would require asciinema install).
- A blog post / launch announcement.
- A man page.

</deferred>

---

*Phase: 04-demo-recording-readme*
*Context: 2026-04-13 03:47 PDT via auto-mode*
