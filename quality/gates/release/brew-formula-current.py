#!/usr/bin/env python3
"""Release-dim verifier — brew-formula-current.

Backs catalog rows: release/brew-formula-current AND install/homebrew
(writes both artifacts).

Asserts:
- GET https://api.github.com/repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb returns 200
- formula's `version "X.Y.Z"` parses
- crates.io reposix-cli `max_version` parses
- formula version == crates.io max_version

Stdlib only (urllib.request, base64, re, json). Cross-platform.

Exit codes: 0 PASS | 1 FAIL.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"
UA = "reposix-quality-gates/0.12.0"
FORMULA_URL = "https://api.github.com/repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb"
CRATES_URL = "https://crates.io/api/v1/crates/reposix-cli"
VERSION_RE = re.compile(r'version\s+"(\d+\.\d+\.\d+(?:-[\w\.]+)?)"')


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def http_get_json(url: str, timeout: int = 30) -> tuple[int, Any, str]:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, json.loads(resp.read().decode("utf-8")), ""
    except urllib.error.HTTPError as e:
        return e.code, None, f"HTTPError: {e}"
    except (urllib.error.URLError, json.JSONDecodeError, OSError) as e:
        return 0, None, f"{type(e).__name__}: {e}"


def write_artifact(filename: str, payload: dict) -> None:
    out = ARTIFACT_DIR / filename
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="brew-formula-current — assert tap formula matches crates.io")
    parser.add_argument("--row-id", default=None, help="optional override (default: writes both artifacts)")
    args = parser.parse_args()

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    formula_version = None
    max_version = None

    fstatus, fbody, ferr = http_get_json(FORMULA_URL)
    if fstatus != 200 or fbody is None:
        asserts_failed.append(f"GET {FORMULA_URL} HTTP 200 — got status={fstatus}; err={ferr}")
    else:
        asserts_passed.append(f"GET {FORMULA_URL} returns HTTP 200")
        b64 = (fbody.get("content") or "").replace("\n", "").strip()
        try:
            formula_text = base64.b64decode(b64).decode("utf-8")
            m = VERSION_RE.search(formula_text)
            if m:
                formula_version = m.group(1)
                asserts_passed.append(f"formula version parsed: {formula_version}")
            else:
                asserts_failed.append("formula text does not contain `version \"X.Y.Z\"`")
        except (ValueError, UnicodeDecodeError) as e:
            asserts_failed.append(f"failed to base64-decode formula content: {e}")

    cstatus, cbody, cerr = http_get_json(CRATES_URL)
    if cstatus != 200 or cbody is None:
        asserts_failed.append(f"GET {CRATES_URL} HTTP 200 — got status={cstatus}; err={cerr}")
    else:
        asserts_passed.append(f"GET {CRATES_URL} returns HTTP 200")
        try:
            max_version = cbody["crate"]["max_version"]
            asserts_passed.append(f"crates.io reposix-cli max_version: {max_version}")
        except (KeyError, TypeError) as e:
            asserts_failed.append(f"crates.io response missing crate.max_version: {e}")

    if formula_version and max_version:
        if formula_version == max_version:
            asserts_passed.append(f"formula version == crates.io max_version ({formula_version})")
        else:
            asserts_failed.append(
                f"formula version {formula_version!r} != crates.io max_version {max_version!r}"
            )

    payload = {
        "ts": now_iso(),
        "row_id": "release/brew-formula-current",
        "formula_url": FORMULA_URL,
        "crates_url": CRATES_URL,
        "formula_version": formula_version,
        "crates_io_max_version": max_version,
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
    }
    write_artifact("brew-formula-current.json", payload)

    payload2 = dict(payload)
    payload2["row_id"] = "install/homebrew"
    write_artifact("install-homebrew.json", payload2)

    return 0 if not asserts_failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
