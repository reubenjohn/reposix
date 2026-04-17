# Phase 28: JIRA Cloud read-only adapter — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-16
**Phase:** 28-jira-cloud-read-only-adapter-reposix-jira-v0-8-0
**Mode:** Autonomous (spec fully defined in ROADMAP.md — no interactive discussion required)
**Areas discussed:** API Endpoints, Status Mapping, ADF Stripping, extensions keys, Security Contract, Rate Limiting, CLI/Env Vars, Test Matrix, Documentation

---

## Note on Discussion Mode

Phase 28 has a complete, detailed specification in `.planning/ROADMAP.md` (lines ~706–731) that pre-answers all implementation decisions. This was an autonomous `/gsd-next` invocation with no user present. All decisions in CONTEXT.md were derived directly from the ROADMAP spec — no gray areas required interactive resolution.

Key decisions extracted from spec:
- POST /rest/api/3/search/jql (not old GET /search, retired Aug 2025)
- cursor pagination via nextPageToken + isLast (total absent)
- Two-field status mapping (statusCategory.key primary, resolution.name override)
- ADF plain-text stripping (not markdown conversion)
- 5 extensions keys: jira_key, issue_type, priority, status_name, hierarchy_level
- 12 required wiremock tests (enumerated)
- read_jira_env pattern matching read_confluence_env_from

## Claude's Discretion

- Internal module layout within `reposix-jira`
- Error message wording for validation failures
- ADF whitespace handling for multi-paragraph bodies

## Deferred Ideas

- ADF-to-Markdown (rich output) — Phase 29+
- JIRA attachments and comments — post Phase 28
- Write path — Phase 29
- Shared ADF library crate — post Phase 29 refactor
