#!/usr/bin/env bash
# quality/gates/agent-ux/rebase-recovery-reconciles.sh
#
# Catalog row: agent-ux/rebase-recovery-reconciles  (RBF-LR-03, Phase 105)
# Kind: shell-subprocess  (real git-remote-reposix helper vs live sim + transcript)
#
# CONTRACT (graded by the unbiased verifier subagent — see the catalog row's
# expected.asserts): drive the REAL git-remote-reposix helper against a live sim
# through two SoT-drift scenarios and prove the DOCUMENTED single-command
# recovery `git pull --rebase && git push` reconciles after drift — with a
# NEGATIVE GUARD proving the pre-fix parentless emission RED-s so the gate bites.
#
# ────────────────────────────────────────────────────────────────────────────
# CURRENT STATUS (2026-07-12, Phase 105 Lane 2 → layer-2): PASS (exit 0).
#
# RBF-LR-03 shipped in two layers:
#   - Layer-1 (90ddaff) chained the synthesized tracking commit onto the
#     tracking tip, removing the `fatal: error while running fast-import` /
#     `does not contain` abort.
#   - Layer-2 (this gate's contract) fixed the SECOND, newly-exposed fetch-time
#     abort that layer-1 uncovered:
#
#       error: cannot lock ref 'refs/reposix/origin/main': is at <T1> but expected <T0>
#
#     Root cause: the `import` helper stream wrote `commit refs/reposix/origin/main`
#     DIRECTLY while the helper ALSO advertised `refspec
#     refs/heads/*:refs/reposix/origin/*`, so BOTH the stream and git fetch wrote
#     that one ref. On drift the stream fast-forwarded it T0→T1 underneath git;
#     git's post-import transaction (expected-old T0) then conflicted with the
#     already-moved T1 → `git pull --rebase` aborted → the documented
#     `git pull --rebase && git push` short-circuited at the `&&`.
#
#     Fix (restore the two-namespace remote-helper contract): the helper now
#     writes a PRIVATE namespace `refs/reposix-import/*`; git fetch maps it into
#     the tracking ns `refs/reposix/origin/*` via `remote.origin.fetch`,
#     remaining the SOLE writer. (fast_import.rs commit/reset targets +
#     main.rs advertised refspec, commit bd5b9cb.)
#
# DELETION COVERAGE (CR-01 / WR-01): Scenario C deletes a record at the SoT
# (REST DELETE) and proves the deletion PROPAGATES through the documented
# recovery — the record's file leaves the working tree after the rebase AND the
# push does NOT resurrect it at the SoT. A DELETION NEGATIVE GUARD feeds `git
# fast-import` the pre-fix overlay (`from`+`M`, no deleteall → issues/2.md
# resurrected) vs the post-fix rebuild (`from`+`deleteall`+`M` → issues/2.md
# dropped), proving the deletion assert bites. Pre-`deleteall`, emit_import_stream
# overlaid M-directives onto the inherited parent tree, so a deleted record
# survived the fetch and was re-created on the next push (silent resurrection).
#
# CONTRACT (this gate, both drift scenarios): the SINGLE documented command
# `git pull --rebase origin main && git push origin main` exits 0 AND the local
# edit reaches the SoT (issue2 version 1→2), with ZERO `fatal: error while
# running fast-import` (layer-1 guard) AND ZERO `cannot lock ref
# 'refs/reposix/origin/main'` (layer-2 guard) on the recovery path. A clobber
# guard additionally asserts the caller's local `refs/heads/main` was moved ONLY
# by the user's own commit (never by fetch) and the private
# `refs/reposix-import/main` staging ref exists. Two negative guards prove the
# gate bites: a parentless non-descendant fast-import RED-s with `does not
# contain` (layer-1), and the CONVERGENCE + `cannot lock ref`-absence asserts
# would fail against a pre-layer-2 binary (layer-2).
# ────────────────────────────────────────────────────────────────────────────
#
# IMPORT-PATH FORCING (PLAN §3 / §5). The RBF-LR-03 bug lives on git's `import`
# capability fetch path. git 2.25 selects `import` unaided; on a modern git
# (>= 2.34) fetch would route via `stateless-connect` (protocol v2). To exercise
# the broken path DETERMINISTICALLY on any git, this gate exports
# GIT_CONFIG_COUNT/KEY_0/VALUE_0 = protocol.version=0 for EVERY git subprocess
# (including the ones `reposix init`/`reposix-sim` shell out to), forcing v0 →
# the `import` path. The stateless-connect path (PLAN §5 open question) is NOT
# exercised here: only git 2.25.1 is installed in this environment, and forcing
# protocol.version=2 on 2.25 errors before the fetch (`bad line length 2`). See
# the STATELESS-CONNECT block below — it records a labelled skip + TODO rather
# than faking a result.
#
# Implements catalog row agent-ux/rebase-recovery-reconciles.
# Usage: rebase-recovery-reconciles.sh [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="agent-ux/rebase-recovery-reconciles"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ROW_SLUG="rebase-recovery-reconciles"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
TS_FILE="$(date -u +%Y-%m-%dT%H-%M-%SZ)"
TRANSCRIPT="${WORKSPACE_ROOT}/quality/reports/transcripts/${ROW_SLUG}-${TS_FILE}.txt"
ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/${ROW_SLUG}.json"
mkdir -p "$(dirname "$TRANSCRIPT")" "$(dirname "$ARTIFACT")"

