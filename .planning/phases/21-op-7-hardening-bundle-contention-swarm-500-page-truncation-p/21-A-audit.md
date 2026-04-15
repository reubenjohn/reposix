---
phase: 21
plan: A
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md
autonomous: true
requirements:
  - HARD-00
user_setup: []
tags: [hardening, audit, credential-hygiene, ssrf]

must_haves:
  truths:
    - "Operator can prove the credential pre-push hook still rejects a literal ATATT3-prefixed secret with high entropy"
    - "Operator can prove the three SSRF wiremock tests in reposix-confluence still pass"
    - "Any gap in either already-done item is captured in AUDIT-NOTES before Wave B starts"
  artifacts:
    - path: ".planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md"
      provides: "Audit result + gap list for HARD-00"
      contains: "pre-push: PASS|FAIL"
  key_links:
    - from: "scripts/hooks/pre-push"
      to: "scripts/hooks/test-pre-push.sh"
      via: "bash execution — hook file scanned by test harness"
      pattern: "ATATT3"
    - from: "crates/reposix-confluence/tests/contract.rs"
      to: "cargo test --workspace"
      via: "test harness discovery of adversarial_ / ssrf-prefixed test fns"
      pattern: "adversarial_.*does_not_trigger_outbound_call"
---

<objective>
Audit the two OP-7 items that were shipped in session-4 drive-bys (commits `f357c92`+`5361fd5` for the credential pre-push hook; `ea5e548` for the SSRF regression tests). This plan produces a signed audit note — it does NOT write new code unless the audit finds a gap. If a gap is found, the audit note spells it out as a follow-up for a later wave; it does NOT absorb that work.

Purpose: Per RESEARCH.md §"Audit of Session-4 Drive-By Items", both items are believed complete. This plan forces us to *prove* that before waves B–E assume green floor. Per CLAUDE.md Operating Principle #1 (close the feedback loop), "shipped" is not the same as "still passing".

Output: `21-A-AUDIT-NOTES.md` with PASS/FAIL for each item, test counts, and the exact command outputs that prove it.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/CONTEXT.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md
@scripts/hooks/pre-push
@scripts/hooks/test-pre-push.sh
@crates/reposix-confluence/tests/contract.rs

<interfaces>
<!-- Key invariants the executor verifies. No code contract changes here. -->

Credential hook expected behavior (scripts/hooks/pre-push):
- Scans staged files for patterns: `ATATT3[A-Za-z0-9]{20,}`, `ghp_[A-Za-z0-9]{36}`,
  `github_pat_[A-Za-z0-9_]{30,}`, and `Bearer ATATT3[A-Za-z0-9]{20,}`.
- Excludes scripts/hooks/ so the hook doesn't self-reject.
- Exits non-zero if any match found; returns zero on clean tree.

SSRF regression tests expected names (crates/reposix-confluence/tests/contract.rs):
- adversarial_links_base_does_not_trigger_outbound_call
- adversarial_webui_link_does_not_trigger_outbound_call
- adversarial_host_in_arbitrary_string_field_is_ignored
Each uses wiremock legit_server + decoy_server with .expect(0) on decoy.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task A1: Verify credential-hygiene pre-push hook still passes 6/6</name>
  <read_first>
    - scripts/hooks/pre-push
    - scripts/hooks/test-pre-push.sh
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md (section "Credential hygiene pre-push hook — COMPLETE")
  </read_first>
  <action>
    Run `bash scripts/hooks/test-pre-push.sh` from repo root. Capture stdout+stderr to a tempfile. The test script creates a detached HEAD, makes throwaway commits with fake tokens (ATATT3..., ghp_..., etc.), runs the hook against each, and cleans up. Expected: exit code 0, output contains "6/6" or equivalent PASS line per test case.

    If exit is non-zero: capture the failure(s) verbatim and list them as gaps in AUDIT-NOTES. Do NOT fix the hook in this plan — that becomes a new plan.

    Also grep the repo for any committed `ATATT3[A-Za-z0-9]{20,}` literal (excluding scripts/hooks/ and the phase directory itself, since CONTEXT.md may reference the prefix as a string). Command: `grep -rE "ATATT3[A-Za-z0-9]{20,}" . --exclude-dir=scripts/hooks --exclude-dir=.planning --exclude-dir=target --exclude-dir=.git || echo "no real tokens found"`. Expected output: the "no real tokens found" literal.
  </action>
  <verify>
    <automated>bash scripts/hooks/test-pre-push.sh</automated>
  </verify>
  <acceptance_criteria>
    - `bash scripts/hooks/test-pre-push.sh` exits 0
    - The grep for `ATATT3[A-Za-z0-9]{20,}` in tracked source (excluding scripts/hooks/ and .planning/) returns no matches
    - AUDIT-NOTES.md (created in task A3) contains a line matching `pre-push: PASS` — verify with `grep -E "^pre-push: PASS$" .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md`
  </acceptance_criteria>
  <done>
    Hook test 6/6 green; no real tokens in repo; outcome captured for AUDIT-NOTES.
  </done>
</task>

