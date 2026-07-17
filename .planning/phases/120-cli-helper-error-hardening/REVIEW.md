---
phase: 120-cli-helper-error-hardening
reviewed: 2026-07-17T00:00:00Z
depth: deep
files_reviewed: 11
files_reviewed_list:
  - crates/reposix-core/src/errmsg.rs
  - crates/reposix-remote/src/backend_dispatch.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-remote/src/bus_url.rs
  - crates/reposix-remote/src/stateless_connect.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/tests/errors_teach_recovery.rs
  - crates/reposix-cli/src/errors.rs
  - crates/reposix-cli/src/sync.rs
  - crates/reposix-cli/src/worktree_helpers.rs
  - crates/reposix-cli/src/init.rs
findings:
  critical: 0
  warning: 3
  info: 1
  total: 4
status: issues_found
---

# Phase 120: Code Review Report (CLI + helper error hardening)

**Reviewed:** 2026-07-17
**Depth:** deep (cross-file credential-flow trace)
**Overall verdict:** CHANGES-REQUIRED

## Summary

The two flagged credential-leak fixes are **SOUND** and genuinely guarded. The
`errmsg::teach` primitive is well-designed (FLAG-1 empty-limb suppression, FLAG-2
forward-compat code slot, all byte-pinned). BUT adversarial grep of the touched
helper + CLI surface found **three additional unredacted credential/URL echo
paths** the wave missed — one of which writes a token into an append-only audit
row (OP-3 violation). Mandatory item #3 ("no OTHER path leaks") therefore FAILS.

## MANDATORY item verdicts

**1. Both credential-leak fixes are SOUND — CONFIRMED.**
- (a) W4 `classify_origin` / `parse_remote_url`: every echo arm now routes the
  URL/origin through `redact_userinfo(...)` — `backend_dispatch.rs:115,122,177,
  182,190`. Before: `anyhow!("atlassian origin `{origin}` requires …")`. After:
  `anyhow!("… `{}` …", redact_userinfo(origin))`. `redact_userinfo`
  (`backend_dispatch.rs:468`) correctly scans EVERY `://…@` authority (handles the
  `reposix::` outer scheme that defeats `Url::parse`) and rewrites `user:secret@`
  → `<redacted>@`. Sound.
- (b) W5 `bus_handler::precheck_mirror_drift` (`bus_handler.rs:452,457`): the
  `git ls-remote` failure now redacts BOTH the URL and git's own stderr before the
  error. Before: `anyhow!("git ls-remote {mirror_url} failed: {}", stderr)`. After:
  `redact_userinfo(mirror_url)` + `redact_userinfo(stderr.trim())`, and the recovery
  cites the creds-free remote NAME. Sound.

**2. Regression tests ASSERT non-leakage — CONFIRMED (no red flags).**
- `bus_handler.rs:614 precheck_mirror_drift_redacts_credentials_and_teaches_on_ls_remote_failure`
  drives the REAL `git ls-remote` subprocess at a credentialed loopback URL and
  asserts `!msg.contains("SECRETTOKEN123")`, `!msg.contains("x-access-token")`,
  `msg.contains("<redacted>@127.0.0.1:1")`, plus `Fix:`/`Recovery:`/`git ls-remote mirror`.
- `tests/errors_teach_recovery.rs:133 malformed_credentialed_bus_url_does_not_leak_userinfo`
  asserts `!stderr.contains("ghp_SUPERSECRET")`, `!stderr.contains("x-access-token")`,
  `<redacted>@evil.example.com`.
- `backend_dispatch.rs:886/900` unit-test `redact_userinfo` directly with negative
  assertions. Every test's body matches its leak-prevention NAME. `test-name-honesty:`
  and `teach-exempt:` annotations across the diff are accurate.

**3. No OTHER error path echoes a credential/origin — FAILS.** Three unredacted
paths found (WR-01, WR-02, WR-03 below).

**4. SC1/SC2 3-part UX bar — MET (minor nits).** `teach` renders headline + `Fix:`
+ optional `Alternative:` + indented `Recovery:`; empty limbs suppressed and pinned
by `errmsg.rs` tests. Sampled CLI (`errors.rs`, `init.rs:394`, `sync.rs`) and helper
(`main.rs`, `bus_url.rs`, `stateless_connect.rs`) sites all carry runnable recovery
lines. Nit: several recovery commands embed `<name>`/`<backend>` placeholders
(`git remote set-url <name> …`) — git-help convention, but not verbatim-runnable
(IN-01).

**5. Bugs / panics / dead code — clean.** No unwrap-on-tainted (`bus_handler`
`.unwrap()`s are len-guarded by their `match`; `parse_want_oid`/`parse_command_keyword`
return `None` on bad bytes). FLAG-2 `code` field is intentional forward-compat, kept
live via derived `Debug`, and byte-pinned by `code_slot_renders_nothing`.

## Warnings

### WR-01: `push_mirror` failure echoes git-push stderr (token-bearing) to stderr AND an append-only audit row

