#!/usr/bin/env bash
#
# apply-w2-refresh.sh -- P117 W2 docs-alignment refresh (Step B), fully decided.
#
# Applies, in order:
#   1. 17 bind/re-bind commands (extracted verbatim from three completed grader
#      transcripts: Grader A = 7 binds + 1 propose-retire; Grader B = 7 binds +
#      1 benchmarks waive; Re-bind pass = 3 binds for rows Grader B could only
#      mark-missing-test on first pass).
#   2. 1 propose-retire (twitter 89.1% row, from Grader A).
#   3. 1 waive (twitter 89.1% row -- NOT present in any transcript, written
#      verbatim per the L0 router's decided text).
#   4. 1 waive (benchmarks/README-md/session-provenance, from Grader B).
#
# Cross-check -- exactly these 17 row-ids MUST be the --row-id targets of the
# 17 bind commands below (order as extracted, grouped by source transcript):
#   - docs/index/latency-24ms-cold-init
#   - docs/index/latency-8ms-read
#   - latency-hero-24ms-mismatch
#   - docs/index/mcp-loop-4883-tokens
#   - docs/index/real-git-working-tree
#   - docs/index/reposix-loop-531-tokens
#   - docs/index/rest-supported-backends
#   - filesystem-layer/cache-bare-repo-with-wal
#   - filesystem-layer/audit-blob-materialize-rows
#   - filesystem-layer/partial-clone-tree-fetched-blobs-lazy
#   - git-remote/bulk-delete-cap-5
#   - git-remote/export-push-path
#   - git-remote/server-field-stripping
#   - docs/reference/cli.md/init_documented
#   - filesystem-layer/extensions-partialclone-signals-promisor
#   - git-remote/stateless-connect-read-path
#   - filesystem-layer/blob-lazy-first-cat
#
# Provenance (seat #57 relief, 2026-07-16):
#   Grader A -- agent-a35534b7c728f276c.jsonl (7 bind + 1 propose-retire, twitter)
#   Grader B -- agent-ab019cd644de5afc2.jsonl (7 bind + 1 waive, benchmarks)
#   Re-bind  -- agent-ae91372bbcc983ddc.jsonl (3 bind, for rows B could only
#               mark-missing-test on first pass)
#   Commands were extracted from each transcript's `CMD: ./target/release/
#   reposix-quality ...` lines, JSON-unescaped, and NOT hand-edited except:
#   (a) prefix rewritten to $BIN, (b) one literal backtick pair inside the
#   stateless-connect-read-path rationale ESCAPED (\`) to prevent bash command
#   substitution when embedded in a double-quoted --rationale string.
#
# This script does NOT run automatically as part of any gate -- it is dispatched
# by hand per .planning/SESSION-HANDOVER.md Step B. It is NOT safe to re-run
# blindly after a partial failure: reposix-quality bind/waive/propose-retire are
# NOT idempotent against a row already in a terminal state (a second bind on an
# already-bound row, or a second waive on an already-waived row, errors). If a
# prior run got partway through, re-run `doc-alignment walk` first to see which
# rows are still outstanding and hand-trim the commands below to match, rather
# than re-running the whole script.
#
# The catalog write path (quality/catalogs/doc-alignment.json) is atomic-rename
# but NOT concurrency-safe -- this script runs every command serially on
# purpose. Never parallelize these invocations.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"

BIN=./target/release/reposix-quality

if [[ ! -x "$BIN" ]]; then
  echo "FATAL: $BIN not found or not executable -- build it first:" >&2
  echo "  cargo build --release -p reposix-quality" >&2
  exit 1
fi

echo "=== apply-w2-refresh.sh: 17 binds, 1 propose-retire, 2 waives ==="

echo "[1/17 bind] docs/index/latency-24ms-cold-init"
$BIN doc-alignment bind --row-id docs/index/latency-24ms-cold-init --claim "278 ms cold init against simulator (CI-canonical)" --source docs/index.md:18-18 --test quality/gates/perf/headline-numbers-cross-check.py --test quality/gates/perf/test_headline_numbers_cross_check.py --grade GREEN --rationale "docs/index.md:18 '**278 ms** cold init — simulator, CI-canonical' matches canonical latency.md:40 'reposix init cold' sim=278 ms; run_cross_check LATENCY_CLAIMS init-axis (L181-186) asserts value==canonical and test_hero_surfaces_match_canonical asserts failures==[] (offline). P115 T6 hash refresh."

