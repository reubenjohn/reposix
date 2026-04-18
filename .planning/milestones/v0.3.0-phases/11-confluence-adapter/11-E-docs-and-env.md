---
phase: 11-confluence-adapter
plan: E
type: execute
wave: 2
depends_on: []
files_modified:
  - docs/decisions/002-confluence-page-mapping.md
  - docs/reference/confluence.md
  - docs/architecture.md
  - docs/demos/index.md
  - README.md
  - CHANGELOG.md
  - .env.example
autonomous: true
requirements:
  - FC-02
  - FC-09
  - SG-08
user_setup: []

must_haves:
  truths:
    - "ADR-002 exists at `docs/decisions/002-confluence-page-mapping.md` following ADR-001's structure and documents the page→issue mapping + lossy-metadata tradeoff"
    - "`.env.example` no longer mentions `TEAMWORK_GRAPH_API`; names the three new Atlassian env vars with dashboard sources"
    - "`docs/reference/confluence.md` exists with endpoint / auth / pagination / rate-limit reference"
    - "`README.md` Tier-5 section lists the Confluence demo alongside the GitHub one"
    - "`docs/architecture.md` crate-topology diagram or text includes `reposix-confluence`"
    - "`CHANGELOG.md` [Unreleased] section summarizes the Phase 11 additions"
    - "`docs/demos/index.md` lists `parity-confluence.sh` (Tier 3B) and `06-mount-real-confluence.sh` (Tier 5)"
  artifacts:
    - path: "docs/decisions/002-confluence-page-mapping.md"
      provides: "Accepted ADR documenting page→issue mapping, auth choice, pagination, lost metadata"
      min_lines: 80
      contains: "ADR-002"
    - path: "docs/reference/confluence.md"
      provides: "User-facing reference for the Confluence backend"
      min_lines: 60
    - path: ".env.example"
      provides: "Renamed var + two new vars with dashboard source instructions"
      contains: "ATLASSIAN_API_KEY"
    - path: "CHANGELOG.md"
      provides: "[Unreleased] section filled in with Phase 11 additions"
      contains: "reposix-confluence"
  key_links:
    - from: "README.md"
      to: "docs/decisions/002-confluence-page-mapping.md"
      via: "Markdown link in Tier-5 section"
      pattern: "002-confluence-page-mapping"
    - from: "docs/decisions/002-confluence-page-mapping.md"
      to: "ADR-001 structure"
      via: "Same section headings (Status, Context, Decision, Consequences, References)"
---

<objective>
Finalize the human-facing surface of Phase 11: ADR-002 (Confluence page mapping), a reference doc, README + architecture updates, CHANGELOG [Unreleased] entry, and the `.env.example` rename / additions. Docs-only plan that runs in parallel with 11-C (contract test) in Wave 2 — non-overlapping files.

Purpose: Without 11-E, the phase ships without the knowledge that the user (and next agent) needs to understand WHY decisions were made. ADR-002 specifically captures the Option-A-flattening choice + the lost-metadata tradeoff that a Phase 11 user will see eat their round-trip fidelity — material enough to deserve git, not just CONTEXT.md.

Output: One new ADR, one new reference page, five edited docs (README, architecture, demos index, CHANGELOG, .env.example).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/11-RESEARCH.md
@.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md
@CLAUDE.md
@docs/decisions/001-github-state-mapping.md
@README.md
@CHANGELOG.md
@.env.example
@docs/demos/index.md

<interfaces>
ADR-001 structure to follow (verbatim headings order):
1. `# ADR-NNN: <title>`
2. metadata bullets: Status, Date, Deciders, Supersedes, Superseded by, Scope
3. `## Context`
4. `## Decision` (with tables + subsections `### Read-path rules`, `### Write-path rules`, `### Unknown-value handling`)
5. `## Consequences`
6. `## References`

`.env.example` current Atlassian section (to be replaced):
```
TEAMWORK_GRAPH_API=
```

README Tier-5 section: find with `grep -n 'Tier 5\|05-mount-real-github' README.md`. The existing row format (table pipe or bullet) is what we match.

