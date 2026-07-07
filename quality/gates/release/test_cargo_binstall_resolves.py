"""quality/gates/release/test_cargo_binstall_resolves.py -- pure-python regression
test for the release/cargo-binstall-resolves classifier.

Filed to fix CI run 28839335746 (v0.13.0 tag): the gate went RED even though
cargo-binstall genuinely resolved the v0.13.0 prebuilt binary (rc=0, ~1.97s)
because the old PASS_SIGNAL grepped only the literal substring
"github.com/reubenjohn/reposix/releases/download/" and current cargo-binstall
prints "has been downloaded from github.com" instead. This test feeds BOTH
wordings (old-URL and new "has been downloaded from") through `classify()`
and asserts both land on PASS, and separately locks in the PARTIAL
(source-compile fallback) and FAIL (genuine no-resolve) branches.

Deliberately stdlib+pytest only -- no cargo/cargo-binstall invocation, no
network. Run: python3 -m pytest quality/gates/release/test_cargo_binstall_resolves.py -x -q
"""
from __future__ import annotations

import importlib.util
import pathlib
import sys

import pytest

_MODULE_PATH = pathlib.Path(__file__).resolve().parent / "cargo-binstall-resolves.py"
_spec = importlib.util.spec_from_file_location("cargo_binstall_resolves", _MODULE_PATH)
assert _spec is not None and _spec.loader is not None
cargo_binstall_resolves = importlib.util.module_from_spec(_spec)
sys.modules["cargo_binstall_resolves"] = cargo_binstall_resolves
_spec.loader.exec_module(cargo_binstall_resolves)

classify = cargo_binstall_resolves.classify


OLD_URL_STDOUT = """\
INFO resolve: Resolving package: 'reposix-cli'
INFO downloading from: 'https://github.com/reubenjohn/reposix/releases/download/v0.12.0/reposix-cli-x86_64-unknown-linux-musl.tar.gz'
INFO This will install the following binaries: reposix => /home/runner/.cargo/bin/reposix
INFO Done in 1.5s
"""

NEW_WORDING_STDOUT = """\
INFO resolve: Resolving package: 'reposix-cli'
WARN The package reposix-cli v0.13.0 (x86_64-unknown-linux-musl) has been downloaded from github.com
INFO This will install the following binaries: reposix => /home/runner/.cargo/bin/reposix
INFO Done in 1.976293126s
"""

NO_RESOLVE_STDOUT = """\
INFO resolve: Resolving package: 'reposix-cli'
ERROR could not find a binstall-compatible release artifact for reposix-cli v0.13.0
ERROR 404 Not Found
"""

SOURCE_FALLBACK_STDOUT = """\
INFO resolve: Resolving package: 'reposix-cli'
WARN Falling back to source install for reposix-cli
INFO running `cargo install reposix-cli`
INFO compiling reposix-cli v0.13.0
"""


def test_old_url_wording_classifies_pass() -> None:
    result = classify(0, OLD_URL_STDOUT)
    assert result["status_label"] == "PASS"
    assert result["exit_code"] == 0
    assert result["asserts_passed"]
    assert not result["asserts_failed"]


def test_new_downloaded_from_wording_classifies_pass() -> None:
    """Regression test for CI run 28839335746 -- the exact wording that
    false-negatived the v0.13.0 post-release gate."""
    result = classify(0, NEW_WORDING_STDOUT)
    assert result["status_label"] == "PASS"
    assert result["exit_code"] == 0
    assert result["matched_signal"] == "has been downloaded from github.com"


def test_genuine_no_resolve_classifies_fail() -> None:
    result = classify(1, NO_RESOLVE_STDOUT)
    assert result["status_label"] == "FAIL"
    assert result["exit_code"] == 1
    assert result["asserts_failed"]


def test_source_compile_fallback_classifies_partial() -> None:
    result = classify(0, SOURCE_FALLBACK_STDOUT)
    assert result["status_label"] == "PARTIAL"
    assert result["exit_code"] == 2
    assert result["fallback_signal"]


def test_rc_nonzero_with_pass_wording_does_not_pass() -> None:
    """A pass-shaped wording is not enough on its own -- rc must be 0 too."""
    result = classify(1, NEW_WORDING_STDOUT)
    assert result["status_label"] != "PASS"


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-x", "-q"]))
