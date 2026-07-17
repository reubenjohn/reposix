"""Tests for snippet-extract.py (DOCS-REPRO-01).

Stdlib-only via subprocess. Mirrors quality/gates/code/check_fixtures
test patterns from P58 SIMPLIFY-05.
"""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
SCRIPT = REPO_ROOT / "quality" / "gates" / "docs-repro" / "snippet-extract.py"


def _load_module():
    """Import snippet-extract.py by file path (hyphenated filename can't be
    `import`ed normally). Used by the pivot-semantics tests below so we can
    monkeypatch all_blocks/load_catalog/load_allowlist and call cmd_check()
    directly instead of shelling out."""
    spec = importlib.util.spec_from_file_location("snippet_extract_under_test", SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _fake_block(i: int) -> dict:
    return {
        "file": "docs/fake.md",
        "start_line": i,
        "end_line": i,
        "lang": "bash",
        "content": f"echo {i}",
        "sha256": f"sha{i}",
        "derived_id": f"snippet/docs/fake.md:{i}-{i}",
    }


def run(args: list[str]) -> tuple[int, str, str]:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        cwd=str(REPO_ROOT),
        check=False,
    )
    return result.returncode, result.stdout, result.stderr


def test_list_emits_valid_json_with_blocks():
    code, stdout, _ = run(["--list"])
    assert code == 0
    payload = json.loads(stdout)
    assert "blocks" in payload
    assert "scope" in payload
    assert "total" in payload
    assert isinstance(payload["blocks"], list)
    assert payload["total"] == len(payload["blocks"])
    # The current docs corpus has 30+ fenced blocks; tolerate growth.
    assert payload["total"] >= 10, f"unexpectedly few blocks: {payload['total']}"


def test_list_block_schema():
    code, stdout, _ = run(["--list"])
    assert code == 0
    payload = json.loads(stdout)
    for block in payload["blocks"]:
        for key in ("file", "start_line", "end_line", "lang", "content", "sha256", "derived_id"):
            assert key in block, f"missing {key} in {block}"
        assert isinstance(block["start_line"], int)
        assert isinstance(block["end_line"], int)
        assert block["end_line"] >= block["start_line"]
        assert block["derived_id"].startswith("snippet/")


def test_check_default_mode_writes_artifact():
    code, _stdout, _stderr = run([])  # default mode is --check
    artifact = (
        REPO_ROOT
        / "quality"
        / "reports"
        / "verifications"
        / "docs-repro"
        / "snippet-coverage.json"
    )
    assert artifact.exists(), "default mode (--check) must write the artifact"
    payload = json.loads(artifact.read_text())
    assert payload["row_id"] == "docs-repro/snippet-coverage"
    assert "asserts_passed" in payload
    assert "asserts_failed" in payload
    assert payload["exit_code"] == code


def test_check_mode_explicit():
    code, _stdout, _stderr = run(["--check"])
    assert code in (0, 1), f"check mode exits 0 (PASS) or 1 (FAIL); got {code}"


def test_write_template_unknown_id_returns_2():
    code, _stdout, stderr = run(["--write-template", "snippet/does/not/exist:99-100"])
    assert code == 2
    assert "unknown" in stderr.lower()


def test_write_template_valid_id_emits_stub():
    # Find a real block via --list, then ask for its template.
    code, stdout, _ = run(["--list"])
    assert code == 0
    payload = json.loads(stdout)
    assert payload["blocks"], "need at least one block to test --write-template"
    derived_id = payload["blocks"][0]["derived_id"]
    code, stdout, _ = run(["--write-template", derived_id])
    assert code == 0
    stub = json.loads(stdout)
    for key in (
        "id", "dimension", "cadence", "kind", "sources", "command", "expected",
        "expected_content_sha256", "verifier", "artifact", "status", "blast_radius",
    ):
        assert key in stub, f"missing {key} in stub"
    assert stub["dimension"] == "docs-repro"
    assert stub["status"] == "NOT-VERIFIED"


def test_line_count_under_cap():
    cap = 250
    lines = SCRIPT.read_text().count("\n")
    assert lines <= cap, f"snippet-extract.py is {lines} lines; cap is {cap}"


def test_pivot_counts_uncovered_not_raw_blocks(tmp_path, monkeypatch):
    """Pins GTH-V15-49 Option B: the pivot advisory fires on UNCOVERED block count,
    not raw block count. 60 raw blocks with only 2 uncovered must NOT trip the pivot
    (2 <= PIVOT_THRESHOLD=50) -- this assertion FAILS against the old
    `len(blocks) > PIVOT_THRESHOLD` logic (60 > 50 would fire) and PASSES against the
    new `len(uncovered) > PIVOT_THRESHOLD` logic.
    """
    mod = _load_module()
    monkeypatch.setattr(mod, "ARTIFACT_PATH", tmp_path / "snippet-coverage.json")

    total_blocks = 60
    uncovered_count = 2
    fake_blocks = [_fake_block(i) for i in range(total_blocks)]
    covered_rows = [{"sources": [f"docs/fake.md:{i}-{i}"]} for i in range(uncovered_count, total_blocks)]

    monkeypatch.setattr(mod, "all_blocks", lambda: fake_blocks)
    monkeypatch.setattr(mod, "load_catalog", lambda: {"rows": covered_rows})
    monkeypatch.setattr(mod, "load_allowlist", lambda: set())

    mod.cmd_check()
    artifact = json.loads(mod.ARTIFACT_PATH.read_text())
    assert artifact["block_count"] == total_blocks
    assert artifact["uncovered_count"] == uncovered_count
    pivot_msgs = [f for f in artifact["asserts_failed"] if "exceed threshold" in f]
    assert not pivot_msgs, (
        f"pivot advisory fired on {total_blocks} raw blocks even though only "
        f"{uncovered_count} are uncovered (threshold {mod.PIVOT_THRESHOLD}): {pivot_msgs}"
    )


def test_pivot_fires_when_uncovered_exceeds_threshold(tmp_path, monkeypatch):
    """Companion to test_pivot_counts_uncovered_not_raw_blocks: when the UNCOVERED
    count itself exceeds PIVOT_THRESHOLD, the advisory must still fire."""
    mod = _load_module()
    monkeypatch.setattr(mod, "ARTIFACT_PATH", tmp_path / "snippet-coverage.json")

    total_blocks = mod.PIVOT_THRESHOLD + 5
    fake_blocks = [_fake_block(i) for i in range(total_blocks)]

    monkeypatch.setattr(mod, "all_blocks", lambda: fake_blocks)
    monkeypatch.setattr(mod, "load_catalog", lambda: {"rows": []})
    monkeypatch.setattr(mod, "load_allowlist", lambda: set())

    mod.cmd_check()
    artifact = json.loads(mod.ARTIFACT_PATH.read_text())
    assert artifact["uncovered_count"] == total_blocks
    pivot_msgs = [f for f in artifact["asserts_failed"] if "exceed threshold" in f]
    assert pivot_msgs, "pivot advisory should fire when uncovered count exceeds threshold"