echo "[2/17 bind] docs/index/latency-8ms-read"
$BIN doc-alignment bind --row-id docs/index/latency-8ms-read --claim "6 ms cached read against simulator" --source docs/index.md:18-18 --test quality/gates/perf/headline-numbers-cross-check.py --test quality/gates/perf/test_headline_numbers_cross_check.py --grade GREEN --rationale "docs/index.md:18 '**6 ms** cached read' matches canonical latency.md:42 'Get one record' sim=6 ms; run_cross_check LATENCY_CLAIMS get-axis (L144-149) asserts value==canonical, test_hero_surfaces_match_canonical asserts failures==[]. Hash refresh (same hero line moved 27->278 ms cold-init bytes)."

echo "[3/17 bind] latency-hero-24ms-mismatch"
$BIN doc-alignment bind --row-id latency-hero-24ms-mismatch --claim "Homepage hero: reposix cold init = 278 ms (CI-canonical sim)" --source docs/index.md:18-18 --test quality/gates/perf/headline-numbers-cross-check.py --test quality/gates/perf/test_headline_numbers_cross_check.py --grade GREEN --rationale "docs/index.md:18 hero '**278 ms** cold init' matches canonical latency.md:40 'reposix init cold' sim=278 ms; run_cross_check LATENCY_CLAIMS init-axis (L181-186) asserts equality and test_hero_surfaces_match_canonical asserts failures==[]."

echo "[4/17 bind] docs/index/mcp-loop-4883-tokens"
$BIN doc-alignment bind --row-id docs/index/mcp-loop-4883-tokens --claim "MCP tool loop generates ~21k output tokens (live) for the same 3-issue read+edit+push" --source docs/index.md:35-39 --test quality/gates/perf/headline-numbers-cross-check.py --test quality/gates/perf/test_headline_numbers_cross_check.py --grade GREEN --rationale "docs/index.md:35 mermaid note '~21k output tokens (live)' derived from token-economy.md:37 GitHub-MCP output-token median 21,171 (round/1000=21); run_cross_check TOKEN_LOOP_CLAIMS output_mcp axis (L216-220) asserts the substring present, test_hero_surfaces_match_canonical asserts failures==[]."

echo "[5/17 bind] docs/index/real-git-working-tree"
$BIN doc-alignment bind --row-id docs/index/real-git-working-tree --claim "reposix creates a real git working tree with cat, grep, edit, git commit, git push" --source docs/index.md:13-13 --test crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path --test crates/reposix-remote/tests/push_conflict.rs::clean_push_emits_ok_and_mutates_backend --grade GREEN --rationale "dark_factory_sim_happy_path asserts a real .git/ working tree (extensions.partialClone=origin + promisor=true + partialclonefilter=blob:none, assert_eq loop L119-136) so cat/grep/edit are vanilla-git ops; clean_push_emits_ok_and_mutates_backend asserts the 'git push' half emits 'ok refs/heads/main' (L266) + one PATCH to the backend (mock .expect(1), L226). docs/index.md:13 still states the real-git-tree + cat/grep/edit/git push surface."

echo "[6/17 bind] docs/index/reposix-loop-531-tokens"
$BIN doc-alignment bind --row-id docs/index/reposix-loop-531-tokens --claim "reposix git-native loop uses ~1.2k output tokens (live) for the 3-issue read+edit+push task" --source docs/index.md:31-34 --test quality/gates/perf/headline-numbers-cross-check.py --test quality/gates/perf/test_headline_numbers_cross_check.py --grade GREEN --rationale "docs/index.md:31 mermaid note '~1.2k output tokens (live)' derived from token-economy.md:37 reposix output-token median 1,213 (_fmt1/1000='1.2'); run_cross_check TOKEN_LOOP_CLAIMS output_reposix axis (L211-215) asserts the substring present and test_run_cross_check_flags_stale_loop_figure pins the ~1.2k/1,213 pairing."

echo "[7/17 bind] docs/index/rest-supported-backends"
$BIN doc-alignment bind --row-id docs/index/rest-supported-backends --claim "reposix supports Jira, GitHub Issues, Confluence as REST-based backends" --source docs/index.md:13-13 --test crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_github --test crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence --test crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_jira --grade GREEN --rationale "docs/index.md:13 names Jira/GitHub Issues/Confluence; each has a real-backend init smoke in agent_flow_real.rs asserting a live REST init URL (run_init_and_assert asserts status.success + reposix::https://<api-host>/ prefix): dark_factory_real_github (api.github.com + /projects/reubenjohn/reposix), dark_factory_real_confluence (/confluence/projects/<space>), dark_factory_real_jira (/jira/projects/<project>). Expanded from github-only citation so all three named backends are covered."

