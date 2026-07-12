"""RBF-FW-11 + P90 honesty rules: catalog-load + grade-time row validation.

Rows minted on/after 2026-05-08T00:00:00Z MUST carry a
`claim_vs_assertion_audit` paragraph (>=50 chars) explaining how
expected.asserts would falsify the row's claim if false. `kind:
shell-subprocess` rows MUST also carry an emitting-transcript contract --
the transcript IS the assertion shape. Sibling module per _freshness.py.

P90 additions (net-new decision logic lands here, NOT in run.py, which is
already over its 350-line anti-bloat cap -- see 90-RESEARCH-runner.md § 2):

- `minted_at` write-once anchor (D90-03 / cross-AI H2): when present it is
  the SOLE audit-cutoff anchor (a hand-edited last_verified can no longer
  move the cutoff). Rows minted on/after P90 (carrying minted_at) whose
  last_verified is >= P90_MINT_CUTOFF but lack minted_at are rejected.
- `coverage_kind` load-time validation (RBF-FW-06 / D90-05): transport/perf
  rows minted >= P90 MUST declare coverage_kind: real-backend (or a
  compliant waiver). Legacy transport rows are RAISE-only (the walker's
  concern), never blocked at load.
- `transcript_evidence_ok` (RBF-FW-08 / M6): the RUNTIME half of the
  shell-subprocess guarantee -- co-located with `_has_transcript_contract`
  (the LOAD-time half) so a future reader sees both halves of one contract.
- `asserts_congruent` (F-K4b / ROADMAP SC2): per-expected-assert congruence
  -- every expected.asserts entry must map to >=1 asserts_passed entry.

Callers MUST skip validate_row for the `docs-alignment` dimension catalog:
that dimension's per-row schema (last_verdict/last_extracted) has no
last_verified key -- see quality/catalogs/README.md "docs-alignment
dimension" -- so the date-cutoff has no anchor field there.
"""
from __future__ import annotations

import hashlib
import re
from pathlib import Path

CUTOFF_ISO = "2026-05-08T00:00:00Z"  # Z suffix, not +00:00, for parser portability.
# Rows minted on/after P90 MUST carry a write-once minted_at anchor. The
# constant is >= the 90-01 mint date (2026-07-04) AND strictly after the P90
# execution date, so no legacy row whose last_verified lands "today" during a
# run is retroactively forced to add minted_at (D90-03 / P90-D2).
P90_MINT_CUTOFF = "2026-07-05T00:00:00Z"
MIN_LEN = 50

# RBF-FW-06 transport/perf detection (P90-D1 tri-state). Regex over comment +
# id ONLY (never expected.asserts -- that would let a meta-row describing the
# check self-match, R-3). Dimension pre-filter cuts docs-row false positives.
_TRANSPORT_RE = re.compile(
    r"\b(push|fetch|round-?trip|clone|p50|p95|latency|throughput|"
    r"real[- ]backend|transport|list_records|list_changed_since)\b",
    re.IGNORECASE,
)
_TRANSPORT_DIMS = {"perf", "agent-ux", "security"}
_COVERAGE_KINDS = {"real-backend", "sim-only", "mechanical", "manual"}

# F-K4b congruence tokenizer: lowercase, split on non-alnum, drop stopwords,
# crude de-plural. Deliberately conservative -- see asserts_congruent.
_STOPWORDS = frozenset({
    "the", "a", "an", "and", "or", "of", "to", "in", "on", "at", "is", "are",
    "be", "as", "so", "if", "it", "its", "this", "that", "with", "for", "by",
    "no", "not", "any", "all", "per", "via", "vs", "e", "g", "ie", "eg",
})


def _has_transcript_contract(row: dict) -> bool:
    """LOAD-time half of the shell-subprocess guarantee: a transcript CONTRACT
    is declared (expected.artifact.transcript_path, row-level transcript_path,
    or a "transcript" mention in expected.asserts as a transitional fallback).
    The RUNTIME half -- the file must actually exist with an argv: line before
    a PASS flip -- lives in `transcript_evidence_ok` below (RBF-FW-08)."""
    expected = row.get("expected", {})
    artifact = expected.get("artifact") if isinstance(expected, dict) else None
    if isinstance(artifact, dict) and artifact.get("transcript_path"):
        return True
    if row.get("transcript_path"):
        return True
    asserts = expected.get("asserts", []) if isinstance(expected, dict) else []
    return any("transcript" in str(a).lower() for a in asserts)


