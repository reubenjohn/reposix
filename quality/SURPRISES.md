# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0-autonomous-execution-protocol.md`
§ "SURPRISES.md format": append one line per unexpected obstacle +
its one-line resolution. **Required reading for the next phase agent.**
The next agent does NOT repeat investigations of things already
journaled here. Format: `YYYY-MM-DD P<N>: <what happened> — <one-line
resolution>`. Anti-bloat: ≤200 lines. When it crosses, archive oldest
50 to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh. Seeded
by P56 (Wave 4-B); P57 takes ownership when the framework skeleton ships.

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

Anti-bloat: ≤200 lines (currently 65). When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface" for the rotation rule.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

---

2026-04-27 P56: GitHub's `releases/latest/download/...` pointer follows
release recency, but release-plz cuts ~8 per-crate releases per version
bump. A non-cli per-crate release published after the cli release moves
the latest pointer and re-breaks `releases/latest/download/reposix-installer.sh`
until the next cli release. — Tracked under MIGRATE-03 (v0.12.1
carry-forward). Recovery options: (a) `gh release create --latest` to
pin the pointer to the cli release in release.yml, or (b) configure
release-plz to publish reposix-cli last in its per-crate sequence.

2026-04-27 P56: release-plz GITHUB_TOKEN-pushed tags do NOT trigger
downstream `on.push.tags` workflows — GH loop-prevention rule for
GITHUB_TOKEN-pushed refs. release.yml's `reposix-cli-v*` glob is
correct, but the tag push from release-plz never fires the workflow.
— Workaround: `gh workflow run release.yml --ref reposix-cli-v0.11.3`
(workflow_dispatch) used as Wave 3 stop-gap. Fix path under MIGRATE-03
(v0.12.1 carry-forward): release-plz workflow uses fine-grained PAT
(non-GITHUB_TOKEN) OR adds a post-tag dispatch step. ~5 LOC.

2026-04-27 P56: install/cargo-binstall metadata in
`crates/reposix-cli/Cargo.toml` + `crates/reposix-remote/Cargo.toml`
is misaligned with release.yml's archive shape (4 mismatches: tag
prefix `v` vs `reposix-cli-v`, archive basename `reposix-cli` vs
`reposix`, target glibc vs musl, `.tgz` vs `.tar.gz`). binstall falls
back to source build, which then itself fails because of the MSRV
bug below. — Catalog row marked PARTIAL (blast_radius P1, "works just
slow"); ~10 LOC `[package.metadata.binstall]` fix tracked under
MIGRATE-03 (v0.12.1 carry-forward).

2026-04-27 P56: Rust 1.82 (project MSRV) cannot `cargo install reposix-cli`
from crates.io — transitive dep `block-buffer-0.12.0` requires
`edition2024` which is unstable on 1.82. cargo install ignores the
project's pinned Cargo.lock so this is invisible to ci.yml's `test`
job (which builds against the workspace lockfile). — Orthogonal MSRV
bug; fix under MIGRATE-03 (v0.12.1 carry-forward) is either cap dep
at `<0.12` or raise MSRV to 1.85.

2026-04-27 P56: curl rehearsal `curl -sLI URL | head -20` under
`set -euo pipefail` exits 23 (FAILED_WRITING_OUTPUT) when GitHub's
HEAD response exceeds 20 lines (their content-security-policy
header is huge); pipefail propagates and bash exits before running
the installer step. — Fixed in `scripts/p56-rehearse-curl-install.sh`:
capture HEAD to a tempfile, then `head -20` on the static file. Same
diagnostic value, no SIGPIPE. Lesson for future verifiers: tempfile-then-grep,
not pipe-into-head, when the upstream response size is unbounded.
