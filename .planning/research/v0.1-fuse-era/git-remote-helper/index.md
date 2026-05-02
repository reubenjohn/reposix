# Research: Git Remote Helper Protocol for `git-remote-reposix`

**Researcher:** subagent (research mode: feasibility + ecosystem)
**Date:** 2026-04-13
**Confidence:** HIGH for protocol mechanics (Context-equivalent: official `gitremote-helpers(7)` man page); MEDIUM-HIGH for design recommendation; HIGH for OSS prior-art survey.

---

## Chapters

| # | Chapter | Summary |
|---|---------|---------|
| 1 | [TL;DR — Recommendation for reposix](./01-tldr.md) | Decision table: capability set, auth, error surface, conflict mode, async bridge; rationale for `import`/`export` over `fetch`/`push`/`connect`. |
| 2 | [The Wire Protocol — Verbatim](./02-wire-protocol.md) | Full protocol conversation: invocation, capabilities, list, import, export, option; spec-quoted semantics; stream terminators; `feature done`. |
| 3 | [Worked Example: One Issue Per File](./03-worked-example.md) | End-to-end trace of one `git push` closing an issue: setup, fast-import stream, helper diff loop, field-level PATCH, response handling, marks persistence. |
| 4 | [Real-World Implementations to Study](./04-real-world-implementations.md) | git-remote-hg (verbatim Python), git-bug (cautionary counter-example), git-remote-gcrypt (minimal Bash), git-remote-s3 (Rust prior art), kernel sources. |
| 5 | [Surfacing API Errors Back to the Agent](./05-error-surfacing.md) | stdout = protocol, stderr = human; Rust error pattern; example terminal output; the stdout-footgun pitfall. |
| 6 | [Handling `git pull` / Divergence](./06-merge-conflicts.md) | Three-way merge via `import`; requirements for deterministic blobs; marks-based incremental re-runs; pitfalls (reordering, whitespace, Markdown round-trips). |
| 7 | [Concrete Rust Skeleton](./07-rust-skeleton.md) | Crate layout, Cargo.toml, `main.rs` dispatch loop, `protocol.rs`, `caps.rs`, async-from-sync bridge with `tokio::runtime::Builder::new_current_thread()`. |
| 8 | [Authentication and Per-Remote Namespacing](./08-authentication.md) | Credential priority order; why alias not URL; anonymous one-shot URL (SHA1 alias); threat model mitigations (audit log, unconfigured-remote rejection, tainted marking). |
| 9 | [Confidence Assessment & Open Questions](./09-confidence.md) | Per-area confidence table; open questions: `connect`, rate-limit surfacing, non-existent refs, marks-file locking. |
| 10 | [Sources](./10-sources.md) | Authoritative (git-scm.com, kernel.org), reference implementations, background reading, project context files. |
