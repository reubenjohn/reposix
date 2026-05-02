← [back to index](./index.md)

# T1 — git mv connectors/guide.md to guides/write-your-own-connector.md; verify content preserved

<task type="auto">
  <name>Task 1: git mv connectors/guide.md to guides/write-your-own-connector.md; verify content preserved</name>
  <files>docs/connectors/guide.md, docs/guides/write-your-own-connector.md</files>
  <read_first>
    - `docs/connectors/guide.md` (source — 465 lines; read the first 50 and last 50 to know anchors)
    - `docs/guides/write-your-own-connector.md` (current 3-line stub from plan 30-02 — MUST be removed before git mv)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/guides/write-your-own-connector.md (preserve verbatim, update relative links)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §"Runtime State Inventory" (reference for link audit)
  </read_first>
  <action>
    Move the file, preserving git history. Sequence:

```bash
# 1. Confirm source and stub both exist
test -f docs/connectors/guide.md
test -f docs/guides/write-your-own-connector.md

# 2. Remove the stub (plan 30-02 placeholder). Git will track this as deletion.
git rm -q docs/guides/write-your-own-connector.md

# 3. Move the content. git recognizes this as rename even after step 2.
git mv docs/connectors/guide.md docs/guides/write-your-own-connector.md

# 4. Verify content preserved
wc -l docs/guides/write-your-own-connector.md
# expected: within +/- 3 of 465 (original line count)

# 5. Verify git sees it as rename (via git status --renames)
git status --short
# expected: R  docs/connectors/guide.md -> docs/guides/write-your-own-connector.md
#           D  docs/guides/write-your-own-connector.md (from step 2)
# If the stub was tiny (3 lines) git may combine the two ops into a single R.
```

After the move, audit internal links WITHIN the moved file. Open `docs/guides/write-your-own-connector.md` and look for:

- References to `../decisions/...` — these work from guides/ (same nesting depth as connectors/). NO change needed.
- References to `../reference/...` — same. NO change needed.
- Absolute GitHub URLs (`https://github.com/reubenjohn/reposix/blob/main/...`) — unaffected by the move. NO change needed.
- Any reference to "this guide" or "this page lives at `docs/connectors/...`" in prose — rephrase to drop the path reference OR update to `docs/guides/write-your-own-connector.md` if the location is load-bearing.

If the moved file contains any self-referential path, grep it:

```bash
grep -n 'connectors/guide' docs/guides/write-your-own-connector.md
# If non-empty: fix each match to refer to the new location or drop the self-reference.
```

Preserve the source-of-truth clause (PATTERNS.md quotes lines 64-65 from original file):

> Do NOT read the above and copy it into your adapter's docs — link to
> `crates/reposix-core/src/backend.rs` as the single source of truth.

Verify this clause is still present unchanged.

Run `mkdocs build --strict` to confirm no dangling links appear anywhere else in the docs tree pointing to `docs/connectors/guide.md`:

```bash
mkdocs build --strict 2>&1 | tee /tmp/mkdocs-30-07-task1.log
```

If `--strict` fails with errors referencing `connectors/guide.md`, the offender is another docs page (not this file). Fix those on a case-by-case basis via `sed`; most likely in `docs/reference/confluence.md`, `docs/development/contributing.md`, or `docs/research/*`.

After successful build, no change needed to `mkdocs.yml` — plan 30-04 already placed the entry under `Guides > Write your own connector: guides/write-your-own-connector.md`.
  </action>
  <verify>
    <automated>test -f docs/guides/write-your-own-connector.md && ! test -f docs/connectors/guide.md && wc -l docs/guides/write-your-own-connector.md | awk '{exit !($1 >= 400)}' && grep -c 'crates/reposix-core/src/backend.rs' docs/guides/write-your-own-connector.md | awk '{exit !($1 >= 1)}' && mkdocs build --strict</automated>
  </verify>
  <acceptance_criteria>
    - `test -f docs/guides/write-your-own-connector.md` returns 0.
    - `test -f docs/connectors/guide.md` returns 1 (does NOT exist).
    - `wc -l docs/guides/write-your-own-connector.md` reports `>= 400` lines (content preserved).
    - `grep -c 'crates/reposix-core/src/backend.rs' docs/guides/write-your-own-connector.md` returns `>= 1` (source-of-truth clause preserved).
    - `grep -c 'single source of truth' docs/guides/write-your-own-connector.md` returns `>= 1`.
    - `git log --follow docs/guides/write-your-own-connector.md | head -20` shows commits from when the file lived at `docs/connectors/guide.md` (git rename detection working).
    - `mkdocs build --strict` exits 0 (no dangling links after move).
    - `grep -rn 'connectors/guide' docs/ .github/ README.md CHANGELOG.md 2>/dev/null` returns 0 matches (or only matches in CHANGELOG historical entries, which are intentional per Phase 25 precedent — historical records keep old filenames).
  </acceptance_criteria>
  <done>
    File moved with git history preserved, internal links still resolve, `mkdocs build --strict` green. Wave 3 plan 30-08 will delete the now-empty `docs/connectors/` directory.
  </done>
</task>
