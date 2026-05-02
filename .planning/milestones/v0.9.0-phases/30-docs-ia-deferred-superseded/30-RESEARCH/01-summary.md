← [back to index](./index.md) · phase 30 research

## Summary

Phase 30 is a pure docs phase: rewrite the landing page, split `architecture.md` + `security.md` into a three-page "How it works" section, add two home-adjacent concept pages, add three guides, write a 5-minute tutorial against the simulator, retune the mkdocs-material theme, and add a progressive-disclosure banned-word linter. Nine requirements (DOCS-01..09), 8+ new/modified pages, 3 mermaid diagrams, 1 linter, 1 tutorial with playwright screenshot proof.

All the tooling we need is already on the machine: `mkdocs` 1.6.1 + `mkdocs-material` 9.7.1, `CairoSVG` 2.7.1, `pillow` 10.4.0 (so `material[imaging]` social cards work with no new installs), `mmdc` (mermaid-cli) 11.12.0 on `$PATH`, Playwright Chromium already cached, the `doc-clarity-review` skill is in place and is the canonical cold-reader validator. `pymdownx.superfences` with the mermaid custom fence is already wired in `mkdocs.yml`, so diagrams already render client-side.

**Primary recommendation:** Take the IA sketch from the source-of-truth note as-is, and structure the plan along the existing subagent-fanout suggestion (Explore → Copy → IA → Diagrams → Tutorial), with one addition — a linter-build plan that ships Vale with a scoped banned-words rule. Use the existing `doc-clarity-review` skill as the "10-second value-prop lands" verification: point it at the rendered `docs/index.md` with a purpose-built prompt and parse its verdict. Scope-split is NOT recommended — the phase fits in one pass if parallelized as suggested, and the artifacts are mutually reinforcing (copy ↔ diagram ↔ tutorial). A split would create handoff friction.

<user_constraints>
