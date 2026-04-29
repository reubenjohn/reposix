# quality/gates/ — Verifier scripts by dimension

Each subdirectory is one dimension of the Quality Gates framework.
A dimension groups related regression checks (e.g., all doc-build checks
live under `docs-build/`).

## Dimensions

| Directory | What it guards | Example gates |
|---|---|---|
| `agent-ux/` | Dark-factory agent experience | sim clone+grep+push round-trip |
| `code/` | Clippy, fmt, test suite | clippy-lint-loaded, cargo-fmt-clean |
| `docs-alignment/` | Doc claims have backing tests | hash-drift walk, coverage ratio |
| `docs-build/` | mkdocs builds cleanly | strict mode, mermaid renders, links |
| `docs-repro/` | Install/tutorial snippets work | container rehearsal, snippet extract |
| `perf/` | Latency and token budgets | golden-path latency envelope |
| `release/` | Release assets exist and work | gh assets, brew formula, installer |
| `security/` | Allowlist enforcement, audit | egress allowlist, audit immutability |
| `structure/` | Freshness invariants, file org | banned words, no loose roadmaps |
| `subjective/` | Human-judged rubrics with TTL | hero clarity, install positioning |

## Adding a new gate

1. Add a catalog row to `quality/catalogs/<dim>.json` (see `catalogs/README.md` for schema).
2. Write the verifier script in `quality/gates/<dim>/`.
3. Follow the exit-code contract: `0` = PASS, `1` = FAIL, `2` = PARTIAL.
4. Write the artifact to the path specified in the catalog row's `artifact` field.
5. The runner discovers it automatically — no wiring in `run.py` needed.

Each `gates/<dim>/README.md` documents the verifiers in that dimension.

## Conventions

- Bash for thin CLI wrappers; Python (stdlib only) for richer validation.
- Scripts must be runnable from repo root (`quality/gates/<dim>/script.sh`).
- Timeout specified per-row in the catalog (`verifier.timeout_s`); the runner enforces it.
- No network calls in `pre-push` cadence gates (must complete offline in <60s).
