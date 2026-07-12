---
phase: P107 — RUSTSEC memmap2 + quinn-proto posture
verified: 2026-07-12T17:35:00Z
verifier: unbiased phase-close verifier (fresh context, committed-artifacts only)
verdict: GREEN
score: 4/4 deliverables PASS
ci_green_dimension: PENDING-ENVIRONMENTAL (main cancelled-by-concurrency, NOT failed)
---

# P107 Verification — RUSTSEC posture gate

**Verdict: GREEN.** All four GREEN-contract deliverables PASS against committed
artifacts, verified independently (not on the executor's word). Gate run live: exit 0.

## Per-deliverable grading

### D1 — Committed cargo-audit evidence artifact — PASS
- `.planning/milestones/v0.14.0-phases/evidence/p107-cargo-audit-2026-07-12.txt` exists,
  is git-tracked, committed at `7cfd165` (`git cat-file -t 7cfd165` = commit; `--stat`
  shows the file, +169 lines).
- Content shows ground truth: **0 live vulns, exit 0**, 419 crate deps, advisory-db HEAD
  `6e3286f4` synced 2026-07-12 (same-day — zero-count is NOT a stale-DB artifact).
- memmap2 0.9.11 == floor (transitive via gix 0.83.0); quinn-proto 0.11.15 == floor and
  `cargo tree -i quinn-proto` = "nothing to print" (transitive-and-absent orphan). Honest.

### D2 — Honest SECURITY.md RUSTSEC posture — PASS
- `SECURITY.md` § "Advisory posture (cargo audit / RUSTSEC)" (line 57+) names BOTH
  RUSTSEC-2026-0185 (quinn-proto, **transitive-and-absent**, floor met, not in resolved
  build) and RUSTSEC-2026-0186 (memmap2, **transitive-and-present** via gix, floor met,
  informational=unsound). Each carries a reachability verdict + why-not-actionable +
  3-layer mitigation (Cargo.lock pins / CI audit.yml / local gate). Specific, no hand-waving.

### D3 — Catalog-first row (predates implementation) — PASS
- Row `security/cargo-audit-rustsec-posture` present in `quality/catalogs/security-gates.json`.
- `git merge-base --is-ancestor 5904e67 d61cbb7` → TRUE: the catalog-mint commit predates
  the gate-implementation commit. Row was introduced exactly at 5904e67 (git log on the file).
- `expected.asserts` encodes all four: cargo audit 0 live vulns; both advisory IDs cleared
  by version floor (with reachability nuance); posture doc names both IDs. blast_radius P1,
  waiver null, cadences pre-push+pre-release.

### D4 — Gate script exists, executable, PASSES — PASS
- `quality/gates/security/cargo-audit-posture.sh` exists, mode `-rwxr-xr-x`.
- **Ran it (my one cargo invocation, no concurrent cargo):**
  `PASS security/cargo-audit-rustsec-posture: 0 live advisories; RUSTSEC-2026-0186 +
  RUSTSEC-2026-0185 cleared by version floor; posture doc present` — **EXIT 0**.
- Defense-in-depth confirmed by reading the script: floors parsed offline from Cargo.lock
  (independent of audit exit code), network-fetch-failure distinguished as exit-2/PARTIAL
  (never conflated with a live CVE), posture-doc grep for both IDs. A green audit alone
  cannot mask a floor regression.

## Honesty / noticing checks — all PASS
- **cargo-audit claim corrected:** SECURITY.md line 53 states cargo-audit is "wired in two
  places" (CI `audit.yml` + the new local gate) — no lingering "planned/in-flight" claim.
- **security-regression.sh honesty:** SECURITY.md line 49 says the umbrella script is
  "not yet implemented," tracked as a GOOD-TO-HAVE. `ls scripts/` confirms NO
  security-regression.sh — doc does not lie about a nonexistent file.
- **Intake RESOLVED:** `.../v0.13.0-phases/surprises-intake/part-01.md` line 58 RUSTSEC
  entry carries "2026-07-12 P107 … **RESOLVED**" with pointers to the evidence artifact,
  SECURITY.md section, and catalog row.
- **Orphan-quinn GOOD-TO-HAVE filed:** GOOD-TO-HAVES-12 in v0.14.0 GOOD-TO-HAVES.md
  (drop orphan quinn/quinn-proto/quinn-udp lockfile entries; correctly warns against an
  opportunistic `cargo update`).

## CI-green dimension — PENDING-ENVIRONMENTAL (main NOT red)
Graded on facts, not conflated. `gh run list --workflow=ci.yml --branch=main`:
- Gate commit `d61cbb7` IS on origin/main (`--is-ancestor d61cbb7 origin/main` = TRUE).
- Its CI run 29201895275 concluded **cancelled** (concurrency storm), NOT failure.
- Runs 29202168183 / 29202086149 / 29201935044 / 29201781589 all **cancelled**; a fresh
  run 29202231485 is in_progress. **No `failure` conclusion anywhere** in the last 12.
- Last conclusive main CI = **success** (29201470806, b398f48).
- **Verdict: main is NOT RED — it is cancelled-by-concurrency.** The CI-green requirement
  is PENDING-ENVIRONMENTAL, awaiting a run that survives the storm. This does NOT roll the
  P107 phase verdict RED (no real test failure exists to block on).

## Verifier noticing (own)
- `quality/catalogs/code.json` shows as modified (`M`) in the working tree — foreign to
  P107; I did not touch it (per constraint). Flag for whichever session owns it.
- The gate asserts quinn-proto via "absent OR floor-met"; today quinn-proto 0.11.15 is
  present-as-orphan, so it passes on the floor branch. Once GOOD-TO-HAVES-12 lands (lockfile
  regen drops the orphan), the same assert passes on the absent branch — gate is robust to
  both states. Good forward-compat design.
- Evidence artifact honestly self-labels its capture point as "(pre-artifact) ~b398f48".

**Overall: P107 = GREEN.** 4/4 deliverables PASS; honesty/noticing all clean; CI-green
is PENDING-ENVIRONMENTAL (not FAILED).
