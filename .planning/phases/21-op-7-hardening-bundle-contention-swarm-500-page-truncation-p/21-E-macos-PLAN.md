---
phase: 21
plan: E
type: execute
wave: 5
depends_on: [21-A]
files_modified:
  - .github/workflows/ci.yml
  - crates/reposix-fuse/tests/nested_layout.rs
  - crates/reposix-fuse/tests/sim_death_no_hang.rs
autonomous: false
requirements:
  - HARD-04
user_setup: []
tags: [hardening, ci, macos, macfuse, parity]

must_haves:
  truths:
    - "CI runs the FUSE integration test suite on both ubuntu-latest and macos-14 runners via a matrix"
    - "On macOS, the runner installs macFUSE via a pinned action and teardown uses `umount -f` instead of `fusermount3 -u`"
    - "The credential pre-push hook test runs in CI on Linux (closes the HARD-00 gap flagged in RESEARCH.md)"
    - "Runner image `macos-14` is pinned (not `macos-latest`) per RESEARCH.md Pitfall 3"
    - "FUSE test teardown reads the unmount command from `$REPOSIX_UNMOUNT_CMD` so the same test binary works on both OSes"
  artifacts:
    - path: ".github/workflows/ci.yml"
      provides: "matrix for integration job + hooks CI step"
      contains: "macos-14"
    - path: "crates/reposix-fuse/tests/nested_layout.rs"
      provides: "teardown honours REPOSIX_UNMOUNT_CMD env var"
      contains: "REPOSIX_UNMOUNT_CMD"
    - path: "crates/reposix-fuse/tests/sim_death_no_hang.rs"
      provides: "same teardown conditional"
      contains: "REPOSIX_UNMOUNT_CMD"
  key_links:
    - from: ".github/workflows/ci.yml"
      to: "crates/reposix-fuse/tests/"
      via: "env: REPOSIX_UNMOUNT_CMD set per runner.os"
      pattern: "REPOSIX_UNMOUNT_CMD"
    - from: ".github/workflows/ci.yml (macOS step)"
      to: "gythialy/macfuse GitHub Action"
      via: "uses: gythialy/macfuse@<pinned>"
      pattern: "gythialy/macfuse"
---

<objective>
Add macOS CI matrix coverage so `reposix-fuse` integration tests run on both Linux (`ubuntu-latest`) and macOS (`macos-14`) runners, with conditional FUSE installation and conditional unmount command. Also wire the credential pre-push hook test as a CI step so HARD-00 stays regression-safe over time.

Purpose: Per HARD-04, macOS + macFUSE parity is today Linux-only (verified: `crates/reposix-fuse/tests/*.rs` hardcodes `fusermount3`). Without this plan, a Mac developer has no CI signal and a Mac reposix user has no assurance the FUSE path works on macOS at all.

This plan carries a `checkpoint:human-verify` because macOS CI costs paid runner minutes per push, and the first run needs human eyes for kext-approval / action-version quirks that research flagged as ASSUMED.

Output: Updated `ci.yml` with `strategy.matrix.os`, FUSE tests reading `$REPOSIX_UNMOUNT_CMD`, a hooks CI step, and a human-verified first green run on macOS.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/CONTEXT.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md
@.github/workflows/ci.yml
@crates/reposix-fuse/tests/nested_layout.rs
@crates/reposix-fuse/tests/sim_death_no_hang.rs
@scripts/hooks/test-pre-push.sh

<interfaces>
Current `integration` job shape (per PATTERNS.md §ci.yml; verify via Read before edits):

```yaml
integration:
  name: integration (mounted FS)
  runs-on: ubuntu-latest
  needs: [test]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Install runtime FUSE binaries
      run: sudo apt-get update && sudo apt-get install -y fuse3
    - name: Verify FUSE available
      run: ls -l /dev/fuse && which fusermount3
    - name: Build release binaries
      run: cargo build --release --workspace --bins
    - name: Run integration tests (requires FUSE)
      run: cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
```

