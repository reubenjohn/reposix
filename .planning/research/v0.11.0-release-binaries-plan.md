# v0.11.0 release binaries plan

**Status:** research, not yet ratified.
**Date:** 2026-04-25.
**Author:** subagent (research-only; no files modified outside this doc).

## TL;DR

Adopt **dist** (formerly `cargo-dist`, axodotdev, v0.31.0 — Feb 2026) as the release-pipeline generator, ship a 5-target matrix (Linux x86_64/aarch64 musl, macOS x86_64/aarch64, Windows x86_64), publish `reposix-core`, `reposix-sim`, `reposix-cache`, the three backend crates, `reposix-remote`, and `reposix-cli` to crates.io in topological order, and add `[package.metadata.binstall]` hints so `cargo binstall reposix-cli` resolves to the GitHub-Releases tarballs without compiling. The current hand-rolled `release.yml` (gnu-only, Linux-only, no installer scripts, no checksum signing, no Homebrew) gets replaced.

## Recommended approach (ranked)

1. **dist (cargo-dist) + crates.io + binstall metadata.** The "uv / ruff" stack — same tool Astral uses for 18+-target releases. One-shot `dist init` writes `.github/workflows/release.yml` for us, plus `install.sh`, `install.ps1`, and Homebrew tap. Active 2026 maintenance, broad platform coverage including musl, generates SHA256SUMS and minisign signatures, supports `cargo binstall` natively (it writes the `[package.metadata.binstall]` block during `dist init`).
2. **GoReleaser via `cargo zigbuild`.** Mature multi-language tool with first-class Rust support since 2024. Useful if we want one release pipeline for Rust + Docker + a hypothetical Go sidecar. Heavier to learn for a Rust-only project; loses the auto-generated install.sh / Homebrew formula that dist gives us.
3. **release-plz (release-only) + handwritten release.yml.** release-plz is great at version-bump PRs and crates.io publishing but does not generate cross-platform binary pipelines. Combine with our existing hand-rolled release.yml. Lowest moving-parts but highest hand-written YAML.
4. **Status quo (hand-rolled `release.yml`).** Works for `x86_64-unknown-linux-gnu`. Does not scale to the 5-target matrix without us reimplementing what dist already gives us — installers, signing, Homebrew, binstall metadata.

Pick (1). The complexity argument from CLAUDE.md OP-3 ("ROI awareness") cuts the other way here: dist *removes* hand-written code, it does not add it.

## Tooling deep-dive

### dist (formerly cargo-dist)

**What it does.** Generates a complete release pipeline: GitHub Actions workflow that builds for every requested target triple, packages tarballs (and `.zip` for Windows), produces SHA256SUMS, generates `install.sh` / `install.ps1` from templates, writes a Homebrew formula and pushes it to a tap repo, writes `[package.metadata.binstall]` so `cargo binstall` works without further config, and (since 0.31.0) supports mirror hosting beyond GitHub Releases.

**Maintenance status (April 2026).** Active. Latest is **v0.31.0 released 2026-02-23**; v0.30.4 shipped 2026-02-16; the repo (axodotdev/cargo-dist) had 271 releases by Feb 2026. Rebranded from `cargo-dist` to `dist` (the binary is now `dist`; the repo title reads `dist (formerly known as cargo-dist)`). Astral's uv and ruff use it for their 18+-platform release pipelines.

**Limitations / pitfalls.**
- Rebrand is in progress: docs and Homebrew still say `cargo-dist`. Use `dist` as the binary name in our scripts.
- The generated `release.yml` is opinionated — we will lose the custom CHANGELOG-extraction step we currently have. dist has its own `--changelog-path` setting; we wire it to `CHANGELOG.md` and accept the format dist expects (Keep-a-Changelog).
- `dist init` rewrites `release.yml` in place. We need to commit the existing one to git first, run `dist init`, diff, and migrate any custom logic (only the changelog-section extractor is custom; everything else dist does better).
- musl targets need `cargo zigbuild` or a musl-cross toolchain on the runner. dist handles this; we just opt in.
- The `aarch64-pc-windows-msvc` target is still cross-compile-only on GitHub Actions (no native runner) and `ring` (transitive via rustls) historically had issues there. We are not shipping Windows ARM in v0.11.0; revisit later.

