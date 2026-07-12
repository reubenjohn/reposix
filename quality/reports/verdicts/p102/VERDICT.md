# P102 Phase-Close Verdict — v0.14.0 D2 self-safe dark-factory hardening (fleet-safety guards)

**Overall: GREEN**

- **Graded HEAD:** `455d22d` (`455d22db6228eb2673d45d3aa0358523beb6c012`) — local HEAD == `origin/main` (push-before-verifier cadence satisfied). Verdict commit lands on top.
- **Verifier:** unbiased phase-close subagent (did NOT implement P102). Graded goal-backward against reality — drove the PreToolUse hook as a real subprocess with crafted JSON payloads and ran the extended evasion probe. SUMMARY/commit claims treated as hypotheses.
- **Cargo discipline:** ZERO cargo invocations (the canonical `cargo run -- init` string was fed to the hook as a payload, never executed).
- **Leaf isolation (applied to the verifier):** every real git write ran inside a `/tmp` clone cd-ed in the same invocation (tat Case 2); the shared repo git state was never mutated. Hook payloads are stdin-only string inspection.

---

## Push cadence + catalog-first ordering — **PASS**

- `git rev-parse origin/main` == `455d22d` == HEAD; `git merge-base --is-ancestor 455d22d origin/main` → YES. P102 commits are on origin/main.
- Catalog-first held: `735acb1` (plan + catalog-first GREEN-contract rows, `2026-07-11 22:45`) **predates** impl commits `bf88470` (`23:03`) and `39a8500` (`23:25`). Rows carry `minted_at: 2026-07-12T05:40:57Z`; graded from asserts that predate the code.

---

## Row 1 — `agent-ux/fleet-safety-tat-identity-reject` (P0) — **GREEN**

Verifier `fleet-safety-tat-identity-reject.sh` → **exit 0**, ALL ASSERTS PASSED. Fresh transcript `fleet-safety-tat-identity-reject-2026-07-12T06-30-18Z.txt` regenerated this session.

Deciding live evidence:
- `git -c user.email=t@t commit -m x` @ shared cwd → **hook_exit=2** (BLOCK); stderr names the rule (fixture identity `t@t`), the WHY (must never reach shared history/origin, cites S-260707-pr-08), and RECOVERY (commit from a `/tmp` clone under a real identity). ✓
- `.githooks/pre-commit` backstop in a throwaway `/tmp` clone → fixture commit **rc=1** (rejected: "pre-commit: BLOCKED — commit identity carries the throwaway fixture token '<t@t>'"), real-identity commit **rc=0** (allowed). ✓
- Control `git -c user.email=dev@example.com commit` @ shared → **exit 0** (no false-positive). ✓
- Quoting tolerance: `'t@t'` / `"t@t"` → **exit 2**; `scott@things.io` (contains `t@t` substring) → **exit 0** (delimiter-bounded). ✓

## Row 2 — `agent-ux/fleet-safety-leaf-isolation-enforce` (P0) — **GREEN**

Verifier → **exit 0**, ALL ASSERTS PASSED. Transcript `...-2026-07-12T06-30-18Z.txt`.

Deciding live evidence:
- `reposix init sim::demo .` @ shared, no cd → **exit 2**; stderr cites ORCHESTRATION.md § Leaf isolation + the same-invocation `/tmp` rule. ✓
- **Canonical-form coverage (the loop-back fix): `cargo run -p reposix-cli -- init sim::demo .` @ shared → exit 2 (BLOCK).** Also blocked: `/usr/bin/reposix init`, `./target/debug/reposix init`, `cargo run … -- attach`, bare `reposix attach`, `reposix sync --reconcile`. ✓
- `cd /tmp/<clone> && reposix init …` → **exit 0** (redirect sanctioned — not a blanket ban). `cd /tmp && cargo run -- init` → exit 0. ✓
- Fail-closed: cwd omitted → exit 2; cd-back (`cd /tmp/x && cd <shared>`) → exit 2; `/tmp/../<shared>` traversal → exit 2; unparseable non-empty payload → exit 2. ✓
- `git worktree remove --force` occurrences in the hook (comment-filtered) = **0** — guard not built on the corruption vector. ✓

