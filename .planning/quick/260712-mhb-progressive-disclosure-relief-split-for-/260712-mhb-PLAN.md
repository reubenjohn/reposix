---
phase: quick-260712-mhb
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md
  - .planning/milestones/v0.14.0-phases/surprises-intake/part-01.md
  - .planning/milestones/v0.14.0-phases/surprises-intake/part-02.md
  - quality/gates/agent-ux/p110-surprises-absorption.sh
  - quality/catalogs/agent-ux.json
autonomous: true
requirements:
  - "milestone-hygiene: SURPRISES-INTAKE.md under the p111 44000 B ceiling"
  - "OP-8 file-size drain (progressive disclosure)"

must_haves:
  truths:
    - "The live SURPRISES-INTAKE.md is well under the 44000 B p111-hygiene ceiling (target <= ~10000 B)."
    - "All 17 entry bodies are preserved byte-for-byte across surprises-intake/part-01.md + part-02.md (no deletion, no summarization)."
    - "The top-level file keeps its preamble (lines 1-33, through '## Entries') unchanged, followed by a '## Split index (OP-8 file-size drain)' section listing all 17 entries as one-liners (date | discovered-by | severity | terminal-status-word) under the two part-file links."
    - "bash quality/gates/agent-ux/p111-milestone-hygiene.sh exits 0."
    - "The drained-invariant gate (p110-surprises-absorption) still verifies 0 OPEN + >=10 terminal by reading the relocated part files."
  artifacts:
    - path: ".planning/milestones/v0.14.0-phases/surprises-intake/part-01.md"
      provides: "Entries 1-10 (verbatim) + Part-1-of-2 header"
      contains: "## 2026-07-11 23:00 | discovered-by: P102"
    - path: ".planning/milestones/v0.14.0-phases/surprises-intake/part-02.md"
      provides: "Entries 11-17 (verbatim) + Part-2-of-2 header"
      contains: "## 2026-07-12 08:35 | discovered-by: P105"
    - path: ".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
      provides: "Preamble + Split index (raw entries relocated out)"
      contains: "## Split index (OP-8 file-size drain)"
    - path: "quality/gates/agent-ux/p110-surprises-absorption.sh"
      provides: "Split-aware drained-invariant counter (scans top-level + part files)"
      contains: "SCAN_FILES"
  key_links:
    - from: ".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
      to: "surprises-intake/part-01.md, surprises-intake/part-02.md"
      via: "markdown links in the Split index"
      pattern: "surprises-intake/part-0[12]\\.md"
    - from: "quality/gates/agent-ux/p110-surprises-absorption.sh"
      to: "surprises-intake/part-*.md"
      via: "SCAN_FILES glob feeding the OPEN/terminal awk counts"
      pattern: "part-\\*\\.md"
---

<objective>
Relieve `.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md` — currently **43988 B**,
hard-capped at **44000 B** by `quality/gates/agent-ux/p111-milestone-hygiene.sh` assert E
(only 12 B headroom) — by relocating all 17 (terminal) entries into two sibling part files,
following the established OP-8 file-size-drain convention already used by the v0.13.0 intake.

v0.14.0 is a CLOSED milestone (awaiting owner tag). All 17 entries are terminal
(RESOLVED / RESOLVED-in-P10x / RESOLVED-by-P113 / DEFERRED / DEFERRED-TO-v0.15.0) — **zero
OPEN**. This is a verbatim relocation, not a deletion or summarization.

Purpose: Restore progressive-disclosure headroom so the next terminal row does not breach
the 44000 B ceiling, using the same tool + layout that drained the v0.13.0 intake.
Output:
- `surprises-intake/part-01.md` (entries 1-10, verbatim) + `part-02.md` (entries 11-17, verbatim)
- Rewritten `SURPRISES-INTAKE.md` = unchanged preamble + a "## Split index" section (target <= ~10000 B)
- `p110-surprises-absorption.sh` made split-aware so the drained-invariant health gate keeps verifying
  the relocated entries (fix-it-twice: the gate reads the exact file being split)