def is_transport_or_perf_row(row: dict, dimension: str | None = None) -> bool:
    """RBF-FW-06 (P90-D1 tri-state): does this row make a transport/perf claim?

    - transport_claim is True  -> True  (explicit author opt-in, highest signal)
    - transport_claim is False -> False (explicit opt-out, regex suppressed)
    - transport_claim absent    -> regex over comment + id, gated to the
      perf/agent-ux/security dimensions (coarse pre-filter cuts doc-row noise).
    """
    tc = row.get("transport_claim")
    if tc is True:
        return True
    if tc is False:
        return False
    if dimension not in _TRANSPORT_DIMS:
        return False
    corpus = f"{row.get('comment', '')} {row.get('id', '')}"
    return bool(_TRANSPORT_RE.search(corpus))


def transcript_evidence_ok(
    row: dict, artifact: dict, repo_root
) -> tuple[bool, str]:
    """RBF-FW-08 RUNTIME half: a kind:shell-subprocess row may only PASS if a
    real transcript exists. Returns (ok, reason). Checks transcript_path
    (artifact-level, then the row's declared contract) -> file exists ->
    contains an `argv:` line naming the invoked binary. Co-located with
    `_has_transcript_contract` so the load-vs-runtime split is discoverable.
    """
    tp = artifact.get("transcript_path")
    if not tp:
        expected = row.get("expected", {}) if isinstance(row.get("expected"), dict) else {}
        exp_art = expected.get("artifact") if isinstance(expected.get("artifact"), dict) else {}
        tp = (exp_art or {}).get("transcript_path") or row.get("transcript_path")
    if not tp:
        return False, "no transcript_path in artifact"
    p = Path(repo_root) / tp
    if not p.exists():
        return False, f"transcript file missing at {tp}"
    text = p.read_text(errors="replace")
    if "argv:" not in text:
        return False, f"transcript at {tp} has no argv: line"
    return True, ""


def _sig_tokens(text: str) -> set[str]:
    """Significant tokens: lowercase, split on non-alnum, drop stopwords, crude
    de-plural (trailing-s on >3-char tokens). Shared by asserts_congruent."""
    out: set[str] = set()
    for tok in re.split(r"[^a-z0-9]+", text.lower()):
        if not tok or tok in _STOPWORDS:
            continue
        if len(tok) > 3 and tok.endswith("s"):
            tok = tok[:-1]
        out.add(tok)
    return out


def _pair_matches(expected_tok: set[str], passed_tok: set[str]) -> bool:
    """A single expected<->passed pair is congruent when the passed entry
    covers enough of the expected entry's significant tokens. Two significant
    tokens shared is the bar (>= per-pair concentration, NOT global-union
    overlap -- the distinction that catches the p86 F6 shape). Very short
    expected asserts (<=2 significant tokens) match on 1 shared token so a
    terse-but-real assertion is not spuriously flagged."""
    shared = len(expected_tok & passed_tok)
    if len(expected_tok) <= 2:
        return shared >= 1
    return shared >= 2


def asserts_congruent(
    expected_asserts: list, asserts_passed: list
) -> tuple[bool, list[str]]:
    """F-K4b / ROADMAP SC2 per-expected-assert congruence.

    EVERY `expected.asserts` entry must map to >=1 `asserts_passed` entry via
    normalized per-pair token matching (see `_pair_matches`). Any unmatched
    expected assert blocks the PASS flip; returns (ok, unmatched_expected).

    Per-pair (not zero-global-overlap): an expected assert whose tokens are
    scattered ONE-each across many passed entries -- present in the union but
    concentrated in none -- is UNMATCHED. That is exactly the p86 F6 shape
    (9 expected vs 17 passed, heavy shared vocabulary, 2 uncovered) that a
    global-overlap strawman would grade PASS dishonestly.

    No-op when EITHER list is empty: the 91 mechanical rows that write no
    asserts_passed are unaffected (backward-compat).
    """
    if not expected_asserts or not asserts_passed:
        return True, []
    passed_tok = [_sig_tokens(str(p)) for p in asserts_passed]
    unmatched: list[str] = []
    for exp in expected_asserts:
        et = _sig_tokens(str(exp))
        if not et:
            continue  # a punctuation-only expected assert cannot be matched on
        if not any(_pair_matches(et, pt) for pt in passed_tok):
            unmatched.append(str(exp))
    return (len(unmatched) == 0), unmatched


