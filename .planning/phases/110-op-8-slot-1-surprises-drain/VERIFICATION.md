---
phase: 110-op-8-slot-1-surprises-drain
row: agent-ux/p110-surprises-absorption
verified: 2026-07-12T00:00:00Z
verdict: GREEN
gate_exit_code: 0
score: 4/4 assertions verified
verifier: Claude (gsd-verifier, unbiased phase-close)
---

# Phase 110 — v0.14.0 OP-8 Slot 1 (SURPRISES-INTAKE drain) — Verification

**Phase goal:** Drain `.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md` per
CLAUDE.md OP-8 — every P102–P109 surprise reaches terminal STATUS
(RESOLVED | DEFERRED | WONTFIX) with a commit SHA or a target-milestone rationale,
and an F-K5 honesty spot-check is authored by a non-orchestrator leaf.

**Verdict: GREEN.** Gate exits 0; all 4 catalog assertions independently confirmed;
honesty spot-check confirms the RESOLVED flips are backed by real code + real SHAs,
not cosmetic status changes.

## Gate result

```
$ bash quality/gates/agent-ux/p110-surprises-absorption.sh
PASS: SURPRISES-INTAKE drained (0 OPEN, 16 terminal); honesty spot-check artifact present
EXIT_CODE=0
```

Produced artifact (`quality/reports/verifications/agent-ux/p110-surprises-absorption.json`):
`status: PASS, open_count: 0, terminal_count: 16`.

## Per-assertion evidence (verified independently of the script)

| # | Catalog assertion | Status | Evidence |
|---|---|---|---|
| 1 | `SURPRISES-INTAKE.md` exists | ✓ VERIFIED | file present, 16 entries under `## Entries` |
| 2 | Zero `**STATUS:** OPEN` lines (fence-aware) | ✓ VERIFIED | Independent grep finds exactly 1 raw `**STATUS:** OPEN` at line 28 — the schema example INSIDE the ` ```markdown ` fence (opens L19, closes L29). Fence-aware awk correctly excludes it → 0 real OPEN. |
| 3 | ≥10 entries carry terminal STATUS | ✓ VERIFIED | 16 terminal (independent `grep -cE`): 10 RESOLVED + 6 DEFERRED + 0 WONTFIX. Matches honesty-check cross-reference exactly. Non-canonical suffixes (`RESOLVED-in-P102`, `DEFERRED-TO-v0.15.0`) match the leading-token regex — no gaming. |
| 4 | `honesty-spot-check.md` exists as F-K5 evidence | ✓ VERIFIED | Present, 109 lines; genuine — deep P105 walk + per-phase table (P103/P106/P107/P108/P109/P102/P105) + signed verdict. NOT a placeholder. |

## Honesty spot-check (the point of an unbiased verifier)

Picked RESOLVED entries and confirmed each resolution is REAL, not a status flip:

- **Entry 11 — fetch-ref-lock, `RESOLVED-in-P105` `bd5b9cb`:** The double-write is
  genuinely gone. `crates/reposix-remote/src/main.rs:202` advertises
  `refspec refs/heads/*:refs/reposix-import/*` and `fast_import.rs:159/195` writes the
  PRIVATE `refs/reposix-import/*` namespace (disjoint from user tracking ns
  `refs/reposix/origin/*`). Fix commit real; matches the claim to the file:line.
- **Entry 14 — release.yml CI-ungated, `RESOLVED` `0d05d7f`:** Real. `ci-green-gate:`
  job at `release.yml:58`; `needs: ci-green-gate` at `:106`. (Honesty caveat in the
  entry is itself honest — the tag-triggered gate's first LIVE fire is P111.)
- **Entry 16 — runner unit tests not collected, `RESOLVED` `3f1458d`:** Real.
  `runner-unit-tests:` job at `ci.yml:419`; DP-2 guard
  `test_fleet_safety_verdicts_untracked.py` wired at `:449`.
- **Entry 8 — three stale v0.13.0 rows, `RESOLVED (eager waive)`:** Real, and the
  backing is a catalog edit not a SHA — confirmed all three rows
  (`p87-surprises-absorption`, `p88-good-to-haves-drained`, `v0.13.0-tag-script-present`)
  carry detailed `waiver` strings citing the P110 drain (rows 1091/1171/1246). Waived,
  not force-passed.

All 14 cited SHAs resolve with commit messages matching the intake claims
(`39a8500`, `2ad2bf5`, `9d78d62`, `61e8222`, `4dd7e10`, `632864d`, `ed42ece`,
`bd5b9cb`, `8afb52d`, `0d05d7f`, `3f1458d`, `3206a2b`, `bf88470`, `90ddaff`).
Supporting evidence the honesty-check itself cites also exists: P106
`quality/reports/verifications/docs-repro/*` (9 JSONs), P107
`evidence/p107-cargo-audit-2026-07-12.txt` (`7cfd165`), P105 `VERIFICATION.md`.

**honesty-spot-check.md is genuine.** It does not rubber-stamp: its deep walk SURFACED
a real ledger-lag — intake row 5 (fetch-ref-lock) was filed at 08:35 reading
"still-live HIGH bug," but the same-phase fix `bd5b9cb` landed LATER in the P105 close;
the outcome-facing walk caught it and corrected the entry to `RESOLVED-in-P105` with
only the genuine residual (git ≥ 2.34 stateless-connect verification, run on 2.25.1)
carried as DEFERRED. This is F-K5 clause (c) doing its job.

## Noticing (OD-3 ownership deliverable)

1. **Row artifact JSON is gitignored — NOT committed (deliberate).**
   `quality/reports/verifications/agent-ux/p110-surprises-absorption.json` matches
   `.gitignore:72` (`quality/reports/verifications/*/*.json`). Committing it requires
   `git add -f` against `.gitignore` — which is EXACTLY the anti-pattern P102 row 5
   (the entry being verified) resolved: force-added report snapshots rot by
   construction; durable proof is the verifier script + catalog row, not a frozen JSON.
   Following that resolved policy, only VERIFICATION.md is committed. (Minor pre-existing
   inconsistency: 5 `p93-*.json` siblings in the same dir are tracked — force-added
   before/against this policy — worth a future sweep, not this phase's scope.)
2. **DEFERRED dispositions are honest and complete.** All 6 name a target
   (v0.15.0 framework-hardening / helper-hardening; coverage-climb phases; owner-gate)
   plus rationale. Entry 7 (shell-coverage FAIL) and entry 15 (release-plz) correctly
   assess push-blocking status (P2 non-blocking / owner-gate + `code.json` foreign-lock)
   rather than hand-waving — no silent scope creep.
3. **HIGH push-correctness item surfaced to owner as required** (Success Criterion 4):
   the fetch-ref-lock was fixed in-phase (`bd5b9cb`) and the residual is verification-only.

## Conclusion

Phase goal ACHIEVED. The intake is fully drained (0 OPEN / 16 terminal), the terminal
flips are backed by real code and real commits, and the F-K5 honesty spot-check is
genuine and even self-corrected a stale entry. Row `agent-ux/p110-surprises-absorption`
grades **PASS**.

---
_Verified: 2026-07-12 — Claude (gsd-verifier), unbiased phase-close grade._
