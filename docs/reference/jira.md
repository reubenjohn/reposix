# JIRA Cloud Integration

`reposix-jira` implements the `BackendConnector` trait against JIRA Cloud REST v3
(`https://{instance}.atlassian.net/rest/api/3`), exposing JIRA issues as POSIX files.

## Setup

### Required Environment Variables

| Variable | Description |
|----------|-------------|
| `JIRA_EMAIL` | Atlassian account email (must match the API token owner) |
| `JIRA_API_TOKEN` | API token from [id.atlassian.com/manage-profile/security/api-tokens](https://id.atlassian.com/manage-profile/security/api-tokens) |
| `REPOSIX_JIRA_INSTANCE` | JIRA Cloud instance subdomain (e.g. `mycompany` for `mycompany.atlassian.net`) |

### Egress Allowlist

You must also permit outbound HTTPS to your JIRA instance:

```bash
export REPOSIX_ALLOWED_ORIGINS="https://mycompany.atlassian.net"
```

`REPOSIX_ALLOWED_ORIGINS` defaults to `http://127.0.0.1:*` (simulator-only). Any real
backend requires an explicit entry.

## Usage

### List Issues

```bash
reposix list --backend jira --project MYPROJECT
```

`--project` is the JIRA project key (e.g. `MYPROJECT`, not the numeric space id).

### Bootstrap as a partial-clone working tree

```bash
reposix init jira::MYPROJECT /tmp/jira-mount
```

The working tree is a real partial-clone git checkout; issues are exposed as
`issues/<id>.md` with YAML frontmatter and a plain-text body. Read access is
`cat` / `grep -r`; writes round-trip through `git push`. See the
[first-run tutorial](../tutorials/first-run.md) for the full flow against the
simulator (the only argument that changes for JIRA is the `init` spec).

## The `--no-truncate` Flag

By default, `list_records` returns at most 500 issues (5 paginated requests of 100 each).
For larger projects, pass `--no-truncate` to raise an error instead of silently capping:

```bash
reposix list --backend jira --project BIGPROJECT --no-truncate
```

This is equivalent to `ConfluenceBackend::list_records_strict` — it returns an error if
the project contains more than 500 issues, prompting you to filter with a more specific
JQL query (planned feature, Phase 29+).

## Authentication Notes

- Uses HTTP Basic auth: `Authorization: Basic base64(email:api_token)`
- The API token is NOT your Atlassian account password; generate one at the URL above
- The `api_token` is redacted in all log output and debug representations
- `JIRA_API_TOKEN` is never echoed in error messages

## Issue Frontmatter

Each mounted issue file has a frontmatter header:

```yaml
---
id: 10001
title: "Fix login bug"
status: InProgress
assignee: "Alice Smith"
labels: []
parent_id: ~
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-11-15T10:30:00Z
version: 1731666600000
extensions:
  jira_key: PROJ-42
  issue_type: Story
  priority: Medium
  status_name: "In Progress"
  hierarchy_level: 0
---
```

The `extensions` block contains JIRA-specific metadata not in the canonical schema.
`hierarchy_level` values: `-1` = subtask, `0` = standard issue, `1` = epic.

## Limitations

- **No strong versioning:** JIRA has no ETag; `version` is synthesized from `updated` timestamp.
- **No attachments:** JIRA attachments are deferred to a future phase.
- **No comments:** JIRA comments are deferred to a future phase.
- **Plain-text bodies:** ADF description is extracted as plain text. Markdown rendering is a future enhancement.
