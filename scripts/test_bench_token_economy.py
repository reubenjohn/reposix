"""Tests for scripts/bench_token_economy.py.

Run: python3 -m pytest scripts/test_bench_token_economy.py -x -q

All tests run offline — no anthropic package required.
The counter= keyword arg in get_or_count() is used for stub injection.
"""
from __future__ import annotations

import json
import pathlib
import sys
import unittest.mock

import pytest

# Allow importing the script as a module
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bench_token_economy as bench  # noqa: E402


# ---------------------------------------------------------------------------
# A1 Tests: SHA-256 cache + API-key guard + offline flag
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

    # First call: cache miss → stub invoked → cache written
    result1 = bench.get_or_count("abc", fixture, offline=False, counter=stub_counter)
    assert result1 == 7
    assert call_count == 1

    # Second call: cache hit → stub NOT invoked
    result2 = bench.get_or_count("abc", fixture, offline=False, counter=stub_counter)
    assert result2 == 7
    assert call_count == 1  # still 1 — cache was used

    # Verify cache content
    cache_path = bench._cache_path(fixture)
    assert cache_path.exists()
    cached = json.loads(cache_path.read_text())
    import hashlib
    assert cached["content_hash"] == hashlib.sha256("abc".encode("utf-8")).hexdigest()
    assert cached["input_tokens"] == 7


def test_cache_miss_on_hash_change_calls_counter(tmp_path: pathlib.Path) -> None:
    """When cache content_hash mismatches fixture, counter is re-invoked."""
    import hashlib

    fixture = tmp_path / "fixture.txt"
    fixture.write_text("abc", encoding="utf-8")

    # Write a stale cache claiming hash of "xyz"
    cache_path = bench._cache_path(fixture)
    stale_hash = hashlib.sha256("xyz".encode("utf-8")).hexdigest()
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
    assert call_count == 1  # counter invoked because hash mismatch

    # Cache should be refreshed with correct hash
    refreshed = json.loads(cache_path.read_text())
    assert refreshed["content_hash"] == hashlib.sha256("abc".encode("utf-8")).hexdigest()
    assert refreshed["input_tokens"] == 42


def test_missing_cache_without_api_key_exits_with_named_variable(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """When cache absent and ANTHROPIC_API_KEY unset, exit message names the variable."""
    fixture = tmp_path / "fixture.txt"
    fixture.write_text("some text", encoding="utf-8")

    # Ensure ANTHROPIC_API_KEY is not set
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)

    with pytest.raises(SystemExit) as exc_info:
        bench.require_api_key_or_cached([fixture])

    exit_msg = str(exc_info.value.code)
    # Must name the variable
    assert "ANTHROPIC_API_KEY" in exit_msg
    # Must NOT print a value (no '=' with a value next to the key name in an assignment sense)
    # Specifically: must not contain any real or stubbed key value
    # We just check the variable name appears, and no '=<value>' pattern follows it
    assert "=" not in exit_msg.replace("ANTHROPIC_API_KEY", "")