<task type="auto">
  <name>Task A2: Verify SSRF regression tests still pass (3/3)</name>
  <read_first>
    - crates/reposix-confluence/tests/contract.rs
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md (section "SSRF regression tests — COMPLETE")
  </read_first>
  <action>
    Run `cargo test -p reposix-confluence --test contract adversarial_ --locked 2>&1 | tee /tmp/21-A-ssrf.log`. The three tests are named:
      - `adversarial_links_base_does_not_trigger_outbound_call`
      - `adversarial_webui_link_does_not_trigger_outbound_call`
      - `adversarial_host_in_arbitrary_string_field_is_ignored`

    Expected: `test result: ok. 3 passed; 0 failed` (and `0 ignored` for the three — none are `#[ignore]`). If any fail, do NOT fix in this plan — record in AUDIT-NOTES as a gap.

    Also grep the file to confirm all three fn names exist and each uses `.expect(0)` on the decoy server. Command:
    `grep -cE "\.expect\(0\)" crates/reposix-confluence/tests/contract.rs` — expect count >= 3.
  </action>
  <verify>
    <automated>cargo test -p reposix-confluence --test contract adversarial_ --locked</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p reposix-confluence --test contract adversarial_ --locked` reports `3 passed; 0 failed`
    - `grep -cE "\.expect\(0\)" crates/reposix-confluence/tests/contract.rs` prints an integer >= 3
    - `grep -c "adversarial_" crates/reposix-confluence/tests/contract.rs` prints an integer >= 3 (function definitions + any `#[tokio::test]` attrs)
    - AUDIT-NOTES.md contains a line matching `ssrf: PASS` — verify with `grep -E "^ssrf: PASS$" .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md`
  </acceptance_criteria>
  <done>
    All three SSRF tests green; decoy-server pattern intact; outcome captured for AUDIT-NOTES.
  </done>
</task>

<task type="auto">
  <name>Task A3: Write 21-A-AUDIT-NOTES.md and commit</name>
  <read_first>
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
    - (results captured from tasks A1 and A2 in /tmp/21-A-ssrf.log and script output)
  </read_first>
  <action>
    Write `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md` with the following exact structure:

    ```markdown
    # Phase 21 Wave A — HARD-00 Audit Notes

    Audit date: <YYYY-MM-DD>
    Auditor: gsd-executor (Wave A)

    ## Results

    pre-push: PASS
    ssrf: PASS

    ## Evidence — credential pre-push hook

    - Script: scripts/hooks/test-pre-push.sh
    - Exit code: 0
    - Test cases green: <N>/<N>
    - Last-modified commits verified: f357c92, 5361fd5
    - Token-in-repo grep (excluding scripts/hooks, .planning, target, .git): no matches

    ## Evidence — SSRF regression tests

    - Command: cargo test -p reposix-confluence --test contract adversarial_ --locked
    - Tests green: 3/3
      - adversarial_links_base_does_not_trigger_outbound_call
      - adversarial_webui_link_does_not_trigger_outbound_call
      - adversarial_host_in_arbitrary_string_field_is_ignored
    - Decoy server pattern (.expect(0)) intact: <count> sites
    - Last-modified commit verified: ea5e548

    ## Gaps

    <If both items are PASS, this section reads exactly "None. HARD-00 closes.">
    <Otherwise: bulleted list of each failing assertion with reproduction command.>

    ## Follow-ups for waves B–E

    - Wave B may proceed with confidence that If-Match semantics in the sim are preserved.
    - Wave C may proceed with confidence that Confluence adversarial-field handling is regression-safe.
    - No new tasks spawned by this audit unless "Gaps" above is non-empty.
    ```

    Fill in `<N>`, `<count>`, and the date by using the actual outputs captured in tasks A1 and A2. If any of A1 / A2 produced a FAIL line, replace "PASS" with "FAIL" in the Results section and populate the Gaps section with the reproduction command + a pointer to the failure line.

    Then commit with: `git add .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md && git commit -m "docs(21-A): audit notes for HARD-00 (credential hook + SSRF tests)"`
  </action>
  <verify>
    <automated>test -f .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md && grep -qE "^(pre-push|ssrf): (PASS|FAIL)$" .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md && git log -1 --oneline -- .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md | grep -q "21-A"</automated>
  </verify>
  <acceptance_criteria>
    - File `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md` exists
    - File contains both `pre-push: PASS` and `ssrf: PASS` (or explicit FAIL lines with matching Gaps entries)
    - File contains a "Gaps" section reading either `None. HARD-00 closes.` or a bulleted list of reproducible failures
    - The file is committed to git (`git log -1 -- path/to/file` returns a commit touching it)
  </acceptance_criteria>
  <done>
    AUDIT-NOTES.md committed; downstream waves have a signed floor to build on.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| dev host → git repo | audit only reads; writes one doc file, no code |
| cargo test → sim (in-process) | SSRF tests already use wiremock; no real egress |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-A-01 | Information Disclosure | AUDIT-NOTES.md | mitigate | Notes must not contain real token strings — only fake-prefix (ATATT3) patterns used by the test harness. Verify by grep before commit. |
| T-21-A-02 | Tampering | AUDIT-NOTES.md | accept | Notes are committed to git; any tampering is visible in history. No signature needed for internal audit. |
| T-21-A-03 | Repudiation | audit itself | mitigate | The commands in the `action` blocks are reproducible; anyone can re-run them. Audit output is not self-attesting; re-running is the defence. |
</threat_model>

<verification>
- `bash scripts/hooks/test-pre-push.sh` exits 0
- `cargo test -p reposix-confluence --test contract adversarial_ --locked` reports 3 passed
- `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-AUDIT-NOTES.md` exists and is committed
- No literal real tokens anywhere in the repo
</verification>

<success_criteria>
HARD-00 is formally closed (or formally flagged with specific gaps). Waves B–E have an attested green floor.
</success_criteria>

<output>
After completion, create `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-A-SUMMARY.md` summarising: both audit items passed (or not), the AUDIT-NOTES location, and explicit confirmation that Wave B may begin.
</output>
