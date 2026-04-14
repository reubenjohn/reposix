# MORNING-BRIEF — v0.3.0

> Date: 2026-04-14 (overnight session 3, ~05:10 UTC handoff).
> Previous briefs: [`MORNING-BRIEF.md`](MORNING-BRIEF.md) (v0.1 / v0.2), [`PROJECT-STATUS.md`](PROJECT-STATUS.md) (timeline through v0.2.0-alpha).
> This brief supersedes both for v0.3 status. The older briefs are still correct for their eras — they are not "stale", just "for earlier releases".

## tl;dr

Phase 11 shipped read-only **Atlassian Confluence Cloud** support end-to-end: adapter crate, CLI dispatch, contract test (parameterized over sim + wiremock + live), Tier 3B + Tier 5 demos, ADR-002, reference docs, a "build-your-own-connector" guide, and a CHANGELOG v0.3.0 block. Workspace is **191/191 passing**, clippy clean, fmt clean, `scripts/demos/smoke.sh` 4/4, `mkdocs build --strict` green. Live-wire verification **did** run successfully tonight against `reuben-john.atlassian.net` space `REPOSIX` (3 seeded pages round-tripped through the adapter and the FUSE mount).

**The one thing left for you to do:** run `bash scripts/tag-v0.3.0.sh` to cut + push the `v0.3.0` annotated tag. The autonomous session deliberately stopped short of pushing the tag — see §"Cutting the tag" below.

## What shipped

| Crate / file | What it does |
|---|---|
| `crates/reposix-confluence/` | Read-only Confluence Cloud REST v2 adapter. 17 wiremock tests; `Tainted<T>` everywhere; redacted `Debug`. Commit `fafec8f`. |
| `reposix list --backend confluence --project <SPACE_KEY>` | Live-verified against `reuben-john.atlassian.net` tonight. Commit `5182b72`. |
| `reposix mount --backend confluence --project <SPACE_KEY>` | FUSE-mounts a Confluence space as `<padded-id>.md` files. Same `Mount::open(Arc<dyn IssueBackend>)` path as `github` and `sim`. |
| `crates/reposix-confluence/tests/contract.rs` | The same contract assertions run against `SimBackend` (always), wiremock (always), and live Confluence (`--ignored`). Commit `f1ec6c1`. |
| `scripts/demos/parity-confluence.sh` + `scripts/demos/06-mount-real-confluence.sh` | Tier 3B + Tier 5 demos; skip cleanly with `SKIP:` if Atlassian env vars absent. Commit `a45b332`. |
| `docs/decisions/002-confluence-page-mapping.md` | ADR for the Option-A flatten decision. |
| `docs/reference/confluence.md` + `docs/connectors/guide.md` | User-facing backend reference + "build-your-own-connector" guide (462 lines). The connector guide previews the **Phase 12 subprocess/JSON-RPC ABI** as the scalable-without-forking successor to the current "fork + add dispatch" model. Commits `234beef`, `4dd73fa`, `eeb8baf`. |
| `CHANGELOG.md` | [Unreleased] block promoted to `[v0.3.0] — 2026-04-14` (this plan, Task 2). |

See [CHANGELOG.md](CHANGELOG.md) `[v0.3.0]` section for the full release notes.

## Connector scalability — the story in one paragraph

The current v0.3 connector story is "fork reposix, add a Cargo dep on `reposix-adapter-<name>`, wire three lines of CLI dispatch." That's fine for the first three adapters (sim, github, confluence) shipped by this repo — it's not fine for the long tail of internal-tracker integrations a real user wants. **Phase 12 lifts the adapter boundary across a subprocess/JSON-RPC ABI** so third parties can ship connectors as standalone binaries: no fork, no Rust-ABI coupling, no recompile of reposix. See [`docs/connectors/guide.md`](docs/connectors/guide.md) — the guide already documents the v0.3 short-term model AND sketches the Phase 12 migration path, so anyone starting an adapter today won't have to throw it away when Phase 12 lands.

## 30-second fix: credentials (if you're starting from a clean `.env`)

> Skip this section if your `.env` already has `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, `REPOSIX_CONFLUENCE_SPACE`. Tonight's session verified those are populated.

1. Visit <https://id.atlassian.com/manage-profile/security/api-tokens>. Create a token if you don't have one.
2. **Note the email at the top-right of that page.** That exact email is `ATLASSIAN_EMAIL` — Atlassian user API tokens are account-scoped, not email-scoped, so the email must match the account the token was issued under. (The session 3 probe originally guessed wrong and burned 4 minutes on it; see [`.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md`](.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md).)
3. Decide a tenant subdomain. Confirm with:
   ```bash
   curl -s -u "$ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY" \
     "https://YOUR_TENANT.atlassian.net/wiki/api/v2/spaces?limit=1" | head -c 200
   ```
   A non-empty JSON response ⇒ you have the right tenant.
4. Pick a space key — the `<KEY>` segment of any page URL `https://<tenant>.atlassian.net/wiki/spaces/<KEY>/...`.

