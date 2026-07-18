#!/usr/bin/env python3
"""rpx_registry_check.py — RPX error-code registry-integrity checker (P121 W0).

Row: quality/catalogs/agent-ux.json -> agent-ux/rpx-codes-registry
Driver: quality/gates/agent-ux/rpx-codes-registry.sh (leg c). Source-grep, no cargo.

The RPX namespace has ONE source of truth: crates/reposix-core/src/codes.rs (the
`REGISTRY: &[ExplainEntry]` array + the `ids` const module). Four legs:

  (1) FORWARD / M3. Every code EMITTED in crates/reposix-cli/** OR
      crates/reposix-remote/** has a matching ExplainEntry. "Emitted" is caught in
      ALL syntaxes so a helper code outside a bare `.code()` cannot slip past:
      `.code("RPX-xxxx")`, `.code(ids::NAME)`, `teach_coded("RPX-xxxx"/ids::NAME, …)`
      first-arg, bare `[RPX-xxxx]` tag literals, and any `const …_FMT/&str = "…
      RPX-xxxx …"`. `ids::NAME` is resolved through codes.rs.
  (2) INTEGRITY. Every code literal matches ^RPX-\\d{4}$ (RPX-123/RPX-12345 flagged);
      every ExplainEntry code is UNIQUE (no shared code, no repeated ids value); no
      entry leaves title/cause/fix/recovery empty ("" or &[]). The Rust suite
      (tests/explain.rs) is the authoritative non-empty check; this is the backstop.
  (3) CONVERSE / M2 (mechanises SC1). Every user-facing TEACHING site in teach_scan's
      CLI_SCOPE/HELPER_SCOPE (a bail!/anyhow! block routing through teach(/Teach::new/
      teach_coded( or carrying inline Fix:+Recovery:) CARRIES a code — a committed
      invariant, not executor diligence. A legitimately codeless site is marked
      `// rpx-code-exempt: ok — <reason>` (or added to RPX_CODE_ALLOWLIST, each
      justified). Scope is IMPORTED from teach_scan, never re-invented.
  (4) UNKNOWN-CODE UX. The explain-meta code (RPX-0900) must exist so `reposix
      explain <bogus>` has a teaching home (runtime assertion: gate + tests/explain.rs).

CATALOG-FIRST: committed in the phase's FIRST commit, before codes.rs / the
`.code()` sites exist. Over the pre-impl tree it EXITS NON-ZERO (leg 3 flags every
not-yet-coded teaching site) — the honest NOT-VERIFIED state. `--self-test` proves
the checker's OWN logic on inline fixtures and stays GREEN independent of the tree.

RESIDUAL LIMIT (honest, like teach_scan): lexical — it does NOT resolve INDIRECTION
(a code built in `let c = format!(...)` then `.code(c)`, or an unseen `ids` alias).
The convention is inline `.code(...)`/`teach_coded(...)` literals, which it DOES see;
resolving indirection is a filed GOOD-TO-HAVE (T-121-03).
"""
from __future__ import annotations

