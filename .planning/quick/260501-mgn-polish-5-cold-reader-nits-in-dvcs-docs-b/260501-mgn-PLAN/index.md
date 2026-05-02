---
phase: 260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/concepts/dvcs-topology.md
  - docs/guides/dvcs-mirror-setup.md
autonomous: true
requirements:
  - dvcs-cold-reader-finding-1-binstall-crate-name
  - dvcs-cold-reader-finding-2-secret-set-interactive-note
  - dvcs-cold-reader-finding-3-sed-readability
  - dvcs-cold-reader-finding-4-slug-hedge
  - dvcs-cold-reader-finding-5-template-anchor

must_haves:
  truths:
    - "docs/concepts/dvcs-topology.md line 134 says `cargo binstall reposix-cli` (NOT `cargo binstall reposix`)"
    - "docs/guides/dvcs-mirror-setup.md secret-set block (lines 61-63 region) has a one-line note flagging interactive prompt + non-TTY pointer"
    - "docs/guides/dvcs-mirror-setup.md cron-update sed example (line 147 region) is more readable than the doubly-escaped form"
    - "docs/concepts/dvcs-topology.md line 61 region states the slug is `confluence` definitively (no 'or your tenant alias' hedge)"
    - "docs/guides/dvcs-mirror-setup.md has a one-line anchor near top (after intro, before Step 1) naming docs/guides/dvcs-mirror-setup-template.yml as the workflow template location"
    - "docs/concepts/dvcs-topology.md still passes quality/gates/docs-alignment/dvcs-topology-three-roles.sh (three role tokens + Q2.2 phrase fragments still present)"
    - "docs/guides/dvcs-mirror-setup.md still passes quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh (Steps 1-5, Cleanup, Backends without webhooks, gh secret set, template reference all present)"
    - "scripts/banned-words-lint.sh exits 0 (no FUSE/kernel/partial-clone/promisor/stateless-connect/fast-import/protocol-v2 leaks introduced)"
    - "scripts/check-docs-site.sh exits 0 (mkdocs strict build still green)"
  artifacts:
    - path: "docs/concepts/dvcs-topology.md"
      provides: "Updated topology doc with binstall crate-name fix + slug hedge removal"
      contains: "cargo binstall reposix-cli"
    - path: "docs/guides/dvcs-mirror-setup.md"
      provides: "Updated setup guide with template anchor + secret-set interactive note + readable sed example"
      contains: "dvcs-mirror-setup-template.yml"
  key_links:
    - from: "docs/concepts/dvcs-topology.md (Pattern C, line ~134)"
      to: "the binstall crate name `reposix-cli` (matches dvcs-mirror-setup.md:89)"
      via: "literal string"
      pattern: "cargo binstall reposix-cli"
    - from: "docs/guides/dvcs-mirror-setup.md (intro region)"
      to: "docs/guides/dvcs-mirror-setup-template.yml"
      via: "anchor sentence near top of file"
      pattern: "dvcs-mirror-setup-template\\.yml"
---

# 260501-mgn — Polish 5 cold-reader nits in DVCS docs (Plan 01)

<objective>
Polish 5 non-critical findings surfaced by the dvcs-cold-reader rubric verdict
(score 8 CLEAR, zero critical-friction). All five are docs-only edits across
two files (docs/concepts/dvcs-topology.md + docs/guides/dvcs-mirror-setup.md).
The verdict artifact at quality/reports/verifications/subjective/dvcs-cold-reader.json
stays AS-IS — owner re-grades on the next /reposix-quality-review run.

Purpose: tighten the cold-reader experience for the DVCS docs cluster. Three
findings fix a correctness gap a copy-paster would hit (binstall crate name,
secret-set non-TTY behavior, slug hedge ambiguity). Two improve grounding
(template anchor near top of setup guide, readable sed example).

Output: one atomic commit `docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

## Chapters

- **[context.md](./context.md)** — Catalog safety pre-audit, exact line-target interfaces for all 5 findings, and verification steps summary. Read this before executing.
- **[task-1.md](./task-1.md)** — Task 1 (the single atomic edit task), threat model, verification contract, success criteria, and output spec.

Full success criteria, output spec, threat model, and verification contract: see [task-1.md](./task-1.md).
