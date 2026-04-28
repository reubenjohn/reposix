"""Catalog I/O helper for the reposix-quality-review skill. P61 SUBJ-02.

Stdlib-only. Loads quality/catalogs/subjective-rubrics.json and exposes
helpers for finding rows + filtering by freshness. Re-uses the runner's
is_stale to avoid duplicating TTL math.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parents[4]
CATALOG_PATH = REPO_ROOT / "quality" / "catalogs" / "subjective-rubrics.json"

# Re-use is_stale + parse_rfc3339 from the runner (single source of truth for
# freshness math). The runners dir is a stdlib path; insert + import.
sys.path.insert(0, str(REPO_ROOT / "quality" / "runners"))
from run import is_stale as _runner_is_stale  # noqa: E402


def load_subjective_catalog() -> dict:
    """Load and return the subjective-rubrics catalog as a dict."""
    return json.loads(CATALOG_PATH.read_text(encoding="utf-8"))


def find_row(catalog: dict, rubric_id: str) -> dict:
    """Return the row matching rubric_id; KeyError if absent."""
    for r in catalog.get("rows", []):
        if r.get("id") == rubric_id:
            return r
    valid = [r["id"] for r in catalog.get("rows", [])]
    raise KeyError(f"rubric not found: {rubric_id!r}; valid: {valid}")


def stale_rows(catalog: dict, now: Optional[datetime] = None) -> list[dict]:
    """Rows that are is_stale OR have last_verified=None.

    The "never verified" case (last_verified=null) is treated as stale because
    the rubric has no recent verdict to trust.
    """
    now = now or datetime.now(timezone.utc)
    out = []
    for r in catalog.get("rows", []):
        if r.get("last_verified") is None or _runner_is_stale(r, now):
            out.append(r)
    return out


def all_rows(catalog: dict) -> list[dict]:
    """Return every row regardless of freshness (used by --force)."""
    return list(catalog.get("rows", []))


# Re-export so dispatchers can `from catalog import is_stale`
is_stale = _runner_is_stale
