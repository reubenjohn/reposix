## v0.13.0 DVCS over REST (PLANNING)

> **Status:** scoped 2026-04-30; extended 2026-05-08 (Path A / Option B ratified — hold the v0.13.0 tag, extend with corrective phases P89–P97). Phases 78–88 derive from `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Phase decomposition" + 15 ratified open-question decisions in `decisions.md` + 4 carry-forward items in `CARRY-FORWARD.md`. Phases 89–97 derive from `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "3 — Proposed v0.13.0 extension phase shape" (the 16 HIGH dark-factory frictions on real Confluence + ~51 HIGH audit findings across P78–P88; framework fixes land first so subsequent code/doc fixes ship into a trustworthy framework). Handover bundle: `.planning/research/v0.13.0-dvcs/{vision-and-mental-model,architecture-sketch,kickoff-recommendations,decisions}.md` + `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` + `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/{REMEDIATION-PLAN,COMPLETENESS-CHECK,PATTERNS,STRATEGIC-REFRAME}.md` + `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`. v0.13.2 cross-link-fidelity (P98–P107) runs as workstream B in parallel — see `.planning/milestones/v0.13.2-phases/ROADMAP.md`.

**Thesis + mental model.** See `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` for the full statement. Two-sentence summary: shift from "VCS over REST" to "DVCS over REST" — confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface; devs `git clone` vanilla, install reposix only to write back, and `git push` via a bus remote that fans out SoT-first then mirror-best-effort. Three roles: SoT-holder (reposix-equipped, writes via bus); mirror-only consumer (vanilla git, read-only); round-tripper (reposix-equipped after `attach`, writes via bus). Mirror-lag observability via plain-git refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`) — vanilla `git fetch` brings them along.

**Recurring success criteria for EVERY phase (P78–P88)** — non-negotiable per CLAUDE.md Operating Principles + the v0.12.0 autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — phase's FIRST commit writes catalog rows under `quality/catalogs/<file>.json` BEFORE any implementation commit.
2. **CLAUDE.md updated in same PR** (QG-07) — every phase that introduces a new file/convention/gate revises the relevant CLAUDE.md section in the same PR.
3. **Per-phase push** (codified 2026-04-30, closes backlog 999.4) — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of phase-close criterion.
4. **Phase close = unbiased verifier subagent dispatch (OP-7)** — isolated subagent grades all catalog rows for the phase against artifacts under `quality/reports/verifications/`; verdict at `quality/reports/verdicts/p<N>/VERDICT.md`; phase does not close on RED.
5. **Eager-resolution preference (OP-8)** — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`. The +2 reservation (P87 + P88) drains them.
6. **Simulator-first (OP-1)** — all phases run end-to-end against the simulator. Two simulator instances serve as "confluence-shaped SoT" + "GitHub-shaped mirror." Real-backend tests gate milestone close, not individual phase closes.
7. **Tainted-by-default (OP-2)** — mirror writes carry tainted bytes from the SoT; the `attach` cache marks all materialized blobs as tainted.
8. **Audit log non-optional (OP-3)** — every bus-remote push writes audit rows to BOTH tables (cache audit + backend audit); mirror push writes a cache-audit row noting mirror-lag delta; webhook-driven syncs write cache-audit rows too.

### Phase 78: Pre-DVCS hygiene — gix bump, WAIVED-row verifiers, multi-source walker

**Goal:** Land three mutually-parallelizable hygiene items as a single light kickoff phase BEFORE the POC + attach work begins — (a) bump `gix` off the yanked `=0.82.0` baseline (closes #29 + #30); (b) land verifier scripts for the 3 currently-WAIVED structure rows in `quality/catalogs/freshness-invariants.json` BEFORE waivers expire 2026-05-15; (c) schema-migrate the docs-alignment walker to watch every source on `Source::Multi` rows (carry-forward `MULTI-SOURCE-WATCH-01` from v0.12.1 P75 via path-(b) `source_hashes: Vec<String>`).

**Requirements:** HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01 · **Depends on:** — (entry-point phase; v0.12.1 SHIPPED 2026-04-30 is the precondition) · **Plan:** [78-PLAN-OVERVIEW](78-pre-dvcs-hygiene/78-PLAN-OVERVIEW.md)

### Phase 79: POC + `reposix attach` core

**Goal:** Ship the POC first (`research/v0.13.0-dvcs/poc/`) to surface design surprises, then implement `reposix attach <backend>::<project>` — builds a fresh cache from REST against an existing checkout, reconciles by `id` in frontmatter, populates the cache reconciliation table, and adds remote `reposix::<sot>?mirror=<existing-origin-url>`. Reconciliation cases (match / backend-deleted via `--orphan-policy` / no-id / duplicate-id / mirror-lag) per `architecture-sketch.md`. Re-attach to different SoT REJECTED (Q1.2); same-SoT IDEMPOTENT (Q1.3). Blobs `Tainted<Vec<u8>>` (OP-2).

**Requirements:** POC-01, DVCS-ATTACH-01..04 · **Depends on:** P78 GREEN · **Plan:** [79-PLAN-OVERVIEW](79-poc-reposix-attach-core/79-PLAN-OVERVIEW/index.md)

**Plans status:**
- **79-01** — POC-01 (throwaway POC at `research/v0.13.0-dvcs/poc/`). **SHIPPED 2026-05-01** (5 commits 660bae0..4e6de2b; SUMMARY at `.planning/phases/79-poc-reposix-attach-core/79-01-SUMMARY.md`; FINDINGS at `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` — 5 INFO + 2 REVISE + 0 SPLIT routing tags).
- **79-02** — DVCS-ATTACH-01..02 (scaffold + cache reconciliation module). PENDING orchestrator's POC-FINDINGS re-engagement decision.
- **79-03** — DVCS-ATTACH-02..04 (tests + idempotency + close). PENDING 79-02.

### Phase 80: Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`)

**Goal:** Wire mirror-lag observability into plain-git refs that vanilla `git fetch` brings along, so `git log refs/mirrors/<sot>-synced-at` reveals staleness to readers who never installed reposix. Two refs in the `refs/mirrors/...` namespace (Q2.1): `<sot>-head` records the SHA of the SoT's `main` at last sync; `<sot>-synced-at` is an annotated tag with a timestamp message. Helper APIs land in `crates/reposix-cache/`. The existing single-backend push (today's `handle_export`) is wired to update both refs on success — pre-bus integration point so observability is in place BEFORE bus phases. Webhook sync also writes both refs (P84). Bus push will update both refs (P83). Q2.3: bus updates both refs (consistency over optimization); webhook becomes a no-op refresh when bus already touched them.

**Requirements:** DVCS-MIRROR-REFS-01..03 · **Depends on:** P79 GREEN · **Plan:** [80-PLAN-OVERVIEW](80-mirror-lag-refs/80-PLAN-OVERVIEW/index.md)

### Phase 81: L1 perf migration — `list_changed_since`-based conflict detection

**Goal:** Replace `handle_export`'s unconditional `list_records` walk with `list_changed_since`-based conflict detection so the bus remote (P82–P83) inherits the cheap path (Q3.1). Net REST cost on success path: one call (`list_changed_since`) + actual REST writes. L1 trusts cache as the prior; `reposix sync --reconcile` is the on-demand escape hatch. L2/L3 hardening defers to v0.14.0.