## Prove it works (commands to copy-paste)

```bash
# From the repo root (adjust the path).
cd /home/reuben/workspace/reposix

# (If starting fresh) update .env with the four Atlassian values:
cat > .env <<'EOF'
ATLASSIAN_API_KEY=<your token from step 1>
ATLASSIAN_EMAIL=<the email from step 2>
REPOSIX_CONFLUENCE_TENANT=<your tenant subdomain>
REPOSIX_CONFLUENCE_SPACE=<any space key you can read>
EOF

# Export them into the shell.
set -a; source .env; set +a
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

# Build release binaries (skip if you already have target/release/reposix from last night).
cargo build --release --workspace --bins --locked
export PATH="$PWD/target/release:$PATH"

# (A) List a real Confluence space from the CLI.
reposix list --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" --format table

# (B) FUSE-mount a real Confluence space — this is the money shot from HANDOFF §9 step B.
mkdir -p /tmp/reposix-conf-mnt
reposix mount /tmp/reposix-conf-mnt --backend confluence --project "$REPOSIX_CONFLUENCE_SPACE" &
MOUNT_PID=$!
sleep 3
ls /tmp/reposix-conf-mnt | head -20
cat /tmp/reposix-conf-mnt/*.md | head -50
fusermount3 -u /tmp/reposix-conf-mnt
wait $MOUNT_PID 2>/dev/null || true

# (C) The two new demos — both skip cleanly with SKIP: when env is unset.
bash scripts/demos/parity-confluence.sh
bash scripts/demos/06-mount-real-confluence.sh

# (D) Full workspace tests + live contract half + smoke regression.
cargo test --workspace --locked                    # expect 191/191
cargo test -p reposix-confluence --locked -- --ignored  # live half (contract_confluence_live)
bash scripts/demos/smoke.sh                        # expect 4 passed, 0 failed
```

## CI secrets (one-shot)

The `integration-contract-confluence` CI job is already wired into `.github/workflows/`. It activates automatically once these four repo secrets are set:

```bash
gh secret set ATLASSIAN_API_KEY        --body "$ATLASSIAN_API_KEY"
gh secret set ATLASSIAN_EMAIL          --body "$ATLASSIAN_EMAIL"
gh secret set REPOSIX_CONFLUENCE_TENANT --body "$REPOSIX_CONFLUENCE_TENANT"
gh secret set REPOSIX_CONFLUENCE_SPACE --body "$REPOSIX_CONFLUENCE_SPACE"
```

Without them the `if:` clause in the workflow skips the job cleanly (no failure, no proof). With them, every push runs the live contract against your tenant.

## Cutting the tag — the single human-gate step

The autonomous session deliberately did **not** push the tag. Per the plan's `autonomous: false` frontmatter and the T-11F-06 mitigation in the threat register, `git push origin v0.3.0` is a permanent and widely-visible action that must pass a human-verify gate.

Tonight's artifacts for you to eyeball before running the script:

1. `CHANGELOG.md` — read the `[v0.3.0] — 2026-04-14` section. Sanity-check the BREAKING callout about the `TEAMWORK_GRAPH_API → ATLASSIAN_API_KEY` env-var rename.
2. `.planning/phases/11-confluence-adapter/11-F-SUMMARY.md` — tonight's execution summary.
3. `docs/decisions/002-confluence-page-mapping.md` — ADR-002 (Option-A flatten decision).
4. `docs/connectors/guide.md` — the 462-line connector guide (user asked for this directly).

Once you've reviewed those and want to ship:

