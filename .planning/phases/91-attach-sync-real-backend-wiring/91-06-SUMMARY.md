# 91-06 SUMMARY — docs (REQUIREMENTS/CLAUDE/comments-attachments) + T2 doc-side findings

**Scope executed:** 91-06-PLAN Tasks 1-2 ONLY (per dispatch instructions). The
plan's checkpoint/push/CI-watch/verifier-dispatch steps are coordinator-owned
and were explicitly skipped — see "Handoff to coordinator" below.

**Commits (7, chronological):**

1. `d34ad75` — REQUIREMENTS.md DVCS-ATTACH-01..04 honest sim(P79)/real-backend(P91)
   flip + traceability table; CLAUDE.md coherent Pattern-C attach example
   (confluence::TokenWorld replacing the incoherent placeholder-GH-mirror +
   sim::demo pairing), attach bullet (real-backend dispatch/pushDefault/
   missing-helper warning), bucket-aware record-path note (D91-13), litmus
   real-and-PASSED note.
2. `62ef624` — trimmed REQUIREMENTS.md prose toward the 20000-char structure
   budget (still lands at 20954/20000; WAIVED until 2026-08-08, tracked in
   GOOD-TO-HAVES-15).
3. `d45404b` — grepped docs/ for comment/attachment access promises; fixed two
   found (docs/index.md capability matrix, docs/tutorials/first-run.md
   per-connector table); confluence.md example filenames unpadded (LOW7);
   flipped both SURPRISES-INTAKE entries to ROUTED-P95 (D91-05/D91-12).
4. `807ec7a` — H3 fix: verified via a real local sim bus push that
   `refs/mirrors/*` live only in the pushing machine's cache, never reach the
   plain-git mirror, and aren't fetched across a bus remote either; rewrote
   dvcs-topology.md's "Two refs" section. Pattern C truthfulness sweep (no
   further fix needed — already correct). mkdocs.yml nav: added `reference/cli.md`
   (MED5). LOW8 (troubleshooting.md real stderr strings) + LOW9 (TokenWorld/
   REPOSIX one-space clarity, verified live against the real Confluence API).
5. `512f03b` — reconciled `deferred-items.md` (nonstandard side-file) into
   SURPRISES-INTAKE (new MEDIUM entry, p87/p88 catalog-honesty gap,
   DEFERRED-P96/P97) and GOOD-TO-HAVES (GOOD-TO-HAVES-15 consolidated
   file-size overages, GOOD-TO-HAVES-16 run.py --dry-run); XS honest callout
   added to quality/PROTOCOL.md.
6. `14331a9` — formalized D91-13 (bucket-aware canonical path layer) in
   91-DECISIONS.md, giving the Wave-5.5 de-facto fix a proper D-number.
7. `4b00435` — honest docs-alignment catalog re-walk (STALE_DOCS_DRIFT on
   ~24 rows bound to docs/index.md / docs/reference/testing-targets.md,
   the mechanical consequence of editing those files) — see "Known blocker"
   below.

**Gates run (all PASS unless noted):** `mkdocs-strict.sh` PASS ×5 reruns;
`mermaid-renders.sh` PASS (6/6 pages); `banned-words-lint.sh --all` PASS;
`verdict.py session-end` → RED, but pre-existing (agent-ux real-backend
transport rows NOT-VERIFIED because no creds/flags in this session; docs-repro
benchmark-claim rows missing verifiers; release/cargo-binstall-resolves
NOT-VERIFIED — none touched by this phase).

**Known blocker for the coordinator's terminal push (NOT resolved in this
session, no-cargo mandate):** `bash quality/gates/docs-alignment/walk.sh`
currently exits 1. Editing `docs/index.md` and `docs/reference/testing-targets.md`
(both whole-file-hash-bound doc-alignment sources) flipped ~24 previously-BOUND
claims to `STALE_DOCS_DRIFT`, which blocks `pre-push` per
`RowState.blocks_pre_push`. Recovery is the standard, sanctioned path:
`/reposix-quality-refresh docs/index.md` and
`/reposix-quality-refresh docs/reference/testing-targets.md` — both top-level
slash commands whose precondition is `cargo build -p reposix-quality --release`,
outside this docs-only wave's explicit "no cargo commands at all" mandate. The
catalog mutation itself was committed (honest state, not silently reverted).
**The coordinator must run both refreshes before `git push origin main`.**

**Handoff to coordinator:** per 91-06-PLAN, this session does NOT run the
pre-push sweep, does NOT `git push`, does NOT `gh run watch`, and does NOT
dispatch the litmus REOPEN gate or the unbiased verifier subagent. Those are
explicitly the coordinator's job. Before pushing, the coordinator must:
1. Run the two `/reposix-quality-refresh` invocations above (docs-alignment
   BLOCKER).
2. Confirm `python3 quality/runners/verdict.py session-end` is GREEN or that
   remaining REDs are the pre-existing ones named above.
3. `git push origin main`, then `gh run watch` confirming pre-pr's
   `real-git-push-e2e` executed and passed in the CI log.
4. Dispatch the milestone-close litmus REOPEN gate + the unbiased verifier
   subagent (neither run by this session).

**Tree state:** clean except the untracked
`.planning/research/doctrine-institutionalization/` directory (another
session's work — never touched, per dispatch instructions).
