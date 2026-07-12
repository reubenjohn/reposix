#!/usr/bin/env bash
# Wrapper: invoke the compiled docs-alignment hash drift walker.
#
# Detects target/release/reposix-quality first (CI / steady state); falls back
# to target/debug/reposix-quality when only debug artifacts exist (developer
# loop). Exits non-zero with a clear stderr message if neither binary exists --
# the runner forwards stderr verbatim so the slash-command hint reaches the
# user.
#
# Source: crates/reposix-quality/src/commands/doc_alignment.rs (verb `walk`).
#
# ---------------------------------------------------------------------------
# GRADE / PERSIST split (D-P96-01 parity, v0.14.0 Lane A):
#
# The Rust `walk` verb recomputes coverage counters (lines_covered,
# coverage_ratio) on every run and writes them back whenever they differ from
# the on-disk bytes -- so a bare GRADE walk silently mutated the committed
# catalog quality/catalogs/doc-alignment.json (the old workaround was a manual
# `git checkout -- quality/catalogs/doc-alignment.json` after every pre-push).
# That is the SAME self-mutation class run.py's --persist gate closes for the
# status catalogs: a read-only GATE run must never dirty a committed artifact.
#
# This wrapper mirrors run.py at the shell boundary WITHOUT touching Rust:
#   * default (GRADE): run the binary against a throwaway /tmp COPY of the
#     catalog via --catalog. Source/test hashes resolve relative to cwd (the
#     repo root the runner cd's into), so drift detection + the exit code are
#     BYTE-IDENTICAL to a real-catalog walk -- only the write target differs.
#     The committed catalog is never touched; `git status` stays clean.
#   * --persist (MINT): run against the real committed catalog so a deliberate
#     coverage/verdict refresh is written. This is the sanctioned way to update
#     doc-alignment.json (e.g. after docs move covered lines); run.py never
#     forwards --persist to a verifier (row.verifier.args == []), so pre-push /
#     pre-pr GATE runs are grade-only by construction.
#
# An explicit `--catalog <path>` in the args is always respected as-is (the
# caller has chosen their own write target -- e.g. crates/reposix-quality/tests
# drive temp catalogs); the grade-copy shim only guards the DEFAULT committed
# catalog.
# ---------------------------------------------------------------------------

set -euo pipefail

readonly REPO_ROOT="$(git rev-parse --show-toplevel)"
readonly RELEASE_BIN="${REPO_ROOT}/target/release/reposix-quality"
readonly DEBUG_BIN="${REPO_ROOT}/target/debug/reposix-quality"
readonly DEFAULT_CATALOG="${REPO_ROOT}/quality/catalogs/doc-alignment.json"

if [[ -x "$RELEASE_BIN" ]]; then
  BIN="$RELEASE_BIN"
elif [[ -x "$DEBUG_BIN" ]]; then
  BIN="$DEBUG_BIN"
else
  printf '%s\n' "docs-alignment/walk: neither target/release/reposix-quality nor target/debug/reposix-quality exists" >&2
  printf '%s\n' "  build the binary first: cargo build -p reposix-quality --release" >&2
  exit 1
fi

# Partition args: strip our own --persist flag; detect a caller-supplied
# --catalog (either `--catalog X` or `--catalog=X`).
PERSIST=0
CALLER_CATALOG=0
FWD_ARGS=()
for a in "$@"; do
  case "$a" in
    --persist) PERSIST=1 ;;
    --catalog|--catalog=*) CALLER_CATALOG=1; FWD_ARGS+=("$a") ;;
    *) FWD_ARGS+=("$a") ;;
  esac
done

# The walker exits non-zero on any blocking row state (STALE_DOCS_DRIFT,
# MISSING_TEST, STALE_TEST_GONE, TEST_MISALIGNED, RETIRE_PROPOSED) and prints
# the slash-command hint to stderr verbatim. Forward stderr unchanged so the
# runner surfaces it in every branch below.

# Caller picked their own catalog target, or an explicit MINT: run directly.
if [[ "$CALLER_CATALOG" -eq 1 ]]; then
  exec "$BIN" walk "${FWD_ARGS[@]}"
fi

if [[ "$PERSIST" -eq 1 ]]; then
  # MINT: deliberately write the committed catalog.
  exec "$BIN" walk --catalog "$DEFAULT_CATALOG" "${FWD_ARGS[@]}"
fi

# GRADE (default): walk a throwaway copy so the committed catalog stays clean.
TMP_CATALOG="$(mktemp -t reposix-doc-alignment-walk.XXXXXX.json)"
trap 'rm -f "$TMP_CATALOG"' EXIT
cp "$DEFAULT_CATALOG" "$TMP_CATALOG"
rc=0
"$BIN" walk --catalog "$TMP_CATALOG" "${FWD_ARGS[@]}" || rc=$?
exit "$rc"
