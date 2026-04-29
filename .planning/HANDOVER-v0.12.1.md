# v0.12.1 HANDOVER — autonomous-run brief

**Created:** 2026-04-29 PT (autonomous-run prep session close).
**Owner:** reuben.
**Deadline:** 5pm PT.
**Mode:** Highly autonomous. **Being ahead of schedule is suspicious** — it suggests skipping steps. Pace through the work; respect the verifier subagent dispatch on every phase close; update CLAUDE.md per phase.

> **Eventually delete this file** — the last commit closing the run removes it.

---

## Entry point

```
/gsd-execute-phase 72
```

Then 73 → 74 → 75 → 76 → 77 in sequence. Each phase is fully scoped:
- ROADMAP entry: `.planning/milestones/v0.12.1-phases/ROADMAP.md` § Phase N.
- Requirements: same file's REQUIREMENTS.md (LINT-CONFIG-*, CONNECTOR-GAP-*, NARRATIVE-*, UX-BIND-*, PROSE-FIX-*, BIND-VERB-FIX-*, SURPRISES-ABSORB-*, GOOD-TO-HAVES-*).
- Research brief: `.planning/phases/<N>-<name>/CONTEXT.md` with 8-12 numbered decisions D-01..D-N. Treat the brief as **normative** — the planner consumes it.

---

## What was prepped (read before starting)

### Repo state at handover

- Working tree clean (last commit: this handover prep).
- Local ahead of origin by ~50 commits.
- **Local tag `v0.12.0` exists.** Push BLOCKED — SSH config drift (`~/.ssh/config` IdentityFile points at `id_github_ed25519` but the actual key is `id_ed25519_github`). Owner pushes manually:
  ```
  git push origin main && git push origin v0.12.0
  ```
- `quality/catalogs/doc-alignment.json`: 388 rows total — 313 BOUND, 22 MISSING_TEST, 23 RETIRE_PROPOSED, 30 RETIRED. `alignment_ratio` 0.8743 (clears v0.12.1 0.85 target). `coverage_ratio` 0.2031 (above 0.10 floor). Pre-push: 21 PASS / 1 FAIL / 3 WAIVED — only `docs-alignment/walk` fails (per-row blocker by design — closes when P72-P74 land).

### Catalog rows targeted by P72-P74

22 MISSING_TEST rows. Inventory by phase:
- **P72 lint-config (9):** README + contributing.md lint/MSRV/test-count rows.
- **P73 connector (4):** auth-header, real-backend-smoke, attachments-excluded, jira-not-implemented.
- **P74 narrative + UX (9 + 1 prose):** 4 narrative-retires, 4 docs/index UX binds, 1 spaces-01 CLI smoke, 1 polish2-06-landing connector-matrix grep, plus linkedin.md:21 prose fix.

23 RETIRE_PROPOSED rows are **owner-TTY-only** to confirm — see "What the owner owes" below.

### +2 phase reservation (OP-8 — new operating principle)

CLAUDE.md OP-8 (added during prep, line ~48 of CLAUDE.md) defines the **+2 phase practice**:
- Every milestone reserves its last two phases for surprises and good-to-haves discovered during planned-phase execution.
- P72-P75 phases append to `.planning/milestones/v0.12.1-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` when they find something out-of-scope. P76 drains SURPRISES; P77 drains GOOD-TO-HAVES.
- **Eager-resolution preference is load-bearing:** if a discovery is < 1 hour, no new dep, no new file outside the planned set — fix it inside the discovering phase. The intake is for items that genuinely don't fit.
- Read OP-8 in CLAUDE.md before starting P72; it's not a suggestion.

---

## Operating principles to honor (don't skip these)

