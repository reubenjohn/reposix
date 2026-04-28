"""Freshness-TTL helpers for the Quality Gates runner. P61 SUBJ-03.

Stdlib-only sibling of run.py. Extracted per the P61 Wave B pivot rule
(run.py exceeded the 390-LOC anti-bloat cap once the STALE branch landed;
keeping run.py under 350 / 390 keeps the runner contract surface tight
while letting the freshness logic grow independently if v0.12.1 adds
more units, e.g. weeks/months).
"""

from __future__ import annotations

import re
from datetime import datetime, timedelta

_DURATION_RE = re.compile(r"^(\d+)([dhm])$")


def parse_duration(s: str) -> timedelta:
    """Parse a short duration string ('30d', '14d', '90d', '5h', '15m').

    Raises ValueError on malformed input.
    """
    m = _DURATION_RE.match(s.strip())
    if not m:
        raise ValueError(f"invalid duration: {s!r}")
    n = int(m.group(1))
    unit = m.group(2)
    if unit == "d":
        return timedelta(days=n)
    if unit == "h":
        return timedelta(hours=n)
    return timedelta(minutes=n)


def is_stale(row: dict, now: datetime, parse_rfc3339) -> bool:
    """True iff row.last_verified + parse_duration(row.freshness_ttl) < now.

    Returns False if either freshness_ttl or last_verified is missing/None
    (the row was never verified or has no TTL contract).

    parse_rfc3339 is injected to avoid an import cycle with run.py.
    """
    ttl = row.get("freshness_ttl")
    lv = row.get("last_verified")
    if not ttl or not lv:
        return False
    try:
        return parse_rfc3339(lv) + parse_duration(ttl) < now
    except (ValueError, TypeError):
        return False
