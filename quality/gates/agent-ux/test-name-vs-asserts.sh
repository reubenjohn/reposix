#!/usr/bin/env bash
# quality/gates/agent-ux/test-name-vs-asserts.sh — RBF-FW-09 (F-K8)
# Scans crates/**/*.rs for #[test]/#[tokio::test] fns whose NAME promises
# real/push/fetch/dark-factory/round-trip/e2e coverage the BODY does not
# deliver. Implements catalog row agent-ux/test-name-vs-asserts.
#
# ─────────────────────────────────────────────────────────────────────────
# Regex-scope trade-off narrative (mirrors banned-production-tokens.sh:7-30
# and CLAUDE.md § "Quality Gates"):
#
#   CATCHES:    any #[test]/#[tokio::test] fn whose identifier contains
#               real | dark_factory | round_trip | roundtrip | end_to_end |
#               e2e | push | fetch (case-insensitive substring match).
#
#   INTENTIONALLY DOES NOT ATTEMPT: automated body analysis (does the fn
#               genuinely invoke `Command::new("git")` push/fetch, or hit a
#               non-127.0.0.1 host?). That judgment is made ONCE, by a
#               human/subagent, at marker-seeding time (R2 § C.1) — the
#               gate itself only checks for the PRESENCE of the per-line
#               marker or #[ignore]-real-backend gate, not the semantic
#               truth of the claim. This is a deliberate scope cut: a test
#               whose marker lies (claims "genuine push coverage" when it
#               isn't) is a code-review problem, not a shell-regex problem.
#
#   FORWARD CONVENTION: a NEW test fn matching the keyword set must EITHER
#               (a) carry `#[ignore = "real-backend; ..."]` (the existing
#               agent_flow_real.rs convention — note the literal substring
#               "real-backend" in the ignore reason, NOT just "sim-child"
#               gating, which does NOT auto-exempt — see dark_factory_
#               sim_happy_path, which IS #[ignore]-gated for a sim-child
#               reason and is STILL in the dishonest baseline below), OR
#               (b) carry a `// test-name-honesty: ok — <reason>` marker,
#               OR (c) be added to the committed baseline fixture (only
#               for genuinely dishonest names whose fix is out of scope —
#               see raise-list-p90.md § 2).
# ─────────────────────────────────────────────────────────────────────────
#
# Allowlist marker: `// test-name-honesty: ok — <reason>` on the fn's own
# signature line, OR on its own line immediately above the fn (mirrors
# `// banned-words: ok` precedent; the standalone-line-above form exists
# because a same-line trailing comment can push a line past rustfmt's
# max_width=100 and get relocated into the fn body — a standalone comment
# line is untouched by rustfmt regardless of length). Per the
# deferral-linter's no-PNN lesson (deferral-pointer-linter.sh:8-15), a BARE
# marker with no `— <reason>` itself RAISEs — it closes the silent-
# exemption loophole where a marker is added without justification.
#
# ⚠ MARKER PLACEMENT WINDOW (6-line LOOKBACK — read before adding a marker).
#   Per matched fn the gate builds context from the `CONTEXT_LINES=6` lines
#   ENDING AT the `fn ...(` line (6 lines above + the signature; see the
#   `sed -n "...line-CONTEXT_LINES...,line p"` below). The `#[test]` attr,
#   any `#[ignore = "real-backend..."]`, AND the `// test-name-honesty: ok`
#   marker must all sit inside it. If the marker drifts >6 lines above the fn
#   (e.g. behind a long `///` block) it is SILENTLY IGNORED; worse, if the
#   `#[test]` attr itself falls out of the window the fn is skipped as a
#   non-test and a dishonest name passes with NO raise. Keep the marker (and
#   #[test]/#[ignore]) hugging the signature — on its own line directly above
#   or as a trailing comment on the `fn` line. Tight placement is deliberate;
#   a preamble-anchored scan removing the distance constraint is a filed
#   GOOD-TO-HAVE (v0.13.0). Mirror note: quality/CLAUDE.md § "Honesty rules".
#
# Baseline mechanism: quality/gates/agent-ux/test-name-vs-asserts.baseline
# is a committed fixture, one `relative/path.rs:fn_name` key per line. Line
# numbers drift with unrelated edits elsewhere in a file; `file:fn` is the
# stable key (a rename would need a new baseline entry anyway, which is the
# point — it forces a conscious look at the renamed test). Gate exits:
#   0  — every un-exempted, un-ignored dishonest-named match is present in
#        the committed baseline (known debt, tracked in raise-list-p90.md).
#   1  — a NEW un-baselined dishonest match appears, OR a bare (no-reason)
#        honesty marker is found.
# A baseline entry that no longer appears in the live match set (i.e. the
# test was renamed, deleted, or fixed) does NOT fail the gate — draining
# the baseline is a good outcome and must never be punished, or nobody
# would ever fix the underlying dishonest test. It prints a WARN so the
# baseline file can be trimmed in a follow-up commit, but exits 0.
#
# Implementation note: uses `grep -rnEi` (not `rg`), matching every other
# gate in this dimension (banned-production-tokens.sh, deferral-pointer-
# linter.sh) — no ripgrep dependency assumed on CI/dev machines.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

