---
verifier: fact-check subagent (zero session context)
date: 2026-05-08
inputs:
  - .planning/research/v0.13.0-real-backend-frictions/README.md
  - .planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md
methodology: read-only filesystem verification against live tree
---

# Fact-check verdict — v0.13.0 corrective-phases consolidation docs

```json
{
  "doc_paths": [
    ".planning/research/v0.13.0-real-backend-frictions/README.md",
    ".planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md"
  ],
  "claims_checked": 16,
  "claims_passed": 12,
  "claims_failed": 1,
  "claims_partial": 3,
  "claims_not_found": 0,
  "claims": [
    {
      "claim": "`.planning/STATE.md:6` says `status: ready-to-tag`",
      "cited": ".planning/STATE.md:6",
      "verified": "FAIL",
      "evidence": "`status: ready-to-tag` is on line 5; line 6 is `last_updated: \"2026-05-01T22:45:00Z\"`. README's pre-P89 housekeeping instruction (line 26) tells the next agent to flip line 6 — wrong line."
    },
    {
      "claim": "`.planning/CHANGELOG.md:9` has the `PENDING owner tag-cut` line",
      "cited": ".planning/CHANGELOG.md:9",
      "verified": "PARTIAL",
      "evidence": "Path wrong: file is at repo root `CHANGELOG.md`, NOT `.planning/CHANGELOG.md`. Line number CORRECT — line 9 has `> **Release status: PENDING owner tag-cut.** Run \\`bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh\\`...`. README hedges `(or wherever)` so partial-credit, but instruction at line 27 will fail if a cold agent literally opens `.planning/CHANGELOG.md`."
    },
    {
      "claim": "`.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` exists with guards",
      "cited": "README.md:24,28",
      "verified": "PASS",
      "evidence": "File present (executable, 2507 bytes); 8 guards numbered 1–8 in source. Note: README does not claim a specific count; STATE.md last_activity claims 8 (exceeds >=6 floor) — both consistent."
    },
    {
      "claim": "12 GREEN verdict files exist (p78,p79,p80..p88 + milestone-v0.13.0)",
      "cited": "README.md:18; REMEDIATION-PLAN.md:371",
      "verified": "PASS",
      "evidence": "All 12 directories under `quality/reports/verdicts/` contain `VERDICT.md`; each grades GREEN by grep `GREEN|brightgreen` head match. Files: p78/VERDICT.md, p79/VERDICT.md, p80..p88/VERDICT.md, milestone-v0.13.0/VERDICT.md."
    },
    {
      "claim": "`v0.13.0` tag NOT pushed",
      "cited": "README.md:7",
      "verified": "PASS",
      "evidence": "`git tag -l 'v0.13*'` returns empty. Local v0.13.0 tag does not exist."
    },
    {
      "claim": "`/gsd-phase` is a valid slash command/skill for ROADMAP CRUD",
      "cited": "README.md:32,45",
      "verified": "PASS",
      "evidence": "Skill present at `~/.claude/skills/gsd-phase/`. Hook at `~/.claude/hooks/gsd-phase-boundary.sh`. Researcher subagent at `~/.claude/agents/gsd-phase-researcher.md`."
    },
    {
      "claim": "P89–P97 NOT yet in `.planning/milestones/v0.13.0-phases/ROADMAP.md`; ROADMAP stops at P88",
      "cited": "README.md:32",
      "verified": "PASS",
      "evidence": "Phase headers found via `^### Phase [0-9]+`: 78,79,80,81,82,83,84,85,86,87,88 — 11 phases, terminating at Phase 88. No P89+ entries."
    },
    {
      "claim": "`crates/reposix-cli/src/attach.rs:147-166` rejects real backends with `not yet wired in P79-02 scaffold` literal",
      "cited": "REMEDIATION-PLAN.md:206",
      "verified": "PASS",
      "evidence": "Lines 147–166 contain the exact match clause; lines 162–165 carry the literal `attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03`. Banned-token regex (P89's RBF-FW-04) would catch this."
    },
    {
      "claim": "`crates/reposix-cli/src/sync.rs:79-92` has parallel `not yet wired in v0.13.0 (sim only); ... P82+` literal",
      "cited": "REMEDIATION-PLAN.md:207",
      "verified": "PASS",
      "evidence": "Lines 79–92 contain match clause; lines 88–91 carry literal `sync --reconcile: backend `{other}` not yet wired in v0.13.0 (sim only); github/confluence/jira land alongside the bus-remote work in P82+`. Same banned-token violation."
    },
    {
      "claim": "`crates/reposix-remote/src/backend_dispatch.rs:234-274` has 3 `instantiate_*` functions, none chaining `.with_audit(...)`",
      "cited": "REMEDIATION-PLAN.md:39 (H-B3) + RBF-B-03",
      "verified": "PARTIAL",
      "evidence": "Three real-backend `instantiate_*` functions present in cited range: `instantiate_github` (234), `instantiate_confluence` (242), `instantiate_jira` (262). None invoke `.with_audit(...)` (grep returns 0 matches in entire file). PARTIAL caveat: the file actually has 4 `instantiate_*` functions total — `instantiate_sim` at line 228 also exists; but it is OUTSIDE the cited range and the OP-3 audit-log claim concerns real backends only, so the cited 234-274 window correctly captures the 3 real-backend functions."
    },
    {
      "claim": "`crates/reposix-cli/tests/agent_flow_real.rs` has zero `attach_real_*` functions",
      "cited": "REMEDIATION-PLAN.md:29 (H-A5) + RBF-A-04",
      "verified": "PASS",
      "evidence": "Test functions present: `dark_factory_real_github` (line 128), `dark_factory_real_confluence` (146), `dark_factory_real_jira` (170). Zero `attach_real_*` or `sync_real_*` functions. Confirms the gap RBF-A-04 closes."
    },
    {
      "claim": "Phase header cross-refs consistent after renumber (insert new P93)",
      "cited": "REMEDIATION-PLAN.md throughout",
      "verified": "PARTIAL",
      "evidence": "P89 deps=none ✓; P90=P89 ✓; P91=P89+P90 ✓; P92=P89+P90+P91 ✓; P93=P89+P90+P91+P92 ✓; P94=P91+P92 (skips P93 — explicit parallelism allowance per §4 line 421); P95=P89..P94 ✓ (includes P93); P96=P89-P95 ✓; P97=P89-P96 ✓. P97 RBF-G-04 cross-refs `P91's RBF-A-05 + P92's RBF-B-06 + P93's RBF-LR-04 + P94's RBF-C-03 + P95's RBF-D-15` — all verified to exist in named phases. P94 not depending on P93 is internally consistent (parallelism note line 421) but readers may flag the visual mismatch with the linear graph at line 418."
    },
    {
      "claim": "No `v0.13.1` strings remain in REMEDIATION-PLAN.md",
      "cited": "REMEDIATION-PLAN.md (negative claim)",
      "verified": "PASS",
      "evidence": "`grep -nE 'v0\\.13\\.1' REMEDIATION-PLAN.md` returns 0 hits."
    },
    {
      "claim": "Total phase count = 9 (P89–P97); `grep -cE \"^### \\*\\*P[89][0-9]\"` returns 9",
      "cited": "REMEDIATION-PLAN.md (structural)",
      "verified": "PASS",
      "evidence": "Exact grep returns `9`. Phase headers present at lines 141 (P89), 171 (P90), 201 (P91), 232 (P92), 262 (P93), 290 (P94), 320 (P95), 362 (P96), 388 (P97). Wave 9 confirmed."
    },
    {
      "claim": "No `milestone-v0.13.1` references anywhere in the consolidation docs",
      "cited": "README.md + REMEDIATION-PLAN.md (negative claim)",
      "verified": "PASS",
      "evidence": "`grep -nE 'milestone-v0\\.13\\.1'` against both files returns 0 hits. All milestone-pointer strings use `milestone-v0.13.0`."
    },
    {
      "claim": "Tag-script guards count + 'currently all pass' sub-claim",
      "cited": "README.md:24",
      "verified": "PARTIAL",
      "evidence": "Script has 8 numbered guards (1: clean tree; 2: on main; 3: workspace version; 4: CHANGELOG entry; 5: cargo test workspace; 6: pre-push runner; 7: P88 verdict GREEN; 8: milestone-v0.13.0 verdict GREEN). Cannot verify 'currently all pass' without running script (read-only mode). Guards 1, 2, 4, 7, 8 would PASS by static check (verdicts GREEN, CHANGELOG entry present); guards 3, 5, 6 require live cargo/python invocation. Hedged PARTIAL because the README claim is implied (next-move-is-tag-cut), not asserted directly."
    }
  ]
}
```

---

# Material findings

## FAIL (1)

- **STATE.md line citation off-by-one.** README at line 26 says "Flip `.planning/STATE.md:6` `status:` from `ready-to-tag` to `extending-via-corrective-phases-p89-p97`." But `status: ready-to-tag` is on **line 5**, not line 6. A cold agent following the housekeeping checklist literally will edit the wrong line (`last_updated`). Fix: change `:6` to `:5` in README line 26 (and surrounding hedging in line 12 which also says `STATE.md:6`).

## PARTIAL (3)

- **CHANGELOG.md path is wrong but README hedges.** README line 27 says `.planning/CHANGELOG.md:9`. Actual path is `CHANGELOG.md` (repo root). Line 9 is correct. README hedges with `(or wherever the "PENDING owner tag-cut" line lives)` so a thorough agent will recover. Recommend dropping the `.planning/` prefix.
- **`backend_dispatch.rs:234-274` claim window contains 3 instantiate_* functions, but file actually has 4 (sim + 3 real).** The cited range correctly bounds the 3 real-backend functions germane to OP-3 audit silence; sim is at line 228 and out of range. Not a defect, but readers may need to verify the line range explicitly.
- **P94 dependency on P93 is implicit.** P94's `Dependencies:` line cites `P91 + P92 GREEN` — does NOT name P93. The §4 dependency graph (line 418) shows linear `P92 → P93 → P94`; the parallelism note at line 421 explicitly allows P91↔P94 to run partially in parallel given file-disjointness. Internally consistent, but visually inconsistent with the linear graph diagram. A reader may flag it as a renumber bug.

## NOT-FOUND (0)

All other claims PASS as stated.

---

# Source-of-truth pointers

| Claim domain | Live evidence at |
|---|---|
| STATE.md status line | `.planning/STATE.md:5` (NOT 6) |
| CHANGELOG PENDING line | `CHANGELOG.md:9` (repo root, NOT `.planning/`) |
| Tag script + guards | `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (8 guards) |
| 12 GREEN verdicts | `quality/reports/verdicts/{p78..p88,milestone-v0.13.0}/VERDICT.md` |
| `attach.rs` real-backend reject literal | `crates/reposix-cli/src/attach.rs:162-165` |
| `sync.rs` real-backend reject literal | `crates/reposix-cli/src/sync.rs:88-91` |
| 3 instantiate_* without with_audit | `crates/reposix-remote/src/backend_dispatch.rs:234,242,262` |
| Zero attach_real_* tests | `crates/reposix-cli/tests/agent_flow_real.rs` (only 3 dark_factory_real_*) |
| ROADMAP terminates at P88 | `.planning/milestones/v0.13.0-phases/ROADMAP.md` (last header line 115 `### Phase 88`) |

---

**End of fact-check.**
