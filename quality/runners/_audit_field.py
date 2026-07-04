"""RBF-FW-11: claim_vs_assertion_audit + kind:shell-subprocess validation.

Rows minted on/after 2026-05-08T00:00:00Z MUST carry a
`claim_vs_assertion_audit` paragraph (>=50 chars) explaining how
expected.asserts would falsify the row's claim if false. `kind:
shell-subprocess` rows MUST also carry an emitting-transcript contract --
the transcript IS the assertion shape. Sibling module per _freshness.py.

Callers MUST skip this for the `docs-alignment` dimension catalog: that
dimension's per-row schema (last_verdict/last_extracted) has no
last_verified key -- see quality/catalogs/README.md "docs-alignment
dimension" -- so the date-cutoff has no anchor field there.
"""
from __future__ import annotations

import hashlib

CUTOFF_ISO = "2026-05-08T00:00:00Z"  # Z suffix, not +00:00, for parser portability.
MIN_LEN = 50


def _has_transcript_contract(row: dict) -> bool:
    """expected.artifact.transcript_path, row-level transcript_path, or a
    "transcript" mention in expected.asserts (transitional fallback)."""
    expected = row.get("expected", {})
    artifact = expected.get("artifact") if isinstance(expected, dict) else None
    if isinstance(artifact, dict) and artifact.get("transcript_path"):
        return True
    if row.get("transcript_path"):
        return True
    asserts = expected.get("asserts", []) if isinstance(expected, dict) else []
    return any("transcript" in str(a).lower() for a in asserts)


def validate_row(row: dict, catalog_path: str, parse_rfc3339) -> None:
    """Raise SystemExit if row minted >= cutoff lacks the audit field, or
    kind:shell-subprocess lacks a transcript contract. Skip for
    docs-alignment catalogs (see module docstring)."""
    lv = row.get("last_verified")
    cutoff = parse_rfc3339(CUTOFF_ISO)
    is_new = lv is None or parse_rfc3339(lv) >= cutoff
    if is_new:
        audit = row.get("claim_vs_assertion_audit")
        if not isinstance(audit, str) or len(audit.strip()) < MIN_LEN:
            raise SystemExit(
                f"FAIL: {catalog_path}: row {row.get('id', '?')} missing "
                f"claim_vs_assertion_audit (>={MIN_LEN} chars required for rows "
                f"minted on/after {CUTOFF_ISO}); see "
                f"quality/catalogs/README.md schema table for the field's contract"
            )
    if row.get("kind") == "shell-subprocess" and not _has_transcript_contract(row):
        raise SystemExit(
            f"FAIL: {catalog_path}: row {row.get('id', '?')} declares "
            f"kind: shell-subprocess but has no transcript-emitting contract "
            f"(expected.artifact.transcript_path or schema-equivalent); see "
            f"quality/catalogs/README.md kind:shell-subprocess paragraph"
        )


def compute_hash(row: dict) -> str | None:
    """sha256 hex of claim_vs_assertion_audit text (stripped); None if absent."""
    audit = row.get("claim_vs_assertion_audit")
    if not isinstance(audit, str):
        return None
    return hashlib.sha256(audit.strip().encode("utf-8")).hexdigest()
