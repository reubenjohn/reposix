---
phase: 102-d2-self-safe-dark-factory-hardening
plan: overview
type: execute
wave: 1
depends_on: []
requirements: [D2-TAT-IDENTITY-HOOK-01, D2-LEAF-ISOLATION-01, D2-SHARED-CONFIG-GUARD-01]
files_modified:
  - .claude/hooks/leaf-isolation-guard.sh
  - .claude/settings.json
  - .githooks/pre-commit
  - scripts/install-hooks.sh
  - quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh
  - quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh
  - quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh
  - CLAUDE.md
  - .planning/ORCHESTRATION.md
autonomous: true

must_haves:
  truths:
    - "A tool-invoked commit under the fixture identity `t <t@t>` against the shared repo is REJECTED with a teaching message (proven by a triggered-hook transcript, not a code read)."
    - "A leaf-setup verb (`reposix init`/`attach`/`sync`/sim-seed) run with cwd inside the shared repo and NO same-invocation `cd /tmp` is BLOCKED before it runs — the shared repo is never touched."
    - "A `git config core.bare` / `user.email` write targeting the shared `.git/config` is BLOCKED pre-execution; the shared `.git/config` is byte-unchanged after the attempt."
    - "All three guards are fail-closed: ambiguous/undeterminable state → default-DENY, never default-allow."
    - "The guard implementation invokes `git worktree remove --force` NOWHERE (it is itself a corruption vector)."
    - "A legitimate planning commit under the real identity, and a correctly-`cd /tmp`-prefixed leaf setup, are NOT blocked (no false-positives)."
  artifacts:
    - path: ".claude/hooks/leaf-isolation-guard.sh"
      provides: "Combined PreToolUse Bash hook: three guard functions (fixture-identity, leaf-setup-location, shared-config-write)"
      min_lines: 60
    - path: ".githooks/pre-commit"
      provides: "Git-native defense-in-depth: rejects fixture-identity author/committer in the shared repo"
      contains: "GIT_AUTHOR_IDENT"
    - path: "quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh"
      provides: "Verifier driving the guard as a subprocess; emits transcript"
    - path: "quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh"
      provides: "Verifier proving leaf-setup block + /tmp-redirect allow + no-worktree-force"
    - path: "quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh"
      provides: "Verifier proving core.bare/user.email write block + config byte-unchanged"
  key_links:
    - from: ".claude/settings.json"
      to: ".claude/hooks/leaf-isolation-guard.sh"
      via: "PreToolUse Bash matcher entry"
      pattern: "leaf-isolation-guard"
    - from: "scripts/install-hooks.sh"
      to: ".githooks/pre-commit"
      via: "core.hooksPath .githooks (shared repo only)"
      pattern: "core.hooksPath"
    - from: "quality/gates/agent-ux/fleet-safety-*.sh"
      to: ".claude/hooks/leaf-isolation-guard.sh"
      via: "subprocess invocation with crafted JSON payload; asserts exit 2 + teaching stderr"
      pattern: "leaf-isolation-guard"
---

<objective>
Make the fleet's leaf-isolation safety substrate MECHANICALLY enforced and fail-closed,
replacing the current doctrine-only prose (ZERO mechanical enforcement today). The shared
`.git/config` was corrupted 4–5× last session — founding anchor `S-260707-pr-08`: a
sim-seed leaf that "forgot to cd" into `/tmp` committed under fixture identity `t <t@t>`
and flipped `core.bare=true`, breaking the shared checkout for every concurrent agent.

Purpose: this is the HARD SERIALIZING GATE (ROADMAP P102). No other phase in v0.14.0, and
no other autonomous fleet run project-wide, starts until this verdict is GREEN. The fleet
must be self-safe before it is scaled.

