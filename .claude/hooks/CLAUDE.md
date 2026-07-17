# .claude/hooks/CLAUDE.md — hook authoring guide

Scoped rules for anything under `.claude/hooks/`. Extends root `CLAUDE.md`.

## The one rule: be silent on the happy path

A hook fires on **every** matching tool call, and whatever it prints to stdout as
`hookSpecificOutput.additionalContext` is injected into the model's context — then
**re-billed on every subsequent turn** (prompt cache). An `additionalContext` line you
emit indiscriminately is a **session-wide token tax that compounds**, not a one-time cost.

> Measured (session `bf1e8875`, 2026-07-16 forensics): boot 48k + growth 104k = 152k
> context. Hook injections were ~10k of the growth — most of it avoidable **no-op
> restatement** (e.g. `leaf-isolation-guard.sh` emitting the same ~45-token "OK + reminder"
> on all ~34 Bash calls ≈ 1.5k tok/session of pure repetition).

So the default is: **emit nothing unless you have something the model doesn't already
know and needs right now.**

## Rules for writing a hook

1. **Silent on the no-op / positive path.** A guard's value is the **block**. On
   allow/pass, return allow (or `exit 0`) with **no `additionalContext`**. Do not restate a
   rule that already lives in a `CLAUDE.md` — the model has it; repeating it every call
   teaches nothing and costs tokens forever.

2. **Speak only when actionable.** Emit `additionalContext` only for genuinely-new,
   right-now-relevant state: a block (see 3), or a threshold crossing (see 4). "Everything
   is fine" is not actionable — it is noise.

3. **Blocks teach on stderr + `exit 2`.** The block message is where a guard earns its
   keep. Give it Rust-compiler-grade UX: the **rule**, the **why**, and a copy-paste
   **recovery** command. Blocks go to stderr with `exit 2`, not to `additionalContext`.

4. **Threshold/throttle any periodic signal.** If a hook must surface recurring state
   (context usage, etc.), gate it on a meaningful delta, not per-call. Exemplar:
   `context-ticker.js` emits only on ≥20k-token drift. For per-agent reminders, route
   through `throttle.sh` (exemplars: `dispatch-doctrine.sh`, `post-dispatch-relay.sh`).

5. **Match precisely — no naive substrings.** Gate on the real invocation, not a
   `contains()` on the command text. `cargo-mutex.sh` fired `"cargo OK"` on a Python
   heredoc that merely *mentioned* `cargo nextest` — a false positive that is both noise
   and a latent correctness smell.

6. **Do the token math.** `~T tokens × N calls/session × every-turn re-bill` is the real
   cost. If T·N is non-trivial and the content is static, it does not belong on the
   positive path.

## Signal vs noise — current inventory

**Good (already actionable-only) — copy these patterns:**
- `stop-uncommitted.sh` — silent unless the tree is dirty/ahead.
- `context-ticker.js` — emits only on ≥20k-token context drift.
- `dispatch-doctrine.sh`, `post-dispatch-relay.sh` — throttled to once per agent / 5 min.
- `session-start-brief.sh`, `precompact-persist.sh` — large but fire once per session /
  per compaction, not per tool call.

**Known-noisy (tracked by `GOOD-TO-HAVES-17`, to be silenced on the allow path):**
- `leaf-isolation-guard.sh` — `emit_allow()` restates the same reminder on **every** Bash
  call. Keep the four block paths (stderr + `exit 2`); drop the allow-path
  `additionalContext`.
- `cargo-mutex.sh` — emits `"cargo OK"` on any command whose text contains
  `cargo `/`rustc `/`cross `. Keep the block (concurrent-build guard); drop the allow-path
  `additionalContext`.

## cargo-mutex.sh: pgrep -x vs -f (comm vs argv match)

`cargo-mutex.sh` gates concurrent builds by checking whether a cargo/rustc build is
already running machine-wide. It matches on the process **executable name** (`comm`, via
`pgrep -x`), never the full argv. A prior version used `pgrep -f 'cargo (…)|rustc '` —
`-f` matches the FULL command-line TEXT of any process, so a shell wait-loop, an editor,
or a `pgrep`/`grep`/`rg` invocation whose argv merely *mentioned* `cargo build` or
`rustc …/target/debug` false-matched. The hook then `exit 2`'d and blocked every
subsequent cargo/rustc Bash call machine-wide until that unrelated process died — and the
failure mode is self-amplifying: a session's own `pgrep -f rustc` diagnostic (run to
investigate the block) becomes the next false match. This deadlocked a live session for
~180k tokens (full incident: `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`,
"cargo-mutex.sh gated on argv text, not process identity").

**Fix:** `pgrep -x 'cargo|cargo-nextest|rustc|cross|clippy-driver'` — exact whole-`comm`
match. A shell/editor/grep's `comm` is `bash`/`zsh`/`nvim`/`pgrep`, never a build-tool
name, regardless of what its argv contains — structurally cannot false-match. This does
NOT weaken the OOM protection (Hard rule 1, `crates/CLAUDE.md` § Build memory budget — VM
OOM-crashed 3x from parallel workspace builds): the comm alternation covers every
build-phase process a genuine second concurrent build would show (`cargo` for
check/build/test/run, `cargo` again for `cargo clippy` which execs `clippy-driver`,
`cargo-nextest`, `rustc`, `cross`), and `comm` cannot be spoofed without spawning a real
build process, so no decoy slips a second build past the gate. Regression coverage:
`.claude/hooks/tests/cargo-mutex-no-false-match.sh` (spawns a decoy `bash` whose argv
contains "cargo build" and asserts the hook does NOT block on it).
