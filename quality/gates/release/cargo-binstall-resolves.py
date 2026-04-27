#!/usr/bin/env python3
"""Release-dim verifier — cargo-binstall-resolves.

Backs catalog row release/cargo-binstall-resolves. Asserts that
`cargo binstall --no-confirm --dry-run <crate>` resolves to a prebuilt
GitHub Release binary (greppable URL match against
github.com/reubenjohn/reposix/releases/download/...).

PARTIAL acceptable per P56 SURPRISES.md row 3: binstall metadata
mismatch is documented expected state until v0.12.1 MIGRATE-03 lands.

Stdlib only (subprocess, argparse, json). Cross-platform.

Exit codes: 0 PASS | 2 PARTIAL | 1 FAIL.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"

PASS_SIGNAL = "github.com/reubenjohn/reposix/releases/download/"
PARTIAL_SIGNALS = (
    "Falling back to source",
    "Falling back to install via 'cargo install'",
    "will be installed from source",
    "running `cargo install",
    "running '/home/runner/.rustup",
    "compiling reposix",
)


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_artifact(filename: str, payload: dict) -> None:
    out = ARTIFACT_DIR / filename
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def run_binstall(crate: str, timeout_s: int = 600) -> tuple[int | None, str, str, str | None]:
    """Run cargo binstall --no-confirm --dry-run <crate>.

    Returns (returncode, stdout, stderr, error_message).
    error_message is set if binstall is not installed.
    """
    cmd = ["cargo", "binstall", "--no-confirm", "--dry-run", crate]
    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True,
            timeout=timeout_s, check=False, shell=False,
        )
        return result.returncode, result.stdout, result.stderr, None
    except FileNotFoundError as e:
        return None, "", "", f"cargo / cargo-binstall not installed: {e}"
    except subprocess.TimeoutExpired as e:
        out = e.stdout.decode("utf-8", "replace") if isinstance(e.stdout, (bytes, bytearray)) else (e.stdout or "")
        err = e.stderr.decode("utf-8", "replace") if isinstance(e.stderr, (bytes, bytearray)) else (e.stderr or "")
        return None, out, err, f"timeout after {timeout_s}s"


def main() -> int:
    parser = argparse.ArgumentParser(description="cargo-binstall-resolves verifier")
    parser.add_argument("--crate", default="reposix-cli")
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    row_id = "release/cargo-binstall-resolves"
    fname = "cargo-binstall-resolves.json"

    rc, stdout, stderr, err = run_binstall(args.crate, timeout_s=args.timeout)
    combined = (stdout or "") + (stderr or "")

    if err:
        asserts_failed.append(err)
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "returncode": rc, "stdout_excerpt": combined[:2000],
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 1

    if "no such command: `binstall`" in combined or "no such command: 'binstall'" in combined:
        asserts_failed.append("cargo-binstall is not installed in the verification environment")
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "returncode": rc, "status_label": "FAIL",
            "diagnostic": "cargo-binstall not installed",
            "stdout_excerpt": combined[:2000],
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 1

    if PASS_SIGNAL in combined and rc == 0:
        asserts_passed.append(
            f"cargo binstall --dry-run {args.crate} resolved prebuilt binary "
            f"({PASS_SIGNAL}...)"
        )
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "returncode": rc, "status_label": "PASS",
            "stdout_excerpt": combined[:2000],
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 0

    fallback_seen = next((s for s in PARTIAL_SIGNALS if s in combined), None)
    if fallback_seen:
        asserts_passed.append(
            f"cargo binstall ran (PARTIAL: source-build fallback observed: {fallback_seen!r}); "
            "documented expected state per P56 SURPRISES.md row 3 until MIGRATE-03 v0.12.1"
        )
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "returncode": rc, "status_label": "PARTIAL",
            "fallback_signal": fallback_seen,
            "stdout_excerpt": combined[:2000],
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        })
        return 2

    asserts_failed.append(
        f"cargo binstall returned {rc} without prebuilt-resolve and without source-fallback signal"
    )
    write_artifact(fname, {
        "ts": now_iso(), "row_id": row_id, "crate": args.crate,
        "returncode": rc, "status_label": "FAIL",
        "stdout_excerpt": combined[:2000],
        "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
    })
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