**Requirements:** DVCS-PERF-L1-01..03 · **Depends on:** P80 GREEN · **Plan:** [81-PLAN-OVERVIEW](81-l1-perf-migration/81-PLAN-OVERVIEW/index.md)

### Phase 82: Bus remote — URL parser, prechecks, fetch dispatch

**Goal:** Stand up the bus remote's read/dispatch surface. URL parser recognizes `reposix::<sot>?mirror=<url>` (Q3.3); `+`-delimited form rejected. CHEAP PRECHECK A (mirror drift via `git ls-remote`) + CHEAP PRECHECK B (SoT drift via `list_changed_since`) run BEFORE stdin read. Bus does NOT advertise `stateless-connect` for fetch (Q3.4). Dispatch-only; WRITE fan-out (steps 4–9) lands in P83.

**Requirements:** DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01..02, DVCS-BUS-FETCH-01 · **Depends on:** P81 GREEN · **Plan:** [82-PLAN-OVERVIEW](82-bus-remote-url-parser/82-PLAN-OVERVIEW/index.md)

### Phase 83: Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection)

**Goal:** Implement the riskiest part of the bus remote — the SoT-first-write algorithm with mirror-best-effort fallback and full fault-injection coverage. Algorithm: read fast-import stream from stdin and buffer; apply REST writes to SoT (confluence); on success, write audit rows to BOTH tables (cache + backend) and update `last_fetched_at`; then `git push` to GH mirror; on mirror failure, write mirror-lag audit row, update `refs/mirrors/<sot>-head` to new SoT SHA but NOT `<sot>-synced-at`, return ok to git (SoT contract satisfied — recoverable on next push). On mirror success, update `<sot>-synced-at` to now and ack. NO helper-side retry on transient mirror-write failures (Q3.6) — surface, audit, let user retry. Fault-injection tests cover kill-GH-push-between-confluence-write-and-ack / kill-confluence-write-mid-stream / simulate-confluence-409-after-precheck.

**Requirements:** DVCS-BUS-WRITE-01..06 · **Depends on:** P82 GREEN · **Plan:** [83-PLAN-OVERVIEW](83-bus-write-fan-out/83-PLAN-OVERVIEW/index.md) (P83-01 + P83-02 SHIPPED)

### Phase 84: Webhook-driven mirror sync — GH Action workflow + setup guide

**Goal:** Ship the reference GitHub Action workflow that keeps the GH mirror current with confluence-side edits — the pull side of the DVCS topology. Workflow at `.github/workflows/reposix-mirror-sync.yml` triggers on `repository_dispatch` (event type `reposix-mirror-sync`) plus a cron safety net (default `*/30`, configurable via workflow `vars` per Q4.1). Workflow runs `reposix init confluence + git push <mirror>` and updates `refs/mirrors/...` refs. Uses `--force-with-lease` against last known mirror ref so a concurrent bus-push's race doesn't corrupt mirror state. First-run handling (no existing mirror refs, empty mirror) is graceful per Q4.3. Latency target: < 60s p95 from confluence edit to GH ref update. Backends without webhooks (Q4.2): cron path is the only sync mechanism.

**Requirements:** DVCS-WEBHOOK-01..04 · **Depends on:** P80 + P83 GREEN · **Plan:** [84-PLAN-OVERVIEW](84-webhook-mirror-sync/84-PLAN-OVERVIEW/index.md)

### Phase 85: DVCS docs — topology, mirror setup, troubleshooting, cold-reader pass

**Goal:** Make v0.13.0 legible to a cold reader. Three new doc surfaces ship: `docs/concepts/dvcs-topology.md` (three roles + diagram from `vision-and-mental-model.md` + when-to-choose-which-pattern guidance + the verbatim Q2.2 clarification *"`refs/mirrors/<sot>-synced-at` is the timestamp the mirror last caught up to <sot> — it is NOT a 'current SoT state' marker"*); `docs/guides/dvcs-mirror-setup.md` (walk-through of webhook + Action setup; backends-without-webhooks fallback per Q4.2; cleanup procedure); troubleshooting matrix entries in `docs/guides/troubleshooting.md` covering bus-remote `fetch first` rejection messages, attach reconciliation warnings, webhook race conditions, cache-desync recovery via `reposix sync --reconcile`. Cold-reader pass via `doc-clarity-review`. Banned-words enforced (no FUSE residue; no jargon-above-Layer-3 leaks).

**Requirements:** DVCS-DOCS-01..04 · **Depends on:** P79 + P80 + P81 + P82 + P83 + P84 ALL GREEN · **Plan:** TBD (P85 plan-overview not yet authored)

**Success criteria:**
1. `docs/concepts/dvcs-topology.md` ships with three roles + diagram + Q2.2 clarification + when-to-choose guidance; banned-words clean.
2. `docs/guides/dvcs-mirror-setup.md` ships with end-to-end webhook + Action walk-through, cron-only fallback (Q4.2), cleanup procedure.
3. Troubleshooting matrix entries land for bus-remote rejection / attach reconciliation / webhook race / cache-desync recovery.
4. `doc-clarity-review` cold-reader pass against a reader who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`; zero critical-friction findings.
5. mkdocs nav + banned-words clean (`bash scripts/check-docs-site.sh` + `scripts/banned-words-lint.sh` GREEN; mermaid diagrams render).
6. Catalog rows land first (docs-alignment binding the three new docs to verifier tests; subjective-rubric for cold-reader pass with `freshness_ttl: 30d`); CLAUDE.md updated.
7. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p85/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Mental model"; `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-driven mirror sync"; `.planning/research/v0.13.0-dvcs/decisions.md` § Q2.2 + § Q4.2 + § "POC scope"; existing `docs/guides/troubleshooting.md`; `.claude/skills/doc-clarity-review/SKILL.md`.

### Phase 86: Dark-factory regression — third arm (vanilla-clone + attach + bus-push)

**Goal:** Extend `quality/gates/agent-ux/dark-factory.sh` (formerly `scripts/dark-factory-test.sh`, migrated in v0.12.0 P59 SIMPLIFY-07) to add a third subprocess-agent transcript: a fresh agent given only the GH mirror URL + a goal completes vanilla-clone + `reposix attach` + edit + bus-push end-to-end with zero in-context learning beyond what the helper's stderr teaches. Reuses the existing dark-factory test harness; no in-prompt instruction beyond the goal statement. The transcript proves the DVCS thesis: a curious developer who has never read a reposix doc can still complete the round-trip because the helper teaches itself via stderr (blob-limit error names `git sparse-checkout`; bus reject names mirror-lag refs; attach errors name `--orphan-policy`).

**Requirements:** DVCS-DARKFACTORY-01..02 · **Depends on:** P79 + P80 + P81 + P82 + P83 + P84 + P85 ALL GREEN · **Plan:** TBD (P86 plan-overview not yet authored)

**Success criteria:**
1. Third arm in dark-factory harness (`dvcs-third-arm` scenario; existing two arms unchanged); subprocess agent prompt: *"The repo at <GH-mirror-url> mirrors a confluence backend. Install reposix, attach, fix the bug in `issues/0001.md` (typo on line 3), push your fix back. You have 10 minutes."*
2. Zero in-context learning required: agent did NOT receive the bus URL syntax / `attach` spelling / `--orphan-policy` / mirror-lag ref namespace; all four are recovered from helper stderr or `--help`.
3. End-to-end success: typo fix lands in confluence (REST GET) AND GH mirror (`git fetch && git log`) AND `refs/mirrors/<sot>-synced-at` advanced; audit rows present in both tables.
4. Catalog row in `agent-ux` dimension (`dvcs-third-arm`, `kind: subagent-graded`, `cadence: pre-pr`, `freshness_ttl: 30d`).
5. Sim AND TokenWorld coverage (CI default + secrets-gated real-backend; milestone-close gate per OP-1).
6. Catalog rows + helper-stderr docs-alignment rows land first; CLAUDE.md updated.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p86/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Success gates" #6; `quality/gates/agent-ux/dark-factory.sh`; `.claude/skills/reposix-agent-flow/SKILL.md`; `docs/reference/testing-targets.md`; `quality/PROTOCOL.md` § "Verifier subagent prompt template".