mkdocs-strict caveat (HANDOFF §8): relative links outside `docs/` are rejected by `mkdocs build --strict`. Use absolute GitHub URLs (`https://github.com/reubenjohn/reposix/blob/main/...`) when linking to `scripts/` or `.planning/` from inside `docs/`.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write ADR-002 (Confluence page mapping)</name>
  <files>
    docs/decisions/002-confluence-page-mapping.md
  </files>
  <action>
    Create `docs/decisions/002-confluence-page-mapping.md` mirroring ADR-001's structure. Required content (executor writes prose using 11-CONTEXT.md + 11-RESEARCH.md):

    Required sections and content:

    **H1 + metadata block** — identical shape to ADR-001's opening.

    **`## Context`** — explain Confluence = hierarchical wiki-pages, reposix = flat issues. Reference the three options from HANDOFF.md §3 (A flatten / B PageBackend trait / C optional parent on Issue). Explain why Option A ships in v0.3: it reuses FUSE + CLI machinery unchanged, whereas B is a phase of trait work and C introduces schematic ambiguity across backends.

    **`## Decision`** — mapping table (`Issue` field | Confluence source | Type coercion). Every row from 11-RESEARCH.md §Pattern Delta / §Protocol mapping:
    - `id` ← `page.id` (`parse::<u64>`; error on non-numeric)
    - `title` ← `page.title`
    - `status` ← `page.status` with branch table: `current|draft` → `Open`; `archived|trashed|deleted` → `Done`; unknown → `Open` (pessimistic forward-compat)
    - `body` ← `page.body.storage.value` (raw XHTML; `""` when body not requested)
    - `created_at` ← `page.createdAt` (ISO 8601 → `chrono::DateTime<Utc>`)
    - `updated_at` ← `page.version.createdAt` (nested under version, NOT top-level)
    - `version` ← `page.version.number`
    - `assignee` ← `page.ownerId` (Atlassian account ID; `None` if absent)
    - `labels` ← `[]` (deferred to v0.4; Confluence labels are a separate endpoint)

    **`### Lost metadata (deliberate)`** subsection — enumerate fields that v0.3 does NOT carry: `parentId`, `parentType`, `spaceId`, `spaceKey`, `_links.{webui,editui,tinyui}`, `position`, `body.atlas_doc_format`. Document that round-tripping a page through `reposix mount --backend confluence` loses this metadata, and that v0.4's extension path is through `Issue.extensions` or a `PageBackend` trait.

    **`### Auth decision`** — Basic auth only (`email:api_token`). No Bearer (requires OAuth 2.0 3LO, out of scope). Cite `00-CREDENTIAL-STATUS.md` for the common `FAILURE_CLIENT_AUTH_MISMATCH` failure mode.

    **`### Pagination decision`** — cursor-in-body via `_links.next` (relative path). Adapter prepends tenant base URL. 500-page cap (same as GitHub).

    **`### Rate-limit decision`** — `Retry-After` header (seconds), `X-RateLimit-Remaining == 0` arms the gate. Different from GitHub's `x-ratelimit-reset` (unix epoch).

    **`### Read-path only (v0.3)`** — `create_issue`, `update_issue`, `delete_or_close` all return `Err(Error::Other("not supported: ..."))`. Write path deferred to v0.4 with a sanitize step for server-authoritative fields.

    **`## Consequences`** — bullet each tradeoff:
    - No `cd`-into-hierarchy UX on the mount.
    - Body is ugly XHTML, not Markdown (`atlas_doc_format` rendering is v0.4).
    - Space resolver is an extra round-trip per `list_issues` (no cache in v0.3).
    - No OAuth path.
    - Page IDs must be u64-parseable; non-numeric surfaces as a clean Error.

    **`## References`** — link ADR-001, 11-CONTEXT.md, 11-RESEARCH.md, the four Atlassian doc URLs, `crates/reposix-confluence/src/lib.rs`, `crates/reposix-core/src/backend.rs`.

    Target: 90-160 lines (ADR-001 is ~105; same ballpark).
  </action>
  <verify>
    <automated>test -f docs/decisions/002-confluence-page-mapping.md &amp;&amp; grep -q '^# ADR-002' docs/decisions/002-confluence-page-mapping.md &amp;&amp; grep -q '^- \*\*Status:\*\* Accepted' docs/decisions/002-confluence-page-mapping.md &amp;&amp; grep -qE '(Option A|option-A|flatten)' docs/decisions/002-confluence-page-mapping.md &amp;&amp; grep -qE '(Lost metadata|lost metadata|deliberate)' docs/decisions/002-confluence-page-mapping.md &amp;&amp; [ "$(wc -l &lt; docs/decisions/002-confluence-page-mapping.md)" -ge 80 ]</automated>
  </verify>
  <done>
    ADR-002 exists, ≥80 lines, ADR-001 structural parity, covers mapping + lost-metadata + auth + pagination + rate-limit + consequences. Commit: `docs(11-E-1): ADR-002 Confluence page to issue mapping`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Write `docs/reference/confluence.md`</name>
  <files>
    docs/reference/confluence.md
  </files>
  <action>
    Create `docs/reference/confluence.md`. Sections:

    **Summary** — one paragraph: `reposix-confluence` is a read-only adapter for Atlassian Confluence Cloud via REST v2, implementing the `IssueBackend` trait. It ships in v0.3.0.

    **CLI surface** — show the two invocations:
    ```bash
    reposix list  --backend confluence --project <SPACE_KEY>
    reposix mount <dir> --backend confluence --project <SPACE_KEY>
    ```
    Note: `--project` is the SPACE KEY (e.g. `REPOSIX`), not a numeric space id; the adapter resolves it internally.

    **Required env vars** — documentation table with columns: Var / What / Where to get it.
    - `ATLASSIAN_API_KEY` — API token; from <https://id.atlassian.com/manage-profile/security/api-tokens>
    - `ATLASSIAN_EMAIL` — the Atlassian account email that issued the token (must match exactly; see Known failure modes)
    - `REPOSIX_CONFLUENCE_TENANT` — the subdomain of your Atlassian Cloud tenant (`<tenant>.atlassian.net`)
    - `REPOSIX_ALLOWED_ORIGINS` — must include `https://<tenant>.atlassian.net` (SG-01 enforcement)

    **Credential setup** — step-by-step: (1) go to id.atlassian.com/manage-profile/security/api-tokens; (2) note the email shown at top-right (THIS is `ATLASSIAN_EMAIL`); (3) "Create API token" with a descriptive label; (4) copy to `.env`; (5) `source .env && export REPOSIX_CONFLUENCE_TENANT=yourtenant`.

    **Auth** — one-paragraph pointer to ADR-002 + summary that it's Basic auth.

    **Pagination** — one paragraph: cursor-in-body via `_links.next`, capped at 500 pages. Link to ADR-002.

    **Rate limiting** — one paragraph: `Retry-After` + `X-RateLimit-Remaining`. Link to [Atlassian rate-limiting docs](https://developer.atlassian.com/cloud/confluence/rate-limiting/).

    **What's NOT supported in v0.3** — bullet list from ADR-002's "Lost metadata" + the no-write-path note. Link to ADR-002 for full list.

    **Known failure modes** — table:
    - `401 FAILURE_CLIENT_AUTH_MISMATCH` → ATLASSIAN_EMAIL doesn't match the token's issuing account. Fix per 00-CREDENTIAL-STATUS.md.
    - `403 Forbidden` → auth works but no permission on the space. Check the space's permissions.
    - `invalid confluence tenant subdomain` → tenant string has characters outside `[a-z0-9-]`. Fix `REPOSIX_CONFLUENCE_TENANT`.
    - `REPOSIX_ALLOWED_ORIGINS must include https://<tenant>.atlassian.net` → set the allowlist env var.

    **Demos** — linked list pointing to `scripts/demos/parity-confluence.sh` (Tier 3B) and `scripts/demos/06-mount-real-confluence.sh` (Tier 5). Use absolute GitHub URLs per mkdocs-strict rule.

    Target: 70-130 lines.
  </action>
  <verify>
    <automated>test -f docs/reference/confluence.md &amp;&amp; [ "$(wc -l &lt; docs/reference/confluence.md)" -ge 60 ] &amp;&amp; grep -q 'ATLASSIAN_API_KEY' docs/reference/confluence.md &amp;&amp; grep -q 'ATLASSIAN_EMAIL' docs/reference/confluence.md &amp;&amp; grep -q 'REPOSIX_CONFLUENCE_TENANT' docs/reference/confluence.md &amp;&amp; grep -q 'FAILURE_CLIENT_AUTH_MISMATCH' docs/reference/confluence.md</automated>
  </verify>
  <done>
    Reference doc exists, ≥60 lines, covers all four env vars and the credential-mismatch failure. Commit: `docs(11-E-2): reference/confluence.md user-facing guide`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Update .env.example, CHANGELOG, README, docs/architecture.md, docs/demos/index.md</name>
  <files>
    .env.example,
    CHANGELOG.md,
    README.md,
    docs/architecture.md,
    docs/demos/index.md
  </files>
  <action>
    **`.env.example`:**
    Replace the Atlassian section. New content:
    ```
    # --------------------------------------------------------------- Atlassian
    # Atlassian Confluence Cloud credentials. Used by reposix-confluence
    # (v0.3+) to read Confluence pages via REST v2 Basic auth.
    #
    # ATLASSIAN_API_KEY: API token issued from
    #   https://id.atlassian.com/manage-profile/security/api-tokens
    #   The token is ACCOUNT-SCOPED: it only works with the email the
    #   account is registered under (see ATLASSIAN_EMAIL).
    # ATLASSIAN_EMAIL:   the account email shown at top-right on that page.
    #                    Must match the token's issuing account EXACTLY.
    # REPOSIX_CONFLUENCE_TENANT: the subdomain of your Atlassian Cloud tenant
    #                    (e.g. `mycompany` for https://mycompany.atlassian.net).
    ATLASSIAN_API_KEY=
    ATLASSIAN_EMAIL=
    REPOSIX_CONFLUENCE_TENANT=
    ```
    Also update the `REPOSIX_ALLOWED_ORIGINS` commented example at the bottom to:
    ```
    # Example for dev against both real backends:
    # REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:*,http://localhost:*,https://api.github.com,https://<tenant>.atlassian.net
    ```
    Remove ALL mentions of `TEAMWORK_GRAPH_API` and `teamwork` from `.env.example`. Confirm with `! grep -qi teamwork .env.example`.

    **`CHANGELOG.md`:**
    Under the `## [Unreleased]` heading, replace the `— Nothing yet.` line with an "Added" + "Changed" block summarizing Phase 11:

    - Added: `crates/reposix-confluence` (describe features — Basic auth, cursor pagination, Retry-After rate gate, 500-page cap), `reposix list --backend confluence`, `reposix mount --backend confluence`, env vars (ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT), `scripts/demos/parity-confluence.sh` (Tier 3B), `scripts/demos/06-mount-real-confluence.sh` (Tier 5), `crates/reposix-confluence/tests/contract.rs` (parameterized over sim + wiremock + live), `docs/decisions/002-confluence-page-mapping.md`, `docs/reference/confluence.md`, CI job `integration-contract-confluence`.
    - Changed: `.env.example` rename `TEAMWORK_GRAPH_API` → `ATLASSIAN_API_KEY` + added two new vars; workspace adds `reposix-confluence` + `base64 = "0.22"` dep.

    Do NOT touch `## [v0.2.0-alpha]` or below.

    **`README.md`:**
    1. Find the Tier 5 reference using `grep -n 'Tier 5\|05-mount-real-github' README.md`. In whatever format is already there (table / list / prose), add a matching entry for `06-mount-real-confluence.sh`: "Mount a real Confluence space via FUSE and `cat` a page. Requires Atlassian creds (see `.env.example`)."
    2. If a "backends supported" quick-start block exists, add the Confluence equivalent:
       ```bash
       # Confluence
       export ATLASSIAN_API_KEY=... ATLASSIAN_EMAIL=... REPOSIX_CONFLUENCE_TENANT=...
       export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
       reposix list --backend confluence --project YOUR_SPACE_KEY --format table
       ```
    3. If a "decisions" or "design" section links to ADR-001, add a sibling link to ADR-002.

    **`docs/architecture.md`:**
    1. Locate the crate-topology section (diagram or bullets). Add `reposix-confluence` as a sibling of `reposix-github`, in whatever format is used.
    2. If there's a Mermaid diagram, render it via the mcp-mermaid tool after editing and eyeball for overlapping labels (per ~/.claude/CLAUDE.md OP #1).
    3. If a security section exists, add one line: "reposix-confluence follows the same SG-01 allowlist / SG-05 tainted-ingress discipline as reposix-github."

    **`docs/demos/index.md`:**
    Add rows for the two new demos in the file's existing table format. Example:
    ```
    | Tier 3B | parity-confluence.sh        | Sim vs real Confluence shape diff |
    | Tier 5  | 06-mount-real-confluence.sh | FUSE-mount a real Confluence space, cat a page, unmount |
    ```
    Adjust column count and format to match the existing table.

    **Final validation** — if `mkdocs.yml` exists at repo root and `mkdocs` is on PATH:
    ```bash
    mkdocs build --strict
    ```
    Otherwise skip; CI will catch issues.
  </action>
  <verify>
    <automated>! grep -qi 'teamwork' .env.example &amp;&amp; grep -q '^ATLASSIAN_API_KEY=' .env.example &amp;&amp; grep -q '^ATLASSIAN_EMAIL=' .env.example &amp;&amp; grep -q '^REPOSIX_CONFLUENCE_TENANT=' .env.example &amp;&amp; grep -q 'reposix-confluence' CHANGELOG.md &amp;&amp; grep -q '06-mount-real-confluence' README.md &amp;&amp; grep -q 'reposix-confluence' docs/architecture.md &amp;&amp; grep -qE 'parity-confluence|06-mount-real-confluence' docs/demos/index.md</automated>
  </verify>
  <done>
    `.env.example` renamed + expanded. CHANGELOG [Unreleased] filled. README lists new demo. Architecture includes new crate. Demos index includes new demos. Commit: `docs(11-E-3): update README, CHANGELOG, architecture, demos index, .env.example for Phase 11`.
  </done>
</task>

<task type="auto">
  <name>Task 4: Docs-site build check + mermaid render (if applicable)</name>
  <files>
    (validation only)
  </files>
  <action>
    If `mkdocs.yml` exists:
    ```bash
    mkdocs build --strict 2>&1 | tail -40
    ```
    If a Mermaid diagram was added to `docs/architecture.md`, render it via the mcp-mermaid tool and check for spaghetti edges, overlapping labels, unreadable node text (per ~/.claude/CLAUDE.md OP #1). Regenerate with adjusted node positions if any of these issues appear.

    If `mkdocs build --strict` rejects a link, fix the link (absolute GitHub URL if the target is outside `docs/`). Do not `--skip-strict`.
  </action>
  <verify>
    <automated>if [ -f mkdocs.yml ] &amp;&amp; command -v mkdocs &gt;/dev/null 2&gt;&amp;1; then mkdocs build --strict &gt;/dev/null; else echo "mkdocs not present; skipping"; fi</automated>
  </verify>
  <done>
    If mkdocs exists: `mkdocs build --strict` succeeds. If a Mermaid diagram was added: it renders cleanly. Otherwise: no-op.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Docs → user action | `.env.example` is a template; accidentally putting real values there and committing would be a credentials leak. The file is committed-as-a-template, `.env` is gitignored. |
| ADR-002 content → future agent | An ADR that misrepresents the implementation is worse than no ADR — it'll be obeyed and will lead the next agent astray. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11E-01 | Information disclosure | `.env.example` accidentally containing real creds | mitigate | `.env.example` values are always empty (`FOO=` with no value). `git diff .env.example` before commit — if any value is non-empty, reject the commit. |
| T-11E-02 | Tampering | ADR-002 drifting from lib.rs over time | mitigate | ADR-002 cites `crates/reposix-confluence/src/lib.rs` in the References section. Future agents reading the ADR are pointed at the code as the source of truth. Code-review skills check ADR ↔ code consistency. |
| T-11E-03 | Repudiation | CHANGELOG entry missing a security-relevant change | mitigate | "Changed" section lists the env-var rename explicitly (it's a breaking change for anyone with `TEAMWORK_GRAPH_API` in their shell). v0.3 release notes (11-F) will re-surface this. |

Block-on-high: T-11E-01 — add a commit-time check. `git diff --cached .env.example | grep -E '^[+].*=.+' | grep -v '#' | head -1` should return empty (any added line with `=` followed by a non-comment non-empty value is suspicious).
</threat_model>

<verification>
Nyquist coverage:
- **File existence + key content:** covered by Task 1 / 2 / 3 verify commands (grep assertions).
- **Line-count lower bounds:** ADR ≥80, reference ≥60. Covered in verify.
- **TEAMWORK_GRAPH_API purged:** `! grep -qi teamwork .env.example` in Task 3.
- **No secrets in .env.example:** covered by T-11E-01 mitigation (commit-time check).
- **Docs build:** Task 4 runs `mkdocs build --strict` if the tool is available.
- **Regression:** None — this plan modifies only docs/config files; `cargo test --workspace` is unaffected.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f docs/decisions/002-confluence-page-mapping.md` returns 0.
2. `grep -q '^# ADR-002' docs/decisions/002-confluence-page-mapping.md` returns 0.
3. `[ "$(wc -l < docs/decisions/002-confluence-page-mapping.md)" -ge 80 ]` returns 0.
4. `grep -qE '(Lost metadata|lost metadata)' docs/decisions/002-confluence-page-mapping.md` returns 0.
5. `test -f docs/reference/confluence.md` returns 0.
6. `[ "$(wc -l < docs/reference/confluence.md)" -ge 60 ]` returns 0.
7. `! grep -qi teamwork .env.example` returns 0.
8. `grep -q '^ATLASSIAN_API_KEY=' .env.example` returns 0.
9. `grep -q '^ATLASSIAN_EMAIL=' .env.example` returns 0.
10. `grep -q '^REPOSIX_CONFLUENCE_TENANT=' .env.example` returns 0.
11. `grep -q 'reposix-confluence' CHANGELOG.md` returns 0.
12. `sed -n '/## \[Unreleased\]/,/## \[/p' CHANGELOG.md | grep -q '### Added'` returns 0 (Unreleased has an Added block).
13. `grep -q '06-mount-real-confluence' README.md` returns 0.
14. `grep -q 'reposix-confluence' docs/architecture.md` returns 0.
15. `grep -qE 'parity-confluence|06-mount-real-confluence' docs/demos/index.md` returns 0.
16. `if [ -f mkdocs.yml ] && command -v mkdocs >/dev/null 2>&1; then mkdocs build --strict >/dev/null; fi` returns 0.
17. `! git diff --cached .env.example 2>/dev/null | grep -E '^\+[A-Z_]+=.+' | grep -v '^\+#'` returns 0 (no real values added; safe template).
</success_criteria>

<rollback_plan>
If `mkdocs build --strict` rejects a link:
1. Absolute GitHub URL for any target outside `docs/`: `[parity demo](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/parity-confluence.sh)`.
2. Inside `docs/`, use relative paths: `[ADR-002](../decisions/002-confluence-page-mapping.md)`.
3. Re-run `mkdocs build --strict` until green.

If the CHANGELOG `## [Unreleased]` section was already populated by another agent between 11-A and 11-E:
1. Don't replace — append the Phase 11 "Added" + "Changed" content underneath any existing entries.
2. Keep the section atomic: one "### Added" and one "### Changed" header, not duplicates.

If README's Tier 5 section has been reorganized since this plan was written:
1. Locate the new home (`grep -rn 'Tier 5' README.md`) and add the Confluence row in that location.
2. Keep format consistent with whatever's already there.

If `.env.example` was restructured:
1. Drop the Atlassian block in the right section (after GitHub, before allowlist) — do not force-fit it into an old layout.
2. The critical invariants: no `TEAMWORK_GRAPH_API`; all three new vars present; all empty values.
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-E-SUMMARY.md` with:
- List of the seven files touched with a one-line summary of each change.
- Confirmation that `! grep -qi teamwork .env.example` passes.
- If a Mermaid diagram was added, a screenshot path + one-line visual assessment.
- If `mkdocs build --strict` was run, the final line of output.
</output>
