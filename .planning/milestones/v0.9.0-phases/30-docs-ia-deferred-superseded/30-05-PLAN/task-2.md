← [back to index](./index.md)

# Task 2: Carve trust-model.md from docs/security.md + add sentinel blocks to architecture.md/security.md

<task type="auto">
  <name>Task 2: Carve trust-model.md from docs/security.md + add sentinel blocks to architecture.md/security.md</name>
  <files>docs/how-it-works/trust-model.md, docs/architecture.md, docs/security.md</files>
  <read_first>
    - `docs/security.md` (source — all 99 lines; lethal-trifecta opener + SG-01..08 table + deferred items)
    - `docs/architecture.md` §"Security perimeter" (lines 225-254 — source diagram content)
    - `docs/how-it-works/trust-model.md` (current skeleton with placeholder mermaid — preserve mermaid fence verbatim)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §"docs/how-it-works/trust-model.md"
  </read_first>
  <action>
    **docs/how-it-works/trust-model.md** — replace the body while preserving the existing mermaid fence verbatim. Target structure:

```markdown
# The trust model

Every deployment of reposix is a textbook **lethal trifecta**[^1]: private data (tracker contents), untrusted input (attacker-influenced issue bodies), and an exfiltration channel (`git push`). reposix cuts one leg architecturally (egress allowlist), hardens another (taint typing on ingress), and accepts the third (sanitize-on-egress) as incurable without giving up the core feature. The audit log is the ledger.

## Lethal trifecta — where each leg lives here

Carve docs/security.md lines 5-11 (the numbered trifecta list):

1. **Private data.** The FUSE mount exposes issue bodies, custom fields, attachments. This is the feature — we do not encrypt the working tree, so an attacker who gains `uid=you` reads it.
2. **Untrusted input.** Every remote ticket / page is attacker-influenced text. Issue bodies can contain prompt-injection payloads, zero-width characters, UTF-8 homoglyphs, embedded URLs.
3. **Exfiltration channel.** `git push` can target any remote the agent chooses unless the allowlist says otherwise.

The cuts:

[Preserve the existing mermaid fence from the skeleton — the trust-perimeter flowchart LR with Tainted/Trusted/Egress subgraphs.]

## The eight guardrails

Carve the SG-01..08 table from docs/security.md lines 21-30 VERBATIM — do not reword, do not drop file:line evidence:

| ID | Mitigation | Evidence | Test |
|----|-----------|----------|------|
| SG-01 | Outbound HTTP allowlist (`REPOSIX_ALLOWED_ORIGINS`) | `crates/reposix-core/src/http.rs` sealed `HttpClient` newtype + per-request URL recheck | `crates/reposix-core/tests/http_allowlist.rs` (7 tests) |
| SG-02 | Bulk-delete cap (>5 deletes refused; `[allow-bulk-delete]` overrides) | `crates/reposix-remote/src/diff.rs::plan` | `crates/reposix-remote/tests/bulk_delete_cap.rs` (3 tests) |
| SG-03 | Frontmatter strip (server-controlled fields removed before save) | `crates/reposix-fuse/src/frontmatter.rs::sanitize_inbound` | `crates/reposix-fuse/tests/frontmatter.rs` |
| SG-04 | Path validator (filename allowlist at FUSE boundary) | `crates/reposix-fuse/src/fs.rs::validate_issue_filename` | `crates/reposix-fuse/tests/validate.rs` |
| SG-05 | Tainted typing (attacker-origin bytes carry `Tainted<T>`) | `crates/reposix-core/src/taint.rs` | inline doctests |
| SG-06 | Append-only audit log (no UPDATE/DELETE on audit table) | `crates/reposix-sim/src/audit.rs` | integration tests in `crates/reposix-sim/tests/` |
| SG-07 | 5-second EIO timeout (dead backend cannot hang kernel) | `crates/reposix-fuse/src/fetch.rs` | `crates/reposix-fuse/tests/timeout.rs` |
| SG-08 | Demo recording shows guardrails firing on camera | `scripts/demos/full.sh` step 8a/8b | `scripts/demos/full.sh` transcript |

(Preserve every file:line pointer — these are evidence, not ornament. If docs/security.md differs in any row, docs/security.md wins — treat this plan's table as transcription only.)

## What's deferred

Carve the deferred-items list from docs/security.md (the last ~10 lines of the file — everything after SG-08 table). Do NOT add new claims here; if security.md lists 3 deferred items, this page lists 3.

[^1]: [Simon Willison, "The lethal trifecta for AI agents"](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/), revised April 2026.
```

