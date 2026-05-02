# PROJECT.md and REQUIREMENTS.md issues

← [back to index](./index.md)

## PROJECT.md issues

### P1 — "Active" requirements section is the v0.1.0 MVD list, all unticked (lines 56–78) [P0]

**Current text (lines 58–67):**
```
**Functional core**
- [ ] **Simulator-first architecture.** ...
- [ ] **Issues as Markdown + YAML frontmatter.** ...
- [ ] **FUSE mount with full read+write.** ...
- [ ] **`git-remote-reposix` helper.** ...
- [ ] **Working CLI orchestrator.** `reposix sim`, `reposix mount`, `reposix demo` ...
...
```

Every bullet here SHIPPED in v0.1.0. The Security guardrails block (lines 69–77) shipped in v0.1.0 / v0.4.1 / v0.5.0. The line "**FUSE mount with full read+write.**" is **post-pivot incorrect** — FUSE was deleted in v0.9.0. The line "`reposix sim`, `reposix mount`, `reposix demo`" advertises commands that no longer exist.

**Recommendation:** delete lines 56–80 (the entire Active section under v1) and replace with a one-line pointer "See REQUIREMENTS.md `## v0.11.0 Requirements` for the active list." Move the v0.1.0 MVD list, if it has historical value, into a `<details>` block labelled "v0.1.0 MVD bring-up — shipped 2026-04-13".

### P2 — "Constraints" section still cites 2026-04-13 timeline + autonomous-build framing (lines 99–106) [P1]

**Current text (line 100):**
```
- **Timeline**: Demo by 2026-04-13 ~08:00 PDT. Hard limit. Project kicked off 2026-04-13 ~00:30 PDT. ~7 hours of autonomous build time.
```

That deadline is 12 days past. Same for line 106's "Decision deadline: At local time 03:30 (T+3h from kickoff)..."

**Recommendation:** replace both lines with current-state constraints (e.g. "Pre-public-launch; tag pushes are owner gates. Autonomous mode never hits a real backend; sim-first per CLAUDE.md OP-1."). Or wrap the whole "Constraints" block in `<details>` as "v0.1.0 launch constraints (historical)".

### P3 — Key Decisions table all rows show `— Pending` (lines 110–123) [P1]

**Current text (lines 110–123):** every row in the Key Decisions table has `— Pending` in the Outcome column, even though these decisions have all been made and lived through 9 milestones (Rust over Python, simulator-first, fuser-default-features-false, rusqlite-bundled, workspace-with-5-crates, etc.).

**Recommendation:** sweep the Outcome column and replace `— Pending` with `Validated` (or "Validated; FUSE-row supplanted by v0.9.0 partial-clone pivot" for the fuser row). Or remove the column entirely — every decision in the table is implicitly Validated by virtue of the project being at v0.11.0.

### P4 — "Current Milestone: v0.11.0" goal underspecifies overnight surprises (lines 124–135) [P1]

**Current text:**
```
**Goal:** Land the helper-multi-backend-dispatch fix carried forward from v0.9.0 ... ; ship `cargo run -p reposix-bench` for honest per-backend latency numbers; close the 9 major + 17 minor doc-clarity findings ... ; capture the playwright screenshots deferred from v0.10.0 ... ; and complete the IssueId→RecordId refactor that's in flight on a parallel runner.

**Carry-forward from v0.9.0 (still open):**
- Helper hardcodes `SimBackend` in `stateless-connect` handler — must land before any v0.11.0 benchmark commits.
```

Three closed items still listed open:
1. Helper-multi-backend-dispatch — **closed `cd1b0b6` 2026-04-25** per `v0.9.0-MILESTONE-AUDIT.md` line 28 ("RESOLVED 2026-04-25").
2. IssueId→RecordId refactor "in flight on a parallel runner" — **closed `2af5491..4ad8e2a` 2026-04-25** per `MORNING-WALKTHROUGH-2026-04-25.md` line 10.
3. The "9 major + 17 minor" pointer is fine; the items remain open.

**Recommended replacement:**
```
**Goal:** Honest per-backend latency benchmarks (`scripts/v0.9.0-latency.sh` extension or new `cargo run -p reposix-bench`); close the 9 major + 17 minor doc-clarity findings (`.planning/notes/v0.11.0-doc-polish-backlog.md`); capture the playwright screenshots deferred from v0.10.0 (cairo libs); resurface real-backend integration coverage now that secret packs decrypt; ship the launch (`docs/blog/2026-04-25-reposix-launch.md` + asciinema). See `.planning/research/v0.11.0/vision-and-innovations.md` for ambitious add-ons.

**Carry-forward already closed (do not re-open):**
- Helper backend URL dispatch — closed `cd1b0b6` (ADR-008).
- IssueId/Issue → RecordId/Record rename — closed `2af5491..4ad8e2a` (ADR-006).
- `reposix doctor` / `reposix gc` / `reposix history` / `reposix at` — overnight surprise round, all in `Unreleased` CHANGELOG block.

**Carry-forward still open:**
- Playwright screenshots (DOCS-11 SC4) — `scripts/take-screenshots.sh` stub.
- 9 major + 17 minor doc-clarity findings — `v0.11.0-doc-polish-backlog.md`.
- `scripts/tag-v0.10.0.sh` push — owner gate.
- `scripts/tag-v0.9.0.sh` push — owner gate (per MORNING-WALKTHROUGH-2026-04-25 §"You can:").
```