Output: three fail-closed guards, all mechanically enforced, all PROVEN by committed
triggered-hook transcripts (not code-read assertions); catalog rows minted NOT-VERIFIED
BEFORE implementation (catalog-first); fix-it-twice doc updates marking the prose hard-stop
superseded-by-mechanism.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/milestones/v0.14.0-phases/ROADMAP.md (Phase 102 section — SC1-7 authoritative)
@.planning/ORCHESTRATION.md (§ "Leaf isolation for reposix/sim/git test setup (HARD STOP)", § "Enforcement map")
@.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md (§ S-260707-pr-08 — founding incident + sketch (a)/(b)/(c))
@CLAUDE.md (§ "Leaf test setup runs in a throwaway `/tmp` clone")
@.claude/hooks/cargo-mutex.sh (fail-closed PreToolUse Bash precedent — MATCH its conventions: exit 2 = BLOCK, stderr names rule + constraint + recovery)
@.githooks/pre-commit (existing git-native hook to EXTEND for guard 1; core.hooksPath wiring already in scripts/install-hooks.sh)
@quality/gates/agent-ux/lib/transcript.sh (shared transcript-writer for kind:shell-subprocess proof artifacts)
@quality/catalogs/README.md (unified row schema — kind:shell-subprocess transcript contract, minted_at write-once, coverage_kind)

<interfaces>
<!-- Hook payload contract (Claude Code PreToolUse). Extract the command + cwd once,
     as cargo-mutex.sh does, then dispatch the three guard functions. -->

PreToolUse stdin JSON shape (as consumed by cargo-mutex.sh):
  { "tool_input": { "command": "<the bash string>", "cwd": "<optional cwd>" }, "cwd": "<session cwd>" }

Extraction precedent (cargo-mutex.sh line 6):
  cmd=$(printf '%s' "$payload" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("tool_input",{}).get("command",""))')
  # ALSO extract cwd: json.get("tool_input",{}).get("cwd") or json.get("cwd") ; fall back to $PWD.

Block contract (cargo-mutex.sh lines 14-16): echo teaching msg to stderr, `exit 2`.
Allow contract (line 17): emit hookSpecificOutput JSON permissionDecision:allow, `exit 0`.

Shared-repo detection: the guard's own canonical shared-repo root is
`${CLAUDE_PROJECT_DIR}` (settings.json already interpolates it). A command is
"leaf-setup in the shared tree" when the effective cwd resolves inside
`${CLAUDE_PROJECT_DIR}` AND the command string carries no same-invocation redirect to
`/tmp` (`cd /tmp/…`, `-C /tmp/…`, `--git-dir=/tmp…`, `--file /tmp…`).

Transcript writer (quality/gates/agent-ux/lib/transcript.sh): source it; it writes
argv + env_keys (NAMES only, no values) + cwd + exit_code + stdout/stderr to
quality/reports/transcripts/<row-slug>-<RFC3339>.txt (required by kind:shell-subprocess).
</interfaces>
</context>

<design_decisions>
Resolved by the planner (the ROADMAP deliberately left the isolation mechanism open):

**One combined hook, not three.** `.claude/hooks/leaf-isolation-guard.sh` — a single
PreToolUse Bash matcher (mirrors the cargo-mutex.sh entry shape) with three internal guard
functions, first-match-blocks. Rationale: the command-string + cwd parse is done ONCE;
one registration, one timeout, one teaching-message convention. Splitting into three
settings.json entries would triple the JSON parse per Bash call for no new signal.

**Guard 1 (fixture-identity reject) is belt-and-suspenders:**
  - Primary: the combined PreToolUse hook rejects a tool-invoked `git commit` /
    `git -c user.email=…` / `GIT_AUTHOR_EMAIL=… git …` carrying a fixture-identity token
    (`t@t`, name `t`, plus a small configurable fixture list) when the effective cwd is
    the shared tree. Catches the exact `S-260707-pr-08` command shape BEFORE it runs.
  - Defense-in-depth: `.githooks/pre-commit` (already installed to the SHARED repo only,
    via `core.hooksPath .githooks`; a `/tmp` clone does NOT inherit it) rejects any commit
    whose `git var GIT_AUTHOR_IDENT`/`GIT_COMMITTER_IDENT` matches the fixture list. This
    catches commits whose identity comes from already-set config rather than the command
    string — a path the command-string scan cannot see. Fixture commits are thus blocked
    in shared history but ALLOWED in a `/tmp` clone (where they belong).

**Guard 2 (leaf isolation) triggers on the SETUP VERBS, not on all git.** Trigger set:
  `reposix init`, `reposix attach`, `reposix sync`, sim-seed (POST to a sim `/issues`
  endpoint / `reposix-sim` seed invocation). When such a verb runs with effective cwd in
  the shared tree and NO same-invocation `/tmp` redirect → BLOCK. A pure planning
  `git add . && git commit` under the real identity is deliberately NOT in the trigger set
  (that is the legitimate path guards 1+3 already fence). This is the direct mechanical
  backstop for ORCHESTRATION.md § Leaf isolation.
  HARD CONSTRAINT: the guard MUST NOT invoke `git worktree remove --force` anywhere in
  setup/cleanup/recovery — it is itself a corruption vector (owner note + ROADMAP SC2).

