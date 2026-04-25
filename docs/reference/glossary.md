---
title: Glossary — every term reposix expects you to know, in plain English
---

# Glossary

Every term that appears as jargon somewhere in the docs has a one-line gloss
here, plus the most authoritative external link we could find. If a page
introduces a term without a gloss, that's a bug — open an issue.

Terms are grouped by where they come from (git, sqlite, web standards,
project-specific) so you can skim by category.

---

## Git internals

### partial clone

A clone that fetches the tree but skips blob contents until you read them. reposix uses `--filter=blob:none` so `git fetch` returns directory structure and metadata in one round trip; file *contents* arrive lazily on first `cat` or `grep`.

External: [git-clone --filter docs](https://git-scm.com/docs/git-clone#Documentation/git-clone.txt---filterltfilter-specgt)

In reposix: foundational v0.9.0 architecture. `reposix init` runs `git fetch --filter=blob:none` so opening a 10 000-issue tree is constant-time, not O(N) network calls.

### sparse-checkout

A git mode that materializes only a subset of paths in your working tree. Other paths exist in the index but aren't on disk.

External: [git-sparse-checkout docs](https://git-scm.com/docs/git-sparse-checkout)

In reposix: the recommended recovery move when the helper rejects a `git fetch` for exceeding `REPOSIX_BLOB_LIMIT`. The error message names the command verbatim so an unfamiliar agent can follow it.

### promisor remote

A git remote that *promises* to deliver missing objects on request. Partial clone marks the origin as a promisor so git can defer blob fetches and ask for them later.

External: [extensions.partialClone config docs](https://git-scm.com/docs/git-config#Documentation/git-config.txt-extensionspartialClone)

In reposix: the helper IS the promisor remote. `reposix init` writes `extensions.partialClone=origin` so git knows to ask the helper for missing blobs.

### stateless-connect

A capability a git remote helper can advertise that lets git tunnel its native [protocol-v2](#protocol-v2) traffic to the helper instead of speaking the helper's custom dialect. Used for read traffic.

External: [git remote-helpers protocol docs](https://git-scm.com/docs/gitremote-helpers#_invocation)

In reposix: the read path. `git fetch` and lazy blob fetches both flow through `stateless-connect`, which the helper proxies into its own bare repo cache.

### fast-import

A git plumbing format that streams commits, trees, and blobs as a single text stream. Designed for high-throughput imports from foreign systems.

External: [git-fast-import docs](https://git-scm.com/docs/git-fast-import)

In reposix: the push path. The helper consumes the fast-import stream, parses each blob, runs push-time conflict detection, and applies the change as a REST write.

### fast-export

The inverse of fast-import: emits commits and tree changes as a stream. The helper relies on git's fast-export to produce the stream that reaches the helper's `export` capability handler.

External: [git-fast-export docs](https://git-scm.com/docs/git-fast-export)

In reposix: silent-bug surface — `fast-export` emits an empty delta if the refspec namespace collapses to `refs/heads/*:refs/heads/*`, which is why our refspec is `refs/heads/*:refs/reposix/*`.

### protocol-v2

The current git wire protocol (default since git 2.26). Capability-negotiated, packetized, designed to be tunnellable through arbitrary transports.

External: [protocol-v2 spec](https://git-scm.com/docs/protocol-v2)

In reposix: the read-path payload format. The helper proxies protocol-v2 frames between git's stdin/stdout and the cache's bare repo.

### refspec

The `<src>:<dst>` mapping that tells git which refs go where. `refs/heads/main:refs/heads/main` means push the local `main` to the remote's `main`.

External: [gitrevisions / refspec docs](https://git-scm.com/docs/git-fetch#Documentation/git-fetch.txt-ltrefspecgt)

In reposix: namespace isolation. Helper advertises `refspec refs/heads/*:refs/reposix/*` so the helper-side refs live in `refs/reposix/`, leaving the agent's `refs/heads/` alone.

### bare repo

A git repository without a working tree. Contains `objects/`, `refs/`, `config`, etc. directly at the top level instead of inside a `.git/` subdirectory.

External: [git-init --bare docs](https://git-scm.com/docs/git-init#Documentation/git-init.txt---bare)

In reposix: every cache is a bare repo at `~/.cache/reposix/<scheme>-<project>.git/`. The helper synthesises commits in the bare repo and serves them to the agent's working tree via protocol-v2.

### extensions.partialClone

A git config knob that tells git "this remote is a promisor — it'll deliver objects lazily when I ask." Required for partial-clone semantics to survive across `git fetch` invocations.

External: [extensions.partialClone docs](https://git-scm.com/docs/git-config#Documentation/git-config.txt-extensionspartialClone)

In reposix: set automatically by `reposix init`. Without this flag, git would treat missing blobs as corruption and abort the next fetch.

### capability advertisement

The first thing a git remote helper writes to stdout: a list of which capabilities (`fetch`, `push`, `stateless-connect`, `export`, `option`) the helper supports. Git reads the list and dispatches accordingly.

External: [git remote-helpers protocol overview](https://git-scm.com/docs/gitremote-helpers#_command_capabilities)

In reposix: the helper advertises `stateless-connect`, `export`, and `option`. Anything else (e.g. `import`) is rejected upfront so git falls back to a path the helper actually implements.

### pkt-line

The framing format used inside protocol-v2 frames. A 4-byte hex length prefix followed by payload bytes. `0000` is the flush packet (end of message).

External: [pkt-line spec](https://git-scm.com/docs/protocol-v2#_pkt_line_format)

In reposix: the helper reads and writes pkt-lines when proxying `stateless-connect` traffic. Most reposix code never touches pkt-lines directly — `gix` handles framing.

### git remote helper

An out-of-process binary git invokes (`git-remote-<scheme>`) for any URL whose scheme it doesn't natively understand. The helper speaks a small line-based protocol on stdin/stdout.

External: [git remote-helpers docs](https://git-scm.com/docs/gitremote-helpers)

In reposix: `git-remote-reposix` is the helper. Git invokes it whenever it sees a `reposix::` URL.

### push round-trip

The full sequence from `git push` to backend write to confirmation: helper consumes the [fast-import](#fast-import) stream, fetches the current backend version of each changed record, compares to the agent's commit base, and either applies the writes or rejects with `fetch first`.

External: (reposix-internal)

In reposix: the load-bearing flow for `git push IS the sync verb`. See the [git layer](../how-it-works/git-layer.md) for the sequence diagram.

### gix

A pure-Rust implementation of git. Faster startup than libgit2 for our workload, with type-safe APIs for objects, refs, and packfiles.

External: [gitoxide on GitHub](https://github.com/Byron/gitoxide)

In reposix: the cache and helper both use `gix` (pinned `=0.82` because gix is pre-1.0) to manipulate the bare repo without shelling out to `git`.

---

## SQLite + storage

### SQLite WAL

Write-Ahead Logging mode. Writes go to a separate `*-wal` file and are merged into the main DB on checkpoint. Readers don't block writers, and crash recovery rolls back the WAL.

External: [SQLite WAL docs](https://sqlite.org/wal.html)

In reposix: `cache.db` runs in WAL mode so the helper and CLI tools can read the audit log concurrently with helper writes. WAL also makes the append-only triggers cheap.

### audit log

Append-only `audit_events_cache` table inside `cache.db`. Every network-touching action (blob materialize, helper protocol event, push accept/reject, egress denial) writes one row. `BEFORE UPDATE/DELETE` triggers prevent in-place tampering.

External: (internal: `crates/reposix-cache/src/cache_schema.sql`)

In reposix: the system's outcome record. `git log` is what the agent intended; `audit_events_cache` is what actually hit the network.

---

## Data formats

### YAML

A human-friendly serialization format that nests indented key-value pairs. reposix uses YAML 1.2 via `serde_yaml`.

External: [YAML 1.2 spec](https://yaml.org/spec/1.2.2/)

In reposix: the format inside frontmatter. Every issue's structured fields (`id`, `title`, `status`, custom fields) are YAML.

### frontmatter

A YAML block at the top of a Markdown file, delimited by `---` lines. Originally a Jekyll convention; now widely used.

External: [Jekyll front-matter docs](https://jekyllrb.com/docs/front-matter/)

In reposix: how every record's structured fields ride alongside its body. A `cat issues/0001.md` shows the YAML frontmatter at the top followed by the Markdown body.

---

## reposix-specific terms

### BackendConnector

The Rust trait that every adapter implements (`reposix-sim`, `reposix-github`, `reposix-confluence`, `reposix-jira`). Methods: `list_records`, `get_record`, `create_record`, `update_record`, `delete_or_close`, `list_changed_since`, plus `supports(BackendFeature)` for capability queries.

External: (internal: `crates/reposix-core/src/backend.rs`)

In reposix: the seam. Every other crate depends on the trait, not on a concrete backend.

### Tainted&lt;T&gt;

A newtype wrapper around bytes (or records) that came from a remote. The type system refuses to let `Tainted<T>` reach an egress sink without going through `sanitize()`, which strips server-controlled fields.

External: (internal: `crates/reposix-core/src/tainted.rs`)

In reposix: the lethal-trifecta cut at the type level. A trybuild compile-fail test asserts you cannot bypass the conversion.

### egress allowlist

The single choke-point that decides whether an outbound HTTP call is allowed. Driven by `REPOSIX_ALLOWED_ORIGINS` (env var, defaults to `http://127.0.0.1:*`). All HTTP construction goes through `reposix_core::http::client()`; clippy's `disallowed_methods` lint catches direct `reqwest::Client::new()` call sites.

External: (internal: `crates/reposix-core/src/http.rs`)

In reposix: the cut between private data and exfiltration. An attacker-influenced URL cannot smuggle bytes to a non-allowlisted origin without first changing the env var.

### lethal trifecta

Simon Willison's name for the three legs an exfiltration attack against an LLM agent needs at the same time: (1) private data, (2) untrusted input, (3) exfiltration channel. If you have all three, you have a bug. reposix has all three by design — and cuts the path between them at every boundary.

External: [Simon Willison on prompt-injection / lethal trifecta](https://simonwillison.net/2024/Dec/19/prompt-injection/)

In reposix: the threat-model framing. Every mitigation in [trust model](../how-it-works/trust-model.md) is named against one of the three legs.

### dark-factory

A pattern coined in StrongDM's release notes: ship code with no human review by replacing review with simulators, swarms, and self-teaching error messages. Reposix's "agent reads stderr and recovers without prompt engineering" property is a dark-factory trait.

External: [agentic-engineering-reference notes](../research/agentic-engineering-reference.md)

In reposix: the design pressure behind every guardrail message. Helper rejects use stock git wording (`fetch first`) so an agent that has never read a reposix doc still recovers correctly.
