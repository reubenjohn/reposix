# Phase 25 CONTEXT — Docs reorg: `InitialReport.md` and `AgenticEngineeringReference.md` out of root (OP-11)

> Status: scoped in session 5, 2026-04-14. User explicitly flagged as "not tonight" — do not start without a session check-in.
> Author: planning agent, session 6 prep.
> Low blast-radius: file moves + cross-ref updates only. No code changes.

## Phase identity

**Name:** Docs reorg — `InitialReport.md` and `AgenticEngineeringReference.md` out of repo root (OP-11).

**Scope tag:** v0.7.0 (docs-only — no Rust code changes, no API changes).

**Addresses:** OP-11 from HANDOFF.md. User flagged the repo root has narrative prose docs that don't belong at the top level. Proposed moves captured in session 3, explicitly NOT executed until a dedicated phase.

## Goal (one paragraph)

The repo root currently holds two large narrative prose documents (`InitialReport.md` and `AgenticEngineeringReference.md`) that belong in `docs/research/` alongside the rest of the user-facing documentation. This phase executes the moves, updates every cross-reference in `CLAUDE.md`, `README.md`, and any planning doc that links these files, catalogs any remaining root-level clutter the sweep identifies, and writes a redirect-note commit at the old location for github.com readers who may have bookmarked these paths. The move is planned as one commit per logical group to keep the git history readable.

## Source design context

From HANDOFF.md §OP-11 (verbatim):

> User flagged: the repo root has narrative prose docs (`InitialReport.md`, `AgenticEngineeringReference.md`) that don't belong at the top level. Along with other root-level clutter the sweep in OP-6 catalogs. Proposed moves (**captured, not executed tonight** — user explicitly said so):
>
> - `InitialReport.md` → `docs/research/initial-report.md` (this is the original architectural argument; move near the rest of docs)
> - `AgenticEngineeringReference.md` → `docs/research/agentic-engineering-reference.md`
> - Update cross-refs: `CLAUDE.md`, `README.md`, any planning doc that links these two.
> - Any other root-level cruft the sweep catalogs.
>
> Kept at root: `README.md`, `CHANGELOG.md`, `HANDOFF.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `Cargo.toml`, `Cargo.lock`, `mkdocs.yml`, `rust-toolchain.toml`, `.env.example`, `.gitignore`. Everything else either belongs in `docs/` or `.planning/` or under a crate.
>
> Plan the move as one commit per logical group, each with a redirect-note committed in the old location if any external-to-repo links might break (github.com has some readers who bookmark these).

## Proposed commit sequence

1. **Commit A — move `InitialReport.md`:** `git mv InitialReport.md docs/research/initial-report.md`. Add redirect stub at `InitialReport.md` (one line: `<!-- Moved to docs/research/initial-report.md -->`). Update `CLAUDE.md` and `README.md` refs.
2. **Commit B — move `AgenticEngineeringReference.md`:** `git mv AgenticEngineeringReference.md docs/research/agentic-engineering-reference.md`. Add redirect stub. Update `CLAUDE.md` and `README.md` refs.
3. **Commit C — catalog sweep:** Run `ls` on repo root; identify any remaining files not on the "kept at root" list; move or archive each. Update `mkdocs.yml` nav if needed.
4. **Commit D — cross-ref audit:** Search all `.md` files under `.planning/` for references to `InitialReport.md` or `AgenticEngineeringReference.md`; update to new paths.

## Design questions

1. **Redirect stub format.** A one-line HTML comment (`<!-- Moved to ... -->`) is invisible to GitHub's rendered markdown — users who land on the old URL see a blank page. Consider a visible markdown redirect note (`> This document has moved to [docs/research/initial-report.md](docs/research/initial-report.md).`) with a link. Define the format before executing.
2. **mkdocs.yml nav.** The docs site nav (`mkdocs.yml`) may not currently include these files (they were at root, outside the `docs/` source dir). After the move, add them to the nav under a new `research/` section. Verify `mkdocs build --strict` stays green.
3. **Root-clutter catalog.** The OP-6 sweep did not enumerate all root-level candidates. This phase should `ls` the root and explicitly list what stays vs what moves before executing any moves — no surprises.

## Canonical refs

- `InitialReport.md` — full architectural argument for FUSE + git-remote-helper. Target: `docs/research/initial-report.md`.
- `AgenticEngineeringReference.md` — dark-factory pattern, lethal trifecta, simulator-first. Target: `docs/research/agentic-engineering-reference.md`.
- `CLAUDE.md` (project root) — §"Quick links" references both files; must be updated.
- `README.md` — may reference these files; audit before commit.
- `mkdocs.yml` — nav configuration; add `research/` section after move.
- `HANDOFF.md §OP-11` — original design capture.
