# quality/gates/docs-repro/

Verifiers backing `quality/catalogs/docs-reproducible.json` (8 rows at Wave A close + 1 row added in Wave B for snippet-coverage; docs-repro dimension).

| Verifier | Catalog rows backed | Cadence |
|---|---|---|
| `snippet-extract.py` | `docs-repro/snippet-coverage` (Wave B) | pre-push |
| `container-rehearse.sh` | `docs-repro/example-0[1,2,4,5]-*` + `docs-repro/tutorial-replay` (Wave C) | post-release |
| `tutorial-replay.sh` | `docs-repro/tutorial-replay` (Wave C; SIMPLIFY-06 + DOCS-REPRO-03 migration target) | post-release |
| `manual-spec-check.sh` | `docs-repro/example-03-claude-code-skill` (Wave C; manual kind) | on-demand |

## Conventions

- **Stdlib-only Python.** Imports limited to `argparse`, `json`, `pathlib`, `subprocess`, `urllib.request`, `datetime`, `sys`, `hashlib`, `re`, `os`. No `requests`, `yaml`, `click`.
- **Default container:** `ubuntu:24.04` (apt-get pre-installs `curl ca-certificates python3 git build-essential`).
- **Cadences:** `kind: container` rows run `post-release` (cost budget); `snippet-extract.py --check` runs `pre-push` (drift-only, no execution).
- **Subprocess safety.** Every `subprocess.run` uses `shell=False` with list args.
- **Graceful docker-absent path.** `container-rehearse.sh` emits `NOT-VERIFIED` artifact + exits 0 when docker is unavailable; runner sees status, applies waiver if attached.
- **Per-verifier line caps:** `snippet-extract.py` <=250 lines; `container-rehearse.sh` + `tutorial-replay.sh` <=150 lines; `manual-spec-check.sh` <=50 lines.
- **Banned-word policy.** Per `quality/gates/structure/banned-words.sh`: prefer `migrate to` / `rewrite as` / `alongside`.

## Pivot rules

- **Container time budget >15min/release for the post-release matrix:** drop multi-persona; ubuntu-only first; mac/windows defer to v0.12.1.
- **>50 fenced code blocks in user-facing docs:** switch `snippet-extract.py` to allow-list mode (explicit catalog inclusion of tracked blocks; uncatalogued blocks not flagged); document cutover in `quality/SURPRISES.md` and `quality/PROTOCOL.md`.
- **Per-snippet rehearsal exceeds 60s on a cold container:** keep that row at `cadence: post-release` only; do NOT block pre-push.

## Cross-references

- `quality/catalogs/docs-reproducible.json` -- 8-row catalog (Wave A; +1 in Wave B)
- `quality/PROTOCOL.md` -- runner contract
- `quality/SURPRISES.md` -- docs-repro pivots journal
- `.planning/docs_reproducible_catalog.json` -- DEPRECATED seed (5 install rows already migrated to release-assets.json by P58)
