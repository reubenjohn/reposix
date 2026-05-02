# The REST-to-POSIX Impedance Mismatch

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here are prospective designs from the v0.1-era research, not a description of the current implementation.

A central challenge in designing an agentic filesystem lies in the fundamental paradigm conflict between RESTful web services and POSIX filesystems. REST APIs are resource-oriented, utilizing non-hierarchical unique identifiers (e.g., `/v3/application/listings/{listing_id}`) and distinct HTTP verbs (GET, POST, PUT, PATCH, DELETE). Conversely, POSIX filesystems rely on strict directory hierarchies and native file operations (read, write, append, truncate). Resolving this impedance mismatch is paramount for ensuring that the FUSE and remote helper components accurately interpret the agent's intent.

## Modeling Hierarchical Directory Structures

To provide a navigable structure for the AI agent, flat REST APIs must be modeled using the Composite software design pattern, wherein abstract nodes are instantiated as either files or directories. A singleton filesystem manager must hold the root directory and evaluate paths during traversal.

| API Concept | POSIX Filesystem Representation | Functional Rationale |
| --- | --- | --- |
| Workspace / Instance | Root Directory (e.g., `/mnt/enterprise/`) | Establishes the highest-level boundary for the FUSE mount point. |
| Project / Repository | Top-level Directory (e.g., `/mnt/enterprise/PROJ-X/`) | Allows agents to scope their `grep` and `find` operations to specific domains. |
| Issue / Ticket | Markdown File (e.g., `PROJ-123.md`) | Enables text manipulation using standard Unix stream editors. |
| Issue Metadata | YAML Frontmatter block within the Markdown file | Provides structured, schema-bound fields (status, assignee, priority) while keeping the content contiguous. |
| Comments and History | Appended text blocks or sub-directories | Maintains a chronological audit trail that the agent can read sequentially. |
| Attachments | Binary files in dedicated sub-directories | Permits the agent to process images, PDFs, or system logs natively without downloading them via API. |

By embedding the issue metadata in YAML frontmatter, the agent interacts with structured data organically. If the agent needs to reassign a ticket, it simply uses a tool like `sed` to replace the `assignee:` line in the text file.

## Differentiating HTTP Verbs Through File Semantics

Translating a filesystem write operation into the correct HTTP verb requires context-aware heuristics within the Git remote helper. If an agent modifies `PROJ-123.md`, the system must determine whether to execute a PUT request (which replaces the entire resource) or a PATCH request (which performs a partial update).

The integration with Git natively solves this ambiguity. When the agent executes `git commit`, the resulting `git diff` isolates the exact lines modified. If the diff reveals that only the `status` field within the YAML frontmatter was altered, the remote helper isolates this key-value pair and constructs a highly targeted `PATCH /rest/api/3/issue/PROJ-123` request containing only the modified field.

Deletion presents a unique edge case. While the HTTP DELETE method is intended to remove a resource, enterprise APIs frequently require accompanying metadata, such as a deletion reason or a resolution code. If an agent simply executes `rm PROJ-123.md`, the system lacks the context to provide this metadata. To accommodate this, the remote helper can be programmed to parse the agent's commit message. If the commit message accompanying the deletion specifies `git commit -m "Closing ticket because issue cannot be reproduced"`, the remote helper intercepts this message, appends it as a JSON payload body to the DELETE request, and satisfies the API's requirements.
