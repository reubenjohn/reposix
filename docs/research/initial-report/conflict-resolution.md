# Intelligent Conflict Resolution via Git Semantics

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here are prospective designs from the v0.1-era research, not a description of the current implementation.

The most substantial operational advantage of routing API interactions through a local Git repository is the complete offloading of conflict resolution from brittle, JSON-based error handling to robust, text-based Git semantics.

## The Brittleness of API Conflicts

In standard LLM integrations relying on direct REST calls or MCP, concurrent modifications pose a significant risk. If an agent attempts to submit a PUT request to update a Jira ticket's description, but a human engineer has modified the ticket seconds earlier, the API will reject the request with an HTTP 409 Conflict status. The agent must then parse the error, execute a new GET request to retrieve the updated state, hold both the new JSON and its intended changes within its context window, and attempt to syntactically synthesize a resolution. This procedure consumes vast amounts of tokens, often induces hallucination, and frequently results in the agent accidentally overwriting the human's changes.

## Native Git Conflict Resolution

When the external tracker is synchronized via a `git-remote-helper`, API conflicts are transformed into standard Git merge conflicts. If the remote Jira state diverges from the agent's local state, the agent's `git push` command is rejected with a standard error: *Updates were rejected because the remote contains work that you do not have locally.*

Because the agent is trained extensively on developer workflows, it intuitively executes a `git pull`. The local Git repository fetches the updated state from the remote helper and attempts a merge. When differences overlap on the same lines of the Markdown file, Git modifies the file to insert universal conflict markers (`<<<<<<< HEAD`, `=======`, `>>>>>>>`).

- **Contextual Isolation:** The agent can isolate the conflict using `cat` or `grep`, isolating exactly what changed without reloading the entire file schema into its context window.
- **Resolution Mechanics:** The agent utilizes standard Unix editors or inline tools like `sed` to selectively delete the conflict markers, preserve the human's additions, and integrate its own updates.
- **Advanced Tooling:** Agents are highly adept at utilizing advanced Git features to aid resolution. They can enable the `diff3` conflict style, which introduces a `||||||| merged common ancestors` block, allowing the agent to see the original baseline before either modification occurred.
- **Automated AI Merging:** Recent advancements in dedicated conflict resolution tools, such as MergeBERT and Git Merger AI, leverage specialized language models that excel at parsing these exact conflict blocks using robust regex engines. By framing API conflicts as Git conflicts, the overall system capitalizes on these heavily optimized code-merging capabilities, treating a conflict in a project management ticket exactly like a conflict in source code. Furthermore, the `git-rerere` (reuse recorded resolution) utility can be enabled to automatically resolve recurring conflicts if the agent frequently updates specific sections of a document.
