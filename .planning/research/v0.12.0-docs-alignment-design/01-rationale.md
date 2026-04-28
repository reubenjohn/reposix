# 01 — Rationale: why docs-alignment exists

## The discovered miss

After v0.12.0's eight phases shipped, the owner reported that `reposix init <confluence>::<space>` no longer materializes the page-tree symlinks the docs describe. The implementation that built those symlinks may still be present in the codebase but is no longer reached from the init path. Crucially, **no test failed** — because no test asserted the shape in the first place.

This is the canonical "legacy promise lost in a pivot" failure mode. v0.9.0 rewrote the world from FUSE to git-native. New tests were written from the implementer's lens ("the new init runs cleanly"), not from the user-facing-surface lens ("Confluence page hierarchy is browsable as nested directories"). The promise lived in `docs/reference/confluence.md` (and possibly archived REQUIREMENTS.md) but had no machine-checkable binding to a test.

The v0.12.0 quality-gates framework grades **new commitments** — every new gate has a catalog row, every catalog row points at a verifier, every verifier produces an artifact. Legacy promises from v0.6, v0.7, v0.8 were never lifted into rows. They became folklore, then they evaporated.

## Why not "just write more tests"

Adding shape assertions to the existing `dark_factory_real_*` tests would catch the Confluence regression specifically, but it doesn't address the structural cause:

- **Tests are derived from the implementation.** When the implementer is the test author, they test what they just typed. There's no adversarial "prove this still does X" pass.
- **No "user story freshness" check.** REQUIREMENTS.md and prose docs accumulate promises across milestones. After a milestone closes, those promises are never re-validated against the current implementation.
- **Architecture pivots silently delete tests.** When v0.9.0 deleted the FUSE test suite, the prose claims about FUSE-shaped behavior remained. Nothing forced a deliberate decision per claim.
- **Tests grow bottom-up (unit) more than top-down (user-facing).** The dark-factory tests are the closest thing to acceptance — but they assert agent UX (cat/grep/edit/commit/push), not workspace shape.

## The principle

**Docs are the spec. Tests are derived from the docs, not from the implementation.** A subagent extracts behavioral claims from the prose with file:line citations. Each claim becomes a catalog row. The row binds a claim to a test. Pre-push fails if a claim has no test or if the cited prose moved without re-grading. CI never decides whether a test passed via LLM — that's deterministic. LLM is only used to extract claims and grade alignment between claim and test.

This is the inverse of code-coverage. Code-coverage asks: are tests reaching the code? Doc-alignment asks: are tests asserting the promises?

## What we explicitly chose against (alternatives considered)

- **Only golden init-shape snapshots per backend.** Useful, narrowly catches the Confluence symlink case, but doesn't generalize to prose claims that aren't shape-shaped (e.g. "outbound HTTP is allowlist-gated").
- **Reachability-as-gate / dead-code detector.** Catches "the function exists but no path reaches it" but says nothing about whether the function does what the docs promise. Complementary, not a replacement.
- **Mining REQUIREMENTS.md alone.** Requirements docs are higher signal but lower coverage than reference docs and concept docs. We mine both.
- **"Reverse" coverage (every test → doc claim).** Most tests are unit/internal; requiring each to bind to a doc claim is over-restrictive. Forward direction (claim → test) is the meaningful one. Outside-in tests like `dark_factory_real_*` arguably should each bind to ≥1 claim, but that's a narrow rule, not a project-wide gate.

## What we explicitly chose for

- **Bidirectional alignment as the framing** — the dimension is named `docs-alignment`, not `docs-coverage`. Alignment captures both directions; coverage suggests one. Renaming pre-implementation costs nothing.
- **Hash-driven drift detection on every push, no LLM in the push path.** Subagents extract and grade only when triggered (doc file in commit diff, phase close, TTL expiry). Daily verdicts come from cargo test exit codes plus deterministic hash walks.
- **Retirement requires explicit human signature.** The path of least resistance must NOT be "delete the claim to make CI green." `confirm-retire` env-guards against agent contexts; agents can `propose-retire` but only humans confirm.

## What this milestone does not aim to fix

- Reverse coverage (test → claim) is out of scope. Maybe a future narrow rule for outside-in tests.
- Migrating existing `quality/gates/` scripts under the new `reposix-quality` umbrella is NOT P64 work — too much scope. Only the new doc-alignment subcommands ship under the umbrella in P64.
- The Confluence symlink regression itself is **not fixed in v0.12.0**. P65 surfaces it as a `MISSING_TEST` row; v0.12.1's gap-closure phases re-implement the dropped behavior and bind tests.
