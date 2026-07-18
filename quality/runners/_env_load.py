"""Conditional ./.env self-sourcing for the Quality Gates runner. P123 SC1 (DRAIN-03).

Stdlib-only sibling of run.py (mirrors _freshness.py / _realbackend.py /
_audit_field.py / _shell_verdict.py — keeps run.py under its anti-bloat LOC
cap). Closes the false-green-preflight / silent-skip gap
(.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md 2026-07-14 20:43 HIGH):
scripts/preflight-real-backends.sh already sources ./.env, but run.py did not,
so `run.py --cadence pre-release-real-backend` silently skipped every
real-backend row to NOT-VERIFIED unless the caller had pre-sourced .env — while
preflight independently reported the backends reachable. The two together gave a
false impression of full coverage.

Precedence: EXISTING-ENV-WINS, per key (os.environ.setdefault). An operator- or
CI-exported credential is NEVER overridden by a stale ./.env value. Its
NON-CLOBBER GUARD is a strict per-key superset of
scripts/preflight-real-backends.sh's file-level `[ -z "${CRED:-}" ]` guard (which
protects only a few named cred vars before sourcing at all): per-key setdefault
protects EVERY key, not just the cred handful. It is the safest reading of
"explicit shell env always wins" and matches the backing row's expected asserts +
the non-clobber unit test.

Parsing scope (deliberately a MINIMAL subset of shell `source`, NOT a superset of
it): `KEY=value` and `export KEY=value` lines (a leading `export ` token is
stripped so a cred written with shell export syntax loads KEY, not "export KEY"),
`#`-comments, blank lines, and surrounding matched single/double quotes on the
value. It does NOT interpret shell variable expansion, command substitution, or
multi-line values. For every credential line preflight can `source`, this loads
the SAME key — so neither path silently loads a cred the other misses (closing the
false-green-preflight divergence) — but it remains a subset of full shell-source
semantics, never a superset of it.

OP-1 fail-closed is UNCHANGED: sourcing .env only makes "creds in .env" effective
— a real backend is still hit only when creds are present AND
REPOSIX_ALLOWED_ORIGINS is non-default (enforced elsewhere). This closes the
silent false-green skip; it does NOT make real-backend rows run unconditionally.

Secret hygiene: the one diagnostic line names loaded KEY names only, never
VALUES — mirroring the kind:shell-subprocess transcript convention (env_keys:
NAMES only, no =value pairs; quality/CLAUDE.md).
"""
from __future__ import annotations

import os
import sys
from pathlib import Path


def load_dotenv_if_present(repo_root: Path) -> None:
    """Source ``repo_root/.env`` into os.environ if it exists — present-only,
    non-clobbering.

    No-op (no error, no output) when the file is absent — the normal CI case.
    Blank lines and ``#``-comment lines are skipped; a line with no ``=`` is
    skipped (malformed, non-fatal). Values are stripped of surrounding matched
    single/double quotes. Each key is applied via ``os.environ.setdefault`` so
    an already-present env var (operator/CI export) always wins over the .env
    value — never ``os.environ[key] = value``.

    Emits ONE stderr line naming the KEY NAMES newly loaded (never their
    values); silent when the file is absent or nothing new was loaded.
    """
    env_path = Path(repo_root) / ".env"
    if not env_path.exists():
        return
    loaded: list[str] = []
    for raw in env_path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue  # malformed — skip, not fatal
        key, _, value = line.partition("=")
        key = key.strip()
        # Strip a leading `export ` token so a line written with shell export
        # syntax (`export KEY=value`) loads KEY, not "export KEY". Without this,
        # scripts/preflight-real-backends.sh (which source-includes .env and so
        # honors `export`) would see the cred while run.py loaded the wrong key
        # and skipped the row to NOT-VERIFIED — the exact false-green-preflight
        # divergence SC1/DRAIN-03 closes. `export` must be a whole token (the
        # char after it is whitespace); `exportFOO=…` is left as key `exportFOO`.
        if key.startswith("export") and key[6:7].isspace():
            key = key[6:].strip()
        if not key:
            continue
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
            value = value[1:-1]
        already_present = key in os.environ
        os.environ.setdefault(key, value)  # existing env always wins
        if not already_present:
            loaded.append(key)
    if loaded:
        # KEY NAMES only — never values (quality/CLAUDE.md env_keys convention).
        print(
            f"run.py: sourced {len(loaded)} var(s) from ./.env: {', '.join(loaded)}",
            file=sys.stderr,
        )
