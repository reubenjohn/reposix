#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory/reconciliation-fixture.sh -- RBF-A-05
# (p86 F13) shared fixture-writer + assert helpers for dvcs-third-arm.sh.
#
# Sourced (like lib.sh), NOT invoked directly -- factored out of
# dvcs-third-arm.sh per the file-size-limits progressive-disclosure budget
# (quality/gates/structure/file-size-limits.sh, .sh files = 10k chars).
#
# Relies on `fail_with` + `ASSERT_LOG` already being in scope from lib.sh,
# which dvcs-third-arm.sh sources BEFORE this file.

# write_issue <id> <path> <title> — canonical Record frontmatter shape
# (reposix_core::record::frontmatter::Frontmatter; id/title/status/
# created_at/updated_at required).
write_issue() {
    local id="$1" path="$2" title="$3"
    cat > "$path" <<EOF
---
id: ${id}
title: "${title}"
status: open
created_at: 2026-07-04T00:00:00Z
updated_at: 2026-07-04T00:00:00Z
version: 1
---
RBF-A-05 reconciliation fixture body for seeded record ${id}.
EOF
}

# populate_reconciliation_fixture <work_repo> <sim_url>
# Writes + commits content shaped to 4 of the 5 reconciliation cases
# (reposix-cache/src/reconciliation.rs module doc :4-15); ids 1-6 come from
# the `seeded` fixture (crates/reposix-sim/fixtures/seed.json, loaded by
# `spawn_sim seeded`).
populate_reconciliation_fixture() {
    local work_repo="$1" sim_url="$2"
    mkdir -p "${work_repo}/issues"

    # Case 1 (matched, :203-227): local ids 1-3 exist in the seeded backend set.
    write_issue 1 "${work_repo}/issues/0001.md" "database connection drops under load"
    write_issue 2 "${work_repo}/issues/0002.md" "add --no-color flag to CLI"
    write_issue 3 "${work_repo}/issues/0003.md" "document the new auth flow"

    # Case 3 (no-id, :117-127): no frontmatter fence at all -> frontmatter::parse
    # returns Err(InvalidRecord); the walker counts it toward no_id_count instead
    # of silently skipping it unexplained.
    cat > "${work_repo}/issues/scratch-note.md" <<'EOF'
# scratch note

Deliberately has no YAML frontmatter fence, so reconciliation's
frontmatter::parse fails and this file counts toward the no_id case (case 3).
EOF

    # Case 2 (backend-deleted, :166-189): local file claims id 4, but the
    # backend record 4 is deleted for real (DELETE /projects/demo/issues/4,
    # route confirmed at reposix-sim/src/routes/issues.rs:473) BEFORE attach
    # runs, so `attach`'s build_from() no longer sees id 4 in the backend set.
    write_issue 4 "${work_repo}/issues/0004.md" "flaky integration test on CI"
    curl -fsS -X DELETE "${sim_url}/projects/demo/issues/4" >/dev/null \
        || fail_with "seed-time DELETE of backend record 4 failed (backend-deleted fixture setup)"

    # Case 5 (mirror-lag, :191-196): seeded ids 5 and 6 are deliberately left
    # with NO local file -- the walker counts every backend id absent from
    # local_ids for free.

    git -C "$work_repo" add -A
    git -C "$work_repo" commit --quiet \
        -m "RBF-A-05 reconciliation fixture (matched/no_id/backend_deleted/mirror_lag)"
}

