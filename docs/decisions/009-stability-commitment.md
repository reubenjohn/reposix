# ADR-009 — Stability commitment: what changes (and what doesn't) after v1.0

| | |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-04-26 |
| **Phase** | v0.11.1 (POLISH2-17, friction matrix Row 13) |
| **Supersedes / amends** | none |

## Context

reposix is pre-1.0. Pre-1.0 versions follow Cargo's "0.x is
breaking-allowed" convention, and we have used that latitude freely:
ADR-008 changed the `reposix::` URL shape between v0.9.0 and v0.10.0,
and the v0.9.0 architecture pivot rewrote the entire FUSE layer to a
git-remote-helper + partial-clone substrate.

Harness authors integrating reposix into coding agents need to know
what they can pin against. A floating dependency on the `reposix-cli`
crate or the `git-remote-reposix` binary is acceptable for prototypes,
but the persona-harness-author audit (friction matrix Row 13) flagged
the absence of any stability commitment as a "block-the-recommend"
risk: harness authors will not commit to a `cargo binstall`-able
version without knowing which surfaces are stable.

This ADR defines the stable contract that takes effect at the v1.0.0
tag. Before v1.0.0 nothing here is binding; after v1.0.0 every item
below is governed by semantic versioning (breaking changes require a
major bump).

## Decision

### Stable surfaces (locked at v1.0.0 under semver)

1. **`reposix init` URL shape.** The canonical form
   `reposix::<scheme>://<host>/<backend>/projects/<project>` (the
   post-v0.10.0 form ratified by ADR-008) is locked. New backends MAY
   be added (additive change, minor bump); the URL shape itself MAY
   NOT change without a major bump.
2. **CLI subcommand surface.** The set
   `reposix init|sim|list|refresh|spaces|log|history|tokens|cost|gc|doctor|--version`
   is locked. New subcommands MAY be added as additive minor versions.
   Removing a subcommand name, or removing/renaming an existing
   argument name, is a breaking change.
3. **Exit codes.** The codes documented in
   [`docs/reference/exit-codes.md`](../reference/exit-codes.md) are
   locked. New exit codes MAY be added as additive minor versions;
   existing codes MAY NOT change meaning. (The helper's exit `2` for
   anyhow-chain failures is documented as "best parse stderr"; a
   future v1.1 may add exit `3` for blob-limit refusal as a
   non-breaking addition.)
4. **`git-remote-reposix` protocol surface.** The advertised
   capabilities `stateless-connect` and `export` are locked, as is the
   refspec namespace `refs/heads/*:refs/reposix/*`. Adding new
   capabilities is additive; removing or renaming either is breaking.
5. **Frontmatter field allowlist.** The server-controlled fields
   (`id`, `created_at`, `version`, `updated_at`) and the rule that
   client writes cannot override them are locked. Adding a new
   server-controlled field is a breaking change for clients that write
   it; we will deprecate-then-remove with at least one minor version
   of warning before the major bump that removes the override path.
6. **`BackendConnector` trait.** Locked under `non_exhaustive`
   discipline. New trait methods land via the existing
   `BackendFeature` extension pattern, gated on capability flags so
   existing implementors compile unchanged. Adding a required method
   to the base trait is a breaking change.

### Surfaces explicitly NOT covered

The following are out of scope for the stability commitment and may
change at any minor version:

- **`reposix-cli` library internals.** The `pub mod` re-exports inside
  the crate are `#[doc(hidden)]` and may move or vanish without
  notice. Integrators must consume the `reposix` binary or the
  `git-remote-reposix` binary, never the lib surface.
- **On-disk cache schema.** The layout under
  `.reposix-cache/<backend>-<project>.git/` is reproducible from the
  backend; we may bump the schema and invalidate caches at a minor
  version with a one-line release-note callout. The cache is a
  rebuildable artifact, not a contract.
- **Simulator HTTP wire format.** The simulator at
  `crates/reposix-sim/` is a test seam, not a public REST API.
  Consumers requiring stability should use one of the real backends
  (GitHub, Confluence, JIRA) instead.
- **Capability matrix.** The per-backend capability tables in
  `docs/concepts/` may add or remove rows as backends evolve. The
  capability-flag mechanism itself is locked (point 6 above), but the
  set of flags and which backend supports which is not.

## Consequences

### Positive

- Harness authors can pin `reposix-cli >=1.0, <2.0` and trust the URL
  shape, exit codes, and protocol surface across the entire major
  series.
- New backends ship as additive minor versions, with no risk of
  invalidating existing harness integrations.
- Internal refactors (e.g. the `confluence/lib.rs` split tracked in
  the v0.11.1+ catalog) are non-breaking by construction because the
  lib surface is hidden.
- The friction-matrix Row 13 risk for harness adopters is closed:
  there is now a written commitment they can cite.

### Negative

- The CLI subcommand and exit-code surfaces are large enough that
  locking them constrains future redesign. Mitigation: the additive
  rule covers all foreseeable extensions, and the
  internal-modules-are-hidden carve-out keeps refactor freedom intact.
- The `BackendConnector` trait sits inside `reposix-core`, which
  third-party connector authors depend on. A future v2.0 will impose
  a real migration cost on those authors. Mitigation: we expect
  12-24 months of v1.x before a v2 is worth contemplating, and the
  `BackendFeature` extension pattern absorbs most additive changes
  without touching the base trait.

### Neutral

- The simulator wire format being explicitly unstable is a deliberate
  asymmetry: it lets us evolve the test seam aggressively while the
  real-backend contract stays frozen. Consumers who built against the
  simulator wire format directly (rather than the binary surface) are
  on notice.

## References

- [ADR-008 — Helper URL-scheme backend dispatch](008-helper-backend-dispatch.md):
  the canonical post-v0.10.0 URL shape locked under point 1 above.
- [`docs/reference/exit-codes.md`](../reference/exit-codes.md): the
  exit-code surface locked under point 3 above.
- `crates/reposix-core/src/backend.rs`: the `BackendConnector` trait
  and `BackendCapabilities` struct (the latter introduced in v0.11.1
  via POLISH2-08) referenced under point 6 above.
- Friction matrix Row 13 (persona-harness-author audit): the
  motivating gap this ADR closes.
