# RUNBOOK ch.03 — the road to v1.0: portions, pre-framed decisions, end state

## §A — The five portions (L0's map)

Per index Amendment 1, L0 scopes the drive into large portions such that the
ENTIRE drive reaches its end state by ~10% of L0's own context, total — not 10%
per portion. Each portion is owned by one L1 portion coordinator, whose own
portion in turn reaches end state by ~10% of THAT L1's own context (the rule
recurses per tier, index Amendment 1):

| Portion | Contents | Milestone artifact |
|---|---|---|
| **1** | v0.13.0 close-out: P92 → P97 + tag | v0.13.0 tag pushed, milestone verdict GREEN incl. 9th probe |
| **2** | Launch-readiness milestone (OD-4 EXECUTIVE RESEQUENCE: runs BEFORE v0.13.2) | hero demo + CI-verified headline numbers + install excellence + Show-HN kit; formalized via `/gsd-new-milestone` |
| **3** | v0.13.2 cross-link (the deferred P98–P107 scope) | v0.13.2 tag |
| **4** | v0.14.x: L2/L3 cache-desync hardening residue + whatever ADRs/intakes routed there | v0.14.x tag(s), formalized via `/gsd-new-milestone` |
| **5** | v1.0.0: stabilization, end-state checklist (§H), ADR-009 semver activation at the tag | v1.0.0 tag + launch kit |

Portion order is owner-ratified (OD-3 serial rule + OD-4 resequence). Reordering
BETWEEN portions is valve E3 (owner); resequencing WITHIN a portion is DP-4.

## §B — Portion 1: P92–P97 phase frames

Canonical scope: `.planning/milestones/v0.13.0-phases/ROADMAP.md` (top-level
ROADMAP.md is index-only). Digest per phase — the L1 charter cites the ROADMAP
section; these frames add only the pre-made decisions:

- **P92 — push-flow correctness (Clusters B+C, RBF-B-01..07).** Rebase-ancestry
  preservation across post-push refetches (no fresh root commits per helper
  fetch); OP-3 audit silence: `helper_push_*` rows on every push;
  `.with_audit()` chained in all three `instantiate_{confluence,github,jira}`;
  `bus_write_audit_completeness.rs` asserts BOTH audit tables; real-push smoke
  vs TokenWorld; behavioral no-retry verifier replaces source-grep. Litmus
  T1+T4 (sim + TokenWorld), REOPEN ≥1 HIGH. **Pre-authorized:** split
  P92a/P92b if recon sizes RBF-B-01 >16h. Security waivers ×2 flip WAIVED→PASS
  after the first real CI run.
- **P93 — L2/L3 ADR + refresh fixes (RBF-LR-01..05).** ADR decision rubric in §C
  below. Code: `refresh_for_mirror_head` stops no-op'ing post-write;
  `SotPartialFail` recovery test (next push reads new SoT via PRECHECK B);
  `partial_failure_recovery_real_confluence` `#[ignore]` smoke. ADR is authored
  top-level (L2 coordinator fan-out), code via `gsd-execute-phase`. Litmus T1+T4.
  **Pre-authorized:** split P93a (ADR) / P93b (code) if the ADR concludes bigger.
- **P94 — export validator + mirror write path (Cluster D, RBF-C-01..07).**
  Export validator vs mirror-tree files (skip-list vs allowlist vs path-prefix →
  small ADR); workflow-YAML self-clobber fix; bus push green vs
  `reubenjohn/reposix-tokenworld-mirror`; T3 8/8; document the 3-step prereq
  chain + bus URL form; unencoded-`?` handling. Litmus T3.
- **P95 — UX/docs/row-migration + root-CLAUDE slim (RBF-D-01..15).** Scope
  frame in §D. **Pre-decided:** the P95a/P95b split is chosen BEFORE plan
  authoring; D-06/D-14 (RAISE-LIST drain) run top-level, the rest
  `gsd-execute-phase`.
- **P96 — surprises absorption (top-level, RBF-S-01..05).** F-K5 honesty
  spot-check by a FRESH subagent (author ≠ orchestrator, content-hash binding);
  drain SURPRISES-INTAKE (each entry → RESOLVED|DEFERRED|WONTFIX with SHA or
  rationale); amend RETROSPECTIVE; `EXTENDED-PENDING-P89-P97` overlay banners on
  the 12 old GREEN verdicts. Empty intake + skipped findings in verdicts = RED.
- **P97 — good-to-haves + milestone close + tag (top-level, RBF-G-01..05).**
  Frame in §E.

