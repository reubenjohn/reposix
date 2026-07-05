# P94 D3 — badges-resolve real-vs-transient determination

**Row:** `docs-build/p94-badges-real-vs-transient`
**Verdict:** **TRANSIENT** (upstream shields.io/Codecov/GitHub badge-endpoint flake — NOT a broken badge URL)
**Determined:** 2026-07-05
**Resolution:** bounded retry/backoff added to `quality/gates/docs-build/badges-resolve.py`

## The friction (as filed)

`badges-resolve` FAILed on pre-push during P93 Wave 1 (and was re-observed as a
pre-push blocker alongside the P93 clippy failure — see SURPRISES-INTAKE
"Pre-push BLOCKED" entry, same window). The filed hypothesis: a shields.io /
Codecov transient network flake rather than a genuinely-broken badge URL, but
unconfirmed. GOOD-TO-HAVES.md badges-resolve entry (MEDIUM/P2) asked for
multiple isolated re-runs spaced apart to distinguish real from transient.

## Evidence — ≥2 spaced isolated re-runs (the determination)

Each run HEADs all 10 badge URLs extracted from `README.md` + `docs/index.md`
and asserts HTTP 200 + image/json content-type.

| Run | UTC time     | Result                          | exit |
|-----|--------------|---------------------------------|------|
| 1   | 23:48:04Z    | 10 PASS, 0 FAIL, 0 pending      | 0    |
| 2   | 23:48:37Z    | 10 PASS, 0 FAIL, 0 pending      | 0    |
| 3   | 23:56:36Z    | 10 PASS, 0 FAIL (post-fix, all `attempts:1`) | 0 |

All three isolated runs, spaced across ~8 minutes, passed cleanly with every
badge returning 200 on the first attempt. No badge URL was ever a deterministic
404 / wrong-content-type across runs. **Conclusion: the pre-push failure was a
TRANSIENT upstream flake** — a single HEAD to a shields.io/Codecov/GitHub badge
endpoint intermittently timed out or returned a transient 5xx/429 under load,
and the old single-shot `head_url` failed the whole gate on that one hiccup.

The 10 badge URLs (all confirmed live, HTTP 200 + correct content-type):
- 4× GitHub Actions `badge.svg` (ci / quality-weekly ×2 / docs) — `image/svg+xml`
- 1× GitHub legacy `workflows/CI/badge.svg` — `image/svg+xml`
- 1× shields.io endpoint (QG-09 quality badge.json) — `image/svg+xml`
- 1× codecov `graph/badge.svg` — `image/svg+xml`
- 3× shields.io static (License-MIT / rust-stable / crates.io reposix-cli) — `image/svg+xml`

## Fix applied (TRANSIENT branch of the catalog contract)

The catalog row's assert for the transient case: *"badges-resolve.py gains a
retry/backoff before failing OR a documented waiver note is added."* A retry is
strictly better than a waiver here — a waiver would blanket-suppress a real
future breakage and would expire, re-surfacing the same flake. Instead,
`head_url()` now retries a TRANSIENT failure (network error, or HTTP
408/425/429/5xx) up to `MAX_ATTEMPTS = 3` with `BACKOFF_S = (1.0, 2.0)`
spacing. A DETERMINISTIC failure (404/403/other-4xx, or a wrong content-type)
still fails on the first attempt — the retry cannot mask a genuinely-dead
badge. The per-URL `attempts` count is recorded in the verifier artifact for
transparency.

Net: `python3 quality/gates/docs-build/badges-resolve.py` exits 0; the
`docs-build/badges-resolve` row reaches green reliably instead of flaking RED
on pre-push.

## Re-run to reproduce

```bash
python3 quality/gates/docs-build/badges-resolve.py   # exit 0; artifact records per-URL attempts
```