### Phase 87: Surprises absorption (+2 reservation slot 1)

**Goal:** Drain `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` per OP-8. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases (P78–P86) plans + verdicts asks: *"did this phase honestly look for out-of-scope items?"* Empty intake is acceptable IF phases produced explicit `Eager-resolution` decisions in their plans; empty intake when verdicts show skipped findings is RED. The +2 reservation is in addition to P78–P86 planned phases per CLAUDE.md OP-8 — eager-resolution preference is the default; surprises absorption picks up only what eager-resolution couldn't.

**Requirements:** DVCS-SURPRISES-01 · **Depends on:** P78 + P79 + P80 + P81 + P82 + P83 + P84 + P85 + P86 ALL GREEN · **Plan:** TBD (P87 plan-overview not yet authored)

**Success criteria:**
1. Every entry in `SURPRISES-INTAKE.md` has terminal STATUS (RESOLVED + commit SHA / DEFERRED + target milestone + rationale / WONTFIX + rationale). No `STATUS: TBD` at phase close.
2. Verifier honesty spot-check samples ≥3 P78–P86 plan/verdict pairs; spot-check report at `quality/reports/verdicts/p87/honesty-spot-check.md`. Empty intake acceptable IF phases produced explicit `Eager-resolution` decisions; empty intake when verdicts show skipped findings → RED.
3. Catalog deltas computed: any SURPRISES entries that flip catalog rows update cleanly; alignment_ratio + coverage_ratio deltas reported in the phase verdict.
4. No silent scope creep: P78–P86 verdicts must have flagged out-of-scope findings via Eager-resolution OR SURPRISES-INTAKE.
5. Catalog rows for v0.14.0 carry-forwards land before TBD → DEFERRED flips; CLAUDE.md updated if any v0.14.0 carry-forward sentences land.
6. Phase close: `git push origin main`; verifier GREEN AND signs honesty spot-check; verdict at `quality/reports/verdicts/p87/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8; `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`; `.planning/milestones/v0.12.1-phases/` P76 verdict (precedent for honesty-check execution).

### Phase 88: Good-to-haves polish (+2 reservation slot 2) — milestone close

**Goal:** Drain `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` per OP-8 sizing rules — XS items always close in-phase; S items close if budget; M items default-defer to v0.14.0. After polish, finalize milestone-close artifacts: CHANGELOG `[v0.13.0]` entry; tag-script at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (≥6 safety guards mirroring v0.12.0 precedent); RETROSPECTIVE.md v0.13.0 section distilled per OP-9 ritual; milestone-close verifier subagent dispatched and GREEN. Owner runs `tag-v0.13.0.sh` and pushes the tag — orchestrator does NOT push the tag.

**Requirements:** DVCS-GOOD-TO-HAVES-01 · **Depends on:** P87 GREEN · **Plan:** TBD (P88 plan-overview not yet authored)

**Success criteria:**
1. GOOD-TO-HAVES.md drained: every entry terminal STATUS — XS closed (commit SHA), S closed-or-deferred (rationale), M default-deferred to v0.14.0 (carry-forward target named).
2. CHANGELOG `[v0.13.0]` finalized: summarizes P78–P88 + lists every shipped REQ-ID by category + names v0.14.0 carry-forward.
3. Tag-script authored at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` with ≥6 safety guards (clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag); tag-gate guards re-run cleanly post-P88.
4. RETROSPECTIVE.md v0.13.0 section distilled (OP-9) BEFORE archive: What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts + autonomous-run findings.
5. Milestone-close verifier dispatched and GREEN at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`: P78–P88 catalog rows all GREEN-or-WAIVED; dark-factory three-arm transcript GREEN against sim AND TokenWorld; no expired waivers without follow-up; RETROSPECTIVE v0.13.0 section exists; +2 reservation operational.
6. STOP at tag boundary: orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.0 ready-to-tag; owner pushes tag."
7. Catalog rows + CLAUDE.md v0.13.0-shipped historical-milestone subsection land first.
8. Phase close: `git push origin main`; milestone-close verifier GREEN; verdict at `quality/reports/verdicts/p88/VERDICT.md` + `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8 + OP-9; `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md`; `.planning/RETROSPECTIVE.md`; `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` + `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (tag-script precedents); `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (carry-forward target).

---