## §C — P93 L2/L3 ADR: the decision rubric

**Question:** how far does v0.13.x go on L2/L3 cache-desync (cache wrong about
SoT state beyond what PRECHECK B catches), vs deferral to v0.14.0?

- **Evidence to gather (L4 lanes, before the ADR draft):** (1) enumerate the
  desync classes from the REMEDIATION-PLAN and P91/P92 litmus transcripts —
  which are reachable TODAY on real backends; (2) what `reposix sync
  --reconcile` (the L1 escape hatch) already covers, with a real transcript;
  (3) cost sketch per option.
- **Options:** (a) full fix in v0.13.x; (b) narrow fix + ADR documenting the
  bounded exposure + teaching-string pointing at `reposix sync --reconcile`;
  (c) defer everything to v0.14.0.
- **Default = per REMEDIATION-PLAN Decision-2 (verbatim):** *"Don't defer fixes
  that v0.13.1's vision needs… Yes, pull them in or block the tag on them. The
  synthesis itself flags both as repeats of the 'deferred-but-the-row-still-
  says-PASS' anti-pattern."* So: option (b) minimum; anything the vision-litmus
  path can hit goes in NOW.
- **Hard rule:** NO re-deferral to v0.14.0 without owner sign-off — option (c)
  is valve E3, full stop. Rows describing deferred behavior must say
  NOT-VERIFIED/WAIVED-with-date, never PASS.
