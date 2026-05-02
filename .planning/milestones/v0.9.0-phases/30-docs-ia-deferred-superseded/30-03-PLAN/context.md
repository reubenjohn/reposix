← [back to index](./index.md)

# Context, interfaces, and threat model

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/CONTEXT.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md
@.planning/notes/phase-30-narrative-vignettes.md

@docs/index.md
@docs/why.md
@docs/mental-model.md
@docs/vs-mcp-sdks.md

<interfaces>
Locked content from the source-of-truth note, to be quoted VERBATIM in tasks below:

1. **Before code block** (docs/index.md hero) — source-of-truth lines 116-146: 30-line bash with five curl/jq round-trips for closing PROJ-42 in JIRA.
2. **After code block** (docs/index.md hero) — source-of-truth lines 152-170: 8-line bash using `sed` + `git commit` + `git push`.
3. **Complement blockquote** (mandatory directly under "after") — source-of-truth lines 173-177:

> You still have full REST access for the operations that need it — JQL
> queries, bulk imports, admin config. reposix just means you don't have
> to reach for it for the hundred small edits you'd otherwise make every
> day.

4. **Three mental-model H2s** — source-of-truth lines 310-312, already locked into plan 30-02 skeleton verbatim:
   - `mount = git working tree`
   - `frontmatter = schema`
   - `` `git push` = sync verb ``

5. **Tokenomy numbers** (vs-mcp-sdks.md citation target) — `docs/why.md` §"token-economy-benchmark": 4,883 tokens (MCP) vs 531 tokens (reposix) — 92.3% reduction.
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Hero copy -> published site | Published docs are public by definition. No PII, tokens, or real endpoints may appear in examples. |
| Example bash -> reader expectation | `$E:$T` shorthand in the before-block examples is a placeholder, not a real token reference — reader must not be encouraged to paste actual credentials into a prompt. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-03-01 | Information Disclosure | Copy leaks real endpoints, emails, or token formats | mitigate | Use `acme.atlassian.net` (documented test domain), `PROJ-42` (synthetic ID), `alice@acme.com` (synthetic email). Reviewer scans diff for `.atlassian.net` subdomains that match real tenants (e.g. `reuben-john.atlassian.net` which appears in live demos). |
| T-30-03-02 | Tampering | Copy accidentally includes banned P1/P2 terms above Layer 3 | mitigate | Vale runs on every commit via `pre-commit-docs` hook (plan 30-01); CI runs `vale --config=.vale.ini docs/` and fails. Any violation is blocked. |
| T-30-03-03 | Repudiation | Complement-line blockquote is silently trimmed/shortened | mitigate | Acceptance criteria grep for exact anchor phrase "You still have full REST access" — removal fails verification. |
</threat_model>