> **v0.13.0 extension (P89–P97) — added 2026-05-08.** The original P78–P88 milestone shipped GREEN against the simulator but the post-tag dark-factory exercise on real Confluence (TokenWorld) + GitHub mirror surfaced 16 HIGH frictions, and a 12-subagent codebase audit surfaced ~51 additional HIGH findings concentrated in P79 / P80 / P83 / P86 / P87 / P88. Owner ratified Path A / Option B: hold the v0.13.0 tag, extend with P89–P97 corrective phases. Framework fixes (P89 + P90) land FIRST so subsequent code/doc fixes (P91–P95) ship into a trustworthy framework; P96 + P97 become the active +2 reservation slots for the EXTENDED v0.13.0 (superseding the original P87 + P88 reservation — P87's intake content folds forward via RBF-S-01). Source of truth for P89–P97: `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md`.

### Phase 89: Framework fixes — real-backend cadence, shell-subprocess kind, milestone-close litmus probe

**Goal:** Build the framework infrastructure that makes every subsequent v0.13.0 extension phase's catalog rows trustworthy. Land the new `cadence: pre-release-real-backend` (gates on `REPOSIX_ALLOWED_ORIGINS` + backend creds; default-skipped in CI; required at milestone-close per F-K1); the new `kind: shell-subprocess` verifier that drives `reposix init/attach/sync/push` as actual subprocesses against a real backend with explicit env-control assertions and produces a transcript artifact (per F-K2); a milestone-close 9th probe that runs the vision document's litmus test verbatim against a real backend before the tag (per F-K3); the `P\d+-\d+` banned-production-error-tokens regex (per F-K7); the deferral-pointer linter (per F-K6); and the structural `claim_vs_assertion_audit` field on every new catalog row with runner cross-check (per Decision 3 — F-K1..K8 patches are necessary but not sufficient).

**Requirements:** RBF-FW-01, RBF-FW-02, RBF-FW-03, RBF-FW-04, RBF-FW-05, RBF-FW-11 · **Depends on:** — (entry-point of the v0.13.0 extension series; v0.13.0 tag held per Path A/Option B) · **Execution mode:** top-level · **Plan:** TBD (P89 plan-overview not yet authored)

**Success criteria:**
1. `quality/PROTOCOL.md` documents `cadence: pre-release-real-backend` + `kind: shell-subprocess` with worked example; `quality/runners/run.py` recognizes the new cadence (default-skips when env not set; requires explicit env to run).
2. Milestone-close verdict template carries a 9th probe entry that runs the vision litmus test against a real backend; absent ⇒ verdict graded RED.
3. Pre-push gate runs the deferral-pointer linter (`grep -rn 'not yet wired in P\d+\|land(s\|ing) (alongside\|in) P\d+\|substrate-gap-deferred' crates/` cross-referenced against named downstream phase's PLAN files; named phase missing the delivery ⇒ BLOCK); banned-production-error-tokens regex `P\d+-\d+` extended in `quality/gates/structure/banned-words.sh` (production strings only).
4. `claim_vs_assertion_audit` field present on every new catalog row P89 / P90 mints; runner cross-check passes (Decision 3).
5. Catalog-first commit mints the 5+ rows in `quality/catalogs/{agent-ux,framework}.json` with `status: NOT-VERIFIED` BEFORE implementation commits land; CLAUDE.md updated in same PR.
6. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p89/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P89" + § "2 — Cross-cutting framework fixes" F-K1 / F-K2 / F-K3 / F-K6 / F-K7; `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § Decision 3; `quality/PROTOCOL.md`; CLAUDE.md § "Quality Gates" taxonomy.

### Phase 90: Framework fixes — catalog-row honesty rules, dishonest-test triage, honesty-check meta-rule

**Goal:** Close the verifier-shape exemptions that let P78–P88 grade GREEN with sim-only coverage. Land catalog-row honesty rules (transport/perf rows MUST carry `coverage_kind: real-backend` or explicit `WAIVED + until_date`; no PASS-with-comment per F-K4a); runner cross-check that `expected.asserts` text aligns with `asserts_passed` artifact strings at grade time (F-K4b); a catalog migration that flips every `kind: subagent-graded` row lacking `dispatch.sh` wiring to `kind: mechanical` or wires `dispatch.sh` if intent was real grading (F-K4c); the `quality/gates/agent-ux/test-name-vs-asserts.sh` triage gate that RAISEs when test name implies real-backend / push / fetch / round-trip but body neither invokes `Command::new("git").arg("push|fetch")` nor speaks to a non-`127.0.0.1` host nor is `#[ignore]` real-backend-gated (F-K8); the absorption-phase honesty-check meta-rule (sample EVERY no-intake phase + spot-check author ≠ orchestrator + rubric "walk one critical example end-to-end mentally — does it work?" + content-hash binding per F-K5); and the milestone-close adversarial pass dispatch where a fresh subagent reads catalog row descriptions only and grades whether assertion would falsify description (Decision 3 / RBF-FW-12). Walking these new gates over the live catalog produces a RAISE LIST that seeds P92 / P94 / P95 work.

**Requirements:** RBF-FW-06, RBF-FW-07, RBF-FW-08, RBF-FW-09, RBF-FW-10, RBF-FW-12 · **Depends on:** P89 GREEN · **Execution mode:** top-level · **Plan:** TBD (P90 plan-overview not yet authored)

**Success criteria:**
1. Pre-push gate runs `quality/gates/agent-ux/test-name-vs-asserts.sh`; flagged rows produce a structured RAISE list at `quality/reports/raise-list-p90.md`.
2. Runner refuses to flip a row PASS if `expected.asserts` text does not align with `asserts_passed` strings (vocabulary mismatch ⇒ verifier graded RED, closes p86 F6).
3. Catalog migration script flips every `kind: subagent-graded` row that lacks `dispatch.sh` wiring to `kind: mechanical` (or wires `dispatch.sh` if intent was real grading); closes p86 F7.
4. Absorption-phase template (consumed by P96) carries the F-K5 meta-rule verbatim — sample includes EVERY no-intake phase, spot-check author ≠ orchestrator, rubric is "walk one critical example end-to-end mentally — does it work?", content-hash binding present.
5. Walking the new gates over the live catalog produces a RAISE LIST of every existing dishonest test/row; the list is committed to `quality/reports/raise-list-p90.md` and seeds P92 / P94 / P95 work.
6. Milestone-close adversarial pass dispatch documented in `quality/PROTOCOL.md`; rubric file at `quality/dispatch/milestone-adversarial.md`; runner blocks GREEN if ≥1 row's audit fails per Decision 3.
7. Catalog rows for RBF-FW-06..12 land first (NOT-VERIFIED) BEFORE implementation commits; CLAUDE.md updated in same PR.
8. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p90/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P90" + § "2 — Cross-cutting framework fixes" F-K4 / F-K5 / F-K8; `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § Decision 3 (milestone-close adversarial pass); CLAUDE.md § "Quality Gates" + § "Subagent delegation rules".

### Phase 91: `reposix attach` + `sync --reconcile` real-backend wiring (Cluster A)

**Goal:** Land confluence/github/jira backends for `attach` and `sync --reconcile` — the work P79-03 was supposed to ship and silently dropped. Wire `attach.rs:147-166` real-backend dispatch (mirroring `refresh.rs:174-240` / `backend_dispatch.rs:234-271` pattern) with credential plumbing threaded; wire `sync.rs:79-92` real-backend dispatch in the same shape; drop `P79-02 scaffold` / `P79-03` / `P82+` phase-ID tokens from production error strings (caught by F-K7 banned-tokens regex from P89); ship `agent_flow_real.rs` `attach_real_{confluence,github,jira}` and `sync_real_{confluence,github,jira}` `#[ignore]` smoke tests (vanilla `git init` + `reposix attach $BACKEND::$PROJECT` + assert post-conditions); make the `dark-factory.sh dvcs-third-arm` harness create a NON-EMPTY work tree + populated bare mirror so reconciliation walk exercises matched / no_id / backend_deleted / mirror_lag / orphan cases (closes p86 F13); flip REQUIREMENTS.md DVCS-ATTACH-01..04 status to reflect real-backend coverage; and replace the incoherent CLAUDE.md attach example (current example uses `git clone git@github.com:...` then `reposix attach sim::demo` per p79 F8).

**Requirements:** RBF-A-01, RBF-A-02, RBF-A-03, RBF-A-04, RBF-A-05, RBF-A-06, RBF-A-07, **QL-001** · **Depends on:** P89 GREEN, P90 GREEN · **Plan:** 6 plans (91-01..91-06; overview 91-OVERVIEW.md)
- [x] 91-01-PLAN.md — catalog-first mint (ql-001-canonical-path-shape, attach-sync-real-backend; NOT-VERIFIED) — SHIPPED 2026-07-04 (91-01-SUMMARY.md)
- [x] 91-02-PLAN.md — Lane 1 QL-001 canonical path-shape + stream-parser fix + fixture re-key + waiver retire — SHIPPED 2026-07-04 (91-02-SUMMARY.md)
- [x] 91-03-PLAN.md — Lane 2 attach/sync real-backend dispatch + reconciliation + smokes (D91-03/04/09) — SHIPPED 2026-07-04 (91-03-SUMMARY.md; 5 commits 1f7fff3..bd5e115; real Confluence attach+sync verified live; catalog row grading deferred to coordinator)
- [x] 91-04-PLAN.md — dvcs-third-arm populate (RBF-A-05) + Confluence hierarchy self-seed + testing-targets doc — SHIPPED 2026-07-04 (commits 1504425/587aee4/6ca3f6d/ac0fcdf)
- [x] 91-05-PLAN.md — litmus verifier rewrite (D91-06) + substrate prep; T2 REOPEN gate left for coordinator — SHIPPED 2026-07-04 (91-05-SUMMARY.md; T2 REOPEN gate closed per coordinator run 2, HIGH=0 MED=3 LOW=1)
- [x] 91-06-PLAN.md — docs (REQUIREMENTS/CLAUDE/comments-attachments) + phase close (push, CI watch, handoff) — Tasks 1-2 SHIPPED 2026-07-04 (91-06-SUMMARY.md; push + CI evidence deliberately deferred to the coordinator's close ritual per the plan's checkpoint step)

**QL-001 — push diff planner path-shape correctness (D90-01, ROUTED from P90 SURPRISES-INTAKE 2026-07-04 06:20, verified repro at 75db262).** The push diff planner cannot round-trip a genuinely git-produced push: a path-shape mismatch across `builder.rs`/`refresh.rs`/`fast_import.rs`/`diff.rs` (three different spellings of the same issue path), no `issues/*.md`-only filter (non-issue blobs like `.reposix/fetched_at.txt` reject the push), and a stream-parser bug that unconditionally swallows one line after the commit-message `data N` block — silently dropping the lowest-id issue as a spurious Delete on every push, including no-op pushes. This is a hard dependency of this phase's own mid-stream litmus gate (SC-6 below): the litmus cannot pass while the planner misclassifies every real-tree push. P90 is F-class (framework) and does not absorb this C-class cross-crate product fix (rationale: `90-DECISIONS.md` D90-01). Six sharpened acceptance criteria (verbatim from the intake entry's numbered list):
1. A real `git push` from a `reposix init`'d tree (or a tree in the canonical `issues/<id>.md` shape) round-trips exactly ONE edited record against the sim as a single PATCH, with zero manual fast-export and misclassification count == 0 (no Creates, no Deletes).
2. A no-op push (pull, no edits, push) produces zero backend writes — no create/update/delete, no record deleted, no version bump on any record.
3. A push of the full seeded tree against a matching sim produces zero Delete actions.
4. A tree containing `.reposix/` metadata (as `reposix refresh` writes) pushes without an `invalid-blob` rejection.
5. The regression runs a REAL `git push` end-to-end at the agent-ux gate; if the harness requires git ≥ 2.34, it asserts the version precondition and fails loud (not silently skips) on older git.
6. All four path-shape sites (`builder.rs`, `refresh.rs`, `fast_import.rs`, `diff.rs`) are grep-verifiable as producing the single canonical spelling for a given id (recommended: `issues/<id>.md` unpadded, the cache/stateless-connect production shape), via one shared `issue_id_from_path` (also consolidates the QL-157 duplicate at `diff.rs:74-77` / `main.rs:432-435`).

**Success criteria:**
1. `reposix attach confluence::REPOSIX --remote-name reposix` against a vanilla mirror clone configures git correctly + reconciles by frontmatter `id` (5 cases per architecture sketch: matched / no-id / backend-deleted / duplicate-id / mirror-lag).
2. `agent_flow_real.rs` `attach_real_{confluence,github,jira}` + `sync_real_{confluence,github,jira}` family GREEN with credentials set, default-skipped without (gated by `cadence: pre-release-real-backend` from P89).
3. `cargo run -p reposix-cli -- attach confluence::REPOSIX` no longer emits any `P\d+-\d+` token to stderr (banned-tokens regex enforced from P89-RBF-FW-04).
4. `dark-factory.sh dvcs-third-arm` populates work tree before attach; reconciliation report names non-zero counts in at least three reconciliation cases (closes p86 F13).
5. T2 in dark-factory exercise (`docs/research/dark-factory-may02/T2.md` round-trip on real Confluence) re-runs and passes 5/5 boxes.
6. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T2 against TokenWorld. If ≥1 HIGH-severity friction surfaces, this phase REOPENS — P92 MUST NOT start until T2 returns ≤0 HIGH frictions. The checkpoint is a phase gate, not a soft success criterion. **Sanctioned-target litmus-body criterion (D90-06):** the litmus verifier's body itself MUST assert the resolved real-backend target is one of the sanctioned three (`docs/reference/testing-targets.md`) and fail loud otherwise — this is the real proof obligation the `pre-release-real-backend` env-gate only heuristically approximates; do not add a second, weaker allowlist check in `_realbackend` instead.
7. Catalog rows + REQUIREMENTS.md DVCS-ATTACH-01..04 status flip + CLAUDE.md attach-example fix land first; CLAUDE.md updated in same PR.
8. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p91/VERDICT.md`.
9. **QL-001's six acceptance criteria above land as part of this phase's GREEN.** The `agent-ux/real-git-push-e2e` waiver (expires 2026-07-31) is retired when QL-001 lands, NOT renewed — its expiry is the intentional backstop if this phase slips past 2026-07-31 (D90-01; P90 confirmed this waiver disposition in 90-05's per-waiver drain, `quality/reports/raise-list-p90.md` § Waivers).

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P91" + § "Cluster A" finding inventory (H-A1..A7); `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T2.md`; `.planning/research/v0.13.0-real-backend-frictions/02-phase-audits-may08/{phase-audit-p79,vision-audit}.md`; `crates/reposix-cli/src/{attach,sync,refresh,backend_dispatch,diff}.rs`; `crates/reposix-cli/tests/agent_flow_real.rs`; `quality/gates/agent-ux/dark-factory.sh`; `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (QL-001 entry, 2026-07-04 06:20); `.planning/phases/90-framework-fixes-honesty-rules/90-DECISIONS.md` D90-01/D90-06.

### Phase 92: Push-flow correctness — rebase recovery + OP-3 audit log silence (Clusters B + C)

**Goal:** Fix the v0.9.0 architectural cornerstone (`git pull --rebase`) and the OP-3 audit-log silence — both broken on every push from a partial-clone working tree. Helper-side fetch must preserve ancestry across post-push refetches (no fresh root commits per fetch — closes dark-factory CLUSTER B / vision-audit F4 / T4 HIGH-1); helper must open `cache.db` with correct gitdir/cwd resolution and `helper_push_*` rows MUST land for OP-3 compliance on every push (closes dark-factory CLUSTER C / vision-audit F3); `instantiate_{confluence,github,jira}` MUST chain `.with_audit(audit_conn)` so `audit_events` table is written on every real-backend write (closes p83 F3); `bus_write_audit_completeness.rs` MUST query `audit_events` (not just `audit_events_cache`) so the OP-3 dual-table assertion is real not metaphorical; `agent_flow_real.rs` ships a smoke that performs `git push` against TokenWorld and asserts BOTH cache + backend audit tables have rows; the dark-factory third-arm extends to drive a real `git push reposix main` against in-process sim + file-bare mirror, asserting OP-3 dual-table audit row presence (closes p86 F1 / SC #3 restoration); behavioral no-retry verifier replaces source-grep at `bus-write-no-helper-retry` (closes p83 F6).

**Requirements:** RBF-B-01, RBF-B-02, RBF-B-03, RBF-B-04, RBF-B-05, RBF-B-06, RBF-B-07 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN · **Plan:** TBD (P92 plan-overview not yet authored)

**Success criteria:**
1. Two-writer conflict scenario in dark-factory T4 (rebase recovery) completes step 6 + step 7 against sim AND TokenWorld (no fresh root commit on helper-side fetch).
2. After every `git push` from a partial-clone working tree (sim + real Confluence + real GH issues + real JIRA), `audit_events_cache` AND `audit_events` BOTH show rows for the action; `cache.db` is created on first push if missing.
3. `bus_write_audit_completeness.rs` queries both tables; OP-3 dual-table assertion is real not metaphorical (closes p83 F3 / p86 F5).
4. Verifier subagent's "honesty spot-check" treats audit-row absence as RED, not "out of scope for this layer" (consumes RAISE LIST output from P90).
5. Behavioral no-retry verifier at `bus-write-no-helper-retry` replaces source-grep approach (closes p83 F6).
6. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T1 + T4 against sim AND TokenWorld. If ≥1 HIGH-severity friction surfaces, this phase REOPENS — P93 MUST NOT start until T1 + T4 return ≤0 HIGH frictions. **NOTE (honesty caveat):** RBF-B-01 (rebase ancestry) is a debugger-required investigation; if it bottoms out >16h at day-1 research, split as P92a (rebase) / P92b (audit) per REMEDIATION-PLAN § "Honesty caveats" #1.
7. Catalog rows mint NOT-VERIFIED first (with `coverage_kind: real-backend` per RBF-FW-06); CLAUDE.md updated in same PR.
8. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p92/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P92" + § "Cluster B" + § "Cluster C" finding inventory (H-B1..B4); `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T{1,4}.md`; `.planning/research/v0.13.0-real-backend-frictions/02-phase-audits-may08/{phase-audit-p83,phase-audit-p86,vision-audit}.md`; `crates/reposix-remote/src/`; `crates/reposix-confluence/src/`; CLAUDE.md § "Operating Principles" OP-3.

### Phase 93: L2/L3 cache-coherence + SotPartialFail recovery (Decision 2 promotion)

**Goal:** Pull two items out of v0.14.0 deferral that the v0.13.0 round-trip vision actually depends on (Decision 2 ratified): (a) L2/L3 cache-coherence redesign so `refresh_for_mirror_head` doesn't silently re-run `list_records` on the no-op-push path; (b) `SotPartialFail` + recovery-via-fetch-replan-push test so partial-failure recovery is exercised, not just architected. Land an ADR in `docs/decisions/` capturing the L2 = re-fetch-on-cache-miss vs L3 = transactional-cache-writes trade-off + chosen path; implement `refresh_for_mirror_head` so it no longer no-ops on the post-write path (honest L1 promise without asterisk per RBF-LR-02); ship the `SotPartialFail` recovery test that simulates SoT-success + mirror-fail and asserts the next push reads new SoT via PRECHECK B and replans correctly; ship `agent_flow_real.rs` `partial_failure_recovery_real_*` `#[ignore]` smoke for at least Confluence (TokenWorld arm).

**Requirements:** RBF-LR-01, RBF-LR-02, RBF-LR-03, RBF-LR-04, RBF-LR-05 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN, P92 GREEN · **Execution mode:** `gsd-execute-phase` for code; ADR (RBF-LR-01) is top-level · **Plan:** TBD (P93 plan-overview not yet authored)

**Success criteria:**
1. L2/L3 ADR landed in `docs/decisions/` with chosen path + trade-off doc; orchestrator runs ADR authorship as top-level work (architectural decision-making outside `gsd-execute-phase` envelope).
2. `cargo test -p reposix-cache --test cache_coherence` passes against the chosen architecture; `refresh_for_mirror_head` no longer no-ops on the post-write path.
3. `partial_failure_recovery_real_confluence` smoke GREEN with credentials, default-skipped without (gated by `cadence: pre-release-real-backend` from P89).
4. CLAUDE.md / docs L1 promise updated to remove the asterisk if RBF-LR-02 lands honest, OR keep asterisk + qualify in `dvcs-topology.md` if architectural reality requires it (per RBF-LR-02 outcome).
5. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T1 + T4 against sim AND TokenWorld (the relevant real backends for cache-coherence + recovery). If ≥1 HIGH-severity friction surfaces, this phase REOPENS — P94 MUST NOT start until T1 + T4 return ≤0 HIGH frictions. **NOTE (honesty caveat):** if RBF-LR-01 ADR concludes wider Cache-crate refactoring is required, split as P93a / P93b — but do NOT defer back to v0.14.0 without explicit owner sign-off (re-instates the C8 anti-pattern Decision 2 closes), per REMEDIATION-PLAN § "Honesty caveats" #4.
6. Catalog rows for RBF-LR-01..05 land first; CLAUDE.md updated in same PR.
7. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p93/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P93" + § "Honesty caveats" #4; `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § Decision 2 (S3 promotion); `.planning/research/v0.13.0-real-backend-frictions/02-phase-audits-may08/phase-audit-p81.md`; `crates/reposix-cache/`; `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (originating deferral).

### Phase 94: Bus-push compatibility with documented mirror setup (Cluster D)

**Goal:** Fix the architectural collision where the helper export validator (`diff.rs:99-123`) rejects exactly the files the documented mirror-setup tells users to commit (`.github/workflows/*.yml`, `README.md`, `.reposix/*`). Land an ADR in `docs/decisions/` choosing skip-list vs allowlist vs path-prefix scope (e.g. `?records_root=pages/`); implement the chosen path so helper export accepts (or skips) non-frontmatter files in the mirror tree (closes H-C1); ensure bus push against a mirror with `.github/workflows/reposix-mirror-sync.yml` succeeds (T3 step 5 passes); fix the workflow YAML self-clobber issue per RBF-C-01 ADR (workflow YAML lives in dedicated branch `refs/heads/sot-mirror`, OR workflow re-commits itself to `/tmp/sot` before push, OR sibling-repo pattern — closes H-C3); document the STEP 0 / PRECHECK A three-step prereq chain (`git remote set-url origin reposix::<sot>?mirror=<plain>` + `git remote add mirror <plain>` + `git fetch mirror`) explicitly in `dvcs-mirror-setup.md` (closes H-C2); document the bus URL form `reposix::<sot>?mirror=<url>` in `dvcs-topology.md`, `dvcs-mirror-setup.md`, `troubleshooting.md` (closes H-I14); and validate `bus_url::parse` against unencoded `?` in mirror value (or move "MUST encode" to documented-silent-encoding behavior per p82 F2/F6).

**Requirements:** RBF-C-01, RBF-C-02, RBF-C-03, RBF-C-04, RBF-C-05, RBF-C-06, RBF-C-07 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN, P92 GREEN, P93 GREEN · **Execution mode:** `gsd-execute-phase` for code; ADR (RBF-C-01) is top-level · **Plan:** TBD (P94 plan-overview not yet authored)

**Success criteria:**
1. `reposix attach confluence::REPOSIX` then `git push` succeeds against `reubenjohn/reposix-tokenworld-mirror` with a `.github/workflows/reposix-mirror-sync.yml` present in the mirror tree.
2. First successful workflow run does NOT delete the workflow file from `main` (or RBF-C-04 ADR documents alternate topology).
3. ADR landed in `docs/decisions/` and reviewed; orchestrator runs ADR authorship as top-level work; chosen path between skip-list / allowlist / path-prefix-scope captured with trade-off doc.
4. T3 in dark-factory exercise (`docs/research/dark-factory-may02/T3.md` bus-push round-trip) re-runs and passes 8/8 boxes.
5. `dvcs-mirror-setup.md` documents the STEP 0 / PRECHECK A three-step prereq chain explicitly; `dvcs-topology.md` + `troubleshooting.md` document the `reposix::<sot>?mirror=<url>` URL form (closes H-I14).
6. `bus_url::parse` validates / errors on unencoded `?` in mirror value (or doc moves "MUST encode" → documented-silent-encoding behavior with worked example, closes p82 F2 + F6).
7. **Mid-stream litmus checkpoint (Decision 1):** After this phase declares GREEN, re-run dark-factory T3 against TokenWorld + GH mirror. If ≥1 HIGH-severity friction surfaces, this phase REOPENS — P95 MUST NOT start until T3 returns ≤0 HIGH frictions. **NOTE (honesty caveat):** if RBF-C-01 ADR conclusion is "path-prefix scope," RBF-C-02 implementation may grow to L; flag for re-scoping after ADR per REMEDIATION-PLAN § "Honesty caveats" #2.
8. Catalog rows for RBF-C-01..07 land first; CLAUDE.md updated in same PR.
9. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p94/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P94" + § "Cluster D" finding inventory (H-C1..C3); `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T3.md`; `.planning/research/v0.13.0-real-backend-frictions/02-phase-audits-may08/{phase-audit-p82,phase-audit-p83,phase-audit-p84,phase-audit-p85,vision-audit}.md`; `crates/reposix-remote/src/diff.rs`; `docs/guides/dvcs-mirror-setup.md`; `docs/concepts/dvcs-topology.md`; `docs/guides/troubleshooting.md`.

### Phase 95: Quality framework upgrade + doc fixes (Clusters E + F + H + I)

**Goal:** Apply the F-K2 / F-K4 / F-K8 framework rules from P89/P90 to existing P78–P88 catalog rows (driven by P90's RAISE LIST); fix init UX nits (the "Next:" hint contradicts itself, WARN-noise on success path, bare `git push` requires `--set-upstream` per dark-factory CLUSTER E F1+F2+F5); refresh README + `first-run.md` tutorial expected output to match actual sim seed (CLUSTER F); fix `testing-targets.md` REPOSIX-vs-TokenWorld gap + add tenant probe verifier (CLUSTER H1); add `reposix attach` mention to README + surface git ≥2.34 requirement; ship Pattern C tutorial at `docs/tutorials/round-tripper.md` cross-linked from `dvcs-topology.md` Pattern C (closes p85 F4 / F11 / F16); migrate every existing P78-P88 catalog row to F-K2/F-K4 honesty rules per the RAISE LIST (each `kind: mechanical` row whose description implies functional verification gets a `coverage_kind` field; transport/perf rows get `cadence: pre-release-real-backend`; vacuous rows get `WAIVED + until_date`); WAIVE `webhook-latency-floor` until `cargo binstall reposix-cli` substrate ships (RBF-D-07); fix the `cargo binstall reposix-cli` `pkg-url` template alignment with release-pipeline archive name (RBF-D-08); implement `scripts/webhook-latency-measure.sh --synthetic` flag (CHANGELOG + RETROSPECTIVE both promised it; doesn't exist — RBF-D-09); fix `init.rs` to write `refs/mirrors/<sot>-{head,synced-at}` so first cron tick of fresh mirror has refs (RBF-D-10) OR drop the `refs/mirrors/*` push line per RBF-C-01 ADR; make `vanilla_fetch_brings_mirror_refs` test exercise actual `git upload-pack --advertise-refs --stateless-rpc` advertisement assertion (RBF-D-11); make `perf_l1.rs` test exercise a non-no-op push OR honestly state the L1 asterisk in CLAUDE.md / docs (RBF-D-12); make DVCS-BUS-FETCH-01 either explicitly assert behavioral fetch over a bus URL succeeds via stateless-connect OR correct row description to "absence of stateless-connect on bus URL" (RBF-D-13); walk the F-K8 dishonest-test triage RAISE LIST end-to-end and fix or `WAIVED + until_date` every flagged row (RBF-D-14); qualify the CLAUDE.md "agent UX is pure git" / "Zero reposix CLI awareness required beyond init/attach" claims with backend-coverage state OR confirm P92's real-backend gate is GREEN such that the unqualified claim is now accurate (RBF-D-15).

**Requirements:** RBF-D-01, RBF-D-02, RBF-D-03, RBF-D-04, RBF-D-05, RBF-D-06, RBF-D-07, RBF-D-08, RBF-D-09, RBF-D-10, RBF-D-11, RBF-D-12, RBF-D-13, RBF-D-14, RBF-D-15 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN, P92 GREEN, P93 GREEN, P94 GREEN · **Execution mode:** `gsd-execute-phase` for D-01..05 / D-08 / D-09 (code/doc); top-level for D-06 / D-14 (RAISE LIST drain is orchestration-shaped fan-out) · **Plan:** TBD (P95 plan-overview not yet authored)

**Success criteria:**
1. Dark-factory T1 (sim end-to-end) re-runs and passes 8/8 boxes (cluster E + F resolved).
2. P78–P88 RAISE LIST from P90 fully drained: every flagged row now passes F-K4/F-K8 rules (commits + WAIVED+until_date entries cited in `quality/reports/raise-list-p90.md` resolution column).
3. mkdocs + mermaid + banned-words + cold-reader passes; new Pattern C tutorial at `docs/tutorials/round-tripper.md` cross-linked from `dvcs-topology.md`.
4. CLAUDE.md "pure git" claim either qualified with backend-coverage state OR P92-real-backend-gate-green; `testing-targets.md` corrected (REPOSIX vs TokenWorld); README adds `reposix attach` mention + git ≥2.34 surface.
5. README + `first-run.md` cold-reader pass GREEN: run `/reposix-quality-review --rubric cold-reader-hero-clarity` after refresh; freshness_ttl artifacts present.
6. **Split decision (honesty caveat #3):** if P90's RAISE LIST is larger than estimated, the orchestrator splits this phase as P95a (RBF-D-01..09 — init UX + tutorial + testing-targets + docs honesty) / P95b (RBF-D-10..15 — catalog migration + framework-rule application + claim qualifier). Decision is made BEFORE plan authoring, after P90's RAISE LIST review, per REMEDIATION-PLAN § "Honesty caveats" #3.
7. Catalog rows for RBF-D-01..15 land first (with appropriate `coverage_kind` / `cadence` / `WAIVED+until_date` per the framework rules from P89/P90); CLAUDE.md updated in same PR.
8. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p95/VERDICT.md` (or `p95a` + `p95b` if split).

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P95" + § "Honesty caveats" #3 + § "Cluster E / F / H / I" finding inventory (H-E1, H-F1, H-G4 qualifier, H-H6, H-I3..I19); `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/{T1.md,SUMMARY.md}`; `.planning/research/v0.13.0-real-backend-frictions/02-phase-audits-may08/{phase-audit-p78,phase-audit-p80,phase-audit-p81,phase-audit-p82,phase-audit-p84,phase-audit-p85}.md`; `quality/reports/raise-list-p90.md` (P90 deliverable); `docs/reference/testing-targets.md`; `docs/tutorials/first-run.md`; CLAUDE.md § Architecture + § "Quick links".

### Phase 96: Surprises absorption (+2 reservation slot 1, OP-8)

**Goal:** Drain anything P89–P95 surfaced but couldn't fix without doubling scope; retroactively file the 16 dark-factory HIGH frictions + ~51 audit HIGH findings as v0.13.0-discovered intake (closes the chronological-defensibility issue from p87 F8 / vision-audit F8). Apply the F-K5 honesty-spot-check meta-rule from P90: spot-check author ≠ orchestrator (dispatched as fresh independent subagent); sample includes EVERY no-intake phase across the entire P78–P95 series; rubric = "walk one critical example end-to-end mentally — does it work?"; content-hash binding (not just file existence). Amend RETROSPECTIVE.md v0.13.0 section: the 16 HIGH dark-factory frictions added under "What Was Inefficient"; root-cause analysis ("framework structurally exempted real-backend flows") added under "Patterns Established (Anti-pattern)" — closes p88 F3 + F9 / H-I18. Apply the `EXTENDED-PENDING-P89-P97` overlay note (per Decision 4 option (b)) to the 12 existing GREEN verdict files at `quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md` + `milestone-v0.13.0/VERDICT.md` — a 2026-05-08-dated banner at the top of each file pointing forward to P97's milestone-close verdict for the post-extension state.

**P96 supersedes P87 as the active +2 reservation slot 1 for the EXTENDED v0.13.0; P87's intake content folds forward via RBF-S-01.**

**Requirements:** RBF-S-01, RBF-S-02, RBF-S-03, RBF-S-04, RBF-S-05 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN, P92 GREEN, P93 GREEN, P94 GREEN, P95 GREEN · **Execution mode:** top-level (orchestration-shaped fan-out: dispatch independent honesty subagent + drain intake + amend RETROSPECTIVE + verdict-overlay sweep) · **Plan:** TBD (P96 plan-overview not yet authored)

**Success criteria:**
1. Every v0.13.0-extension `SURPRISES-INTAKE.md` entry has terminal STATUS (RESOLVED + commit SHA / DEFERRED + target milestone + rationale / WONTFIX + rationale). No `STATUS: TBD` at phase close. P87's prior intake content folds forward (RBF-S-01).
2. Retroactive intake entries for the 16 dark-factory HIGH frictions + ~51 audit HIGH findings filed against their originating phases (P79 / P80 / P83 / P86 / P87 / P88 etc.); each entry traces to the originating audit file (closes vision-audit F8 — chronologically defensible because dark-factory was post-tag, but documenting the gap is OP-9 honesty).
3. Independent (non-orchestrator) honesty-spot-check subagent dispatched per F-K5 meta-rule from P90: sample includes EVERY no-intake phase, rubric = "walk one critical example end-to-end mentally — does it work?", content-hash binding present; spot-check report at `quality/reports/verdicts/p96/honesty-spot-check.md`; verdict GREEN under F-K5 rubric.
4. RETROSPECTIVE.md v0.13.0 amendment merged: the 16 HIGH dark-factory frictions under "What Was Inefficient"; root-cause "framework structurally exempted real-backend flows" under "Patterns Established (Anti-pattern)" (closes p88 F3 + F9 / H-I18).
5. `EXTENDED-PENDING-P89-P97` overlay banner applied (per Decision 4 option (b)) to all 12 verdicts: `quality/reports/verdicts/p7{8,9}/`, `quality/reports/verdicts/p8{0..8}/`, and `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. Each banner is 2026-05-08-dated and points forward to P97's milestone-close verdict.
6. Verifier honesty spot-check graded RED if any P78–P95 verdict shows skipped findings without an Eager-resolution OR intake entry (CLAUDE.md OP-8 honesty-check).
7. Catalog rows for RBF-S-01..05 land first; CLAUDE.md updated in same PR (notably: §0.8 SESSION-END-STATE framework if revised + the +2 reservation transition note).
8. Phase close: `git push origin main`; verifier subagent grades GREEN AND signs honesty spot-check; verdict at `quality/reports/verdicts/p96/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P96"; `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § Decision 4 (verdict-overlay option (b)); CLAUDE.md § "Operating Principles" OP-8 + OP-9 + § "Meta-rule: when an owner catches a quality miss, fix it twice"; `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`; `.planning/milestones/v0.12.1-phases/` P76 verdict (precedent for honesty-check execution); the 12 existing GREEN verdict files at `quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md` + `milestone-v0.13.0/VERDICT.md`.

### Phase 97: Good-to-haves polish + milestone close (+2 reservation slot 2, OP-9 ritual) — tag v0.13.0

**Goal:** Drain `GOOD-TO-HAVES.md` for the v0.13.0 extension (XS items always close; M items default-defer to v0.14.0) per OP-8 sizing rules; cold-reader pass on all revised docs (`/reposix-quality-review --all-stale`); RETROSPECTIVE.md distillation for the v0.13.0 extension per OP-9 ritual (What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons); milestone-close verdict using the new 9-probe template from P89 (RBF-FW-03) — **OVERWRITES `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`** (replacing the prior 2026-05-01 GREEN verdict with the post-extension verdict per Decision 4); the 9th probe is the vision litmus test against TokenWorld and requires P91's RBF-A-05 + P92's RBF-B-06 + P93's RBF-LR-04 + P94's RBF-C-03 + P95's RBF-D-15 ALL green for the probe to fire green; tag `v0.13.0` (owner runs `tag-v0.13.0.sh` and pushes the tag — orchestrator does NOT push the tag).

**P97 supersedes P88 as the active +2 reservation slot 2 for the EXTENDED v0.13.0; tag-script and CHANGELOG carry forward from P88's prior work.**

**Requirements:** RBF-G-01, RBF-G-02, RBF-G-03, RBF-G-04, RBF-G-05 · **Depends on:** P89 GREEN, P90 GREEN, P91 GREEN, P92 GREEN, P93 GREEN, P94 GREEN, P95 GREEN, P96 GREEN · **Execution mode:** top-level · **Plan:** TBD (P97 plan-overview not yet authored)

**Success criteria:**
1. `GOOD-TO-HAVES.md` (v0.13.0 extension) drained: every entry terminal STATUS — XS closed (commit SHA), S closed-or-deferred (rationale), M default-deferred to v0.14.0 (carry-forward target named).
2. Cold-reader pass GREEN: `/reposix-quality-review --all-stale` produces no critical-friction findings; freshness artifacts present in `quality/reports/verifications/subjective/`.
3. RETROSPECTIVE.md v0.13.0 extension section exists with all five OP-9 subheadings (What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons) + substantive content sourced from SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts (P89–P96) + autonomous-run findings.
4. Milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is GREEN under the new 9-probe template from P89 (the prior 2026-05-01 verdict is OVERWRITTEN per Decision 4); the 9th probe (vision litmus test against TokenWorld) ran and passed; closes H-G2 + H-I19 by construction.
5. Re-running the dark-factory exercise produces ≤ 5 frictions (down from 37) and ZERO HIGH (down from 16).
6. `tag-v0.13.0.sh` re-runs cleanly post-P97; tag-gate guards (≥6 safety guards mirroring v0.12.0 precedent) all pass against the post-extension tree; CHANGELOG `[v0.13.0]` updated to enumerate P78–P97 (extension series called out distinctly).
7. STOP at tag boundary: orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.0 (extended) ready-to-tag; owner pushes tag."
8. Catalog rows + CLAUDE.md v0.13.0-shipped historical-milestone subsection (with extension series annotated) land first; CLAUDE.md updated in same PR.
9. Phase close: `git push origin main`; milestone-close verifier GREEN; verdict at `quality/reports/verdicts/p97/VERDICT.md` + the OVERWRITTEN `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md` § "P97"; `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md` § Decision 4 (verdict overwrite); CLAUDE.md § "Operating Principles" OP-8 + OP-9; `.planning/milestones/v0.13.0-phases/{GOOD-TO-HAVES.md,tag-v0.13.0.sh}`; `.planning/RETROSPECTIVE.md`; `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` + `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (tag-script precedents); `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (target of overwrite).