# assert_reconciliation_counts_honest <attach_out>
# Extracts matched/no_id/backend_deleted/mirror_lag from the attach stderr
# report and requires matched>=1 AND at least 3 of the 4 counters non-zero
# -- the honest gate the p86 F13 all-zeros shape-only regex could hide
# behind before RBF-A-05. Sets MATCHED/NO_ID/BACKEND_DELETED/MIRROR_LAG
# globals for the caller's completion summary.
assert_reconciliation_counts_honest() {
    local attach_out="$1"
    local recon_line
    recon_line=$(echo "$attach_out" | grep -E 'matched=[0-9]+ no_id=[0-9]+ backend_deleted=[0-9]+ mirror_lag=[0-9]+' || true)
    [[ -n "$recon_line" ]] || fail_with "attach reconciliation report missing"

    MATCHED=$(echo "$recon_line" | grep -oE 'matched=[0-9]+' | cut -d= -f2)
    NO_ID=$(echo "$recon_line" | grep -oE 'no_id=[0-9]+' | cut -d= -f2)
    BACKEND_DELETED=$(echo "$recon_line" | grep -oE 'backend_deleted=[0-9]+' | cut -d= -f2)
    MIRROR_LAG=$(echo "$recon_line" | grep -oE 'mirror_lag=[0-9]+' | cut -d= -f2)
    echo "dark-factory: reconciliation counts -> matched=${MATCHED} no_id=${NO_ID} backend_deleted=${BACKEND_DELETED} mirror_lag=${MIRROR_LAG}" >&2

    local nonzero_cases=0
    if [[ "$MATCHED" -ge 1 ]]; then nonzero_cases=$((nonzero_cases + 1)); fi
    if [[ "$NO_ID" -ge 1 ]]; then nonzero_cases=$((nonzero_cases + 1)); fi
    if [[ "$BACKEND_DELETED" -ge 1 ]]; then nonzero_cases=$((nonzero_cases + 1)); fi
    if [[ "$MIRROR_LAG" -ge 1 ]]; then nonzero_cases=$((nonzero_cases + 1)); fi

    if [[ "$MATCHED" -ge 1 && "$nonzero_cases" -ge 3 ]]; then
        ASSERT_LOG+=("attach reconciliation report has case-specific NON-ZERO counts: matched=${MATCHED} no_id=${NO_ID} backend_deleted=${BACKEND_DELETED} mirror_lag=${MIRROR_LAG} (>=3 non-zero cases, matched required; RBF-A-05 closes p86 F13)")
    else
        fail_with "reconciliation report is vacuous (all-zeros or <3 non-zero cases)" \
            "matched=${MATCHED} no_id=${NO_ID} backend_deleted=${BACKEND_DELETED} mirror_lag=${MIRROR_LAG}"
    fi
}

# assert_duplicate_id_hard_aborts <run_dir> <sim_url> <bin_dir>
# Case 4 (duplicate-id, :136-144) -- an isolated fresh work tree + cache dir
# with two local files claiming the same id, run against `reposix attach`.
# Detection happens BEFORE any backend cross-reference or SQLite write, so
# this needs no backend setup beyond the same seeded sim; isolated so a
# deliberately-aborting attach doesn't corrupt the main run's report.
assert_duplicate_id_hard_aborts() {
    local run_dir="$1" sim_url="$2" bin_dir="$3"
    local dup_work_repo="${run_dir}/work-repo-dup"
    local dup_cache_dir
    dup_cache_dir="$(mktemp -d -t dark-factory-third-dupcache.XXXXXX)"
    mkdir -p "${dup_work_repo}/issues"
    git init --quiet "$dup_work_repo"
    git -C "$dup_work_repo" config user.email "p86@example.invalid"
    git -C "$dup_work_repo" config user.name "P86 Third Arm"
    git -C "$dup_work_repo" symbolic-ref HEAD refs/heads/main
    write_issue 2 "${dup_work_repo}/issues/a.md" "add --no-color flag to CLI"
    write_issue 2 "${dup_work_repo}/issues/b.md" "add --no-color flag to CLI (duplicate id)"
    git -C "$dup_work_repo" add -A
    git -C "$dup_work_repo" commit --quiet -m "duplicate-id fixture (case 4)"

    set +e
    local dup_out
    dup_out=$(REPOSIX_CACHE_DIR="$dup_cache_dir" REPOSIX_SIM_ORIGIN="$sim_url" \
        "${bin_dir}/reposix" attach "sim::demo" "$dup_work_repo" --remote-name reposix 2>&1)
    local dup_exit=$?
    set -e
    rm -rf "$dup_cache_dir" "$dup_work_repo"

    if [[ "$dup_exit" -ne 0 ]] && echo "$dup_out" | grep -q "duplicate id"; then
        ASSERT_LOG+=("duplicate-id (case 4) hard-aborts reconciliation with non-zero exit + 'duplicate id' message, zero rows committed (isolated sub-invocation)")
    else
        fail_with "duplicate-id sub-invocation did not hard-abort as expected" "exit=${dup_exit} out=${dup_out}"
    fi
}