This is a DOC/planning-artifact relocation. NO cargo, NO build, NO test-suite, NO push.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md
@.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md
@.planning/milestones/v0.13.0-phases/surprises-intake/part-01.md
@scripts/split_ledger.py
@quality/gates/agent-ux/p111-milestone-hygiene.sh
@quality/gates/agent-ux/p110-surprises-absorption.sh

<facts>
Measured from the current committed file (all read-only, already verified during planning):

- Total file: 43988 B. Preamble (lines 1-33, ending at the `## Entries` heading): 1299 B.
  Line 33 is exactly `## Entries`; line 35 is the first `## 20..` entry header.
- Movable entries block (line 35 -> EOF): 42688 B, 17 entries at `^## 20YY-` boundaries:

  | # | line | header (date \| discovered-by \| severity) | terminal STATUS word |
  |---|------|--------------------------------------------|----------------------|
  | 1 | 35  | 2026-07-11 23:00 \| discovered-by: P102 (adversarial code-review) \| severity: HIGH | RESOLVED-in-P102 |
  | 2 | 55  | 2026-07-11 23:00 \| discovered-by: P102 (adversarial code-review) \| severity: HIGH | RESOLVED-in-P102 |
  | 3 | 74  | 2026-07-11 23:00 \| discovered-by: P102 (adversarial code-review) \| severity: MEDIUM | RESOLVED-in-P102 |
  | 4 | 91  | 2026-07-11 23:00 \| discovered-by: P102 (adversarial code-review) \| severity: HIGH | RESOLVED-in-P102 |
  | 5 | 109 | 2026-07-11 23:00 \| discovered-by: P102 (adversarial code-review) \| severity: MEDIUM | RESOLVED-in-P102 |
  | 6 | 131 | 2026-07-12 07:13 \| discovered-by: P104 (github-helper-path 404 fix verifier) \| severity: MEDIUM | DEFERRED-TO-v0.15.0 |
  | 7 | 149 | 2026-07-12 07:35 \| discovered-by: v0.14.0 health-triage lane (main gate sweep) \| severity: MEDIUM | DEFERRED |
  | 8 | 192 | 2026-07-12 07:40 \| discovered-by: v0.14.0 health-triage lane (main gate sweep) \| severity: LOW | RESOLVED |
  | 9 | 238 | 2026-07-12 07:13 \| discovered-by: P104 (github-helper-path 404 fix verifier) \| severity: MEDIUM | DEFERRED-TO-v0.15.0 |
  | 10 | 255 | 2026-07-12 08:10 \| discovered-by: P105 (RBF-LR-03 rebase-recovery research) \| severity: HIGH | RESOLVED |
  | 11 | 313 | 2026-07-12 08:35 \| discovered-by: P105 (RBF-LR-03 rebase-recovery gate, Lane 2) \| severity: HIGH | RESOLVED-in-P105 |
  | 12 | 394 | 2026-07-12 09:40 \| discovered-by: P105 (RBF-LR-03 docs fix-twice lane, ownership noticing) \| severity: MEDIUM | DEFERRED-TO-v0.15.0 |
  | 13 | 433 | 2026-07-12 \| discovered-by: D2 re-seal Wave 1 (shell/planning lane) \| severity: HIGH | DEFERRED-TO-v0.15.0 |
  | 14 | 487 | 2026-07-12 15:57 \| discovered-by: C2-wave-2 (CI-gate fix-twice) \| severity: MEDIUM | RESOLVED |
  | 15 | 510 | 2026-07-12 \| discovered-by: GSD-quick (release-plz RED fix) \| severity: MEDIUM | DEFERRED |
  | 16 | 545 | 2026-07-12 \| discovered-by: GSD-quick (fleet-safety untrack fix) \| severity: MEDIUM | RESOLVED |
  | 17 | 576 | 2026-07-12 20:59 \| discovered-by: P111 (milestone-close CI-wait) \| severity: MEDIUM | RESOLVED |

  (The status-word column is the EXPECTED output for the Split-index derivation in Task 2 —
  use it only as a self-check against the scripted extraction, never transcribe by hand.)

