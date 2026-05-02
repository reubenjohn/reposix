[index](./index.md)

# 7. Risks and Open Questions

## Confirmed risks with mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **Lazy-fetch fan-out.** `git cat-file -p` triggers one helper process per blob. `git grep` across the working tree triggers one per missing blob. | Medium | Sparse-checkout batching (one RPC per checkout, not per blob). Blob limit enforcement refuses bulk fetches with an actionable error message. Agent pre-warm via `git fetch --filter=blob:none` before bulk operations. |
| **`stateless-connect` documented as "experimental, for internal use only."** | Low | Stable in practice since git 2.21 (2019). Used internally by `git-remote-http`. No breaking changes in 7 years. |
| **Minimum git version requirement.** | Low | `>= 2.27` for full `filter` support over protocol v2. `>= 2.34` to be safe (includes partial-clone improvements). Pin in `CLAUDE.md` and `README`. CI must test on git >= 2.34 (alpine:latest has 2.52). |

## Open questions

1. **Cache eviction policy for `reposix-cache`.** The local bare-repo cache grows as more blobs are fetched. Options: LRU eviction (complex with git's object store), TTL-based re-fetch (simpler), per-project disk quota, or manual `reposix gc` command. Decision deferred to implementation planning.

2. **Atomicity of REST write + bare-repo-cache update.** If the REST POST succeeds but the local cache update fails, we get divergence. Preferred ordering: bare-cache-first, then REST (rollback = `git update-ref refs/heads/main <old>`). A background reconciler may be needed for edge cases.

3. **Threat model update for push-through-export flow.** The helper is the only component authorized to emit REST writes, which is correct for the threat model. But the push path means the helper must validate every commit's content against the frontmatter field allowlist before translating to REST. `research/threat-model-and-critique.md` needs an update.

4. **Non-issue files in push.** If an agent pushes changes to files outside issue-tracking paths (e.g., `.planning/` files), the helper must decide: reject, or silently commit to the bare cache only (nothing flows to REST). The latter is more consistent with "the repo is a real git remote."

5. **`import` deprecation timeline.** The `import` capability is redundant once `stateless-connect` handles all fetch paths. Keep for one release cycle (v0.10), then remove.

6. **Stream-parsing performance for export.** The POC reads the fast-import stream in memory. A production helper should use a state-machine parser that streams REST writes as it goes, with a commit-or-rollback barrier at the `done` terminator.
