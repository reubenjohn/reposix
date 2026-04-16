#!/usr/bin/env bash
#
# green-gauntlet.sh — run every gate a release needs to clear.
#
# This is the pre-tag check any agent or human should run before
# `scripts/tag-vX.Y.Z.sh`. The tag script already re-runs a subset of
# these (cargo test + smoke), but fmt/clippy/mkdocs need to be green
# too, and a single named command beats typing five pipelines.
#
# Each gate exits non-zero on the first red check. Timings are
# printed so regressions stand out.
#
# Usage:
#   bash scripts/green-gauntlet.sh [--quick|--full]
#
# Modes:
#   default  — fmt, clippy, test, smoke (fast, default).
#   --full   — default + mkdocs --strict + FUSE `--ignored` tests.
#              Requires `fusermount3` on PATH and a reasonable time
#              budget (~60s extra).
#   --quick  — fmt + clippy only. For tight inner-dev loops.
#
# Exit codes: 0 = all green, 1 = any gate red.

set -euo pipefail

readonly GREEN='\033[0;32m'
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode="${1:-default}"

print_header() {
  printf '%b\n' "${BOLD}== [${1}] ${2}${NC}"
}

run_gate() {
  local name="$1"
  local cmd="$2"
  local start end elapsed
  start="$(date +%s)"
  print_header "running" "$name"
  if eval "$cmd" > /tmp/green-gauntlet-${name}.log 2>&1; then
    end="$(date +%s)"
    elapsed=$((end - start))
    printf '%b %s %b %ds\n' "${GREEN}✓${NC}" "$name" "${GREEN}ok${NC}" "$elapsed"
    return 0
  fi
  end="$(date +%s)"
  elapsed=$((end - start))
  printf '%b %s %b %ds\n' "${RED}✖${NC}" "$name" "${RED}FAIL${NC}" "$elapsed" >&2
  printf '%b\n' "${YELLOW}→${NC} last 30 lines of /tmp/green-gauntlet-${name}.log:" >&2
  tail -n 30 "/tmp/green-gauntlet-${name}.log" | sed 's/^/    /' >&2
  return 1
}

failed=0
run_gauntlet() {
  local quick="${1:-}"
  local full="${2:-}"

  # Always-run gates.
  run_gate fmt 'cargo fmt --all --check' || failed=$((failed + 1))
  run_gate clippy 'cargo clippy --workspace --all-targets --locked -- -D warnings' \
    || failed=$((failed + 1))

  # Quick-mode stops here.
  [[ "$quick" == "quick" ]] && return

  # Default gates.
  run_gate test 'cargo test --workspace --locked' || failed=$((failed + 1))

  if command -v reposix-sim >/dev/null 2>&1 \
      || [[ -x target/release/reposix-sim ]] \
      || [[ -x target/debug/reposix-sim ]]; then
    if [[ -x target/release/reposix-sim ]]; then
      export PATH="${repo_root}/target/release:$PATH"
    elif [[ -x target/debug/reposix-sim ]]; then
      export PATH="${repo_root}/target/debug:$PATH"
    fi
    run_gate smoke 'bash scripts/demos/smoke.sh' || failed=$((failed + 1))
  else
    printf '%b smoke %b — no reposix-sim binary on PATH\n' \
      "${YELLOW}⊘${NC}" "${YELLOW}skipped${NC}"
    printf '   (hint: cargo build --release --workspace --bins, then re-run)\n'
  fi

  # Full-mode adds mkdocs + FUSE integration tests.
  if [[ "$full" == "full" ]]; then
    if command -v mkdocs >/dev/null 2>&1; then
      run_gate mkdocs-strict 'mkdocs build --strict' || failed=$((failed + 1))
    else
      printf '%b mkdocs-strict %b — mkdocs not on PATH\n' \
        "${YELLOW}⊘${NC}" "${YELLOW}skipped${NC}"
    fi
    if command -v fusermount3 >/dev/null 2>&1; then
      run_gate fuse-mount-tests \
        'cargo test --release -p reposix-fuse --locked --features fuse-mount-tests -- --test-threads=1' \
        || failed=$((failed + 1))
    else
      printf '%b fuse-mount-tests %b — fusermount3 not on PATH\n' \
        "${YELLOW}⊘${NC}" "${YELLOW}skipped${NC}"
    fi
  fi
}

case "$mode" in
  --quick) run_gauntlet quick "" ;;
  --full)  run_gauntlet "" full ;;
  default) run_gauntlet "" "" ;;
  *)
    printf 'usage: %s [--quick|--full]\n' "$0" >&2
    exit 2 ;;
esac

printf '\n'
if [[ "$failed" -eq 0 ]]; then
  printf '%b\n' "${GREEN}${BOLD}✓ green gauntlet passed${NC}"
  exit 0
fi
printf '%b\n' "${RED}${BOLD}✖ ${failed} gate(s) red${NC}" >&2
exit 1
