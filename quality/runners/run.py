#!/usr/bin/env python3
"""Quality Gates runner — single entry point for cadence-tagged gates.

Contract: .planning/research/v0.12.0/naming-and-architecture.md § runner-contract.
Cadence semantics + cadence-specific freshness rules: quality/PROTOCOL.md.
Stdlib only; cross-platform (linux + macOS). Anti-bloat: per-dimension
verifiers live under quality/gates/<dim>/.

Each catalog row carries `cadences: list[str]` — one gate may fire at
multiple cadences. The 8 cadences are pre-commit, pre-push, pre-pr, weekly,
pre-release, post-release, on-demand, pre-release-real-backend (env-gated).

Usage:
  python3 quality/runners/run.py --cadence <cadence>            # GATE  (validate-only)
  python3 quality/runners/run.py --cadence <cadence> --persist  # MINT  (writes catalog)

GRADE / PERSIST split (D-P96-01, .planning/CONSULT-DECISIONS.md): a bare
cadence run is VALIDATE-ONLY -- it grades every in-scope row in memory, writes
per-row artifacts under quality/reports/verifications/, and STILL blocks RED
via the exit code, but it does NOT write graded status back to
quality/catalogs/. Only --persist (the phase-close / verifier-subagent MINT
invocation) mutates the committed catalog. This stops a read-only pre-push GATE
run from self-mutating the catalog as a side effect (the HIGH self-mutation
bug: a pre-push flipped docs-build.json and dirtied the tree at push time).
Hooks (pre-commit/pre-push) and CI (pre-pr/weekly/pre-release/post-release) run
WITHOUT --persist by design; catalog-first minting / un-waiving pass --persist.

Exit codes:
  0 — every P0+P1 row in scope is PASS or WAIVED.
  1 — any P0+P1 row in scope is FAIL/PARTIAL/NOT-VERIFIED.
"""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# P61 SUBJ-03: freshness helpers extracted to a sibling module per the
# Wave B pivot rule (anti-bloat cap on run.py).
from _freshness import parse_duration as _parse_duration_impl
from _freshness import is_stale as _is_stale_impl
import _realbackend  # P89 RBF-FW-01: env-gate + exit-code map (sibling per anti-bloat rule)
import _audit_field  # P89 RBF-FW-11: claim_vs_assertion_audit + kind:shell-subprocess cross-check
import _shell_verdict  # D-P96-01 (extended): deterministic committed verdict for kind:shell-subprocess
import _env_load  # P123 SC1/DRAIN-03: conditional ./.env self-sourcing (present-only, non-clobbering)
import _persist_guard  # P123 SC2/DRAIN-04: committed-GREEN downgrade guard (git-HEAD baseline)

# Re-export so existing callers and tests can keep doing `from run import parse_duration`.
parse_duration = _parse_duration_impl

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"
REPORTS_DIR = REPO_ROOT / "quality" / "reports"
BLAST_RADIUS_ORDER = {"P0": 0, "P1": 1, "P2": 2}

VALID_CADENCES = (
    "pre-commit", "pre-push", "pre-pr", "weekly", "pre-release", "post-release",
    "on-demand", "pre-release-real-backend", "post-push",
)
# `post-push` (D-CONV-4, 2026-07-12): runs at phase/milestone-close AFTER the
# phase push has LANDED on main -- orchestrator/verifier-side, NOT the pre-push
# git hook. It confirms the LATEST ci.yml run on main concluded success (the
# `code/ci-green-on-main` row). This is deliberately a DISTINCT cadence from
# pre-push: a pre-push-tagged CI-green probe is CIRCULAR (CI has not yet run the
# commit being pushed), which is exactly why D-CONV-1 demoted ci-job-status out
# of the pre-* path. Running AFTER the push closes the systemic hole where a
# phase shipped GREEN while its push turned main RED and nobody checked.