## Row 3 — `agent-ux/fleet-safety-shared-config-write-guard` (P0) — **GREEN**

Verifier → **exit 0**, ALL ASSERTS PASSED. Transcript `...-2026-07-12T06-30-19Z.txt`.

Deciding live evidence:
- `git config core.bare true` and `git config user.email t@t` @ shared → **exit 2** (BLOCK); stderr names core.bare/user.email + recovery. ✓
- **Shared `.git/config` sha256 BEFORE == AFTER = `e16c5b5d…7bf460` — byte-unchanged** (PreToolUse blocks pre-execution, write never ran). ✓
- `git config --file /tmp/<clone>/.git/config core.bare true` → exit 0; `-f /tmp/…` short flag → exit 0; `git config --get core.bare` READ → exit 0 (no over-block). cd-back write → exit 2. ✓

---

## Extended evasion probe — **32/32 PASS, 0 FAIL**

`fleet-safety-evasion-probe.sh` → exit 0. All BAD vectors (quoting, env-prefix, cd-back, traversal, canonical binary/cargo forms, symlink-to-shared cwd, `--local`, unparseable/truncated/non-object JSON) BLOCK rc=2; all GOOD vectors (`/tmp` redirects, `-f`/`--file` /tmp, reads, genuine throwaway /tmp path, empty/`{}` payload) ALLOW rc=0. No guard over-blocks legit /tmp work.

---

## NOTICED

1. **Transcript `argv:` names the wrapper fn, not the real binary (minor honesty).** All three `kind: shell-subprocess` transcripts record `argv: scenario` (the `write_transcript_and_artifact` helper's function arg) rather than the actual `bash .claude/hooks/leaf-isolation-guard.sh` subprocess invocations. The STDOUT block shows every real hook invocation + its exit code, so the proof is genuine and not fabricated, and these rows are `transport_claim: false` (not held to the real-binary argv rule for transport rows). Still, a strict read of the PROTOCOL shell-subprocess contract wants `argv:` to name the driven binary. Non-blocking; suggest the transcript helper record the inner `bash $HOOK` argv (or the driven command) for these driver-style gates. Worth a GOOD-TO-HAVE.
2. **Artifact JSON `asserts_passed`/`asserts_failed` are empty arrays.** The per-row `.json` records only `exit_code` + `transcript_path`; the assert detail lives in the transcript (graded there per the shell-subprocess contract). If these rows are ever run through `run.py --persist` with the F-K4b `asserts_congruent` gate armed (they carry `minted_at`), an empty `asserts_passed` could false-demote PASS→FAIL. This session's grade is transcript-based and correct; flag for the runner-path interaction — verify a `--persist` cadence run of these rows doesn't trip congruence before relying on the runner to re-mint them.
3. **Guard A fixture identity is a hardcoded `t@t` allowlist-of-one.** A different throwaway identity (not `t@t`) evades guard A at the PreToolUse layer; the `.githooks/pre-commit` backstop is likewise keyed on `<t@t>`. Documented honestly in both headers as a configurable alternation, and the founding incident was specifically `t@t`. Acceptable for P102 scope; the extension point is clearly marked.
4. **Coverage boundary is documented honestly** (hook header lines 32–37): the PreToolUse hook only fires on the Claude Code Bash *tool*; subprocess/script-spawned writes bypass it, with only the pre-commit COMMIT backstop catching the fixture-identity case (not `reposix init`/`git config` non-commit writes). A binary-side backstop for guards B/C is filed GOOD-TO-HAVE, not built here. No overclaim.

---

## Verdict

All three P0 fleet-safety rows **GREEN** against live grade-time regeneration. Canonical `cargo run -- init` blocks; shared `.git/config` byte-unchanged; push cadence and catalog-first ordering both hold. Rows minted `PASS` @ `last_verified: 2026-07-12T06:31:26Z`. No blocker; noticing items are non-blocking follow-ups.

_Verified: 2026-07-12T06:31:26Z — unbiased phase-close verifier (Claude)._
