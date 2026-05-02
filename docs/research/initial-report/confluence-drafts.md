# Navigating Confluence Hierarchies and Draft Lifecycles

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here (Confluence draft-branching) are prospective designs from the v0.1-era FUSE pivot, not a description of the current `git-remote-reposix` implementation.

Integrating a comprehensive knowledge management system like Confluence introduces distinct challenges related to deeply nested hierarchies, binary attachment storage, and the ephemeral nature of draft documents.

## Directory Structuring and Attachment Algorithms

Confluence spaces are structured as trees of parent and child pages. This structure maps perfectly to nested POSIX directories, where each directory contains an `index.md` representing the page's core content alongside child directories for sub-pages.

Handling Confluence attachments requires specific algorithmic mapping to ensure efficiency. Confluence 8.1 introduced an optimized hierarchical file system for attachment storage based on the Content ID of the original attachment version. The storage layout algorithm calculates paths using a modulo arithmetic sequence to prevent directory overloading. The algorithm defines a domain as the Content ID modulo 65535. The first folder is the domain modulo 256, and the second folder is the domain divided by 256. The FUSE daemon replicates this logic to present attachments predictably. If an agent requires a system architecture diagram attached to a Confluence page, it simply accesses `attachments/12345678.1.png`, allowing it to pass the binary data to a multimodal vision model without executing complex API download orchestrations.

## The Draft State Lifecycle and Git Branching

Confluence heavily relies on collaborative editing and draft states. As a user creates or edits a page, Confluence automatically saves a draft every thirty seconds to prevent data loss. These drafts are distinct from published versions; a page possesses an `isNewPage()` flag indicating if it has ever been published, and draft entities are flagged separately from standard `ContentEntityObject` instances.

Exposing separate endpoints for drafts and published pages via an MCP server creates significant confusion for an autonomous agent. However, within a Git-backed filesystem, the concept of a draft directly parallels the concept of a Git branch. This alignment provides a native, intuitive workflow for the agent:

- **The Published State:** The `main` or `master` branch of the FUSE-mounted Git repository represents the definitive, published state of the Confluence space.
- **Initiating a Draft:** When an agent is tasked with creating a new architectural document, it executes a `git checkout -b draft/system-architecture`.
- **Continuous Saving:** As the agent generates content and frequently commits to this local branch, the `git-remote-helper` pushes these changes to the Confluence API targeting the `/draft` endpoints. This updates the Confluence draft state in the background without affecting the live, published page.
- **Finalizing and Publishing:** Once the agent completes its review, it executes a `git checkout main`, merges the draft branch, and pushes the result. The remote helper interprets this push to the `main` branch as a final publication event, triggering the Confluence API to convert the draft into a published version.

By mapping drafts to Git branches, the agent utilizes its deep, pre-trained understanding of version control mechanics, entirely sidestepping the proprietary intricacies of Confluence's state machine.
