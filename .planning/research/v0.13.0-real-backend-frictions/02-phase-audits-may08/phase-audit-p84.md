# Phase P84 Audit — Webhook-driven mirror sync (GH Action workflow + setup guide)
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 6
- MISALIGNED items: 9
- SUSPECT items: 2

The phase shipped six catalog rows that PASS today, but the rows describe **structural invariants on a YAML file** rather than the load-bearing claim "Confluence edit lands in mirror within p95 ≤ 120s." The phase's central runtime guarantee is not exercised by any verifier (synthetic n=1 placeholder satisfies the threshold vacuously; real-backend run gated forever on the documented substrate gap). Worse, the documented mirror-setup walk-through contains a self-defeating step: the very first successful workflow run will force-push SoT-only content to mirror's `main`, deleting `.github/workflows/reposix-mirror-sync.yml` from the default branch and severing the workflow from itself.

## Findings

### F1 — Workflow self-clobbers itself on first successful run [SEVERITY: HIGH]
**Claim in plan / docs:** `docs/guides/dvcs-mirror-setup.md` Step 2 has the owner `git push origin main` a `.github/workflows/reposix-mirror-sync.yml` into the mirror repo's `main`. Step 4 says: "trigger it manually... It pushes the rendered Markdown to refs/heads/main of the mirror repo with --force-with-lease... The job exits 0."
**Reality:** The workflow `cd /tmp/sot && git push mirror "refs/heads/main:refs/heads/main" --force-with-lease=...` (template lines 89-98). `/tmp/sot` is a fresh `reposix init` checkout — its `main` history contains only SoT-derived commits and zero connection to the mirror's previous history (see `crates/reposix-cli/src/init.rs:1-15`: init runs `git init <path>` from scratch). After the first successful push, mirror's `main` no longer contains `.github/workflows/reposix-mirror-sync.yml` because that file was never in the SoT. GitHub Actions reads `repository_dispatch`/`schedule` workflow definitions from the default branch — once the workflow vanishes from `main`, every subsequent dispatch + cron tick has nothing to run. The workflow can run successfully **at most once**, and after that dispatch becomes a no-op silently.
**Evidence:**
- `docs/guides/dvcs-mirror-setup-template.yml:86-98` (force-push from `/tmp/sot`)
- `docs/guides/dvcs-mirror-setup.md:35-48` (Step 2 commits the workflow into mirror main)
- `crates/reposix-cli/src/init.rs:1-15` (init builds a fresh `.git`)
- Corroborating dark-factory observation: `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T2-attach.md:41-44` notes the live `reubenjohn/reposix-tokenworld-mirror` clone contains only `README.md` + the workflow YAML — meaning the live mirror has NEVER had a successful end-to-end sync at the time of audit (consistent with the SURPRISES-INTAKE substrate gap, but also consistent with the self-clobber hypothesis above).
**Why it matters:** The documented user flow promises a working sync loop; in fact the loop is one-shot. Even if the binstall + yanked-gix substrate gap (SURPRISES-INTAKE 2026-05-01 16:43) is fixed, the FIRST successful sync deletes the workflow from main. No catalog row checks this; no troubleshooting entry warns about it. The fix requires either (a) a separate branch for the workflow YAML, (b) the workflow re-committing itself to `/tmp/sot` before push, or (c) making the workflow live in a sibling repo. Architecture-level decision required.