# --- transcript accumulation ------------------------------------------------
# SECURITY (CLAUDE.md threat model, exfiltration leg): env_keys emits variable
# NAMES only, never values. The `cut -d= -f1` is load-bearing.
ENV_KEYS="$(env | cut -d= -f1 | sort | tr '\n' ',' | sed 's/,$//')"
: > "$TRANSCRIPT"
tlog() { printf '%s\n' "$*" >> "$TRANSCRIPT"; }
tlog "row_id: ${ROW_ID}"
tlog "ts: ${TS}"
tlog "argv: $0 $*"
tlog "cwd: $(pwd)"
tlog "env_keys: ${ENV_KEYS}"
tlog "git_version: $(git --version 2>&1)"

# Assertion bookkeeping. asserts_passed/failed are newline-delimited; converted
# to JSON arrays at the end.
PASSED=()
FAILED=()
BLOCKED=0            # 1 → residual ref-lock bug present → NOT-VERIFIED
REASON=""
pass() { PASSED+=("$1"); tlog "PASS: $1"; echo "  PASS: $1" >&2; }
fail() { FAILED+=("$1"); tlog "FAIL: $1"; echo "  FAIL: $1" >&2; }

json_array() {
  # $@ → JSON array of strings
  python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "$@"
}

FINAL_EXIT=0
write_artifact() {
  local exit_code="$1" status="$2"
  local ap af
  ap="$(json_array "${PASSED[@]+"${PASSED[@]}"}")"
  af="$(json_array "${FAILED[@]+"${FAILED[@]}"}")"
  local rel_transcript="${TRANSCRIPT#"${WORKSPACE_ROOT}"/}"
  python3 - "$ARTIFACT" "$TS" "$ROW_ID" "$exit_code" "$status" "$rel_transcript" "$REASON" "$ap" "$af" <<'PY'
import json,sys
path,ts,row,ec,status,tp,reason,ap,af=sys.argv[1:10]
json.dump({
  "ts": ts, "row_id": row, "exit_code": int(ec), "status": status,
  "reason": reason, "transcript_path": tp,
  "asserts_passed": json.loads(ap), "asserts_failed": json.loads(af),
}, open(path,"w"), indent=2)
PY
  tlog "exit_code: ${exit_code}"
  tlog "status: ${status}"
}

# --- cleanup / finalize -----------------------------------------------------
RUN_DIR="$(mktemp -d /tmp/${ROW_SLUG}.XXXXXX)"
SIM_PID=""
finish() {
  local ec="$1"
  [[ -n "$SIM_PID" ]] && { kill "$SIM_PID" 2>/dev/null; wait "$SIM_PID" 2>/dev/null; }
  rm -rf "$RUN_DIR"
  local status="PASS"
  [[ "$ec" -eq 75 ]] && status="NOT-VERIFIED"
  [[ "$ec" -ne 0 && "$ec" -ne 75 ]] && status="FAIL"
  write_artifact "$ec" "$status"
  echo "rebase-recovery-reconciles: exit ${ec} (${status}). transcript: ${TRANSCRIPT}" >&2
  exit "$ec"
}

# --- FORCE the import path (see header) -------------------------------------
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=protocol.version
export GIT_CONFIG_VALUE_0=0
tlog "import-forcing: GIT_CONFIG protocol.version=0 exported for all git subprocesses"

# --- build + resolve binaries -----------------------------------------------
# Single machine-wide cargo slot (crates/CLAUDE.md). Build the three crates the
# gate drives; jobs=2 is pinned in .cargo/config.toml.
echo "rebase-recovery-reconciles: building reposix-remote / reposix-cli / reposix-sim ..." >&2
if ! ( cd "$WORKSPACE_ROOT" && cargo build -q -p reposix-remote -p reposix-cli -p reposix-sim 2>&1 | tail -5 ); then
  fail "cargo build failed"
  finish 1
