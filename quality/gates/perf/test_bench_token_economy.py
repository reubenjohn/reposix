"""quality/gates/perf/test_bench_token_economy.py -- perf dimension bench tests.

Run: python3 -m pytest quality/gates/perf/test_bench_token_economy.py -x -q

All tests run offline -- no anthropic package, no ANTHROPIC_API_KEY, no network.

Two layers are covered:
  * A1 -- the retained count_tokens cache/enrichment IO surface (get_or_count,
    _cache_path, require_api_key_or_cached). Demoted from the headline but kept
    importable; the `counter=` kwarg injects a stub.
  * B  -- the live JSONL session-usage headline path (P115 amendment #10):
    median math + capture loading + captures-driven main() regen.
"""
from __future__ import annotations

import json
import pathlib
import sys

import pytest

# Allow importing the script as a module
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bench_token_economy as bench  # noqa: E402


# ---------------------------------------------------------------------------
# A1: retained count_tokens cache surface (offline enrichment path)
# ---------------------------------------------------------------------------


def test_cache_roundtrip_hits_on_identical_content(tmp_path: pathlib.Path) -> None:
    """Cache write on first call; cache read (no counter invoke) on second call."""
    fixture = tmp_path / "fixture.txt"
    fixture.write_text("abc", encoding="utf-8")

    call_count = 0

    def stub_counter(text: str, client) -> int:  # noqa: ANN001
        nonlocal call_count
        call_count += 1
        return 7

    result1 = bench.get_or_count("abc", fixture, offline=False, counter=stub_counter)
    assert result1 == 7
    assert call_count == 1

    result2 = bench.get_or_count("abc", fixture, offline=False, counter=stub_counter)
    assert result2 == 7
    assert call_count == 1  # cache hit -- stub NOT re-invoked

    cache_path = bench._cache_path(fixture)
    assert cache_path.exists()
    cached = json.loads(cache_path.read_text())
    import hashlib
    assert cached["content_hash"] == hashlib.sha256(b"abc").hexdigest()
    assert cached["input_tokens"] == 7


def test_cache_miss_on_hash_change_calls_counter(tmp_path: pathlib.Path) -> None:
    """When cache content_hash mismatches fixture, counter is re-invoked."""
    import hashlib

    fixture = tmp_path / "fixture.txt"
    fixture.write_text("abc", encoding="utf-8")

    cache_path = bench._cache_path(fixture)
    stale_hash = hashlib.sha256(b"xyz").hexdigest()
    cache_path.write_text(json.dumps({
        "content_hash": stale_hash,
        "input_tokens": 999,
        "source": "fixture.txt",
        "model": bench.COUNT_MODEL,
        "counted_at": "2000-01-01T00:00:00Z",
    }))

    call_count = 0

    def stub_counter(text: str, client) -> int:  # noqa: ANN001
        nonlocal call_count
        call_count += 1
        return 42

    result = bench.get_or_count("abc", fixture, offline=False, counter=stub_counter)
    assert result == 42
    assert call_count == 1

    refreshed = json.loads(cache_path.read_text())
    assert refreshed["content_hash"] == hashlib.sha256(b"abc").hexdigest()
    assert refreshed["input_tokens"] == 42


def test_missing_cache_without_api_key_exits_with_named_variable(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """When cache absent and ANTHROPIC_API_KEY unset, exit message names the variable."""
    fixture = tmp_path / "fixture.txt"
    fixture.write_text("some text", encoding="utf-8")
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)

    with pytest.raises(SystemExit) as exc_info:
        bench.require_api_key_or_cached([fixture])

    exit_msg = str(exc_info.value.code)
    assert "ANTHROPIC_API_KEY" in exit_msg
    assert "=" not in exit_msg.replace("ANTHROPIC_API_KEY", "")


def test_offline_mode_refuses_api_call_on_cache_miss(tmp_path: pathlib.Path) -> None:
    """--offline: SystemExit on cache miss without calling count_tokens at all."""
    fixture = tmp_path / "fixture.txt"
    fixture.write_text("no cache here", encoding="utf-8")
    assert not bench._cache_path(fixture).exists()

    def bad_counter(text: str, client) -> int:  # noqa: ANN001
        raise AssertionError("count_tokens must NOT be called in --offline mode")

    with pytest.raises(SystemExit):
        bench.get_or_count("no cache here", fixture, offline=True, counter=bad_counter)


# ---------------------------------------------------------------------------
# B: JSONL session-usage headline path (P115 amendment #10)
# ---------------------------------------------------------------------------


