# v0.15.0 Surprises Intake — Part 8

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain) — `part-07.md`
> was already at 30153 bytes (over the 20000-byte `*.md` ceiling, currently WARN-only
> per `structure/file-size-limits`'s waiver); this entry opens a fresh part rather than
> growing an already-over-budget file further. Index: `../SURPRISES-INTAKE.md`.

## 2026-07-19 | discovered-by: Cycle-2 bundled `/gsd-quick` task (d) executor | severity: MEDIUM | **RESOLVED same commit**

**What:** `quality/runners/test_freshness_synth.py` invoked `run.py --cadence weekly` as
a subprocess to exercise the STALE-P2 label mechanism for exactly ONE row
(`subjective/headline-numbers-sanity`), but `--cadence weekly` iterates EVERY
weekly-tagged row across EVERY catalog file, not just the target row. Two classes of
unrelated noise leaked into the test's result:

1. `quality/catalogs/release-assets.json` carries 14 weekly rows (8x
   `release/crates-io-max-version/*`, `install/homebrew`, `release/brew-formula-current`,
   `release/gh-assets-present`, `install/curl-installer-sh`,
   `install/powershell-installer-ps1`, `install/build-from-source`) that all make live
   `urllib.request` HTTP calls (crates.io API, GitHub API). A flaky or sandboxed-offline
   network turns any of these P1 rows FAIL, which flips `compute_exit_code` to 1 —
   nondeterministically breaking the test's `returncode == 0` assertion (the "stale-P2
   flake" surfaced in the PR#77 family).
2. `quality/catalogs/docs-reproducible.json` carries 2 weekly rows
   (`benchmark-claim/8ms-cached-read`, `benchmark-claim/89.1-percent-token-reduction`)
   with a deliberate catalog-first `verifier.script: null` stub and no waiver — every
   weekly run of `run_row` on these emits a `verifier not found at None` NOT-VERIFIED
   artifact, printed straight into the runner's stdout as unrelated leakage.

Locate-or-file search performed first (per this task's charter):
`grep -rn "test_freshness_synth\|stale-P2\|verifier not found at None"
.planning/milestones/v0.15.0-phases/` returned only the Cycle-2 handover pointer
(`RELIEF-HANDOVER-C2-wave-3.md:239-254`, itself the charter for this fix) — no prior
exact-match tracking row existed. `GTH-V15-55`
(`good-to-haves/part-06.md:26`, the `cdn.simpleicons.org` embed) is confirmed a
DIFFERENT live-network smell, not this one.

**Why out-of-scope for the discovering session:** N/A — fixed inline in the same session
that filed this entry (Rule 1/eager-fix; single Python test file, no new dependency,
well under the 1-hour threshold).

**Fix landed (same commit as this entry):**
`quality/runners/test_freshness_synth.py` gained `_neutralize_other_weekly_rows` — before
invoking the `--cadence weekly` subprocess, every weekly-cadence row OTHER than the
target row (across every catalog under `quality/catalogs/`) gets a temporary near-future
waiver, which makes `run_row` short-circuit to WAIVED BEFORE it can reach either the
live-network branch or the null-verifier-missing branch. The existing
`backup_catalog` fixture was generalized to `backup_catalogs` (backs up/restores EVERY
catalog file, not just `subjective-rubrics.json`) so the mutation never survives the
test, pass or fail. Verified network-denied:
`unshare -rn -- python3 -m pytest quality/runners/test_freshness_synth.py -v` passes in
~0.07s under a network namespace with no route to the internet (sanity-checked
separately: a bare `urllib.request.urlopen` call under the same `unshare -rn` fails with
`Temporary failure in name resolution`, confirming the isolation is real, not
coincidental). Before the fix, the same subprocess made 20+ live HTTP round trips and
took multiple seconds even when the network was healthy.

**Hermeticity convention this establishes (fix-twice, tagged in the SAME commit):** any
test that reaches a live network resource, directly or transitively (e.g. via a
subprocess that fans out into unrelated live-network catalog rows), must pass
deterministically OFFLINE — mock, stub, or short-circuit the probe before it fires.
Catalog row: `structure/hermetic-test-network-isolation`
(`quality/catalogs/freshness-invariants.json`, `verifier.script: null`,
`kind: manual`, `blast_radius: P2`) records this contract as a manual/documentation
gate (no automated cross-repo test-corpus scanner exists yet — that would be the
natural follow-up, filed separately if it doesn't already have a GOOD-TO-HAVE). CLAUDE.md
update: `quality/CLAUDE.md` § "Hermetic test convention" (new subsection, same commit).

**STATUS:** RESOLVED (same commit that introduces this entry — see
`quality/runners/test_freshness_synth.py` diff).

## 2026-07-19 | discovered-by: Cycle-2 task (d) executor (own `--persist` verification pass) | severity: HIGH | STATUS: OPEN

**What:** Minting the new `structure/hermetic-test-network-isolation` row's PASS status
(via `python3 quality/runners/run.py --cadence pre-pr --persist`, run once to verify the
row grades correctly) tripped a genuine, pre-existing, latent bug in
`quality/catalogs/agent-ux.json` row `agent-ux/real-git-push-e2e`: its `comment` field
documents an environment precondition ("On this dev box (git 2.25.1) the verifier's
version precondition short-circuits to NOT-VERIFIED (exit 75)") that is now FALSE — the
executing box's git has since been upgraded to **2.50.1** (>= the row's 2.34 gate). With
the precondition no longer met, the verifier actually EXECUTES for real on `--persist`,
producing a genuine status change and a fresh `last_verified` timestamp >= the P90 mint
cutoff (2026-07-05) — but the row is legacy (no `minted_at`, per its own comment:
"legacy row (no minted_at, D91-11 / PLAN-CHECK MF-2)"). `_audit_field.validate_row`
correctly rejects this at the NEXT catalog **load** with `SystemExit` ("has last_verified
>= 2026-07-05T00:00:00Z but lacks a write-once minted_at anchor") — but because
`load_catalog` raises `SystemExit` uncaught, this doesn't just FAIL that one row, it
**crashes the entire `run.py` process before grading begins**, for EVERY cadence that
loads `agent-ux.json` (i.e. every cadence except ones that never touch that catalog).
Reproduced live: this exact crash is what caused my own new gate
(`structure/hermetic-test-network-isolation`, whose verifier subprocess-invokes
`run.py --cadence weekly`) to grade a false FAIL on the first `--persist` attempt — not a
flake in my gate, a real self-inflicted catalog corruption from `agent-ux.json`'s OWN
persist-write landing (alphabetically) before my row was graded in the same run.

**Why out-of-scope for the discovering session:** Pre-existing (not caused by this
task's changes to `test_freshness_synth.py` or the new hermeticity gate) and in an
unrelated dimension (`agent-ux`, not `structure`/hermeticity). Per the scope-boundary
rule, fixed the fallout in my own working tree (reverted the corrupted `agent-ux.json` /
`code.json` via `git checkout --`, re-minted my own row cleanly via `--cadence pre-push`
instead, which does not touch this row) rather than editing `agent-ux.json` outside my
charter.

**Sketched resolution:** Add `minted_at` (RFC3339, now) + a `claim_vs_assertion_audit`
(>=50 chars) to `agent-ux/real-git-push-e2e` in the SAME commit that refreshes its stale
"git 2.25.1" comment to reflect the box's actual git version — matching the pattern used
by every other post-P90 row in this catalog. Until this lands, **any future
`--persist` mint that includes `pre-pr`/`pre-release` cadence on a box with git >= 2.34
will crash `run.py` entirely** (not just this row) the first time it re-grades this row
for real; the next agent who hits this should not re-diagnose from scratch — this entry
has the root cause and the fix.

**STATUS:** OPEN.