- `scripts/split_ledger.py FILE --first-entry-line N --budget BYTES` does the verbatim split:
  keeps lines [1..N-1] as preamble in the rewritten INDEX, partitions line N..EOF at `^## `
  boundaries, greedily packs entries into `<stem>/part-NN.md` parts (stem = `surprises-intake`),
  writes each part with header `<title-line> — Part K of M` + a `> Split from ...` blockquote,
  and runs a built-in byte-exact round-trip check (rebuilt == orig_body) that returns exit 1
  on any mismatch. Budget 24000 packs entries 1-10 into part-01 (~21.2k) and 11-17 into
  part-02 (~21.2k) = exactly 2 balanced parts.

- The p111-hygiene gate (assert E) caps ONLY the top-level file at 44000 B and (assert F)
  requires 0 `**STATUS:** OPEN` lines in it (fence-aware). After the split the top-level file
  is ~4k with zero entry bodies -> both pass with huge headroom.

- HIDDEN CONSEQUENCE (must be handled): `quality/gates/agent-ux/p110-surprises-absorption.sh`
  (catalog row `agent-ux/p110-surprises-absorption`, cadence on-demand, blast_radius P2 — never
  blocks a push, currently committed status FAIL) counts `**STATUS:** (RESOLVED|DEFERRED|WONTFIX)`
  lines IN THE TOP-LEVEL FILE and requires `>=10`. Relocating the entries empties that count, so
  a live run would newly FAIL assert #3. The entries are RELOCATED, not deleted, so the honest
  fix (per CLAUDE.md "health is a maintained asset" + "fix it twice") is to make that verifier
  read the part files too — Task 3. (This differs from the v0.13.0 p87 row, which was WAIVED
  because that milestone's CARRY-FORWARD BANNER genuinely DELETED entries. Here nothing is
  deleted, so keep the gate truthfully green rather than waiving it.)
</facts>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Relocate all 17 entries into 2 verbatim part files via split_ledger.py</name>
  <files>
    .planning/milestones/v0.14.0-phases/surprises-intake/part-01.md (new),
    .planning/milestones/v0.14.0-phases/surprises-intake/part-02.md (new),
    .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md (rewritten by the tool; index reformatted in Task 2)
  </files>
  <action>
Use the committed, proven drain tool — do NOT hand-roll the body move (byte-exact relocation
of 42688 B is exactly what the tool + its built-in round-trip check exist for).

Run each command block after `cd /home/reuben/workspace/reposix` in the SAME Bash invocation
(cwd resets between calls). This edits planning docs in the shared repo — normal GSD behavior,
NOT a leaf/reposix/sim/git-config setup, so the leaf-isolation hook does not apply.

1. Capture the pristine entries block from git BEFORE the tool rewrites the file (needed for the
   independent byte-exact check in Task 2):

   ```
   cd /home/reuben/workspace/reposix
   WORK=$(mktemp -d)
   echo "$WORK" > /tmp/mhb-workdir   # stash the path so later Bash calls can re-read it
   git show HEAD:.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md | tail -n +35 > "$WORK/entries-orig.txt"
   wc -c "$WORK/entries-orig.txt"    # expect 42688
   ```

2. Run the split (first entry is line 35; budget 24000 -> exactly 2 parts):

   ```
   cd /home/reuben/workspace/reposix
   python3 scripts/split_ledger.py \
     .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md \
     --first-entry-line 35 --budget 24000
   ```

   The tool prints `OK: ... -> INDEX (N bytes) + 2 parts ...` and per-part sizes, and returns
   non-zero if the verbatim round-trip fails. If it reports anything other than exactly 2 parts,
   STOP and report — do not proceed.

