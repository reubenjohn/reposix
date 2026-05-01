#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-force-with-lease-race
# CADENCE: pre-pr (~1s wall time)
# INVARIANT: --force-with-lease=refs/heads/main:<SHA-A> rejects
#            when the remote main has advanced to <SHA-B> via a
#            concurrent push. Mirror state is untouched (still
#            <SHA-B>) after the failed push.
#
# Status until P84-01 T04: FAIL (stub). T04 replaces with the full
# ~50-line file:// bare-repo race walk-through.
set -euo pipefail
echo "FAIL: T04 not yet shipped (race walk-through harness)"
exit 1
