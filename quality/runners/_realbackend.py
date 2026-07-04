"""Env-gating helper for cadence: pre-release-real-backend. P89 RBF-FW-01.

Rows tagged with this cadence MUST have REPOSIX_ALLOWED_ORIGINS set to a
non-127.0.0.1 origin AND at least one sanctioned-target credential set
complete (Confluence / GitHub / JIRA — docs/reference/testing-targets.md).
Env not configured -> default-skip: status NOT-VERIFIED, never an HTTP
call against a missing allowlist. Per OD-2 (89-OWNER-DECISIONS.md) the
skip is NEVER a pass state: at milestone-close this cadence must EXECUTE;
creds/targets missing at milestone-close is hard RED at the verdict layer
— no waiver, no until_date, no PASS-with-comment, no skip-counts-as-pass.

Sibling module per the _freshness.py precedent — keeps run.py under its
anti-bloat cap. Stdlib only (run.py:1-30 header contract).
"""

from __future__ import annotations

import re
from typing import Mapping

_CRED_SETS = {
    "confluence": ("ATLASSIAN_API_KEY", "ATLASSIAN_EMAIL", "REPOSIX_CONFLUENCE_TENANT"),
    "github": ("GITHUB_TOKEN",),
    "jira": ("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE"),
}
_NON_LOCAL_RE = re.compile(r"^https?://(?!127\.0\.0\.1)")

CADENCE_NAME = "pre-release-real-backend"

# sysexits.h EX_TEMPFAIL (75) repurposed: "verifier completed but the
# conditions for a real grade are not met — preserve NOT-VERIFIED, do NOT
# overwrite to FAIL". 89-06's milestone-close SLOT verifier exits 75 when
# env IS set but the P91-P95 substrate has not landed. Per OD-2, exit-75
# must NEVER be reachable for creds-missing-at-milestone-close — that state
# is hard RED (quality/PROTOCOL.md § "Verifier exit-code conventions").
EXIT_NOT_VERIFIED = 75


def is_skipped(row: dict, env: Mapping[str, str]) -> bool:
    """Return True iff row is tagged pre-release-real-backend AND env not configured."""
    if CADENCE_NAME not in row.get("cadences", []):
        return False
    origins = env.get("REPOSIX_ALLOWED_ORIGINS", "")
    if not origins or not _NON_LOCAL_RE.search(origins):
        return True
    return not any(all(env.get(k) for k in keys) for keys in _CRED_SETS.values())


def skip_reason(env: Mapping[str, str]) -> str:
    """Human-readable reason string for the skip artifact."""
    origins = env.get("REPOSIX_ALLOWED_ORIGINS", "")
    if not origins:
        return "REPOSIX_ALLOWED_ORIGINS unset"
    if not _NON_LOCAL_RE.search(origins):
        return f"REPOSIX_ALLOWED_ORIGINS={origins} matches 127.0.0.1 (local-only)"
    return (
        "no credential set complete (need Confluence: ATLASSIAN_API_KEY"
        "+ATLASSIAN_EMAIL+REPOSIX_CONFLUENCE_TENANT, OR GitHub: GITHUB_TOKEN, "
        "OR JIRA: JIRA_EMAIL+JIRA_API_TOKEN+REPOSIX_JIRA_INSTANCE)"
    )


def map_exit_code_to_status(exit_code: int) -> str:
    """Map a verifier exit code to a runner row status.

    Exit 0  -> PASS
    Exit 2  -> PARTIAL      (pre-existing runner convention, preserved)
    Exit 75 -> NOT-VERIFIED (sysexits.h EX_TEMPFAIL repurposed)
    Other   -> FAIL

    Single source of truth for run.py's non-timeout exit-code branch.
    Narrow by design: runner-wide status preservation is P90 RBF-FW-07.
    """
    if exit_code == 0:
        return "PASS"
    if exit_code == 2:
        return "PARTIAL"
    if exit_code == EXIT_NOT_VERIFIED:
        return "NOT-VERIFIED"
    return "FAIL"