| Principle | Where it lives | Why it matters here |
|---|---|---|
| **Verifier subagent grades GREEN** | CLAUDE.md OP-7 / QG-06 | Every phase close dispatches `gsd-verifier` (Path A via `Task` from top-level). Verdict at `quality/reports/verdicts/p<N>/VERDICT.md`. Executing agent does NOT talk verifier out of RED. |
| **CLAUDE.md update per phase** | CLAUDE.md QG-07 | Each phase introducing a file/convention/gate updates CLAUDE.md in the same PR. P72-P77 each gain an H3 subsection ≤30 lines under "v0.12.1 — in flight". |
| **Build memory budget** | CLAUDE.md § "Build memory budget" | One cargo invocation at a time. Per-crate over workspace where possible. The VM has crashed twice from parallel cargo. |
| **+2 phase practice** | CLAUDE.md OP-8 | See above. Append to intake files instead of expanding scope or silently skipping. |
| **No push, no tag, no release** | global CLAUDE.md "Executing actions with care" | Owner pushes (SSH config drift means they have to anyway). Don't `git push`, don't `git tag --push`, don't run `cargo publish`. Local commits only. |
| **Catalog-first commits** | quality/PROTOCOL.md | First commit of each phase writes the catalog rows / verifier-script stubs that define GREEN; subsequent commits reference row IDs. |
| **Subagent delegation** | CLAUDE.md § "Subagent delegation rules" | gsd-planner for plans (don't write PLAN.md by hand); gsd-executor for execution; gsd-code-reviewer after each phase ships; verifier subagent on close. |

---

## Per-phase budget (5pm pacing)

| Phase | Effort | Run cap | Notes |
|---|---|---|---|
| P72 lint-config | 3.5 hr | 4 hr | Heaviest. 9 verifiers. Front-load while context fresh. |
| P73 connector | 2-3 hr | 3 hr | 2 new Rust tests + 1 rebind + 1 prose-or-retire. |
| P74 narrative + UX | 1.5-2 hr | 2 hr | Fastest. Mostly shell scripts + a one-line prose edit. |
| P75 bind-verb fix | 30-45 min | 1 hr | Small Rust fix + regression test + live walk smoke. |
| P76 surprises | variable | 1.5 hr | Depends on what P72-P75 surfaced. |
| P77 good-to-haves | whatever fits | 30 min floor | Time-boxed to 5pm. XS items first. |
| **Total** | **~9 hr** | **~12 hr** | Realistic ~7 hr of focused work + ~2 hr verifier dispatch + 1 hr CLAUDE.md updates. |

If you're done much before 5pm, you've skipped steps. Common skips: (a) skipping CLAUDE.md updates per phase, (b) inline-grading instead of dispatching the verifier, (c) folding multiple phases into one commit, (d) handwaving REQUIREMENTS.md flips. Don't.

---

## What the owner owes (do NOT do these — wait for owner)

1. **Push v0.12.0 tag** — `git push origin main && git push origin v0.12.0` (manually, after fixing SSH config OR via HTTPS push). Tag exists locally.
2. **Bulk-confirm 23 RETIRE_PROPOSED rows** — env-guarded; needs TTY without `$CLAUDE_AGENT_CONTEXT` set. Owner runs:
   ```bash
   for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
     target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
   done
   git add quality/catalogs/doc-alignment.json && git commit -m "docs(v0.12.1): bulk-confirm 23 retirements"
   ```
3. **Confirm 4 NEW propose-retires** (added by P74 — narrative-rows). Same TTY pattern.
4. **Bump Cargo.toml to 0.12.1** (optional; signals active dev tracks v0.12.1 next-release).

The next agent should NOT attempt these — they're owner-TTY-only or consequential.

---

## What's NEW since the previous handover

- **CLAUDE.md OP-8** (the +2 phase practice) added.
- **6 new phase research briefs** under `.planning/phases/{72,73,74,75,76,77}-*/CONTEXT.md`.
- **2 new intake files:** `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` under `.planning/milestones/v0.12.1-phases/`.
- **ROADMAP** rewritten with concrete P72-P77 entries replacing the generic "Phase 72+" stub.
- **REQUIREMENTS** grew from 18 to 40 items (LINT-CONFIG-* through GOOD-TO-HAVES-*).
- **STATE.md** flipped milestone cursor to v0.12.1 in-flight.
- **Local v0.12.0 tag** created; push pending owner SSH fix.

---

## Cleanup criterion

This file deletes itself when:
- All 6 phases (P72-P77) ship verifier-GREEN.
- Owner has pushed v0.12.0 tag and confirmed retires.
- A v0.12.1 milestone-close verdict is graded GREEN at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md`.

The commit closing the autonomous run includes `git rm .planning/HANDOVER-v0.12.1.md` and writes a session summary inline in the commit message.

---

## Optional reading

- `.planning/milestones/v0.12.1-phases/ROADMAP.md` — concrete phase-by-phase scope.
- `.planning/milestones/v0.12.1-phases/SESSION-LOG-2026-04-28.md` — previous session's full log.
- CLAUDE.md § "Operating Principles (project-specific)" — OP-1..OP-8 (OP-8 is new).
- CLAUDE.md § "v0.12.1 — in flight" — high-level mental model.
- `quality/PROTOCOL.md` — runtime contract for the quality gates.
- `quality/SURPRISES.md` — project-wide pivot journal (different from `SURPRISES-INTAKE.md` — that one's phase-scoped).