```bash
# FIRST: deal with the pre-existing uncommitted drift in the working tree.
# At handoff, `git status --short` showed the following modifications carried
# over from prior phases (NOT from Phase 11-F — Phase 11-F only added
# MORNING-BRIEF-v0.3.md, scripts/tag-v0.3.0.sh, and edits to CHANGELOG.md,
# MORNING-BRIEF.md, PROJECT-STATUS.md):
#
#    D .claude/scheduled_tasks.lock                          (ephemeral lock — safe to delete)
#    M benchmarks/RESULTS.md                                 (timestamp-only diff)
#    M crates/reposix-confluence/Cargo.toml                  (adds `url` workspace dep)
#    M crates/reposix-confluence/src/lib.rs                  (WR-01 + WR-02 hardening
#                                                             from 11-REVIEW.md —
#                                                             percent-encode space_key +
#                                                             numeric-only space_id taint check)
#    ?? .planning/phases/11-confluence-adapter/11-REVIEW.md  (code-review artifact)
#
# Three options:
#   (a) Review the diffs and COMMIT them as a pre-release hardening bundle, e.g.
#         git add crates/reposix-confluence benchmarks/RESULTS.md \
#                 .planning/phases/11-confluence-adapter/11-REVIEW.md
#         git commit -m "chore(pre-release): WR-01/WR-02 hardening + review artifacts"
#         git rm .claude/scheduled_tasks.lock
#         git commit -m "chore: drop ephemeral scheduled_tasks.lock"
#       Then cargo test + clippy to confirm the hardening didn't regress anything.
#   (b) git stash the drift, cut the tag, pop and land the hardening as v0.3.1.
#   (c) git checkout the drift + delete the untracked files (destructive — only if
#       you're sure the hardening is unwanted).
#
# RECOMMENDED: (a). The WR-01/WR-02 diffs are security hardening and belong in v0.3.

# After the working tree is clean, verify the workspace is still green
# (the tag script also runs these, but do them once yourself).
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
bash scripts/demos/smoke.sh

# THEN cut the tag. The script enforces six safety guards (branch=main, clean tree, tag doesn't already
# exist locally OR on origin, CHANGELOG has [v0.3.0] section, cargo test green, smoke.sh green) and
# will exit non-zero WITHOUT tagging if any guard fails. No override flag — fix the root cause.
bash scripts/tag-v0.3.0.sh
```

**The single command that this session stopped short of running:**

```bash
git push origin v0.3.0
```

The `tag-v0.3.0.sh` script wraps that command. Running the script IS the push. There is no other step. After the push succeeds, optionally create a GitHub Release at <https://github.com/reubenjohn/reposix/releases/new?tag=v0.3.0> and paste the CHANGELOG `[v0.3.0]` section as the body.

## Known open gaps

Per `~/.claude/CLAUDE.md` OP #6 ("ground truth obsession"), being loud about what's NOT shipped:

- **`atlas_doc_format` → Markdown rendering** is deferred to v0.4. Page bodies in v0.3 are raw Confluence storage XHTML. Human-readable but not as tidy as GitHub's plain markdown. ADR-002 documents the decision.
- **Write path** (`create_issue` / `update_issue` / `delete_or_close`) on `ConfluenceReadOnlyBackend` returns `NotSupported`. v0.4.
- **`PageBackend` trait** (ADR-002 Option-B) is deferred. v0.3 flattens Confluence's page hierarchy into `parent_id` frontmatter metadata; there is no `cd` into subpages. v0.4 will add a sibling trait if user feedback warrants.
- **Phase 12 subprocess/JSON-RPC connector ABI** — the scalable-without-forking successor to today's "fork + add dispatch" model. Documented in `docs/connectors/guide.md` and `ROADMAP.md §Phase 12`. Not started.
- **Labels, attachments, comments** on Confluence pages — not exposed by the adapter. v0.4+.
- **Swarm harness against Confluence** (`--mode confluence-direct`) — Phase 11 stretch goal; deferred because rate limits make a 50-client 30s run expensive.
- **FUSE write path through `IssueBackend::update_issue`** — still routes through the sim-specific REST shape in `crates/reposix-fuse/src/fetch.rs`. Same v0.3-era deferral noted in v0.2.0-alpha notes; no new work tonight.
- **`git-remote-reposix` rewire through `IssueBackend`** — still hardcodes the simulator. Mechanical but not done.

## Stats

| Metric | v0.2.0-alpha | v0.3.0 |
|---|---|---|
| Workspace tests | 168 | **191** (+23 from `reposix-confluence` unit + contract tests) |
| Commits since prior tag | — | 24 atomic commits across 11-A..11-F |
| `cargo clippy --all-targets -- -D warnings` | clean | clean |
| `cargo fmt --all --check` | clean | clean |
| `mkdocs build --strict` | green | green |
| `scripts/demos/smoke.sh` | 4/4 | **4/4** |
| Backends | `sim`, `github` | `sim`, `github`, **`confluence`** |

## Handoff

If there's a next overnight agent: your starting points in order are this file, then [`HANDOFF.md`](HANDOFF.md) (for the broader overnight-mission style), then [`.planning/phases/11-confluence-adapter/`](.planning/phases/11-confluence-adapter/) for Phase 11's internal artifacts, then the `[v0.3.0]` block in `CHANGELOG.md`. The Phase 12 subprocess connector ABI is the obvious next mission and is already scaffolded in `docs/connectors/guide.md`.

## Sign-off

— Claude Opus 4.6 1M context, 2026-04-13 / 2026-04-14 (overnight session 3).
