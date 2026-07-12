# v0.13.0 GOOD-TO-HAVES — Part 8 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## 2026-07-07 | ~180-row doc-alignment backlog likely harbors more Haiku-backfill false-BONDS + latent TEST_DRIFT rows — needs systematic re-grade | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Wave F1b + C2-f handover) | severity: MEDIUM

**What:** The ~180-row doc-alignment backlog (flagged elsewhere in this file's
"backlog's size means the dimension's headline ratios are not yet trustworthy signal"
entry) likely contains more Haiku-backfill false-BONDS of the shape Wave F1b already
corrected once, plus latent `STALE_TEST_DRIFT` rows on the two permanently-yellow
benchmark claims (`token-89-percent`, `latency-8ms`) that haven't been systematically
re-checked since the backlog accumulated.

**Acceptance:** a systematic re-grade pass over the ~180-row backlog in v0.14.0 —
re-verify each row's citation still resolves to content that actually supports the bound
claim (not just that the file/line exists), and specifically check the two benchmark-claim
rows for drift against current `docs/benchmarks/*.md` values.

**Why deferred:** M-sized systematic audit work, not a mechanical filing action; needs a
dedicated docs-alignment-touching phase per this project's own routing convention.

**Default disposition:** MEDIUM — real trust-in-signal risk for the docs-alignment
dimension's headline ratios, but no immediate correctness hazard. Target: v0.14.0
scoping session.

**STATUS:** OPEN

## 2026-07-07 | Pre-commit size soft-warnings: `crates/reposix-cli/src/main.rs` and `quality/gates/agent-ux/zero-shot-onboarding.sh` both over budget | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** `crates/reposix-cli/src/main.rs` is 21279 chars (> 20000 budget) and
`quality/gates/agent-ux/zero-shot-onboarding.sh` is 10572 chars (> 10000 budget) — both
surfaced as pre-commit soft-warnings (non-blocking) during the v0.13.1 session.

**Acceptance:** split each file along its natural seams once either keeps growing
further past its budget (e.g. `main.rs` by subcommand-dispatch group;
`zero-shot-onboarding.sh` by scenario-phase into a sibling script or sourced helper).

**Why deferred:** soft-warning only (non-blocking), real split work not a mechanical
filing action; joins the existing consolidated file-size-overages entry
(GOOD-TO-HAVES-15) as a candidate for the same pre-2026-08-08 waiver-renewal /
structure-hygiene pass.

**Default disposition:** LOW — soft warning only, no blocking hazard today. Target:
fold into GOOD-TO-HAVES-15's consolidated split pass or the next structure-touching
phase.

**STATUS:** OPEN

## 2026-07-07 | Cosmetic front-door UX: stray `builtin seed loaded` INFO line prints before `reposix sim`'s clean banner | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** A stray `INFO ... builtin seed loaded inserted=6` tracing line prints BEFORE
the sim's clean banner on `reposix sim` — the documented happy path a first-time user
follows. It's not wrong (seeding did happen), but it clutters the first thing a new user
sees before the intended clean banner.

**Acceptance:** demote the line to debug-level tracing (so it doesn't print at the
default log level), or fold its content into the documented banner block so the
front-door output stays clean and single-purpose.

**Why deferred:** cosmetic-only, `crates/reposix-sim` source edit — out of this lane's
mechanical-filing envelope (no code edits).

**Default disposition:** LOW — cosmetic only, no functional hazard. Target: the next
`reposix-sim`-touching phase or a front-door-polish pass.

**STATUS:** OPEN

## 2026-07-07 | Shared push-test helper that always injects a per-test `REPOSIX_CACHE_DIR` tempdir | discovered-by: p94/protocol.rs CRLF-flake fix executor | severity: LOW

**What:** `resolve_cache_path` defaults to a global XDG dir
(`crates/reposix-cache/src/path.rs:22`) with no test-scoping, so push tests that forget to
pin `REPOSIX_CACHE_DIR` share a persistent `~/.cache/reposix/...` cache and
cross-contaminate. This exact footgun caused the p94 CI flake (fixed in 7f17cee by pinning
tempdirs on two `protocol.rs` tests). `crates/reposix-remote/tests/common.rs` already exists
— add a helper there that always injects a per-test tempdir so the next push test can't
forget. Warning comments already exist at `exit_codes.rs:109` and `push_conflict.rs:166`.

**Acceptance:** add a shared helper (e.g. in `crates/reposix-remote/tests/common.rs`) that
allocates a per-test tempdir and sets `REPOSIX_CACHE_DIR` to it, so no push test can silently
share the global XDG cache; retrofit the existing warning-commented call sites
(`exit_codes.rs:109`, `push_conflict.rs:166`) onto it.

**Why deferred:** ~30min test-ergonomics change touching `reposix-remote` test code (cargo)
— out of the mechanical-filing envelope; not blocking (the two flaky tests are already
pinned by 7f17cee).

**Default disposition:** LOW — test-ergonomics guardrail (~30min). Target: the next
`reposix-remote`-test-touching phase or a v0.14.0 test-hardening pass.

**STATUS:** OPEN