def apply_pass_gates(row: dict, artifact: dict, repo_root) -> None:
    """Grade-time PASS gates (RBF-FW-08 + F-K4b). Mutates row['status'] ->
    FAIL and appends to artifact['asserts_failed'] when a would-be PASS fails
    a honesty gate. Kept out of run.py (already over its anti-bloat cap) per
    90-RESEARCH-runner.md § 2. No-op unless row['status'] == 'PASS'.

    - RBF-FW-08 (M6): a kind:shell-subprocess row must carry real transcript
      evidence (file exists + argv: line) before it may PASS -- the load-time
      contract check (`_has_transcript_contract`) only proves the contract was
      declared, not that a transcript was emitted.
    - F-K4b / ROADMAP SC2: a would-be PASS is blocked unless EVERY
      expected.asserts entry maps to >=1 asserts_passed entry (per-pair
      congruence). Gated on minted_at so only NEW-regime rows are held to the
      bar -- legacy prose asserts, which do not token-map their terse
      asserts_passed, are exempt (mirrors D90-05; see 90-02 zero-false-RED
      evidence).
    """
    if row.get("status") != "PASS":
        return
    if row.get("kind") == "shell-subprocess":
        ok, why = transcript_evidence_ok(row, artifact, repo_root)
        if not ok:
            row["status"] = "FAIL"
            artifact.setdefault("asserts_failed", []).append(
                f"shell-subprocess PASS blocked: {why}"
            )
        # F-K4b congruence does NOT apply to a shell-subprocess row: its honesty
        # axis is the freshly regenerated transcript (checked immediately above),
        # NOT a per-assert asserts_passed<->expected mapping. The shared helper
        # quality/gates/agent-ux/lib/transcript.sh writes asserts_passed=[] by
        # design, so before this explicit return a transcript-PASS row fell
        # through to the F-K4b block below and survived ONLY on the empty-list
        # no-op in asserts_congruent(). That made a correctly-minted P0 row
        # (e.g. the agent-ux/fleet-safety-* guards) one helper change away from a
        # FALSE-DEMOTE: the instant asserts_passed became non-empty and did not
        # token-map every expected assert, F-K4b would flip a transcript-proven
        # PASS to FAIL. Returning here makes the exemption explicit and provably
        # independent of asserts_passed contents. (No coverage is lost: F-K4b
        # was already a guaranteed no-op for these rows.)
        return
    if row.get("minted_at"):
        ok, unmatched = asserts_congruent(
            (row.get("expected") or {}).get("asserts") or [],
            artifact.get("asserts_passed") or [],
        )
        if not ok:
            row["status"] = "FAIL"
            artifact.setdefault("asserts_failed", []).append(
                "F-K4b: expected assert(s) not covered by any asserts_passed "
                "entry: " + " | ".join(unmatched)
            )


