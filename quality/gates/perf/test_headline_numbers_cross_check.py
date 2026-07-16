"""quality/gates/perf/test_headline_numbers_cross_check.py -- perf-dimension gate tests.

Run: python3 -m pytest quality/gates/perf/test_headline_numbers_cross_check.py -x -q

All tests run offline -- committed markdown only, no network, no API key.

Two layers:
  * UNIT -- canonical parsing + cross-check logic against throwaway tmp docs
    (both the GREEN all-match path and each RED drift class, asserting the
    teaching output names the surface, the canonical value, and a fix).
  * INTEGRATION -- `test_hero_surfaces_match_canonical` runs the real check
    against the committed repo files. This is the fn the doc-alignment rows
    `docs/index/latency-8ms-read` and `latency-cached-read-8ms` bind to.
"""
from __future__ import annotations

import importlib.util
import pathlib

import pytest

# The gate script has hyphens in its name -- load it by path.
_GATE_PATH = pathlib.Path(__file__).resolve().parent / "headline-numbers-cross-check.py"
_spec = importlib.util.spec_from_file_location("headline_cross_check", _GATE_PATH)
gate = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gate)  # type: ignore[union-attr]


# ---------------------------------------------------------------------------
# Canonical-doc fixtures (minimal but structurally faithful).
# ---------------------------------------------------------------------------

LATENCY_MD = """\
# latency

| Step | sim | github |
|------|-----|--------|
| `reposix init` cold | 278 ms | 830 ms |
| List records [^N] | 7 ms (N=6) | 779 ms |
| Get one record | 6 ms | 320 ms |
| Helper `capabilities` probe | 5 ms | 5 ms |
"""

TOKEN_MD = """\
# token economy

| Axis | reposix | MCP | reposix advantage |
|------|--------:|----:|:------------------|
| Output tokens (agent generates) | 1,213 | 21,171 | **~94.3% fewer** |
| Cost per session (USD) | $0.2076 | $0.8278 | ~74.9% cheaper |
"""

INDEX_MD = """\
-   **`~94% fewer output tokens`** · **`~75% cheaper`** than GitHub-MCP.
-   **`6 ms`** cached read · **`278 ms`** cold init — simulator, CI-canonical.

    Note over Agent,reposix: git-native loop · ~1.2k output tokens (live)
    Note over Agent,MCP: MCP tool loop · ~21k output tokens (live)

reposix's `6 ms` cache read is measured against the in-process simulator, but ...

Sim cold init is `278 ms` (soft threshold `500 ms`); list-issues `7 ms`; capabilities probe `5 ms`.
"""

README_MD = """\
*`6 ms` / `278 ms` are simulator-measured (CI-canonical sim figures); ...*

- **`~94% fewer output tokens` · `~75% cheaper per session`** — for the identical task.
- **`6 ms`** — read one issue from the local cache after first fetch.
- **`278 ms`** — `reposix init` cold bootstrap against the simulator (CI-canonical).
"""

CONCEPTS_MD = """\
| **Latency, cached read** | `6 ms` ([sim](../benchmarks/latency.md)) | `200 ms` | `100 ms` |
| **Latency, cold init / first call** | `278 ms` cold init ([sim](../benchmarks/latency.md)) | one tool-list | per-call HTTPS |

    identical task the git-native (`reposix`) arm is **~94.3% fewer output
    tokens** and **~74.9% cheaper per session** than the GitHub-MCP arm.
"""


def _write_repo(
    tmp_path: pathlib.Path,
    *,
    index: str = INDEX_MD,
    readme: str = README_MD,
    concepts: str = CONCEPTS_MD,
    latency: str = LATENCY_MD,
    token: str = TOKEN_MD,
) -> pathlib.Path:
    (tmp_path / "docs" / "benchmarks").mkdir(parents=True)
    (tmp_path / "docs" / "concepts").mkdir(parents=True)
    (tmp_path / "docs" / "benchmarks" / "latency.md").write_text(latency)
    (tmp_path / "docs" / "benchmarks" / "token-economy.md").write_text(token)
    (tmp_path / "docs" / "index.md").write_text(index)
    (tmp_path / "README.md").write_text(readme)
    (tmp_path / "docs" / "concepts" / "reposix-vs-mcp-and-sdks.md").write_text(concepts)
    return tmp_path


# ---------------------------------------------------------------------------
# Canonical parsing
# ---------------------------------------------------------------------------


def test_parse_latency_canonical() -> None:
    assert gate.parse_latency_canonical(LATENCY_MD) == {"get": 6, "list": 7, "init": 278}


def test_parse_latency_canonical_missing_raises() -> None:
    with pytest.raises(SystemExit) as exc:
        gate.parse_latency_canonical("| Step | sim |\n| foo | bar |\n")
    assert "latency" in str(exc.value.code).lower()


def test_parse_latency_canonical_missing_init_raises() -> None:
    """A latency table with get+list but no 'reposix init cold' row is a hard failure."""
    no_init = "| Step | sim |\n| List records | 7 ms |\n| Get one record | 6 ms |\n"
    with pytest.raises(SystemExit) as exc:
        gate.parse_latency_canonical(no_init)
    assert "init" in str(exc.value.code).lower()


