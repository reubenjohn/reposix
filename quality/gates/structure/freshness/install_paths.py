"""Install-snippet ordering invariants.

Asserts the README + docs/index.md hero install snippets lead with a
package-manager command (brew / cargo binstall / curl|sh / powershell-irm)
BEFORE any source-compile snippet (git clone / cargo build --release).

Catalog rows:
- structure/install-leads-with-pkg-mgr-docs-index
- structure/install-leads-with-pkg-mgr-readme
"""

from __future__ import annotations

import re
from pathlib import Path

from ._shared import make_artifact, write_artifact

_PKG_MGR_RE = re.compile(
    r"(?:brew install|cargo binstall|curl[^\n]*\| ?sh|powershell[^\n]*irm)",
    re.IGNORECASE | re.MULTILINE,
)
_SOURCE_COMPILE_RE = re.compile(
    r"git clone https?://|cargo build --release",
    re.IGNORECASE | re.MULTILINE,
)


def _verify_install_leads(row: dict, repo_root: Path, target_rel: str) -> int:
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    target = repo_root / target_rel
    if not target.exists():
        asserts_failed.append(f"target file not found: {target_rel}")
    else:
        text = target.read_text(encoding="utf-8")
        pm = _PKG_MGR_RE.search(text)
        src = list(_SOURCE_COMPILE_RE.finditer(text))
        first_src = min((m.start() for m in src), default=None)
        if pm is None:
            asserts_failed.append(
                f"no pkg-mgr command (brew/binstall/curl|sh/powershell-irm) found in {target_rel}"
            )
        elif first_src is not None and pm.start() >= first_src:
            asserts_failed.append(
                f"pkg-mgr command at offset {pm.start()} appears AFTER source-compile snippet at offset {first_src} in {target_rel}"
            )
        else:
            asserts_passed.append(
                f"pkg-mgr command at offset {pm.start()} appears BEFORE source-compile in {target_rel}"
            )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_install_leads_with_pkg_mgr_docs_index(row: dict, repo_root: Path) -> int:
    return _verify_install_leads(row, repo_root, "docs/index.md")


def verify_install_leads_with_pkg_mgr_readme(row: dict, repo_root: Path) -> int:
    return _verify_install_leads(row, repo_root, "README.md")