The tool creates the part files with the canonical header
`# v0.14.0 Surprises Intake (P110 source-of-truth) — Part K of 2` + the `> Split from ...`
blockquote (identical shape to the v0.13.0 `surprises-intake/part-01.md` reference), and
rewrites the top-level file to preamble + a provisional `## Split index` section. Task 2
reformats that index to add the terminal-status-word column.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && test -f .planning/milestones/v0.14.0-phases/surprises-intake/part-01.md && test -f .planning/milestones/v0.14.0-phases/surprises-intake/part-02.md && test ! -e .planning/milestones/v0.14.0-phases/surprises-intake/part-03.md && [ "$(grep -cE '^## 20[0-9][0-9]-' .planning/milestones/v0.14.0-phases/surprises-intake/part-01.md)" = 10 ] && [ "$(grep -cE '^## 20[0-9][0-9]-' .planning/milestones/v0.14.0-phases/surprises-intake/part-02.md)" = 7 ] && echo "OK 2 parts 10+7"</automated>
  </verify>
  <done>Exactly two part files exist (no part-03); part-01 holds 10 entry headers, part-02 holds 7; split_ledger.py reported its round-trip check passed (exit 0).</done>
</task>

<task type="auto">
  <name>Task 2: Reformat the top-level Split index (status-word one-liners) + independent byte-exact proof</name>
  <files>.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md</files>
  <action>
Rewrite the top-level file so the "## Split index" section matches the v0.13.0 convention
VERBATIM in shape but adds the terminal-status-word per the task spec
(`date | discovered-by | severity | terminal-status-word`). Preamble (lines 1-33, through
`## Entries`) MUST stay byte-identical.

Run after `cd /home/reuben/workspace/reposix`:

```
cd /home/reuben/workspace/reposix
F=.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md
DIR=.planning/milestones/v0.14.0-phases/surprises-intake
WORK=$(cat /tmp/mhb-workdir)

# Guard: line 33 must still be the '## Entries' heading (split_ledger preserved the preamble).
[ "$(sed -n '33p' "$F")" = "## Entries" ] || { echo "FAIL: preamble boundary moved"; exit 1; }

# Per-part bullet builder: emit '  - <header-without-"## "> | <first STATUS token>'.
bullets() {
  awk '
    function flush() { if (h != "") printf "  - %s | %s\n", h, s }
    /^## 20[0-9][0-9]-/ { flush(); h=substr($0,4); s=""; next }
    h != "" && s == "" && /^\*\*STATUS:\*\*/ {
      t=$0; sub(/^\*\*STATUS:\*\*[[:space:]]*/, "", t);
      match(t, /^[^ ,(]+/); s=substr(t, RSTART, RLENGTH); sub(/[.,]$/, "", s)
    }
    END { flush() }
  ' "$1"
}

n1=$(grep -cE '^## 20[0-9][0-9]-' "$DIR/part-01.md")
n2=$(grep -cE '^## 20[0-9][0-9]-' "$DIR/part-02.md")

{
  head -n 33 "$F"
  printf '\n## Split index (OP-8 file-size drain)\n\n'
  printf 'This ledger approached the milestone-hygiene 44000 B ceiling (`quality/gates/agent-ux/p111-milestone-hygiene.sh` assert E) and was split into 2 per-part child files under `surprises-intake/` via `scripts/split_ledger.py` (byte-exact round-trip verified). Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here. All %d v0.14.0 entries are terminal (RESOLVED / DEFERRED) — zero OPEN.\n\n' "$((n1 + n2))"
  printf -- '- [`surprises-intake/part-01.md`](surprises-intake/part-01.md) — %d entries:\n' "$n1"
  bullets "$DIR/part-01.md"
  printf -- '- [`surprises-intake/part-02.md`](surprises-intake/part-02.md) — %d entries:\n' "$n2"
  bullets "$DIR/part-02.md"
} > "$WORK/new-index.md"

# Preamble must be byte-identical before we install the rewrite.
cmp <(head -n 33 "$F") <(head -n 33 "$WORK/new-index.md") && cp "$WORK/new-index.md" "$F"
wc -c "$F"   # expect well under 44000, ~4000
```

Then run the independent byte-exact relocation proof (belt-and-suspenders on top of the tool's
own round-trip check) — concatenating the two part BODIES (each part minus its 4-line header:
title, blank, blockquote, blank) must reproduce the pristine entries block byte-for-byte:

