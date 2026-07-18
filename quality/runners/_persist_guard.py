"""Committed-GREEN downgrade guard for the Quality Gates runner. P123 SC2 (DRAIN-04).

Stdlib-only sibling of run.py (mirrors _freshness.py / _realbackend.py /
_audit_field.py / _shell_verdict.py / _env_load.py — keeps run.py under its
anti-bloat LOC cap). Closes the silent-catalog-corruption gap
(.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md 2026-07-14 20:44 HIGH):
a real rotation's `--persist` run downgraded `vision-litmus` PASS->FAIL on an
env-skip false negative, caught ONLY because the diff happened to be reviewed
before staging. This guard makes that class of silent regression structurally
impossible without an explicit `--allow-downgrade` opt-in.

## The exact rule (non-negotiable — SC2 regression-vs-TTL semantics)

The guard fires ONLY on an EXPLICIT regression: a committed `{PASS, WAIVED}`
baseline flipping to a freshly-graded `{FAIL, PARTIAL}` (a verifier actually ran
and produced a worse grade). That transition is REFUSED unless `--allow-downgrade`
is passed.

A transition to `NOT-VERIFIED` is NEVER a violation, REGARDLESS of cause
(freshness-TTL expiry, missing-verifier demotion, env-gated skip, exit-75
NOT-VERIFIED convention, or any other demotion channel). `NOT-VERIFIED` is this
project's designed-in "row went stale / couldn't be graded" channel, not a
regression — and rows minted through it (e.g. the freshness-invariants mints in
later waves of this very phase) must NEVER need `--allow-downgrade` to persist,
or the guard would deadlock the phase's own freshness mints. The status value
alone (`NOT-VERIFIED` vs `FAIL`/`PARTIAL`) is the ONLY signal this guard consults:
the transient `_stale` / `_verifier_missing` / `_skipped_real_backend` flags are
popped before persistence (see run.py) and are not reliably available here — and
they are not needed, because the line the guard draws is "did a verifier actually
run and produce a worse grade" (blocked) vs "did the row become ungraded for any
reason" (always allowed).

## The baseline is git HEAD, not the working tree

Correctness rests entirely on comparing against the LAST COMMITTED catalog state
(`git show HEAD:<path>`), not the on-disk working copy — a working-tree-dirty or
in-memory comparison is gameable (an attacker/buggy verifier that first writes a
worse status to disk would defeat a same-run comparison). A brand-new catalog
file (or brand-new row) absent from HEAD has no committed baseline and is exempt
by design — this guard scopes strictly to downgrades of PRE-EXISTING green rows,
never a blanket freeze on minting new rows.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

# A committed row in one of these statuses is "green" — a downgrade away from it
# is the regression this guard polices.
_GREEN = frozenset({"PASS", "WAIVED"})
# ...but ONLY an explicit worse grade counts. NOT-VERIFIED is deliberately absent
# (see module docstring: it is the designed-in ungraded channel, never blocked).
_REGRESSION = frozenset({"FAIL", "PARTIAL"})


def committed_head_statuses(repo_root: Path, cat_path: Path) -> dict[str, str] | None:
    """Return `{row_id: status}` from the LAST COMMITTED version of ``cat_path``.

    Reads `git show HEAD:<path-relative-to-repo_root>`. Returns ``None`` — a
    "no baseline to compare against" signal, NOT an error — when:
      * ``cat_path`` is not under ``repo_root`` (unexpected layout),
      * the path does not exist in HEAD (a brand-new catalog file),
      * git is unavailable / the subprocess fails (e.g. not a git repo), or
      * the committed blob is not parseable as the expected `{"rows": [...]}` JSON.

    ``None`` makes the caller's guard a no-op for that catalog (every row exempt),
    which is the correct fail-open posture for a brand-new file: there is no
    committed green status that could be silently regressed.
    """
    try:
        rel = Path(cat_path).resolve().relative_to(Path(repo_root).resolve())
    except (ValueError, OSError):
        return None
    try:
        result = subprocess.run(
            ["git", "show", f"HEAD:{rel.as_posix()}"],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
        )
    except (OSError, ValueError):
        return None
    if result.returncode != 0:
        return None
    try:
        data = json.loads(result.stdout)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(data, dict):
        return None
    return {
        row["id"]: row.get("status")
        for row in data.get("rows", [])
        if isinstance(row, dict) and "id" in row
    }


def refuse_downgrade_without_flag(
    committed: dict[str, str] | None,
    new_rows: list[dict],
) -> list[tuple[str, str, str]]:
    """Detect committed-GREEN -> explicit-regression downgrades.

    Returns a list of ``(row_id, old_status, new_status)`` for every row whose
    committed status is in ``{PASS, WAIVED}`` AND whose fresh grade is in
    ``{FAIL, PARTIAL}``. An empty list means "no downgrade detected — safe to
    persist". ``committed is None`` (no HEAD baseline) returns ``[]`` — nothing
    to compare against, so nothing is ever blocked on a brand-new catalog.

    A transition to ``NOT-VERIFIED`` is never reported (see module docstring):
    it is a legitimate ungraded/stale demotion, not a regression, and must stay
    unconditionally allowed so freshness-TTL mints never deadlock against the guard.
    """
    if committed is None:
        return []
    violations: list[tuple[str, str, str]] = []
    for row in new_rows:
        rid = row.get("id")
        if rid is None or rid not in committed:
            continue  # brand-new row (absent from HEAD) — exempt by design
        old = committed[rid]
        new = row.get("status")
        if old in _GREEN and new in _REGRESSION:
            violations.append((rid, old, new))
    return violations
