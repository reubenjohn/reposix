# Governance, Authentication, and POSIX Permission Mapping

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here (POSIX RBAC mapping, AgentFS auditing) are prospective designs from the v0.1-era FUSE pivot, not a description of the current `git-remote-reposix` implementation.

A paramount concern when deploying autonomous agents is the enforcement of security, identity boundaries, and role-based access control (RBAC). FUSE architectures provide a unique mechanism for translating granular, cloud-based permissions into localized, operating system-level restrictions.

## Authentication and Identity Management

The proposed architecture requires the agent to authenticate via established, non-interactive protocols. This is achieved by provisioning a Personal Access Token (PAT) or a dedicated service account authenticated via OAuth 2.0. For platforms like GitHub and Jira, the token is securely passed to the Git remote helper via configuration files or environment variables. When interacting with systems that enforce multi-factor authentication, such as Google Keep, dedicated application passwords are utilized to bypass interactive prompts. Consequently, all actions performed by the FUSE layer are cryptographically attributed to the agent's specific identity, preserving accurate audit trails within the SaaS application's native logging infrastructure.

## Translating Cloud RBAC to POSIX Bitmasks

To prevent the agent from attempting unauthorized actions that would waste compute resources and API quotas before being rejected, the FUSE layer must dynamically translate cloud permissions into local UNIX file permissions (`chmod`/`chown`).

In Unix, file permissions are managed via read (`r`), write (`w`), and execute (`x`) bits. Applications like Jira and Confluence employ complex global and project-specific permission schemes. The FUSE daemon interrogates the respective platform's permission APIs to determine the agent's access level and applies the corresponding POSIX bitmask to the virtual files.

| Application Permission Set | Translated POSIX Permission | Local Behavioral Enforcement |
| --- | --- | --- |
| Jira: Browse Projects / Confluence: Assets Viewers | `r--` (Read-Only) | The agent can `cat` the file, but any `write()` or `append()` system call is immediately rejected by the Linux kernel with a *Permission denied* error. |
| Jira: Edit Issues / Confluence: Assets Managers | `rw-` (Read/Write) | The agent possesses standard modification rights, allowing it to alter file contents and save changes. |
| Jira: Administer Projects | `root` Ownership | The agent is granted full directory traversal and modification capabilities within the project scope. |

For example, if an agent lacking the "Edit Issues" permission attempts to execute `echo "fix implemented" >> PROJ-101.md`, the operation fails instantly at the local VFS layer. This immediate, localized feedback loop prevents the agent from assembling an elaborate JSON API payload only to have it rejected by the remote server, significantly optimizing the agent's decision-making cycle. For administrative permissions such as Jira's "Bulk change" capability—which allows destructive edits across thousands of issues—the FUSE daemon can globally restrict recursive write operations (`chmod -R -w`) to prevent a malfunctioning agent from causing widespread data corruption.

## AgentFS and Auditable Sandboxing

The overarching security posture is further enhanced by encapsulating the FUSE layer within an agent-specific filesystem framework like AgentFS. AgentFS operates as a SQLite-backed virtual filesystem that records every file operation, state change, and tool call into a structured write-ahead log (WAL). This architecture guarantees strict auditability, allowing human operators to query the agent's complete operational timeline using standard SQL commands. Furthermore, the single-file nature of the SQLite database enables exact reproducibility; an operator can snapshot the `agent.db` file at any point in time to analyze the specific sequence of FUSE operations that led to a particular decision or error.
