# Origin-of-truth frontmatter enforcement

[← index](./index.md)

**Problem.** v0.13.0's bus remote handles "one ISSUES backend (SoT) + one plain-git mirror." That topology has only one place writes can land — confluence (or JIRA, or GH Issues, whichever is configured as SoT) — and the mirror is read-only from the SoT's perspective. There's no way to misroute a write because there's only one writable target.

The next topology — fanning out across **two ISSUES backends** (e.g., GH Issues + JIRA simultaneously, with confluence as a third reference SoT) — opens the misrouting failure mode. A record originally created in JIRA gets edited locally, gets pushed via a bus that fans out to both JIRA and GH Issues, and the helper has no signal that the record's "home" is JIRA. If the GH Issues fan-out succeeds and JIRA fails, the record now exists in two backends with conflicting versions and no clear winner.

**Sketch.** A frontmatter field that records the record's origin backend at create time and is checked at every push:

```yaml
---
id: 42
origin_backend: jira
title: ...
---
```

- The `BackendConnector::create_record` impl writes `origin_backend: <self.name>` into frontmatter at creation.
- The bus remote's push-time conflict detection extends with: *"for every record being written, check that `origin_backend` matches the target backend; if not, reject with `error: record 42 originated in jira, cannot push to gh-issues. To migrate origin, use `reposix migrate-origin <id> <new-backend>` (see docs)."*
- A new `reposix migrate-origin <id> <new-backend>` subcommand handles the legitimate migration case (e.g., decommissioning JIRA in favor of GH Issues): rewrites the field, audits the migration, requires confirmation flag.
- The frontmatter field allowlist (CLAUDE.md "Threat model" → "Frontmatter field allowlist") extends to treat `origin_backend` as server-controlled. Clients cannot rewrite it via `git push`; the migrate command is the only legitimate path.

**Why v0.14.0, not later.** Multi-backend bus is the natural follow-on once DVCS ships and one team starts wanting JIRA + GH Issues simultaneously. Without origin-of-truth enforcement, the first such deployment hits silent data corruption. v0.14.0 lands the guardrail BEFORE the multi-backend bus generalizes from 1+1 to 2+1 in a future milestone.

**Success gate.**
- `BackendConnector::create_record` impls in sim, confluence, GH Issues, JIRA all stamp `origin_backend`.
- Bus remote rejects mismatched pushes with the error message above; integration test covers the JIRA-record-pushed-to-GH-Issues failure path.
- `reposix migrate-origin` ships with audit row + dry-run mode + confirmation flag.
- Doc page `docs/concepts/origin-of-truth.md` explains the model, when migration is appropriate, and the failure modes the field protects against.

**Open questions for the planner.**

- `Q-OOT.1` Backfill policy for records that predate the field. Probably: helper assumes the SoT-of-record is whichever backend the record currently exists in; ambiguous if it exists in two. Migration tool's first run does a one-shot stamp.
- `Q-OOT.2` What happens when a record is intentionally cross-posted (rare but real — a PM wants the same issue tracked in JIRA and GH Issues for different audiences)? Probably: cross-posting is a "two records with the same content, different IDs" pattern, not a "one record with two origins" pattern. Document explicitly.
- `Q-OOT.3` Does the field interact with `Tainted<T>`? The value is server-controlled (set by `create_record`), so it's untainted by construction. But it IS read from frontmatter on push, so the read path needs to validate it didn't get edited offline. Treat the field as part of the existing server-controlled-fields allowlist.
