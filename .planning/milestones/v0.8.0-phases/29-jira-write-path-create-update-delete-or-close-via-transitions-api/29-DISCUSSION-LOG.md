# Phase 29: JIRA write path — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-16
**Phase:** 29 — jira-write-path-create-update-delete-or-close-via-transitions-api
**Mode:** Power/autonomous — ROADMAP spec was authoritative; no interactive Q&A needed
**Areas discussed:** ADF write encoding, ADF read path upgrade, create_issue flow, update_issue flow, delete_or_close transitions strategy, BackendFeature changes

---

## ADF Body Encoding for Writes

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal paragraph wrapper | `{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"..."}]}]}` | ✓ |
| Full Markdown-to-ADF | Convert Issue.body Markdown to structured ADF (headings, lists, code blocks) | |

**Selected:** Minimal paragraph wrapper
**Notes:** ROADMAP spec explicitly calls this out. Full Markdown-to-ADF deferred — write tests only need plain text round-trips. The minimal wrapper is what JIRA renders correctly for programmatic issue creation.

---

## ADF Read Path Upgrade

| Option | Description | Selected |
|--------|-------------|----------|
| Keep plain text | Leave `adf_to_plain_text` as-is | |
| Upgrade to Markdown (copy-adapt) | Add `adf_to_markdown` to `reposix-jira/src/adf.rs`, no cross-crate dep | ✓ |
| Cross-crate dep on reposix-confluence | Import `adf_to_markdown` from confluence crate | |

**Selected:** Upgrade to Markdown (copy-adapt)
**Notes:** Write tests need round-trip verification. Plain text → ADF → Markdown should produce equivalent content. Cross-crate dep avoided (circular risk, added compile dependency). Copy-adapt is ~50 lines and matches the established pattern.

---

## `delete_or_close` Transition Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Transitions API only (no DELETE fallback) | Fail if no done transitions found | |
| Transitions API + DELETE fallback | As per ROADMAP spec — DELETE if no transitions, log WARN | ✓ |
| DELETE always | Skip transitions discovery entirely | |

**Selected:** Transitions API + DELETE fallback
**Notes:** ROADMAP spec is prescriptive. Fallback requires admin permission; logging WARN makes this visible.

---

## issuetype Cache Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| OnceLock per backend instance | Cache per session, lazy init on first create_issue | ✓ |
| Always re-fetch | Simple, no state | |
| Configurable TTL | Over-engineered for this phase | |

**Selected:** OnceLock per backend instance
**Notes:** "Cache per session" is explicit in ROADMAP spec.

---

## Claude's Discretion

- Internal struct shape for issuetype cache (`Vec<String>` vs typed struct)
- Whether to include `labels` in `create_issue` body (D-17 allows it)
- Error message wording for transition failures
- Exact `resolution.name` string in retry logic

## Deferred Ideas

- Full Markdown-to-ADF for rich descriptions — post-Phase 29
- `BackendFeature::Workflows` for named transitions
- Sub-task creation via parent_id
