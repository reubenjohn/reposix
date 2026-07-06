---
quick: 260706-crf
title: DVCS cold-reader fixes (7 findings across 3 docs, pre-v0.13.0 tag)
date: 2026-07-06
type: docs
---

# Quick: DVCS cold-reader fixes

Cold-reader hardening pass on three DVCS docs before the v0.13.0 tag. Additive/corrective
doc edits only; no Rust changes (code grepped only to verify findings 1 & 6).

## Findings

1. **BLOCKER (verify-first)** — `dvcs-topology.md` claimed `refs/mirrors/*` never reach the
   GH mirror; contradicted by `dvcs-mirror-setup.md` Step 4 + the workflow template's literal
   `git push mirror refs/mirrors/...`. Verify against code, then narrow the claim.
2. **HIGH** — three-roles table said writes are "atomic via the bus remote"; contradicted by
   the same doc's "not a true 2PC". Drop "atomic" from the SoT+mirror pair cell.
3. **HIGH** — troubleshooting duplicate-record Fix heading said "before you re-push" while the
   Symptom says the duplicate already exists. Reword for correct timing.
4. **MEDIUM** — internal phase/decision codes leaked into user-facing topology prose. Move to
   an HTML comment.
5. **MEDIUM** — "fine-grained PAT with `repo` scope" is invalid PAT vocabulary. Reword (Step 5
   + troubleshooting table).
6. **LOW (verify)** — duplicate-record section missing the `audit_events_cache` diagnostic.
   Confirm real op names, then add the query.
7. **LOW-nit** — `gh secret set` non-TTY caveat sat below the code block. Move above.

## Constraints
- Doc-only. One cargo invocation machine-wide (none needed).
- Gates seen PASS: banned-words, mkdocs-strict, mermaid-renders, cross-link anchor.
- Commit atomically, push to main before reporting.
