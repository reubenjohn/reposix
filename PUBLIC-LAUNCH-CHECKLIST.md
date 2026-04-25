# Public launch checklist

The pre-flight, day-of, and post-launch list the owner runs through before pulling the trigger on HN / Twitter / blog. Mirrors the launch playbooks of TigerBeetle, Polar, and similar early-stage OSS projects. Every line should be tickable in under five minutes.

## Pre-flight (do these in order)

- [ ] LICENSE in place (Apache-2.0 + MIT confirmed in workspace metadata; `deny.toml` aligned)
- [ ] CONTRIBUTING.md / SECURITY.md / CODE_OF_CONDUCT.md present
- [ ] Issue + PR templates configured (`.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md`)
- [ ] `dependabot.yml` configured
- [ ] `cargo-deny` + `cargo-audit` running in CI (weekly schedule + on-PR for audit)
- [ ] README hero leads with measured numbers — no naked adjectives above the fold
- [ ] mkdocs site builds with `mkdocs build --strict`
- [ ] All tests green on `main`: `cargo test --workspace --locked`
- [ ] `bash scripts/banned-words-lint.sh` green
- [ ] `bash scripts/banned-words-lint.sh --all` green (QA mode covers Layer 3 too)
- [ ] CHANGELOG covers everything since the last tag — `git log --oneline <last-tag>..HEAD` reconciles
- [ ] At least 3 working examples in `examples/` (currently 5: shell loop, Python agent, Claude Code skill, conflict resolve, blob-limit recovery)
- [ ] Tag the release: `bash scripts/tag-v0.10.0.sh`
- [ ] GitHub release page populated from CHANGELOG body

## Day-of-launch checklist

- [ ] Final smoke test: `cargo run -p reposix-sim &` + `reposix init sim::demo /tmp/launch-smoke && cd /tmp/launch-smoke && git checkout origin/main && cat issues/0001.md` works end-to-end in under 30 s
- [ ] Author a launch announcement at announcement-time (the Phase 50 archival sweep removed the early-draft blog post per POLISH-11; v0.11.0 launch material gets re-authored when launch is imminent)
- [ ] HN submission: title `≤ 80` chars; URL points to the mkdocs site; first comment from owner with the 5-line bootstrap snippet
- [ ] Twitter / Mastodon thread — 5–7 posts; lead with the cost differential, not the architecture
- [ ] Discord / Slack: post to relevant communities (Rust users, agent-builders, Claude Code / Cursor channels)
- [ ] Reach out to 3–5 friendly reviewers — have them validate the 5-min tutorial cold (no priors)
- [ ] Post to relevant GitHub awesome-lists (`awesome-rust`, `awesome-claude-code` if it exists, agent-tooling collections)

## Post-launch

- [ ] Watch GitHub stars + issues for the first 24 h; pin a "thanks for stopping by" issue if traffic is heavy
- [ ] Triage incoming with the BackendConnector proposal template
- [ ] Acknowledge first contributors in CHANGELOG
- [ ] Update CHANGELOG with the launch date in the `[v0.10.0]` block (or a `## [Launched]` callout)
- [ ] Capture telemetry: docs traffic, README clone count, `git clone` referrers — feed into v0.11.0 prioritization

## Risks to mitigate

- **"Why not just MCP?"** — link to [`docs/concepts/reposix-vs-mcp-and-sdks.md`](docs/concepts/reposix-vs-mcp-and-sdks.md). MCP is fine for tools that need rich schemas; reposix complements MCP for the operations every agent does constantly.
- **"Where are the benchmarks?"** — link to [`docs/benchmarks/v0.9.0-latency.md`](docs/benchmarks/v0.9.0-latency.md). The token claim sits in [`benchmarks/RESULTS.md`](benchmarks/RESULTS.md).
- **"Is this audited?"** — link to [`SECURITY.md`](SECURITY.md) and the [trust-model](docs/how-it-works/trust-model.md). The lethal-trifecta cuts are listed there.
- **"How does it work without MCP?"** — link to [`docs/how-it-works/git-layer.md`](docs/how-it-works/git-layer.md).

## Anti-patterns (don't)

- Don't lead with adjectives. Lead with numbers.
- Don't promise a feature that's `pending-secrets` or in-flight.
- Don't claim reposix supersedes MCP — P1 framing matters; MCP is fine for tools that need rich schemas.
- Don't link to internal artifacts that won't survive a cold reader: `.planning/`, `git log`, internal Slack threads.
- Don't run `git push --force` to `main` for any reason during launch week.

## Optional polish before launch

If the owner wants a bigger announcement, ship one or more of these v0.11.0 backlog items first:

- [ ] OpenTelemetry tracing — OTLP spans on every fetch / blob materialization / push, with `taint.origin`, `audit.row_id`, `agent.harness_hint`, `project.key` attributes for dark-factory queries
- [ ] Multi-project helper process — single daemon serving N partial clones from a shared cache; 20× memory drop for agents juggling many projects
- [ ] Plugin registry mock — `reposix init <plugin>::<project>` auto-resolves a community connector
- [ ] Conflict resolution UI — `reposix conflict <id>` field-level frontmatter diff when `git pull --rebase` hits a YAML merge

Each is ~2–4 days. Each closes a credible "but what about ..." question. None is required.
