#!/usr/bin/env bash
# quality/gates/perf/latency-bench/lib.sh -- shared helpers for latency-bench dispatcher.
#
# Sourced by latency-bench.sh and the per-backend probe scripts. NOT executable
# on its own. See ../latency-bench.sh for the dispatcher entry point.

# Portable millisecond timer using `date +%s%N` (GNU coreutils on Linux).
now_ms() { date +%s%N | awk '{ print int($1 / 1000000) }'; }

# median3 <a> <b> <c> — print the median of three integers. Used to
# absorb network jitter on real-backend probes (Phase 54 plan §Risk areas).
median3() {
    local a=$1 b=$2 c=$3
    printf '%s\n' "$a" "$b" "$c" | sort -n | sed -n '2p'
}

# time_step <command...> — run the command, print elapsed ms on stdout.
# Suppresses the command's own output. Errors are tolerated (returns the
# elapsed time even on non-zero exit) so a single transient failure doesn't
# abort the bench.
time_step() {
    local t0 t1
    t0=$(now_ms)
    "$@" >/dev/null 2>&1 || true
    t1=$(now_ms)
    echo $((t1 - t0))
}

# median3_step <command...> — run the command 3x, return the median ms.
median3_step() {
    local s1 s2 s3
    s1=$(time_step "$@")
    s2=$(time_step "$@")
    s3=$(time_step "$@")
    median3 "$s1" "$s2" "$s3"
}

# count_blob_materializations <cache_dir_for_repo>
# Reads the cache.db audit table and counts `op='materialize'` rows. Path
# layout matches resolve_cache_path: <REPOSIX_CACHE_DIR>/reposix/<backend>-<project>.git/cache.db.
count_blob_materializations() {
    local db="$1/cache.db"
    if [[ ! -f "$db" ]]; then
        echo "0"
        return
    fi
    sqlite3 "$db" "SELECT COUNT(*) FROM audit_events_cache WHERE op='materialize'" 2>/dev/null || echo "0"
}

# fmt_ms <ms> — generic cell: "$ms ms" or empty/n/a passthrough.
fmt_ms() {
    local v="$1"
    if [[ -z "$v" ]]; then
        echo ""
    elif [[ "$v" == "n/a" ]]; then
        echo "n/a"
    else
        echo "${v} ms"
    fi
}

# fmt_ms_n <ms> <n> — list-row cell: "$ms ms (N=$n)" or empty/n/a passthrough.
fmt_ms_n() {
    local v="$1" n="$2"
    if [[ -z "$v" ]]; then
        echo ""
    elif [[ "$v" == "n/a" ]]; then
        echo "n/a"
    else
        echo "${v} ms (N=${n})"
    fi
}