**docs/architecture.md** — prepend a sentinel admonition block to the file (do NOT delete; Wave 3 plan 30-08 handles deletion after link audit):

```markdown
!!! warning "This file has been carved into `docs/how-it-works/` and is scheduled for deletion"
    Content migrated during Phase 30 (v0.9.0):
    - Read path / Write path / async bridge → [The filesystem layer](how-it-works/filesystem.md)
    - `git push` round-trip / Optimistic concurrency → [The git layer](how-it-works/git.md)
    - Security perimeter → [The trust model](how-it-works/trust-model.md)

    This file will be deleted in Wave 3 (plan 30-08) after an internal-link grep-audit confirms no dangling references. If you are reading this and the deletion has not happened, the audit must still be pending.

---

```

The sentinel block goes BEFORE the current H1 (`# Architecture`). Preserve the rest of the file verbatim — do not edit any existing architecture.md prose or diagrams here.

**docs/security.md** — prepend a sentinel admonition block (same pattern):

```markdown
!!! warning "This file has been carved into `docs/how-it-works/trust-model.md` and is scheduled for deletion"
    Content migrated during Phase 30 (v0.9.0):
    - Lethal trifecta + SG-01..08 table + deferred items → [The trust model](how-it-works/trust-model.md)

    This file will be deleted in Wave 3 (plan 30-08) after an internal-link grep-audit confirms no dangling references.

---

```

The sentinel block goes BEFORE the current H1 (`# Security`). Preserve the rest verbatim.

Both sentinels use `!!! warning` (not `note`) so they visually distinguish from stub markers elsewhere.

After writing, run:

```bash
mkdocs build --strict 2>&1 | tee /tmp/mkdocs-30-05.log
# expected: exit 0. Both architecture.md and security.md are in not_in_nav (set by plan 30-04), so they won't appear in the sidebar but still build.

grep -c 'SG-0[1-8]' docs/how-it-works/trust-model.md
# expected: 8 (all eight guardrails rows present)

grep -c 'scheduled for deletion' docs/architecture.md docs/security.md
# expected: 1 per file (2 total)
```
  </action>
  <verify>
    <automated>test -f docs/how-it-works/trust-model.md && grep -c '```mermaid' docs/how-it-works/trust-model.md | grep -q '^1$' && grep -c 'SG-0[1-8]' docs/how-it-works/trust-model.md | awk '{exit !($1 >= 8)}' && grep -q 'lethal trifecta' docs/how-it-works/trust-model.md && grep -c 'scheduled for deletion' docs/architecture.md | grep -q '^1$' && grep -c 'scheduled for deletion' docs/security.md | grep -q '^1$' && grep -c '^# Architecture$' docs/architecture.md | grep -q '^1$' && mkdocs build --strict</automated>
  </verify>
  <acceptance_criteria>
    - trust-model.md has exactly 1 mermaid fence.
    - `grep -c 'SG-0[1-8]' docs/how-it-works/trust-model.md` returns `>= 8` (all eight SG rows present).
    - `grep -c 'crates/reposix-core/src/http.rs' docs/how-it-works/trust-model.md` returns `>= 1` (SG-01 evidence).
    - `grep -c 'crates/reposix-remote/src/diff.rs' docs/how-it-works/trust-model.md` returns `>= 1` (SG-02 evidence).
    - `grep -c 'lethal trifecta' docs/how-it-works/trust-model.md` returns `>= 1`.
    - `grep -c '^\[\^1\]:' docs/how-it-works/trust-model.md` returns `1` (footnote preserved from skeleton).
    - `grep -c 'scheduled for deletion' docs/architecture.md` returns `1` (sentinel present).
    - `grep -c 'scheduled for deletion' docs/security.md` returns `1` (sentinel present).
    - `grep -B1 '^# Architecture$' docs/architecture.md | grep -q -- '---'` (sentinel block before H1, separated by `---` horizontal rule).
    - `wc -l docs/how-it-works/trust-model.md` reports `>= 80`.
    - `mkdocs build --strict` exits 0.
  </acceptance_criteria>
  <done>
    trust-model.md carved from security.md with SG-01..08 table verbatim, lethal-trifecta opener, and one mermaid diagram. architecture.md + security.md carry sentinel blocks pointing readers at how-it-works/. Wave 3 can now safely delete the sources.
  </done>
</task>