def test_parse_token_canonical() -> None:
    assert gate.parse_token_canonical(TOKEN_MD) == {
        "output": 94.3,
        "cost": 74.9,
        "output_reposix": 1213,
        "output_mcp": 21171,
    }


def test_parse_token_canonical_missing_raises() -> None:
    with pytest.raises(SystemExit) as exc:
        gate.parse_token_canonical("| Axis | x |\n| foo | 3% |\n")
    assert "token" in str(exc.value.code).lower()


# ---------------------------------------------------------------------------
# Cross-check logic
# ---------------------------------------------------------------------------


def test_run_cross_check_green_on_matching_surfaces(tmp_path: pathlib.Path) -> None:
    repo = _write_repo(tmp_path)
    failures, latency, token = gate.run_cross_check(repo)
    assert failures == []
    assert latency == {"get": 6, "list": 7, "init": 278}
    assert token == {
        "output": 94.3,
        "cost": 74.9,
        "output_reposix": 1213,
        "output_mcp": 21171,
    }


def test_run_cross_check_flags_stale_latency(tmp_path: pathlib.Path) -> None:
    """A lingering 8 ms hero figure against a 6 ms canonical is caught + taught."""
    stale = INDEX_MD.replace("**`6 ms`** cached read", "**`8 ms`** cached read")
    repo = _write_repo(tmp_path, index=stale)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("8 ms" in f and "6 ms" in f for f in failures)
    # Teaching: names the surface line and a copy-paste fix.
    assert any("docs/index.md:" in f and "Fix:" in f for f in failures)


def test_run_cross_check_flags_stale_cold_init(tmp_path: pathlib.Path) -> None:
    """A lingering 27 ms cold-init hero figure against a 278 ms canonical is caught."""
    stale = INDEX_MD.replace("**`278 ms`** cold init", "**`27 ms`** cold init")
    repo = _write_repo(tmp_path, index=stale)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("27 ms" in f and "278 ms" in f for f in failures)
    assert any("docs/index.md:" in f and "Fix:" in f for f in failures)


def test_run_cross_check_flags_stale_loop_figure(tmp_path: pathlib.Path) -> None:
    """A homepage loop figure that drifts from the canonical median is caught + taught."""
    stale = INDEX_MD.replace(
        "git-native loop · ~1.2k output tokens (live)",
        "git-native loop · ~5k output tokens (live)",
    )
    repo = _write_repo(tmp_path, index=stale)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("~1.2k output tokens (live)" in f and "1,213" in f for f in failures)


def test_run_cross_check_flags_stale_list(tmp_path: pathlib.Path) -> None:
    stale = INDEX_MD.replace("list-issues `7 ms`", "list-issues `9 ms`")
    repo = _write_repo(tmp_path, index=stale)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("9 ms" in f and "7 ms" in f and "list" in f.lower() for f in failures)


def test_run_cross_check_flags_missing_claim(tmp_path: pathlib.Path) -> None:
    """If the prose is restructured so a claim regex no longer matches, that is a failure."""
    reworded = INDEX_MD.replace("**`6 ms`** cached read", "a very fast cached read")
    repo = _write_repo(tmp_path, index=reworded)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("could NOT locate" in f and "cached read" in f for f in failures)


def test_run_cross_check_flags_stale_token(tmp_path: pathlib.Path) -> None:
    stale = README_MD.replace("~94% fewer output tokens", "~89% fewer output tokens")
    repo = _write_repo(tmp_path, readme=stale)
    failures, _, _ = gate.run_cross_check(repo)
    assert any("README.md" in f and "94% fewer output tokens" in f for f in failures)


def test_main_returns_zero_on_green(tmp_path, monkeypatch) -> None:
    repo = _write_repo(tmp_path)
    monkeypatch.setattr(gate, "REPO_ROOT", repo)
    assert gate.main([]) == 0


def test_main_returns_one_on_drift(tmp_path, monkeypatch) -> None:
    stale = INDEX_MD.replace("**`6 ms`** cached read", "**`8 ms`** cached read")
    repo = _write_repo(tmp_path, index=stale)
    monkeypatch.setattr(gate, "REPO_ROOT", repo)
    assert gate.main([]) == 1


# ---------------------------------------------------------------------------
# INTEGRATION -- the real committed hero surfaces (doc-alignment bind target).
# ---------------------------------------------------------------------------


def test_hero_surfaces_match_canonical() -> None:
    """Every hero headline on the committed docs matches its canonical source.

    Bound by doc-alignment rows `docs/index/latency-8ms-read` and
    `latency-cached-read-8ms`: the '6 ms cached read' hero claim is true iff
    this integration check is GREEN against the real files.
    """
    failures, _, _ = gate.run_cross_check(gate.REPO_ROOT)
    assert failures == [], "hero headline drift:\n" + "\n".join(failures)
