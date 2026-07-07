#!/usr/bin/env python3
"""Release-dim verifier — cargo-binstall-resolves.

Backs catalog row release/cargo-binstall-resolves. Asserts that
`cargo binstall --no-confirm --dry-run <crate>` resolves to a prebuilt
GitHub Release binary. PASS asserts the INVARIANT (binstall resolved a
prebuilt binary and exited 0), not a single fragile vendor-output
substring — see REGRESSION NOTE at PASS_SIGNALS below.

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

# REGRESSION NOTE (2026-07-07, CI run 28839335746): cargo-binstall's stdout
# wording for "resolved a prebuilt release binary" has changed across
# versions -- v0.13.0's release gate went RED not because the installer was
# broken (binstall resolved the real v0.13.0 prebuilt in ~1.97s, rc=0) but
# because the old verifier grepped ONLY the literal URL substring
# "github.com/reubenjohn/reposix/releases/download/", and current
# cargo-binstall prints "has been downloaded from github.com" instead of
# echoing the full download URL. Asserting a single vendor surface string is
# brittle by construction -- the next binstall release can reword this
# again. PASS_SIGNALS below is therefore a set of ACCEPTED wordings
# (case-insensitive substring match), not a single string. If a FUTURE
# binstall version reword this a third time, the run will neither match
# PASS_SIGNALS nor a PARTIAL_SIGNAL and will correctly fall through to
# FAIL -- when that happens, treat it as "investigate wording drift first",
# add the new observed wording to PASS_SIGNALS, and only conclude the
# installer is actually broken if a real 404/no-resolve is present in the
# combined output.
PASS_SIGNALS = (
    "github.com/reubenjohn/reposix/releases/download/",  # binstall <=1.x URL echo
    "has been downloaded from github.com",  # binstall >=1.y summary wording
)
INSTALL_BINARIES_SIGNAL = "will install the following binaries"
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


def classify(rc: int | None, combined: str) -> dict:
    """Pure classification of a completed binstall run into PASS/PARTIAL/FAIL.

    Decoupled from subprocess execution so it can be unit-tested with
    canned stdout/stderr fixtures (no cargo/cargo-binstall invocation
    required) — see test_cargo_binstall_resolves.py.

    Returns a dict with keys: status_label, exit_code, asserts_passed,
    asserts_failed, and (when relevant) matched_signal / fallback_signal.
    """
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []

    if "no such command: `binstall`" in combined or "no such command: 'binstall'" in combined:
        asserts_failed.append("cargo-binstall is not installed in the verification environment")
        return {
            "status_label": "FAIL", "exit_code": 1,
            "diagnostic": "cargo-binstall not installed",
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        }

    combined_lower = combined.lower()
    pass_signal_hit = next(
        (s for s in PASS_SIGNALS if s.lower() in combined_lower), None
    )
    install_line_hit = INSTALL_BINARIES_SIGNAL.lower() in combined_lower

    if rc == 0 and pass_signal_hit and install_line_hit:
        asserts_passed.append(
            f"cargo binstall --dry-run resolved prebuilt binary "
            f"(matched signal {pass_signal_hit!r}, install-binaries line present)"
        )
        return {
            "status_label": "PASS", "exit_code": 0,
            "matched_signal": pass_signal_hit,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        }

    fallback_seen = next((s for s in PARTIAL_SIGNALS if s in combined), None)
    if fallback_seen:
        asserts_passed.append(
            f"cargo binstall ran (PARTIAL: source-build fallback observed: {fallback_seen!r}); "
            "documented expected state per P56 SURPRISES.md row 3 until MIGRATE-03 v0.12.1"
        )
        return {
            "status_label": "PARTIAL", "exit_code": 2,
            "fallback_signal": fallback_seen,
            "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
        }

    asserts_failed.append(
        f"cargo binstall returned {rc} without prebuilt-resolve and without source-fallback signal"
    )
    return {
        "status_label": "FAIL", "exit_code": 1,
        "asserts_passed": asserts_passed, "asserts_failed": asserts_failed,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="cargo-binstall-resolves verifier")
    parser.add_argument("--crate", default="reposix-cli")
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()

    row_id = "release/cargo-binstall-resolves"
    fname = "cargo-binstall-resolves.json"

    rc, stdout, stderr, err = run_binstall(args.crate, timeout_s=args.timeout)
    combined = (stdout or "") + (stderr or "")

    if err:
        write_artifact(fname, {
            "ts": now_iso(), "row_id": row_id, "crate": args.crate,
            "returncode": rc, "stdout_excerpt": combined[:2000],
            "asserts_passed": [], "asserts_failed": [err],
        })
        return 1

    result = classify(rc, combined)
    write_artifact(fname, {
        "ts": now_iso(), "row_id": row_id, "crate": args.crate,
        "returncode": rc, "stdout_excerpt": combined[:2000],
        **{k: v for k, v in result.items() if k != "exit_code"},
    })
    return result["exit_code"]


if __name__ == "__main__":
    raise SystemExit(main())