**Guard 3 (shared-config write) blocks pre-execution.** Trigger: a `git config` WRITE
  (not `--get`/`--list`/`--get-all`) of key `core.bare` / `user.email` / `user.name`
  whose target is the shared `.git/config` (effective cwd in shared tree, no
  `--file /tmp…` / `--git-dir=/tmp…` / same-invocation `cd /tmp` redirect, not
  `--global`/`--system`). Because PreToolUse blocks BEFORE the tool runs, the write never
  lands → shared `.git/config` byte-unchanged. `--global`/`--system` are out of scope
  (they don't touch the shared repo's `.git/config`); `core.hooksPath` writes (used by
  install-hooks.sh) are explicitly NOT in the blocked-key set.

**Fail-closed everywhere:** if the effective cwd cannot be determined, a triggering verb
  is treated as shared-tree and BLOCKED (default-deny). Teaching-free error = defect: each
  block names the rule, the constraint, and the concrete recovery (cd into your `/tmp`
  clone under a real identity).

**Catalog dimension: agent-ux** (not a new `fleet-safety` dimension). Justification: a new
  dimension requires a schema migration (README line 25 enumerates the allowed dimensions),
  a new catalog file, new runner/verdict routing, and README enum extension — cost the
  charter says to avoid absent a strong reason. agent-ux already houses the dark-factory /
  fleet-safety-adjacent gates; the three rows slot in as `agent-ux/fleet-safety-*`.
</design_decisions>

<tasks>

<task type="auto">
  <name>Task 1 (Wave 1): Combined leaf-isolation guard hook + git-native fixture-identity backstop + wiring</name>
  <files>.claude/hooks/leaf-isolation-guard.sh, .claude/settings.json, .githooks/pre-commit, scripts/install-hooks.sh</files>
  <action>
Implement the single combined PreToolUse Bash hook `.claude/hooks/leaf-isolation-guard.sh`,
matching cargo-mutex.sh conventions (`set -eu`; read payload; extract `tool_input.command`
+ effective cwd via python3 json; exit 2 = BLOCK with teaching stderr; exit 0 = allow with
hookSpecificOutput JSON). Three internal guard functions, first-match-blocks:

  guard_fixture_identity  — BLOCK a `git commit`/`git -c user.email=…`/`GIT_AUTHOR_EMAIL=…`
    carrying a fixture-identity token (`t@t`, author name `t`, plus a FIXTURE_IDENTITIES
    list defined once at top of file) when effective cwd is inside ${CLAUDE_PROJECT_DIR}
    and no same-invocation `/tmp` redirect is present. (per D2-TAT-IDENTITY-HOOK-01)
  guard_leaf_setup_location — BLOCK a setup verb (`reposix init|attach|sync`, sim-seed)
    when effective cwd is inside ${CLAUDE_PROJECT_DIR} and the command carries NO
    same-invocation `/tmp` redirect (`cd /tmp/…`, `-C /tmp/…`, `--git-dir=/tmp…`).
    (per D2-LEAF-ISOLATION-01)
  guard_shared_config_write — BLOCK a `git config` WRITE of core.bare|user.email|user.name
    targeting the shared `.git/config` (cwd in shared tree, not --global/--system, no
    `--file /tmp…`/`--git-dir=/tmp…`/`cd /tmp` redirect). (per D2-SHARED-CONFIG-GUARD-01)

Fail-closed: undeterminable effective cwd → treat as shared → BLOCK any triggering verb.
Each BLOCK stderr MUST name: (rule) + (constraint, cite ORCHESTRATION.md § Leaf isolation)
+ (recovery: `cd /tmp/<clone> && <cmd>` under a real identity). NON-triggering commands
(planning commit under real identity, correctly-/tmp-prefixed setup) fall through to the
allow branch. The hook MUST NOT invoke `git worktree remove --force` anywhere.

Register in `.claude/settings.json`: add a PreToolUse `{ "matcher": "Bash", ... }` entry
invoking the hook (timeout 5), sibling to the cargo-mutex entry.

