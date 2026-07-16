#!/usr/bin/env bash
# quality/gates/perf/latency-bench/regen-guard.sh -- refuse to clobber the
# CI-canonical sections of docs/benchmarks/latency.md.
#
# Sourced by emit-markdown.sh before it overwrites $OUT. NOT executable on
# its own -- defines a function (regen_guard_check) plus its constants;
# the caller invokes it and reacts to the return code.
#
# WHY THIS EXISTS (SESSION-HANDOVER.md §5, P115 T6 item 5 "regen-clobber
# guard"): docs/benchmarks/latency.md's headline figures (e.g. "Get one
# record = 6 ms", "List records = 7 ms" at the time this guard was written)
# plus its "Provenance & methodology" / "PATCH figures -- known caveat"
# sections are hand-curated from a specific reviewed CI run
# (bench-latency-v09; see that file's own "Provenance" section for why CI,
# not a dev machine, is the reference). emit-markdown.sh's Markdown
# template has no notion those sections exist -- every run stamps its OWN
# fixed template, which is structurally different from the curated file
# (no Provenance section, no PATCH caveat, no "Corrected" banner). A bare
# local run of quality/gates/perf/latency-bench.sh would silently destroy
# the curated content and stamp dev-machine noise -- or blank cells, for
# any backend without local credentials -- in its place.
#
# FIX SHAPE: docs/benchmarks/latency.md carries a
# `reposix:regen-guard:protected-begin` / `-end` marker comment (both lines
# live together at end-of-file, NOT wrapping the content above -- inserting
# lines before the cited sections would shift every docs-alignment citation
# bound into this file by line number, tripping STALE_DOCS_DRIFT on all of
# them for no reason; end-of-file placement changes nothing above it). This
# guard refuses to let emit-markdown.sh overwrite a file containing that
# marker unless the caller sets an explicit, informed override -- a
# refuse-and-explain gate, not a merge (the template still can't reproduce
# the curated prose, so an override run requires manually restoring it
# afterward; the error message says so).

REGEN_GUARD_BEGIN_MARKER='<!-- reposix:regen-guard:protected-begin'
REGEN_GUARD_OVERRIDE_VAR='REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE'

# regen_guard_check <out_path> -- returns 0 if it is safe to overwrite
# out_path (no file yet, or no protected marker found), 1 (with a teaching
# error on stderr) if out_path holds a protected CI-canonical section and
# no explicit override was given.
regen_guard_check() {
    local out_path="$1"

    [[ -f "$out_path" ]] || return 0
    grep -qF "$REGEN_GUARD_BEGIN_MARKER" "$out_path" || return 0
    [[ "${REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE:-}" == "1" ]] && return 0

    cat >&2 <<EOF
error: refusing to regenerate ${out_path} -- protected CI-canonical section detected

  what was protected: ${out_path} carries a 'reposix:regen-guard:protected-begin'
  marker, meaning it holds hand-curated, CI-measured figures (e.g. the
  "Get one record" / "List records" table rows) plus the "Provenance &
  methodology" and "PATCH figures -- known caveat" write-ups sourced from a
  specific CI run. This script's Markdown template does not reproduce those
  sections, so overwriting here would silently destroy them and stamp
  dev-machine (or blank, credential-less) numbers in their place.

  why: this file's canonical numbers come from a reviewed CI run
  (bench-latency-v09), not from whoever last happened to run this script on
  their laptop. Clobbering a reviewed measurement with unreviewed local
  noise defeats the entire point of citing a reproducible run log.

  recovery -- pick one:
    1. Preview a local/sim run WITHOUT touching the tracked file:
         OUT=/tmp/latency-preview.md bash quality/gates/perf/latency-bench.sh
    2. Deliberately publish a new CI-measured run (e.g. after re-running
       bench-latency-v09 and copying its numbers by hand, or via the
       weekly bench/refresh-latency cron PR): opt in explicitly, then
       manually restore the Provenance / PATCH-caveat prose the template
       does not generate:
         ${REGEN_GUARD_OVERRIDE_VAR}=1 bash quality/gates/perf/latency-bench.sh

  protected marker: reposix:regen-guard:protected-begin (${out_path})
EOF
    return 1
}
