#!/usr/bin/env python3
"""Quality Gates release-dimension verifier — gh-assets-present.

Catalog row: release/gh-assets-present (quality/catalogs/release-assets.json).

Asserts that a GitHub Release for a given <repo>/<tag> has at least
<min-assets> assets uploaded, every asset.state == "uploaded".

Stdlib only (urllib.request, argparse, json, pathlib, datetime, sys).
Cross-platform (linux + macOS).

Usage:
  python3 quality/gates/release/gh-assets-present.py \\
      --repo reubenjohn/reposix --tag latest --min-assets 7

Exit codes: 0 PASS | 1 FAIL.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def http_get_json(url: str, timeout: int = 30) -> tuple[int, Any, str]:
    """Return (status_code, parsed_json_or_None, error_message)."""
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "reposix-quality-gates/0.12.0"},
    )
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


def build_url(repo: str, tag: str) -> str:
    base = f"https://api.github.com/repos/{repo}/releases"
    if tag == "latest":
        return f"{base}/latest"
    return f"{base}/tags/{tag}"


def main() -> int:
    parser = argparse.ArgumentParser(description="Assert GH Release has >=N uploaded assets")
    parser.add_argument("--repo", required=True, help="<owner>/<name>")
    parser.add_argument("--tag", default="latest")
    parser.add_argument("--min-assets", type=int, default=7)
    args = parser.parse_args()

    url = build_url(args.repo, args.tag)
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []

    status, body, err = http_get_json(url)
    if status != 200 or body is None:
        asserts_failed.append(
            f"GET {url} returns HTTP 200 — got status={status}; err={err}"
        )
        write_artifact(
            "gh-assets-present.json",
            {
                "ts": now_iso(),
                "row_id": "release/gh-assets-present",
                "url": url,
                "asserts_passed": asserts_passed,
                "asserts_failed": asserts_failed,
                "actual_count": 0,
            },
        )
        return 1
    asserts_passed.append(f"GET {url} returns HTTP 200")

    assets = body.get("assets") or []
    actual = len(assets)
    if actual >= args.min_assets:
        asserts_passed.append(
            f"len(assets)={actual} >= min-assets={args.min_assets}"
        )
    else:
        asserts_failed.append(
            f"len(assets)={actual} < min-assets={args.min_assets}"
        )

    not_uploaded = [a.get("name", "?") for a in assets if a.get("state") != "uploaded"]
    if not not_uploaded:
        asserts_passed.append("every asset.state == 'uploaded'")
    else:
        asserts_failed.append(
            f"assets not in 'uploaded' state: {not_uploaded}"
        )

    write_artifact(
        "gh-assets-present.json",
        {
            "ts": now_iso(),
            "row_id": "release/gh-assets-present",
            "url": url,
            "tag": body.get("tag_name"),
            "actual_count": actual,
            "asserts_passed": asserts_passed,
            "asserts_failed": asserts_failed,
        },
    )
    return 0 if not asserts_failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