PATTERN='\bfn[[:space:]]+[A-Za-z0-9_]*(real|dark_factory|round_?trip|end_to_end|e2e|push|fetch)[A-Za-z0-9_]*[[:space:]]*\('
MARKER='// test-name-honesty: ok'
BASELINE_FILE="quality/gates/agent-ux/test-name-vs-asserts.baseline"
CONTEXT_LINES=6

new_raises=0
bare_marker_violations=0
declare -A seen_baseline_keys=()

while IFS=: read -r file line content; do
  # Extract the fn identifier following the `fn` keyword.
  fnname="$(sed -E 's/^.*\bfn[[:space:]]+([A-Za-z_][A-Za-z0-9_]*).*$/\1/' <<< "$content")"
  [[ -z "$fnname" ]] && continue

  ctx="$(sed -n "$(( line > CONTEXT_LINES ? line - CONTEXT_LINES : 1 )),${line}p" "$file")"

  # Not actually a #[test]/#[tokio::test] fn (e.g. a production fn/method
  # named `real_main`, `push_mirror`, `log_helper_fetch_error`) — skip.
  if ! grep -qE '#\[(tokio::)?test(\(.*\))?\]' <<< "$ctx"; then
    continue
  fi

  # Structural exemption: #[ignore = "real-backend; ..."] gating. Note the
  # literal "real-backend" substring requirement — a sim-child #[ignore]
  # reason (e.g. dark_factory_sim_happy_path) does NOT qualify; it stays a
  # baseline entry.
  if grep -qE '#\[ignore[[:space:]]*=[[:space:]]*"real-backend' <<< "$ctx"; then
    continue
  fi

  key="${file}:${fnname}"

  # The marker may live on the fn's own signature line, OR on its own line
  # immediately above the fn (the latter is how the marker-seeding pass
  # placed all 56 initial exemptions — a same-line trailing comment risks
  # rustfmt relocating it into the fn body once the line exceeds
  # max_width=100, which would silently defeat same-line-only detection).
  # Search the whole #[test]-lookback context window either way.
  marker_line="$(grep -F "$MARKER" <<< "$ctx" | tail -1 || true)"
  if [[ -n "$marker_line" ]]; then
    # Marker present. Require ` — <reason>` (em dash + non-empty text)
    # after the marker text; a bare marker itself RAISEs.
    after="${marker_line#*$MARKER}"
    reason="$(sed -E 's/^[[:space:]]*—[[:space:]]*//' <<< "$after")"
    if [[ -z "${reason// /}" || "$after" == "$reason" ]]; then
      printf '✖ %s:%s: %s — bare test-name-honesty marker (no reason)\n' "$file" "$line" "$fnname" >&2
      printf '   owner_hint: append "— <reason>" explaining why the name is honest, or remove the marker\n' >&2
      bare_marker_violations=1
    fi
    continue
  fi

  # No marker, not ignore-exempt: this is a dishonest-name candidate.
  # Expected iff already in the committed baseline.
  if [[ -f "$BASELINE_FILE" ]] && grep -qxF "$key" "$BASELINE_FILE"; then
    seen_baseline_keys["$key"]=1
    continue
  fi

  printf '✖ %s:%s: %s — test name promises real/push/fetch/round-trip/e2e/dark-factory coverage, but carries no test-name-honesty marker and is not in the committed baseline\n' "$file" "$line" "$fnname" >&2
  new_raises=1
done < <(grep -rnEi --include='*.rs' --exclude-dir=target "$PATTERN" crates/ 2>/dev/null || true)

if [[ -f "$BASELINE_FILE" ]]; then
  while IFS= read -r baseline_key; do
    [[ -z "$baseline_key" ]] && continue
    if [[ -z "${seen_baseline_keys[$baseline_key]:-}" ]]; then
      echo "WARN: baseline entry '$baseline_key' no longer matches — the underlying test may have been fixed/renamed/removed; trim it from $BASELINE_FILE in a follow-up commit" >&2
    fi
  done < "$BASELINE_FILE"
fi

if [[ $new_raises -eq 1 || $bare_marker_violations -eq 1 ]]; then
  echo "" >&2
  echo "owner_hint: fix the test body to genuinely exercise the claim, add '// test-name-honesty: ok — <reason>' if the name IS honest, or (only for pre-existing debt) add the file:fn key to $BASELINE_FILE and record it in raise-list-p90.md § 2" >&2
  echo "see: quality/catalogs/agent-ux.json row agent-ux/test-name-vs-asserts" >&2
  exit 1
fi

echo "PASS: test-name-vs-asserts RAISE set matches the committed baseline (no new dishonest names, no bare honesty markers)"