import os
import re
import sys
from pathlib import Path

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
# Reuse the P120 scope constants + block enumerator — do NOT re-invent scope.
from teach_scan import (  # noqa: E402
    CLI_SCOPE,
    HELPER_SCOPE,
    blank_noncode,
    _balance,
    _MACRO,
    _has_exempt_marker,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
CODES_RS = "crates/reposix-core/src/codes.rs"
EMIT_ROOTS = ["crates/reposix-cli", "crates/reposix-remote"]
EXPLAIN_META_CODE = "RPX-0900"  # the unknown-code teaching path's own code (W2)

# --- ALLOWLIST (leg M2 escape hatch — EMPTY at W0) ---------------------------
# A flagged teaching block whose ORIGINAL text contains any substring here is
# treated as an intentional codeless site. Prefer the inline
# `// rpx-code-exempt: ok — <reason>` marker (line-drift-robust); reserve this set
# for sites a marker cannot reach. Each entry MUST carry a justifying comment.
RPX_CODE_ALLOWLIST: set[str] = set()

_RPX_ANY = re.compile(r"RPX-\d+")
_RPX_OK = re.compile(r"^RPX-\d{4}$")
_RPX4 = re.compile(r"RPX-\d{4}")
_IDS_TAIL = r"(?:reposix_core::)?(?:codes::)?ids::(\w+)"

# Emission syntaxes (leg 1 / M3). Scanned over comment-stripped, string-KEPT text.
_EMIT_CODE_LIT = re.compile(r'\.code\s*\(\s*"(RPX-\d{4})"')
_EMIT_CODE_IDS = re.compile(r"\.code\s*\(\s*" + _IDS_TAIL)
_EMIT_TC_LIT = re.compile(r'\bteach_coded\s*\(\s*"(RPX-\d{4})"')
_EMIT_TC_IDS = re.compile(r"\bteach_coded\s*\(\s*" + _IDS_TAIL)
_EMIT_BRACKET = re.compile(r"\[(RPX-\d{4})\]")
_CONST_STR = re.compile(r'\bconst\s+(\w+)\s*:\s*&(?:\'static\s+)?str\s*=\s*"([^"]*)"')

# Registry parse (leg 2). Over comment-stripped codes.rs text.
_IDS_CONST = re.compile(r'\bconst\s+(\w+)\s*:\s*&(?:\'static\s+)?str\s*=\s*"(RPX-\d{4})"')
_ENTRY_CODE_LIT = re.compile(r'\bcode\s*:\s*"(RPX-\d{4})"')
_ENTRY_CODE_IDS = re.compile(r"\bcode\s*:\s*" + _IDS_TAIL)
_EMPTY_FIELD = re.compile(r'\b(title|cause|fix|recovery)\s*:\s*(?:""|&\s*\[\s*\])')


def _lineno(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def strip_comments(text: str) -> str:
    """Blank `//` and `/* */` comments to spaces (newlines preserved) while
    KEEPING string literals intact — so `.code("RPX-0001")` survives but a
    `// see RPX-9999` comment cannot masquerade as an emission. String-aware
    (a `//` inside a `"..."` or `r"..."` is not a comment)."""
    out: list[str] = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == "r" and i + 1 < n and text[i + 1] in '"#':
            p = i - 1
            prev = text[p] if p >= 0 else ""
            if not (prev.isalnum() or prev == "_"):
                k, h = i + 1, 0
                while k < n and text[k] == "#":
                    h += 1
                    k += 1
                if k < n and text[k] == '"':
                    term = '"' + "#" * h
                    end = text.find(term, k + 1)
                    end = n if end == -1 else end + len(term)
                    out.append(text[i:end])
                    i = end
                    continue
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
            out.append(text[i:j])
            i = j
            continue
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = i
            while j < n and text[j] != "\n":
                j += 1
            out.append(" " * (j - i))
            i = j
            continue
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
            out.append("".join("\n" if ch == "\n" else " " for ch in text[i:j]))
            i = j
            continue
        out.append(c)
        i += 1
    return "".join(out)


def _has_token_marker(all_lines: list[str], start_line_idx: int, token: str) -> bool:
    """True if `token` sits on the block's own start line or within 2 comment/
    blank lines immediately above it (same window shape as teach_scan)."""
    if 0 <= start_line_idx < len(all_lines) and token in all_lines[start_line_idx]:
        return True
    j, budget = start_line_idx - 1, 2
    while j >= 0 and budget > 0:
        ln = all_lines[j]
        if token in ln:
            return True
        s = ln.strip()
        if s == "" or s.startswith("//"):
            j -= 1
            budget -= 1
            continue
        break
    return False


def parse_registry(text: str) -> tuple[set[str], dict[str, str], list[str]]:
    """Return (entry_codes, ids_map, integrity_errors) from codes.rs text."""
    code = strip_comments(text)
    ids_map: dict[str, str] = {}
    ids_dups: list[str] = []
    for m in _IDS_CONST.finditer(code):
        name, val = m.group(1), m.group(2)
        if name in ids_map and ids_map[name] != val:
            ids_dups.append(name)
        ids_map[name] = val
    entry_codes: set[str] = set()
    seen: list[str] = []
    for m in _ENTRY_CODE_LIT.finditer(code):
        seen.append(m.group(1))
    for m in _ENTRY_CODE_IDS.finditer(code):
        name = m.group(1)
        if name in ids_map:
            seen.append(ids_map[name])
    errors: list[str] = []
    # uniqueness across ExplainEntry code fields.
    dups = sorted({c for c in seen if seen.count(c) > 1})
    for c in dups:
        errors.append(f"{CODES_RS}: code {c} defined in more than one ExplainEntry (must be unique)")
    entry_codes = set(seen)
    # ids const value collisions (two consts holding the same code).
    val_owners: dict[str, list[str]] = {}
    for name, val in ids_map.items():
        val_owners.setdefault(val, []).append(name)
    for val, owners in sorted(val_owners.items()):
        if len(owners) > 1:
            errors.append(f"{CODES_RS}: code {val} bound to multiple ids consts {sorted(owners)}")
    # format: any RPX-<digits> not exactly 4 digits.
    for m in _RPX_ANY.finditer(code):
        tok = m.group(0)
        if not _RPX_OK.match(tok):
            errors.append(
                f"{CODES_RS}:{_lineno(code, m.start())}: malformed code {tok!r} "
                f"— codes are 4-digit zero-padded (RPX-\\d{{4}})"
            )
    # non-empty fields.
    for m in _EMPTY_FIELD.finditer(code):
        errors.append(
            f"{CODES_RS}:{_lineno(code, m.start())}: ExplainEntry field {m.group(1)!r} "
            f"is empty — every code must teach a non-empty cause/fix/recovery"
        )
    return entry_codes, ids_map, errors


def extract_emitted(text: str, ids_map: dict[str, str]) -> list[tuple[str, int]]:
    """All emitted codes (code, lineno) across the M3 syntaxes in one file."""
    code = strip_comments(text)
    out: list[tuple[str, int]] = []

    def add(c: str, pos: int) -> None:
        out.append((c, _lineno(code, pos)))

    for m in _EMIT_CODE_LIT.finditer(code):
        add(m.group(1), m.start())
    for m in _EMIT_TC_LIT.finditer(code):
        add(m.group(1), m.start())
    for m in _EMIT_BRACKET.finditer(code):
        add(m.group(1), m.start())
    for rx in (_EMIT_CODE_IDS, _EMIT_TC_IDS):
        for m in rx.finditer(code):
            name = m.group(1)
            if name in ids_map:
                add(ids_map[name], m.start())
            # unknown ids:: name is caught by leg 2 (its const, if any, is parsed)
    for m in _CONST_STR.finditer(code):
        name, val = m.group(1), m.group(2)
        if name.endswith("_FMT") or _RPX4.search(val):
            for cm in _RPX4.finditer(val):
                add(cm.group(0), m.start())
    return out


def scan_missing_codes(text: str, rel: str) -> list[str]:
    """Leg M2: every user-facing teaching block in `text` must carry a code."""
    blanked = blank_noncode(text)
    all_lines = text.split("\n")
    msgs: list[str] = []
    covered = 0
    for m in _MACRO.finditer(blanked):
        if m.start() < covered:
            continue
        end = _balance(blanked, m.end() - 1)
        covered = end
        block = text[m.start() : end]
        start_idx = text.count("\n", 0, m.start())
        # non-user-facing (teach-exempt) blocks never need a code.
        if _has_exempt_marker(all_lines, start_idx):
            continue
        # explicit codeless disposition.
        if _has_token_marker(all_lines, start_idx, "rpx-code-exempt: ok"):
            continue
        if any(sig in block for sig in RPX_CODE_ALLOWLIST):
            continue
        is_teaching = (
            "teach(" in block
            or "Teach::new" in block
            or "teach_coded(" in block
            or ("Fix:" in block and "Recovery:" in block)
        )
        if not is_teaching:
            continue
        has_code = bool(
            _RPX4.search(block)
            or ".code(" in block
            or "teach_coded(" in block
            or re.search(r"\bids::\w+", block)
        )
        if not has_code:
            snippet = block.splitlines()[0].strip()[:80] if block.strip() else "<empty>"
            msgs.append(
                f"{rel}:{start_idx + 1}: user-facing teaching error carries no RPX code "
                f"— attach one with `.code(ids::…)` / `teach_coded(ids::…, …)`, or mark "
                f"`// rpx-code-exempt: ok — <reason>` if it intentionally has none  [{snippet}]"
            )
    return msgs


def run(repo_root: Path) -> int:
    errors: list[str] = []
    notes: list[str] = []

    codes_path = repo_root / CODES_RS
    if codes_path.is_file():
        entry_codes, ids_map, integ = parse_registry(codes_path.read_text(encoding="utf-8"))
        errors.extend(integ)
    else:
        entry_codes, ids_map = set(), {}
        notes.append(f"{CODES_RS} not present yet (pre-W1) — registry treated as empty")

    # leg 1 (forward) across both emitting crates.
    for root in EMIT_ROOTS:
        base = repo_root / root
        if not base.is_dir():
            continue
        for p in sorted(base.rglob("*.rs")):
            rel = p.relative_to(repo_root).as_posix()
            for c, ln in extract_emitted(p.read_text(encoding="utf-8"), ids_map):
                if c not in entry_codes:
                    errors.append(
                        f"{rel}:{ln}: {c} referenced but not in REGISTRY "
                        f"— add an ExplainEntry to {CODES_RS}"
                    )

    # leg 3 (converse / M2) over the teach_scan scope.
    for rel in CLI_SCOPE + HELPER_SCOPE:
        p = repo_root / rel
        if not p.is_file():
            errors.append(f"scope file missing: {rel}")
            continue
        errors.extend(scan_missing_codes(p.read_text(encoding="utf-8"), rel))

    # leg 4 (unknown-code UX has a home): the explain-meta code must exist.
    if EXPLAIN_META_CODE not in entry_codes:
        errors.append(
            f"{CODES_RS}: explain-meta code {EXPLAIN_META_CODE} missing "
            f"— `reposix explain <unknown>` needs it to teach `reposix explain --list` (W2)"
        )

    for n in notes:
        print(f"rpx_registry_check: note: {n}")
    if errors:
        print(
            f"rpx_registry_check: FAIL — {len(errors)} registry-integrity issue(s):",
            file=sys.stderr,
        )
        for e in errors:
            print("  " + e, file=sys.stderr)
        print(
            "\nFix: every emitted RPX code needs an ExplainEntry in "
            f"{CODES_RS}, every entry a non-empty cause/fix/recovery + a unique "
            "4-digit code, and every user-facing teaching error a `.code(...)` "
            "(or a `// rpx-code-exempt: ok — <reason>` marker).",
            file=sys.stderr,
        )
        return 1
    print(
        f"rpx_registry_check: clean — {len(entry_codes)} registered code(s); "
        f"all emitted codes registered; all teaching sites coded."
    )
    return 0


# --- Self-test: inline fixtures proving the checker's OWN logic ----------------
def self_test() -> int:
    failures = 0

    def check(name: str, got, want) -> None:
        nonlocal failures
        ok = got == want
        print(f"  [{'ok' if ok else 'FAIL'}] {name}: expected {want}, got {got}")
        if not ok:
            failures += 1

    reg = (
        'pub mod ids { pub const A: &str = "RPX-0001"; pub const B: &str = "RPX-0900"; }\n'
        'const R: &[ExplainEntry] = &[\n'
        '  ExplainEntry { code: ids::A, title: "t", cause: "c", fix: "f", recovery: &["r"] },\n'
        '  ExplainEntry { code: "RPX-0900", title: "t", cause: "c", fix: "f", recovery: &["r"] },\n'
        '];\n'
    )
    entry_codes, ids_map, integ = parse_registry(reg)
    check("registry parses two entry codes", entry_codes == {"RPX-0001", "RPX-0900"}, True)
    check("clean registry has no integrity errors", len(integ), 0)

    # forward: registered literal PASSES, unregistered RAISES.
    check(
        "emitted+registered .code literal resolves",
        [c for c, _ in extract_emitted('e.code("RPX-0001")', ids_map) if c not in entry_codes],
        [],
    )
    check(
        "emitted-but-unregistered .code literal is caught",
        [c for c, _ in extract_emitted('e.code("RPX-0002")', ids_map) if c not in entry_codes],
        ["RPX-0002"],
    )

    # M3: all emission syntaxes are seen.
    m3 = (
        'x.code(ids::A);\n'
        'teach_coded("RPX-0003", h, f, a, r);\n'
        'assert!(s.contains("[RPX-0004]"));\n'
        'const E_FMT: &str = "boom RPX-0005 boom";\n'
    )
    got_m3 = sorted({c for c, _ in extract_emitted(m3, ids_map)})
    check("M3 sees code/teach_coded/bracket/FMT + ids resolution",
          got_m3, ["RPX-0001", "RPX-0003", "RPX-0004", "RPX-0005"])

    # duplicate + malformed + empty-field integrity.
    dup = (
        'const R: &[ExplainEntry] = &[\n'
        '  ExplainEntry { code: "RPX-0001", title: "t", cause: "c", fix: "f", recovery: &["r"] },\n'
        '  ExplainEntry { code: "RPX-0001", title: "t", cause: "c", fix: "f", recovery: &["r"] },\n'
        '];\n'
    )
    _, _, dup_err = parse_registry(dup)
    check("duplicate ExplainEntry code is flagged", any("more than one" in e for e in dup_err), True)
    _, _, bad_fmt = parse_registry('let c = "RPX-123";\n')
    check("malformed RPX-123 is flagged", any("malformed" in e for e in bad_fmt), True)
    _, _, empty_err = parse_registry(
        'ExplainEntry { code: "RPX-0007", title: "t", cause: "", fix: "f", recovery: &["r"] }'
    )
    check("empty cause field is flagged", any("is empty" in e for e in empty_err), True)

    # M2 converse: coded PASSES, codeless RAISES, marked PASSES.
    coded = 'fn a() { bail!("{}", teach_coded(ids::A, "h", "f", "a", &["r"])) }'
    check("M2: coded teaching block passes", len(scan_missing_codes(coded, "x.rs")), 0)
    codeless = 'fn b() { bail!("{}", teach("h", "f", "a", &["r"])) }'
    check("M2: codeless teaching block is flagged", len(scan_missing_codes(codeless, "x.rs")), 1)
    marked = (
        "fn c() {\n"
        "    // rpx-code-exempt: ok — internal invariant, not user-facing\n"
        '    bail!("{}", teach("h", "f", "a", &["r"]))\n'
        "}"
    )
    check("M2: rpx-code-exempt marker exempts a codeless block",
          len(scan_missing_codes(marked, "x.rs")), 0)
    teach_exempt = (
        "fn d() {\n"
        "    // teach-exempt: ok — surfaces a subprocess error verbatim\n"
        '    bail!("git failed: {e}")\n'
        "}"
    )
    check("M2: non-teaching teach-exempt block needs no code",
          len(scan_missing_codes(teach_exempt, "x.rs")), 0)

    if failures:
        print(f"rpx_registry_check --self-test: {failures} FAILURE(s)", file=sys.stderr)
        return 1
    print("rpx_registry_check --self-test: PASS (forward + M3 + integrity + M2 all proven)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return run(REPO_ROOT)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
