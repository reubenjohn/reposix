# Vision & Innovations Brainstorm — Post-v0.10.0 Trajectory

**Status:** Research draft, not a roadmap mutation. Owner reads, picks signal, ignores noise.
**Author context:** Drafted 2026-04-24 evening, immediately after v0.10.0 (Docs & Narrative Shine) shipped and v0.11.0 (Performance & Sales Assets) entered planning. The owner asked: "periodically re-think the vision and propose original innovations." This document is the artifact.
**Inputs cited (not re-derived):** `.planning/PROJECT.md`, `.planning/research/v0.10.0-post-pivot/milestone-plan.md`, `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`, `docs/research/initial-report.md`, `docs/research/agentic-engineering-reference.md`, `CLAUDE.md`, `docs/index.md`, `docs/concepts/*.md`, `.planning/notes/v0.11.0-doc-polish-backlog.md`, `.planning/CATALOG.md`.

---

## 1. Where reposix sits today (one paragraph)

reposix as of 2026-04-25 is a Rust workspace that turns any REST issue tracker into a real git working tree. v0.9.0 deleted the FUSE layer and replaced it with a `stateless-connect` + `export` hybrid `git-remote-reposix` helper backed by a local bare-repo cache (`reposix-cache`); the working tree is now a bona fide partial-clone checkout, not a virtual filesystem. v0.10.0 made that pivot legible — three concept pages, three how-it-works pages with mermaid diagrams, a 5-minute tutorial verified by `scripts/tutorial-runner.sh`, a banned-words linter, a README hero with measured numbers (`8 ms` cache read, `24 ms` cold init, `9 ms` list, `5 ms` capabilities probe), and a CHANGELOG that finalised the architecture story. The agent UX promise is: **after `reposix init <backend>::<project> <path>`, every operation is a git command the agent already knows from pre-training; reposix is invisible.** Five-second positioning: *"reposix turns issue trackers into git repositories so coding agents stop burning 100k tokens on MCP schemas and start spending zero on `cat`."*

---

## Chapters

- **[innovations.md](./innovations.md)** — §3a–3f: the six highest-conviction innovations (pitch, mechanism sketch, phase-to-prove-it, why-now).
- **[innovations-g-m.md](./innovations-g-m.md)** — §3g–3m: plugin registry, swarm replay, capability negotiation, gc/archive, streaming push, research paper, other ideas.

- **[strategy.md](./strategy.md)** — §2 Five-year vision · §4 Cuts and trade-offs · §5 Next-3-month plan · §6 Originality audit · §7 Open questions · §8 Owner decisions (2026-04-25).
