"""Shared helpers for the freshness-invariants verifier modules.

Stdlib only. Extracted from freshness-invariants.py during the
file-size-limits split so every per-invariant module imports the same
artifact-shape + catalog-load semantics.
"""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# Repo root is four levels up: quality/gates/structure/freshness/_shared.py
REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent.parent
CATALOG_PATH = REPO_ROOT / "quality" / "catalogs" / "freshness-invariants.json"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_artifact(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def load_row(row_id: str) -> dict:
    data = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
    for r in data["rows"]:
        if r["id"] == row_id:
            return r
    raise SystemExit(f"FAIL: row {row_id!r} not found in {CATALOG_PATH}")


def bash_check(cmd: str, repo_root: Path) -> tuple[int, str, str]:
    """Run `bash -c cmd` from repo_root; return (exit_code, stdout, stderr)."""
    result = subprocess.run(
        ["bash", "-c", cmd],
        capture_output=True,
        text=True,
        cwd=str(repo_root),
        check=False,
    )
    return result.returncode, result.stdout, result.stderr


def make_artifact(
    row: dict,
    exit_code: int,
    asserts_passed: list[str],
    asserts_failed: list[str],
    **extra: Any,
) -> dict:
    art: dict[str, Any] = {
        "ts": now_iso(),
        "row_id": row["id"],
        "exit_code": exit_code,
        "stdout": "",
        "stderr": "",
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
    }
    art.update(extra)
    return art
