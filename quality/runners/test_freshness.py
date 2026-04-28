"""Tests for the freshness-TTL extension (P61 SUBJ-03).

Stdlib pytest only. No extra deps. Matches existing repo test pattern
(see scripts/test_bench_token_economy.py for a similar pattern).

Run:
    pytest quality/runners/test_freshness.py -v
"""

from __future__ import annotations

import json
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

# Make the runner importable as a top-level module.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from run import (  # noqa: E402
    compute_exit_code,
    is_stale,
    parse_duration,
    run_row,
)


def test_parse_duration_30d() -> None:
    assert parse_duration("30d") == timedelta(days=30)


def test_parse_duration_14d() -> None:
    assert parse_duration("14d") == timedelta(days=14)


def test_parse_duration_90d() -> None:
    assert parse_duration("90d") == timedelta(days=90)


def test_parse_duration_invalid_raises() -> None:
    with pytest.raises(ValueError):
        parse_duration("not-a-duration")


def test_is_stale_expired_30d_at_4mo() -> None:
    """Row last verified 2026-01-01 with TTL=30d, evaluated at 2026-04-27, is STALE."""
    row = {"freshness_ttl": "30d", "last_verified": "2026-01-01T00:00:00Z"}
    now = datetime(2026, 4, 27, tzinfo=timezone.utc)
    assert is_stale(row, now) is True


def test_is_stale_within_180d_window() -> None:
    """Same row but TTL=180d, evaluated at 2026-04-27, is NOT STALE."""
    row = {"freshness_ttl": "180d", "last_verified": "2026-01-01T00:00:00Z"}
    now = datetime(2026, 4, 27, tzinfo=timezone.utc)
    assert is_stale(row, now) is False


def test_is_stale_no_last_verified() -> None:
    row = {"freshness_ttl": "30d", "last_verified": None}
    assert is_stale(row, datetime.now(timezone.utc)) is False


def test_is_stale_no_freshness_ttl() -> None:
    row = {"freshness_ttl": None, "last_verified": "2026-01-01T00:00:00Z"}
    assert is_stale(row, datetime.now(timezone.utc)) is False


def _stale_row(rid: str, blast: str) -> dict:
    return {
        "id": rid,
        "freshness_ttl": "30d",
        "last_verified": "2026-01-01T00:00:00Z",
        "verifier": {"script": "nonexistent.sh"},
        "artifact": f"quality/reports/verifications/test/{rid.replace('/', '-')}.json",
        "blast_radius": blast,
    }


def test_stale_waivered_row_stays_waivered() -> None:
    row = _stale_row("test/stale-and-waivered", "P1")
    row["waiver"] = {"until": "2026-12-31T00:00:00Z", "reason": "test"}
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "quality/reports/verifications/test").mkdir(parents=True)
        updated, _ = run_row(row, root, datetime(2026, 4, 27, tzinfo=timezone.utc))
        assert updated["status"] == "WAIVED"
        assert updated.get("_stale") is not True


def test_stale_p1_row_blocks_exit_code() -> None:
    row = _stale_row("test/stale-p1", "P1")
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "quality/reports/verifications/test").mkdir(parents=True)
        updated, _ = run_row(row, root, datetime(2026, 4, 27, tzinfo=timezone.utc))
        assert updated["status"] == "NOT-VERIFIED"
        assert updated.get("_stale") is True
        assert compute_exit_code([updated]) == 1
        art = json.loads((root / row["artifact"]).read_text())
        assert art.get("stale") is True


def test_stale_p2_row_does_not_block_exit_code() -> None:
    row = _stale_row("test/stale-p2", "P2")
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "quality/reports/verifications/test").mkdir(parents=True)
        updated, _ = run_row(row, root, datetime(2026, 4, 27, tzinfo=timezone.utc))
        assert updated["status"] == "NOT-VERIFIED"
        assert updated.get("_stale") is True
        assert compute_exit_code([updated]) == 0
