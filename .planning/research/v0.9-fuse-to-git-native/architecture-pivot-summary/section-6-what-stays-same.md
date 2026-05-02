[index](./index.md)

# 6. What Stays the Same

The pivot is a transport-layer change. The application layer is preserved:

- **`BackendConnector` trait** (`crates/reposix-core/src/backend.rs`) and all backend implementations. These are the REST API adapters. They move from being called by FUSE callbacks to being called by `reposix-cache`.
- **Audit log** (SQLite WAL, append-only, no UPDATE/DELETE on the audit table). The helper continues to write a row for every network-touching action.
- **Tainted-by-default policy.** Any byte from a remote (including the simulator) is tainted. Tainted content must not be routed into actions with side effects on other systems.
- **Allowed-origins egress allowlist** (`REPOSIX_ALLOWED_ORIGINS`). The helper and cache crate refuse to talk to any origin not in the allowlist. Default: `http://127.0.0.1:*`.
- **Frontmatter field allowlist.** Server-controlled fields (`id`, `created_at`, `version`) are stripped on the inbound path before serialization.
- **Simulator as default/testing backend.** All development, unit tests, and autonomous agent loops use the simulator. Real backends require explicit credentials and a non-default allowlist.
- **`reposix-core` shared types** (`Issue`, `Project`, `RemoteSpec`, `Error`).
- **Issue serialization format** (YAML frontmatter + Markdown body, via `serde_yaml` 0.9).
