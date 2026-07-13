#!/usr/bin/env bash
# quality/gates/agent-ux/lib/litmus-flow.sh — sourced-only helper for
# milestone-close-vision-litmus.sh (RBF-FW-03). Factored out under the 10k
# .sh file-size budget, mirroring dark-factory/reconciliation-fixture.sh.
#
# Provides:
#   _litmus_flow            — the STEP 3-6 vanilla-clone+attach+edit+push
#                             round-trip; wrapped by write_transcript_and_artifact
#                             so its stdout/stderr become the transcript.
#   patch_litmus_artifact   — rewrites asserts_passed/failed into the artifact
#                             the transcript lib emitted (F-K4b congruence).
#
# Caller MUST export before sourcing/invoking: BIN, SPACE, MIRROR_URL,
# PROTECTED_IDS, STATE_FILE, REPO_ROOT. `pass`/`fail` are defined by the
# caller. Runs with errexit OFF (the transcript lib toggles set +e), so
# every step checks exit status explicitly and returns 0/1/2.

_litmus_flow() {
  local run cache tree recon md id target marker cache_bare audit_db
  local pclone rurl summary matched bdel cache_rows core_rows
  run="$(mktemp -d -t litmus-run.XXXXXX)"; cache="${run}/cache"; tree="${run}/tree"
  export REPOSIX_CACHE_DIR="$cache"
  cache_bare="${cache}/reposix/confluence-${SPACE}.git"
  audit_db="${cache}/reposix/confluence-${SPACE}.audit.db"

  echo "\$ git clone <mirror> ${tree}"
  git clone --quiet "$MIRROR_URL" "$tree" || { fail "vanilla git clone of GH mirror failed"; return 1; }
  pass "vanilla clone obtained"

  echo "\$ reposix attach confluence::${SPACE} --remote-name reposix --mirror-name origin"
  recon="$("$BIN" attach "confluence::${SPACE}" "$tree" --remote-name reposix --mirror-name origin 2>&1)"
  echo "$recon"
  pclone="$(git -C "$tree" config --get extensions.partialClone || true)"
  rurl="$(git -C "$tree" config --get remote.reposix.url || true)"
  if [ "$pclone" = "reposix" ] && [[ "$rurl" == reposix::* ]]; then
    pass "reposix attach modified git config (extensions.partialClone=reposix, remote.reposix.url present)"
  else
    fail "attach did not set expected git config (partialClone='$pclone')"; return 1
  fi

  # GUARD A: reconciliation must show real overlap, zero backend_deleted —
  # else a push would delete records that DO exist on the backend.
  summary="$(printf '%s\n' "$recon" | grep -oE 'matched=[0-9]+ no_id=[0-9]+ backend_deleted=[0-9]+ mirror_lag=[0-9]+' | tail -1)"
  matched="$(printf '%s' "$summary" | sed -n 's/.*matched=\([0-9]*\).*/\1/p')"
  bdel="$(printf '%s' "$summary" | sed -n 's/.*backend_deleted=\([0-9]*\).*/\1/p')"
  echo "reconciliation: ${summary:-<none>}"
  if [ -z "$summary" ] || [ "${matched:-0}" -lt 1 ] || [ "${bdel:-1}" -ne 0 ]; then
    fail "reconciliation unsafe/non-overlapping (matched=${matched:-?} backend_deleted=${bdel:-?}). Refusing to push a delete-shaped diff (mass-delete guard). HIGH friction: documented happy path disagrees with binary — hard RED (OD-2)."
    return 1
  fi
  pass "reconciliation safe: matched=${matched} backend_deleted=0 (push cannot mass-delete)"

  # GUARD B (Wave-5.5 update): confluence records live under the pages/
  # bucket — the canonical confluence spelling per bucket_for_backend
  # (reposix-core), refresh output, and docs/reference/confluence.md. The
  # push planner is now id-keyed (bucket-agnostic), so a bucket mismatch can
  # no longer mass-delete; this guard remains as defense-in-depth asserting
  # the mirror substrate is in the documented confluence shape.
  if ! compgen -G "${tree}/pages/*.md" > /dev/null; then
    fail "working tree carries zero pages/*.md records — the confluence bucket is pages/ (bucket_for_backend, Wave-5.5). Substrate not litmus-ready: repopulate the mirror via reposix refresh --backend confluence."
    return 1
  fi

  # Pick an editable, NON-protected record from the confluence bucket.
  target=""
  for md in "$tree"/pages/*.md; do
    [ -e "$md" ] || continue
    id="$(basename "$md" .md)"
    case "$PROTECTED_IDS" in *" $id "*) continue ;; esac
    target="$md"; break
  done
  [ -z "$target" ] && { fail "no editable non-protected pages/<id>.md record in the clone"; return 1; }
  echo "edit target: $target (protected denylist honoured)"

  marker="litmus-marker-$(date -u +%s)"
  # VISIBLE text, never an HTML comment: Confluence's storage sanitizer
  # strips comments server-side, so a comment-only edit is a content no-op —
  # the PUT returns 200 WITHOUT minting a new version and the REST confirm
  # below can never see the marker (observed against real TokenWorld,
  # transcript 2026-07-04T21-36-37Z). Drop any prior run's marker line first
  # so the sanctioned page carries at most one marker instead of accumulating.
  sed -i '/litmus-marker-/d' "$target"
  printf '\n%s\n' "$marker" >> "$target"
  git -C "$tree" config user.email "litmus@reposix.invalid"
  git -C "$tree" config user.name "reposix-litmus"
  git -C "$tree" add -A && git -C "$tree" commit --quiet -m "litmus edit ${marker}"
  pass "edit + commit succeeded"

  # Out-of-band SoT drift self-heal (manager decision D1, aligns GTH-V15-09).
  # The `origin` mirror can lag the backend (a peer push / direct REST edit
  # advanced the SoT after our last mirror sync), so our push carries a stale
  # base and the helper rejects it with git's "fetch first". On rejection run
  # the ONE documented recovery — `git pull --rebase && git push` from
  # docs/guides/troubleshooting.md § "DVCS push/pull issues" — against the BUS
  # remote `reposix` (backend-current SoT), NEVER `origin` (the stale mirror;
  # after attach `branch.<b>.remote` still points at origin for fetch, so the
  # recovery is remote-explicit on purpose). BOUNDED: exactly one
  # fetch-rebase-retry; a rejection persisting past it is a REAL coherence bug
  # that MUST surface red — we never loop or swallow it.
  echo "\$ git push reposix main"
  if git -C "$tree" push reposix main; then
    pass "git push reposix main succeeded (real helper round-trip, not a synthetic stream)"
  else
    echo "↻ push rejected — running documented recovery: git pull --rebase reposix main + retry (out-of-band SoT drift; troubleshooting.md § DVCS push/pull)" >&2
    if git -C "$tree" pull --rebase reposix main && git -C "$tree" push reposix main; then
      pass "git push reposix main succeeded after the documented recovery (fetch reposix + rebase + retry self-healed out-of-band SoT drift)"
    else
      fail "git push reposix main still rejected after the ONE documented recovery (git pull --rebase reposix main + retry) — a rejection persisting past the bounded self-heal is a REAL coherence bug, not mirror lag. Recover manually: cd \"$tree\" && git pull --rebase reposix main && git push reposix main."
      return 1
    fi
  fi

  # STEP 4 box 5: server-side confirm via REST — a DIRECT Confluence read,
  # independent of every reposix code path. (`reposix list` is deliberately
  # lazy: list_records returns metadata with EMPTY bodies, so grepping its
  # JSON for the marker can never succeed — observed litmus run 5,
  # 2026-07-04. The direct read is also the more honest probe: it cannot
  # share a bug with the code under test.)
  id="$(basename "$target" .md)"
  if curl -sS -u "${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}" \
      "https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/wiki/api/v2/pages/${id}?body-format=storage" \
      | grep -qF "$marker"; then
    pass "server-side change confirmed via REST (page ${id} body carries the marker)"
  else
    fail "REST read does NOT show the pushed edit — push acked but SoT unchanged (documented happy path disagrees with binary). HIGH friction, hard RED (OD-2)."
    return 1
  fi

  # STEP 5: dual-table audit assertion (OP-3).
  cache_rows="$(sqlite3 "$cache_bare/cache.db" "SELECT COUNT(*) FROM audit_events_cache WHERE op LIKE 'helper_push%' OR op='tree_sync';" 2>/dev/null || echo 0)"
  core_rows="$(sqlite3 "$audit_db" "SELECT COUNT(*) FROM audit_events WHERE method IN ('PUT','POST','DELETE') AND path LIKE '%/pages%';" 2>/dev/null || echo 0)"
  if [ "${cache_rows:-0}" -ge 1 ] && [ "${core_rows:-0}" -ge 1 ]; then
    pass "dual-table audit: audit_events_cache ($cache_rows) AND audit_events ($core_rows) both have rows for the action"
  else
    fail "dual-table audit incomplete (cache=$cache_rows core=$core_rows) — OP-3 requires BOTH"; return 1
  fi

  # refs/mirrors advancement (mirror-lag contract).
  if git -C "$cache_bare" for-each-ref 'refs/mirrors/*' | grep -q 'refs/mirrors/'; then
    pass "refs/mirrors/<sot>-{head,synced-at} advanced"
  else
    fail "refs/mirrors/<sot>-{head,synced-at} did NOT advance after the bus push"; return 1
  fi

  # STEP 8 cleanup: this flow edits an existing matched record (no page
  # creation), so nothing to sweep; the protected denylist guarantees
  # 7766017/7798785 are never touched.
  return 0
}

# Rewrite asserts_passed/failed into the transcript-lib artifact so grade-time
# F-K4b congruence holds (every expected assert token-matches a passed entry).
patch_litmus_artifact() {
  local artifact="$1" state="$2" rc="$3"
  python3 - "$artifact" "$state" "$rc" <<'PY'
import json, sys
art, state, rc = sys.argv[1], sys.argv[2], int(sys.argv[3])
passed, failed = [], []
for line in open(state):
    line = line.rstrip("\n")
    if line.startswith("PASS:"): passed.append(line[5:])
    elif line.startswith("FAIL:"): failed.append(line[5:])
passed.append("transcript artifact emitted per RBF-FW-02 convention")
try:
    d = json.load(open(art))
except Exception:
    d = {"row_id": "agent-ux/milestone-close-vision-litmus-real-backend", "exit_code": rc}
d["asserts_passed"] = passed
d["asserts_failed"] = failed
d["exit_code"] = rc
d["status"] = {0: "PASS", 2: "PARTIAL", 75: "NOT-VERIFIED"}.get(rc, "FAIL")
json.dump(d, open(art, "w"), indent=0)
PY
}