echo "[8/17 bind] filesystem-layer/cache-bare-repo-with-wal"
$BIN doc-alignment bind --row-id filesystem-layer/cache-bare-repo-with-wal --claim "Cache is a real bare git repo with cache.db in SQLite WAL mode" --source docs/how-it-works/filesystem-layer.md:57-57 --test crates/reposix-cache/src/cache.rs::open --grade GREEN --rationale "cache.rs:75 gix::init_bare builds the bare repo and cache.rs:143 open_cache_db opens cache.db, which sets journal_mode=WAL (db.rs:54); source drifted 56->57"

echo "[9/17 bind] filesystem-layer/audit-blob-materialize-rows"
$BIN doc-alignment bind --row-id filesystem-layer/audit-blob-materialize-rows --claim "Every blob materialization writes an audit row to cache.db" --source docs/how-it-works/filesystem-layer.md:57-57 --test crates/reposix-cache/src/audit.rs::log_materialize --grade GREEN --rationale "audit.rs:44-46 still INSERTs op='materialize' into audit_events_cache on blob materialization; source drifted 56->57"

echo "[10/17 bind] filesystem-layer/partial-clone-tree-fetched-blobs-lazy"
$BIN doc-alignment bind --row-id filesystem-layer/partial-clone-tree-fetched-blobs-lazy --claim "Partial clone fetches the tree (filenames, OIDs) but skips blob contents until read" --source docs/how-it-works/filesystem-layer.md:49-49 --test crates/reposix-cache/tests/blobs_are_lazy.rs::no_blob_objects_after_build_from --grade GREEN --rationale "blobs_are_lazy.rs:44-48 asserts blob_count==0 and tree_count>=1 after build_from (tree present, blobs lazy); source drifted 48->49"

echo "[11/17 bind] git-remote/bulk-delete-cap-5"
$BIN doc-alignment bind --row-id git-remote/bulk-delete-cap-5 --claim "Any push that would delete more than 5 files is rejected with SG-02 cap" --source docs/reference/git-remote.md:81-81 --test crates/reposix-remote/tests/bulk_delete_cap.rs::six_deletes_refuses_and_calls_no_delete --grade GREEN --rationale "bulk_delete_cap.rs:121-128 asserts 6 deletes yield non-zero exit + stderr 'refusing to push'/'cap is 5' with DELETE expect(0); refreshes drifted test_body_hash"

echo "[12/17 bind] git-remote/export-push-path"
$BIN doc-alignment bind --row-id git-remote/export-push-path --claim "export is the push path: helper parses fast-import stream, runs conflict detection, applies REST writes on success" --source docs/reference/git-remote.md:23-23 --test crates/reposix-remote/tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest --grade GREEN --rationale "push_conflict.rs:189-200 drives an export stream and asserts stdout 'error refs/heads/main fetch first' with PATCH/POST/DELETE expect(0): export push path parses the stream + runs conflict detection; refreshes drifted hashes"

echo "[13/17 bind] git-remote/server-field-stripping"
$BIN doc-alignment bind --row-id git-remote/server-field-stripping --claim "Server-controlled fields (id, version, created_at, updated_at) are stripped before PATCH/POST via sanitize()" --source docs/reference/git-remote.md:107-113 --test crates/reposix-remote/tests/push_conflict.rs::frontmatter_strips_server_controlled_fields --grade GREEN --rationale "push_conflict.rs:365-376 asserts the PATCH body drops attacker id=999999 and omits the version/id fields entirely (sanitize strips server-controlled fields); refreshes drifted test_body_hash"

echo "[14/17 bind] docs/reference/cli.md/init_documented"
$BIN doc-alignment bind --row-id docs/reference/cli.md/init_documented --claim "reposix init subcommand documented with backend::project spec form" --source docs/reference/cli.md:42-65 --test crates/reposix-cli/tests/cli.rs::init_help_documents_spec_argument --grade GREEN --rationale "cli.rs:254-263 asserts init --help contains BACKEND::PROJECT/<backend>::<project> and each of sim/github/confluence/jira; matches the doc's spec-form + four-backend section"

echo "[15/17 bind] filesystem-layer/extensions-partialclone-signals-promisor"
$BIN doc-alignment bind --row-id filesystem-layer/extensions-partialclone-signals-promisor --claim "extensions.partialClone=origin tells git the remote is a promisor that delivers missing blobs" --source docs/how-it-works/filesystem-layer.md:58-58 --test crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path --test crates/reposix-cache/tests/partial_clone_serves.rs::filtered_clone_then_checkout_serves_materialized_blobs --grade GREEN --rationale "agent_flow.rs:119-136 asserts extensions.partialClone==origin AND remote.origin.promisor==true AND remote.origin.partialclonefilter==blob:none (the promisor config); partial_clone_serves.rs:170-184 proves the promisor delivers missing blobs via a real git clone --filter=blob:none + checkout serving issues/1.md 'body of issue 1'. dark_factory_sim_happy_path is #[ignore]d (spawns sim)."

