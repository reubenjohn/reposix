# Top-level files audit

[← index](./index.md)

| File | Verdict | Notes |
|---|---|---|
| `LICENSE-MIT` + `LICENSE-APACHE` | **KEEP** | Required for crates.io publish + dual-license `MIT OR Apache-2.0` advertised in `Cargo.toml`. |
| `README.md` | **KEEP** | Recently rewritten in Phase 45 (332→102 lines). Still mentions FUSE in v0.9.0 release-notes section — that's correct historical context, not stale code. |
| `CHANGELOG.md` | **KEEP** | 68 KB but exhaustive; CI doc-clarity skill verified. FUSE references are all in release-note sections (correctly historical). |
| `CONTRIBUTING.md` | **KEEP** | 148 lines, 8 KB. Read line-by-line: zero `reposix-fuse` / `reposix mount` / `fusermount` references in active prose. Aligned with v0.9.0+ pivot. |
| `CODE_OF_CONDUCT.md` | **KEEP** | Standard CC text; referenced by CONTRIBUTING. |
| `SECURITY.md` | **KEEP** | Threat-model summary; cites v0.1-fuse-era research as historical. |
| `CLAUDE.md` | **KEEP** | The agent-onboarding spec; referenced by every subagent run. |
| `PUBLIC-LAUNCH-CHECKLIST.md` | **KEEP, but FIX** | 30 unchecked checkboxes; references `scripts/tag-v0.10.0.sh` despite project being on v0.11.0. **UPDATE** the tag command to `scripts/tag-v0.11.0.sh` once that script is authored. Otherwise current. |
| `Cargo.toml` + `Cargo.lock` | **KEEP** | Workspace declared 9 crates correctly. Version `0.11.0` (note: STATE.md says owner needs to bump `0.11.0-dev` → `0.11.0` — current `Cargo.toml` already has `0.11.0`, so the bump may already be done; double-check before tag push). |
| `mkdocs.yml` | **KEEP** | Has uncommitted modification per `git status`. |
| `clippy.toml` | **KEEP** | 4 lines; one purpose; correct. |
| `rust-toolchain.toml` | **KEEP** | Pins stable + components. |
| `rustfmt.toml` | **KEEP** | 3 lines. |
| `deny.toml` | **KEEP** | License + advisory + bans + sources allowlist; aligned with `LICENSE-{MIT,APACHE}` files. |
| `.gitignore` | **KEEP** | Comprehensive; includes the `.claude/skills/` carve-out per OP-4. |
| `.env.example` | **KEEP** | 2.2 KB; documents env-var names. |
| `.pre-commit-config.yaml` | **KEEP** | Single banned-words hook; aligned with CI. |
| `.editorconfig` | **MISSING** (not present) | Optional; would help editor consistency. Not a blocker. |
| `.gitattributes` | **MISSING** | Optional; useful for `* text=auto` + `*.png binary` declarations. Not a blocker. |
| `requirements-bench.txt` | **KEEP** | 18 bytes; pins `pytest` for `bench_token_economy.py` tests. |
| `benchmarks/{README.md,RESULTS.md,fixtures/}` | **KEEP** | OP-8 token-economy results; live + cited. |
| `examples/{01..05}/` | **KEEP** | All 5 dirs current per `PUBLIC-LAUNCH-CHECKLIST` line "currently 5". |
| `research/scratch/` | **DELETE empty dir** | `git ls-files | grep '^research/scratch'` returns 0 files — only `.gitignore` keeps it tracked. The README convention prevents committing scratch; the empty dir adds noise. |

Hidden-cruft alignment:

- `clippy.toml` (workspace) + `[lints]` blocks in individual crates (not inspected here) should not contradict; spot-checked clippy.toml-banned-methods are catchable from the `check_clippy_lint_loaded.sh` script.
- `rust-toolchain.toml` channel `stable` + `Cargo.toml` `rust-version = "1.82"` — consistent.
- `deny.toml` license allowlist matches `LICENSE-{MIT,APACHE}` (`MIT OR Apache-2.0` is on the list).
- `mkdocs.yml` + `docs/` IA + `docs/.banned-words.toml` — checked by `check-docs-site.sh`.
- No contradictions found between the configs.