fi
BIN_DIR="${WORKSPACE_ROOT}/target/debug"
if [[ ! -x "${BIN_DIR}/reposix" || ! -x "${BIN_DIR}/reposix-sim" || ! -x "${BIN_DIR}/git-remote-reposix" ]]; then
  fail "expected reposix, reposix-sim, git-remote-reposix binaries in ${BIN_DIR}"
  finish 1
fi
export PATH="${BIN_DIR}:${PATH}"

# --- sim + shared env -------------------------------------------------------
PORT=7988
SIM_URL="http://127.0.0.1:${PORT}"
export REPOSIX_SIM_ORIGIN="$SIM_URL"          # honoured by translate_spec_to_url (init.rs:56)
export REPOSIX_ALLOWED_ORIGINS="$SIM_URL"     # egress allowlist (OP-1 fail-closed)
# Real, NON-t@t fixture identity (leaf-isolation guard rejects t@t).
export GIT_AUTHOR_NAME="RBF Gate" GIT_AUTHOR_EMAIL="gate@rbf.invalid"
export GIT_COMMITTER_NAME="RBF Gate" GIT_COMMITTER_EMAIL="gate@rbf.invalid"

cd "$RUN_DIR"    # sim/init/git/curl all run with cwd inside /tmp (leaf isolation)

if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
  fail "port ${PORT} already serving before spawn — refusing to run against unknown state"
  finish 1
fi
"${BIN_DIR}/reposix-sim" --bind "127.0.0.1:${PORT}" --db "${RUN_DIR}/sim.db" >"${RUN_DIR}/sim.log" 2>&1 &
SIM_PID=$!
for _ in $(seq 1 50); do
  kill -0 "$SIM_PID" 2>/dev/null || { fail "reposix-sim died during startup — see sim.log"; finish 1; }
  curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1 && break
  sleep 0.1
done

FATAL_RE='fatal: error while running fast-import|does not contain'
REFLOCK_RE="cannot lock ref 'refs/reposix/origin/main'"

# ============================================================================
# NEGATIVE GUARD — proves the gate BITES: a parentless changed snapshot fed to
# `git fast-import` against a seeded tracking ref is refused with `does not
# contain` (the exact pre-fix RED baseline). git-version-agnostic (drives
# `git fast-import` directly, mirroring the reposix-remote round-trip unit
# test). If this string does NOT appear, the gate cannot detect the regression
# it exists to guard → hard FAIL.
# ============================================================================
echo "rebase-recovery-reconciles: NEGATIVE GUARD (parentless non-descendant → does not contain)" >&2
NG="${RUN_DIR}/neg"; mkdir -p "$NG"; ( cd "$NG" && git init -q )
# stream 1: seed a clean PARENTLESS commit on refs/reposix/origin/main
{
  printf 'blob\nmark :1\ndata 3\nv1\n'
  printf 'commit refs/reposix/origin/main\nmark :2\ncommitter t <t@guard.invalid> 0 +0000\ndata 5\nseed\nM 100644 :1 issues/1.md\n'
} > "${NG}/seed.fi"
( cd "$NG" && git fast-import --quiet < "${NG}/seed.fi" ) >/dev/null 2>&1
SEED_TIP="$( cd "$NG" && git rev-parse refs/reposix/origin/main 2>/dev/null )"
# stream 2: a DIFFERENT parentless commit to the SAME ref → non-descendant → refused
{
  printf 'blob\nmark :1\ndata 3\nv2\n'
  printf 'commit refs/reposix/origin/main\nmark :2\ncommitter t <t@guard.invalid> 0 +0000\ndata 7\nchanged\nM 100644 :1 issues/1.md\n'
} > "${NG}/bad.fi"
NG_ERR="$( cd "$NG" && git fast-import --quiet < "${NG}/bad.fi" 2>&1 1>/dev/null )"
NG_TIP="$( cd "$NG" && git rev-parse refs/reposix/origin/main 2>/dev/null )"
tlog "--- NEGATIVE GUARD stderr ---"; tlog "$NG_ERR"
if echo "$NG_ERR" | grep -qE 'does not contain' && [[ "$NG_TIP" == "$SEED_TIP" ]]; then
  pass "NEGATIVE GUARD: parentless changed snapshot refused with \`does not contain\`, ref unchanged — the gate provably bites"
else
  fail "NEGATIVE GUARD did not reproduce \`does not contain\` (stderr: ${NG_ERR}) — the gate cannot detect the regression it guards"
  finish 1
fi

