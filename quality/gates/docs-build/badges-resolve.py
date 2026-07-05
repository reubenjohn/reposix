#!/usr/bin/env python3
"""quality/gates/docs-build/badges-resolve.py -- BADGE-01 verifier (P60 Wave C).

HEADs every badge URL extracted from README.md + docs/index.md and asserts
HTTP 200 + Content-Type contains 'image' (or 'json' for the QG-09
endpoint URL).

Exit codes:
  0 -- PASS (all URLs 200 + correct content-type)
  1 -- FAIL (any 4xx/5xx OR wrong content-type)
  2 -- PARTIAL (transient or documented Wave-F-pending URLs)

Post-Wave-F (2026-04-27): WAVE_F_PENDING_URLS is empty; all 8 badge
URLs (including the QG-09 endpoint) HEAD'd unconditionally + PASS.
The skip-branch in main() is preserved as a no-op for future
multi-wave URL migrations.

Honors --row-id <id> for catalog discrimination (the row in
quality/catalogs/docs-build.json + the back-compat row in
quality/catalogs/freshness-invariants.json:structure/badges-resolve
both point at this single verifier; the artifact path is row-specific).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
ARTIFACT_DIR_DOCS_BUILD = REPO_ROOT / "quality" / "reports" / "verifications" / "docs-build"
ARTIFACT_DIR_STRUCTURE = REPO_ROOT / "quality" / "reports" / "verifications" / "structure"

# Markdown badge image extraction: matches ![<alt>](https://...image-url...)
# This regex captures the INNER image URL only -- the bracket-bang prefix is
# required, so the OUTER link target in [![alt](inner)](outer) is NOT matched.
BADGE_IMG_RE = re.compile(r"!\[[^\]]*\]\((https?://[^)]+)\)")

# QG-09 endpoint URLs landed in P60 Wave F (2026-04-27); set is empty.
# The skip-branch in main() is now a no-op; all URLs HEAD'd unconditionally.
# Future maintenance: if a new URL needs to land via a multi-wave migration,
# repopulate this set with a comment naming the target wave + expected
# unblock date.
WAVE_F_PENDING_URLS: set[str] = set()

TIMEOUT_S = 10
USER_AGENT = "reposix-quality-gates/1.0"

# P94 D3 (docs-build/p94-badges-real-vs-transient): bounded retry/backoff for
# TRANSIENT upstream flakes. The badges-resolve check FAILed intermittently on
# pre-push across P93/P94 (SURPRISES-INTAKE + GOOD-TO-HAVES badges-resolve
# entry); two spaced isolated re-runs (2026-07-05, both exit 0, all 10 badges
# HTTP 200) proved the failure is a TRANSIENT shields.io/Codecov/GitHub badge
# endpoint flake, NOT a broken URL. A single network hiccup or a transient 5xx/
# 429 from shields.io used to fail the whole gate; we now retry those before
# declaring FAIL, so the check reaches green reliably. A DETERMINISTIC failure
# (404/403 or a wrong content-type) is NOT retried — a genuinely-dead badge
# still fails fast, so the retry cannot mask a real breakage.
MAX_ATTEMPTS = 3
BACKOFF_S = (1.0, 2.0)  # slept after attempt 1 and attempt 2 (bounded)
# Statuses treated as transient (retryable): request timeout, too-early, rate
# limit, and the 5xx server-side family shields.io/Codecov emit under load.
TRANSIENT_HTTP = frozenset({408, 425, 429, 500, 502, 503, 504})


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def extract_badge_urls(path: Path) -> list[str]:
    """Return deduplicated list of badge image URLs from one markdown file."""
    text = path.read_text(encoding="utf-8")
    seen: list[str] = []
    for match in BADGE_IMG_RE.finditer(text):
        url = match.group(1)
        if url not in seen:
            seen.append(url)
    return seen


def head_url(url: str) -> tuple[int | None, str | None, str | None, int]:
    """HEAD `url`, retrying TRANSIENT failures with bounded backoff.

    Returns (status_code, content_type, error_message, attempts). A transient
    failure — a network error (status None: DNS/connect/timeout) or a transient
    HTTP status (`TRANSIENT_HTTP`: 408/425/429/5xx) — is retried up to
    MAX_ATTEMPTS with BACKOFF_S spacing before the last result is returned. A
    DETERMINISTIC status (200 success; 404/403/other 4xx failure) returns
    immediately on the first attempt, so a genuinely-broken badge still fails
    fast and the retry cannot mask a real breakage (P94 D3 determination:
    the recurring pre-push failure was a transient upstream flake).
    """
    req = urllib.request.Request(
        url, method="HEAD", headers={"User-Agent": USER_AGENT}
    )
    status: int | None = None
    ctype: str | None = None
    err: str | None = None
    for attempt in range(1, MAX_ATTEMPTS + 1):
        try:
            with urllib.request.urlopen(req, timeout=TIMEOUT_S) as resp:
                return resp.status, resp.headers.get("Content-Type", ""), None, attempt
        except urllib.error.HTTPError as e:
            ctype = e.headers.get("Content-Type", "") if e.headers else ""
            status, err = e.code, str(e)
            if e.code not in TRANSIENT_HTTP:
                return status, ctype, err, attempt  # deterministic — do not retry
        except (urllib.error.URLError, TimeoutError, OSError) as e:
            status, ctype, err = None, None, str(e)
        if attempt < MAX_ATTEMPTS:
            time.sleep(BACKOFF_S[min(attempt - 1, len(BACKOFF_S) - 1)])
    return status, ctype, err, MAX_ATTEMPTS


def main() -> int:
    parser = argparse.ArgumentParser(description="BADGE-01 verifier")
    parser.add_argument("--row-id", default="docs-build/badges-resolve")
    args = parser.parse_args()

    # Collect URLs from README.md + docs/index.md (deduped across files).
    urls: list[str] = []
    for src in [REPO_ROOT / "README.md", REPO_ROOT / "docs" / "index.md"]:
        for u in extract_badge_urls(src):
            if u not in urls:
                urls.append(u)

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    url_results: dict[str, dict] = {}
    has_partial = False

    for url in urls:
        if url in WAVE_F_PENDING_URLS:
            url_results[url] = {
                "status": "WAVE-F-PENDING",
                "note": "URL lands in P60 Wave F (QG-09 publish)",
            }
            has_partial = True
            continue

        status, ctype, err, attempts = head_url(url)
        url_results[url] = {
            "status": status, "content_type": ctype, "error": err,
            "attempts": attempts,
        }

        ctype_l = (ctype or "").lower()
        if status != 200:
            asserts_failed.append(
                f"{url} HEAD => {status} after {attempts} attempt(s) "
                f"(expected 200) [{err}]"
            )
        elif "image" not in ctype_l and "json" not in ctype_l:
            asserts_failed.append(
                f"{url} HEAD => Content-Type {ctype!r} (expected image/* or */json)"
            )
        else:
            asserts_passed.append(f"{url} HEAD => 200 + {ctype}")

    # Determine exit code.
    if asserts_failed:
        exit_code = 1  # FAIL
    elif has_partial:
        exit_code = 2  # PARTIAL (Wave F pending)
    else:
        exit_code = 0  # PASS

    artifact = {
        "ts": now_iso(),
        "row_id": args.row_id,
        "exit_code": exit_code,
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
        "badge_urls_checked": list(urls),
        "url_results": url_results,
    }

    # Artifact path discriminated by --row-id so the dimension-native row
    # and the back-compat row write to distinct artifact files.
    if args.row_id == "structure/badges-resolve":
        artifact_path = ARTIFACT_DIR_STRUCTURE / "badges-resolve.json"
    else:
        artifact_path = ARTIFACT_DIR_DOCS_BUILD / "badges-resolve.json"
    artifact_path.parent.mkdir(parents=True, exist_ok=True)
    artifact_path.write_text(
        json.dumps(artifact, indent=2) + "\n", encoding="utf-8"
    )

    pending = sum(
        1 for v in url_results.values() if v.get("status") == "WAVE-F-PENDING"
    )
    print(
        f"badges-resolve: {len(asserts_passed)} PASS, "
        f"{len(asserts_failed)} FAIL, {pending} pending; exit={exit_code}"
    )
    for url, res in url_results.items():
        print(f"  {url} => {res}")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
