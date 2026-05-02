← [back to index](./index.md)

# 9. Confidence Assessment & Open Questions

| Area | Confidence | Notes |
|------|------------|-------|
| Wire protocol mechanics | HIGH | Verified directly against `gitremote-helpers(7)` upstream docs and felipec/git-remote-hg source. |
| `import`/`export` vs `fetch`/`push` choice | HIGH | Spec explicitly recommends `import` for non-git remotes; multiple precedents (hg, bzr, mediawiki). |
| Fast-import format details | HIGH | Quoted directly from `git-fast-import(1)`. |
| Marks-based incremental sync | MEDIUM-HIGH | Pattern is clear; the corner case of "marks file is corrupt" needs a recovery story (probably: delete marks → re-import everything, with a stern stderr warning). |
| Real merge conflicts via `import` | MEDIUM | Mechanism is sound but depends on deterministic blob rendering; needs a CI test that round-trips the same logical state through 10 imports without producing a divergent SHA. |
| Async-from-sync bridge | HIGH | Standard tokio pattern; same as planned for `reposix-fuse`. |
| Authentication scheme | MEDIUM | Env-var-priority approach is conventional but the per-alias namespacing decision should be reviewed against any existing reposix CLI conventions. |
| git-bug as a precedent | HIGH | Explicitly does NOT use this protocol; we're choosing the harder path deliberately for the agent UX win. Should be documented in `Key Decisions`. |

### Open questions to resolve in implementation

1. **Should the helper also implement `connect`?** No — REST is not git-pack. But if we ever want `git ls-remote reposix::...` to work fast, we can advertise `connect` and refuse it (returning `fallback`), which is cheaper than instantiating a full `import`. Decide during implementation.
2. **How do we surface API rate-limit headers (X-RateLimit-Remaining, etc.) to the agent?** Probably as warnings on stderr when remaining < 10% of limit. Worth exposing so the agent learns to back off.
3. **What about `git fetch` of a non-existent remote ref?** Spec says: respond to `list` without that ref; git will report `[no such ref]`. No special handling needed.
4. **Multi-process safety on the marks file.** If two `git push` invocations run concurrently against the same alias, marks file races. Solution: `flock(LOCK_EX)` on the marks file across the entire helper invocation. Simple and matches what fast-import itself does.