# ============================================================================
# DELETION NEGATIVE GUARD (CR-01) — proves the deletion assert BITES: a record
# removed at the SoT must DROP from the fetched tree. We feed `git fast-import`
# the exact two shapes the helper can emit against a seeded {1,2} tracking ref:
#   - PRE-FIX overlay  (`from <tip>` + `M issues/1.md`, NO deleteall) → the
#     ancestor tree is inherited and issues/2.md is RESURRECTED.
#   - POST-FIX rebuild (`from <tip>` + `deleteall` + `M issues/1.md`)  → the
#     tree is rebuilt from scratch and issues/2.md is DROPPED.
# If the overlay did NOT retain issues/2.md (or the rebuild did NOT drop it) the
# Scenario-C deletion assert below could not distinguish the regression it
# guards → hard FAIL. git-version-agnostic (drives `git fast-import` directly).
# ============================================================================
echo "rebase-recovery-reconciles: DELETION NEGATIVE GUARD (overlay resurrects vs deleteall drops issues/2.md)" >&2
DG="${RUN_DIR}/delguard"; mkdir -p "$DG"; ( cd "$DG" && git init -q )
{
  printf 'blob\nmark :1\ndata 3\nv1\n'
  printf 'blob\nmark :2\ndata 3\nv2\n'
  printf 'commit refs/reposix-import/main\nmark :3\ncommitter t <t@guard.invalid> 0 +0000\ndata 5\nseed\nM 100644 :1 issues/1.md\nM 100644 :2 issues/2.md\n'
} > "${DG}/seed.fi"
( cd "$DG" && git fast-import --quiet < "${DG}/seed.fi" ) >/dev/null 2>&1
DG_TIP="$( cd "$DG" && git rev-parse refs/reposix-import/main )"
# PRE-FIX overlay (no deleteall) → issues/2.md RETAINED.
{
  printf 'blob\nmark :1\ndata 3\nv1\n'
  printf 'commit refs/reposix-import/main\nmark :2\ncommitter t <t@guard.invalid> 0 +0000\ndata 4\ndrop\nfrom %s\nM 100644 :1 issues/1.md\n' "$DG_TIP"
} > "${DG}/overlay.fi"
( cd "$DG" && git fast-import --quiet < "${DG}/overlay.fi" ) >/dev/null 2>&1
OVERLAY_LS="$( cd "$DG" && git ls-tree -r --name-only refs/reposix-import/main )"
# POST-FIX rebuild (deleteall) from the SAME seed tip → issues/2.md DROPPED.
( cd "$DG" && git update-ref refs/reposix-import/main "$DG_TIP" )
{
  printf 'blob\nmark :1\ndata 3\nv1\n'
  printf 'commit refs/reposix-import/main\nmark :2\ncommitter t <t@guard.invalid> 0 +0000\ndata 4\ndrop\nfrom %s\ndeleteall\nM 100644 :1 issues/1.md\n' "$DG_TIP"
} > "${DG}/rebuild.fi"
( cd "$DG" && git fast-import --quiet < "${DG}/rebuild.fi" ) >/dev/null 2>&1
REBUILD_LS="$( cd "$DG" && git ls-tree -r --name-only refs/reposix-import/main )"
tlog "--- DELETION NEGATIVE GUARD ---"
tlog "overlay_tree=[${OVERLAY_LS//$'\n'/,}] rebuild_tree=[${REBUILD_LS//$'\n'/,}]"
if echo "$OVERLAY_LS" | grep -q 'issues/2.md' && ! echo "$REBUILD_LS" | grep -q 'issues/2.md'; then
  pass "DELETION NEGATIVE GUARD: pre-fix overlay (\`from\`+\`M\`, NO deleteall) RESURRECTS issues/2.md; post-fix rebuild (\`from\`+\`deleteall\`+\`M\`) DROPS it — the Scenario-C deletion assert provably bites (CR-01)"
else
  fail "DELETION NEGATIVE GUARD did not distinguish overlay-vs-rebuild (overlay=[${OVERLAY_LS//$'\n'/ }] rebuild=[${REBUILD_LS//$'\n'/ }]) — the deletion assert cannot detect the CR-01 regression it guards"
  finish 1
fi

# ── helper: init a fresh clone with its own cache, checkout main ────────────
init_clone() {  # $1=name  $2=cachevar-out
  local name="$1"
  local cache="${RUN_DIR}/cache_${name}"
  REPOSIX_CACHE_DIR="$cache" "${BIN_DIR}/reposix" init sim::demo "${RUN_DIR}/${name}" >"${RUN_DIR}/init_${name}.log" 2>&1
  ( cd "${RUN_DIR}/${name}" && git checkout -q -B main refs/reposix/origin/main )
  echo "$cache"
}

