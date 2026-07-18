#!/usr/bin/env bash
# quality/gates/agent-ux/lib/t4-real-backend-flow.sh -- sourced-only helper
# for t4-conflict-rebase-ancestry-real-backend.sh (B4, P0, quick 260712-phc).
# Factored out under the 10k .sh file-size budget (quality/CLAUDE.md "File-size
# limits"), mirroring the dark-factory/lib.sh + lib/litmus-flow.sh precedent:
# a sourced-only helper the caller invokes AFTER defining hard_fail_exit() /
# note_pass() and clearing the env-gate / sanctioned-target / git-version /
# cargo-build preconditions.
#
# Caller MUST leave these in scope before sourcing + calling:
#   BIN_DIR, SPACE, PROTECTED_IDS, hard_fail_exit(), note_pass()
#
# Provides:
#   _t4_real_backend_flow        -- two-cache bootstrap + bucket-aware
#                                    record select + two-writer conflict /
#                                    refetch-ancestry scenario body.
#   _t4_assert_safe_push_diff    -- MASS-DELETE GUARD: refuses (hard exit 1,
#                                    never pushes) any pending-push diff that
#                                    deletes a file or touches a protected
#                                    fixture id.
#
# See the header of t4-conflict-rebase-ancestry-real-backend.sh for the full
# rationale (confluence pages/ bucket, mass-delete guard provenance, HIGH-1
# regression this scenario locks).

_t4_checkout_or_fail() {  # _t4_checkout_or_fail <tree> <label>
  # SC5b (DRAIN-01): surfaces the REAL captured git stderr as the failure
  # detail instead of a hardcoded "requires git >= 2.34 stateless-connect
  # fetch" string, which is misleading BY CONSTRUCTION at this call site --
  # the caller (t4-conflict-rebase-ancestry-real-backend.sh, lines ~179-193)
  # already verified `git --version >= 2.34` before this function ever runs,
  # so any failure reaching here is NEVER a version problem. Most commonly
  # the Confluence list-vs-get oid-drift class documented in
  # .planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/
  # 114-VERIFICATION.md. `2>&1 >/dev/null` duplicates stderr onto the command
  # substitution's capture pipe FIRST, then redirects stdout to /dev/null --
  # so $err holds ONLY the real stderr, never the (irrelevant) checkout stdout.
  local tree="$1" label="$2" err
  err="$(git -C "$tree" checkout -B main refs/reposix/origin/main 2>&1 >/dev/null)" \
    || hard_fail_exit "git checkout -B main (${label}) failed" "$err"
}