Extend `.githooks/pre-commit` (defense-in-depth, guard 1): after the existing fmt/runner
steps, reject when `git var GIT_AUTHOR_IDENT` or `GIT_COMMITTER_IDENT` matches the fixture
list; teaching message + non-zero exit. Keep the existing personal-global chain intact.
Confirm `scripts/install-hooks.sh` still points core.hooksPath at `.githooks` (no change
expected; verify the fixture-reject runs in the shared repo, not in a /tmp clone).
  </action>
  <verify>
    <automated>bash -n .claude/hooks/leaf-isolation-guard.sh && bash -n .githooks/pre-commit && python3 -c 'import json,sys; json.load(open(".claude/settings.json"))' && grep -q leaf-isolation-guard .claude/settings.json && grep -cv '^[[:space:]]*#' .claude/hooks/leaf-isolation-guard.sh | grep -q . && [ "$(grep -v '^[[:space:]]*#' .claude/hooks/leaf-isolation-guard.sh | grep -c 'worktree remove --force')" = 0 ]</automated>
  </verify>
  <done>Hook + git backstop + registration in place, valid syntax, valid JSON, zero `git worktree remove --force` occurrences (comment-filtered), hook registered in settings.json.</done>
</task>

<task type="auto">
  <name>Task 2 (Wave 2): Three verifier scripts + committed triggered-hook transcripts (proof, not claim)</name>
  <files>quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh, quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh, quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh</files>
  <action>
Write the three verifier scripts named in the catalog rows (agent-ux dim). Each DRIVES the
combined hook as a REAL subprocess with a crafted JSON payload (kind:shell-subprocess),
sources quality/gates/agent-ux/lib/transcript.sh, and emits a transcript to
quality/reports/transcripts/<row-slug>-<RFC3339>.txt (argv + env_keys NAMES-only + cwd +
exit_code + stdout/stderr). CRITICAL — none of these may mutate the shared repo: they feed
the hook a JSON payload and assert its exit code + stderr; the hook BLOCKS pre-execution so
the underlying git/reposix command never runs. Any real git write the verifier itself needs
(e.g. proving `.githooks/pre-commit` rejects a fixture commit) MUST run in a throwaway
`/tmp` clone created + `cd`-ed in the SAME invocation (leaf-isolation HARD-STOP applies to
the verifier too).

  fleet-safety-tat-identity-reject.sh — assert: payload `git -c user.email=t@t commit -m x`
    at cwd=shared → hook exit 2 + teaching stderr; AND in a /tmp clone with core.hooksPath
    set to the repo .githooks, a commit under `t <t@t>` is refused (pre-commit non-zero);
    AND a control payload under a real identity → allow (exit 0). No false-positive.
  fleet-safety-leaf-isolation-enforce.sh — assert: payload `reposix init sim::demo .` at
    cwd=shared, no cd → exit 2 + teaching stderr citing ORCHESTRATION.md; AND the same
    prefixed `cd /tmp/<clone> && …` → allow (exit 0); AND fail-closed (undeterminable cwd
    → block); AND grep proves zero `git worktree remove --force` in the hook (comment-filtered).
  fleet-safety-shared-config-write-guard.sh — assert: payload `git config core.bare true`
    (and `git config user.email t@t`) at cwd=shared → exit 2 + teaching stderr; capture
    sha256 of the real shared `.git/config` before and after → byte-unchanged; AND a
    `--file /tmp/<clone>/.git/config core.bare true` payload → allow.

Each script exits 0 only when ALL its asserts pass; writes its verifier artifact under
quality/reports/verifications/agent-ux/ and the transcript. Commit the transcripts.
  </action>
  <verify>
    <automated>bash quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh && bash quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh && bash quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh && ls quality/reports/transcripts/fleet-safety-*-*.txt</automated>
  </verify>
  <done>All three verifiers exit 0; three committed transcripts under quality/reports/transcripts/ each containing an `argv:` line + exit_code; shared `.git/config` sha256 unchanged across the guard-3 run.</done>
</task>

<task type="auto">
  <name>Task 3 (Wave 2): Fix-it-twice doc updates — mark prose hard-stop superseded-by-mechanism</name>
  <files>CLAUDE.md, .planning/ORCHESTRATION.md</files>
  <action>
