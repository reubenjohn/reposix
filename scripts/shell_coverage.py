#!/usr/bin/env python3
"""kcov-based aggregate line-coverage for reposix shell scripts.

Every in-scope shell script counts in the denominator — never-run scripts sit
at 0%; we mandate an *aggregate* average above a ratchet floor, not per-file
bars. No whitelisting: the corpus rule below is the single source of truth.

Subcommands:
  run        Drive every harness under quality/gates/code/shell-coverage-tests/
             through kcov, merge, grade aggregate vs the floor, write a JSON
             artifact. Exit 1 if below floor.
  enumerate  Print the in-scope corpus (one path per line) for debugging.

Pure Python 3 stdlib — parses kcov's coverage.json with the json module.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

# ── Corpus rule ──
# In scope: any shell script (extension .sh/.bash OR bash/sh shebang) EXCEPT
# anything under an excluded root. The harness/test dir is excluded so
# harnesses never grade their own coverage; the pre-push test harness is a test.
EXCLUDED_DIR_PARTS = (".planning", "target", ".git", "node_modules")
EXCLUDED_REL_PREFIXES = (
    ".claude/worktrees/",
    "quality/gates/code/shell-coverage-tests/",
)
EXCLUDED_REL_EXACT = (
    ".githooks/test-pre-push.sh",
    "scripts/shell_coverage.py",  # this driver is Python, not a target
)

SHEBANG_SHELL_RE = re.compile(r"^#!.*\b(ba)?sh\b")
# A line kcov would never mark coverable: blank, pure comment, shebang, or a
# structural-only token line (block delimiters / keywords with no executable
# expression on them).
STRUCTURAL_ONLY_RE = re.compile(
    r"^\s*(fi|done|else|esac|do|then|\{|\}|\(|\)|;;|;;&|;&|in)\s*$"
)
# A `case` label line, e.g. `*)` / `foo)` / `a|b)` — pattern + `)` with no `(`
# (which would be a function def / command sub). kcov treats these as structural.
CASE_LABEL_RE = re.compile(r"^[^()\s][^()]*\)$")


def repo_root() -> Path:
    out = subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True
    ).strip()
    return Path(out)


def is_shell_script(path: Path) -> bool:
    if path.suffix in (".sh", ".bash"):
        return True
    try:
        with path.open("r", errors="replace") as fh:
            first = fh.readline()
    except (OSError, UnicodeDecodeError):
        return False
    return bool(SHEBANG_SHELL_RE.match(first))


def enumerate_corpus(root: Path) -> list[Path]:
    """Sorted absolute paths of in-scope shell scripts (tracked files only)."""
    tracked = subprocess.check_output(
        ["git", "ls-files"], cwd=root, text=True
    ).splitlines()
    out: list[Path] = []
    for rel in tracked:
        if any(part in EXCLUDED_DIR_PARTS for part in Path(rel).parts):
            continue
        if any(rel.startswith(p) for p in EXCLUDED_REL_PREFIXES):
            continue
        if rel in EXCLUDED_REL_EXACT:
            continue
        abs_path = root / rel
        if not abs_path.is_file():
            continue
        if is_shell_script(abs_path):
            out.append(abs_path)
    return sorted(out)


def coverable_line_count(path: Path) -> int:
    """Bash-aware count of lines kcov would treat as coverable.

    Excludes blank, pure-comment, shebang, structural-only, case-label,
    backslash-continuation, and heredoc-body lines. Tuned so
    |counter - kcov_total| stays within 15% on executed scripts (the
    anti-gaming validation) — a fair stand-in for the total_lines of the
    scripts kcov never executes.
    """
    n = 0
    try:
        lines = path.read_text(errors="replace").splitlines()
    except OSError:
        return 0
    in_heredoc = False
    heredoc_term = ""
    prev_continued = False  # previous physical line ended a statement with `\`
    for idx, raw in enumerate(lines):
        stripped = raw.strip()
        continues = raw.rstrip().endswith("\\")
        # Heredoc bodies are data — kcov does not count them (<<EOF..EOF window).
        if in_heredoc:
            if stripped.lstrip("\t") == heredoc_term:
                in_heredoc = False
            continue
        # A backslash-continuation line folds into the previous statement, which
        # kcov attributes to its first line — so it is not separately coverable.
        if prev_continued:
            prev_continued = continues
            continue
        if not stripped:
            continue
        if idx == 0 and stripped.startswith("#!"):
            continue
        if stripped.startswith("#"):  # a `\` in a comment does not continue it
            continue
        m = re.search(r"<<-?\s*[\"']?([A-Za-z_][A-Za-z0-9_]*)[\"']?", raw)
        if m and "<<<" not in raw:  # heredoc intro line IS executable
            in_heredoc = True
            heredoc_term = m.group(1)
        if STRUCTURAL_ONLY_RE.match(stripped) or CASE_LABEL_RE.match(stripped):
            continue
        n += 1
        prev_continued = continues
    return n


# ── kcov drive + parse ──
@dataclass
class FileCov:
    path: str  # repo-relative
    covered: int
    total: int
    seen: bool  # True if kcov executed it

    @property
    def pct(self) -> float:
        return 100.0 * self.covered / self.total if self.total else 0.0


@dataclass
class Result:
    aggregate_pct: float
    floor: float
    covered_total: int
    line_total: int
    files: list[FileCov]
    asserts_passed: list[str] = field(default_factory=list)
    asserts_failed: list[str] = field(default_factory=list)
    validation_warnings: list[str] = field(default_factory=list)


def run_harnesses(root: Path, workdir: Path) -> dict[str, tuple[int, int]]:
    """Run each harness under kcov, merge, return {rel: (covered, total)}."""
    harness_dir = root / "quality/gates/code/shell-coverage-tests"
    harnesses = sorted(harness_dir.glob("*.sh")) if harness_dir.is_dir() else []
    if not harnesses:
        return {}

    run_dirs: list[Path] = []
    for i, h in enumerate(harnesses):
        out_dir = workdir / f"run{i}"
        cmd = [
            "kcov",
            "--include-path=" + str(root),
            "--exclude-path=" + str(harness_dir),
            "--exclude-path=" + str(root / ".git"),
            "--exclude-path=" + str(root / "target"),
            "--exclude-path=" + str(root / ".planning"),
            "--exclude-path=" + str(root / "node_modules"),
            str(out_dir),
            str(h),
        ]
        env = dict(os.environ)
        env.setdefault("REPOSIX_SHELL_COVERAGE", "1")  # hermetic-intent marker
        proc = subprocess.run(
            cmd, cwd=str(root), env=env, capture_output=True, text=True
        )
        if proc.returncode != 0:
            sys.stderr.write(
                f"WARN: harness {h.name} exited {proc.returncode} under kcov "
                f"(coverage still collected)\n"
            )
            tail = (proc.stderr or "").strip().splitlines()[-5:]
            for ln in tail:
                sys.stderr.write(f"    {ln}\n")
        if out_dir.is_dir():
            run_dirs.append(out_dir)

    if not run_dirs:
        return {}

    merged = workdir / "merged"
    subprocess.run(
        ["kcov", "--merge", str(merged)] + [str(d) for d in run_dirs],
        capture_output=True,
        text=True,
    )
    cov_json = merged / "kcov-merged" / "coverage.json"
    if not cov_json.is_file():
        # Single-run fallback: kcov puts coverage.json in the run dir itself.
        if len(run_dirs) == 1:
            cov_json = _find_coverage_json(run_dirs[0])
        if not cov_json or not cov_json.is_file():
            return {}

    return _parse_coverage_json(cov_json, root)


def _find_coverage_json(run_dir: Path) -> Path | None:
    direct = run_dir / "coverage.json"
    if direct.is_file():
        return direct
    hits = list(run_dir.glob("**/coverage.json"))
    return hits[0] if hits else None


def _parse_coverage_json(cov_json: Path, root: Path) -> dict[str, tuple[int, int]]:
    data = json.loads(cov_json.read_text())
    out: dict[str, tuple[int, int]] = {}
    for entry in data.get("files", []):
        fpath = Path(entry["file"])
        try:
            rel = str(fpath.resolve().relative_to(root.resolve()))
        except ValueError:
            continue
        covered = int(entry.get("covered_lines", 0))
        total = int(entry.get("total_lines", 0))
        out[rel] = (covered, total)
    return out


def compute(
    root: Path,
    floor: float,
    keep_workdir: bool = False,
    cobertura_out: Path | None = None,
) -> Result:
    corpus = enumerate_corpus(root)
    corpus_rel = {str(p.relative_to(root)) for p in corpus}

    workdir = Path(tempfile.mkdtemp(prefix="shellcov-"))
    try:
        seen = run_harnesses(root, workdir)
        # Codecov ingestion: the merged kcov run emits a cobertura.xml covering
        # every script kcov executed (never-run corpus files are absent — they
        # are the 0%-denominator the JSON artifact/floor account for, not
        # Codecov). Copy it out before the tempdir is reaped so CI can upload it.
        if cobertura_out is not None:
            merged_cobertura = workdir / "merged" / "kcov-merged" / "cobertura.xml"
            if merged_cobertura.is_file():
                cobertura_out.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(merged_cobertura, cobertura_out)
            else:
                sys.stderr.write(
                    f"WARN: no merged cobertura.xml at {merged_cobertura} — "
                    f"Codecov artifact {cobertura_out} not written\n"
                )
    finally:
        if not keep_workdir:
            shutil.rmtree(workdir, ignore_errors=True)

    files, validation_warnings = [], []  # type: ignore[var-annotated]
    covered_total = line_total = 0

    for rel in sorted(corpus_rel):
        abs_path = root / rel
        if rel in seen:
            covered, ktotal = seen[rel]
            counter_total = coverable_line_count(abs_path)
            # Anti-gaming: the counter must track kcov's total on files we CAN
            # see, else its denominator for unseen files is untrustworthy.
            if ktotal > 0:
                drift = abs(counter_total - ktotal) / ktotal
                if drift > 0.15:
                    validation_warnings.append(
                        f"WARN {rel}: counter={counter_total} kcov={ktotal} "
                        f"drift={drift*100:.1f}% (>15%)"
                    )
            files.append(FileCov(rel, covered, ktotal, seen=True))
            covered_total += covered
            line_total += ktotal
        else:
            total = coverable_line_count(abs_path)
            files.append(FileCov(rel, 0, total, seen=False))
            line_total += total

    aggregate_pct = 100.0 * covered_total / line_total if line_total else 0.0

    asserts_passed: list[str] = []
    asserts_failed: list[str] = []

    asserts_passed.append("kcov is available")

    counter_assert = (
        "coverable-line counter validated within 15% of kcov on all executed scripts"
    )
    (asserts_failed if validation_warnings else asserts_passed).append(counter_assert)

    floor_assert = (
        "aggregate shell line-coverage >= floor (quality/shell-coverage-floor.txt)"
    )
    below = round(aggregate_pct, 2) < round(floor, 2)
    (asserts_failed if below else asserts_passed).append(floor_assert)

    return Result(
        aggregate_pct=aggregate_pct,
        floor=floor,
        covered_total=covered_total,
        line_total=line_total,
        files=files,
        asserts_passed=asserts_passed,
        asserts_failed=asserts_failed,
        validation_warnings=validation_warnings,
    )


def print_table(res: Result) -> None:
    seen_n = sum(1 for f in res.files if f.seen)
    covered_gt0 = sum(1 for f in res.files if f.covered > 0)
    zero_n = sum(1 for f in res.files if f.covered == 0)
    print(f"{'FILE':<64} {'COV':>6} {'TOT':>6} {'PCT':>7}  SEEN")
    print("-" * 92)
    for f in sorted(res.files, key=lambda x: (x.pct, -x.total, x.path)):
        mark = "yes" if f.seen else " - "
        print(
            f"{f.path:<64} {f.covered:>6} {f.total:>6} {f.pct:>6.1f}%  {mark}"
        )
    print("-" * 92)
    print(
        f"corpus={len(res.files)} scripts  seen-by-kcov={seen_n}  "
        f"covered>0%={covered_gt0}  at-0%={zero_n}"
    )
    print(
        f"AGGREGATE {res.aggregate_pct:.2f}%  "
        f"({res.covered_total}/{res.line_total} lines)  floor={res.floor:.2f}%"
    )
    if res.validation_warnings:
        print("\nCOUNTER VALIDATION WARNINGS (anti-gaming):")
        for w in res.validation_warnings:
            print("  " + w)
    else:
        print("counter validation: clean (all executed files within 15%)")


def write_json(res: Result, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "aggregate_pct": round(res.aggregate_pct, 4),
        "floor": res.floor,
        "covered_total": res.covered_total,
        "line_total": res.line_total,
        "files": [
            {
                "path": f.path,
                "covered": f.covered,
                "total": f.total,
                "pct": round(f.pct, 2),
                "seen": f.seen,
            }
            for f in sorted(res.files, key=lambda x: (x.pct, x.path))
        ],
        "asserts_passed": res.asserts_passed,
        "asserts_failed": res.asserts_failed,
        "validation_warnings": res.validation_warnings,
    }
    dest.write_text(json.dumps(payload, indent=2) + "\n")


def read_floor(floor_file: Path) -> float:
    try:
        return float(floor_file.read_text().strip())
    except (OSError, ValueError):
        return 0.0


def cmd_run(args: argparse.Namespace) -> int:
    root = repo_root()
    if shutil.which("kcov") is None:
        sys.stderr.write(
            "FAIL: kcov not installed — run: sudo apt-get install -y kcov\n"
        )
        return 1
    floor_file = Path(args.floor_file)
    if not floor_file.is_absolute():
        floor_file = root / floor_file
    floor = read_floor(floor_file)

    cobertura_out: Path | None = None
    if args.cobertura_out:
        cobertura_out = Path(args.cobertura_out)
        if not cobertura_out.is_absolute():
            cobertura_out = root / cobertura_out

    res = compute(
        root, floor, keep_workdir=args.keep_workdir, cobertura_out=cobertura_out
    )
    print_table(res)

    if cobertura_out is not None and cobertura_out.is_file():
        print(f"cobertura: {cobertura_out}")

    if args.json:
        dest = Path(args.json)
        if not dest.is_absolute():
            dest = root / dest
        write_json(res, dest)
        print(f"\nartifact: {dest}")

    if round(res.aggregate_pct, 2) < round(floor, 2):
        sys.stderr.write(
            f"FAIL: aggregate {res.aggregate_pct:.2f}% < floor {floor:.2f}%\n"
        )
        return 1
    return 0


def cmd_enumerate(args: argparse.Namespace) -> int:
    root = repo_root()
    for p in enumerate_corpus(root):
        print(p.relative_to(root))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_run = sub.add_parser("run", help="drive harnesses, grade aggregate coverage")
    p_run.add_argument(
        "--floor-file", default="quality/shell-coverage-floor.txt"
    )
    p_run.add_argument("--json", default=None, help="write JSON artifact here")
    p_run.add_argument(
        "--cobertura-out",
        default=None,
        help="copy the merged kcov cobertura.xml here (for Codecov upload)",
    )
    p_run.add_argument(
        "--keep-workdir", action="store_true", help="keep kcov temp dir (debug)"
    )
    p_run.set_defaults(func=cmd_run)

    p_enum = sub.add_parser("enumerate", help="print the in-scope corpus")
    p_enum.set_defaults(func=cmd_enumerate)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
