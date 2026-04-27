# quality/gates/docs-build/

Verifiers backing `quality/catalogs/docs-build.json` (4 rows, docs-build dimension). Plus the back-compat shared verifier `badges-resolve.py` co-anchored from `quality/catalogs/freshness-invariants.json:structure/badges-resolve` (the P57 row points at the same file; both rows discriminate via `--row-id`).

| Verifier | Catalog rows backed | Cadence |
|---|---|---|
| `mkdocs-strict.sh` | `docs-build/mkdocs-strict` | pre-push |
| `mermaid-renders.sh` | `docs-build/mermaid-renders` | pre-push |
| `link-resolution.py` | `docs-build/link-resolution` | pre-push |
| `badges-resolve.py` | `docs-build/badges-resolve` + `structure/badges-resolve` | pre-push + weekly |

## Conventions

- **Stdlib-only Python.** Imports limited to `argparse`, `json`, `pathlib`, `subprocess`, `urllib.request`, `datetime`, `sys`, `hashlib`, `re`, `os`, `html.parser`. No `requests`, `yaml`, `click`.
- **Bash POSIX-where-possible.** `set -euo pipefail`, `readonly` for constants, `$(dirname "${BASH_SOURCE[0]}")` for self-relative path arithmetic.
- **Per-verifier line caps:** `<=250` lines. This README: `<=50` lines. The migrated pre-push hook body: `<=30` lines per P60 Wave E SIMPLIFY-10.
- **Exit codes** per the QG-04 contract: `0` = PASS, `1` = FAIL, `2` = PARTIAL.
- **`ci_friendly`.** Verifiers run on warm cache in under their catalog `timeout_s`; cold cache is documented as worst-case in row owner_hint.
- **Banned-word policy.** Per `quality/gates/structure/banned-words.sh`: prefer `migrate to` / `rewrite as` / `alongside`.

## Pivot rules

- **Path-move shim rule (OP-5 reversibility).** When a verifier moves from `scripts/` to here, leave a thin shim at the old path (`exec bash`/`exec python3` to the canonical home). Shim survives one merge cycle; P63 SIMPLIFY-12 audits.
- **Chicken-and-egg endpoint badge URL.** `badges-resolve.py` maintains a `WAVE_F_PENDING_URLS` set covering URLs whose host is reachable only after Wave F seeds `docs/badge.json`. Pending URLs grade PARTIAL until Wave F lands the publish + the README badge edit.
- **Greater than 5 broken links found by `link-resolution.py`.** Fix the worst (top 5), file backlog item for the rest, do NOT block phase close.

## Cross-references

- `quality/catalogs/docs-build.json` -- 4-row catalog (Wave A)
- `quality/catalogs/freshness-invariants.json:structure/badges-resolve` -- back-compat row (P57 anchored, shares verifier with docs-build/badges-resolve)
- `quality/PROTOCOL.md` -- runner contract
- `CLAUDE.md` § "Docs-site validation" -- mkdocs-strict + mermaid playwright walks rationale