def test_offline_mode_refuses_api_call_on_cache_miss(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """--offline: SystemExit on cache miss without calling count_tokens at all."""
    fixture = tmp_path / "fixture.txt"
    fixture.write_text("no cache here", encoding="utf-8")

    # No cache file exists
    assert not bench._cache_path(fixture).exists()

    # Monkeypatch count_tokens to raise AssertionError if called
    def bad_counter(text: str, client) -> int:  # noqa: ANN001
        raise AssertionError("count_tokens must NOT be called in --offline mode")

    with pytest.raises(SystemExit):
        bench.get_or_count("no cache here", fixture, offline=True, counter=bad_counter)


# ---------------------------------------------------------------------------
# A2 Tests: end-to-end main() + stale-cache warning
# ---------------------------------------------------------------------------


def _write_token_cache(
    fixtures_dir: pathlib.Path,
    fixture_name: str,
    text: str,
    input_tokens: int,
) -> None:
    """Helper: write a valid .tokens.json sidecar in tmp_path."""
    import hashlib

    fixture_path = fixtures_dir / fixture_name
    cache = {
        "content_hash": hashlib.sha256(text.encode("utf-8")).hexdigest(),
        "input_tokens": input_tokens,
        "source": fixture_name,
        "model": bench.COUNT_MODEL,
        "counted_at": "2026-04-15T00:00:00Z",
    }
    bench._cache_path(fixture_path).write_text(json.dumps(cache))


def test_main_offline_with_mcp_and_reposix_cache_writes_results(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """main(["--offline"]) with valid caches writes RESULTS.md with correct content."""
    import json as _json

    fixtures_dir = tmp_path / "fixtures"
    fixtures_dir.mkdir()

    # Write minimal fixture files
    mcp_text = '{"tools": [{"name": "create_issue", "schema": {}}]}'
    reposix_text = "$ ls issues/\n0001.md  0002.md\n"

    mcp_fixture = fixtures_dir / "mcp_jira_catalog.json"
    mcp_fixture.write_bytes(mcp_text.encode("utf-8"))

    reposix_fixture = fixtures_dir / "reposix_session.txt"
    reposix_fixture.write_text(reposix_text, encoding="utf-8")

    # Write valid token caches
    _write_token_cache(fixtures_dir, "mcp_jira_catalog.json", mcp_text, 100)
    _write_token_cache(fixtures_dir, "reposix_session.txt", reposix_text, 10)

    # Redirect FIXTURES and BENCH_DIR so main() loads from tmp_path
    monkeypatch.setattr(bench, "FIXTURES", fixtures_dir)
    monkeypatch.setattr(bench, "BENCH_DIR", tmp_path)

    # Redirect RESULTS to a tmp file
    results_path = tmp_path / "RESULTS.md"
    monkeypatch.setattr(bench, "RESULTS", results_path)

    exit_code = bench.main(["--offline"])
    assert exit_code == 0

    content = results_path.read_text()
    # Column header must use real-tokenizer language, not chars/4
    assert "Anthropic" in content and "count_tokens" in content
    assert "chars / 4" not in content
    # Both scenarios present
    assert "MCP-mediated" in content
    assert "reposix" in content
    # Reduction percentage: 100 * (1 - 10/100) = 90.0%
    assert "90.0%" in content


def test_main_falls_back_gracefully_when_per_backend_fixtures_absent(
    tmp_path: pathlib.Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """main(["--offline"]) succeeds on base 2 fixtures even without github/confluence fixtures."""
    fixtures_dir = tmp_path / "fixtures"
    fixtures_dir.mkdir()

    mcp_text = '{"tools": []}'
    reposix_text = "$ ls\n"

    mcp_fixture = fixtures_dir / "mcp_jira_catalog.json"
    mcp_fixture.write_bytes(mcp_text.encode("utf-8"))

    reposix_fixture = fixtures_dir / "reposix_session.txt"
    reposix_fixture.write_text(reposix_text, encoding="utf-8")

    _write_token_cache(fixtures_dir, "mcp_jira_catalog.json", mcp_text, 5)
    _write_token_cache(fixtures_dir, "reposix_session.txt", reposix_text, 2)

    monkeypatch.setattr(bench, "FIXTURES", fixtures_dir)
    monkeypatch.setattr(bench, "BENCH_DIR", tmp_path)

    results_path = tmp_path / "RESULTS.md"
    monkeypatch.setattr(bench, "RESULTS", results_path)

    # github_issues.json and confluence_pages.json do NOT exist — must not crash
    assert not (fixtures_dir / "github_issues.json").exists()
    assert not (fixtures_dir / "confluence_pages.json").exists()

    exit_code = bench.main(["--offline"])
    assert exit_code == 0
    assert results_path.exists()
