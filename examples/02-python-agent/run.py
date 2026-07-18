#!/usr/bin/env python3
"""02-python-agent -- find issues mentioning "database", label severity: medium.

Stdlib only. The only reposix-specific call is `reposix init sim::demo <path>`;
everything after that is `subprocess.run(["git", ...])` and string IO.
"""
from __future__ import annotations

import os
import pathlib
import re
import shutil
import subprocess
import sys
import urllib.error
import urllib.request

WORK = pathlib.Path(os.environ.get("WORK", "/tmp/reposix-example-02"))
SIM_URL = os.environ.get("SIM_URL", "http://127.0.0.1:7878")


def add_target_debug_to_path() -> None:
    """If `target/debug/reposix` exists relative to this file, prepend to PATH."""
    here = pathlib.Path(__file__).resolve()
    debug = here.parent.parent.parent / "target" / "debug"
    if (debug / "reposix").is_file():
        os.environ["PATH"] = f"{debug}{os.pathsep}{os.environ['PATH']}"


def run(*args: str, cwd: pathlib.Path | None = None) -> subprocess.CompletedProcess:
    """Thin wrapper around subprocess.run with check=True and captured output."""
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True)


def sim_reachable() -> bool:
    try:
        with urllib.request.urlopen(f"{SIM_URL}/projects/demo/issues", timeout=2) as r:
            return r.status == 200
    except (urllib.error.URLError, TimeoutError):
        return False


FRONTMATTER_RE = re.compile(r"\A---\n(?P<fm>.*?)\n---\n", re.DOTALL)


def add_severity(text: str, severity: str) -> str | None:
    """Insert `severity: <severity>` into the frontmatter of `text`.

    Returns the new text if a change was made, else None (e.g. if `severity:`
    is already present).
    """
    m = FRONTMATTER_RE.match(text)
    if not m:
        return None
    fm = m.group("fm")
    if re.search(r"^severity:", fm, re.MULTILINE):
        return None
    new_fm = fm.rstrip() + f"\nseverity: {severity}\n"
    return f"---\n{new_fm}---\n" + text[m.end():]


def main() -> int:
    add_target_debug_to_path()
    os.environ.setdefault("REPOSIX_ALLOWED_ORIGINS", SIM_URL)

    if not sim_reachable():
        print(f"FAIL: sim not reachable at {SIM_URL}", file=sys.stderr)
        print(
            "Start it: reposix-sim --bind 127.0.0.1:7878 "
            "--seed-file crates/reposix-sim/fixtures/seed.json --ephemeral",
            file=sys.stderr,
        )
        return 1

    if WORK.exists():
        shutil.rmtree(WORK)
    WORK.parent.mkdir(parents=True, exist_ok=True)

    # 1. Bootstrap.
    run("reposix", "init", "sim::demo", str(WORK))

    # 2. Materialise blobs and check out main. Trailing fatal from the
    #    init's best-effort fetch is harmless -- the ref is already in
    #    place. We re-run fetch and ignore its exit.
    subprocess.run(["git", "fetch", "origin"], cwd=WORK, capture_output=True, text=True)
    run("git", "checkout", "-q", "-B", "main", "refs/reposix/origin/main", cwd=WORK)

    # 3. Find issues whose body mentions "database" (case-insensitive).
    #    Records live under the canonical `issues/<id>.md` bucket (QL-001),
    #    not at the repo root.
    targets: list[pathlib.Path] = []
    for path in sorted(WORK.glob("issues/*.md")):
        body = path.read_text(encoding="utf-8")
        if re.search(r"database", body, re.IGNORECASE):
            targets.append(path)
    rels = [str(p.relative_to(WORK)) for p in targets]
    print(f"matched {len(targets)} issue(s) mentioning 'database': {rels}")

    # Fail loud on a zero-match run: the demo seed always contains at least
    # one issue body mentioning 'database'. A zero result means the glob
    # missed the records (e.g. a path-shape regression) -- exiting 0 here
    # would let the container-rehearse gate score a vacuous GREEN.
    if not targets:
        print(
            "FAIL: matched 0 issues mentioning 'database' -- expected >=1 in "
            f"the demo seed. Looked under {WORK}/issues/*.md; verify records "
            "materialised at the canonical issues/<id>.md path.",
            file=sys.stderr,
        )
        return 1

    # 4. Splice severity into the frontmatter.
    changed: list[pathlib.Path] = []
    for path in targets:
        new_text = add_severity(path.read_text(encoding="utf-8"), "medium")
        if new_text is None:
            print(f"  {path.name}: already labelled, skipped")
            continue
        path.write_text(new_text, encoding="utf-8")
        changed.append(path)
        print(f"  {path.relative_to(WORK)}: severity=medium added")

    if not changed:
        print("nothing to commit")
        return 0

    # Earned-congruence marker (harvested by container-rehearse.sh; DRAIN-22):
    # the agent matched >=1 'database' issue (fail-loud guard above) AND applied
    # a severity label to at least one of them.
    print(
        "ASSERT-PASS: agent found 'database' in an issue body and applied a "
        "severity: medium label",
        flush=True,
    )

    # 5. Stage, commit, push. Stage by the issues/<id>.md-relative path.
    run("git", "add", *(str(p.relative_to(WORK)) for p in changed), cwd=WORK)
    run(
        "git",
        "-c", "user.email=example@reposix.dev",
        "-c", "user.name=reposix-example",
        "commit",
        "-m", f"label severity:medium on {len(changed)} issue(s)",
        cwd=WORK,
    )
    push = subprocess.run(
        ["git", "push", "origin", "main"], cwd=WORK, capture_output=True, text=True
    )
    print(push.stdout, end="")
    print(push.stderr, end="", file=sys.stderr)
    if push.returncode != 0:
        return push.returncode

    # Earned-congruence markers (harvested by container-rehearse.sh; DRAIN-22):
    # the push exited 0 and, having reached here, the whole run succeeds.
    print("ASSERT-PASS: git push origin main reported success (push returncode 0)", flush=True)
    print(
        "ASSERT-PASS: python3 examples/02-python-agent/run.py completed and exits 0",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
