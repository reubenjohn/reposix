#!/usr/bin/env python3
"""Release-dim verifier — crates-io-max-version.

Backs catalog rows release/crates-io-max-version/<crate> for each
published reposix crate (9 crates total). Asserts crates.io
max_version matches the workspace.package.version in Cargo.toml.

Stdlib only. Reads Cargo.toml via tomllib (Python 3.11+) when
available; falls back to a small regex-based extractor for the
single key we need on Python 3.8/3.9/3.10. Sleeps 1s after each
invocation so the runner's sequential 9-crate sweep stays at
~1 req/sec under crates.io's unauthenticated rate limit.

Exit codes: 0 PASS | 1 FAIL.
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
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"
UA = "reposix-quality-gates/0.12.0 (https://github.com/reubenjohn/reposix)"
WORKSPACE_VERSION_RE = re.compile(
    r'\[workspace\.package\][^\[]*?\nversion\s*=\s*"([^"]+)"',
    re.DOTALL,
)


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_artifact(filename: str, payload: dict) -> None:
    out = ARTIFACT_DIR / filename
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def http_get_json(url: str, timeout: int = 30) -> tuple[int, Any, str]:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, json.loads(resp.read().decode("utf-8")), ""
    except urllib.error.HTTPError as e:
        return e.code, None, f"HTTPError: {e}"
    except (urllib.error.URLError, json.JSONDecodeError, OSError) as e:
        return 0, None, f"{type(e).__name__}: {e}"


def workspace_version() -> tuple[str | None, str]:
    """Return (version, err). Prefer tomllib (Python 3.11+); fall back to regex."""
    cargo = REPO_ROOT / "Cargo.toml"
    if not cargo.exists():
        return None, f"Cargo.toml missing at {cargo}"
    try:
        import tomllib  # type: ignore[import-not-found]
    except ImportError:
        text = cargo.read_text(encoding="utf-8")
        m = WORKSPACE_VERSION_RE.search(text)
        if not m:
            return None, "regex did not find [workspace.package].version"
        return m.group(1), ""
    try:
        with cargo.open("rb") as f:
            data = tomllib.load(f)
        return data["workspace"]["package"]["version"], ""
    except (KeyError, OSError) as e:
        return None, f"tomllib failed: {e}"


def main() -> int:
    parser = argparse.ArgumentParser(description="crates-io-max-version verifier")
    parser.add_argument("--crate", required=True)
    args = parser.parse_args()

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    row_id = f"release/crates-io-max-version/{args.crate}"
    fname = f"crates-io-max-version-{args.crate}.json"

    ws_ver, ws_err = workspace_version()
    if not ws_ver:
        asserts_failed.append(f"could not read workspace.package.version: {ws_err}")
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "workspace_version": None, "max_version": None,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        time.sleep(1)
        return 1

    asserts_passed.append(f"workspace.package.version = {ws_ver}")
    url = f"https://crates.io/api/v1/crates/{args.crate}"
    status, body, err = http_get_json(url)

    max_version = None
    if status != 200 or body is None:
        asserts_failed.append(f"GET {url} HTTP 200 — got status={status}; err={err}")
    else:
        asserts_passed.append(f"GET {url} returns HTTP 200")
        try:
            max_version = body["crate"]["max_version"]
            asserts_passed.append(f"crates.io max_version = {max_version}")
        except (KeyError, TypeError) as e:
            asserts_failed.append(f"response missing crate.max_version: {e}")

    if max_version and ws_ver:
        if max_version == ws_ver:
            asserts_passed.append(
                f"crates.io max_version == workspace version ({ws_ver})"
            )
        else:
            asserts_failed.append(
                f"crates.io max_version {max_version!r} != workspace version {ws_ver!r}"
            )

    write_artifact(fname, {
        "ts": now_iso(), "row_id": row_id, "crate": args.crate,
        "workspace_version": ws_ver, "max_version": max_version, "url": url,
        "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
    })
    time.sleep(1)
    return 0 if not asserts_failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
