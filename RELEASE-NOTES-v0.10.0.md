# reposix v0.10.0 — Docs & Narrative Shine

**Release date:** 2026-04-25
**Tag:** `v0.10.0` · [CHANGELOG entry](CHANGELOG.md#v0100--2026-04-25)
**Companion:** [launch blog post](docs/blog/2026-04-25-reposix-launch.md)

## What's new (TL;DR)

- A Diátaxis-structured docs site a cold reader absorbs in 10 seconds.
- A 5-minute first-run tutorial that runs end-to-end against the simulator with zero credentials.
- A "How it works" trio (filesystem · git · trust model), each with one mermaid diagram.
- Mental-model and `vs MCP / SDKs` pages that name the trade-offs honestly.
- Five end-to-end agent loop examples in `examples/`, including a two-agent contention scenario.
- A banned-words linter enforcing P1/P2 progressive-disclosure framing rules.

## Highlights

### Diátaxis-structured docs site

`mkdocs.yml` nests under six sections — Concepts, Tutorials, How it works, Guides, Reference, Benchmarks. The hero is a 102-line README leading with three measured numbers (`8 ms` cached read, `24 ms` cold init, `92.3%` token reduction). Redirect stubs keep external links intact.

### Banned-words linter

`scripts/banned-words-lint.sh` enforces the P1/P2 framing rules:

- **P1** — the "complement, never supersede" rule for MCP framing applies everywhere in `docs/`.
- **Layer 1 (Hero)** and **Layer 2 (Concepts / Tutorials / Guides)** forbid plumbing vocabulary (`partial-clone`, `promisor`, `protocol-v2`, `fast-import`, `stateless-connect`, plus FUSE-era terms). Layer 3 is unrestricted.

Pre-commit and CI invoke the same script. A `<!-- banned-words: ok -->` marker handles deliberate exceptions.

### Mental-model + "vs MCP" positioning

`docs/concepts/mental-model-in-60-seconds.md` distills the model: clone = snapshot, frontmatter = schema, `git push` = sync verb. `docs/concepts/reposix-vs-mcp-and-sdks.md` names the trade-offs without claiming MCP is wrong — reposix complements MCP for operations every agent does constantly.

### Five working examples

`examples/` ships five runnable agent loops: `01-shell-loop`, `02-python-agent`, `03-claude-code-skill`, `04-conflict-resolve` (two-agent contention; second agent recovers from `error refs/heads/main fetch first` with no in-context learning), `05-blob-limit-recovery` (agent reads the `git sparse-checkout` teaching string from stderr and narrows scope).

### Project hygiene

`SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, issue + PR templates, `dependabot.yml`, `cargo-deny` (`deny.toml`), and `cargo-audit` (weekly + on-PR) all landed. Per-crate Cargo metadata is complete.

## Hard numbers

Measured against the in-process simulator on a stock laptop ([reproducer](docs/benchmarks/v0.9.0-latency.md)):

| Step                                              | sim (ms) |
|---------------------------------------------------|---------:|
| `reposix init <backend>::<project> <path>` cold  |       24 |
| List issues (REST round-trip)                    |        9 |
| Get one issue (REST round-trip)                  |        8 |
| PATCH issue (REST round-trip)                    |        8 |
| Helper `capabilities` probe                      |        5 |

Soft thresholds (regression-flagged via `WARN:` lines, not CI-blocking): sim cold init `< 500 ms`, real-backend step `< 3 s`. Real-backend cells are blank until secret packs decrypt.

## Breaking changes

None. The v0.9.0 architecture pivot already absorbed the breaking changes (`reposix mount` removed, `reposix demo` removed, `crates/reposix-fuse/` deleted). v0.10.0 is docs + ergonomics.

## Upgrade path

```bash
git pull
cargo install --path crates/reposix-cli
```

Working tree contents do not change for v0.9.0 users. Earlier versions follow the v0.9.0 migration recipe in [`CHANGELOG.md`](CHANGELOG.md).

## What's next (v0.11.0 preview)

The v0.11.0 milestone is in flight. Six items have already landed on `main`:

1. **`IssueId` → `RecordId`, `Issue` → `Record`** — workspace-wide hard rename ([ADR-006](docs/decisions/006-issueid-to-recordid-rename.md)). YAML wire format unchanged.
2. **`reposix doctor`** — 14 setup checks, severity tiers, copy-pastable fix commands, `--fix` mode for the deterministic subset.
3. **`reposix history`** + **`reposix at <ts>`** — time-travel via private git tags ([ADR-007](docs/decisions/007-time-travel-via-git-tags.md)). Highest-novelty entry in the v0.11.0 brainstorm.
4. **Helper URL-scheme backend dispatch** ([ADR-008](docs/decisions/008-helper-backend-dispatch.md)) — closes the v0.9.0 Phase 32 carry-forward tech debt.
5. **`reposix gc`** — cache eviction (LRU / TTL / all). Tree, commit, ref objects are never touched.
6. **`reposix tokens`** — token-economy ledger over the audit log, with an honest MCP-equivalent baseline.

[v0.11.0 preview notes](RELEASE-NOTES-v0.11.0-PREVIEW.md) tracks the in-flight cut.