### P5 — "Previously Validated Milestone: v0.9.0" carries `tech_debt` framing that has flipped (lines 164–189, esp. line 188) [P2]

The v0.9.0 milestone-audit verdict flipped from `tech_debt` to `passed` 2026-04-25 (`v0.9.0-MILESTONE-AUDIT.md` line 24). PROJECT.md line 188 still implies v0.9.0 has unresolved tech debt: "Carry-forward from v0.9.0 (tech debt): Helper hardcodes `SimBackend` ...". That carry-forward closed.

**Recommendation:** strike the v0.9.0 tech-debt line in this section, or replace with "Carry-forward from v0.9.0: none (helper backend dispatch landed 2026-04-25 commit `cd1b0b6`; verdict flipped `tech_debt` → `passed`)."

---

## REQUIREMENTS.md issues

### R1 — DOCS-01..11 still show `[ ]` Active checkboxes (lines 23–33) but PROJECT.md marks them all Validated [P0]

**Current text (REQUIREMENTS.md lines 23–33):**
```
- [ ] **DOCS-01**: ... ≤ 250 above-fold words ...
- [ ] **DOCS-02**: Three-page "How it works" section ...
...
- [ ] **DOCS-11**: README updated to point to mkdocs site ...
```

**PROJECT.md lines 44–54:** every DOCS-01..11 has `✓` and a phase pointer.
**MILESTONES.md lines 11–24:** lists DOCS-01..11 as Key accomplishments shipped.

This is a contradiction between active-list and validated-list. **Direct collision; one is wrong.**

**Recommendation:** REQUIREMENTS.md should track the **active** milestone only. After v0.10.0 close, the v0.10.0 Requirements section should be moved into a "Validated milestones" archive section — exactly like the v0.9.0 ARCH section already is (lines 60–75). Specifically: relabel REQUIREMENTS.md line 1 "# Requirements — Active milestone: v0.10.0" → "# Requirements — Active milestone: v0.11.0". Move DOCS section (lines 8–56) under a "## v0.10.0 Requirements (Validated) — Docs & Narrative Shine" header. Author the new "## v0.11.0 Requirements" section based on PROJECT.md's revised Goal block (see P4).

### R2 — Traceability table (lines 184–203) marks ARCH-01..18 as `planning` [P1]

**Current text (lines 185–203):**
```
| ARCH-01 | 31 | planning |
| ARCH-02 | 31 | planning |
...
| ARCH-18 | 36 | planning |
| ARCH-19 | 36 | shipped |
```

ARCH-01..18 all shipped (PROJECT.md lines 25–43 mark them ✓ Validated). Only ARCH-19 says shipped — and even that's inconsistent because ARCH-19 was `pending-secrets` per `v0.9.0-MILESTONE-AUDIT.md` line 39.

**Recommended replacement:** flip every `planning` to `shipped`. Keep ARCH-19 as `shipped (pending-secrets)` until secrets decrypt.

### R3 — Note line 205 says ARCH-01..19 shipped [P2]

**Current text (line 205):**
```
*(v0.9.0 ARCH-01..19 all shipped 2026-04-24. DOCS-01..09 (originally deferred from v0.9.0) are owned by the active v0.10.0 section above. DOCS-10 and DOCS-11 are new in v0.10.0.)*
```

This contradicts the table directly above. Rewrite the whole traceability table consistent with this note.

### R4 — `Active milestone:` line 1–4 still says v0.10.0 [P0]

**Current text:**
```
# Requirements — Active milestone: v0.10.0 Docs & Narrative Shine

**Active milestone:** v0.10.0 Docs & Narrative Shine (planning_started 2026-04-24).
**Previous validated milestone:** v0.9.0 Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24, see "v0.9.0 Requirements (Validated)" section below).
```

Wrong header. v0.11.0 is the active milestone (STATE.md frontmatter line 3 confirms).

**Recommended replacement:**
```
# Requirements — Active milestone: v0.11.0 Performance & Sales Assets

**Active milestone:** v0.11.0 Performance & Sales Assets (planning_started 2026-04-25).
**Previous validated milestones:** v0.10.0 Docs & Narrative Shine (SHIPPED 2026-04-25); v0.9.0 Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24).
```