Current FUSE test teardown idiom (verified by grep — `fusermount3` appears in both test files):
```rust
// Exact sites enumerated in task E1; pattern is direct Command invocation
std::process::Command::new("fusermount3").args(["-u", mnt]).status();
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task E1: Parametrise FUSE test teardown via $REPOSIX_UNMOUNT_CMD</name>
  <files>
    crates/reposix-fuse/tests/nested_layout.rs,
    crates/reposix-fuse/tests/sim_death_no_hang.rs
  </files>
  <read_first>
    - crates/reposix-fuse/tests/nested_layout.rs (every fusermount3 call site)
    - crates/reposix-fuse/tests/sim_death_no_hang.rs (every fusermount3 call site)
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section ".github/workflows/ci.yml" — env var contract)
  </read_first>
  <action>
    Replace every hardcoded `fusermount3 -u <mnt>` teardown invocation with an `unmount()` helper that reads `$REPOSIX_UNMOUNT_CMD`, defaulting to `fusermount3 -u`.

    **Step 1 — add the helper.** At the top of `nested_layout.rs` (and mirror into `sim_death_no_hang.rs`), add:

    ```rust
    /// Run the appropriate unmount command for the current runner.
    /// On Linux this defaults to `fusermount3 -u <mnt>`; on macOS CI sets
    /// `REPOSIX_UNMOUNT_CMD=umount -f` so the same test binary works.
    fn unmount(mnt: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
        let cmd_str = std::env::var("REPOSIX_UNMOUNT_CMD")
            .unwrap_or_else(|_| "fusermount3 -u".to_string());
        let mut parts = cmd_str.split_whitespace();
        let prog = parts.next().expect("REPOSIX_UNMOUNT_CMD is empty");
        let args: Vec<&str> = parts.collect();
        std::process::Command::new(prog)
            .args(&args)
            .arg(mnt)
            .status()
    }
    ```

    If the crate already uses a `tests/common/mod.rs` (check first), put the helper there and `mod common;` from each test file. Otherwise duplicating the 12-line helper in both files is acceptable.

    **Step 2 — replace teardown call sites.** In both files, grep for `fusermount3` and replace each `Command::new("fusermount3")...` used for teardown with `unmount(&mnt_path)`. Preserve the existing error handling (`.ok()`, `.expect(...)`, `.unwrap()`) at each site — do not silently change error behaviour.

    If any `fusermount3` reference is for something OTHER than unmount (e.g. `which fusermount3` availability check), LEAVE IT. Only teardown call sites are in scope.

    **Step 3 — verify locally (Linux only; macOS is CI-only).**
    ```
    cargo test -p reposix-fuse --locked
    cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
    cargo clippy -p reposix-fuse --all-targets -- -D warnings
    ```
    All three must exit 0. The integration tests pass because the default (`fusermount3 -u`) is the same as the old hardcoded string.

    **Step 4 — commit.**
    `git add crates/reposix-fuse/tests/nested_layout.rs crates/reposix-fuse/tests/sim_death_no_hang.rs && git commit -m "refactor(21-E): parametrise FUSE teardown via REPOSIX_UNMOUNT_CMD (macOS CI prep)"`
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --locked && cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 && cargo clippy -p reposix-fuse --all-targets -- -D warnings</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q "REPOSIX_UNMOUNT_CMD" crates/reposix-fuse/tests/nested_layout.rs` succeeds
    - `grep -q "REPOSIX_UNMOUNT_CMD" crates/reposix-fuse/tests/sim_death_no_hang.rs` succeeds
    - `grep -qE "fn unmount" crates/reposix-fuse/tests/nested_layout.rs` (or tests/common/mod.rs) succeeds
    - `grep -cE "Command::new\(\"fusermount3\"\)[^\n]*\"-u\"" crates/reposix-fuse/tests/nested_layout.rs` returns 0
    - `grep -cE "Command::new\(\"fusermount3\"\)[^\n]*\"-u\"" crates/reposix-fuse/tests/sim_death_no_hang.rs` returns 0
    - `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1` exits 0 on the Linux dev host
    - `cargo clippy -p reposix-fuse --all-targets -- -D warnings` exits 0
  </acceptance_criteria>
  <done>
    Teardown is OS-agnostic; Linux behaviour unchanged; macOS ready for the matrix wiring in E2.
  </done>