def _write_capture(
    captures_dir: pathlib.Path,
    name: str,
    arm: str,
    output: int,
    cache_create: int,
    input_context: int,
    cost: float,
    *,
    run_label: bool = True,
) -> None:
    """Write one scrubbed session-usage capture JSON under *captures_dir*."""
    rec = {
        "arm": arm,
        "backend": "github",
        "output_tokens": output,
        "cache_creation_input_tokens": cache_create,
        "total_input_context_tokens": input_context,
        "total_cost_usd": cost,
    }
    if run_label:
        rec["run_label"] = name
    (captures_dir / f"{name}.json").write_text(json.dumps(rec))


def _seed_six_plus_smoke(captures_dir: pathlib.Path) -> None:
    """Seed 3 reposix + 3 mcp github captures (all medians -> 90% reduction) plus
    one run_label-less smoke row that MUST be excluded."""
    for i, (out, cc, ic, cost) in enumerate(
        [(100, 10, 1000, 0.1), (200, 20, 2000, 0.2), (300, 30, 3000, 0.3)], start=1
    ):
        _write_capture(captures_dir, f"reposix-github-run{i}", "reposix-mediated", out, cc, ic, cost)
    for i, (out, cc, ic, cost) in enumerate(
        [(1000, 100, 10000, 1.0), (2000, 200, 20000, 2.0), (4000, 400, 40000, 4.0)], start=1
    ):
        _write_capture(captures_dir, f"mcp-github-run{i}", "mcp-mediated", out, cc, ic, cost)
    # Smoke probe: no run_label -> excluded from medians.
    _write_capture(captures_dir, "mcp-kan-smoke", "mcp-mediated", 999, 999, 999, 9.9, run_label=False)


def test_median_odd_and_even() -> None:
    assert bench.median([300, 100, 200]) == 200
    assert bench.median([1, 2, 3, 4]) == 2.5
    with pytest.raises(ValueError):
        bench.median([])


def test_load_headline_captures_excludes_smoke(tmp_path: pathlib.Path) -> None:
    captures = tmp_path / "captures"
    captures.mkdir()
    _seed_six_plus_smoke(captures)

    records = bench.load_headline_captures(captures)
    assert len(records) == 6  # smoke (no run_label) excluded
    assert all("smoke" not in r["run_label"] for r in records)


def test_load_headline_captures_empty_dir_exits(tmp_path: pathlib.Path) -> None:
    captures = tmp_path / "captures"
    captures.mkdir()
    with pytest.raises(SystemExit):
        bench.load_headline_captures(captures)


def test_compute_arm_medians_and_reductions(tmp_path: pathlib.Path) -> None:
    captures = tmp_path / "captures"
    captures.mkdir()
    _seed_six_plus_smoke(captures)

    records = bench.load_headline_captures(captures)
    arm_medians = bench.compute_arm_medians(records)
    assert arm_medians["reposix-mediated"]["n"] == 3
    assert arm_medians["mcp-mediated"]["n"] == 3
    # per-axis median-of-3
    assert arm_medians["reposix-mediated"]["output"] == 200
    assert arm_medians["mcp-mediated"]["output"] == 2000

    reductions = bench.compute_reductions(arm_medians)
    # 100 * (1 - 200/2000) = 90.0 on every axis by construction
    for axis in ("output", "cache_create", "input_context", "cost"):
        assert round(reductions[axis], 1) == 90.0
    assert round(reductions["cost_multiple"], 1) == 10.0


def test_main_offline_regenerates_doc_from_captures(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """main(["--offline"]) reads captures, writes a deterministic doc with the
    four-axis headline; a second run is a byte-identical no-op."""
    captures = tmp_path / "captures"
    captures.mkdir()
    _seed_six_plus_smoke(captures)

    results_path = tmp_path / "token-economy.md"
    monkeypatch.setattr(bench, "CAPTURES", captures)
    monkeypatch.setattr(bench, "BENCH_DIR", tmp_path)
    monkeypatch.setattr(bench, "RESULTS", results_path)

    assert bench.main(["--offline"]) == 0
    content = results_path.read_text()

    # Live methodology, not the retired synthetic count_tokens-on-fixture one.
    assert "MCP" in content
    assert "90.0%" in content            # computed reduction, all axes
    assert "reposix" in content
    assert "Methodology" in content
    assert "scripts/demo.sh" not in content
    assert "modeled on" not in content

    # Deterministic: a second run rewrites the SAME bytes.
    first = content
    assert bench.main(["--offline"]) == 0
    assert results_path.read_text() == first