echo "[16/17 bind] git-remote/stateless-connect-read-path"
$BIN doc-alignment bind --row-id git-remote/stateless-connect-read-path --claim "stateless-connect is the v0.9.0 read path, tunneling protocol-v2 fetch with --filter=blob:none" --source docs/reference/git-remote.md:23-23 --test crates/reposix-remote/tests/stateless_connect_e2e.rs::stateless_connect_advertises_then_eof --test crates/reposix-cache/tests/partial_clone_serves.rs::filtered_clone_then_checkout_serves_materialized_blobs --grade GREEN --rationale "stateless_connect_e2e.rs:122-134 asserts the stateless-connect tunnel advertisement contains \`version 2\` (protocol-v2) and flush-terminates, proving stateless-connect is the read path tunneling protocol-v2; partial_clone_serves.rs:161-174 asserts a git clone --filter=blob:none succeeds against the same git upload-pack the stateless-connect path spawns."

echo "[17/17 bind] filesystem-layer/blob-lazy-first-cat"
$BIN doc-alignment bind --row-id filesystem-layer/blob-lazy-first-cat --claim "A bare POSIX cat never triggers a network call — it is a local read; blob contents materialize at git checkout/fetch time, not at cat (the tree is fetched once at init)" --source docs/how-it-works/filesystem-layer.md:43-43 --test crates/reposix-cache/tests/partial_clone_serves.rs::filtered_clone_then_checkout_serves_materialized_blobs --test crates/reposix-cache/tests/blobs_are_lazy.rs::no_blob_objects_after_build_from --grade GREEN --rationale "partial_clone_serves.rs:179-184 proves blob contents arrive at git clone --filter=blob:none + checkout (asserts checked-out issues/1.md contains 'body of issue 1'), then reads it via a local std::fs::read_to_string; blobs_are_lazy.rs:44-49 asserts blob_count==0 after build_from (tree fetched at init, contents lazy until checkout/fetch)."

echo "[propose-retire] docs/social/twitter/token-reduction-92pct"
$BIN doc-alignment propose-retire --row-id docs/social/twitter/token-reduction-92pct --claim "reposix achieves 89.1% fewer tokens than MCP-mediated baseline in token-economy benchmark (number now matches measured)" --source docs/social/twitter.md:18-18 --rationale "Superseded by docs/concepts/reposix-vs-mcp-and-sdks.md:43-45 (commit d2fd85c), which RETIRES the synthetic count_tokens-over-fixture 89.1% figure. bench_token_economy.py was rewritten (P115 T5 amendment #10) to the live GitHub-capture JSONL methodology; _assert_headline_reduction now asserts ~94.3% (main() no longer emits 89.1%). twitter.md:18 already reads '~94% fewer output tokens' (live median-of-3). Same documented decision that RETIRE_CONFIRMED the index/README/concepts 89.1% siblings (e.g. token-baseline-mcp-4883); replacement is a DIFFERENT live claim, not a rebinding."

echo "[waive] docs/social/twitter/token-reduction-92pct (RETIRE_PROPOSED, human-confirm pending)"
$BIN doc-alignment waive --row-id docs/social/twitter/token-reduction-92pct --until 2026-10-10T00:00:00Z --reason "RETIRE_PROPOSED: 89.1% synthetic count_tokens figure superseded by documented decision (docs/concepts/reposix-vs-mcp-and-sdks.md:43-45, commit d2fd85c); confirm-retire is human-only; twitter.md:18 prose already reads ~94%" --tracked-in "P117 follow-up: owner confirm-retire twitter 89.1% row + backfill-mint a new ~94% row for twitter.md:18"

echo "[waive] benchmarks/README-md/session-provenance (GTH-V15-15 follow-up tracked)"
$BIN doc-alignment waive --row-id benchmarks/README-md/session-provenance --until 2026-10-10T00:00:00Z --reason "No committed test falsifies the reposix_session.txt provenance claim (ANSI-stripped/live-GitHub/no-mnt/no-demo); candidate bench_token_economy.py --offline only regenerates numbers from captures, and test_main_offline uses synthetic seeds asserting against the generated doc, not the fixture" --tracked-in "P117 W6 doc-alignment refresh (GTH-V15-15)"

echo "=== apply-w2-refresh.sh: all 20 commands applied ==="
echo "Next: $BIN doc-alignment walk -- confirm docs-alignment:-prefixed blocking lines are gone."

