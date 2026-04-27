#!/usr/bin/env python3
"""SESSION-END-STATE framework — verifier-graded session done-ness.

Per HANDOVER.md §0.8. Replaces self-reported "DONE" with verifier-graded
PASS/FAIL/PARTIAL. Every session declares a contract of claims, each
backed by a verifier command (or an artifact path that an unbiased
subagent produces). The session is GREEN only when every claim PASSes.

Subcommands:
  init [--session-id ID]   bootstrap claims, write JSON + MD contract
  list                     list all claim ids + statuses
  status                   summary counts (PASS / FAIL / PARTIAL / NOT-VERIFIED)
  verify [--claim ID]      run verifier for one or all claims; update statuses
  verdict                  print summary, write VERDICT.md, exit 0 iff all PASS
  record-artifact ID PATH  record an externally-produced artifact for claim ID

Files:
  .planning/SESSION-END-STATE.json          source of truth (this script writes)
  .planning/SESSION-END-STATE.md            prose contract (human-readable)
  .planning/SESSION-END-STATE-VERDICT.md    written by `verdict`
  .planning/verifications/<category>/...    per-claim artifacts (playwright,
                                            crates-io, cargo, etc.)

Stdlib only. Complements scripts/catalog.py (which tracks per-FILE
dispositions); end-state tracks per-CLAIM verifications.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
STATE_JSON = REPO_ROOT / ".planning" / "SESSION-END-STATE.json"
STATE_MD = REPO_ROOT / ".planning" / "SESSION-END-STATE.md"
VERDICT_MD = REPO_ROOT / ".planning" / "SESSION-END-STATE-VERDICT.md"
VERIF_DIR = REPO_ROOT / ".planning" / "verifications"

SCHEMA_VERSION = 1

VALID_STATUSES = {"PASS", "FAIL", "PARTIAL", "NOT-VERIFIED"}
VALID_VERIFIER_TYPES = {"shell", "command-empty-output", "artifact-json-exists"}


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def load_state() -> dict[str, Any]:
    if not STATE_JSON.exists():
        return {}
    return json.loads(STATE_JSON.read_text(encoding="utf-8"))


def save_state(state: dict[str, Any]) -> None:
    STATE_JSON.parent.mkdir(parents=True, exist_ok=True)
    STATE_JSON.write_text(
        json.dumps(state, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def workspace_version() -> str:
    """Read [workspace.package].version from the root Cargo.toml."""
    cargo = REPO_ROOT / "Cargo.toml"
    in_workspace_pkg = False
    for raw in cargo.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line.startswith("[workspace.package]"):
            in_workspace_pkg = True
            continue
        if in_workspace_pkg and line.startswith("[") and line != "[workspace.package]":
            break
        if in_workspace_pkg and line.startswith("version"):
            return line.split("=", 1)[1].strip().strip('"')
    raise RuntimeError("could not parse [workspace.package].version from Cargo.toml")


def how_it_works_pages() -> list[str]:
    src = REPO_ROOT / "docs" / "how-it-works"
    if not src.is_dir():
        return []
    return sorted(p.name for p in src.glob("*.md"))


def source_mermaid_pages() -> list[str]:
    """Every docs page whose Markdown source contains a ```mermaid fence.

    Returned as repo-relative paths under docs/, e.g. "index.md",
    "how-it-works/git-layer.md". Used to build the playwright-artifact
    claim set (one claim per source-mermaid page).
    """
    docs = REPO_ROOT / "docs"
    if not docs.is_dir():
        return []
    pages: list[str] = []
    for md in sorted(docs.rglob("*.md")):
        try:
            text = md.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if "```mermaid" in text:
            pages.append(str(md.relative_to(docs)))
    return pages


def published_crates() -> list[str]:
    """Crates that ship to crates.io. Stays in sync with §3b verify block."""
    return [
        "reposix-core",
        "reposix-cache",
        "reposix-sim",
        "reposix-github",
        "reposix-confluence",
        "reposix-jira",
        "reposix-remote",
        "reposix-cli",
    ]


# ---------------------------------------------------------------------------
# Bootstrap claim definitions
# ---------------------------------------------------------------------------


