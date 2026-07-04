#!/usr/bin/env python3
"""quality/gates/structure/cli-subcommand-parity.py -- structure/cli-subcommand-parity verifier.

Parses the clap `enum Cmd` in crates/reposix-cli/src/main.rs and checks:

  1. Every variant EXCEPT `Version` (which has no dedicated cli.md prose
     section by long-standing convention -- it's a one-liner already
     covered in the top-of-page command listing) has a matching
     `## `reposix <name>`` section in docs/reference/cli.md.
  2. Every variant (INCLUDING Version) has a row in the per-subcommand
     table in docs/reference/exit-codes.md.

No cargo invocation -- pure-Python regex/text checks, <1s.

Implements catalog row structure/cli-subcommand-parity.
"""
from __future__ import annotations

import datetime
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
MAIN_RS = REPO_ROOT / "crates/reposix-cli/src/main.rs"
CLI_MD = REPO_ROOT / "docs/reference/cli.md"
EXIT_CODES_MD = REPO_ROOT / "docs/reference/exit-codes.md"
ARTIFACT = REPO_ROOT / "quality/reports/verifications/structure/cli-subcommand-parity.json"

# Variants with no dedicated `## reposix <name>` prose section in cli.md.
EXEMPT_FROM_CLI_SECTION = {"Version"}


def camel_to_kebab(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower()


def find_variants(src: str) -> list[str]:
    m = re.search(r"enum Cmd \{(.*?)\n\}", src, re.DOTALL)
    if not m:
        raise SystemExit("FAIL: could not locate `enum Cmd { ... }` in main.rs -- regex drifted")
    body = m.group(1)
    # Top-level variants are indented exactly 4 spaces, start with an
    # uppercase identifier, and are immediately followed by `{` (struct
    # variant), `(` (tuple variant), or `,` (unit variant).
    return re.findall(r"^\s{4}([A-Z][A-Za-z0-9]*)\s*[({,]", body, re.MULTILINE)


def main() -> int:
    main_src = MAIN_RS.read_text(encoding="utf-8")
    cli_md = CLI_MD.read_text(encoding="utf-8")
    exit_codes_md = EXIT_CODES_MD.read_text(encoding="utf-8")

    variants = find_variants(main_src)
    if not variants:
        raise SystemExit("FAIL: zero Cmd variants parsed from main.rs -- regex drifted from enum shape")

    missing_cli_section: list[str] = []
    missing_exit_row: list[str] = []
    for v in variants:
        slug = camel_to_kebab(v)
        if v not in EXEMPT_FROM_CLI_SECTION:
            heading = re.compile(rf"^## `reposix {re.escape(slug)}(\s|`)", re.MULTILINE)
            if not heading.search(cli_md):
                missing_cli_section.append(v)
        row_pattern = re.compile(rf"`{re.escape(slug)}`")
        if not row_pattern.search(exit_codes_md):
            missing_exit_row.append(v)

    ok = not missing_cli_section and not missing_exit_row
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    non_exempt_count = len(variants) - len(EXEMPT_FROM_CLI_SECTION & set(variants))
    if missing_cli_section:
        asserts_failed.append(f"docs/reference/cli.md missing section(s) for: {missing_cli_section}")
    else:
        asserts_passed.append(f"all {non_exempt_count} non-exempt Cmd variants have a cli.md section")
    if missing_exit_row:
        asserts_failed.append(f"docs/reference/exit-codes.md missing row(s) for: {missing_exit_row}")
    else:
        asserts_passed.append(f"all {len(variants)} Cmd variants have an exit-codes.md row")

    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    ARTIFACT.write_text(
        json.dumps(
            {
                "ts": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                "row_id": "structure/cli-subcommand-parity",
                "exit_code": 0 if ok else 1,
                "variants_checked": variants,
                "asserts_passed": asserts_passed,
                "asserts_failed": asserts_failed,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    if not ok:
        print("FAIL: cli-subcommand-parity", file=sys.stderr)
        for m in missing_cli_section:
            print(f"  missing cli.md section: ## `reposix {camel_to_kebab(m)}`", file=sys.stderr)
        for m in missing_exit_row:
            print(f"  missing exit-codes.md row for: {camel_to_kebab(m)}", file=sys.stderr)
        return 1
    print(f"PASS: cli-subcommand-parity ({len(variants)} variants checked)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
