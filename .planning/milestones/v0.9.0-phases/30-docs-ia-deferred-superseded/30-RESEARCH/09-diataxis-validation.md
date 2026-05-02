← [back to index](./index.md) · phase 30 research

## Diátaxis Validation of the Proposed IA

The source-of-truth note's IA sketch holds up well against Diátaxis — one flag, no blockers.

| Proposed page | Diátaxis category | Validation |
|---------------|-------------------|------------|
| Home (index.md) | (narrative hero, above Diátaxis) | PASS — Diátaxis explicitly does not cover marketing/landing pages. |
| Why reposix | Explanation | PASS — "why" is Explanation by definition. |
| Mental model in 60 seconds | Explanation | PASS — propositional, study-mode. |
| reposix vs MCP / SDKs | Explanation | PASS — comparison is a form of Explanation. |
| Try it in 5 minutes | Tutorial | PASS — guided first encounter. |
| How it works / filesystem.md | Explanation | PASS — propositional architecture. |
| How it works / git.md | Explanation | PASS. |
| How it works / trust-model.md | Explanation | PASS. |
| Connect to GitHub / Jira / Confluence | How-to | PASS — working developer, specific goal. |
| Write your own connector | How-to | PASS — working developer, specific goal. Has a Reference dependency (the trait). |
| Integrate with your agent | How-to | PASS — work-mode. |
| Running two agents safely | How-to | PASS. |
| Custom fields and frontmatter | How-to | **FLAG** — could cross into Reference. See note below. |
| Troubleshooting | How-to | PASS — specific problem, specific fix, in work-mode. |
| Reference / CLI | Reference | PASS. |
| Reference / HTTP API | Reference | PASS. |
| Reference / Simulator | Reference | PASS — it's dev tooling, user looks up flags. |
| Reference / git-remote-reposix | Reference | PASS. |
| Reference / Frontmatter schema | Reference | PASS — describes the schema. |
| Decisions (ADRs) | Reference (decision records) | PASS. |
| Research | Explanation | PASS — study-mode long-form. |

**One flag — "Custom fields and frontmatter":** The phrase "how to use custom fields" is How-to, but if the page ends up describing the YAML schema field-by-field it drifts into Reference. **Recommendation:** split into two pages if content grows — a short how-to guide ("how do I add a custom field to my issue?") and a Reference page ("the frontmatter schema"). For Phase 30, ship as a How-to stub; let post-launch usage drive the split.

**Framework citation:** `[CITED: diataxis.fr/start-here]`.

## Mental Model in 60 seconds — format guidance

Per competitor pattern F (Turso) and the source-of-truth note's explicit three-key enumeration (*mount = git working tree · frontmatter = schema · `git push` = sync verb*):

**Recommended structure:**
- ~300-400 words total (Turso /concepts precedent)
- Three H2 sections, one per conceptual key
- Each section: 1 sentence of equation, 1-2 sentences of explanation, 1 code snippet (3-5 lines)
- Zero diagrams — diagrams are the payoff of how-it-works, not the setup
- End with a "Now what" block pointing to `/tutorial/` or `/how-it-works/`

**Source-of-truth key phrasings (use verbatim):**
1. "**mount = git working tree**"
2. "**frontmatter = schema**"
3. "**`git push` = sync verb**"

These three phrasings are locked; the planner and copy subagent do not re-derive them.

## Tutorial Pattern — 5-minute first run

**Recommended structure (per Pattern H — Cloudflare Workers):**

1. **Prerequisites (30 seconds).** One bullet list: `cargo`, `fuse3` (Linux), `git`, `curl`, `jq`. No paragraphs.
2. **Step 1: Start the simulator (1 min).** `target/release/reposix-sim --bind 127.0.0.1:7878 --seed-file ...`. Show the expected output. `curl /healthz` to verify.
3. **Step 2: Mount the tracker as a folder (1 min).** `reposix mount /tmp/reposix-mnt --backend http://127.0.0.1:7878`. `ls /tmp/reposix-mnt/issues/`.
4. **Step 3: Edit an issue (1 min).** `cat` the issue. Use `printf > file` (not `sed -i` — per existing demo.md step 6 guidance on FUSE filename constraints).
5. **Step 4: `git push` the change (1 min).** `git init`, `git remote add`, `git push`. `curl` the simulator to see the `version` bumped 1 → 2 (this is the "aha" per Pattern G).
6. **What just happened (30 seconds).** Three-sentence recap linking to `/how-it-works/` for the reveal.

**Total: ~5 minutes if the reader types, ~3 if they paste.** The "aha" hits in step 4. Do NOT save the aha for step 6.

**Cleanup guidance:** End with `fusermount3 -u /tmp/reposix-mnt && pkill reposix-sim`. Mirrors `scripts/demo.sh` step 9 pattern.

**Source:** `docs/demo.md` already contains step-accurate content for steps 3/4/6/7 — carve from there. Simulator-first per CLAUDE.md OP #1.