def bootstrap_claims() -> list[dict[str, Any]]:
    """Generate the bootstrap claim list for this session.

    Each claim is one row in the contract. The verifier `command` is a single
    shell line; for `command-empty-output`, PASS = empty stdout; for `shell`,
    PASS = exit 0.
    """
    claims: list[dict[str, Any]] = []

    # ---- Freshness invariants (CLAUDE.md "Freshness invariants") ----
    claims.append({
        "id": "freshness/no-version-pinned-filenames",
        "category": "freshness-invariant",
        "description": (
            "No version-pinned filenames (vN.N.N) outside CHANGELOG and "
            ".planning/milestones/v*-phases/. Catches §0.3-class drift."
        ),
        "verifier": {
            "type": "command-empty-output",
            "command": (
                "find docs scripts -type f "
                "| grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+' "
                "| grep -v CHANGELOG || true"
            ),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    # Source-compile pattern detection: `git clone https://` (URL → real
    # clone-and-build, not the prose "agent can `git clone`") AND
    # `cargo build --release` (workspace-wide source build). The verifier
    # asserts pkg-mgr install appears BEFORE either source-compile marker.
    install_check_template = (
        "python3 -c \"import re,sys; t=open('{path}').read(); "
        "pm=re.search(r'(?im)(?:brew install|cargo binstall|curl[^\\n]*\\| ?sh|powershell[^\\n]*irm)', t); "
        "src=[m.start() for m in re.finditer(r'(?im)git clone https?://|cargo build --release', t)]; "
        "first_src=min(src) if src else None; "
        "sys.exit(0 if pm and (first_src is None or pm.start() < first_src) else 1)\""
    )

    claims.append({
        "id": "freshness/install-leads-with-pkg-mgr/docs-index",
        "category": "freshness-invariant",
        "description": (
            "docs/index.md hero must show a package-manager install command "
            "(brew/binstall/curl|sh/PowerShell-irm) BEFORE any 'git clone "
            "https://' or 'cargo build --release' source-compile snippet. "
            "Catches §0.2 drift. Bare prose 'git clone' (e.g. 'agent can git "
            "clone') does NOT trip the check — only URL-form source builds."
        ),
        "verifier": {
            "type": "shell",
            "command": install_check_template.format(path="docs/index.md"),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    claims.append({
        "id": "freshness/install-leads-with-pkg-mgr/README",
        "category": "freshness-invariant",
        "description": (
            "README.md must show a package-manager install command BEFORE "
            "any 'git clone https://' or 'cargo build --release' source-"
            "compile snippet. §0.2."
        ),
        "verifier": {
            "type": "shell",
            "command": install_check_template.format(path="README.md"),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    claims.append({
        "id": "freshness/benchmarks-in-mkdocs-nav",
        "category": "freshness-invariant",
        "description": (
            "Every docs/benchmarks/*.md must appear in mkdocs.yml nav OR in "
            "not_in_nav. No benchmark behind an absolute github URL. §0.4."
        ),
        "verifier": {
            "type": "shell",
            "command": (
                "python3 -c \"import sys,pathlib,re; "
                "y=open('mkdocs.yml').read(); "
                "missing=[p.name for p in pathlib.Path('docs/benchmarks').glob('*.md') "
                "if p.name not in y]; "
                "print('\\n'.join(missing)); sys.exit(1 if missing else 0)\""
            ),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    claims.append({
        "id": "freshness/no-loose-roadmap-or-requirements",
        "category": "freshness-invariant",
        "description": (
            "No loose v*ROADMAP*.md / v*REQUIREMENTS*.md outside *phases/ or "
            ".planning/archive/. §0.5."
        ),
        "verifier": {
            "type": "command-empty-output",
            "command": (
                "find .planning/milestones -maxdepth 2 "
                "\\( -name '*ROADMAP*' -o -name '*REQUIREMENTS*' \\) "
                "| grep -v phases | grep -v archive || true"
            ),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    claims.append({
        "id": "freshness/no-orphan-docs",
        "category": "freshness-invariant",
        "description": (
            "Every docs/**/*.md must be in mkdocs.yml nav: OR in not_in_nav: "
            "OR a redirect_maps target. mkdocs --strict already enforces this; "
            "this row is a tripwire if --strict is ever softened."
        ),
        "verifier": {
            "type": "shell",
            "command": "bash scripts/check-docs-site.sh > /dev/null 2>&1",
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    # ---- §0.1 — mermaid render artifacts per source-mermaid page ----
    # Enumerates EVERY docs/**/*.md page that source-references a ```mermaid
    # fence (not just how-it-works/). The §0.1.b fix to mermaid-render.js
    # fixed all of them at once, but the verifier insists on artifact JSON
    # for every page so a regression on any page raises a FAIL row.
    for rel in source_mermaid_pages():
        slug_path = rel[: -len(".md")] if rel.endswith(".md") else rel
        # Use the full path-with-slashes as both claim id and artifact path
        # so two pages with the same basename in different dirs don't collide.
        artifact = f".planning/verifications/playwright/{slug_path}.json"
        claims.append({
            "id": f"mermaid-renders/{slug_path}",
            "category": "doc-page-with-mermaid",
            "description": (
                f"docs/{rel} — playwright walk on a cache-cold navigation "
                "must show every <pre.mermaid> block has at least one <svg> "
                "child AND zero render-error console messages. §0.1 + §0.1.b."
            ),
            "verifier": {
                "type": "artifact-json-exists",
                "command": (
                    f"python3 -c \"import json,sys,pathlib; "
                    f"p=pathlib.Path('{artifact}'); "
                    f"sys.exit(2) if not p.exists() else None; "
                    f"d=json.loads(p.read_text()); "
                    f"mc=d.get('mermaid_count',0); "
                    f"sv=d.get('svg_counts',[]); "
                    f"ce=d.get('console_errors',[]); "
                    f"sys.exit(0 if (mc==0 or (len(sv)==mc and all(x>0 for x in sv))) "
                    f"and not ce else 1)\""
                ),
            },
            "artifact": artifact,
            "blocked_by_claim": [],
            "status": "NOT-VERIFIED",
            "last_verified_at": None,
            "last_run_log_path": None,
        })

    # ---- §3a — CI green on origin/main HEAD ----
    # Query CI runs targeting HEAD specifically, not "latest completed."
    # Why: GitHub's concurrency group cancels older in-flight CI runs when a
    # new commit lands; the most-recent-completed run is therefore often the
    # superseded one and cancelled, even though HEAD's own run is healthy.
    # Semantic: PASS if HEAD's most recent CI run is success OR currently
    # in_progress (optimistic — the verifier subagent re-runs at session
    # close). FAIL only if HEAD's CI completed with a non-success conclusion.
    claims.append({
        "id": "ci/main-green-on-head",
        "category": "ci-status",
        "description": (
            "CI workflow targeting `origin/main` HEAD must be either "
            "in_progress or completed with conclusion=success. A failed/"
            "cancelled completion on HEAD is FAIL. §3a."
        ),
        "verifier": {
            "type": "shell",
            "command": (
                "sha=$(git rev-parse origin/main); "
                "row=$(gh run list --branch main --workflow CI --commit \"$sha\" "
                "--limit 1 --json status,conclusion --jq '.[0]'); "
                "status=$(echo \"$row\" | python3 -c "
                "'import json,sys; print(json.load(sys.stdin).get(\"status\",\"missing\"))'); "
                "conclusion=$(echo \"$row\" | python3 -c "
                "'import json,sys; print(json.load(sys.stdin).get(\"conclusion\",\"\"))'); "
                "test \"$status\" = in_progress -o "
                "\\( \"$status\" = completed -a \"$conclusion\" = success \\)"
            ),
        },
        "artifact": None,
        "blocked_by_claim": [],
        "status": "NOT-VERIFIED",
        "last_verified_at": None,
        "last_run_log_path": None,
    })

    # ---- §3b — every published crate at workspace version on crates.io ----
    # Use grep instead of `python3 -c` to avoid shell-quoting headaches; this
    # mirrors the state-gather block at the top of HANDOVER.md.
    ws = workspace_version()
    for crate in published_crates():
        claims.append({
            "id": f"crates-io/{crate}-at-workspace-version",
            "category": "crate-publish",
            "description": (
                f"crates.io max_version of {crate} must equal workspace "
                f"version ({ws}). §3b."
            ),
            "verifier": {
                "type": "shell",
                "command": (
                    f"v=$(curl -s https://crates.io/api/v1/crates/{crate} "
                    f"| grep -o '\"max_version\":\"[^\"]*\"' | head -1 "
                    f"| sed 's/.*://;s/\"//g'); "
                    f"test \"$v\" = \"{ws}\""
                ),
            },
            "artifact": f".planning/verifications/crates-io/{crate}.json",
            "blocked_by_claim": [],
            "status": "NOT-VERIFIED",
            "last_verified_at": None,
            "last_run_log_path": None,
        })

    return claims


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------


def cmd_init(args: argparse.Namespace) -> int:
    """Bootstrap or refresh the claim contract."""
    existing = load_state() if STATE_JSON.exists() else {}
    session_id = (
        args.session_id
        or existing.get("session_id")
        or str(uuid.uuid4())
    )
    started_at = existing.get("session_started_at") or now_iso()
    claims = bootstrap_claims()

    # Preserve PASS/FAIL state for claims that already existed in the prior file
    # (so re-running `init` is idempotent and doesn't blow away verifier results).
    if not args.fresh:
        prior_by_id = {c["id"]: c for c in existing.get("claims", [])}
        for claim in claims:
            prior = prior_by_id.get(claim["id"])
            if prior is not None:
                for key in ("status", "last_verified_at", "last_run_log_path"):
                    if prior.get(key) is not None:
                        claim[key] = prior[key]

    state = {
        "schema_version": SCHEMA_VERSION,
        "session_id": session_id,
        "session_started_at": started_at,
        "workspace_version": workspace_version(),
        "claims": claims,
    }
    save_state(state)
    write_contract_md(state)
    print(f"init: wrote {STATE_JSON.relative_to(REPO_ROOT)} ({len(claims)} claims)")
    print(f"      wrote {STATE_MD.relative_to(REPO_ROOT)}")
    print(f"      session_id = {session_id}")
    return 0


def write_contract_md(state: dict[str, Any]) -> None:
    """Render the human-readable prose contract."""
    lines: list[str] = []
    lines.append("# SESSION-END-STATE — current contract")
    lines.append("")
    lines.append(
        "Auto-generated by `scripts/end-state.py init`. **Do not hand-edit;** "
        "run `init` to regenerate. The session is GREEN only when every claim "
        "below has status `PASS` per `scripts/end-state.py verdict`."
    )
    lines.append("")
    lines.append(f"- Session ID: `{state['session_id']}`")
    lines.append(f"- Session started at: `{state['session_started_at']}`")
    lines.append(f"- Workspace version: `{state['workspace_version']}`")
    lines.append(f"- Schema version: `{state['schema_version']}`")
    lines.append("")
    lines.append("## Claims")
    lines.append("")
    by_cat: dict[str, list[dict[str, Any]]] = {}
    for c in state["claims"]:
        by_cat.setdefault(c["category"], []).append(c)
    for cat in sorted(by_cat):
        lines.append(f"### {cat}")
        lines.append("")
        for c in by_cat[cat]:
            lines.append(f"- **`{c['id']}`** — {c['description']}")
            v = c["verifier"]
            lines.append(f"  - verifier ({v['type']}): `{v['command']}`")
            if c.get("artifact"):
                lines.append(f"  - artifact: `{c['artifact']}`")
            lines.append(f"  - status: `{c.get('status', 'NOT-VERIFIED')}`")
        lines.append("")
    lines.append("---")
    lines.append("")
    lines.append(
        "_Verifier output is written to "
        "`.planning/verifications/_logs/<claim-id>.txt`. Artifact files (e.g. "
        "playwright JSON, crates-io JSON) are written under the matching "
        "`.planning/verifications/<category>/` subdir._"
    )
    STATE_MD.parent.mkdir(parents=True, exist_ok=True)
    STATE_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_verifier(claim: dict[str, Any]) -> tuple[str, Path]:
    """Execute one claim's verifier; return (status, log_path)."""
    log_dir = VERIF_DIR / "_logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    safe_id = claim["id"].replace("/", "__")
    log_path = log_dir / f"{safe_id}.txt"
    v = claim["verifier"]
    vtype = v["type"]
    cmd = v["command"]
    if vtype not in VALID_VERIFIER_TYPES:
        log_path.write_text(f"unknown verifier type: {vtype}\n", encoding="utf-8")
        return "FAIL", log_path

    try:
        proc = subprocess.run(
            ["bash", "-c", cmd],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=180,
        )
    except subprocess.TimeoutExpired as e:
        log_path.write_text(
            f"TIMEOUT after {e.timeout}s\nstdout:\n{e.stdout or ''}\n"
            f"stderr:\n{e.stderr or ''}\n",
            encoding="utf-8",
        )
        return "FAIL", log_path

    log_path.write_text(
        f"command: {cmd}\nexit: {proc.returncode}\n"
        f"--- stdout ---\n{proc.stdout}\n"
        f"--- stderr ---\n{proc.stderr}\n",
        encoding="utf-8",
    )

    if vtype == "shell":
        return ("PASS" if proc.returncode == 0 else "FAIL"), log_path
    if vtype == "artifact-json-exists":
        # exit 2 = missing artifact (NOT-VERIFIED — work hasn't been done yet)
        if proc.returncode == 2:
            return "NOT-VERIFIED", log_path
        return ("PASS" if proc.returncode == 0 else "FAIL"), log_path
    if vtype == "command-empty-output":
        if proc.returncode != 0:
            return "FAIL", log_path
        return ("PASS" if proc.stdout.strip() == "" else "FAIL"), log_path
    return "FAIL", log_path


def cmd_verify(args: argparse.Namespace) -> int:
    state = load_state()
    if not state:
        print("verify: no SESSION-END-STATE.json — run `init` first", file=sys.stderr)
        return 1
    targets = state["claims"]
    if args.claim:
        targets = [c for c in state["claims"] if c["id"] == args.claim]
        if not targets:
            print(f"verify: no claim with id {args.claim!r}", file=sys.stderr)
            return 1

    pass_count = fail_count = nv_count = 0
    for claim in targets:
        status, log_path = run_verifier(claim)
        claim["status"] = status
        claim["last_verified_at"] = now_iso()
        claim["last_run_log_path"] = str(log_path.relative_to(REPO_ROOT))
        marker = {"PASS": "✓", "FAIL": "✖", "PARTIAL": "~", "NOT-VERIFIED": "·"}.get(
            status, "?"
        )
        print(f"  {marker} {status:<13} {claim['id']}")
        if status == "PASS":
            pass_count += 1
        elif status == "NOT-VERIFIED":
            nv_count += 1
        else:
            fail_count += 1

    save_state(state)
    # Re-render the prose contract so its `status:` lines reflect the latest
    # verify run — otherwise the markdown drifts from JSON. Caught by the
    # unbiased verifier subagent in Row 8.
    write_contract_md(state)
    print(f"verify: {pass_count} PASS, {fail_count} FAIL, {nv_count} NOT-VERIFIED")
    return 0 if fail_count == 0 and nv_count == 0 else 1


def cmd_list(args: argparse.Namespace) -> int:
    state = load_state()
    if not state:
        print("list: no SESSION-END-STATE.json — run `init` first", file=sys.stderr)
        return 1
    for c in state["claims"]:
        print(f"  [{c.get('status','NOT-VERIFIED'):<13}] {c['id']}")
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    state = load_state()
    if not state:
        print("status: no SESSION-END-STATE.json — run `init` first", file=sys.stderr)
        return 1
    counts: dict[str, int] = {s: 0 for s in VALID_STATUSES}
    for c in state["claims"]:
        counts[c.get("status", "NOT-VERIFIED")] += 1
    total = sum(counts.values())
    print(f"  total claims: {total}")
    for s in ("PASS", "FAIL", "PARTIAL", "NOT-VERIFIED"):
        print(f"  {s:<13} {counts[s]}")
    return 0


def cmd_record_artifact(args: argparse.Namespace) -> int:
    """Record an externally-produced artifact (e.g. playwright JSON written by
    a subagent) and re-verify the claim that depends on it."""
    state = load_state()
    if not state:
        print("record-artifact: no SESSION-END-STATE.json", file=sys.stderr)
        return 1
    claim = next((c for c in state["claims"] if c["id"] == args.claim_id), None)
    if claim is None:
        print(f"record-artifact: no claim {args.claim_id!r}", file=sys.stderr)
        return 1
    p = Path(args.path)
    if not p.is_absolute():
        p = REPO_ROOT / p
    if not p.exists():
        print(f"record-artifact: missing {p}", file=sys.stderr)
        return 1
    expected = claim.get("artifact")
    if expected:
        expected_abs = REPO_ROOT / expected
        if p.resolve() != expected_abs.resolve():
            print(
                f"record-artifact: path {p} does not match claim's artifact {expected}",
                file=sys.stderr,
            )
            return 1
    sig = hashlib.sha256(p.read_bytes()).hexdigest()
    claim["verifier_signature"] = f"sha256:{sig}"
    status, log_path = run_verifier(claim)
    claim["status"] = status
    claim["last_verified_at"] = now_iso()
    claim["last_run_log_path"] = str(log_path.relative_to(REPO_ROOT))
    save_state(state)
    print(f"record-artifact: {args.claim_id} → {status} (sig sha256:{sig[:12]}…)")
    return 0 if status == "PASS" else 1


def cmd_verdict(args: argparse.Namespace) -> int:
    """Render the verdict markdown and exit non-zero if any claim is not PASS."""
    state = load_state()
    if not state:
        print("verdict: no SESSION-END-STATE.json — run `init` first", file=sys.stderr)
        return 1

    counts: dict[str, int] = {s: 0 for s in VALID_STATUSES}
    by_status: dict[str, list[dict[str, Any]]] = {s: [] for s in VALID_STATUSES}
    for c in state["claims"]:
        s = c.get("status", "NOT-VERIFIED")
        counts[s] += 1
        by_status[s].append(c)

    overall = (
        "GREEN" if counts["FAIL"] == 0 and counts["NOT-VERIFIED"] == 0
        else "RED"
    )

    lines: list[str] = []
    lines.append(f"# SESSION-END-STATE-VERDICT — {overall}")
    lines.append("")
    lines.append(f"- Session ID: `{state['session_id']}`")
    lines.append(f"- Generated at: `{now_iso()}`")
    lines.append(f"- Workspace version: `{state['workspace_version']}`")
    lines.append("")
    lines.append("| status | count |")
    lines.append("|---|---|")
    for s in ("PASS", "FAIL", "PARTIAL", "NOT-VERIFIED"):
        lines.append(f"| {s} | {counts[s]} |")
    lines.append("")

    for s in ("FAIL", "NOT-VERIFIED", "PARTIAL"):
        if by_status[s]:
            lines.append(f"## {s}")
            lines.append("")
            for c in by_status[s]:
                lines.append(f"- `{c['id']}` — {c['description']}")
                if c.get("last_run_log_path"):
                    lines.append(f"  - log: `{c['last_run_log_path']}`")
                if c.get("artifact"):
                    lines.append(f"  - artifact (expected): `{c['artifact']}`")
            lines.append("")

    if by_status["PASS"]:
        lines.append("## PASS")
        lines.append("")
        for c in by_status["PASS"]:
            lines.append(f"- `{c['id']}`")
        lines.append("")

    lines.append("---")
    lines.append("")
    lines.append(
        "_Verdict above is computed from the last `verify` run per claim. To "
        "refresh, run `python3 scripts/end-state.py verify` then re-run "
        "`verdict`._"
    )

    VERDICT_MD.parent.mkdir(parents=True, exist_ok=True)
    VERDICT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")

    for s in ("PASS", "FAIL", "PARTIAL", "NOT-VERIFIED"):
        print(f"  {s:<13} {counts[s]}")
    print(f"verdict: {overall}  →  {VERDICT_MD.relative_to(REPO_ROOT)}")

    return 0 if overall == "GREEN" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_init = sub.add_parser("init", help="bootstrap claim contract")
    p_init.add_argument("--session-id", help="override session UUID")
    p_init.add_argument(
        "--fresh",
        action="store_true",
        help="discard prior PASS/FAIL state on re-init",
    )
    p_init.set_defaults(func=cmd_init)

    p_list = sub.add_parser("list", help="list all claim ids and statuses")
    p_list.set_defaults(func=cmd_list)

    p_status = sub.add_parser("status", help="summary counts only")
    p_status.set_defaults(func=cmd_status)

    p_verify = sub.add_parser("verify", help="run verifier(s) and update statuses")
    p_verify.add_argument("--claim", help="verify a single claim by id")
    p_verify.set_defaults(func=cmd_verify)

    p_verdict = sub.add_parser("verdict", help="render verdict file; exit 0 iff GREEN")
    p_verdict.set_defaults(func=cmd_verdict)

    p_rec = sub.add_parser(
        "record-artifact",
        help="record an externally-produced artifact and re-verify",
    )
    p_rec.add_argument("claim_id")
    p_rec.add_argument("path")
    p_rec.set_defaults(func=cmd_record_artifact)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
