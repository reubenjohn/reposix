#!/usr/bin/env python3
"""Release-dim verifier — brew-formula-current.

Backs catalog rows: release/brew-formula-current AND install/homebrew
(writes both artifacts).

Asserts:
- GET https://api.github.com/repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb returns 200
- formula's `version "X.Y.Z"` parses
- crates.io reposix-cli `max_version` parses
- `[workspace.package]` version parses from REPO_ROOT/Cargo.toml (local file)
- GET https://api.github.com/repos/reubenjohn/reposix/releases/latest returns 200
  and its `tag_name` (leading `v` stripped) parses as the latest published release
- WINDOW-AWARE coherence predicate (fail-closed; only reached when all four
  versions — formula / crates.io max / workspace / latest GitHub release — parsed):
    * PASS in steady state:  formula == crates.io max_version
    * PASS in an in-flight release window:  crates.io max_version == workspace version
      (crates.io already has the just-merged bump) AND formula == latest GitHub release
      (the tap correctly tracks the last COMPLETED release; release.yml simply hasn't
      fired for the new v-tag yet)
    * else FAIL: the formula is stale relative to a COMPLETED release (the real bug
      this gate guards)

Stdlib only (urllib.request, base64, re, json + a local Cargo.toml read).
Cross-platform.

Exit codes: 0 PASS | 1 FAIL.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "release"
CARGO_TOML = REPO_ROOT / "Cargo.toml"
UA = "reposix-quality-gates/0.12.0"
FORMULA_URL = "https://api.github.com/repos/reubenjohn/homebrew-reposix/contents/Formula/reposix.rb"
CRATES_URL = "https://crates.io/api/v1/crates/reposix-cli"
GH_RELEASE_URL = "https://api.github.com/repos/reubenjohn/reposix/releases/latest"
VERSION_RE = re.compile(r'version\s+"(\d+\.\d+\.\d+(?:-[\w\.]+)?)"')
# version = "X.Y.Z" line, scoped by the caller to the [workspace.package] table only.
WS_VERSION_RE = re.compile(r'^\s*version\s*=\s*"(\d+\.\d+\.\d+(?:-[\w\.]+)?)"')


def parse_workspace_version(cargo_text: str) -> str | None:
    """Parse `version = "X.Y.Z"` from the [workspace.package] table only.

    Scoped to that table so we never accidentally match a version key in another
    table (e.g. [workspace.metadata.dist] cargo-dist-version). reposix-cli inherits
    this via `version.workspace = true`, so the workspace version IS the reposix-cli
    publish version.
    """
    in_section = False
    for line in cargo_text.splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_section = stripped == "[workspace.package]"
            continue
        if in_section:
            m = WS_VERSION_RE.match(line)
            if m:
                return m.group(1)
    return None


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def http_get_json(url: str, timeout: int = 30) -> tuple[int, Any, str]:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, json.loads(resp.read().decode("utf-8")), ""
    except urllib.error.HTTPError as e:
        return e.code, None, f"HTTPError: {e}"
    except (urllib.error.URLError, json.JSONDecodeError, OSError) as e:
        return 0, None, f"{type(e).__name__}: {e}"


def write_artifact(filename: str, payload: dict) -> None:
    out = ARTIFACT_DIR / filename
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="brew-formula-current — assert tap formula matches crates.io")
    parser.add_argument("--row-id", default=None, help="optional override (default: writes both artifacts)")
    args = parser.parse_args()

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    formula_version = None
    max_version = None
    workspace_version = None
    gh_release_version = None

    fstatus, fbody, ferr = http_get_json(FORMULA_URL)
    if fstatus != 200 or fbody is None:
        asserts_failed.append(f"GET {FORMULA_URL} HTTP 200 — got status={fstatus}; err={ferr}")
    else:
        asserts_passed.append(f"GET {FORMULA_URL} returns HTTP 200")
        b64 = (fbody.get("content") or "").replace("\n", "").strip()
        try:
            formula_text = base64.b64decode(b64).decode("utf-8")
            m = VERSION_RE.search(formula_text)
            if m:
                formula_version = m.group(1)
                asserts_passed.append(f"formula version parsed: {formula_version}")
            else:
                asserts_failed.append("formula text does not contain `version \"X.Y.Z\"`")
        except (ValueError, UnicodeDecodeError) as e:
            asserts_failed.append(f"failed to base64-decode formula content: {e}")

    cstatus, cbody, cerr = http_get_json(CRATES_URL)
    if cstatus != 200 or cbody is None:
        asserts_failed.append(f"GET {CRATES_URL} HTTP 200 — got status={cstatus}; err={cerr}")
    else:
        asserts_passed.append(f"GET {CRATES_URL} returns HTTP 200")
        try:
            max_version = cbody["crate"]["max_version"]
            asserts_passed.append(f"crates.io reposix-cli max_version: {max_version}")
        except (KeyError, TypeError) as e:
            asserts_failed.append(f"crates.io response missing crate.max_version: {e}")

    # Local file: the [workspace.package] version reposix-cli publishes at.
    try:
        cargo_text = CARGO_TOML.read_text(encoding="utf-8")
        workspace_version = parse_workspace_version(cargo_text)
        if workspace_version:
            asserts_passed.append(f"[workspace.package] version parsed: {workspace_version}")
        else:
            asserts_failed.append(
                f"could not parse [workspace.package] version = \"X.Y.Z\" in {CARGO_TOML}"
            )
    except OSError as e:
        asserts_failed.append(f"failed to read {CARGO_TOML}: {e}")

    # NEW network dependency: the latest COMPLETED GitHub release. Fail closed on error.
    gstatus, gbody, gerr = http_get_json(GH_RELEASE_URL)
    if gstatus != 200 or gbody is None:
        asserts_failed.append(f"GET {GH_RELEASE_URL} HTTP 200 — got status={gstatus}; err={gerr}")
    else:
        asserts_passed.append(f"GET {GH_RELEASE_URL} returns HTTP 200")
        tag = gbody.get("tag_name")
        if tag:
            gh_release_version = tag[1:] if tag.startswith("v") else tag
            asserts_passed.append(
                f"latest GitHub release tag: {tag} -> version {gh_release_version}"
            )
        else:
            asserts_failed.append("GitHub releases/latest response missing tag_name")

    # Window-aware coherence predicate. Only reached when ALL FOUR versions parsed;
    # otherwise the per-source failure asserts above already fail the gate (fail closed).
    if formula_version and max_version and workspace_version and gh_release_version:
        if formula_version == max_version:
            asserts_passed.append(
                f"steady state: formula version == crates.io max_version ({formula_version})"
            )
        elif max_version == workspace_version and formula_version == gh_release_version:
            asserts_passed.append(
                f"in-flight release window: crates.io={max_version}==workspace, "
                f"formula={formula_version}==latest GitHub release ({gh_release_version}) "
                f"— crates.io already has the just-merged bump; release.yml has not yet "
                f"fired for the new v-tag, so the formula correctly still tracks the last "
                f"COMPLETED release"
            )
        else:
            asserts_failed.append(
                f"formula {formula_version!r} != crates.io {max_version!r}, and this is NOT a "
                f"legitimate in-flight release window because "
                f"crates.io ({max_version}) != workspace ({workspace_version}) "
                f"OR formula ({formula_version}) != latest GitHub release ({gh_release_version}) "
                f"— the formula is stale relative to a COMPLETED release. "
                f"Recovery: re-run / check the .github/workflows/release.yml "
                f"upload-homebrew-formula job for the latest reposix-cli-v* tag "
                f"(HOMEBREW_TAP_TOKEN secret missing/expired is the usual root cause)."
            )

    payload = {
        "ts": now_iso(),
        "row_id": "release/brew-formula-current",
        "formula_url": FORMULA_URL,
        "crates_url": CRATES_URL,
        "gh_release_url": GH_RELEASE_URL,
        "formula_version": formula_version,
        "crates_io_max_version": max_version,
        "workspace_version": workspace_version,
        "gh_release_version": gh_release_version,
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
    }
    write_artifact("brew-formula-current.json", payload)

    payload2 = dict(payload)
    payload2["row_id"] = "install/homebrew"
    write_artifact("install-homebrew.json", payload2)

    return 0 if not asserts_failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
