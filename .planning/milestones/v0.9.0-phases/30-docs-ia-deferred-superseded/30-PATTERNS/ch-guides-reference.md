# Pattern Assignments — Guides and Reference pages

← [back to index](./index.md)

### `docs/guides/write-your-own-connector.md` (MOVE from `docs/connectors/guide.md`)

**Analog:** `docs/connectors/guide.md` (465 lines) — **preserve verbatim**. This is a file move, not a rewrite.

**Source-of-truth clause (lines 64–65 of `docs/connectors/guide.md` — must be preserved intact):**

```markdown
Do NOT read the above and copy it into your adapter's docs — link to
`crates/reposix-core/src/backend.rs` as the single source of truth.
```

**Internal-link pattern used throughout (`docs/connectors/guide.md` line 451 onward):**

```markdown
- [`crates/reposix-core/src/backend.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
  — `IssueBackend` trait, `BackendFeature`, `DeleteReason`.
- [ADR-001 GitHub state mapping](../decisions/001-github-state-mapping.md)
```

**Divergence from analog:** Two relative-link updates are required:
1. Path `../decisions/...` still works from `docs/guides/` (same depth as `docs/connectors/`). No change needed.
2. Path `../reference/confluence.md` still works (same depth).
3. External repo links (using `https://github.com/reubenjohn/reposix/blob/main/...`) are unaffected by the move.

**Nav entry** moves from `Connectors > Building your own` to `Guides > Write your own connector`. Phase 11 and Phase 12 ROADMAP pointers at lines 8–14 and 363–399 stay intact.

---

### `docs/guides/integrate-with-your-agent.md` (NEW — greenfield)

**Analog (prose voice):** `docs/connectors/guide.md` step-by-step pattern (lines 74–165) + `docs/why.md` voice for the agent-framing context.

**Analog step pattern (`docs/connectors/guide.md` lines 74–100) — `### Step N. Name` H3 + prose + fenced code + terse trailing explanation:**

```markdown
### Step 1. Cargo skeleton

\`\`\`bash
cargo new --lib reposix-adapter-foo
cd reposix-adapter-foo
\`\`\`

Copy the dependency list from [`reposix-confluence/Cargo.toml`](...) as a starting point. The minimum set is:

\`\`\`toml
[dependencies]
reposix-core = "..."               # match the version of reposix you're targeting
...
\`\`\`
```

**Voice excerpt to echo (`docs/why.md` lines 89–98):**

```markdown
Every modern foundation model has been trained on:

- The Linux man pages (`man 1 grep`, `man 7 regex`, `man 2 open` — all there).
- Hundreds of thousands of open-source shell scripts.
- Countless Stack Overflow answers that use `sed`, `awk`, `jq`, `find`.
- Every commit message in every public Git repo of any size.
```

**Divergence from analog:** No existing source for this page. Planner + subagent author from scratch. Suggested structure (research-aligned):
1. `## With Claude Code` — prompt pattern showing a system prompt that mentions the mount path; include expected token savings (~75×, link to `why.md`).
2. `## With Cursor` — `.cursorrules` pattern.
3. `## With a custom SDK` — 20 lines of Python / TypeScript doing `subprocess.run(["git", "push"])` against a reposix mount.
4. `## Gotchas` — taint boundary, allowlist setup, REPOSIX_ALLOWED_ORIGINS walkthrough.

Above-Layer-3 banned words: FUSE, daemon, mount (noun), kernel. Vale rule enforces.

---

### `docs/guides/troubleshooting.md` (NEW — stub that grows post-launch)

**Analog (prose + failure-case voice):** `docs/demo.md` §"Limitations / honest scope" (lines 272–298) — shows honest-accounting tone.

**Analog pattern (`docs/demo.md` lines 274–293):**

```markdown
## Limitations / honest scope

This demo page was captured at v0.1 alpha (2026-04-13) and shows the simulator-only narrative...

- **The demo script itself still targets the simulator** — it's the fastest, cred-free path to demonstrate ...
- **No man page, .deb, or brew formula.** Clone-and-`cargo build`.
- **Linux only.** FUSE3/FUSE2. macOS-via-macFUSE is a follow-up.
```

**Divergence from analog:** Troubleshooting is work-mode (How-to), not honest-scope (Explanation). Each entry is a Symptom/Cause/Fix triad, not a limitation bullet. Three initial stub entries per RESEARCH.md line 943:
1. FUSE mount fails — likely `fuse3` not installed; run `sudo apt install fuse3`.
2. `git push` rejected with `bulk-delete` — SG-02 fired; either reduce scope or append `[allow-bulk-delete]` to commit message.
3. Audit log query — `sqlite3 /tmp/demo-sim.db 'SELECT ...'` — pattern from `docs/demo.md` lines 234–241.

Page is explicitly a stub; CHANGELOG notes "grows post-launch." Don't pad.

---

### `docs/guides/connect-{github,jira,confluence}.md` (NEW — stubs)

**Analog:** `docs/reference/confluence.md` (full backend reference) — lift credential-env-var section. `docs/demos/index.md` for the tier-5 real-backend scripts.

**Voice excerpt pattern (`docs/why.md` lines 73–79):**

```bash
REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://api.github.com' \
    GITHUB_TOKEN="$(gh auth token)" \
    reposix list --backend github --project octocat/Hello-World --format table
```

**Divergence from analog:** Each of these is a stub that links to (a) the existing `docs/reference/{confluence,jira}.md` reference page, (b) the `scripts/demos/05-mount-real-github.sh` tier-5 script, and (c) the `REPOSIX_ALLOWED_ORIGINS` + credential env-var requirement. Do NOT duplicate `reference/` content — single source of truth.

---

### `docs/reference/simulator.md` (NEW — carved from architecture + reference/http-api)

**Analog:** `docs/reference/cli.md` (lines 1–76, the flag-table-by-subcommand reference voice) + `docs/reference/http-api.md` (lines 1–60, the endpoint-by-endpoint reference voice) + `docs/architecture.md` §"System view" simulator-row.

**Analog flag-table pattern (`docs/reference/cli.md` lines 47–58):**

```markdown
## `reposix sim`

Spawn the REST simulator as a subprocess.

| Flag | Default | Purpose |
|------|---------|---------|
| `--bind` | `127.0.0.1:7878` | Listen address. |
| `--db` | `runtime/sim.db` | SQLite file. |
| `--seed-file` | — | Path to JSON seed (e.g. `crates/reposix-sim/fixtures/seed.json`). |
| `--no-seed` | off | Don't seed even if `--seed-file` is given. |
| `--ephemeral` | off | Use in-memory SQLite instead of `--db`. |
| `--rate-limit` | `100` | Per-agent requests/sec. |
```

**Analog endpoint-table pattern (`docs/reference/http-api.md` lines 7–59):** reference-voice endpoint listings with purpose-and-example pairs.

**Divergence from analog:** This page frames the simulator as **dev tooling** (research-agreed), not core architecture. Opening sentence should be "The simulator is the default testing backend for reposix. …" Lift the sim route table from `reference/http-api.md` (already correct). Lift the CLI flag table from `reference/cli.md` `reposix sim` subcommand. Add a new "Seeding + fixtures" section describing `crates/reposix-sim/fixtures/seed.json` (not yet documented anywhere user-facing).
