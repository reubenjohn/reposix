# Roadmap (v0.2+)

## What shipped in v0.1

See the [Home page](../index.md#what-shipped-in-v01). Short version: all 17 original Active Requirements from `PROJECT.md` delivered, 133 tests green, 8 security guardrails enforced and demo-visible.

## v0.2 priorities (strongest signals from v0.1 reviews)

The three phase review documents (`phase 1`, `phase 2+3`, `phase S`) accumulated 15 findings beyond the HIGH findings that were fixed before shipping. They are listed here in priority order.

### Security

1. **`X-Reposix-Agent` trust model.** Client-controlled header means an attacker can spoof a victim's rate-limit bucket and forge audit-log attribution. Fix: sign the header with an HMAC shared between sim and its configured remotes.
2. **Rate-limit `DashMap` unboundedness (M-01).** Unique agent headers grow the map without bound. Add an LRU cap (e.g. 10 000 entries) with eviction.
3. **FUSE SIGTERM handling (M-04).** Daemon currently relies on CLI watchdog + `UmountOnDrop`. If the daemon is killed via SIGTERM outside the CLI, the mount leaks until `fusermount3 -u`. Install a SIGTERM handler that unmounts.
4. **Audit-log PII redaction.** v0.1 stores first 256 chars of request body verbatim. A PII-scrubbing pass (ADR: denylist vs allowlist vs hash) should land before any non-simulator deployment.

### Correctness

5. **CRLF + non-UTF-8 in `git-remote-reposix` (H-01/H-02).** `ProtoReader` pipes blob bytes through a String-based reader. Switch to `Vec<u8>` framing. Tests: push a `.md` file containing `\r\n`, push binary attachments.
6. **Deterministic blob bytes (M-03).** Currently `frontmatter::render` and git's normalized blob differ on trailing newlines — every push emits PATCHes for "unchanged" issues. Fix: normalized-compare in `diff::plan`, not byte-compare.
7. **FUSE `create()` id divergence (H-04).** Kernel dirent uses the user-chosen path; server assigns `max_id+1`. Align these (e.g. server responds with the chosen id if it doesn't collide).
8. **Protocol error-line emission (H-03).** Backend errors during import/export currently propagate via `?`, leaving git with a torn pipe. Emit proper `error refs/heads/main <reason>` lines.
9. **IPv6 allowlist parser.** Partial (handles bracketed host), full audit and test coverage against all RFC 3986 edge cases.

### Features

10. **Real-backend adapter for GitHub Issues.** The simulator's API shape is already GitHub-flavored; a real adapter mostly needs auth + pagination. This is the single most valuable feature for making reposix a daily driver.
11. **Adversarial swarm harness.** The StrongDM "10k agent QA team" pattern at miniature scale. Simulator is ready; harness is ~half a day of work (spawn N agents, each drives `reposix` CLI in a loop against the mount, measures tail latency + failure rate).
12. **Marks-based incremental `import`.** v0.1 recomputes the full tree on every pull. At ≤100 issues this is fine; at 10 000 it's noticeably slow. Marks-file support standardizes the incremental bytes and costs.
13. **FUSE attribute cache invalidation.** Currently uses default 1s TTL. Under swarm load, push-based invalidation via `Session::notifier().inval_inode()` would eliminate stale reads.
14. **Conflict-aware git merging.** Today a 409 surfaces as a push failure. Full `<<<<<<< HEAD` marker resolution requires git-side work but is the project's north-star UX.

### Observability

15. **Simulator dashboard (`/_audit`).** A tiny embedded HTML page that polls recent rows. Research [`simulator-design.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/research/simulator-design.md) §4 lays out the design.
16. **Per-phase CI timing + coverage trend.** Current CI produces coverage via `cargo-llvm-cov`. Add a Codecov coverage-gate + a CI time-trend dashboard.
17. **Token-economy benchmark harness.** A `crates/reposix-bench/` that measures the 98.7% claim against representative workflows. See [why.md](../why.md#token-economy-benchmark).

## Long-term north stars

- **macOS via macFUSE.** Requires a kernel extension + entitlement; architecturally similar to Linux FUSE. Re-evaluate when the Linux mount is battle-tested.
- **Windows via ProjFS / WinFsp.** A different abstraction; likely a parallel `reposix-projfs` crate instead of extending `reposix-fuse`.
- **A "real dark factory" deployment.** Simulated agents, a deliberately-broken real workflow, a large-scale exfil-surface test. This is the proof of the proof-of-usage.

## Known non-goals

- Web UI / dashboard as a primary user-facing surface. Agents don't need it; humans use the CLI + `git`.
- Support for a monolithic SaaS product (single hosted reposix). Local-first only.
- Picking a side between JSON-API-shaped backends and git-shaped ones. reposix is the impedance-matcher.

## How to extend

Start with `/gsd-add-phase` in the project root. The `.planning/ROADMAP.md` is the living scope — append there, then follow the plan → execute → review cycle. The [Contributing page](contributing.md) has the details.