**Setup steps.**
1. `cargo install cargo-dist --locked` (or `brew install cargo-dist`).
2. `cd /home/reuben/workspace/reposix && dist init` — interactive; pick the 5-target matrix, enable shell + powershell + homebrew installers, tell it the Homebrew tap repo (`reubenjohn/homebrew-reposix`, to be created).
3. Commit the generated `[workspace.metadata.dist]` block in `Cargo.toml`, the new `.github/workflows/release.yml`, and the Homebrew tap config.
4. Create `homebrew-reposix` repo under `reubenjohn/` (empty) and add a deploy key / PAT scoped to that repo as `HOMEBREW_TAP_TOKEN` secret on the main repo.
5. Tag `v0.11.0` — pipeline runs.

### cargo-binstall

**What it does.** A user-side tool: `cargo binstall reposix-cli` fetches the crate metadata from crates.io, follows the `repository` field to GitHub, looks for a release artifact whose name matches the templated `pkg-url` (or our explicit `[package.metadata.binstall]`), downloads, verifies the checksum if present, and drops the binary into `~/.cargo/bin/`. Falls back to `cargo install --locked` if no prebuilt is found.

**Maintenance status (April 2026).** Active; recent release on **2026-04-13** with full asset matrix including signed `cargo-binstall-aarch64-apple-darwin.full.zip`. Hosted under cargo-bins org.

**Limitations / pitfalls.**
- Requires the crate to be **published to crates.io**. Without a crates.io entry binstall has no anchor.
- `[package.metadata.binstall]` template strings reference `{ name }`, `{ version }`, `{ target }`, `{ archive-suffix }`. If our archive layout differs from the dist default, we override the template explicitly.
- Default discovery works without explicit metadata for projects that use dist (dist sets up the conventional layout). If dist is used end-to-end, we get binstall support "for free."

**Setup steps.** None beyond the dist setup, *if* we publish to crates.io. Otherwise we need to host the metadata in a published index crate.

### GoReleaser (alternative, not recommended for this project)

**What it does.** Cross-language release tool. Builds via `cargo zigbuild`, generates archives, sha256 sums, GitHub release upload, changelog, Docker images, Homebrew taps. Mature, heavy.

**Maintenance status.** Very active; first-class Rust support since 2024.

**Limitations.** Doesn't generate `install.sh` in the dist style by default. Doesn't write `[package.metadata.binstall]`. Adds a YAML config file in a different schema from anything else in the repo. Right tool for polyglot orgs; overkill for a Rust-only project where dist exists.

### release-plz (complementary, not a binary distributor)

Useful for the **crates.io** half: opens version-bump PRs, publishes in topological order, tags releases. Does not build binaries. Worth adopting *in addition to* dist — release-plz handles the crates.io publish, dist handles the binary release. Both run on tag push and don't conflict.

## Target platform matrix

