← [back to index](./index.md)

# Task 02-T05 — Catalog-first atomic commit + per-phase push (78-02 terminal)

<read_first>
- `CLAUDE.md` § "Push cadence — per-phase" — verbatim rule.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose with citations; tools validate and mint" — confirms the .sh + catalog flip ship together (validation + mint = one commit).
</read_first>

<action>
Stage the catalog-first atomic commit. The commit MUST contain:
- 3 new .sh files (the verifiers).
- 1 modified `quality/catalogs/freshness-invariants.json` (rows flipped).
- 1 modified `quality/gates/structure/freshness-invariants.py` (3 path-forward comments).
- 3 new `quality/reports/verifications/structure/<row>.json` artifact files
  (written by the runner during T04 dry-run; .gitignored? Check first — if
  artifacts are gitignored, omit). Search for these in the gitignore: `git
  check-ignore quality/reports/verifications/structure/no-loose-top-level-planning-audits.json`.
  If untracked, omit; if tracked, include.

```bash
git add \
  quality/gates/structure/no-loose-top-level-planning-audits.sh \
  quality/gates/structure/no-pre-pivot-doc-stubs.sh \
  quality/gates/structure/repo-org-audit-artifact-present.sh \
  quality/catalogs/freshness-invariants.json \
  quality/gates/structure/freshness-invariants.py
# Add report artifacts only if not gitignored:
# git add quality/reports/verifications/structure/{no-loose-top-level-planning-audits,no-pre-pivot-doc-stubs,repo-org-audit-artifact-present}.json

git commit -m "$(cat <<'EOF'
quality(structure): land 3 TINY verifiers + flip WAIVED → PASS (HYGIENE-02)

- quality/gates/structure/no-loose-top-level-planning-audits.sh (TINY 5-30 line shape)
- quality/gates/structure/no-pre-pivot-doc-stubs.sh (TINY)
- quality/gates/structure/repo-org-audit-artifact-present.sh (TINY)
- catalog rows flipped WAIVED → PASS in same atomic commit (catalog-first; quality/PROTOCOL.md Principle A)
- waiver blocks deleted (auto-renewal would defeat catalog-first principle; expiry was 2026-05-15)
- freshness-invariants.py path-forward comments added; Python branches retained as regression net
- runner-end-to-end: python3 quality/runners/run.py --cadence pre-push GREEN with new .sh dispatch

Phase 78 / Plan 02 / HYGIENE-02.
Carries forward WAIVED-STRUCTURE-ROWS-03 from CARRY-FORWARD.md.
EOF
)"

git push origin main
```

Pre-push hook runs the runner gates BEFORE the push. If pre-push BLOCKS:
- Most-likely failure: runner doesn't dispatch .sh (T04 should have caught
  this, but a stale catalog path could still surface). Diagnose, fix, NEW
  commit (not amend), re-push.
- Less-likely: a synthetic offender file got accidentally added. Check
  `git diff HEAD~1` for unintended `.planning/MILESTONE-AUDIT-*.md` or
  `docs/<stub>.md` that the smoke-tests left behind.
</action>

<acceptance_criteria>
- `git log -1 --oneline` shows the HYGIENE-02 commit.
- `git show HEAD --stat` shows ≥4 files: 3 new .sh + 1 catalog edit + 1 freshness-invariants.py edit (artifact JSONs may or may not appear depending on gitignore).
- `git push origin main` exits 0 (push lands; pre-push GREEN).
- The remote `main` is up to date.
- `python3 quality/runners/run.py --cadence pre-push` exits 0 immediately after push (no regression).
- `gh issue list` does NOT need any close action here (no GH issues track HYGIENE-02; the catalog row IS the tracker).
</acceptance_criteria>
