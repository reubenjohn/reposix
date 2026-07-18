#!/usr/bin/env python3
"""Release-dim verifier — installer-asset-bytes (dual-mode).

Backs catalog rows: install/curl-installer-sh, install/powershell-installer-ps1,
install/build-from-source. Stdlib only. Cross-platform.

Mode A: --url <u> --min-bytes <n> [--magic <prefix>]
  HEAD url; HTTP 200 + size >= min-bytes; if --magic, first 32 bytes prefix-match.

Mode B: --ci-check <wf> [--branch <br>]
  gh run list --workflow=<wf> --branch=<br> --status=success --limit=1; assert >=1 run.

Exit codes: 0 PASS | 1 FAIL.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"
UA = "reposix-quality-gates/0.12.0"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_artifact(filename: str, payload: dict) -> None:
    out = ARTIFACT_DIR / filename
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def head_or_get_size(url: str, timeout: int = 30) -> tuple[int, int, str]:
    """HEAD; if no content-length fall back to ranged GET. Returns (status, size, err)."""
    req = urllib.request.Request(url, method="HEAD", headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            cl = resp.headers.get("Content-Length")
            if cl:
                return resp.status, int(cl), ""
    except urllib.error.HTTPError as e:
        return e.code, -1, f"HEAD HTTPError: {e}"
    except (urllib.error.URLError, OSError):
        pass
    return ranged_get_size(url, timeout=timeout)


def ranged_get_size(url: str, timeout: int = 30) -> tuple[int, int, str]:
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Range": "bytes=0-65535"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = resp.read()
            cr = resp.headers.get("Content-Range") or ""
            if "/" in cr:
                total = cr.split("/", 1)[1]
                if total.isdigit():
                    return resp.status, int(total), ""
            return resp.status, len(data), ""
    except urllib.error.HTTPError as e:
        return e.code, -1, f"GET HTTPError: {e}"
    except (urllib.error.URLError, OSError) as e:
        return 0, -1, f"{type(e).__name__}: {e}"


def fetch_magic(url: str, n: int = 32, timeout: int = 30) -> tuple[bytes, str]:
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Range": f"bytes=0-{n - 1}"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.read()[:n], ""
    except (urllib.error.URLError, urllib.error.HTTPError, OSError) as e:
        return b"", f"{type(e).__name__}: {e}"


def url_to_row_artifact(url: str) -> tuple[str, str]:
    if "reposix-installer.sh" in url:
        return "install/curl-installer-sh", "install-curl-installer-sh.json"
    if "reposix-installer.ps1" in url:
        return "install/powershell-installer-ps1", "install-powershell-installer-ps1.json"
    safe = url.rsplit("/", 1)[-1] or "asset"
    return f"install/{safe}", f"install-{safe}.json"


def mode_a(args: argparse.Namespace) -> int:
    row_id, fname = url_to_row_artifact(args.url)
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    status, size, err = head_or_get_size(args.url)
    if status != 200 or size < 0:
        asserts_failed.append(f"GET {args.url} HTTP 200 — got status={status}; err={err}")
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "url": args.url, "size": size,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 1
    asserts_passed.append(f"GET {args.url} returns HTTP 200")
    if size >= args.min_bytes:
        asserts_passed.append(f"size={size} >= min-bytes={args.min_bytes}")
    else:
        asserts_failed.append(f"size={size} < min-bytes={args.min_bytes}")
    if args.magic:
        data, merr = fetch_magic(args.url)
        if data.startswith(args.magic.encode("utf-8")):
            asserts_passed.append(f"body starts with magic {args.magic!r}")
        else:
            asserts_failed.append(f"body does NOT start with magic {args.magic!r}; got {data[:16]!r}; err={merr}")
    write_artifact(fname, {
        "ts": now_iso(), "row_id": row_id, "url": args.url, "size": size,
        "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
    })
    return 0 if not asserts_failed else 1


def mode_b(args: argparse.Namespace) -> int:
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    row_id = "install/build-from-source"
    fname = "install-build-from-source.json"
    # NOTE: gh CLI quirk — combining `--branch <br>` with `--json ...`
    # silently returns [] (empty array) even when matching runs exist.
    # Workaround: query without --branch, then post-filter on `headBranch`.
    cmd = [
        "gh", "run", "list", "--workflow", args.ci_check,
        "--limit", "20",
        "--json", "databaseId,conclusion,headBranch,status",
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30,
                                check=False, shell=False)
    except FileNotFoundError as e:
        asserts_failed.append(f"gh CLI not installed: {e}")
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 1
    if result.returncode != 0:
        asserts_failed.append(f"gh run list exit {result.returncode}: {result.stderr.strip()[:300]}")
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "stdout": result.stdout, "stderr": result.stderr,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 1
    asserts_passed.append(f"gh run list --workflow {args.ci_check} exit 0")
    try:
        runs = json.loads(result.stdout or "[]")
    except json.JSONDecodeError as e:
        asserts_failed.append(f"gh JSON parse failed: {e}")
        runs = []
    matching = [
        r for r in runs
        if r.get("headBranch") == args.branch and r.get("conclusion") == "success"
    ]
    if matching:
        asserts_passed.append(
            f"latest successful run found on {args.branch} (databaseId={matching[0].get('databaseId')})"
        )
    else:
        asserts_failed.append(
            f"no successful run found for {args.ci_check}@{args.branch} in last 20 runs"
        )
    write_artifact(fname, {
        "ts": now_iso(), "row_id": row_id, "workflow": args.ci_check, "branch": args.branch,
        "runs_total": len(runs), "matching_count": len(matching),
        "matching_runs": matching[:3],
        "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
    })
    return 0 if not asserts_failed else 1


def main() -> int:
    parser = argparse.ArgumentParser(description="installer-asset-bytes — dual-mode release verifier")
    parser.add_argument("--url")
    parser.add_argument("--min-bytes", type=int, default=1024)
    parser.add_argument("--magic")
    parser.add_argument("--ci-check")
    parser.add_argument("--branch", default="main")
    args = parser.parse_args()
    if bool(args.url) == bool(args.ci_check):
        parser.error("specify exactly one of --url (Mode A) or --ci-check (Mode B)")
    return mode_a(args) if args.url else mode_b(args)


if __name__ == "__main__":
    raise SystemExit(main())