```
cd /home/reuben/workspace/reposix
DIR=.planning/milestones/v0.14.0-phases/surprises-intake
WORK=$(cat /tmp/mhb-workdir)
cat <(tail -n +5 "$DIR/part-01.md") <(tail -n +5 "$DIR/part-02.md") > "$WORK/recombined.txt"
cmp "$WORK/recombined.txt" "$WORK/entries-orig.txt" && echo "BYTE-EXACT: part bodies == original entries block"
```

Sanity-check the emitted Split index against the status-word column in <facts> (10 bullets under
part-01, 7 under part-02, correct status words). If any bullet's status word disagrees, STOP —
the extraction is wrong; do not hand-patch.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && DIR=.planning/milestones/v0.14.0-phases/surprises-intake && WORK=$(cat /tmp/mhb-workdir) && F=.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md && cat <(tail -n +5 "$DIR/part-01.md") <(tail -n +5 "$DIR/part-02.md") > "$WORK/recombined.txt" && cmp "$WORK/recombined.txt" "$WORK/entries-orig.txt" && [ "$(wc -c < "$F")" -lt 10000 ] && grep -q '^## Split index (OP-8 file-size drain)$' "$F" && [ "$(grep -cE '^  - 20[0-9][0-9]-' "$F")" = 17 ] && echo "OK byte-exact + index + <10k"</automated>
  </verify>
  <done>Part bodies recombine byte-for-byte to the original entries block (cmp exit 0); top-level file is < 10000 B; it contains the "## Split index (OP-8 file-size drain)" heading with 17 status-word one-liner bullets under the two part links; preamble unchanged (cmp exit 0).</done>
</task>

<task type="auto">
  <name>Task 3: Keep the drained-invariant gate honest (split-aware) + run the required hygiene gate</name>
  <files>quality/gates/agent-ux/p110-surprises-absorption.sh, quality/catalogs/agent-ux.json</files>
  <action>
The relocation empties the top-level file that `p110-surprises-absorption.sh` counts, so its
`>=10 terminal` assert would newly FAIL even though every entry still exists (in the part files).
Make the verifier read the part files too, so the drained-invariant stays TRUTHFULLY verified
(entries relocated, not deleted). This is fix-it-twice on the gate that reads the split file —
NOT application/cargo code.

Edit `quality/gates/agent-ux/p110-surprises-absorption.sh`:

1. Immediately AFTER the `INTAKE=".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"`
   line, insert a split-aware scan list (falls back to just the top-level file when no parts
   exist, so it stays correct pre-split too):

   ```
   PARTS_DIR=".planning/milestones/v0.14.0-phases/surprises-intake"
   SCAN_FILES=("${INTAKE}")
   # OP-8 file-size drain (2026-07-12): entries relocated verbatim into part files; count across both.
   if compgen -G "${PARTS_DIR}/part-*.md" > /dev/null 2>&1; then
     SCAN_FILES+=("${PARTS_DIR}"/part-*.md)
   fi
   ```

