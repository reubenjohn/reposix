# Phase 44 — Doc Clarity Review (release gate)

**Goal.** Run inline doc-clarity-review-style audits on every user-facing page and eliminate critical friction points so DOCS-10 is satisfied and the v0.10.0 release gate is open.

**Methodology.** Inline cold-reader audit (the orchestrator runs the rubric directly — no Claude subprocess fan-out). Each user-facing page is judged by the eight checks in the runner prompt: hook, glossary leaks, code/command runnability, cross-link integrity, promise vs delivery, number freshness, paragraph density, calls-to-action.

**Severity.**
- Critical — actively misleading, broken link, dead copy-paste command, P1/P2 violation the linter missed. MUST fix.
- Major — friction (cold reader stuck), missing definition, weak hook. Backlog.
- Minor — style nit, occasional clunky phrasing, slightly stale number. Backlog.

**Tooling promoted this phase.** `scripts/check_doc_links.py` — verifies relative Markdown links across the 19 user-facing pages, becomes a reusable pre-commit/CI guard.

**Carry-forward to Phase 45.**
- README.md outside the v0.9.0 quickstart is largely v0.7-era copy with `reposix mount` commands; Phase 45 owns the README hero rewrite + adjective-replacement pass.
- mkdocs anchor warnings deferred from Phase 43 are not addressed here.

**Inputs.**
- `.planning/REQUIREMENTS.md` DOCS-10
- `.planning/ROADMAP.md` Phase 44
- `docs/.banned-words.toml` (P1/P2 layer rules)
- `docs/benchmarks/v0.9.0-latency.md` (canonical numbers)
