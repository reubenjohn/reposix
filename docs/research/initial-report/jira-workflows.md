# Advanced State Management: Jira Workflows and Custom Fields

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here (Jira workflow validation, custom field mapping) are prospective designs from the v0.1-era FUSE pivot, not a description of the current `git-remote-reposix` implementation.

The abstraction breaks down if the filesystem allows the agent to perform actions that the underlying application explicitly prohibits. Jira is notoriously complex, enforcing rigid workflows, proprietary data types, and required custom fields that complicate arbitrary file modification.

## Validating Workflow Transitions

In GitHub Issues, an issue state is generally binary: open or closed. Jira, however, governs state through strict workflows comprising specific statuses (e.g., *To Do*, *In Progress*, *In Review*, *Done*) and strictly defined directional transitions. A transition dictates the permissible paths an issue may take; an issue currently marked "Open" cannot be transitioned to "Done" if the workflow demands it first pass through "In Progress". Furthermore, Jira administrators configure conditions (which restrict who can perform a transition) and validators (which ensure specific input is provided before the transition succeeds).

If an agent blindly modifies the YAML frontmatter to `status: DONE` and attempts to push, the Jira API will return an error if a valid transition path does not exist. To proactively manage this, the FUSE daemon must execute a dynamic schema validation during the `read()` system call. When the agent opens the file, the daemon queries the `/rest/api/3/issue/{issueId}/transitions` endpoint to determine the currently available transitions. The daemon dynamically injects these valid states into the file as a non-destructive YAML comment:

```yaml
---
status: "IN PROGRESS"
# Valid next states:
assignee: "agent-alpha"
---
```

If the agent ignores this guidance and commits an invalid state, the `git push` operation will fail. The Git remote helper intercepts the Jira API HTTP 400 Bad Request response, formats the error text, and outputs it to the stderr stream of the shell. The agent, operating in a standard terminal loop, reads the standard error output, recognizes the workflow violation, amends the file to a valid state, and retries the push.

## Translating Custom Field Data Types

Jira's extensibility relies on custom fields, which the Atlassian Forge platform requires to be submitted in specific primitive or complex data types. Internal Jira IDs such as `customfield_10020` or `customfield_10035` are opaque and hostile to an AI agent attempting to infer context. The parser expects precise types: strings for raw text, nested maps for object types, user objects for assignments, and strictly formatted `CalendarDate` or `Date` strings for temporal data.

To solve this, the FUSE and remote helper components must act as a bidirectional translation layer. Utilizing configuration mapping files (analogous to the architecture in the `jira2md` repository), internal IDs are mapped to human-readable keys. `customfield_10006` becomes `story_points`, and `customfield_10000` becomes `sprint`. When the agent writes an integer to the `story_points` field in the YAML, the parser intercepts this, verifies it against the expected `Number` return type, and constructs the appropriate nested JSON structure required by the Atlassian API.
