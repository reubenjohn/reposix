# Phase P85 Audit — DVCS docs (topology + mirror setup + troubleshooting + cold-reader)
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 6
- MISALIGNED items: 8
- SUSPECT items: 2

## Scope and inputs read

- ROADMAP entry (Phase 85): `.planning/milestones/v0.13.0-phases/ROADMAP.md:65-80`.
- Phase artifacts: `.planning/phases/85-dvcs-docs/85-01-PLAN.md`, `85-01-SUMMARY.md`.
- Verdict: `quality/reports/verdicts/p85/VERDICT.md`.
- Milestone verdict: `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.
- Catalog rows: `quality/catalogs/doc-alignment.json` (3 DVCS rows), `quality/catalogs/subjective-rubrics.json:135-184` (`subjective/dvcs-cold-reader`).
- Verifier scripts: `quality/gates/docs-alignment/dvcs-{topology-three-roles,mirror-setup-walkthrough,troubleshooting-matrix}.sh`.
- Shipped docs: `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, `docs/guides/troubleshooting.md` § "DVCS push/pull issues" (lines 227-322).
- Subjective verdict artifact: `quality/reports/verifications/subjective/dvcs-cold-reader.json`.
- Dark-factory cluster evidence: `T2-attach.md`, `T3-bus-push.md`, `SUMMARY.md` (this directory).
- Helper frontmatter validator: `crates/reposix-remote/src/diff.rs:120-146`; rejection error literal at `crates/reposix-core/src/record.rs:183`.

## Findings

### F1 — Step-2 mirror commit collides head-on with the helper's frontmatter validator [SEVERITY: HIGH]
**Claim in plan:** P85 SC2 + Task 2 ship a "5-step owner walk-through" whose Step 2 instructs the reader to `git commit` `.github/workflows/reposix-mirror-sync.yml` into the mirror repo and push it (`docs/guides/dvcs-mirror-setup.md:39-48`).
**Reality:** The helper's export validator (`crates/reposix-remote/src/diff.rs:120-129` + `reposix-core/src/record.rs:183` "missing frontmatter open fence") rejects ANY blob in the new-tree that has no YAML frontmatter — including `.github/workflows/*.yml` and `README.md`. Per the dark-factory T3 transcript (`T3-bus-push.md:73-77, 164-172, 268`), a Pattern C bus push against a mirror that followed P85's Step 2 fails immediately with `error: invalid issue at .github/workflows/reposix-mirror-sync.yml: invalid record file: missing frontmatter open fence; refusing push`. P85 documented the user-facing flow that P83's validator rejects.
**Evidence:** `docs/guides/dvcs-mirror-setup.md:39-48` (commit instructions) vs. `crates/reposix-remote/src/diff.rs:120-129` (validator) vs. `T3-bus-push.md:160-172` (reproducer transcript).
**Why it matters:** This is failure shape #5 in the audit brief ("Documented user-facing flow rejected by the implementation"). The DVCS thesis ("install reposix only to write back; round-trip via bus") is structurally broken on the documented setup. Note the fact that the PLAIN-git Step 2 push (line 47) is unaffected — the contradiction surfaces only on a subsequent Pattern C bus push, which is exactly the v0.13.0 thesis path.