</task>

<task type="auto">
  <name>Task E2: Add macOS matrix + hooks step to ci.yml</name>
  <files>
    .github/workflows/ci.yml
  </files>
  <read_first>
    - .github/workflows/ci.yml (full file)
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section ".github/workflows/ci.yml")
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md (section "Pattern 5: macOS + macFUSE CI")
  </read_first>
  <action>
    **Step 1 — update the `integration` job.** Apply the matrix extension pattern from PATTERNS.md §ci.yml. Pin `macos-14` (Sonoma), NOT `macos-latest`, per RESEARCH.md Pitfall 3.

    Final shape (paste verbatim, adapting only names that differ from the actual current file):

    ```yaml
    integration:
      name: integration (mounted FS)
      strategy:
        fail-fast: false
        matrix:
          os: [ubuntu-latest, macos-14]
      runs-on: ${{ matrix.os }}
      needs: [test]
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - uses: Swatinem/rust-cache@v2
        - name: Install FUSE (Linux)
          if: runner.os == 'Linux'
          run: sudo apt-get update && sudo apt-get install -y fuse3
        - name: Install macFUSE (macOS)
          if: runner.os == 'macOS'
          uses: gythialy/macfuse@v1
        - name: Verify FUSE available (Linux)
          if: runner.os == 'Linux'
          run: ls -l /dev/fuse && which fusermount3
        - name: Verify FUSE available (macOS)
          if: runner.os == 'macOS'
          run: which mount_macfuse || which mount_osxfuse || true
        - name: Build release binaries
          run: cargo build --release --workspace --bins
        - name: Run integration tests (requires FUSE)
          env:
            REPOSIX_UNMOUNT_CMD: ${{ runner.os == 'macOS' && 'umount -f' || 'fusermount3 -u' }}
          run: cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
    ```

    **Step 2 — pin the `gythialy/macfuse` action.** RESEARCH.md labels this as ASSUMED. Before committing:
    - Use the Bash tool to curl `https://api.github.com/repos/gythialy/macfuse/releases/latest` (or WebFetch https://github.com/gythialy/macfuse/releases) to confirm the action exists and find the current tag/SHA.
    - If the action exists and has a `v1` (or newer major) tag, pin to that tag (`gythialy/macfuse@v1`).
    - If the project publishes SHA-pinning recommendations, pin to the SHA.
    - If the action does NOT exist or is unmaintained, STOP and surface in the checkpoint (E3) with the fallback option documented. Do NOT attempt `brew install --cask macfuse` in a CI step — kext approval will hang the runner.

    **Step 3 — add the pre-push hook test step.** Per RESEARCH.md §Open Questions #3, this closes a small regression gap in HARD-00. Inside the existing `test` job (not the integration job; the hook test is pure bash and has no FUSE dependency), append after the existing `cargo test` step:

    ```yaml
    - name: Test pre-push credential hook
      run: bash scripts/hooks/test-pre-push.sh
    ```

    **Step 4 — local YAML syntax check.** Run:
    ```
    python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
    ```
    Exit 0 means the file parses. Cargo tests do NOT validate GH Actions expression syntax (`${{ ... }}`); visual inspection plus a dry push to a throwaway branch is the real check (handled in E3).

    **Step 5 — commit WITHOUT pushing.**
    `git add .github/workflows/ci.yml && git commit -m "ci(21-E): add macOS matrix + pre-push hook test (HARD-04 + HARD-00)"`

    Do NOT push. Task E3 is a checkpoint BEFORE push.
  </action>
  <verify>
    <automated>python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && grep -q "macos-14" .github/workflows/ci.yml && grep -q "REPOSIX_UNMOUNT_CMD" .github/workflows/ci.yml && grep -q "test-pre-push.sh" .github/workflows/ci.yml && grep -q "gythialy/macfuse" .github/workflows/ci.yml</automated>
  </verify>
  <acceptance_criteria>
    - `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` exits 0 (valid YAML)
    - `grep -q "macos-14" .github/workflows/ci.yml` (pinned runner, not macos-latest)
    - `grep -qE "matrix:\s*$" .github/workflows/ci.yml` — matrix block present
    - `grep -q "REPOSIX_UNMOUNT_CMD" .github/workflows/ci.yml`
    - `grep -q "gythialy/macfuse@" .github/workflows/ci.yml` (action referenced with @-pin)
    - `grep -q "runner.os == 'Linux'" .github/workflows/ci.yml` — conditional Linux step
    - `grep -q "runner.os == 'macOS'" .github/workflows/ci.yml` — conditional macOS step
    - `grep -q "test-pre-push.sh" .github/workflows/ci.yml` — hook step present
    - `grep -c "macos-latest" .github/workflows/ci.yml` returns 0 (no un-pinned macOS runner)
    - Commit exists: `git log -1 --oneline -- .github/workflows/ci.yml | grep -q "21-E"`
  </acceptance_criteria>
  <done>
    ci.yml contains matrix + hooks step; YAML parses; commit made without push.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task E3: Human-verify first macOS CI run + decide push strategy</name>
  <files>.github/workflows/ci.yml</files>
  <read_first>
    - .github/workflows/ci.yml (as committed in E2)
    - This task's `<how-to-verify>` block (it IS the script for the human)
  </read_first>
  <action>
    This is a human-verify checkpoint. The executor does NOT push or modify code here; the USER drives the verification per `<how-to-verify>`. After the user responds with one of the `<resume-signal>` options, the executor (or a follow-up plan) acts on the response: record outcome in SUMMARY.md, revert matrix if blocked, or re-pin action if retry requested. No new code lands in this task itself.
  </action>
  <what-built>
    - `crates/reposix-fuse/tests/{nested_layout,sim_death_no_hang}.rs` now honour `REPOSIX_UNMOUNT_CMD` (E1)
    - `.github/workflows/ci.yml` has `[ubuntu-latest, macos-14]` matrix for the integration job, `gythialy/macfuse` pinned action on macOS steps, and a new `bash scripts/hooks/test-pre-push.sh` step in the `test` job (E2)
    - Commits are made locally but NOT pushed
  </what-built>
  <how-to-verify>
    1. Confirm local state:
       - `git log --oneline -5` shows the E1 + E2 commits with `21-E` prefix
       - `git status` is clean
    2. Decide push strategy. Options:
       - **A. Push to a throwaway branch first** (recommended): `git checkout -b phase-21-E-macos-trial && git push -u origin phase-21-E-macos-trial`. Watch the Actions tab on GitHub. The ubuntu-latest leg of the matrix should pass in ~same time as today. The macos-14 leg is the unknown — look for:
         - gythialy/macfuse action install step: did it succeed or fail? If it failed with "action not found" or similar, the action version needs re-pinning. If it failed with a kext-approval / SIP error, `macos-14` is still rejecting the kext and we need to accept that HARD-04 is blocked pending a different approach (documented in the SUMMARY).
         - Build step: macOS Rust toolchain via dtolnay/rust-toolchain@stable — should be fine.
         - Integration test step: `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1` — this is the real test. If it times out on mount, the kext is not loaded. If individual tests fail on specific assertions, that's a real parity bug to investigate.
       - **B. Push directly to `main`** (only if confident): `git push origin main`. Same observations, but any macOS failure blocks the entire main-branch CI until reverted.
    3. After watching the first macOS CI run, document the outcome:
       - SUCCESS: integration leg green on both OSes → proceed, no further action
       - PARTIAL: Linux green, macOS has specific fixable failure → file follow-up with exact failure text (can be a separate plan); push to main with `fail-fast: false` so Linux signal is preserved
       - BLOCKED: macOS CI fundamentally cannot run (action deprecated, kext approval impossible on GH runners in 2026) → document this as a known limitation in 21-E-SUMMARY.md; revert the matrix change from ci.yml and keep only the E1 teardown refactor + hooks step; file an OP-7a follow-up for a future approach (self-hosted Mac runner, or wait for GH Actions macFUSE support to mature)
  </how-to-verify>
  <verify>
    <automated>test -f .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-E-SUMMARY.md && grep -q "macOS CI first-run result" .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-E-SUMMARY.md</automated>
  </verify>
  <resume-signal>
    Type one of:
    - "macos-green" — both OS legs passed; task complete, no changes needed
    - "macos-partial: <description>" — specific fixable failure; executor files follow-up, proceeds
    - "macos-blocked: <reason>" — revert matrix from ci.yml, keep E1 refactor + hooks step, document follow-up
    - "retry with pin @<tag-or-sha>" — executor re-pins gythialy/macfuse action and re-verifies
  </resume-signal>
  <acceptance_criteria>
    - User has observed at least one CI run that exercises the macos-14 matrix leg (success, failure, or skip)
    - The outcome is recorded in `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-E-SUMMARY.md` under a "macOS CI first-run result" section
    - If outcome is "macos-blocked", `.github/workflows/ci.yml` no longer contains `macos-14` (matrix reverted), but still contains `test-pre-push.sh` and `REPOSIX_UNMOUNT_CMD` (E1 refactor kept)
    - If outcome is "macos-green" or "macos-partial", `.github/workflows/ci.yml` still contains `macos-14`
    - `gh run list --workflow=ci.yml --limit 3 --json conclusion,headBranch | jq` shows at least one run after the Phase 21-E push (verified manually by user)
  </acceptance_criteria>
  <done>User has signalled one of the resume-signal responses; 21-E-SUMMARY.md records the outcome and any remediation taken.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| third-party GH Action (gythialy/macfuse) → CI runner | action code executes with runner-level privileges; must be pinned |
| macOS CI runner → test binaries | FUSE kext loads into kernel; trust boundary is the CI runner itself |
| CI pushing to GitHub → public Actions logs | no secrets in FUSE integration tests; logs are safe |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-E-01 | Tampering | gythialy/macfuse action unpinned | mitigate | Pin to `@v1` or SHA per E2 step 2. SHA-pin preferred for supply-chain hardening; tag-pin acceptable for a dev-only tool (no secret access in integration job). |
| T-21-E-02 | Elevation of Privilege | macFUSE kext loads into macOS kernel on CI runner | accept | Runner is ephemeral; no persistence between runs. Kext is the minimum trust required to test FUSE on macOS at all. |
| T-21-E-03 | Denial of Service | macOS runner minutes expensive; matrix doubles CI cost | accept | `fail-fast: false` ensures Linux signal survives macOS flakes. If cost becomes painful, a follow-up can path-filter the macOS leg to only run on PRs that touch crates/reposix-fuse/. |
| T-21-E-04 | Information Disclosure | CI logs on push might capture env vars | accept | `REPOSIX_UNMOUNT_CMD` is not a secret; no tokens in FUSE integration job. Pre-push hook test runs on a detached HEAD with fake-prefix tokens only — no real secrets possible. |
</threat_model>

<verification>
- E1: `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1` green on Linux dev host
- E1: `grep` acceptance criteria listed under E1 all succeed
- E2: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` exits 0
- E2: grep acceptance criteria for ci.yml all succeed
- E3: `gh run list --workflow=ci.yml --limit 3` shows at least one run after the Phase 21-E push
- E3: 21-E-SUMMARY.md contains a "macOS CI first-run result" section
</verification>

<success_criteria>
HARD-04 closes via one of three documented paths:
1. macOS CI green on macos-14 — full parity
2. macOS CI partially green with follow-up filed — parity-in-progress
3. macOS CI blocked — documented limitation + OP-7a follow-up filed; E1 refactor + hooks step still ship as durable wins

HARD-00's hook-test-in-CI gap (RESEARCH.md §Open Questions #3) also closes regardless of the macOS outcome.
</success_criteria>

<output>
After completion, update `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-E-SUMMARY.md` with:
- Files modified (ci.yml, two FUSE test files)
- Decision made at E3 checkpoint (green / partial / blocked)
- macFUSE action pin chosen
- Link to first CI run
- Any follow-up plan IDs filed
</output>
