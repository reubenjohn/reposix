---
phase: 260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b
plan: 01
subsystem: docs/concepts + docs/guides (DVCS docs cluster polish)
tags: [docs, dvcs, cold-reader-polish, quick]
requires: []
provides:
  - docs/concepts/dvcs-topology.md (binstall crate-name fix + slug hedge removal)
  - docs/guides/dvcs-mirror-setup.md (template anchor + secret-set interactive note + readable sed example)
affects: []
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - docs/concepts/dvcs-topology.md
    - docs/guides/dvcs-mirror-setup.md
decisions:
  - Anchor (Finding 5) inserted at line 21 (after intro line 19, before Step 1 line 23) — keeps lines 5-11 byte-identical so docs-alignment row hash does not drift.
  - Sed example (Finding 3) collapsed to single-line `\*` form inside double quotes — sed receives identical bytes vs the doubly-escaped `\\*` form, no portability regression.
  - Verdict artifact at quality/reports/verifications/subjective/dvcs-cold-reader.json left AS-IS per owner instruction; next /reposix-quality-review run regrades.
metrics:
  duration: ~12 minutes
  completed: 2026-05-01
---

# Quick 260501-mgn: Polish 5 cold-reader nits in DVCS docs Summary

One-liner: closed 5 non-critical findings from the dvcs-cold-reader rubric verdict (binstall crate name, secret-set non-TTY note, sed readability, slug-hedge ambiguity, template-location anchor) in one atomic commit across `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md`.

## Findings closed

Each finding maps verbatim to the rationale captured in `quality/reports/verifications/subjective/dvcs-cold-reader.json`:

1. **Finding 1 — binstall crate name** (`docs/concepts/dvcs-topology.md:134`): changed `cargo binstall reposix` → `cargo binstall reposix-cli`. Matches CLAUDE.md "Webhook-driven mirror sync (v0.13.0 P84+)" and the existing `docs/guides/dvcs-mirror-setup.md:89` reference. Bare `reposix` is the workspace name and would 404 on binstall.
2. **Finding 2 — secret-set interactive note** (`docs/guides/dvcs-mirror-setup.md`, after Step 3 secrets/vars block at line 71): added one-sentence blockquote-Note flagging that `gh secret set <NAME>` (without `--body`) prompts interactively, with stdin-pipe and `--body` examples for non-TTY use. Anchor matches the existing `> **Tip:**` convention at line 33.
3. **Finding 3 — sed readability** (`docs/guides/dvcs-mirror-setup.md:151`): collapsed multi-line, doubly-escaped `sed -i "s|cron: '\\*/30 \\* \\* \\* \\*'|cron: '\\*/15 \\* \\* \\* \\*'|" \\` + continuation into single-line `sed -i "s|'\*/30 \* \* \* \*'|'\*/15 \* \* \* \*'|" .github/workflows/reposix-mirror-sync.yml`. Identical sed semantics; substantially more readable.
4. **Finding 4 — slug hedge** (`docs/concepts/dvcs-topology.md:61`): replaced "the host slug renders as `confluence` (or your tenant alias)" with "the host slug is `confluence`. (The slug always names the backend kind, not your tenant. The four canonical slugs are `sim`, `github`, `confluence`, `jira`.)". Removes the misleading hedge; cites the four canonical slugs from CLAUDE.md "Mirror-lag refs".
5. **Finding 5 — template anchor** (`docs/guides/dvcs-mirror-setup.md:21`): inserted one-sentence blockquote anchor between the intro section (lines 1-19) and "## Step 1" (now line 23) naming `docs/guides/dvcs-mirror-setup-template.yml` as the workflow template location and pointing to Step 2 for the curl command.

## Atomic commit

| SHA | Message |
|---|---|
| `2b9e9c9` | `docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)` |

`git show --name-only HEAD` confirms exactly two files changed: `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md`.

## Gate verifications

All four required gates PASS post-commit:

| Gate | Command | Exit |
|---|---|---|
| docs-alignment (topology) | `bash quality/gates/docs-alignment/dvcs-topology-three-roles.sh` | 0 (PASS — three roles + Q2.2 phrasing) |
| docs-alignment (setup walkthrough) | `bash quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh` | 0 (PASS — Steps 1-5 + Cleanup + cron-only fallback + template ref) |
| banned-words | `bash scripts/banned-words-lint.sh` | 0 (PASS — no Layer-1/Layer-2 jargon leaks) |
| mkdocs strict | `bash scripts/check-docs-site.sh` | 0 (PASS — strict build green; new anchor link resolves) |

## Catalog-safety: source_hash drift did NOT fire

Per the plan's `<catalog_safety_pre_audit>`:

- `docs-alignment/dvcs-topology-three-roles-bound` hashes `docs/concepts/dvcs-topology.md` **lines 5-11** (intro paragraph). Both edits (line 61, line 134) lie OUTSIDE this range — verified post-edit by re-reading lines 1-15 and confirming byte-identical content.
- `docs-alignment/dvcs-mirror-setup-walkthrough-bound` hashes `docs/guides/dvcs-mirror-setup.md` **lines 5-11** (intro paragraph + "What you need before you start" header). All three edits (line 71 note, line 151 sed, line 21 anchor) lie OUTSIDE this range — verified post-edit by re-reading lines 1-30 and confirming lines 5-11 are byte-identical to the pre-edit file.
- `docs-alignment/dvcs-troubleshooting-matrix-bound` hashes `docs/guides/troubleshooting.md` line 227 only. troubleshooting.md was not edited.

No `reposix-quality doc-alignment plan-refresh` invocation needed. Walker run on next pre-push will not fire `STALE_DOCS_DRIFT` on either bound row.

## Verdict artifact unchanged

`quality/reports/verifications/subjective/dvcs-cold-reader.json` was not touched (`git diff HEAD~1 HEAD -- <path>` empty). Per owner instruction, the next `/reposix-quality-review` run regrades the rubric against the polished docs.

## Deviations from Plan

None — plan executed exactly as written. All five edits landed verbatim per the `<interfaces>` section; no Rule 1-3 auto-fixes triggered; no architectural questions surfaced.

## Self-Check: PASSED

- `docs/concepts/dvcs-topology.md` — FOUND (line 134 contains `cargo binstall reposix-cli`; line 61 starts with "the host slug is `confluence`")
- `docs/guides/dvcs-mirror-setup.md` — FOUND (line 21 contains `Workflow template location`; line 71 contains `prompts interactively`; line 151 contains the single-line sed form)
- Commit `2b9e9c9` — FOUND in `git log --oneline -3`
- All four gate scripts — exit 0
