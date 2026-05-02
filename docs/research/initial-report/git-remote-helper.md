# Distributed Synchronization: The Git Remote Helper Protocol

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here are prospective designs from the v0.1-era research, not a description of the current implementation.

While FUSE provides the local POSIX interface, an agent requires a mechanism to manage state, track history, and synchronize changes with the upstream application. Simply editing a FUSE-mounted file triggers synchronous network requests, which can lead to high latency and trigger API rate limits. A superior architectural design separates the local workspace from the remote synchronization layer by utilizing Git as the intermediate state manager.

## Internal Mechanics of Git Remote Helpers

Git is fundamentally designed to interact with remote repositories using protocols like SSH, HTTP, and Git's native protocol. However, the architecture includes an extensibility mechanism known as `git-remote-helpers`, which permits Git to communicate with arbitrary remote systems. When an agent executes a command targeting a custom protocol—for instance, `git fetch jira::https://company.atlassian.net`—Git detects the `jira::` prefix and invokes an executable named `git-remote-jira` located in the system's execution path.

This remote helper acts as a translation layer between the remote REST API and the local Git object database. The Git object database models a repository using four primary object types: blobs (file contents), trees (directory structures), commits (snapshots of trees with metadata), and tags. The `remote.h` internal API allows the helper to access remote configuration variables, including `pushurl_nr` (the quantity of push URLs) and `fetch_refspec` (the rules for mapping remote branches to local tracking branches). The helper communicates with the core Git process via standard input and standard output, receiving commands like `list` to discover remote branches and `fetch` to download specific object data.

## Translating API State to Git Objects

To synchronize an issue tracker like GitHub Issues or Jira, the remote helper must pull the current state of the issues via REST API GET requests and dynamically construct corresponding Git blobs and trees. Each issue is instantiated as a Markdown file, and the entire collection is hashed and packaged into a commit representing the current state of the remote project.

When the agent finishes resolving a bug, it executes a `git commit` and `git push`. The remote helper intercepts the push command and analyzes the resulting Git delta. If a new file was created in the local repository, the helper translates this into a POST request to the Jira API to create a new ticket. If an existing file was modified, the helper extracts the differences and issues the appropriate PATCH or PUT request to update the remote issue. This asynchronous synchronization model isolates the agent from network latency during the editing phase and ensures that all state changes are batched into cohesive transactions.

## The Git-Bug Paradigm and Lamport Timestamps

The concept of representing issues within Git is highly refined in open-source projects like `git-bug`, an offline-first bug tracker embedded directly within the Git repository structure. `git-bug` operates by storing bug reports as specific Git objects independent of the primary source code branches. A critical architectural feature of `git-bug` is its implementation of custom bridges to external trackers like Jira, GitLab, and GitHub.

Instead of forcing a rigid hierarchical synchronization that is prone to conflict, `git-bug` utilizes Lamport timestamps—a form of logical clock—to handle the time-based ordering of operations. When an agent or human adds a comment or modifies an issue's state, the action is recorded as an immutable operation associated with a logical timestamp. Because these operations are sequentially appended rather than destructively overwritten, concurrent modifications rarely result in standard merge conflicts. When the remote helper executes a push or pull against the Jira bridge, the operations are serialized and applied in their correct logical order, guaranteeing consistency across distributed instances without requiring complex, real-time locking mechanisms.