_t4_real_backend_flow() {
  local run_dir a b cache_a cache_b root_b page_file_a page_basename page_file_b
  local b_push1_log b_push1_exit new_root_b commit_count_after md id

  run_dir="/tmp/t4-conflict-rebase-ancestry-real-backend-$$"
  mkdir -p "$run_dir"
  a="${run_dir}/A"; b="${run_dir}/B"
  cache_a="${run_dir}/cacheA"; cache_b="${run_dir}/cacheB"

  echo "t4-conflict-rebase-ancestry-real-backend: reposix init A (own cache) against confluence::${SPACE}" >&2
  REPOSIX_CACHE_DIR="$cache_a" "${BIN_DIR}/reposix" init "confluence::${SPACE}" "$a" \
    || hard_fail_exit "reposix init A (confluence::${SPACE}) failed"
  git -C "$a" config user.email "writer-a-confluence@example.invalid"
  git -C "$a" config user.name "writer-A-confluence"

  echo "t4-conflict-rebase-ancestry-real-backend: reposix init B (own cache) against confluence::${SPACE}" >&2
  REPOSIX_CACHE_DIR="$cache_b" "${BIN_DIR}/reposix" init "confluence::${SPACE}" "$b" \
    || hard_fail_exit "reposix init B (confluence::${SPACE}) failed"
  git -C "$b" config user.email "writer-b-confluence@example.invalid"
  git -C "$b" config user.name "writer-B-confluence"

  _t4_checkout_or_fail "$a" "A"
  _t4_checkout_or_fail "$b" "B"
  note_pass "two independent working trees against TokenWorld: A and B each bootstrapped via reposix init confluence::${SPACE} with SEPARATE REPOSIX_CACHE_DIR caches (cacheA/cacheB)"

  # ROOT_B captured right after checkout, BEFORE any edits -- the ancestry
  # baseline the refetch below must not disturb.
  root_b="$(git -C "$b" rev-list --max-parents=0 refs/heads/main | tail -1)"

  # Bucket-aware record path: confluence == pages/, NOT issues/. Honor the
  # protected-fixture denylist DURING selection (mirrors lib/litmus-flow.sh's
  # target-picking loop), never as a post-hoc check only.
  page_file_a=""
  for md in "$a"/pages/*.md; do
    [ -e "$md" ] || continue
    id="$(basename "$md" .md)"
    case "$PROTECTED_IDS" in *" $id "*) continue ;; esac
    page_file_a="$md"
    break
  done
  [ -n "$page_file_a" ] || hard_fail_exit "no editable non-protected pages/<id>.md record found in A's checkout" "$a/pages (bucket_for_backend(confluence)==pages, never issues)"
  page_basename="$(basename "$page_file_a")"
  page_file_b="${b}/pages/${page_basename}"

  # The backend's `updated_at` cursor comparison needs A's edit to land in a
  # strictly later wall-clock second than B's `last_fetched_at` cursor, or
  # the conflict-detection precheck can collide at whatever timestamp
  # precision the backend stores (verbatim rationale from the sim-arm sibling).
  sleep 2

  echo "t4-conflict-rebase-ancestry-real-backend: A edits + pushes (baseline)" >&2
  { echo ""; echo "A-edit-real-backend-$(date -u +%s)"; } >> "$page_file_a"
  git -C "$a" add "pages/${page_basename}"
  git -C "$a" commit --quiet -m "A: edit ${page_basename} (real-backend conflict/ancestry probe)"
  _t4_assert_safe_push_diff "$a" "A baseline"
  REPOSIX_CACHE_DIR="$cache_a" git -C "$a" push origin main \
    || hard_fail_exit "A's baseline push failed" "expected a clean single-writer push to succeed against real TokenWorld"
  note_pass "A's baseline push (no conflict) against real TokenWorld succeeded"

  echo "t4-conflict-rebase-ancestry-real-backend: B edits the SAME record (stale base) + pushes (expect rejection)" >&2
  { echo ""; echo "B-edit-real-backend-$(date -u +%s)"; } >> "$page_file_b"
  git -C "$b" add "pages/${page_basename}"
  git -C "$b" commit --quiet -m "B: edit ${page_basename} (real-backend conflict/ancestry probe)"
  _t4_assert_safe_push_diff "$b" "B stale-base"

  b_push1_log="${run_dir}/b-push1.log"
  set +e
  REPOSIX_CACHE_DIR="$cache_b" git -C "$b" push origin main > "$b_push1_log" 2>&1
  b_push1_exit=$?
  set -e
  if [[ "$b_push1_exit" -eq 0 ]]; then
    hard_fail_exit "B's stale-base push should have been REJECTED but exited 0" "$(cat "$b_push1_log")"
  fi
  grep -qE 'version mismatch|fetch first' "$b_push1_log" \
    || hard_fail_exit "B's rejected push did not name the expected conflict (version mismatch / fetch first)" "$(cat "$b_push1_log")"
  note_pass "two independent working trees against TokenWorld: B's stale-base push against the SAME record A just pushed was correctly rejected (version mismatch / fetch first) -- conflict reject proven"

  echo "t4-conflict-rebase-ancestry-real-backend: B recovers via git fetch origin" >&2
  REPOSIX_CACHE_DIR="$cache_b" git -C "$b" fetch origin \
    || hard_fail_exit "B's recovery git fetch origin failed" "this is the exact HIGH-1 regression path -- a refetch after a rejected push must succeed"

  new_root_b="$(git -C "$b" rev-list --max-parents=0 refs/reposix/origin/main | tail -1)"
  [[ "$new_root_b" == "$root_b" ]] \
    || hard_fail_exit "HIGH-1 REGRESSED: refetch produced a NEW root commit (no ancestry to the prior tip)" "ROOT_B(before)=$root_b ROOT_B(after)=$new_root_b"
  note_pass "B's recovery refetch produced no fresh root on refetch -- refs/reposix/origin/main's root commit stayed IDENTICAL before and after the refetch against real TokenWorld (no fresh disconnected root, HIGH-1 stays fixed)"

  commit_count_after="$(git -C "$b" rev-list --count refs/reposix/origin/main)"
  [[ "$commit_count_after" -gt 1 ]] \
    || hard_fail_exit "refetch did not advance refs/reposix/origin/main at all" "commit count = $commit_count_after"
  note_pass "refs/reposix/origin/main genuinely advanced past the shared root (commit count=${commit_count_after}) -- the ancestry assertion above is not vacuous, matching the sim arm's non-vacuous check"

  # This flow only edits an existing matched page in place -- nothing to sweep.
  rm -rf "$run_dir"

  echo "T4-CONFLICT-REBASE-ANCESTRY-REAL-BACKEND COMPLETE -- two-writer conflict correctly rejected against real TokenWorld; recovery refetch preserves ancestry (no fresh root)." >&2
}

# MASS-DELETE GUARD: refuse any delete-shaped diff before EITHER push. See
# t4-conflict-rebase-ancestry-real-backend.sh's header for the full
# rationale + the Wave-5.5 NOTICING (push planner now id-keyed / bucket
# agnostic; this guard is defense-in-depth, not a known-live planner bug).
_t4_assert_safe_push_diff() {
  local tree="$1" label="$2" diff
  diff="$(git -C "$tree" diff --name-status HEAD~1 HEAD 2>&1)" \
    || hard_fail_exit "${label}: could not compute the pending-push diff" "$diff"
  echo "${label} pending-push diff (HEAD~1..HEAD):" >&2
  printf '%s\n' "$diff" | sed 's/^/  /' >&2
  if printf '%s\n' "$diff" | grep -qE '^D'; then
    hard_fail_exit "MASS-DELETE GUARD: ${label} push would delete file(s) -- refusing to push a delete-shaped diff (confluence pages/ bucket safety, per milestone-close-vision-litmus.sh)" "$diff"
  fi
  while IFS=$'\t' read -r _status fname; do
    [ -z "$fname" ] && continue
    local base id
    base="$(basename "$fname")"
    id="${base%.md}"
    case "$PROTECTED_IDS" in
      *" $id "*) hard_fail_exit "MASS-DELETE GUARD: ${label} diff touches PROTECTED fixture id ${id}" "${fname} is on the never-edit denylist (${PROTECTED_IDS})" ;;
    esac
  done <<< "$diff"
  note_pass "${label} pending-push diff is a safe single-file in-place edit (no deletions, no protected fixture ids touched)"
}