issue_version() {  # $1=id → prints SoT version int
  curl -s "${SIM_URL}/projects/demo/issues/$1" \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['version'])" 2>/dev/null || echo "-1"
}

# ============================================================================
# SCENARIO A — peer git-push drift.
# A edits issue1 + pushes (SoT drifts). B holds an unpushed local commit on a
# DIFFERENT record (issue2, so the rebase is a clean replay, isolating the
# fetch-recovery mechanism from merge-conflict resolution). B runs the DOCUMENTED
# single command `git pull --rebase && git push`. Expected: exit 0, no fatal
# fast-import, B's issue2 edit reaches the SoT (version 1 → 2).
# ============================================================================
echo "rebase-recovery-reconciles: SCENARIO A (peer git-push drift)" >&2
CACHE_A="$(init_clone A)"
CACHE_B="$(init_clone B)"
( cd "${RUN_DIR}/A" && printf '\nEdit by A\n' >> issues/1.md && git add -A && git commit -q -m "A edits issue1" \
    && REPOSIX_CACHE_DIR="$CACHE_A" git push -q origin main ) >"${RUN_DIR}/A_push.log" 2>&1
sleep 2
( cd "${RUN_DIR}/B" && printf '\nEdit by B\n' >> issues/2.md && git add -A && git commit -q -m "B edits issue2" )
A_BEFORE="$(issue_version 2)"
( cd "${RUN_DIR}/B" && REPOSIX_CACHE_DIR="$CACHE_B" git pull --rebase origin main \
    && REPOSIX_CACHE_DIR="$CACHE_B" git push origin main ) >"${RUN_DIR}/A_recovery.log" 2>&1
A_RECOVERY_EXIT=$?
A_AFTER="$(issue_version 2)"
tlog "--- SCENARIO A recovery.log ---"; tlog "$(cat "${RUN_DIR}/A_recovery.log")"
tlog "SCENARIO A: recovery_exit=${A_RECOVERY_EXIT} issue2_before=${A_BEFORE} issue2_after=${A_AFTER}"

# FIX-LAYER assertion (the P105 fix's proven win): the fatal fast-import abort
# is GONE regardless of the residual ref-lock bug.
if grep -qE "$FATAL_RE" "${RUN_DIR}/A_recovery.log"; then
  fail "SCENARIO A: \`fatal: error while running fast-import\` / \`does not contain\` STILL present — the P105 parent-chaining fix regressed"
else
  pass "SCENARIO A FIX-LAYER: no \`fatal: error while running fast-import\` / \`does not contain\` on the recovery path (P105 parent-chaining fix holds)"
fi

# CONVERGENCE assertion (the documented recovery must actually work). Assert the
# edit LANDED — issue2's SoT version incremented by exactly 1 (before+1) — rather
# than a hardcoded absolute: both scenarios share one sim SoT, so issue2's base
# version differs per scenario (Scenario A leaves it at 2, so Scenario B's base
# is 2 → 3). before+1 is the drift-source-agnostic convergence signal.
A_EXPECT=$(( A_BEFORE + 1 ))
if [[ "$A_RECOVERY_EXIT" -eq 0 && "$A_AFTER" == "$A_EXPECT" ]]; then
  pass "SCENARIO A (peer git-push drift): documented \`git pull --rebase && git push\` exits 0 and B's edit converged (issue2 v${A_BEFORE}→v${A_AFTER})"
else
  BLOCKED=1
  REASON="RBF-LR-03 second bug: fetch-time double-write of refs/reposix/origin/main → \`cannot lock ref\` → \`git pull --rebase\` exits non-zero → documented \`git pull --rebase && git push\` short-circuits. Filed HIGH: .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md (P105). Repro: .planning/phases/105-rbf-lr-03-rebase-recovery/repro/repro-fetch-ref-lock.sh"
  if grep -qE "$REFLOCK_RE" "${RUN_DIR}/A_recovery.log"; then
    fail "SCENARIO A: documented recovery did NOT converge — \`cannot lock ref 'refs/reposix/origin/main'\` (recovery_exit=${A_RECOVERY_EXIT}, issue2 v${A_BEFORE}→v${A_AFTER}); RBF-LR-03 second bug, filed HIGH"
  else
    fail "SCENARIO A: documented recovery did NOT converge (recovery_exit=${A_RECOVERY_EXIT}, issue2 v${A_BEFORE}→v${A_AFTER})"
  fi
fi

