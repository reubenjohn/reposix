# 7. Comparison with Current FUSE Design

← [back to index](./index.md)

| Aspect | FUSE (current) | Git-native (proposed) |
|--------|---------------|----------------------|
| **First read** | Live API call (~500ms) | Blob fetch + cache (once, ~500ms) |
| **Subsequent reads** | Live API call (~500ms) | Local file read (~1ms) |
| **Directory listing** | Live API call (~200ms) | Local tree (~1ms) |
| **Write** | FUSE write handler -> immediate API call | `git push` -> batch REST writes |
| **Write batching** | None (one API call per write syscall) | Natural (all changes in one commit = one push) |
| **Conflict detection** | None (last write wins) | Push-time version comparison |
| **Offline capability** | None (all ops require network) | Full (read/write/commit offline, push when online) |
| **Agent learning** | None needed | None needed (P2: learns from errors) |
| **Change tracking** | None (no diff capability) | `git diff` / `git log` show full history |
| **Dependencies** | fuser crate, fusermount3, /dev/fuse, Linux only | git >= 2.27 (everywhere git runs) |
| **Platform support** | Linux only (WSL2 quirky, macOS via macFUSE) | Linux, macOS, Windows, WSL2 |
| **Concurrent access** | Race conditions on overlapping writes | Git merge semantics (well-understood) |
| **Rollback** | None | `git revert`, `git reset` |

The git-native model is strictly superior for the agentic use case. The FUSE model's only advantage -- transparent filesystem integration without any git awareness -- is irrelevant when the consumer is an LLM agent that already knows git.