- **Escalation:** if evidence shows option (a) is architecturally forced (the
  narrow fix can't be bounded honestly) → valve E2 fable consult with the desync
  enumeration as the digest.

## §D — P95 scope frame

Three buckets, pre-scoped; resist additions (DP-5 applies):

1. **Row migration:** ~92 legacy P78–P88 catalog rows → F-K2/F-K4/F-K8 shapes
   (`coverage_kind`, `pre-release-real-backend` cadence, WAIVED+`until_date`).
   Mechanical, sonnet lanes, verify with the runner after each batch.
2. **Root-CLAUDE slim (load-bearing):** root `CLAUDE.md` is at 39,910/40,000
   bytes. Slim target ≤36,000 via pointerization (ORCHESTRATION.md pattern:
   ~6-line summary + pointer). Every byte cut must land in a pointed-to home,
   not vanish. Until this lands, ANY CLAUDE.md addition pairs with an equal
   removal.
3. **Discoverability + docs:** P91 T2-run-2 MED/LOW findings; README/first-run
   refresh + cold-reader pass (`/doc-clarity-review`); Pattern-C tutorial
   `docs/tutorials/round-tripper.md`; init UX nits; "pure git" claim qualifier;
   drain docs-repro ×5 + subjective ×3 waivers (due 2026-09-15).

## §E — P97 close ritual + tag (the exact sequence)

1. GOOD-TO-HAVES drain: XS always closes; M default-defers (OP-8 slot 2).
2. `/reposix-quality-review --all-stale` (subjective rubrics fresh).
3. OP-9 RETROSPECTIVE section (5 subheadings) BEFORE archiving; raw intakes
   travel with the archive.
4. Nine-probe milestone verdict OVERWRITES
   `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. The **9th probe** is
   non-skippable: `python3 quality/runners/run.py --cadence
   pre-release-real-backend` exit 0 — vision litmus vs TokenWorld; requires
   RBF-A-05/B-06/LR-04/C-03/D-15 all green. **OD-2 verbatim:** *"If
   pre-release-real-backend cannot EXECUTE against the sanctioned target at
   milestone-close, the verdict is RED… No owner-waiver. No until_date. No
   PASS-with-comment. No skip-counts-as-pass."*
5. Only after milestone verdict GREEN: re-enable `tag-v0.13.0.sh` (remove
   `.disabled`), CHANGELOG covering P78–P97, tag + push. **Tag authority:** the
   ROADMAP's older "owner pushes the tag" is superseded by OD-3, which delegates
   tag pushes contingent on GREEN (SESSION-HANDOVER records the narrowing) —
   push it yourself, record `[SELF]` in the ledger citing OD-3.
6. Release pipeline verification (post-tag): assets present, installer URLs
   resolve, `releases/latest` correct, binstall works (this unblocks
   `scripts/webhook-latency-measure.sh`, parked in SURPRISES-INTAKE).
7. PR #61 disposition (its hold expires at P97), milestone archive per
   `/gsd-complete-milestone` conventions, STATE.md cursor → Portion 2.

## §F — Portion 2: launch-readiness scoping rubric

North star (OD-3): *"highly polished tool for thousands of devs to adopt like
wildfire."* Litmus sentence for EVERY candidate item: **"a skeptical dev installs
in 5 minutes, sees agents filing tickets via pure git, checks the numbers, finds
them CI-verified."** If an item doesn't move that sentence, it goes to v0.13.2 or
backlog — no exceptions without the owner.

Must-haves (pre-ratified by OD-4):
- **CI-verified headline numbers** — mechanize `benchmark-claim/8ms-cached-read`
  and `89.1-percent-token-reduction` (GOOD-TO-HAVES-04). This RETIRES the
  chronic-yellow weekly rows (D-CONV-2's "yellow by design" ends here). Numbers
  regenerate in CI, README badges/claims bind to the generating job.
- **Asciinema hero demo** — the pure-git agent workflow, recorded against the
  simulator, embedded in README + docs landing. Cold-reader pass after.
- **Install-path excellence** — fresh-container verification of every install
  path (binstall, brew, curl), timed ≤5 min; error messages teach recovery.
- **Show-HN kit** — draft post, benchmark table with links to CI runs, FAQ.

Formalize via `/gsd-new-milestone` (requirements → roadmap → phases), then the
A-L1 loop resumes. Owner reviews milestone scope before phase 1 dispatch (E3 —
milestone boundary).

## §G — Portions 3–5: v0.13.2, v0.14.x, v1.0

- **v0.13.2 (Portion 3):** the deferred cross-link scope (old P98–P107). Rescope
  via `/gsd-new-milestone` against what launch-readiness changed; expect the 10x
  rule to shrink the phase count rather than grow it (some items will be dead).
- **v0.14.x (Portion 4):** L2/L3 residue per the P93 ADR's deferral list; any
  intake entries routed `v0.14`; file-size waiver reservation (post-v0.13.2
  candidate, per raise-list §5). Formalize via `/gsd-new-milestone`.
- **v1.0.0 (Portion 5):** no new features — stabilization + §H checklist +
  ADR-009 activation. **ADR-009** (`docs/decisions/009-stability-commitment.md`,
  Accepted) defines what the semver commitment covers; the v1.0.0 tag ACTIVATES
  it. Release notes state the commitment explicitly. Version bump, CHANGELOG,
  tag, release verification, launch kit ships.

## §H — The v1.0 end-state checklist (gradeable)

The final session dispatches a FRESH verifier (HCI — not itself) to grade every
line PASS/FAIL from artifacts. Any FAIL → v1.0.0 does not tag.

**Gates + honesty**
- [ ] All milestone verdicts GREEN, incl. every 9th probe (pre-release-real-backend
      exit 0 vs sanctioned targets) — no waiver blocks on P0 rows, per OD-2.
- [ ] Zero undrained BLOCKER ledger rows; zero expired waivers; intake files
      empty or dispositioned; RETROSPECTIVE current per OP-9.
- [ ] Weekly verdict fully green — chronic-yellow rows retired by CI-verified
      numbers (§F), not by deletion.
- [ ] Dark-factory arms (sim + dvcs-third-arm) green in CI; litmus T1/T3/T4
      transcripts committed and fresh.

**Adoption surfaces**
- [ ] Install verified on fresh container ≤5 min for every documented path;
      installer URLs + badges resolve; `releases/latest` points at the
      multi-platform release.
- [ ] Hero demo (asciinema) live in README + docs; cold-reader + subjective
      rubrics fresh (≤30-day TTL); headline numbers link to the CI runs that
      produced them.
- [ ] mkdocs strict + mermaid + link gates green; doc-alignment ratios above
      floors with zero STALE rows; troubleshooting covers the top 5 real-backend
      failure shapes.

**Security + hygiene**
- [ ] Allowlist enforcement + audit-immutability gates green; dual audit tables
      asserted end-to-end; cred-hygiene green; `JIRA_TEST_PROJECT` gap resolved
      or documented as owner-accepted.
- [ ] PR queue empty/dispositioned; stale branches cleaned (owner-named); no
      orphan processes; root CLAUDE.md ≤40k with slim margin; ORCHESTRATION.md +
      this runbook updated with any doctrine learned en route (fix-it-twice).

**Release**
- [ ] Tags v0.13.0 → v1.0.0 all pushed with assets; crates.io versions current;
      ADR-009 activation stated in v1.0.0 release notes; CHANGELOG complete.
- [ ] Show-HN kit committed; `.planning/CONSULT-DECISIONS.md` tells the judgment
      story end-to-end (non-empty, every valve crossing recorded).