**File:** `crates/reposix-remote/src/bus_handler.rs:341` and `:344-348` (source of leak: `:569-575`)
**Issue:** On mirror-push failure, `stderr_tail` is built from raw `git push
<mirror_remote_name> main` stderr with NO redaction (`push_mirror`, lines 569-575),
then (1) written verbatim into the OP-3 audit row via
`log_helper_push_partial_fail_mirror_lag(&oid_hex, exit_code, &stderr_tail)` (line
341), and (2) echoed to stderr via `crate::diag(… tail={stderr_tail})` (line 344).
When the LOCAL mirror remote is a token-in-URL clone — a case the file's own
`resolve_mirror_remote_name` doc explicitly acknowledges ("the LOCAL remote's own
URL may still carry `user:token@`") — git's `unable to access
'https://TOKEN@github.com/…'` / `Authentication failed for 'https://TOKEN@github.com/…'`
message carries the token in the username position, which modern git does NOT
redact. This is the SAME leak class the T-120-02 fix repaired in the sibling
`precheck_mirror_drift` (which redacts git stderr precisely because "an older git
could echo it") — but was left unredacted here, and additionally violates OP-3
("credentials must never reach an audit row"). No regression test guards this arm.
Directly contradicts the W5 "helper surface complete (SC2)" claim.
**Fix:**
```rust
// in the MirrorResult::Failed arm, before the audit + diag:
let safe_tail = crate::backend_dispatch::redact_userinfo(&stderr_tail);
// ... log_helper_push_partial_fail_mirror_lag(&oid_hex, exit_code, &safe_tail);
// ... crate::diag(&format!("… tail={safe_tail}"));
```
Prefer redacting inside `push_mirror` when constructing `stderr_tail` so the
un-redacted string never escapes the function. Add a regression test mirroring
`precheck_mirror_drift_redacts_credentials_and_teaches_on_ls_remote_failure`.

### WR-02: `reposix sync` echoes the raw remote URL (incl. `?mirror=` creds) on parse failure

**File:** `crates/reposix-cli/src/sync.rs:118`
**Issue:** `teach(&format!("the reposix remote URL in this tree could not be parsed:
`{url}`. …"))` interpolates the RAW `url` from `resolve_reposix_remote_url` — the
full `remote.*.url` INCLUDING any `?mirror=https://user:token@…` component (the code
echoes `url`, not the `sot` it stripped). A hand-edited or token-in-URL-cloned tree
(exactly the "may have been hand-edited" case this error exists to diagnose) leaks
the credential to stderr. P120 W2 REWROTE this very line and kept the raw echo, even
though the sibling helper code redacts and the wave's whole purpose is credential-leak
hardening. Untested.
**Fix:** `redact_userinfo(&url)` (reachable via
`reposix_remote::backend_dispatch::redact_userinfo`, already imported in this file):
```rust
&format!("… could not be parsed: `{}`.\n(underlying: {e:#})", backend_dispatch::redact_userinfo(&url))
```

### WR-03: `cache_path_from_worktree` echoes the raw remote URL on parse failure

**File:** `crates/reposix-cli/src/worktree_helpers.rs:211`
**Issue:** `.with_context(|| format!("parse reposix remote url={url}"))` echoes the
raw config URL (same source + same leak class as WR-02). Pre-existing, but in a
P120-touched file and symmetric to the exact leaks the wave fixed on the helper side
(OD-3 ownership: the wave owns this surface). Feeds `history`/`tokens`/`cost`/`gc`.
**Fix:** wrap in `redact_userinfo(&url)` (add the import) — ideally reuse `teach` here
too for consistency with the adjacent no-remote arm.

## Info

### IN-01: Placeholder tokens in some "copy-paste recovery" lines are not verbatim-runnable

**File:** `crates/reposix-remote/src/bus_url.rs:74`; `crates/reposix-cli/src/errors.rs:44,71`
**Issue:** Recovery lines like `git remote set-url <name> '…'` and
`reposix attach <backend>::<project> .` contain `<name>`/`<backend>` placeholders —
standard git-help convention but not literally paste-and-run. The SC1/SC2 bar says
"copy-paste recovery command."
**Fix:** Acceptable as-is (matches `git`'s own convention); if tightening, pair each
placeholder line with one fully-concrete example (as `errors.rs:44` already does with
`reposix init sim::demo /tmp/demo`).

## NOTICED (owner deliverable)

- **The audit-row credential leak (WR-01) is the most serious miss** — a token can
  land in the append-only `helper_push_partial_fail_mirror_lag` table, which OP-3
  forbids, and it survives on modern git via the token-in-username form. The wave
  hardened its sibling `precheck_mirror_drift` but not `push_mirror`, so the "helper
  complete" claim is over-stated.
- The W4 `classify_origin` fix is sound, but its integration regression
  (`malformed_credentialed_bus_url_does_not_leak_userinfo`) drives the generic
  "unrecognised backend" arm (`evil.example.com`), NOT the atlassian-specific
  `Some(other)`/`None` arms it was framed around; those redact identically but have no
  dedicated credentialed test. `redact_userinfo` is unit-tested, so the fix holds — a
  coverage nit, not a leak.
- `env_example` returns safe placeholders (never real secrets) — good.
- `sync.rs` echoes `{url}` while the very next line strips it to `sot` — a reader
  could reasonably expect the stripped form; the raw echo is both a leak and mildly
  confusing.

---

_Reviewed: 2026-07-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
