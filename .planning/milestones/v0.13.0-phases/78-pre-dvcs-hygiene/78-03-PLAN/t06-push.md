← [back to index](./index.md)

# Task 03-T06 — Per-phase push (78-03 terminal)

<read_first>
- `CLAUDE.md` § "Push cadence — per-phase" — verbatim rule.
</read_first>

<action>
Stage + commit the schema migration. Single commit (or two if the SHA
substitution requires the second commit per T05):

```bash
git add \
  crates/reposix-quality/src/catalog.rs \
  crates/reposix-quality/src/commands/doc_alignment.rs \
  crates/reposix-quality/tests/walk.rs \
  quality/catalogs/doc-alignment.json \
  CLAUDE.md

git commit -m "$(cat <<'EOF'
quality(docs-alignment): walker AND-compares per-source hashes (MULTI-SOURCE-WATCH-01)

Schema migration path-b per .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:

- Row: add source_hashes: Vec<String> parallel-array (legacy source_hash kept for back-compat).
- Catalog::load: one-time backfill — promote legacy source_hash into source_hashes[0].
- verbs::walk: AND-compare per-source hashes; any-index drift fires STALE_DOCS_DRIFT;
  drifted source index surfaces in the diagnostic line.
- verbs::bind: write source_hashes on every code path (new row, Single result,
  Multi append, Multi same-source rebind); back-compat with source_hash maintained
  as source_hashes[0] for one release cycle.
- merge-shards: compute + store per-source hashes on Multi rows.
- 5 regression tests in crates/reposix-quality/tests/walk.rs covering stable /
  first-drift / non-first-drift / legacy-backfill / multi-same-source-rebind.
- Catalog row doc-alignment/multi-source-watch-01-non-first-drift minted via
  reposix-quality bind (catalog-first per quality/PROTOCOL.md Principle A).
- CLAUDE.md § "v0.12.1 P75" path-(a) tradeoff paragraph updated to cite P78-03
  closure.

Closes the v0.12.1 P75 carry-forward false-negative window before P85 DVCS
docs add multi-source rows.

Phase 78 / Plan 03 / MULTI-SOURCE-WATCH-01.
EOF
)"

# If a second commit is needed for the SHA substitution per T05:
# (run after the first commit; first commit's SHA is the substitution value)
NEW_SHA=$(git rev-parse HEAD)
# Substitute <P78-03 commit> placeholder with the real SHA in CLAUDE.md, then:
git add CLAUDE.md
git commit -m "docs: cite P78-03 SHA ${NEW_SHA} in CLAUDE.md § v0.12.1 P75 path-(a) tradeoff"

git push origin main
```

If pre-push BLOCKS, treat as phase-internal failure per CLAUDE.md (NEW
commit; never amend). The most-likely failure modes for this plan:
- Workspace test regression that local nextest didn't catch (different
  test binary linkage in CI vs local). Diagnose via `gh run view` after
  push attempt; fix; new commit; re-push.
- Walker drift on a row the migration surfaced. Eager-resolve via
  `/reposix-quality-refresh <doc>` (top-level slash command per
  CLAUDE.md "slash commands top-level only"); commit fix; re-push.
- `cargo fmt` drift introduced by the schema-migration edits.
  `cargo fmt --all`; re-stage; new commit; re-push.

After `git push origin main` exits 0, this plan is COMPLETE. The phase
verifier-subagent dispatch (P78 phase-close protocol) runs at the
**phase boundary** orchestrated by the parent /gsd-execute-phase runner —
NOT by this plan. This plan's terminal output: "78-03 pushed; ready for
phase-close protocol."
</action>

<acceptance_criteria>
- `git log --oneline | head -2` shows the schema-migration commit (and the SHA-substitution commit if used).
- `git push origin main` exits 0 (push lands; pre-push GREEN).
- The remote `main` is up to date.
- `gh run view --json conclusion` (or equivalent CI status check) shows GREEN for the latest run.
- The walker on the live catalog post-push exits 0.
- The MULTI-SOURCE-WATCH-01 catalog row is present + BOUND in `quality/catalogs/doc-alignment.json`.
- CLAUDE.md cites the real commit SHA (no `<P78-03 commit>` placeholder remains).
</acceptance_criteria>
