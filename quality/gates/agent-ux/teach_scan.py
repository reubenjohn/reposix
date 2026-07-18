#!/usr/bin/env python3
"""teach_scan.py — multi-line-aware source scanner for the Rust-compiler-grade
3-part error bar (Phase 120 / P120).

WHAT IT ENFORCES
----------------
Every `bail!(...)`, `anyhow!(...)` (and `return Err(anyhow!(...))`, which is just
an `anyhow!(...)` invocation) on the ENUMERATED CLI / helper surface must EITHER
route through the shared teaching builder / a named shared helper, OR carry a
`Fix:`/`Recovery:` teaching body inline, OR be explicitly dispositioned with a
`// teach-exempt: ok — <reason>` marker. A NEW teaching-less error site — SINGLE-
OR MULTI-LINE — added to the scope RAISEs, so the bar cannot silently rot.

The scanner is MULTI-LINE-AWARE by construction: it does not scan single lines.
It blanks every string/char literal and comment (so a `)` inside a string or a
`bail!(` inside a doc comment can never miscount), then walks each real
`bail!(`/`anyhow!(` invocation and accumulates the FULL balanced-paren macro
block — single- or multi-line — before judging it. This closes the "reverse
hole" where a teaching-less bail! split across lines evaded a line-oriented grep.

A block PASSES iff ANY of:
  (i)   it mentions `teach(` / `teach_coded(` / `Teach::new` / one of the named
        shared helpers (spec_parse_error | missing_env_var_error |
        cache_build_error | missing_cache_db_error | malformed_bus_url_error |
        missing_env_error). `teach_coded(` is the P121 coded sibling of `teach(`
        (a `RPX-xxxx` tag + `Explain:` nudge on the same 3-part body); it is a
        first-class teaching call, MIRRORING how `rpx_registry_check.py` treats
        `teach_coded(` as a valid teaching emission;
  (ii)  it contains BOTH literal anchors `Fix:` and `Recovery:` in its own text;
  (iii) it is preceded (its own line, or within 2 comment/blank lines above) by a
        `// teach-exempt: ok` marker.
Otherwise the block RAISEs (exit 1) with a teaching stderr naming file:line — the
gate dogfoods the very bar it enforces.

DOCUMENTED RESIDUAL LIMITATIONS (honest — NOT "un-rottable" in the absolute)
---------------------------------------------------------------------------
The scanner keys on LEXICAL presence within the macro block. It therefore does
NOT catch:
  * INDIRECTION ESCAPES — `bail!(some_var)` / `return Err(e)` where the teaching-
    less string was built elsewhere (a `let msg = format!(...)` above the bail!,
    or a helper the scanner does not know by name). The message's anchors live
    outside the block the scanner reads.
  * `format!`-INTO-`bail!` where the `Fix:`/`Recovery:` anchors are assembled in a
    helper fn not on the named-helper allowlist.
  * OTHER error constructors not in the trigger set — `ensure!(...)`,
    `.ok_or_else(|| Error::…)`, hand-rolled `Err(MyError{..})`. Only `bail!` /
    `anyhow!` (the codebase's dominant user-facing-error idiom) are scanned.
These are LOW-likelihood here: the convention across crates/reposix-{cli,remote}
is inline string literals or the named shared helpers, both of which the scanner
DOES see. The honest claim is: "un-rottable for the direct inline `bail!`/`anyhow!`
pattern over the enumerated scope, single- or multi-line" — exactly what the code
below enforces, no more. Widening the trigger set or resolving indirection is a
filed GOOD-TO-HAVE, not a claim made here.

SCOPE
-----
`--scope cli`    → the enumerated reposix-cli subcommand/helper files.
`--scope helper` → the enumerated reposix-remote git-helper files.
Adding a new subcommand file to the surface = adding it to the constant below (a
reviewable one-line change, never a silent gap).

SELF-TEST
---------
`--self-test` runs 3 inline fixtures proving: a teaching multi-line bail! PASSES,
a teaching-less multi-line bail! RAISEs, and a marked bail! PASSES. The scanner's
own multi-line + marker logic is thereby regression-covered.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

# --- Explicit, documented scope constants (match the P120 retrofit set) -------
# CLI surface: crates/reposix-cli/src/{...}.rs
CLI_SCOPE = [
    "crates/reposix-cli/src/errors.rs",
    "crates/reposix-cli/src/init.rs",
    "crates/reposix-cli/src/attach.rs",
    "crates/reposix-cli/src/list.rs",
    "crates/reposix-cli/src/refresh.rs",
    "crates/reposix-cli/src/spaces.rs",
    "crates/reposix-cli/src/sync.rs",
    "crates/reposix-cli/src/gc.rs",
    "crates/reposix-cli/src/history.rs",
    "crates/reposix-cli/src/tokens.rs",
    "crates/reposix-cli/src/cost.rs",
    "crates/reposix-cli/src/worktree_helpers.rs",
    "crates/reposix-cli/src/cache_db.rs",
    "crates/reposix-cli/src/main.rs",
]
# Helper surface: crates/reposix-remote/src/{...}.rs
HELPER_SCOPE = [
    "crates/reposix-remote/src/main.rs",
    "crates/reposix-remote/src/bus_url.rs",
    "crates/reposix-remote/src/backend_dispatch.rs",
    "crates/reposix-remote/src/stateless_connect.rs",
    "crates/reposix-remote/src/write_loop.rs",
    "crates/reposix-remote/src/bus_handler.rs",
    "crates/reposix-remote/src/precheck.rs",
]

# A block PASSES if it routes through the builder or a named shared helper.
# `teach(` and its P121 coded sibling `teach_coded(` are BOTH recognized via the
# optional `(?:_coded)?` group — mirroring `rpx_registry_check.py`'s
# `\bteach_coded\s*\(` matcher so the two sibling gates agree that a `teach_coded(`
# site is a teaching call (P121 wired ~30 CLI+helper error paths onto `teach_coded`;
# without this the scanner reads every one as teaching-LESS — the P122-W2-01 gap).
_PASS_CALL = re.compile(
    r"\bteach(?:_coded)?\s*\(|Teach::new|"
    r"\b(?:spec_parse_error|missing_env_var_error|cache_build_error|"
    r"missing_cache_db_error|malformed_bus_url_error|missing_env_error)\b"
)
# Macro-invocation start on the enumerated trigger set. Matched over the
# BLANKED (code-only) text so a `bail!(` inside a string/comment never triggers.
# Negative lookbehind excludes `mybail!` / identifier-suffixed matches.
_MACRO = re.compile(r"(?<![A-Za-z0-9_])(?:bail|anyhow)\s*!\s*\(")

_EXEMPT = "teach-exempt: ok"


def _blank(span: str) -> str:
    """Blank a consumed span to spaces, preserving newlines (line-number safe)."""
    return "".join("\n" if ch == "\n" else " " for ch in span)


def blank_noncode(text: str) -> str:
    """Return `text` with every string literal, char literal, line comment and
    (nesting) block comment blanked to spaces — newlines preserved. Rust
    lifetimes (`'a`) are correctly NOT treated as char literals; raw strings
    (`r"..."`, `r#"..."#`) are handled. Structural parens in code survive; a
    `)` inside a blanked string/char/comment does not, so paren balancing over
    the result is sound."""
    out: list[str] = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        # line comment // ... \n
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = i
            while j < n and text[j] != "\n":
                j += 1
            out.append(_blank(text[i:j]))
            i = j
            continue
        # nesting block comment /* ... */
        if c == "/" and i + 1 < n and text[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth > 0:
                if text[j] == "/" and j + 1 < n and text[j + 1] == "*":
                    depth += 1
                    j += 2
                    continue
                if text[j] == "*" and j + 1 < n and text[j + 1] == "/":
                    depth -= 1
                    j += 2
                    continue
                j += 1
            out.append(_blank(text[i:j]))
            i = j
            continue
        # raw string r"..." / r#"..."# (only at a token boundary, not ident tail)
        if c == "r":
            p = i - 1
            while p >= 0 and text[p] in " \t":
                p -= 1
            prevc = text[p] if p >= 0 else ""
            if not (prevc.isalnum() or prevc == "_"):
                k = i + 1
                hashes = 0
                while k < n and text[k] == "#":
                    hashes += 1
                    k += 1
                if k < n and text[k] == '"':
                    term = '"' + "#" * hashes
                    end = text.find(term, k + 1)
                    end = n if end == -1 else end + len(term)
                    out.append(_blank(text[i:end]))
                    i = end
                    continue
        # normal string "..."
        if c == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(_blank(text[i:j]))
            i = j
            continue
        # char literal '...' (vs lifetime 'a — NOT a char literal)
        if c == "'":
            if i + 1 < n and text[i + 1] == "\\":  # escaped char '\n' '\'' '\u{..}'
                j = i + 2
                if j < n and text[j] == "u" and j + 1 < n and text[j + 1] == "{":
                    while j < n and text[j] != "}":
                        j += 1
                    j += 1  # past }
                else:
                    j += 1  # single escape body char
                if j < n and text[j] == "'":
                    j += 1
                out.append(_blank(text[i:j]))
                i = j
                continue
            if i + 2 < n and text[i + 2] == "'":  # simple char 'x' / ')' / '('
                out.append(_blank(text[i : i + 3]))
                i += 3
                continue
            # otherwise a lifetime/label — emit the tick as ordinary punctuation
            out.append("'")
            i += 1
            continue
        out.append(c)
        i += 1
    return "".join(out)


def _balance(code: str, open_paren: int) -> int:
    """From the `(` at `open_paren`, return the index just past its matching
    `)`, balancing over the blanked (code-only) text."""
    depth, j, n = 0, open_paren, len(code)
    while j < n:
        ch = code[j]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return j + 1
        j += 1
    return n  # unbalanced — treat as running to EOF (defensive)


def _has_exempt_marker(all_lines: list[str], start_line_idx: int) -> bool:
    """True if a `// teach-exempt: ok` marker sits on the block's own start line
    or within 2 comment/blank lines immediately above it."""
    if 0 <= start_line_idx < len(all_lines) and _EXEMPT in all_lines[start_line_idx]:
        return True
    j, budget = start_line_idx - 1, 2
    while j >= 0 and budget > 0:
        ln = all_lines[j]
        if _EXEMPT in ln:
            return True
        s = ln.strip()
        if s == "" or s.startswith("//"):
            j -= 1
            budget -= 1
            continue
        break
    return False


def scan_text(text: str) -> list[tuple[int, str]]:
    """Return a list of (line_no, first_line_snippet) for every un-dispositioned
    `bail!`/`anyhow!` block in `text`."""
    code = blank_noncode(text)
    all_lines = text.split("\n")
    raises: list[tuple[int, str]] = []
    covered = 0  # skip macro starts nested inside an already-captured block
    for m in _MACRO.finditer(code):
        if m.start() < covered:
            continue
        end = _balance(code, m.end() - 1)
        covered = end
        block = text[m.start() : end]
        # (i) shared builder / named helper
        if _PASS_CALL.search(block):
            continue
        # (ii) inline 3-part anchors
        if "Fix:" in block and "Recovery:" in block:
            continue
        # (iii) explicit exempt marker
        start_line_idx = text.count("\n", 0, m.start())
        if _has_exempt_marker(all_lines, start_line_idx):
            continue
        snippet = block.splitlines()[0].strip()[:80] if block.strip() else "<empty>"
        raises.append((start_line_idx + 1, snippet))
    return raises


def scan_file(path: Path) -> list[str]:
    """Scan one file; return teaching RAISE messages (empty == clean)."""
    text = path.read_text(encoding="utf-8", errors="replace")
    msgs: list[str] = []
    for line_no, snippet in scan_text(text):
        msgs.append(
            f"{path}:{line_no}: un-dispositioned error block — route through "
            f"teach()/a shared helper or add '// teach-exempt: ok — <reason>'  "
            f"[{snippet}]"
        )
    return msgs


def run_scope(scope: str, repo_root: Path) -> int:
    files = {"cli": CLI_SCOPE, "helper": HELPER_SCOPE}.get(scope)
    if files is None:
        print(f"teach_scan.py: unknown --scope {scope!r} (expected cli|helper)",
              file=sys.stderr)
        return 2
    all_msgs: list[str] = []
    for rel in files:
        p = repo_root / rel
        if not p.is_file():
            print(f"teach_scan.py: scope file missing: {rel}", file=sys.stderr)
            return 2
        all_msgs.extend(scan_file(p))
    if all_msgs:
        print(
            f"teach_scan.py --scope {scope}: "
            f"{len(all_msgs)} un-dispositioned error block(s):",
            file=sys.stderr,
        )
        for msg in all_msgs:
            print("  " + msg, file=sys.stderr)
        print(
            "\nFix: route each block through reposix_core::errmsg::teach(...) or a "
            "named shared helper (spec_parse_error / missing_env_var_error / "
            "cache_build_error / missing_cache_db_error / malformed_bus_url_error / "
            "missing_env_error), OR — if the site is genuinely not user-facing "
            "teaching — add a `// teach-exempt: ok — <reason>` marker directly above it.",
            file=sys.stderr,
        )
        return 1
    print(f"teach_scan.py --scope {scope}: clean "
          f"({len(files)} files, no un-dispositioned bail!/anyhow! block).")
    return 0


# --- Self-test: 3 inline fixtures exercising the multi-line + marker logic -----
_FIX_TEACHING = '''
fn a() -> anyhow::Result<()> {
    bail!(
        "{}",
        reposix_core::errmsg::teach(
            "headline",
            "do the thing",
            "or try the other thing",
            &["copy-paste recovery"],
        )
    )
}
'''

_FIX_TEACHINGLESS = '''
fn b() -> anyhow::Result<()> {
    // a multi-line teaching-less bail! — the reverse hole; MUST raise
    bail!(
        "spec `{spec}` is malformed (missing `::`) \\
         and cannot be parsed",
        spec = spec,
    )
}
'''

_FIX_MARKED = '''
fn c() -> anyhow::Result<()> {
    // teach-exempt: ok — internal invariant, not user-facing
    bail!(
        "internal: oid map desynced at {n}",
        n = n,
    )
}
'''

# P122-W2-01 regression: a `teach_coded(...)` site is a teaching call and PASSES,
# exactly like the `teach(...)` sibling — the scanner must not read the P121 coded
# idiom as teaching-LESS.
_FIX_TEACH_CODED = '''
fn e() -> anyhow::Result<()> {
    bail!(
        "{}",
        reposix_core::errmsg::teach_coded(
            ids::INIT_NESTED_IN_REPO,
            "headline",
            "do the thing",
            "or try the other thing",
            &["copy-paste recovery"],
        )
    )
}
'''


def self_test() -> int:
    cases = [
        ("teaching multi-line bail! PASSES", _FIX_TEACHING, 0),
        ("teaching-less multi-line bail! RAISES", _FIX_TEACHINGLESS, 1),
        ("marked bail! PASSES", _FIX_MARKED, 0),
        ("teach_coded(...) bail! PASSES (P122-W2-01)", _FIX_TEACH_CODED, 0),
    ]
    failures = 0
    for name, fixture, expected_raises in cases:
        got = len(scan_text(fixture))
        ok = got == expected_raises
        print(f"  [{'ok' if ok else 'FAIL'}] {name}: "
              f"expected {expected_raises} raise(s), got {got}")
        if not ok:
            failures += 1
    # Bonus: a `)` inside a string literal in a teaching-less block must NOT
    # break paren balancing (still exactly one raise, not zero-or-many).
    tricky = 'fn d() { bail!("oops a paren ) inside a string and a char \')\'"); }'
    got_tricky = len(scan_text(tricky))
    ok_tricky = got_tricky == 1
    print(f"  [{'ok' if ok_tricky else 'FAIL'}] string/char-paren balance is sound: "
          f"expected 1 raise, got {got_tricky}")
    if not ok_tricky:
        failures += 1
    if failures:
        print(f"teach_scan.py --self-test: {failures} FAILURE(s)", file=sys.stderr)
        return 1
    print("teach_scan.py --self-test: PASS (multi-line detection + marker "
          "exemption + paren-balance all proven)")
    return 0


def main(argv: list[str]) -> int:
    repo_root = Path(__file__).resolve().parents[3]
    if "--self-test" in argv:
        return self_test()
    if "--scope" in argv:
        idx = argv.index("--scope")
        if idx + 1 >= len(argv):
            print("teach_scan.py: --scope requires an argument (cli|helper)",
                  file=sys.stderr)
            return 2
        return run_scope(argv[idx + 1], repo_root)
    print(__doc__)
    print("usage: teach_scan.py --scope {cli|helper} | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