def now_iso() -> str:
    """Return current UTC time as RFC3339 with Z suffix."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_rfc3339(s: str) -> datetime:
    """Parse RFC3339 with or without Z; return tz-aware datetime."""
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    return datetime.fromisoformat(s)


def discover_catalogs() -> list[Path]:
    """Glob catalog files. Skip allow-list sidecars + README.md.

    D-CONV-3 (2026-07-04): orphan-scripts.json used to be skipped here because
    it carried the retired scalar `cadence` key -- this runner only recognizes
    the `cadences` list shape. It has since been converted to the list schema
    (scripts/migrations/2026-05-cadence-to-list.py's shape) and every row now
    carries a `claim_vs_assertion_audit`, so it participates like any other
    catalog: it is discovered, graded by run.py, and rolled into verdict.py's
    badge like every other dimension.
    """
    out: list[Path] = []
    for p in sorted(CATALOG_DIR.glob("*.json")):
        if p.stem.endswith("-allowlist"):
            continue
        out.append(p)
    return out


def load_catalog(path: Path) -> dict:
    """Load one catalog file; verify wrapper has dimension + rows keys."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        raise SystemExit(f"FAIL: {path}: invalid JSON: {e}") from e
    for k in ("dimension", "rows"):
        if k not in data:
            raise SystemExit(f"FAIL: {path}: wrapper missing {k!r}")
    # P89 RBF-FW-11: fail loud BEFORE any verifier runs. Skipped for the
    # docs-alignment dimension, whose distinct per-row schema (last_verdict/
    # last_extracted) has no last_verified key for the date-cutoff to anchor
    # on (quality/catalogs/README.md "docs-alignment dimension").
    if data["dimension"] != "docs-alignment":
        for row in data.get("rows", []):
            _audit_field.validate_row(row, str(path), parse_rfc3339, data["dimension"])
    return data


def save_catalog(path: Path, data: dict) -> None:
    """Write catalog back; preserve wrapper field order ($schema, comment, dimension, rows).

    Uses ensure_ascii=False so em-dashes and other Unicode survive round-trips.
    Caller decides whether to invoke save_catalog() — see catalog_dirty().
    """
    ordered: dict[str, Any] = {}
    for key in ("$schema", "comment", "dimension", "rows"):
        if key in data:
            ordered[key] = data[key]
    for key, val in data.items():
        if key not in ordered:
            ordered[key] = val
    path.write_text(
        json.dumps(ordered, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def catalog_dirty(original: dict, updated: dict) -> bool:
    """True iff any row's status changed. Timestamp-only updates do not count
    as a semantic change — they belong in the artifact, not the committed
    catalog. This keeps `git status` clean across pre-push runs that just
    re-confirm an already-known status."""
    orig_rows = {r["id"]: r.get("status") for r in original.get("rows", [])}
    new_rows = {r["id"]: r.get("status") for r in updated.get("rows", [])}
    return orig_rows != new_rows


def is_in_scope(row: dict, cadence: str, now: datetime) -> bool:
    """Row applies to this cadence AND any waiver is still in effect (or absent).

    Rows carry `cadences: list[str]`; a row is in scope iff the requested
    cadence is one of the row's tagged cadences. The legacy scalar
    `cadence` key is no longer recognized — every catalog row migrated
    to the list shape per scripts/migrations/2026-05-cadence-to-list.py.
    """
    if cadence not in row.get("cadences", []):
        return False
    return True  # waiver doesn't drop scope; it just changes status to WAIVED


def waiver_active(row: dict, now: datetime) -> bool:
    waiver = row.get("waiver")
    if waiver is None:
        return False
    until = waiver.get("until")
    if not until:
        return False
    try:
        return parse_rfc3339(until) > now
    except (ValueError, TypeError):
        return False


def is_stale(row: dict, now: datetime) -> bool:
    """Thin wrapper around _freshness.is_stale; injects parse_rfc3339."""
    return _is_stale_impl(row, now, parse_rfc3339)


def sort_by_blast_radius(rows: list[dict]) -> list[dict]:
    return sorted(rows, key=lambda r: BLAST_RADIUS_ORDER.get(r.get("blast_radius", "P2"), 99))


def write_artifact(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def run_row(row: dict, repo_root: Path, now: datetime) -> tuple[dict, float]:
    """Invoke verifier (or short-circuit), write artifact, return (updated row, elapsed_s).

    Sets in-memory `status` + `last_verified` for the caller; persistence
    back to the catalog is gated by `--persist` (and `catalog_dirty()`) in
    `main()` -- a bare cadence GATE run grades but never writes (D-P96-01).
    """
    started = time.monotonic()
    artifact_path = repo_root / row["artifact"] if row.get("artifact") else None

    # Real-backend env-gate skip (RBF-FW-01); must precede the WAIVED case.
    # RBF-FW-07b (M8, AMENDED D90-04): a cred-less skip is FAIL-CLOSED for ALL
    # pre-release-real-backend rows incl. the P0 litmus -- status flips (and
    # persists) NOT-VERIFIED so a stale real PASS can never ride a cred-less
    # milestone-close (the skip-as-pass channel OD-2 forbids). The prior REAL
    # grade is preserved in write-history fields, and the artifact carries an
    # explicit env-missing marker so the churn is explained, not silent.
    if _realbackend.is_skipped(row, os.environ):
        prior_status = row.get("status")
        artifact = {
            "ts": now_iso(), "row_id": row["id"], "exit_code": None,
            "skipped_real_backend": True,
            "skip_reason": "env-missing",  # machine marker (RBF-FW-07b)
            "skip_detail": _realbackend.skip_reason(os.environ),  # human recovery text
            "asserts_passed": [], "asserts_failed": [],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        # Preserve the prior REAL grade ONCE (never overwritten by a subsequent
        # skip: a second cred-less run sees prior_status NOT-VERIFIED and skips
        # this branch, leaving last_real_grade untouched -> idempotent).
        if prior_status in ("PASS", "FAIL", "PARTIAL"):
            row["last_real_grade"] = prior_status
            row["last_real_verified"] = row.get("last_verified")
        row["status"] = "NOT-VERIFIED"
        row["_skipped_real_backend"] = True  # transient; stripped before save
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # WAIVED case
    if waiver_active(row, now):
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": 0,
            "waiver": row["waiver"],
            "asserts_passed": [],
            "asserts_failed": [],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        row["status"] = "WAIVED"
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # STALE case (freshness TTL expired). Per P61 SUBJ-03: STALE is a flavor
    # of NOT-VERIFIED with a clearer label; compute_exit_code semantics
    # unchanged (P0+P1 NOT-VERIFIED -> exit 1; P2 NOT-VERIFIED -> exit 0).
    if is_stale(row, now):
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": None,
            "stale": True,
            "freshness_ttl": row["freshness_ttl"],
            "last_verified_input": row["last_verified"],
            "asserts_passed": [],
            "asserts_failed": [
                f"freshness expired: last_verified={row['last_verified']} + ttl={row['freshness_ttl']} < now"
            ],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        row["status"] = "NOT-VERIFIED"
        row["_stale"] = True  # transient flag for print_row_summary; not persisted
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # Verifier-not-found case
    script_rel = row.get("verifier", {}).get("script")
    script_abs = (repo_root / script_rel) if script_rel else None
    if not script_abs or not script_abs.exists():
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": None,
            "error": f"verifier not found at {script_rel}",
            "asserts_passed": [],
            "asserts_failed": [],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        # RBF-FW-07a (cross-AI H4): a missing verifier script is a
        # framework-integrity FAILURE, not a reason to preserve a stale PASS.
        # Flip UNCONDITIONALLY to NOT-VERIFIED; the artifact's distinct
        # `error: verifier not found at ...` marker (above) lets a downstream
        # reader tell a deploy glitch from a real regression at a glance.
        row["status"] = "NOT-VERIFIED"
        row["_verifier_missing"] = True  # transient; drives summary line, stripped before save
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # Subprocess case
    args = row.get("verifier", {}).get("args", []) or []
    timeout_s = int(row.get("verifier", {}).get("timeout_s", 30))
    suffix = script_abs.suffix
    if suffix == ".py":
        cmd = [sys.executable, str(script_abs), *args]
    elif suffix in (".sh", ""):
        cmd = ["bash", str(script_abs), *args]
    else:
        cmd = [str(script_abs), *args]

    # OBSERVABILITY NET (PERMANENT — keep AFTER the pre-pr-hang diagnosis).
    # Emit a per-gate START marker on run.py's OWN stdout (the stream CI
    # captures), FLUSHED, BEFORE the subprocess.run that could wedge. A hung
    # verifier otherwise prints NOTHING until it COMPLETES — only the DONE/
    # PASS/FAIL summary is emitted (by the caller, via print_row_summary) — so
    # a gate that never returns is INVISIBLE in the preserved CI log and its
    # name cannot be recovered. With this line the LAST START before a
    # timeout/SIGKILL names the wedged gate. Distinct `-> START` prefix (NOT a
    # `[STATUS]` summary line) so stdout-scraping tests (test_freshness_synth.py)
    # are unaffected; the caller's DONE/PASS/FAIL summary line is UNCHANGED.
    print(f"    -> START  {row['id']}  ({row.get('blast_radius', 'P2')})", flush=True)

    timed_out = False
    stdout = ""
    stderr = ""
    exit_code: int | None = None
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
            cwd=str(repo_root),
            shell=False,
            check=False,
        )
        exit_code = result.returncode
        stdout = result.stdout
        stderr = result.stderr
    except subprocess.TimeoutExpired as e:
        timed_out = True
        stdout = e.stdout.decode("utf-8", "replace") if isinstance(e.stdout, (bytes, bytearray)) else (e.stdout or "")
        stderr = e.stderr.decode("utf-8", "replace") if isinstance(e.stderr, (bytes, bytearray)) else (e.stderr or "")

    # Try to parse the artifact the verifier may have written (it has authoritative
    # asserts_passed / asserts_failed). If verifier wrote it, we keep its body and
    # only annotate top-level metadata. Otherwise we synthesize.
    # RBF-FW-02: existing dict-merge preserves verifier-written transcript_path (confirmed 89-04 step 6).
    artifact: dict[str, Any] = {}
    if artifact_path and artifact_path.exists():
        try:
            artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            artifact = {}
    artifact.setdefault("ts", now_iso())
    artifact["row_id"] = row["id"]
    artifact["exit_code"] = None if timed_out else exit_code
    artifact["timed_out"] = timed_out
    artifact.setdefault("stdout", stdout)
    artifact.setdefault("stderr", stderr)
    artifact.setdefault("asserts_passed", [])
    artifact.setdefault("asserts_failed", [])
    # RBF-FW-11: persist content-hash on the normally-graded path so verifier
    # subagents detect drift between row mint and grading run. Not persisted
    # on the real-backend-skip/WAIVED/STALE/verifier-not-found short-circuits
    # above -- those artifacts describe a non-execution, not a grade.
    audit_hash = _audit_field.compute_hash(row)
    if audit_hash:
        artifact["claim_vs_assertion_audit_hash"] = audit_hash

    if timed_out:
        row["status"] = "FAIL"
    else:
        # 0->PASS, 2->PARTIAL, 75->NOT-VERIFIED (RBF-FW-01), else FAIL.
        row["status"] = _realbackend.map_exit_code_to_status(exit_code)
        if exit_code == _realbackend.EXIT_NOT_VERIFIED:
            # Transient flag (89-06): distinguishes "verifier ran and
            # legitimately exited 75" from the verifier-not-found case at
            # line ~221 — both produce status NOT-VERIFIED, but only one is
            # a missing-script problem. Without this flag, main()'s summary
            # line falsely printed "verifier not found" for a verifier that
            # DID run (see 89-06 SUMMARY.md "Deviations"). Not persisted.
            row["_exit75_not_verified"] = True

    # Grade-time PASS honesty gates (RBF-FW-08 transcript evidence + F-K4b
    # per-expected-assert congruence). Logic lives in _audit_field to keep
    # run.py under its anti-bloat budget; no-op unless status == PASS.
    _audit_field.apply_pass_gates(row, artifact, repo_root)

    if artifact_path:
        if row.get("kind") == "shell-subprocess":
            # DETERMINISTIC committed verdict (D-P96-01, extended). A
            # kind:shell-subprocess row grades from its freshly-regenerated
            # transcript, so its committed artifact records ONLY the graded
            # result (exit_code + the asserts the scenario evaluated) plus a
            # STABLE transcript_path. Volatile fields the enrichment above added
            # -- the per-run `ts`, captured stdout/stderr, timed_out, the audit
            # hash -- are DROPPED here so a read-only pre-push GATE run never
            # re-dirties the tracked JSON (the stop-on-dirty hazard P96 left open
            # for per-row artifacts). Serialized through the SAME module
            # transcript.sh uses, so the gate's own write and this write-back are
            # byte-identical and the two producers never fight over the file.
            write_artifact(
                artifact_path,
                _shell_verdict.canonical_verdict(
                    row["id"],
                    artifact.get("exit_code"),
                    artifact.get("transcript_path"),
                    artifact.get("asserts_passed") or [],
                    artifact.get("asserts_failed") or [],
                ),
            )
        else:
            write_artifact(artifact_path, artifact)
    row["last_verified"] = artifact["ts"]
    return row, time.monotonic() - started


def compute_exit_code(rows: list[dict]) -> int:
    """Exit 0 iff every P0+P1 row is PASS or WAIVED."""
    for r in rows:
        blast = r.get("blast_radius", "P2")
        if blast in ("P0", "P1") and r.get("status") not in ("PASS", "WAIVED"):
            return 1
    return 0


def print_row_summary(row: dict, elapsed_s: float, extra: str = "") -> None:
    status = row.get("status", "?")
    # P61 SUBJ-03: a NOT-VERIFIED row that flipped via the freshness branch
    # carries a transient _stale=True flag; render as [STALE] for clarity.
    label = "STALE" if (status == "NOT-VERIFIED" and row.get("_stale")) else status
    blast = row.get("blast_radius", "?")
    rid = row.get("id", "?")
    suffix = f" -> {extra}" if extra else ""
    print(f"    [{label:<13}] {rid}  ({blast}, {elapsed_s:.2f}s){suffix}")


def _build_arg_parser() -> argparse.ArgumentParser:
    """Construct the CLI parser. Extracted from main() so tests can introspect
    the --persist default without driving a full run (test_run.py)."""
    parser = argparse.ArgumentParser(description="Quality Gates runner")
    parser.add_argument("--cadence", required=True, choices=VALID_CADENCES)
    parser.add_argument(
        "--persist",
        action="store_true",
        help=(
            "MINT MODE: write graded statuses back to quality/catalogs/*.json. "
            "Default OFF -- a bare cadence run is VALIDATE-ONLY (grades in "
            "memory, writes per-row artifacts, still blocks RED via the exit "
            "code) but does NOT mutate the committed catalog. Only the explicit "
            "phase-close / verifier-subagent grading invocation passes "
            "--persist (D-P96-01)."
        ),
    )
    parser.add_argument(
        "--allow-downgrade",
        action="store_true",
        help=(
            "DOWNGRADE OVERRIDE (P123 SC2): by default --persist REFUSES to "
            "write a committed-GREEN (PASS/WAIVED, per `git show HEAD:`) row "
            "back at an EXPLICIT worse grade (FAIL/PARTIAL) — it prints the row "
            "id, old->new status, and this flag as the recovery. Passing "
            "--allow-downgrade restores the unconditional-write behavior for "
            "that regression, still printing a loud per-row notice (never "
            "silent). A demotion to NOT-VERIFIED is NOT a downgrade and never "
            "needs this flag. Default OFF."
        ),
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    # P123 SC1/DRAIN-03: self-source ./.env FIRST — before any catalog load or
    # real-backend gating — so a `pre-release-real-backend` cadence exercises
    # real creds even when the caller did not pre-source .env into the shell.
    # Present-only + non-clobbering (existing env wins); a harmless no-op for
    # cadences that never consult real-backend creds and for CI (no .env).
    _env_load.load_dotenv_if_present(REPO_ROOT)
    args = _build_arg_parser().parse_args(argv)

    mode = ("MINT (--persist: catalog writes ON)" if args.persist
            else "validate-only (catalog writes OFF)")
    print(f"quality/runners/run.py --cadence {args.cadence}  [{mode}]")
    now = datetime.now(timezone.utc)

    catalogs = discover_catalogs()
    if not catalogs:
        print(f"  (no catalogs under {CATALOG_DIR})")
        return 0

    all_rows: list[dict] = []
    pending_mint: list[str] = []  # catalogs with unpersisted flips (validate-only)
    blocked_downgrade_catalogs: list[str] = []  # P123 SC2: --persist refused a downgrade
    counts = {"PASS": 0, "FAIL": 0, "PARTIAL": 0, "WAIVED": 0, "NOT-VERIFIED": 0}

    for cat_path in catalogs:
        # P123 SC3 (DRAIN-05, cites GTH-V15-01): a --persist MINT run holds an
        # OS-level advisory flock across the ENTIRE per-catalog read-modify-write
        # (load_catalog -> grade -> save_catalog), so two concurrent --persist
        # runners cannot both read the same pre-mutation on-disk snapshot and then
        # lost-update each other's flips with a stale full-file overwrite. The lock
        # MUST wrap load_catalog THROUGH save_catalog -- a narrower scope (only
        # around the write, or only around the committed_head_statuses read below)
        # leaves the lost-update window open. A validate-only run takes the
        # nullcontext branch and never opens or contends for the lock file at all
        # (backs structure/persist-catalog-write-locked; validate-only stays
        # lock-free per that row's second expected assert).
        persist_cm = (
            _persist_guard.catalog_persist_lock(REPO_ROOT)
            if args.persist else contextlib.nullcontext()
        )
        with persist_cm:
            original = load_catalog(cat_path)
            # Deep-copy via JSON round-trip so we can compare status before/after
            # without sharing references that mutate during run_row().
            data = json.loads(json.dumps(original))
            in_scope = [r for r in data["rows"] if is_in_scope(r, args.cadence, now)]
            if not in_scope:
                continue
            in_scope_sorted = sort_by_blast_radius(in_scope)
            print(f"  catalog: {cat_path.name} ({len(data['rows'])} rows; {len(in_scope_sorted)} in scope)")
            # Map id -> original status for last_verified rollback below.
            orig_status_by_id = {r["id"]: r.get("status") for r in original["rows"]}
            orig_lv_by_id = {r["id"]: r.get("last_verified") for r in original["rows"]}
            for row in in_scope_sorted:
                updated, elapsed = run_row(row, REPO_ROOT, now)
                counts[updated.get("status", "NOT-VERIFIED")] += 1
                extra = ""
                if updated.get("status") == "NOT-VERIFIED":
                    if updated.get("_stale"):
                        extra = (
                            f"freshness expired: last_verified+{row.get('freshness_ttl')} < now "
                            f"(input last_verified={orig_lv_by_id.get(updated['id'])})"
                        )
                    elif updated.get("_skipped_real_backend"):
                        extra = "skipped: env not set (real-backend origins/creds absent)"
                    elif updated.get("_exit75_not_verified"):
                        extra = "verifier exited 75 (NOT-VERIFIED convention; not a missing-script error)"
                    elif updated.get("_verifier_missing"):
                        extra = (
                            f"verifier not found at {row.get('verifier', {}).get('script')} "
                            f"(RBF-FW-07a: demoted from prior status; deploy glitch vs regression "
                            f"-> see artifact `error` field)"
                        )
                    else:
                        extra = f"verifier not found at {row.get('verifier', {}).get('script')}"
                elif updated.get("status") == "WAIVED":
                    w = updated.get("waiver") or {}
                    extra = f"waived until {w.get('until', '?')} — {w.get('reason', '')[:60]}"
                print_row_summary(updated, elapsed, extra)
                all_rows.append(updated)
            # Roll back last_verified for rows whose status did NOT change;
            # persist only on real status flips. Per-run timestamp churn
            # belongs in the artifact, not the catalog (see catalog_dirty).
            for row in data["rows"]:
                row.pop("_stale", None)  # transient render flags — never persisted
                row.pop("_skipped_real_backend", None)
                row.pop("_exit75_not_verified", None)
                row.pop("_verifier_missing", None)
                rid = row.get("id")
                if rid in orig_status_by_id and row.get("status") == orig_status_by_id[rid]:
                    row["last_verified"] = orig_lv_by_id[rid]
            # D-P96-01: only an explicit --persist MINT run writes graded status
            # back to disk. A bare cadence GATE run (pre-push/pre-pr hook + CI) is
            # validate-only -- it computed the flip in memory (compute_exit_code
            # below STILL blocks RED off that in-memory status), but must NOT
            # self-mutate the committed catalog. Record the pending flip so a human
            # sees it without any tree write.
            if catalog_dirty(original, data):
                if args.persist:
                    # P123 SC2 (DRAIN-04): refuse to silently downgrade a committed-
                    # GREEN (PASS/WAIVED, per `git show HEAD:`) row to an EXPLICIT
                    # regression (FAIL/PARTIAL) without --allow-downgrade. A demotion
                    # to NOT-VERIFIED (freshness-TTL/missing-verifier/env-skip/exit-75)
                    # is NOT a downgrade and is always allowed (see _persist_guard).
                    committed = _persist_guard.committed_head_statuses(REPO_ROOT, cat_path)
                    violations = _persist_guard.refuse_downgrade_without_flag(
                        committed, data["rows"])
                    if violations and not args.allow_downgrade:
                        for row_id, old, new in violations:
                            print(
                                f"REFUSED to persist {row_id}: committed status was "
                                f"{old}, this run graded {new}. Pass --allow-downgrade "
                                f"to override: python3 quality/runners/run.py --cadence "
                                f"{args.cadence} --persist --allow-downgrade",
                                file=sys.stderr,
                            )
                        blocked_downgrade_catalogs.append(cat_path.name)
                    else:
                        if violations:  # --allow-downgrade set: persist, but loudly.
                            for row_id, old, new in violations:
                                print(
                                    f"ALLOWED downgrade (--allow-downgrade): "
                                    f"{row_id} {old} -> {new}",
                                    file=sys.stderr,
                                )
                        save_catalog(cat_path, data)
                else:
                    pending_mint.append(cat_path.name)

    if pending_mint and not args.persist:
        print(
            f"note: validate-only run -- {len(pending_mint)} catalog(s) have "
            f"status flips NOT persisted ({', '.join(sorted(set(pending_mint)))}). "
            f"To mint the new grade(s): "
            f"python3 quality/runners/run.py --cadence {args.cadence} --persist"
        )

    if blocked_downgrade_catalogs:
        print(
            f"note: --persist REFUSED a committed-GREEN downgrade in "
            f"{len(blocked_downgrade_catalogs)} catalog(s) "
            f"({', '.join(sorted(set(blocked_downgrade_catalogs)))}); those files "
            f"were NOT written (see the REFUSED line(s) above). Re-run with "
            f"--allow-downgrade only if the regression is intended.",
            file=sys.stderr,
        )

    exit_code = compute_exit_code(all_rows)
    summary = (
        f"summary: {counts['PASS']} PASS, {counts['FAIL']} FAIL, "
        f"{counts['PARTIAL']} PARTIAL, {counts['WAIVED']} WAIVED, "
        f"{counts['NOT-VERIFIED']} NOT-VERIFIED -> exit={exit_code}"
    )
    print(summary)
    # P123 SC2: a blocked downgrade always surfaces as a failing run, never
    # swallowed into an otherwise-green exit (a refused P2 downgrade would not
    # trip compute_exit_code on its own).
    if blocked_downgrade_catalogs:
        exit_code = max(exit_code, 1)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