2. In BOTH awk blocks (the OPEN_COUNT one and the TERMINAL_COUNT one):
   - add `FNR==1 { in_fence=0 }` as the FIRST rule (resets the code-fence state per file so a
     fence in one file cannot bleed into the next), and
   - change the awk input from `"${INTAKE}"` to `"${SCAN_FILES[@]}"`.

   Leave the fence toggle and the `**STATUS:** OPEN` / `**STATUS:** (RESOLVED|DEFERRED|WONTFIX)`
   match logic exactly as-is. (The top-level preamble still carries the schema-template
   `**STATUS:** OPEN` line inside a ```markdown fence — fence-awareness must keep skipping it;
   that is why the FNR reset + existing toggle both stay.)

3. Keep the file's `INTAKE`-based existence check and the `HONESTY` / `ARTIFACT` paths unchanged.

Then update the catalog row so its contract matches reality (append-only, JSON must stay valid):
in `quality/catalogs/agent-ux.json`, on the `agent-ux/p110-surprises-absorption` row, add
`".planning/milestones/v0.14.0-phases/surprises-intake/part-*.md"` to its `evidence` array and
append one sentence to its `comment`, e.g.:
`"OP-8 (2026-07-12, quick 260712-mhb): the 17 entries were relocated verbatim into surprises-intake/part-*.md to relieve the p111 44000 B ceiling; the verifier now counts OPEN/terminal STATUS across the top-level index AND the part files (entries preserved, not deleted — byte-exact round-trip verified)."`
Do NOT flip the row `status` or add a `waiver` — the gate is truthfully green after the fix.

Now run BOTH gates and confirm exit 0:

```
cd /home/reuben/workspace/reposix
bash quality/gates/agent-ux/p110-surprises-absorption.sh; echo "p110 exit: $?"
bash quality/gates/agent-ux/p111-milestone-hygiene.sh;  echo "p111 exit: $?"
python3 -c "import json,sys; json.load(open('quality/catalogs/agent-ux.json')); print('agent-ux.json valid JSON')"
```

p110 must print `PASS: SURPRISES-INTAKE drained (0 OPEN, 17 terminal); honesty spot-check artifact present`
and exit 0. p111 must print its `PASS: P111 milestone-close hygiene ...` line and exit 0.
If p110 reports a terminal count other than 17, or either gate is non-zero, STOP and report.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && bash quality/gates/agent-ux/p110-surprises-absorption.sh && bash quality/gates/agent-ux/p111-milestone-hygiene.sh && python3 -c "import json; json.load(open('quality/catalogs/agent-ux.json'))" && echo "BOTH GATES PASS + catalog valid"</automated>
  </verify>
  <done>`p111-milestone-hygiene.sh` exits 0 (REQUIRED — the whole reason for the split); `p110-surprises-absorption.sh` exits 0 reporting 17 terminal / 0 OPEN by reading the part files; `agent-ux.json` is valid JSON with the row's evidence/comment reflecting the split.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| (none introduced) | This is a planning-artifact relocation + a shell health-gate maintenance edit. No new runtime code path, no external/tainted input crosses any boundary, no egress surface, no `Tainted<T>` handling. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-mhb-01 | Tampering (content loss) | verbatim move of 42688 B across part files | mitigate | split_ledger.py's built-in byte-exact round-trip check PLUS an independent `cmp` of concatenated part bodies vs. the git-pristine entries block (Task 2). |
| T-mhb-02 | Tampering (silent gate regression) | p110-surprises-absorption drained-invariant gate | mitigate | Make the verifier split-aware (Task 3) and run it to exit 0 — the health gate keeps verifying the relocated entries instead of silently failing. |
</threat_model>

<verification>
Phase-level checks (all doc/gate, no cargo/build/test-suite):

1. `wc -c .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md` — well under 44000 B (target < 10000).
2. Part bodies recombine byte-for-byte to the original entries block (`cmp`, Task 2) — no content lost.
3. `bash quality/gates/agent-ux/p111-milestone-hygiene.sh` exits 0 (the required acceptance gate).
4. `bash quality/gates/agent-ux/p110-surprises-absorption.sh` exits 0 (17 terminal, 0 OPEN, split-aware).
5. Preamble (lines 1-33, through `## Entries`) byte-identical to the pre-split original.
6. `agent-ux.json` remains valid JSON.
</verification>

<success_criteria>
- SURPRISES-INTAKE.md < 10000 B; two sibling part files hold all 17 entries verbatim (byte-exact proven).
- Top-level file = unchanged preamble + a "## Split index (OP-8 file-size drain)" section with 17
  `date | discovered-by | severity | terminal-status-word` one-liners linking the two part files,
  matching the v0.13.0 convention.
- `p111-milestone-hygiene.sh` AND `p110-surprises-absorption.sh` both exit 0.
- No cargo/build/test-suite invocation anywhere; no push.
</success_criteria>

<output>
After completion, create `.planning/quick/260712-mhb-progressive-disclosure-relief-split-for-/260712-mhb-SUMMARY.md`.
</output>