# LAYER-2 negative guard: `cannot lock ref 'refs/reposix/origin/main'` MUST NOT
# appear on the recovery path (the pre-layer-2 double-writer bug). Absence is
# the positive signal the two-namespace split holds.
if grep -qE "$REFLOCK_RE" "${RUN_DIR}/A_recovery.log"; then
  fail "SCENARIO A LAYER-2: \`cannot lock ref 'refs/reposix/origin/main'\` present — the import stream is STILL double-writing the tracking ref git owns (namespace-collapse regression)"
else
  pass "SCENARIO A LAYER-2: no \`cannot lock ref 'refs/reposix/origin/main'\` on the recovery path — helper writes the private refs/reposix-import/* ns, git fetch is the sole tracking-ref writer"
fi

# CLOBBER guard: the caller's local refs/heads/main must have moved ONLY via the
# user's own rebase/commit (its tip is reachable-from the local HEAD the user
# built), and the private staging ref refs/reposix-import/main must exist —
# never refs/heads/* written by the helper stream.
A_PRIV="$( cd "${RUN_DIR}/B" && git rev-parse --verify --quiet refs/reposix-import/main 2>/dev/null )"
A_LOCAL_SUBJECT="$( cd "${RUN_DIR}/B" && git log -1 --format=%s refs/heads/main 2>/dev/null )"
tlog "SCENARIO A clobber: refs/reposix-import/main=${A_PRIV:-<absent>} local_head_subject='${A_LOCAL_SUBJECT}'"
if [[ -n "$A_PRIV" && "$A_LOCAL_SUBJECT" == "B edits issue2" ]]; then
  pass "SCENARIO A CLOBBER-GUARD: private refs/reposix-import/main present and local refs/heads/main tip is the user's own commit (\`B edits issue2\`) — the helper never clobbered the working branch"
else
  fail "SCENARIO A CLOBBER-GUARD: expected private refs/reposix-import/main present + local HEAD subject 'B edits issue2'; got priv='${A_PRIV:-<absent>}' subject='${A_LOCAL_SUBJECT}'"
fi

# IMPORT-CHAIN assert (IN-01 → maps catalog expected.asserts[6]): emit_import_stream
# must chain the synth commit via `from <parent>` (NOT re-mint a parentless root) and
# write ONLY the private refs/reposix-import/* ns. A parentless re-mint leaves the
# private ref a 1-commit root; a `from`-chained fetch after drift advances it to a
# >=2-commit linear chain. Combined with the LAYER-2 guard (no refs/reposix/origin/main
# write) this gives assert #7 a concrete 1:1 shell assertion.
A_IMPORT_COUNT="$( cd "${RUN_DIR}/B" && git rev-list --count refs/reposix-import/main 2>/dev/null || echo 0 )"
tlog "SCENARIO A import-chain: refs/reposix-import/main rev-list count=${A_IMPORT_COUNT}"
if [[ -n "$A_PRIV" && "${A_IMPORT_COUNT:-0}" -ge 2 ]]; then
  pass "SCENARIO A IMPORT-CHAIN (assert #7): emit_import_stream chained via \`from <parent>\` — refs/reposix-import/main is a ${A_IMPORT_COUNT}-commit linear chain (a parentless re-mint would be 1), written to the private refs/reposix-import/* ns only, never refs/reposix/origin/main"
else
  fail "SCENARIO A IMPORT-CHAIN (assert #7): expected refs/reposix-import/main to be a >=2-commit \`from\`-chained history; got count='${A_IMPORT_COUNT}' priv='${A_PRIV:-<absent>}'"
fi

# ============================================================================
# SCENARIO B — external REST PATCH drift.
# C holds an unpushed local commit (issue2). A direct `curl -X PATCH` moves the
# SoT (issue1) — NOT a git push. C runs the DOCUMENTED recovery. Same expected
# outcome as Scenario A.
# ============================================================================
echo "rebase-recovery-reconciles: SCENARIO B (external REST PATCH drift)" >&2
CACHE_C="$(init_clone C)"
( cd "${RUN_DIR}/C" && printf '\nEdit by C\n' >> issues/2.md && git add -A && git commit -q -m "C edits issue2" )
tlog "--- SCENARIO B external PATCH issue1 ---"
curl -s -X PATCH "${SIM_URL}/projects/demo/issues/1" \
  -H 'content-type: application/json' \
  -d '{"body":"EXTERNALLY EDITED VIA REST\n"}' >>"$TRANSCRIPT" 2>&1
tlog ""
sleep 2
B_BEFORE="$(issue_version 2)"
( cd "${RUN_DIR}/C" && REPOSIX_CACHE_DIR="$CACHE_C" git pull --rebase origin main \
    && REPOSIX_CACHE_DIR="$CACHE_C" git push origin main ) >"${RUN_DIR}/B_recovery.log" 2>&1