Fix-it-twice meta-rule (same PR as the mechanism). Do NOT delete the historical prose —
mark it superseded-by-mechanism.
  - CLAUDE.md § "Leaf test setup runs in a throwaway `/tmp` clone": add a line noting the
    rule is now mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh` (three
    fail-closed guards) + the `.githooks/pre-commit` fixture-identity backstop; keep the
    prose hard-stop as the human-readable contract.
  - .planning/ORCHESTRATION.md § "Enforcement map": add a row
    `Leaf isolation (reposix/sim/git test setup) | .claude/hooks/leaf-isolation-guard.sh (exit 2) + .githooks/pre-commit | blocking hook`.
  - .planning/ORCHESTRATION.md § "Leaf isolation for reposix/sim/git test setup (HARD STOP)":
    append a note that the prior prose hard-stop is now backstopped mechanically (name the
    hook + the three guards + the agent-ux catalog rows), marked superseded-by-mechanism,
    not deleted.
DO NOT touch .planning/STATE.md or .planning/MANAGER-HANDOVER.md (concurrent writer).
  </action>
  <verify>
    <automated>grep -q leaf-isolation-guard CLAUDE.md && grep -q leaf-isolation-guard .planning/ORCHESTRATION.md && grep -q 'Leaf isolation' .planning/ORCHESTRATION.md</automated>
  </verify>
  <done>Both docs reference the mechanism; Enforcement map has the new blocking-hook row; prose hard-stop retained + marked superseded-by-mechanism.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| leaf agent → shared repo git state | a dispatched leaf's Bash command crosses into the coordinator's shared `.git/config` + object store + working tree (worktrees are NOT isolated; cwd resets between Bash calls) |
| fixture identity → real history/origin | the throwaway `t <t@t>` test identity crossing into shared commits / `git push` to origin |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-102-01 | Spoofing | commit identity in shared repo | mitigate | guard 1: PreToolUse fixture-token reject + `.githooks/pre-commit` `git var` ident check — fixture identity cannot reach shared history/origin |
| T-102-02 | Tampering | shared `.git/config` (`core.bare`, `user.email`) | mitigate | guard 3: PreToolUse blocks the write pre-execution; shared config byte-unchanged (the exact `S-260707-pr-08` corruption) |
| T-102-03 | Tampering | shared working tree / object store via forgot-to-cd setup | mitigate | guard 2: leaf-setup verb blocked when cwd is shared and no same-invocation `/tmp` redirect; fail-closed on undeterminable cwd |
| T-102-04 | Elevation of Privilege | guard's own recovery path | mitigate | HARD: guard never invokes `git worktree remove --force` (itself a corruption vector); verified by comment-filtered grep == 0 |
| T-102-05 | Denial of Service | false-positive blocking legit planning commits | accept | guard 2 triggers on setup verbs only, not bare `git commit`; control-identity/`--no-verify` paths documented; verifier asserts no false-positive |
</threat_model>

<verification>
Phase-level (proven, not code-read — ROADMAP SC1-4):
1. Fixture-identity commit against shared repo observed to FAIL with a clear rejection (transcript).
2. Forgot-to-cd `reposix init` observed BLOCKED before touching the shared repo (transcript).
3. `git config core.bare`/`user.email` write to shared `.git/config` BLOCKED; config byte-unchanged (sha256 before==after).
4. All three guards fail-closed (default-deny on ambiguous cwd); guard invokes `git worktree remove --force` nowhere.
5. CLAUDE.md + ORCHESTRATION.md revised same-PR (fix-it-twice); prose marked superseded-by-mechanism, not deleted.
6. Three agent-ux catalog rows conform to the unified schema (minted NOT-VERIFIED before implementation).
</verification>

<success_criteria>
Reproduced from ROADMAP § Phase 102 SC1-7 for catalog-row traceability. Phase closes only
on unbiased verifier GREEN at quality/reports/verdicts/p102/VERDICT.md, AFTER
`git push origin main`. This is the HARD SERIALIZING GATE — no other v0.14.0 phase and no
autonomous fleet run project-wide starts until this verdict is GREEN.
</success_criteria>

<verdict_location>
quality/reports/verdicts/p102/VERDICT.md
</verdict_location>

<output>
After completion, create `.planning/phases/102-d2-self-safe-dark-factory-hardening/102-SUMMARY.md`
</output>
