"""STUB (RED phase, P123 SC1 / DRAIN-03) — real implementation lands in the
GREEN commit. This placeholder exists only so quality/runners/test_run.py's
TestEnvSelfSourcing imports cleanly and fails on assertions (not ImportError),
giving a clean RED. Do NOT ship this stub.
"""
from __future__ import annotations

from pathlib import Path


def load_dotenv_if_present(repo_root: Path) -> None:  # noqa: ARG001
    """STUB: no-op until the GREEN commit implements ./.env self-sourcing."""
    return None