B_RECOVERY_EXIT=$?
B_AFTER="$(issue_version 2)"
tlog "--- SCENARIO B recovery.log ---"; tlog "$(cat "${RUN_DIR}/B_recovery.log")"
tlog "SCENARIO B: recovery_exit=${B_RECOVERY_EXIT} issue2_before=${B_BEFORE} issue2_after=${B_AFTER}"

if grep -qE "$FATAL_RE" "${RUN_DIR}/B_recovery.log"; then
  fail "SCENARIO B: \`fatal: error while running fast-import\` / \`does not contain\` STILL present on external-REST-drift recovery"
else
  pass "SCENARIO B FIX-LAYER: no \`fatal: error while running fast-import\` / \`does not contain\` on the external-REST-drift recovery path"
fi

B_EXPECT=$(( B_BEFORE + 1 ))
if [[ "$B_RECOVERY_EXIT" -eq 0 && "$B_AFTER" == "$B_EXPECT" ]]; then
  pass "SCENARIO B (external REST PATCH drift): documented \`git pull --rebase && git push\` exits 0 and C's edit converged (issue2 v${B_BEFORE}→v${B_AFTER})"
else
  BLOCKED=1
  [[ -z "$REASON" ]] && REASON="RBF-LR-03 second bug (see SURPRISES-INTAKE)"
  if grep -qE "$REFLOCK_RE" "${RUN_DIR}/B_recovery.log"; then
    fail "SCENARIO B: documented recovery did NOT converge — \`cannot lock ref 'refs/reposix/origin/main'\` (recovery_exit=${B_RECOVERY_EXIT}, issue2 v${B_BEFORE}→v${B_AFTER}); RBF-LR-03 second bug, filed HIGH"
  else
    fail "SCENARIO B: documented recovery did NOT converge (recovery_exit=${B_RECOVERY_EXIT}, issue2 v${B_BEFORE}→v${B_AFTER})"
  fi
fi

# LAYER-2 negative guard (external-REST-drift variant).
if grep -qE "$REFLOCK_RE" "${RUN_DIR}/B_recovery.log"; then
  fail "SCENARIO B LAYER-2: \`cannot lock ref 'refs/reposix/origin/main'\` present on the external-REST-drift recovery path (namespace-collapse regression)"
else
  pass "SCENARIO B LAYER-2: no \`cannot lock ref 'refs/reposix/origin/main'\` on the external-REST-drift recovery path"
fi

# CLOBBER guard (Scenario B / clone C).
B_PRIV="$( cd "${RUN_DIR}/C" && git rev-parse --verify --quiet refs/reposix-import/main 2>/dev/null )"
B_LOCAL_SUBJECT="$( cd "${RUN_DIR}/C" && git log -1 --format=%s refs/heads/main 2>/dev/null )"
tlog "SCENARIO B clobber: refs/reposix-import/main=${B_PRIV:-<absent>} local_head_subject='${B_LOCAL_SUBJECT}'"
if [[ -n "$B_PRIV" && "$B_LOCAL_SUBJECT" == "C edits issue2" ]]; then
  pass "SCENARIO B CLOBBER-GUARD: private refs/reposix-import/main present and local refs/heads/main tip is the user's own commit (\`C edits issue2\`) — the helper never clobbered the working branch"
else
  fail "SCENARIO B CLOBBER-GUARD: expected private refs/reposix-import/main present + local HEAD subject 'C edits issue2'; got priv='${B_PRIV:-<absent>}' subject='${B_LOCAL_SUBJECT}'"
fi

# ============================================================================
# SCENARIO C — record DELETED at the SoT (CR-01 regression coverage / WR-01).
# D holds an unpushed local commit on issue1. issue2 is DELETED at the SoT via
# REST DELETE (204). D runs the DOCUMENTED recovery. Expected: exit 0, the
# deletion PROPAGATES (issues/2.md gone from D's working tree after the rebase),
# and the push does NOT resurrect issue2 (SoT stays 404). Pre-`deleteall` this
# RED-s: the overlay tree retains issues/2.md and the push re-creates it.
# ============================================================================
echo "rebase-recovery-reconciles: SCENARIO C (record deleted at SoT must not resurrect)" >&2
CACHE_D="$(init_clone D)"
if [[ ! -f "${RUN_DIR}/D/issues/2.md" ]]; then
  fail "SCENARIO C precondition: issues/2.md must exist in clone D before deletion; got $(ls "${RUN_DIR}/D/issues" 2>/dev/null | tr '\n' ' ')"
  finish 1
