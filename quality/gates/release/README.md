# quality/gates/release/

Verifiers backing `quality/catalogs/release-assets.json` (16 rows, release dimension).

| Verifier | Catalog rows backed | Cadence |
|---|---|---|
| `gh-assets-present.py` | `release/gh-assets-present` | weekly |
| `installer-asset-bytes.py` | `install/curl-installer-sh`, `install/powershell-installer-ps1`, `install/build-from-source` | weekly |
| `brew-formula-current.py` | `release/brew-formula-current`, `install/homebrew` (writes both artifacts) | weekly |
| `crates-io-max-version.py` | `release/crates-io-max-version/<crate>` (one row per published crate; 9 rows) | weekly |
| `cargo-binstall-resolves.py` | `release/cargo-binstall-resolves` | post-release |

## Conventions

- **Stdlib only.** No `requests`, `pyyaml`, `click`. Imports limited to
  `argparse`, `json`, `pathlib`, `subprocess`, `urllib.request`,
  `urllib.error`, `datetime`, `sys`, `re`, `base64`, `time`, `tomllib`
  (Python 3.11+, with regex fallback for `Cargo.toml`).
- **Exit codes.** `0` PASS, `1` FAIL, `2` PARTIAL. The runner maps these
  to catalog `status` per `quality/runners/run.py`.
- **Artifact path.** Each verifier writes `quality/reports/verifications/release/<slug>.json`
  with `asserts_passed: [...]` and `asserts_failed: [...]` arrays so the
  runner + verifier-subagent can grade against the catalog row's
  `expected.asserts`.
- **Subprocess safety.** Every `subprocess.run` uses `shell=False` with list
  args (no shell interpolation; no command injection via `--crate`/`--url`).
- **Rate-limit safety.** `crates-io-max-version.py` sleeps 1 s after each
  invocation so the runner's sequential 9-crate sweep stays at ~1 req/sec
  under the unauthenticated crates.io rate limit.

## Known PARTIAL state

`release/cargo-binstall-resolves` is documented PARTIAL until v0.12.1 lands
the `[package.metadata.binstall]` rewrite (`Falling back to source` signal).
See `quality/SURPRISES.md` (2026-04-27 P56 row 3) and `MIGRATE-03` carry-forward.

## Cross-references

- `quality/catalogs/release-assets.json` — 16-row catalog (Wave A, P58)
- `quality/PROTOCOL.md` § runner contract
- `quality/SURPRISES.md` — known release-pipeline pivots