### F2 — `webhook-latency-floor` row passes vacuously and has no freshness_ttl, so it stays GREEN forever [SEVERITY: HIGH]
**Claim in plan:** ROADMAP P84 SC4 + REQ DVCS-WEBHOOK-04: "p95 latency target ≤ 120s from confluence edit to GH ref update" — falsifiable threshold. Verdict (line 25): "p95=5s within 120s threshold."
**Reality:** `quality/reports/verifications/perf/webhook-latency.json` contains `"method": "synthetic-dispatch", "n": 1, "p50_seconds": 5, "p95_seconds": 5` — the 5s value measures **dispatch-API call → runner pickup**, NOT confluence-edit → mirror-ref-update (the JSON's own `note` field admits this). The catalog row's `freshness_ttl: null` (line 983 of `quality/catalogs/agent-ux.json`) means the placeholder satisfies `p95 ≤ 120` permanently — there is no future-dated re-grade requirement.
**Evidence:**
- `quality/reports/verifications/perf/webhook-latency.json:1-11`
- `quality/catalogs/agent-ux.json:980-989` (`freshness_ttl: null`, `cadences: ["pre-release"]`)
- `quality/gates/agent-ux/webhook-latency-floor.sh:14-20` (verifier reads p95 only; doesn't check `n`, `method`, `measured_at`, or `verdict`)
- SURPRISES-INTAKE entry 2026-05-01 16:43 says "the row's freshness_ttl + the post-release re-measurement together close the loop" — but the row's freshness_ttl is `null` not (e.g.) `90d`, so there's no enforced re-measurement deadline.
**Why it matters:** The load-bearing requirement ("agent edit propagates to mirror in ≤120s") has zero coverage. The row pretends a synthetic 5s dispatch latency is the real measurement. With no TTL, the placeholder will outlive the milestone indefinitely. This is exactly the "URL-shape only" / "exists-only" failure mode named in AUDIT-BRIEF § "Failure shapes" — the row's `description` (p95 ≤ 120s) implies functional verification; the assertion is a structural read of a JSON field whose value is documentedly synthetic.

### F3 — `cargo binstall reposix-cli` cannot succeed; `release/cargo-binstall-resolves` was waived in v0.12.0 and waiver is still active in v0.13.0 [SEVERITY: HIGH]
**Claim in plan:** Workflow YAML line 58: `cargo binstall --no-confirm reposix-cli`. PLAN-OVERVIEW line 92-93: "cargo binstall reposix-cli (NOT 'reposix' — published crate name per RESEARCH Pitfall 2)". Verdict (line 33): "`cargo binstall reposix-cli` (NOT bare `reposix`): present (line 58)" graded PASS.
**Reality:** `crates/reposix-cli/Cargo.toml:19-21` declares `pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.{ archive-format }"` (i.e., expects a file named `reposix-cli-x86_64-unknown-linux-gnu.tgz`). `.github/workflows/release.yml:166-188` stages release archives as `reposix-${VERSION_TAG}-${target}.tar.gz` (e.g., `reposix-v0.11.3-x86_64-unknown-linux-gnu.tar.gz`) with extension `.tar.gz`. Both name and archive format are wrong vs what binstall metadata expects. The release dimension catalog row `release/cargo-binstall-resolves` carries a WAIVED status (`quality/catalogs/release-assets.json:535-545`) explicitly documenting this mismatch as a 4-point misalignment that "v0.12.1 MIGRATE-03 ships". v0.12.1 has not shipped; v0.13.0 is current. The waiver expires 2026-07-26.
**Evidence:**
- `crates/reposix-cli/Cargo.toml:19-21`
- `.github/workflows/release.yml:166-188` (archive name + format)
- `quality/catalogs/release-assets.json:535-545` (active WAIVER)
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:68-76` (P84 T05 confirms binstall fall-through to source-build, which then fails on yanked gix=0.82.0)
**Why it matters:** Step 4 of the user-facing setup guide ("Smoke-test with a manual run") instructs the owner to run `gh workflow run reposix-mirror-sync.yml` and expect the install step to take ~10s. In reality the workflow halts at step 2 every run on every published version. The verifier's binstall structural grep (`quality/gates/agent-ux/webhook-trigger-dispatch.sh:54-62`) only checks the *spelling* `reposix-cli` vs `reposix` — it never verifies the binstall command actually resolves. The verdict's PASS for this row is form-over-substance (CLUSTER A in the dark-factory taxonomy).

### F4 — None of the 5 structural verifiers run in CI; only locally in `--cadence pre-pr`, which no automation invokes [SEVERITY: MED]
**Claim in plan:** PLAN-OVERVIEW T06: "Run `python3 quality/runners/run.py --cadence pre-pr --tag webhook` to flip the 5 pre-pr rows FAIL → PASS." VERDICT (line 11-15): all 5 PASS.
**Reality:** `cadences: ["pre-pr"]` for all 5 (`quality/catalogs/agent-ux.json` rows 808, 845, 880, 916, 951). The pre-push hook (`.githooks/pre-push:57`) calls `--cadence pre-push` only. CI workflows (`.github/workflows/quality-pre-release.yml:46`, `quality-post-release.yml:42`, `quality-weekly.yml:34`) call `pre-release`, `post-release`, `weekly`. Grep across `.github/workflows/` for `pre-pr` returns zero invocations. `freshness_ttl: null` on all 5 rows means the runner won't re-grade them on its own. Result: these verifiers fired exactly once in T06 (2026-05-01 16:48 UTC, per the catalog `last_verified` fields) and have not run since.
**Evidence:**
- `.githooks/pre-push:57`
- `.github/workflows/quality-{pre,post}-release.yml`, `quality-weekly.yml` (no `pre-pr`)
- `quality/catalogs/agent-ux.json` rows 802-810 / 838-846 / 873-881 / 909-917 / 944-952 (cadence + freshness_ttl)
**Why it matters:** Future drift in `docs/guides/dvcs-mirror-setup-template.yml` (or in the live mirror-repo copy) is undetected by automation. Specifically, the live copy at `reubenjohn/reposix-tokenworld-mirror:.github/workflows/reposix-mirror-sync.yml` could change without the template, and only a manual `python3 quality/runners/run.py --cadence pre-pr` would catch it. The `webhook-trigger-dispatch.sh` byte-equal-modulo-whitespace check is a useful invariant; it just isn't *enforced*.

### F5 — `webhook-trigger-dispatch.sh` requires `gh auth status` + access to a private/public org repo; if either is missing the verifier FAILs noisily — yet the row PASSes today only because the auditor's local `gh` is authed [SEVERITY: MED]
**Claim in plan:** PLAN-OVERVIEW T02: "T02's verifier asserts (a) the template exists in the canonical repo, AND (b) `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml` returns 200 — so the test fails if either copy is missing." VERDICT (line 11): PASS.
**Reality:** `quality/gates/agent-ux/webhook-trigger-dispatch.sh:31-39` calls `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml -q .content` and `base64 -d`s the result. If the user is not `gh auth login`'d, or has no access to that repo, the verifier prints `FAIL: live copy ... unreachable` and exits 1. The verifier has no fallback (no `pre-push`/CI-context detection, no skip-when-no-network mode). Combined with F4 (verifier never runs in CI), this means: PASS today depends on (a) someone running `--cadence pre-pr` locally, (b) with `gh` authed against the user's account, (c) which has access to `reubenjohn/reposix-tokenworld-mirror`. Different graders see different verdicts.
**Evidence:** `quality/gates/agent-ux/webhook-trigger-dispatch.sh:31-39`.
**Why it matters:** This pattern (verifier output depends on grader's local state) is a quality-framework integrity issue. A new contributor or a CI sandbox would see RED for "the YAML is fine"; the catalog state would silently flip on a push. The verifier should either (a) skip with PARTIAL when `gh` is unavailable, (b) embed a checked-in snapshot of the live copy and diff against it, or (c) be tagged `cadence: weekly` with a clear "owner-only" gate.

### F6 — Verdict's "byte-equal mod whitespace" claim is a stale snapshot; the verifier passes only when the auditor has live API access [SEVERITY: MED]
**Claim in verdict:** Line 11: "workflow YAML present in both copies, byte-equal mod whitespace". Line 23: "byte-equal-mod-ws verified by trigger-dispatch verifier" (commit `e4fb6da`).
**Reality:** Same as F5 — but the second-order issue is that the verdict assertion is a one-shot read of the live mirror at a specific moment. The catalog row's `last_verified: 2026-05-01T16:48:03Z` reflects the moment, but `freshness_ttl: null` means there is no "this assertion is stale after N days" pressure. If the mirror's live copy drifts tomorrow, no automation flags it.
**Evidence:** `quality/catalogs/agent-ux.json:801-803`, `quality/gates/agent-ux/webhook-trigger-dispatch.sh:42-44`.
**Why it matters:** Equivalent to F4 in the freshness dimension; surfaced as a separate finding because the byte-equal contract is a load-bearing claim (the template file in `docs/` is what users curl; it must match the live workflow). Without freshness_ttl + automation, drift is invisible.

### F7 — Documented `scripts/webhook-latency-measure.sh --synthetic` flag does not exist [SEVERITY: MED]
**Claim in docs:** `CHANGELOG.md:22`: "Owner-runnable `scripts/webhook-latency-measure.sh --synthetic` measures dispatch -> ref-update p95 against the live mirror." `.planning/RETROSPECTIVE.md:29`: "`scripts/webhook-latency-measure.sh --synthetic` preserves the diagnosis without doubling P84's scope." `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:74` notes the gap: "TODO: add `--synthetic` flag to the script".
**Reality:** `scripts/webhook-latency-measure.sh` accepts no flags. It runs the interactive 10-iteration TokenWorld measurement loop (`for i in $(seq 1 10)`, line 24). Running with `--synthetic` does nothing — the unrecognized arg is silently ignored.
**Evidence:** `scripts/webhook-latency-measure.sh:1-77` (no argparse / getopts / `case $1`).
**Why it matters:** The published CHANGELOG and the v0.13.0 RETROSPECTIVE both promise an entrypoint that doesn't exist. An owner running the documented command gets the interactive 10-real-edit flow instead of the synthetic dispatch measurement they expected — and the SURPRISES-INTAKE deferral assumed the synthetic flag would let the v0.13.x release re-measure without manual edits. Cluster H (doc-vs-impl drift) regression.

### F8 — Cluster G/H from dark-factory: bus push rejects `.github/workflows/*.yml` — but the mirror-setup guide commits exactly that file [SEVERITY: HIGH]
**Claim in plan / docs:** `docs/guides/dvcs-mirror-setup.md:35-48` Step 2 instructs the owner to commit `.github/workflows/reposix-mirror-sync.yml` to the mirror repo's main. The DVCS topology (`docs/concepts/dvcs-topology.md`, "round-tripper" pattern) lets a user `reposix attach` against this mirror and `git push reposix` via the bus URL.
**Reality:** Already documented by the dark-factory exercise (T3-bus-push.md F12, SUMMARY.md cluster H): the helper's frontmatter validator rejects any blob without `---` fences. Bus push against a mirror that follows the documented setup fails with `invalid record file: missing frontmatter open fence`. The audit re-cites this for completeness because P84's setup guide is the surface that *generates* the mirror state which P82+P83's bus push then refuses.
**Evidence:** `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T3-bus-push.md:160-175`, SUMMARY.md cluster G/H.
**Why it matters:** P84 ships the setup guide that creates the failure surface for P82/P83. Cross-phase coherence break: P84's expected mirror state and P82+P83's accepted mirror state are inconsistent. This is not a P84-only fix — but P84 is the surface where it manifests for owners following the docs.

### F9 — `webhook-cron-fallback.sh` does not check that the workflow_dispatch trigger is present, weakening the Q4.2 "backends without webhooks" backstop [SEVERITY: LOW]
**Claim in plan:** Catalog row description (`agent-ux/webhook-backends-without-webhooks`): "after trim, at least one trigger remains (schedule or workflow_dispatch)".
**Reality:** `webhook-backends-without-webhooks.sh:41` asserts `on_block.get('schedule') or 'workflow_dispatch' in on_block`. But the *positive* check (template HAS workflow_dispatch in the first place) is only present in the comment header of `webhook-cron-fallback.sh:1-6`, not in any assertion. If a future edit removes both `repository_dispatch` AND `workflow_dispatch` from the template (leaving only `schedule`), the cron-fallback verifier still PASSes (cron literal grep + concurrency block grep both fire), and the Q4.2 trim verifier still PASSes (schedule-only is "at least one trigger"). The combo loses manual-dispatch capability with zero CI signal.
**Evidence:** `quality/gates/agent-ux/webhook-cron-fallback.sh:11-32`, `quality/gates/agent-ux/webhook-backends-without-webhooks.sh:25-44`.
**Why it matters:** Defense-in-depth gap on the template's manual-dispatch path. Low severity because the template currently has `workflow_dispatch:` (line 23) and no edit pressure.

### F10 — ROADMAP says "Latency target: < 60s p95"; verifier threshold is `<=120`; verdict assertion is `p95=5s within 120s` — three numbers, no clear single SC [SEVERITY: LOW]
**Claim in plan / ROADMAP:** ROADMAP line 61: "Latency target: < 60s p95 from confluence edit to GH ref update." PLAN-OVERVIEW line 41-42: "< 60s p95 (aspirational); 120s p95 (falsifiable threshold per ROADMAP SC4)." JSON `target_seconds: 60`. Verifier `[ "$P95" -le 120 ]`.
**Reality:** Three different numeric thresholds appear (60 aspirational target, 120 falsifiable threshold, the synthetic value 5 reported as headline). The "< 60s" in the ROADMAP is treated as marketing-soft; only "≤ 120s" is checked. The catalog row description says `p95 ≤ 120` only.
**Evidence:**
- `.planning/milestones/v0.13.0-phases/ROADMAP.md:61`
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW/index.md` (description above)
- `quality/reports/verifications/perf/webhook-latency.json:8` (`target_seconds: 60`)
- `quality/gates/agent-ux/webhook-latency-floor.sh:19`
**Why it matters:** Doc-drift / clarity issue. The 60s "aspirational" floor leaks into the JSON `target_seconds` field but never into a real check. A future reader can't tell if 90s p95 is PASS, soft-FAIL, or marketing-FAIL.

### F11 — Mirror-lag refs push (template lines 102-104) writes `refs/mirrors/confluence-head` + `-synced-at` BUT the CLAUDE.md spec says these refs are written by `reposix-cache` on init/sync [SEVERITY: LOW]
**Claim in CLAUDE.md:** "Mirror-lag refs. Cache writes `refs/mirrors/<sot>-head` (direct ref) and `refs/mirrors/<sot>-synced-at` (annotated tag, message body `mirror synced at <RFC3339>`) per successful sync. Refs live in the cache's bare repo; vanilla `git fetch` brings them along."
**Reality:** Template lines 102-104 do `git push mirror "refs/mirrors/confluence-head" "refs/mirrors/confluence-synced-at" || echo warn`. These refs are pushed FROM `/tmp/sot` (the working tree) — meaning they must exist in `/tmp/sot/.git/refs/mirrors/`. Per CLAUDE.md they are written by the cache layer on init. Cross-checking is OK structurally. But the workflow swallows failures (`|| echo "warn: ..."`) — if the refs aren't populated in `/tmp/sot` for any reason (e.g., a future cache regression), the workflow's refs-push silently fails with no visible signal.
**Evidence:**
- `docs/guides/dvcs-mirror-setup-template.yml:102-104`
- `CLAUDE.md:33-35` (mirror-lag refs spec)
- No verifier in P84 asserts the refs landed on the live mirror after a sync.
**Why it matters:** The "vanilla git fetch brings these along" promise (load-bearing for mirror-only consumers per `docs/concepts/dvcs-topology.md`) has no end-to-end check. If the workflow drops them silently, the staleness signal vanishes for downstream readers without anyone noticing.

### F12 — `gh repo create --public` (Setup Step 1) creates a TRULY-EMPTY mirror — but Setup Step 2 then commits the workflow, making it FRESH-BUT-WORKFLOW. Q4.3.b "truly-empty" path is never actually reached in the documented flow [SEVERITY: LOW]
**Claim in plan:** Q4.3 has two sub-cases: 4.3.a fresh-but-readme (lease-push branch), 4.3.b truly-empty (plain-push branch). T03 verifier covers both.
**Reality:** Following Setup Step 1 + Step 2 verbatim:
- Step 1: `gh repo create <org>/<space>-mirror --public` → truly-empty (no README, no main ref).
- Step 2: `git clone` (fails — empty repo can be cloned but yields a non-bare empty checkout); `git add .github/workflows/reposix-mirror-sync.yml && git push origin main` — this push CREATES `refs/heads/main` on the mirror at the workflow-commit SHA.
- Step 4: workflow runs against a FRESH-BUT-WORKFLOW-YML mirror (4.3.a-style; lease-push branch).
- The truly-empty 4.3.b path is reachable only if the owner skips Step 2 and immediately triggers the workflow on a `gh repo create`-only mirror. The setup guide does not describe this flow.
**Evidence:**
- `docs/guides/dvcs-mirror-setup.md:23-48` (Step 1 + Step 2)
- `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh:83-111` (4.3.b harness path)
**Why it matters:** Coverage-vs-reality mismatch. The 4.3.b verifier exercises a code path that the documented user-flow never hits. Investing in the verifier is fine for defensive-depth, but the verdict's claim "first-run handled gracefully" rests on a flow that doesn't match the docs. Low severity because the 4.3.a path IS exercised and IS the one users hit; the over-coverage is harmless except for slight verdict-honesty inflation.

### F13 — Verdict claims SURPRISES-INTAKE entry is "LEGITIMATE" but does not call out that the deferred work is the load-bearing claim, not a tangential one [SEVERITY: MED]
**Claim in verdict:** Lines 50-58 grade the SURPRISES-INTAKE entry as "LEGITIMATE" — properly scoped, eager-resolution carve-out doesn't apply.
**Reality:** The SURPRISES-INTAKE entry (2026-05-01 16:43, severity HIGH) defers the binstall + yanked-gix substrate gap. The entry says: "no amount of T05-internal work can produce real timings until v0.13.x lands on crates.io with working binstall." But the SC4 falsifiable threshold (p95 ≤ 120s) IS the load-bearing claim of P84 — without real timings, the claim is unverified. Treating the substrate gap as "out-of-scope for P84" is correct from a workload-sizing view; but the verdict's GREEN grade hinges on the synthetic placeholder satisfying the verifier, which sidesteps that the load-bearing claim was deferred. The phase shipped GREEN by lowering the bar (verifier reads p95 from a synthetic JSON) rather than by meeting the bar.
**Evidence:**
- `quality/reports/verdicts/p84/VERDICT.md:50-70`
- `quality/reports/verifications/perf/webhook-latency.json:10` (verbatim deferral note)
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:68-76`
**Why it matters:** This is the verifier-honesty pattern AUDIT-BRIEF § "Failure shapes" #2 ("'substrate gap' / 'deferred' deferrals masquerading as GREEN"). The verdict's "Latency JSON honesty note" (lines 60-70) admits the bar was lowered, but the top-level "GREEN. Zero blockers." headline doesn't propagate the caveat. A fair verdict would be PARTIAL or YELLOW: verifier-form-passes + load-bearing-claim-deferred.

### F14 — Catalog `_provenance_note` cites `GOOD-TO-HAVES-01` deferral as the reason all 6 rows are hand-edited; this is a quality-framework debt that has NOT been verified-resolved [SEVERITY: SUSPECT]
**Claim in plan:** All 6 rows carry: "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension."
**Reality:** Did not validate whether GOOD-TO-HAVES-01 is open/closed at v0.13.0 close.
**Evidence:** Repeated string across `quality/catalogs/agent-ux.json` rows 776, 815, 852, 888, 924, 959.
**What would settle it:** Read `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` § GOOD-TO-HAVES-01; if the entry shipped, the hand-edit rationale is stale and the rows should have been re-minted via the `reposix-quality bind` verb. If still open, this is a tracked debt — not a P84-specific finding, but one that compounds F4 + F6 (manual mints not auto-graded + no freshness_ttl = the manual-mint contract has zero auto-enforcement).
**Why it matters:** Hand-edited rows + no auto-rebind + no freshness ttl is a triple-decker grounding gap. The framework's stated invariant (Principle A: rows minted via `reposix-quality bind`) is broken for the entire P84 row family.

### F15 — Verifier's "PASS" output for `webhook-trigger-dispatch.sh` does not include the live-copy SHA or last-modified timestamp; future drift is invisible to the runner artifact [SEVERITY: SUSPECT]
**Claim:** Catalog row's `artifact: "quality/reports/verifications/agent-ux/webhook-trigger-dispatch.json"` is supposed to record the verification metadata.
**Reality:** Did not read the artifact JSON to confirm what's in it. The verifier shell only echoes `PASS: ...` to stdout (`webhook-trigger-dispatch.sh:72-73`) — there's no `jq` / `tee` / write-to-file. Whether the runner harness wraps the shell's stdout into the artifact is harness-dependent.
**What would settle it:** Read `quality/reports/verifications/agent-ux/webhook-trigger-dispatch.json` (if present) and confirm whether the live-copy SHA from `gh api`'s response is captured; if not, the artifact records only PASS/FAIL without the input fingerprint, so a future re-grade can't compare against last-known state.
**Why it matters:** Drift-detection robustness. Low priority because F4 already documents that no automation auto-grades these rows; if F4 is fixed, F15 becomes a near-term concern.

## Summary cross-reference (severity table)

| F# | Severity | Topic |
|---|---|---|
| F1 | HIGH | Workflow self-clobbers — deletes its own YAML from main on first run |
| F2 | HIGH | Latency floor row passes vacuously, no freshness_ttl |
| F3 | HIGH | `cargo binstall reposix-cli` cannot resolve in any release; PASS is form-only |
| F4 | MED | 5 structural verifiers run nowhere automatic (cadence pre-pr, no CI invocation) |
| F5 | MED | `webhook-trigger-dispatch.sh` requires gh-auth; PASS depends on grader's local state |
| F6 | MED | Byte-equal-mod-ws is a stale snapshot, no freshness pressure |
| F7 | MED | `--synthetic` flag claimed by CHANGELOG/RETROSPECTIVE, doesn't exist |
| F8 | HIGH | Bus push rejects the workflow file the setup guide commits (cross-phase) |
| F9 | LOW | `cron-fallback` verifier doesn't lock workflow_dispatch presence |
| F10 | LOW | Three latency thresholds documented (60 aspirational / 120 falsifiable / 5 reported) |
| F11 | LOW | Mirror-lag-refs push failure is silent; no end-to-end ref-presence check |
| F12 | LOW | Q4.3.b "truly-empty" verifier path is unreachable via documented setup flow |
| F13 | MED | Verdict GREEN headline doesn't propagate "load-bearing claim deferred" caveat |
| F14 | SUSPECT | GOOD-TO-HAVES-01 hand-edit waiver may be stale; would need cross-check |
| F15 | SUSPECT | Verifier artifact may not capture live-copy fingerprint; would need cross-check |

## Recommendations for v0.13.1 framework-fix work
1. **F1 fix is architectural** — store the workflow YAML in a dedicated branch (not `main`), or have the workflow re-add its own file to `/tmp/sot` before pushing, or stop force-pushing main and instead push to `refs/heads/sot-mirror`. This is a P85-doc + P84-template surgery.
2. **F2/F3/F4/F5/F6 are framework-shape**: introduce a `cadence: post-release` re-grade of the 6 webhook rows + `freshness_ttl: 90d` so the synthetic placeholder cannot live forever; either ship a CI job that runs `--cadence pre-pr` on PRs, or move the structural greps to `pre-push`. Ship a CI-friendly fallback for the `gh api` live-copy check (e.g., commit a snapshot SHA into the catalog and diff against `gh api --jq .sha`).
3. **F7 is two-line fix** — add `--synthetic` to the script (or update CHANGELOG/RETROSPECTIVE to remove the flag).
4. **F8** is the cross-cutting helper-frontmatter-validator change tracked in the dark-factory cluster G/H.
5. **F13** — the verifier-prompt template should require: "if the load-bearing claim depends on a deferred substrate, grade YELLOW or PARTIAL even if the verifier exits 0."