fi
( cd "${RUN_DIR}/D" && printf '\nEdit by D\n' >> issues/1.md && git add -A && git commit -q -m "D edits issue1" )
DEL_CODE="$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "${SIM_URL}/projects/demo/issues/2")"
tlog "SCENARIO C: DELETE issue2 → HTTP ${DEL_CODE}"
sleep 2
( cd "${RUN_DIR}/D" && REPOSIX_CACHE_DIR="$CACHE_D" git pull --rebase origin main \
    && REPOSIX_CACHE_DIR="$CACHE_D" git push origin main ) >"${RUN_DIR}/C_recovery.log" 2>&1
C_RECOVERY_EXIT=$?
C_HAS_FILE=no; [[ -f "${RUN_DIR}/D/issues/2.md" ]] && C_HAS_FILE=yes
C_ISSUE2_SOT="$(issue_version 2)"   # -1 == 404 → deleted, not resurrected
tlog "--- SCENARIO C recovery.log ---"; tlog "$(cat "${RUN_DIR}/C_recovery.log")"
tlog "SCENARIO C: recovery_exit=${C_RECOVERY_EXIT} worktree_has_issue2=${C_HAS_FILE} sot_issue2_version=${C_ISSUE2_SOT} delete_http=${DEL_CODE}"

if grep -qE "$FATAL_RE" "${RUN_DIR}/C_recovery.log"; then
  fail "SCENARIO C: \`fatal: error while running fast-import\` / \`does not contain\` on the deletion recovery path"
else
  pass "SCENARIO C FIX-LAYER: no \`fatal: error while running fast-import\` / \`does not contain\` on the deletion recovery path"
fi

if [[ "$C_RECOVERY_EXIT" -eq 0 && "$C_HAS_FILE" == "no" && "$C_ISSUE2_SOT" == "-1" ]]; then
  pass "SCENARIO C (record deleted at SoT): documented \`git pull --rebase && git push\` exits 0, the deletion PROPAGATED (issues/2.md gone from the working tree after rebase) and the push did NOT resurrect issue2 (SoT 404) — CR-01 deleteall full-rebuild holds"
else
  BLOCKED=1
  [[ -z "$REASON" ]] && REASON="CR-01: a record deleted at the SoT survived \`git pull --rebase && git push\` (missing deleteall → overlay tree retains and resurrects the deleted record)."
  fail "SCENARIO C: deletion did NOT propagate cleanly (recovery_exit=${C_RECOVERY_EXIT}, worktree_has_issue2=${C_HAS_FILE}, sot_issue2_version=${C_ISSUE2_SOT}); expected exit 0 + issues/2.md gone + SoT 404 — CR-01 regression (overlay tree resurrects the deleted record)"
fi

# ============================================================================
# STATELESS-CONNECT (PLAN §5 open question) — modern-git (>= 2.34) path.
# Only git 2.25.1 is installed here; forcing protocol.version=2 on 2.25 errors
# before the fetch (`bad line length 2`). We therefore CANNOT verify whether the
# stateless-connect fetch path also breaks. Record a labelled skip + TODO rather
# than fake a result (charter: verify against reality, never fake).
# ============================================================================
GIT_VER="$(git --version | grep -oE '[0-9]+\.[0-9]+' | head -1)"
GIT_MAJ="${GIT_VER%%.*}"; GIT_MIN="${GIT_VER#*.}"
if [[ "$GIT_MAJ" -gt 2 || ( "$GIT_MAJ" -eq 2 && "$GIT_MIN" -ge 34 ) ]]; then
  tlog "STATELESS-CONNECT: modern git (${GIT_VER}) present — TODO: re-run both scenarios WITHOUT the protocol.version=0 forcing to exercise the stateless-connect path and record whether it also breaks (PLAN §5). If it also breaks, that is a SEPARATE cache-side fix site — file to SURPRISES-INTAKE, do NOT expand this gate."
  echo "  NOTE: modern git present — stateless-connect §5 check is a TODO in this gate (see transcript)." >&2
else
  tlog "STATELESS-CONNECT: SKIPPED — git ${GIT_VER} < 2.34; protocol.version=2 errors on 2.25 before the fetch (bad line length 2). PLAN §5 open question UNRESOLVED in this environment; needs a modern-git CI run. Not faked."
fi

# ============================================================================
# Final verdict.
# ============================================================================
if [[ "$BLOCKED" -eq 1 ]]; then
  echo "rebase-recovery-reconciles: NOT-VERIFIED — the documented single-command recovery does not converge (RBF-LR-03 second bug, filed HIGH). See ${TRANSCRIPT}." >&2
  finish 75
fi
echo "rebase-recovery-reconciles: PASS — both drift scenarios recover via the documented \`git pull --rebase && git push\`." >&2
finish 0