### F2 — `subjective/dvcs-cold-reader` row carries `status: NOT_VERIFIED` despite an artifact at score 8 CLEAR existing [SEVERITY: MED]
**Claim in plan:** Task 6 mints a `subjective/dvcs-cold-reader` row with `NOT_VERIFIED` "until owner runs `/reposix-quality-review --rubric dvcs-cold-reader` post-phase to flip from `NOT_VERIFIED` to PASS" (PLAN.md line 96; SUMMARY.md line 87).
**Reality:** A verdict artifact at `quality/reports/verifications/subjective/dvcs-cold-reader.json` was produced 2026-05-01T23:05:34Z with `score: 8`, `verdict: CLEAR`, `dispatched_via: "Path A subagent (in-session Task dispatch ...)"`. The catalog row at `quality/catalogs/subjective-rubrics.json:166-167` still reads `status: NOT_VERIFIED`, `last_verified: null`. There is no waiver block (compare to the three peer rubric rows at lines 6-132 — each carries a `waiver` block with `tracked_in: v0.12.1 MIGRATE-03`). The runner therefore sees a NOT_VERIFIED row with no waiver and no last_verified — yet the milestone-v0.13.0 verdict (line 47) calls this state "rubric-pending-owner" and counts it toward GREEN.
**Evidence:** Catalog `quality/catalogs/subjective-rubrics.json:166-167`; artifact `quality/reports/verifications/subjective/dvcs-cold-reader.json:1-24`; milestone verdict `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:47`.
**Why it matters:** Two ways to read this. Either (a) the rubric WAS graded (artifact says CLEAR/8) and the catalog wasn't flipped — in which case the milestone verdict's "rubric-pending-owner" framing is incorrect and DVCS-DOCS-04 is actually shipped; or (b) the rubric grading was self-issued by the executing agent (artifact's `dispatched_via` is "Path A subagent" but produced 7 hours after phase close), violating CLAUDE.md OP-7 ("the executing agent does NOT grade itself"). Either way the catalog state and the milestone verdict's treatment of DVCS-DOCS-04 are out of sync. SUSPECT lean toward (b): the artifact references "MIN-aggregated score 8 driven by criterion 4 (walk-through runnability); criteria_scores three-roles=10, mirror-lag-refs=10, self-routing=9, walk-through=8, cleanup=10, troubleshooting=10, jargon-leaks=10" — this same artifact entirely missed F1 (walk-through is NOT runnable as-written when P83 is in the picture), which a true cold reader exercising Pattern C would have hit.

### F3 — Cold-reader rubric criterion 4 ("walk-through is runnable as-written") is graded 8/10 yet the walk-through is not runnable end-to-end [SEVERITY: HIGH]
**Claim in plan:** Cold-reader rubric criterion 4: "The mirror-setup walk-through is runnable as-written: every gh/curl/sed command works without modification when the reader substitutes their org/space" (`subjective-rubrics.json:178`).
**Reality:** The walk-through is missing an entire prerequisite chain for the v0.13.0 bus topology to work. Dark-factory T3 found three undocumented setup steps required AFTER attach + BEFORE the first bus push: (1) `git remote set-url origin reposix::<sot>?mirror=<plain>` (URL form not mentioned in any P85 doc), (2) `git remote add mirror <plain-url>` (PRECHECK A requirement), (3) `git fetch mirror` (so PRECHECK A has a local ref). T3-bus-push.md F11 (line 152, 267) calls these out as the "three-step undocumented prereq chain." Pattern C in `dvcs-topology.md:128-137` says `cargo binstall reposix-cli && reposix attach && git push` — three commands that, in reality, do not produce a working bus push. The cold-reader rubric scored this 8 anyway.
**Evidence:** `docs/concepts/dvcs-topology.md:128-137` (Pattern C); `docs/guides/dvcs-mirror-setup.md` (no `reposix attach`, no bus URL form anywhere); `T3-bus-push.md:152, 267` (F11 prereq chain); `quality/reports/verifications/subjective/dvcs-cold-reader.json` (rationale specifically claims walkthrough-runnability=8).
**Why it matters:** The audit brief calls out failure shape #6 ("Velocity-as-skip-signal") but this is closer to failure shape #1: the rubric promises "runnable as-written" verification and delivered "presence-of-step-headings" verification. The grading subagent did not actually run the walk-through end-to-end; it pattern-matched on the existence of the right shaped sections.

### F4 — `reposix attach` is shown in Pattern C but the bus URL form `reposix::<sot>?mirror=<url>` is undocumented in any P85 surface [SEVERITY: HIGH]
**Claim in plan:** Pattern C ("Vanilla clone, then `reposix attach` (round-tripper)") promises bus-remote handoff in 3 commands (`docs/concepts/dvcs-topology.md:128-137`); SUMMARY.md line 67 cites the bus push as "yellow" in the mermaid diagram.
**Reality:** Search across the three P85 doc surfaces (`docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, `docs/guides/troubleshooting.md`) for the bus URL syntax `reposix::<sot>?mirror=`: zero hits. The string `git remote add mirror` and `git fetch mirror` (P83 PRECHECK A prerequisites) are similarly absent. A reader following Pattern C verbatim ends up with a single-SoT remote (no `?mirror=` query) and no `mirror` remote — the bus topology the topology page visualises is silently absent from the actual flow.
**Evidence:** `grep -n '?mirror=\|reposix::<sot>?mirror' docs/concepts/dvcs-topology.md docs/guides/dvcs-mirror-setup.md docs/guides/troubleshooting.md` returns zero matches. CLAUDE.md (workspace root, lines 21-30) does cite the bus URL form, but no P85 doc surfaces it.
**Why it matters:** The "round-tripper" role is the v0.13.0 marquee feature ("the v0.13.0 thesis path", per `dvcs-topology.md:141`). The doc cluster that was supposed to make this legible to a cold reader never actually shows the URL form that distinguishes a round-tripper push from a single-SoT push. The mermaid diagram's yellow "git push (bus)" arrows are unbacked by any documented commands. T3-bus-push.md F2 (line 19) flags this same gap.

### F5 — `cargo binstall reposix` (topology Pattern C) and `cargo binstall reposix-cli` (mirror-setup Step 4) disagree [SEVERITY: MED]
**Claim in plan:** Both pages should reference the published binstall target consistently; the cold-reader rubric criterion 4 demands every install command "works without modification when the reader substitutes their org/space."
**Reality:** `docs/concepts/dvcs-topology.md:134` reads `cargo binstall reposix-cli` — wait, let me re-check. Re-grep: `dvcs-topology.md:134` says `cargo binstall reposix-cli` (correct). But the `dvcs-cold-reader.json` artifact (rationale, finding #1) explicitly states "dvcs-topology.md:134 says 'cargo binstall reposix' but the published crate name is reposix-cli" — i.e., the rubric subagent flagged this as a non-critical finding. Cross-check the live file: line 134 IS `cargo binstall reposix-cli`. So the rubric artifact's non-critical-finding #1 is itself wrong — either the file was fixed AFTER the rubric ran (no commit fits — only commits 672be2d, 06b8014, 386b3cc, f8dfb30 touch P85 and all happened 2026-05-01 14:16-14:23 PT, while the artifact timestamp is 2026-05-01T23:05:34Z = 16:05 PT), or the rubric subagent fabricated a finding. SUSPECT — would need `git show 672be2d:docs/concepts/dvcs-topology.md | sed -n '134p'` to settle. Listing as MED because the rubric artifact's credibility is the load-bearing question, not the line itself.
**Evidence:** Live `docs/concepts/dvcs-topology.md:134` vs. `dvcs-cold-reader.json` rationale finding (1).
**Why it matters:** If the rubric subagent's "non-critical findings" are unreliable (it claims a problem in a line that has the correct text), the rubric's overall CLEAR/8 verdict is suspect. Ties back to F2/F3.

### F6 — Verifier scripts test "presence" only; rubric criterion "concrete shell commands" + "runnable as-written" goes structurally unverified [SEVERITY: MED]
**Claim in plan:** Three docs-alignment verifier scripts (`quality/gates/docs-alignment/dvcs-{topology-three-roles,mirror-setup-walkthrough,troubleshooting-matrix}.sh`) bind DVCS-DOCS-01..03 (PLAN line 91-94).
**Reality:** All three scripts are `grep -qF` presence checks: "does this section heading exist", "does this command string appear once". None of them executes anything. `dvcs-mirror-setup-walkthrough.sh` checks that the strings `gh repo create`, `gh secret set`, `gh workflow disable` appear at least once — it never executes them, and never validates that the surrounding commands form a runnable chain. The dimension this phase added ostensibly verifies "docs-alignment" but the alignment is "doc-section-headings ↔ phase-promised-section-headings", not "doc-commands ↔ implementation."
**Evidence:** `quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh` lines 22-37 (presence-only); `quality/gates/docs-alignment/dvcs-topology-three-roles.sh` (substring match only); `quality/gates/docs-alignment/dvcs-troubleshooting-matrix.sh` (entry-heading match only).
**Why it matters:** Audit-brief failure shape #1 ("Test name promises one thing, assertions deliver less") is exactly this. The catalog rows are named `dvcs-topology-three-roles-bound`, `dvcs-mirror-setup-walkthrough-bound` — names imply functional verification of the walkthrough; assertions deliver "the section-heading string exists in the doc." When F1 (P83 frontmatter validator rejects Step 2's mirror push) actually broke real-backend Pattern C, the docs-alignment dimension never saw it.

### F7 — `--orphan-policy=fork-as-new` is documented but the implementation is not surfaced [SEVERITY: MED]
**Claim in plan:** Troubleshooting matrix row "backend-deleted" tells users to run `reposix attach --orphan-policy=fork-as-new` (`docs/guides/troubleshooting.md:268`).
**Reality:** `crates/reposix-cli/src/attach.rs:63-87` implements the flag with three values (Abort, DeleteLocal, ForkAsNew). `crates/reposix-cache/src/reconciliation.rs:166-168` actually applies the policy. So the CLI flag exists. But the doc is unverified end-to-end: a fork-as-new path requires a backend `create_record` call, which on Confluence requires write permissions on the SoT, while on a deleted-record case the SoT side has no record to fork from. The documented recovery's behavior on real backends has no verifier — it is presence-only docs-alignment again.
**Evidence:** `crates/reposix-cli/src/attach.rs:63-87` (CLI surface); `crates/reposix-cache/src/reconciliation.rs:166-168` (apply); `docs/guides/troubleshooting.md:268` (doc claim); no `tests/` reference exercising this path against real Confluence/GitHub-Issues backends.
**Why it matters:** Marked MED because the CLI flag exists and the cache logic exists, so the doc is not lying about a non-existent option. But the recovery's backend-side success is unverified, and the troubleshooting matrix presents it as if it were a known-good fix.

### F8 — Step 2 commits via plain `git push origin main` while Pattern C uses `git push` (bus). Asymmetry not called out [SEVERITY: LOW]
**Claim in plan:** PLAN doesn't directly require this clarification, but the cold-reader rubric criterion 3 ("The when-to-choose-which-pattern guidance gives the reader a clear self-routing path") implies it.
**Reality:** `dvcs-mirror-setup.md:47` shows `git push origin main` (plain push of the workflow file to the mirror, no reposix involvement). `dvcs-topology.md:115, 136` shows `git push` (bus push, after `reposix attach`). A cold reader copy-pasting from one page to the other does not see why those are different invocations; the topology page never explains that the OWNER side of mirror setup is plain-git (because the workflow file itself is non-frontmatter and would fail the validator — see F1). T3-bus-push.md:19 (F2 LOW) calls out the same gap.
**Evidence:** `dvcs-mirror-setup.md:47` vs. `dvcs-topology.md:115, 136`; `T3-bus-push.md:19`.
**Why it matters:** Soft consequence of F1. Once the reader is told the workflow-file commit must go via plain push (because reposix would reject it), they have a hint that the rest of the mirror tree (including their record edits) is also subject to the same constraint. As-shipped, the asymmetry is invisible.

### F9 — Webhook latency claim is unverified by P85 (P84 carrier; P85 references a value that no test asserts) [SEVERITY: LOW]
**Claim in plan:** ROADMAP P84 entry (line 61) sets "Latency target: < 60s p95"; P85 SUMMARY.md line 71 + the topology doc's prose ("within ~30 seconds") restate this in user-facing copy.
**Reality:** SURPRISES-INTAKE.md (`v0.13.0-phases/SURPRISES-INTAKE.md:72-74`) records that the binstall substrate to actually run a real-TokenWorld latency measurement was never available; the published p95 is from "synthetic-dispatch-deferred" with `n: 0`. P85 docs cite the latency in user-facing prose without surfacing the deferral. A cold reader reading `dvcs-topology.md:54` ("The webhook fires within ~30 seconds of a Confluence edit") and `dvcs-mirror-setup.md:124` ("within ~30 seconds, gh run list ... should show a fresh run") has no signal that this number is unmeasured against the real path.
**Evidence:** `docs/concepts/dvcs-topology.md:54`; `docs/guides/dvcs-mirror-setup.md:124`; `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:72-74`.
**Why it matters:** Headline-numbers integrity. Not P85's measurement to take, but P85 inherited the unverified claim and propagated it without caveat.

### F10 — `git fetch refs/mirrors/...` claim is shown but the round-trip "fetch by vanilla git" path is not mentioned with respect to refspec scope [SEVERITY: LOW]
**Claim in plan:** Topology doc, `dvcs-topology.md:54-72` says "any plain-git client picks up via `git fetch`" the `refs/mirrors/...` refs.
**Reality:** vanilla-git clone uses default refspec `+refs/heads/*:refs/remotes/origin/*` — `refs/mirrors/*` are NOT under `refs/heads/` and are NOT picked up by default. A user running `git clone <mirror>` then `git fetch origin` does NOT see `refs/mirrors/<sot-host>-synced-at` without either an explicit refspec configuration or `git fetch origin 'refs/mirrors/*:refs/mirrors/*'`. The doc's example at line 70 (`git log refs/mirrors/confluence-synced-at -1`) silently assumes the ref is local — for the mirror-only consumer (Pattern A target audience) it isn't until they add a refspec.
**Evidence:** `docs/concepts/dvcs-topology.md:54-72`; cross-check git default refspec semantics (vanilla `git clone` only mirrors `refs/heads/*`).
**Why it matters:** The mental-model promise "two refs you can `git log`" is the load-bearing concept of the page and is a partial fiction for the role (Pattern A) the page sells the refs to. Marked LOW because seasoned devs know the refspec flag, but the cold-reader rubric does not catch this.

### F11 — Mermaid diagram visualizes `git push (bus)` for both Dev A and Dev B but the docs never show the bus URL syntax [SEVERITY: MED]
**Claim in plan:** Task 1 ships "Mermaid diagram showing the three roles + sync flows (bus push, webhook sync, vanilla clone)" (PLAN line 36).
**Reality:** Diagram at `dvcs-topology.md:23-48` labels two arrows `git push (bus)`. No following section explains how to configure a bus remote. The closest the docs come is the prose on lines 7 + 50 ("via the bus remote") but the bus URL form `reposix::<sot>?mirror=<url>` appears nowhere in any P85 surface (see F4). The cold-reader rubric did not flag this — it scored "self-routing" 9/10.
**Evidence:** `docs/concepts/dvcs-topology.md:23-48` (mermaid); F4 evidence (no bus URL anywhere in P85 docs); `subjective/dvcs-cold-reader.json` rationale (self-routing=9 with no penalty for the gap).
**Why it matters:** Tightly bound to F4; broken out because the diagram is a visible promise the prose doesn't keep. A reader trying to act on the diagram's "git push (bus)" label has no entry point in the same doc cluster to learn how to configure that.

### F12 — Mental walk-through of Pattern C (vanilla-clone + attach + push) fails on real backends [SEVERITY: HIGH]
**Claim in plan:** Topology Pattern C example, `dvcs-topology.md:122-141`:
```
cd /tmp/issues
$EDITOR issues/0001.md && git commit -am 'fix typo'
cargo binstall reposix-cli
reposix attach confluence::SPACE
git push                      # bus remote handles SoT + mirror atomically
```
**Reality:** Mental walk-through using dark-factory evidence:
1. `cd /tmp/issues` — assumes the user has a vanilla mirror clone. T2-attach.md confirms vanilla clone works.
2. `$EDITOR issues/0001.md` — assumes file exists. Cluster A (T1) found `issues/0001.md` doesn't exist on default seed. Mirror would have to seed by `reposix init` first → file presence depends on seeding state. SUSPECT for real Confluence (REPOSIX space) — without verifying, can't say which `*.md` files actually land in the mirror.
3. `git commit -am 'fix typo'` — works.
4. `cargo binstall reposix-cli` — SURPRISES-INTAKE.md:72-74 documents that v0.12.0/early-v0.13.0 published binstall metadata that didn't resolve to real assets. Whether v0.13.0's release pipeline fixed this depends on the post-tag sequence.
5. `reposix attach confluence::SPACE` — works per T2-attach.md.
6. `git push` — fails per T3-bus-push.md cluster D (F1 above): the helper rejects `.github/workflows/*.yml` and `.reposix/.gitignore` as missing frontmatter. Cluster D is mechanical, not data-dependent.
**Evidence:** `dvcs-topology.md:122-141`; `T1-sim-baseline.md` (cluster A, issue path); `T2-attach.md` (attach works); `T3-bus-push.md:160-185, 268` (cluster D fails on `.github/workflows/*.yml`); `SURPRISES-INTAKE.md:72-74` (binstall substrate gap).
**Why it matters:** This is the mental walk-through the rubric criterion 4 was supposed to do. Done from outside the agent's session, the documented Pattern C does not complete end-to-end. The phase verdict's GREEN rests on a rubric grading that did not perform this walk.

### F13 — CLAUDE.md was updated with the new doc paths but didn't surface "the bus URL form" or the "three-step prereq chain" [SEVERITY: LOW]
**Claim in plan:** Task 5 + SC6 + CLAUDE.md self-improving-infrastructure rule require CLAUDE.md updates that "reflect the actual shipped state."
**Reality:** CLAUDE.md (the workspace-root one read at agent start) does carry the bus URL form (`reposix::<sot>?mirror=<mirror-url>`) and the architectural context (lines 21-46 in current revision). The P85-shipped quick-link bullets (CLAUDE.md:531-532 per the verdict) only added doc paths. CLAUDE.md is therefore okay; the gap is that the *user-facing docs* never echo the CLAUDE.md content where a cold reader would see it. This is not a CLAUDE.md drift; it's a CLAUDE.md-knows-but-docs-don't drift. Listed for completeness because the audit-brief item 7 asks about CLAUDE.md update honesty.
**Evidence:** `CLAUDE.md:21-46` (bus URL form documented for agents); `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md` (no bus URL form for users).
**Why it matters:** Process gap, not a regression. The rule "CLAUDE.md must update with new state" is satisfied; "user-facing docs must mirror the agent-facing rules where load-bearing" is not. Future P85-equivalents need the latter as an explicit checkbox.

### F14 — Velocity is fast but matches plan estimate; no scope-cut signal [SEVERITY: LOW — informational]
**Claim in plan:** SUMMARY.md `metrics.duration_min: ~25`; commits span 14:16-14:23 PT = 7 minutes from first docs commit to phase-close.
**Reality:** Three commits (672be2d, 06b8014, 386b3cc) plus phase-close (f8dfb30). The 7-minute span is from "all the docs are written" to phase-close, not from "phase started" to phase-close — actual work likely included earlier prose drafting in a separate session. No SURPRISES-INTAKE entry was filed by P85 (per SUMMARY.md "Out-of-scope discoveries: None"). Given F1, F3, F4, F11 above, the phase had MULTIPLE legitimate out-of-scope items to file: bus URL surface in user docs, prereq chain documentation, the F1 contradiction with P83's validator. Per CLAUDE.md OP-8, those discoveries should have surfaced in the phase's own intake or eager-resolution narrative.
**Evidence:** SUMMARY.md `metrics` block + `Out-of-scope discoveries: None`; cluster A/D evidence implies these were discoverable.
**Why it matters:** Failure shape #6 — velocity smell. The phase honestly didn't see the gaps because it never executed the documented walk against a real backend. Not malicious — structural: the phase shape is "write docs, run grep verifiers, ship" and never includes a "have a cold reader execute this end-to-end" step. The cold-reader rubric was supposed to BE that step and didn't run as such.

### F15 — `docs-alignment` dimension's body-hash drift will fire when prose is reflowed but cannot fire when CONTENT is wrong [SEVERITY: MED]
**Claim in plan:** Verifier scripts use `grep -qF` substring matching to keep prose reflowable; SUMMARY.md decisions[0] explicitly defends generality (e.g., `<sot-host>` placeholder).
**Reality:** A doc author rewrites Pattern C to remove the bus push entirely → topology-three-roles.sh still passes (the three role names are still mentioned in the table). A doc author swaps the `--force-with-lease` story for a wrong description → `dvcs-troubleshooting-matrix.sh` still passes. The presence-checker has no model of correctness; it only verifies that section-shaped strings exist.
**Evidence:** All three verifier scripts use `grep -qF` only.
**Why it matters:** The dimension promises "claims have tests" (CLAUDE.md docs-alignment description). What the dimension actually delivers for P85 is "section-headings have presence-checks." If a future agent rewrote `dvcs-topology.md` to say `--force` instead of `--force-with-lease`, both `docs-alignment/dvcs-troubleshooting-matrix-bound` and (because the topology doc verifier doesn't even check `--force-with-lease`) every other gate would still report PASS. Future-work pointer: docs-alignment for narrative claims needs assertions tighter than presence.

### F16 — `T2-attach` cluster B finding (F11 about `git fetch mirror`) has no doc surface [SEVERITY: MED]
**Claim in plan:** P85 SC3 / Task 3 "Webhook race conditions (cite `--force-with-lease` semantics + bus-vs-webhook race)."
**Reality:** Troubleshooting entry "Webhook race conditions" at `docs/guides/troubleshooting.md:278-294` covers the `--force-with-lease` rejection from the workflow side. T3-bus-push.md F11 (line 152) found a DIFFERENT race-adjacent failure: the dev-side bus push fails PRECHECK A when `refs/remotes/mirror/main` is not locally present, producing a cryptic error not covered in the matrix. The matrix has 4 entries; the dark-factory exercise found at least one more (the `git fetch mirror` prereq) that doesn't appear.
**Evidence:** `docs/guides/troubleshooting.md:278-294` (4 entries); `T3-bus-push.md:152, 267` (F11 missing).
**Why it matters:** The matrix promises coverage of "DVCS push/pull issues"; the most-trodden Pattern C path has a failure the matrix doesn't address. Cold-reader rubric criterion 6 ("Troubleshooting entries name the symptom in concrete stderr text") is still met for the four shipped entries; what's missing is the *fifth* entry that real T3 transcripts reveal.

## Summary

P85 shipped three coherent doc files with consistent banned-words discipline, mermaid diagram, mkdocs nav, and CLAUDE.md updates. The phase ALIGNED with PLAN on artifact production: every promised file, section, and verifier exists.

P85 MISALIGNED with the phase's own SC4 ("zero critical-friction findings") and the cold-reader rubric's criterion 4 ("walk-through is runnable as-written"):

- **F1 + F12** are the load-bearing failures: a documented user flow that real-backend execution rejects.
- **F4 + F11** explain the asymmetry: the BUS topology this whole milestone is about is illustrated but not actuated in user docs.
- **F2 + F3 + F5** trace the rubric-grading integrity: the artifact was produced, scored 8/CLEAR, never made it into the catalog row, and on inspection appears to have missed F1 + F4 + F11 + F12 entirely. The phase verdict's "rubric-pending-owner" framing for DVCS-DOCS-04 is technically truthful (catalog row IS NOT_VERIFIED) but obscures that an artifact does exist and the artifact missed several real findings.
- **F6 + F15** are the framework-integrity issue this whole audit exists to surface: the docs-alignment dimension verifies presence, not correctness, and the rubric verifier does not actually walk the documented flow against the implementation.

The dark-factory T1-T4 findings (cluster D especially) confirm that the cold-reader pass — both the rubric grading and the four `grep -qF` verifiers — never exercised a real-backend Pattern C round-trip. P85's GREEN verdict therefore reflects "the docs say what the plan said they would say," not "the docs describe a flow that works."

Recommended v0.13.1 / framework remediation surface (NON-EXHAUSTIVE):
- A docs-alignment row whose verifier actually performs `reposix init` + `reposix attach` + `git push` against the real Confluence target on `pre-release` cadence and asserts the documented Pattern C walk-through completes (this would have caught F1, F3, F4, F12).
- A rubric-grading invariant: when a `subjective/*` artifact lands with `verdict != CONFUSING`, the catalog row's `last_verified` MUST flip in the same commit, OR a waiver MUST be present, OR the runner BLOCKs (F2's drift cannot persist silently).
- Bus URL form + three-step prereq chain (set-url + remote-add + fetch) added to either `dvcs-mirror-setup.md` (after Step 5, "Configure your local working tree as a round-tripper") or as a new tutorial `docs/tutorials/round-tripper.md` cross-linked from Pattern C (closes F4, F11, F16).
- Helper validator either skips non-frontmatter blobs or treats `.github/workflows/*.yml` + `README.md` + `.reposix/*` as mirror-passthrough (closes F1 in the implementation rather than the docs); alternatively, the docs explicitly forbid committing those files to the bus-push-managed branches (closes F1 in the docs and constrains the topology).