def validate_row(
    row: dict, catalog_path: str, parse_rfc3339, dimension: str | None = None
) -> None:
    """Raise SystemExit on a load-time honesty violation. Skip for
    docs-alignment catalogs (see module docstring). Covers: minted_at
    write-once anchor (D90-03), claim_vs_assertion_audit requirement,
    coverage_kind on transport rows (RBF-FW-06/D90-05), kind:shell-subprocess
    transcript contract, and the OD-2 waiver prohibition."""
    lv = row.get("last_verified")
    minted = row.get("minted_at")
    cutoff = parse_rfc3339(CUTOFF_ISO)
    # D90-03 (cross-AI H2): minted_at, when present, is the SOLE immutable
    # audit-cutoff anchor -- a hand-edited last_verified no longer moves the
    # cutoff. Absent -> the legacy null-or-last_verified heuristic.
    if minted is not None:
        is_new = parse_rfc3339(minted) >= cutoff
    else:
        is_new = lv is None or parse_rfc3339(lv) >= cutoff
        # Post-P90 rows MUST carry minted_at (closes the backdate dodge for new
        # rows). Null-lv legacy rows are NOT forced -- the 5 null-last_verified
        # legacy rows keep loading (89-07 already backfilled their audit field).
        if lv is not None and parse_rfc3339(lv) >= parse_rfc3339(P90_MINT_CUTOFF):
            raise SystemExit(
                f"FAIL: {catalog_path}: row {row.get('id', '?')} has last_verified "
                f">= {P90_MINT_CUTOFF} but lacks a write-once minted_at anchor -- "
                f"rows minted on/after P90 MUST carry minted_at (D90-03); see "
                f"quality/catalogs/README.md schema table for the field's contract"
            )
    if is_new:
        audit = row.get("claim_vs_assertion_audit")
        if not isinstance(audit, str) or len(audit.strip()) < MIN_LEN:
            raise SystemExit(
                f"FAIL: {catalog_path}: row {row.get('id', '?')} missing "
                f"claim_vs_assertion_audit (>={MIN_LEN} chars required for rows "
                f"minted on/after {CUTOFF_ISO}); see "
                f"quality/catalogs/README.md schema table for the field's contract"
            )
    # RBF-FW-06 / D90-05: transport/perf rows minted >= P90 (i.e. carrying
    # minted_at) MUST declare coverage_kind: real-backend, or a compliant
    # waiver -- NO PASS-with-comment. Legacy transport rows (no minted_at) are
    # RAISE-only (the walker's concern), NEVER blocked at load -- hard-blocking
    # them would turn pre-push RED on ~every catalog before the P95 migration
    # phase exists (the deferral-loop the framework fixes prevent, not create).
    if minted is not None and is_transport_or_perf_row(row, dimension):
        ck = row.get("coverage_kind")
        waiver = row.get("waiver")
        compliant_waiver = bool(waiver) and bool(waiver.get("until"))
        if ck != "real-backend" and not compliant_waiver:
            raise SystemExit(
                f"FAIL: {catalog_path}: row {row.get('id', '?')} is a transport/perf "
                f"claim minted >= P90 but lacks coverage_kind: real-backend (and has "
                f"no compliant waiver) -- F-K4a/D90-05 forbids PASS-with-comment on "
                f"transport claims; add coverage_kind: real-backend or set "
                f"transport_claim: false if this is not a transport claim (see "
                f"quality/catalogs/README.md coverage_kind row)"
            )
    coverage_kind = row.get("coverage_kind")
    if coverage_kind is not None and coverage_kind not in _COVERAGE_KINDS:
        raise SystemExit(
            f"FAIL: {catalog_path}: row {row.get('id', '?')} has invalid "
            f"coverage_kind={coverage_kind!r} -- must be one of "
            f"{sorted(_COVERAGE_KINDS)} (see quality/catalogs/README.md)"
        )
    if row.get("kind") == "shell-subprocess" and not _has_transcript_contract(row):
        raise SystemExit(
            f"FAIL: {catalog_path}: row {row.get('id', '?')} declares "
            f"kind: shell-subprocess but has no transcript-emitting contract "
            f"(expected.artifact.transcript_path or schema-equivalent); see "
            f"quality/catalogs/README.md kind:shell-subprocess paragraph"
        )
    # OD-2 mechanical enforcement (P89 cross-AI review, finding H3): waivers
    # are FORBIDDEN on real-backend-cadence rows -- "no waiver, no until_date,
    # no PASS-with-comment" is a load-time contract, not prose.
    if "pre-release-real-backend" in row.get("cadences", []) and row.get("waiver"):
        raise SystemExit(
            f"FAIL: {catalog_path}: row {row.get('id', '?')} carries a waiver "
            f"but is tagged cadence pre-release-real-backend -- OD-2 forbids "
            f"waivers on real-backend gates; remove the waiver block (see "
            f"quality/PROTOCOL.md OD-2 hard-RED skip-semantics)"
        )


def compute_hash(row: dict) -> str | None:
    """sha256 hex of claim_vs_assertion_audit text (stripped); None if absent."""
    audit = row.get("claim_vs_assertion_audit")
    if not isinstance(audit, str):
        return None
    return hashlib.sha256(audit.strip().encode("utf-8")).hexdigest()