| Platform                | Target triple                  | Tier | Binary names                                              | Notes |
|-------------------------|--------------------------------|------|-----------------------------------------------------------|-------|
| Linux x86_64 (musl)     | `x86_64-unknown-linux-musl`    | 1    | `reposix`, `git-remote-reposix`, `reposix-sim`            | Default Linux artifact. Static; runs on Alpine, Debian, RHEL, Amazon Linux. Avoids glibc-version pain. |
| Linux aarch64 (musl)    | `aarch64-unknown-linux-musl`   | 1    | same                                                      | Apple Silicon Linux VMs, Graviton, Raspberry Pi 64-bit. Built via `cargo zigbuild`. |
| macOS aarch64           | `aarch64-apple-darwin`         | 1    | same                                                      | Apple Silicon Macs (the user's machine). Native build on `macos-14` runner. |
| macOS x86_64            | `x86_64-apple-darwin`          | 1    | same                                                      | Intel Macs. Cross-compile from `macos-14` runner. |
| Windows x86_64          | `x86_64-pc-windows-msvc`       | 1    | `reposix.exe`, `git-remote-reposix.exe`, `reposix-sim.exe`| Native build on `windows-2022`. `rusqlite` bundled handles SQLite. |
| Windows aarch64         | `aarch64-pc-windows-msvc`      | —    | —                                                         | **Skip in v0.11.0.** Cross-compile only; ring/rustls historically rough; no demand yet. |
| Linux x86_64 glibc      | `x86_64-unknown-linux-gnu`     | —    | —                                                         | **Drop.** musl supersedes it; keeping both doubles the artifact list with no functional benefit. |

**Why musl-default for Linux.** rusqlite-bundled and reqwest-rustls both compile cleanly to musl (verified on the rust-cli-book and rusqlite-issue-914 references). Static binaries Just Work across distros. The only musl gotcha — `reqwest` pulling in `openssl-sys` when not configured for rustls — is already moot for us because workspace `Cargo.toml` sets `reqwest = { ..., default-features = false, features = ["json", "rustls-tls"] }`. Drop the gnu target.

## Setup steps for reposix

1. **Verify build prerequisites are clean for new targets** (one-shot local sanity check before invoking dist):
   - `rustup target add x86_64-unknown-linux-musl aarch64-apple-darwin x86_64-pc-windows-msvc`
   - `cargo zigbuild --release -p reposix-cli -p reposix-remote --target x86_64-unknown-linux-musl` (after `pip install cargo-zigbuild`).
   - Smoke-test rusqlite-bundled on the musl target. Expected to pass — rusqlite/issue-914 confirms this is the canonical workaround.
2. **Add crates.io metadata**: every workspace crate already has `description`, `license`, `repository`, `homepage` set via `workspace.package`. Add `readme = "../../README.md"` to any crate missing it (the binary crates have it). No `publish = false` exists, so defaults are correct.
3. **Install dist:** `cargo install cargo-dist --locked` (yields `dist` binary).
4. **Run `dist init`** in repo root. Answer:
   - Targets: 5-target matrix above.
   - Installers: `shell`, `powershell`, `homebrew`.
   - Hosting: GitHub Releases.
   - Tap repo: `reubenjohn/homebrew-reposix` (create empty repo first).
   - Cargo profile: `release`.
   - Multi-package mode: workspace, but only build binaries from `reposix-cli` and `reposix-remote` (dist supports filtering).
5. **Review generated `[workspace.metadata.dist]` block** in `Cargo.toml`. Pin `cargo-dist-version` so CI is reproducible.
6. **Replace `.github/workflows/release.yml`** with the dist-generated one. Migrate the custom CHANGELOG-extraction by handing dist `--changelog-path CHANGELOG.md` (it understands Keep-a-Changelog format).
7. **Add release-plz** as a separate workflow (`.github/workflows/release-plz.yml`) for the crates.io publish half, with the topological order baked in (see next section).
8. **Wire secrets:** `CARGO_REGISTRY_TOKEN` (crates.io), `HOMEBREW_TAP_TOKEN` (PAT scoped to `homebrew-reposix`).
9. **Dry-run on a pre-release tag** like `v0.11.0-rc.1` — dist supports the `--dry-run` flag, but a real RC tag exercises the whole pipeline.
10. **Tag `v0.11.0`** → release pipeline produces tarballs, installers, Homebrew bottle bump, crates.io publishes, binstall metadata is live.
11. **Verify (close the loop, OP-1):**
    - `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/reubenjohn/reposix/releases/download/v0.11.0/reposix-installer.sh | sh` on a fresh Linux VM.
    - `brew install reubenjohn/reposix/reposix` on the Mac.
    - `cargo binstall reposix-cli --version 0.11.0` on a third box.
    - For each, run `reposix --version` and `git-remote-reposix --help` to prove both binaries landed.

## crates.io publish question

**Yes, publish.** Three reasons:
1. Without a crates.io entry, `cargo binstall reposix-cli` and `cargo install reposix-cli` both fail. Those are two of the four install paths the user wants (the other two — `curl | sh` and `brew install` — work without crates.io).
2. crates.io presence is a discoverability signal. A crate that does not publish reads as "not for external use" to most Rust devs.
3. The workspace already has all the required metadata (`description`, `license = "MIT OR Apache-2.0"`, `repository`, `homepage`). No `publish = false` exists. The work is purely operational.

**Topological publish order** (each must land on crates.io before its dependents try):

1. `reposix-core` — leaf; no internal deps.
2. `reposix-cache` — depends on `reposix-core`.
3. `reposix-sim` — depends on `reposix-core`.
4. `reposix-github`, `reposix-confluence`, `reposix-jira` — each depends on `reposix-core` (+ `reposix-cache`). Can publish in parallel after step 2.
5. `reposix-remote` — depends on core, cache, all three backends.
6. `reposix-cli` — depends on everything in 1–5 plus `reposix-sim`.
7. `reposix-swarm` — internal test harness; **set `publish = false`**; not user-facing.

The path-dep `{ path = "../reposix-cache" }` form must be augmented to `{ path = "...", version = "0.11.0" }` so cargo accepts the publish; this is a one-line edit per consumer crate. release-plz handles the version bumps automatically once configured.

**Reserve names early.** Even if we don't ship 0.11.0 today, publish a `0.11.0-alpha.0` placeholder for each crate so the names are registered and a malicious squatter can't grab `reposix-cli`. (Crates.io does not allow name takeovers, but does allow "abandoned name reclaim" after 90 days — a reserved alpha closes that window.)

## Tutorial step rewrite (first-run.md step 1)

Current step 1 is a `cargo build`. Replace with:

```markdown
## 1. Install reposix

Pick whichever of these matches your machine. Each one ends with `reposix` and `git-remote-reposix` on your `PATH`. **No Rust toolchain required for the prebuilt paths.**

**Linux & macOS — one-line installer (recommended):**

    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh

**Windows (PowerShell):**

    powershell -ExecutionPolicy Bypass -c "irm https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.ps1 | iex"

**Homebrew (macOS, Linux):**

    brew install reubenjohn/reposix/reposix

**cargo-binstall (any platform with a Rust toolchain installed):**

    cargo binstall reposix-cli

**From source (slowest; useful for hacking on reposix itself):**

    cargo install --locked --git https://github.com/reubenjohn/reposix reposix-cli reposix-remote

Verify:

    reposix --version
    git-remote-reposix --help

Both binaries must resolve. The git remote helper is what `git fetch` and `git push` invoke under the hood when the URL starts with `reposix::`; if it is missing from `PATH`, step 6 will fail with `git: 'remote-reposix' is not a git command`.
```

Step 2 onward stays as-is (the existing `cargo run -p reposix-sim` becomes `reposix sim` once the CLI is installed; both forms keep working since `cargo run` is still valid for source checkouts).

## Risks / open questions

1. **dist Cargo.toml schema churn.** dist has had breaking-config changes between minor versions (0.6 → 0.22 → 0.31). Mitigation: pin `cargo-dist-version` in our config; subscribe to dist release notes; treat dist upgrades as their own GSD phase.
2. **Homebrew tap repo bootstrap.** Requires a separate empty repo (`reubenjohn/homebrew-reposix`) and a PAT secret. Without it, dist's homebrew installer setting fails on first run. Cheap to set up but easy to forget; put it in the v0.11.0 PLAN as an explicit task.
3. **musl + git protocol-v2.** The bare-repo cache in `reposix-cache` shells out to `git` (>= 2.34) for protocol-v2 fetch. Static musl binaries don't bundle git; the *user* must have `git >= 2.34` on PATH. Document explicitly in the tutorial. (The current `Tech stack` section in CLAUDE.md already lists this requirement.)
4. **Windows MSVC + bundled rusqlite.** Bundled rusqlite needs a C compiler at build time. GitHub-hosted `windows-2022` runners ship MSVC, so dist's default config works. Worth a smoke test on the first run; called out in the verification step above.
5. **crates.io reservation race.** If we hold off publishing reservations to crates.io, someone else could grab `reposix-cli`. Mitigate by publishing `0.11.0-alpha.0` placeholders within the v0.11.0 phase even if the rest of the pipeline takes longer.
6. **Audit-log promise during install.** The tutorial's audit-row promise (step 7) depends on the `reposix sim` binary being able to write to `~/.cache/reposix/`. Static musl binary creates the dir lazily — no install-time setup needed — but document the path explicitly.
7. **Signature story.** dist supports minisign but not sigstore/cosign as of 0.31.0. If supply-chain signing becomes a hard requirement (lethal-trifecta threat model says it should), we may need to bolt cosign on top. Out of scope for v0.11.0; track in the v0.12.x backlog.
8. **release-plz / dist coordination.** Both run on tag push. Verify they don't race for the GitHub release object: dist creates the release; release-plz only publishes to crates.io. Their workflows are orthogonal in practice but worth a dry-run.

## Sources

- [axodotdev/cargo-dist GitHub](https://github.com/axodotdev/cargo-dist) — repo title "dist (formerly known as cargo-dist)", v0.31.0 (2026-02-23) latest, 271 releases.
- [axodotdev/cargo-dist Releases](https://github.com/axodotdev/cargo-dist/releases) — release cadence through 2026.
- [cargo-dist documentation](https://axodotdev.github.io/cargo-dist/) — supported target triples (matches the 5-target matrix above).
- [Arch Linux cargo-dist 0.31.0](https://archlinux.org/packages/extra/x86_64/cargo-dist/) — distro packaging confirms current version.
- [cargo-bins/cargo-binstall GitHub](https://github.com/cargo-bins/cargo-binstall) — `[package.metadata.binstall]` schema and discovery flow.
- [cargo-binstall release 2026-04-13](https://github.com/cargo-bins/cargo-binstall/releases) — recent activity.
- [astral-sh/uv release pipeline (DeepWiki)](https://deepwiki.com/astral-sh/uv/11.1-build-system-and-release-pipeline) — production reference for cargo-dist on a Rust workspace.
- [astral-sh/ruff release process](https://deepwiki.com/astral-sh/ruff/8.4-release-process) — 18+-target dist pipeline pattern.
- [rusqlite issue #914 — segfault on cross-compile to musl](https://github.com/rusqlite/rusqlite/issues/914) — confirms `bundled` feature is the canonical musl fix.
- [rusqlite issue #1201 — cross-compile](https://github.com/rusqlite/rusqlite/issues/1201) — bundled works on Windows + musl without system libsqlite3.
- [cross-rs/cross issue #243 — reqwest aarch64-musl](https://github.com/rust-embedded/cross/issues/243) — confirms rustls-tls is the cross-compile fix; openssl-sys is the trap.
- [GoReleaser Rust support](https://goreleaser.com/customization/builds/rust/) — alternative considered and rejected.
- [Cargo workspace publishing reference](https://doc.rust-lang.org/cargo/reference/publishing.html) — topological-order rule for path-dep workspaces.
- [Tweag — Publish all your crates everywhere all at once](https://www.tweag.io/blog/2025-07-10-cargo-package-workspace/) — modern multi-crate publish patterns.

Word count: ~1,950.
