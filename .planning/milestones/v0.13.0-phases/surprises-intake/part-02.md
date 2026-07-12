# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 2 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-04 | discovered-by: P90 90-03 (confirmed live by 90-05) | severity: MEDIUM

**What:** All 4 subjective-rubrics rows' `verifier.args` pass bare rubric slugs (`cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`, `dvcs-cold-reader`) to `--rubric`, but `.claude/skills/reposix-quality-review/dispatch.sh`'s case statement keys on the FULL `subjective/<slug>` id (`"subjective/cold-reader-hero-clarity")`, `"subjective/install-positioning"|"subjective/headline-numbers-sanity"`, `"subjective/dvcs-cold-reader"`). A bare-slug invocation path falls through to no matching case (and, upstream, a runner/catalog `find_row` lookup keyed on the full id would `KeyError`/miss for a bare slug) — the two spellings are inconsistent across the catalog-row/dispatcher boundary. Confirmed live (not hypothetical) by both 90-03 (which wired the `dvcs-cold-reader` dispatch case) and 90-05 (which renewed the 3 subjective waivers and re-read the same `verifier.args` while doing so).

**Why out-of-scope for P90:** P90's mandate is the honesty-rules framework fixes (RBF-FW-06..12); reconciling a pre-existing bare-slug-vs-full-id inconsistency in the dispatcher/catalog contract is a small but distinct wiring fix, not one of the chartered honesty rules, and touching the dispatcher script is outside 90-03/90-05's task envelopes.

**Sketched resolution:** Normalize on full row ids (`subjective/<slug>`) in every row's `verifier.args`, OR make `dispatch.sh`'s case statement accept both the bare slug and the full id (e.g. strip a leading `subjective/` before the case match). Either way, add a wiring smoke test that invokes each of the 4 rows' exact `verifier.script` + `verifier.args` and asserts the dispatcher recognizes the rubric (not just that the rubric name appears somewhere in the script). Home: P92 (or the next quality window that touches the dispatcher).

**STATUS:** OPEN

## 2026-07-04 | discovered-by: P91 91-03 (D91-09 token-scrub) | severity: LOW

**What:** `quality/gates/structure/banned-production-tokens.sh` catches only the suffixed phase-ID shape `\bP\d{2,3}-\d+\b` (e.g. `P79-02`, `P83-01`). The no-suffix `P\d{2,3}\+` shape (e.g. `P82+`, `P83+`) — used as a "lands in P82 and later" forward-reference — is NOT caught by any structure gate (framework research B(f); confirmed this pass: `P82+` in `sync.rs`/`reconciliation.rs` sailed past pre-push before P91-03 scrubbed it by grep + the deferral-pointer linter, never by the banned-token gate). Only the *deferral-pointer linter* (`lands? (alongside|in) P\d+`) covers a subset of these phrasings; a bare `action=FORK_AS_NEW (TODO P82+)` inside an eprintln string was covered by neither.

**Why out-of-scope for eager-resolution in P91-03:** extending the regex to `\bP\d{2,3}[-+]\d*\b` (or adding a `P\d{2,3}\+` alternative) is NOT cheap: existing production comments carry `P82+`/`P83+` forward-references across `reposix-remote/src/{main.rs:125, precheck.rs, bus_handler.rs}` (historical, some without `// banned-words: ok` markers). Turning the regex on now would newly BLOCK pre-push on legitimate historical comment lines in files outside P91-03's envelope — a cross-file marker sweep + regression risk, not a one-line change. Per D91-09 ("intake entry if not cheaply extendable").

**Sketched resolution:** In a structure-dimension window (P97 good-to-haves, or whenever `banned-production-tokens.sh` is next touched): (1) add the `P\d{2,3}\+` alternative to `PATTERN`, (2) sweep `crates/**/*.rs` for the now-caught historical hits and either reword them to drop the phase ID or add a per-line `// banned-words: ok` marker with rationale, (3) update the CLAUDE.md "Banned-token regex scope" section + the script header to document the widened shape. Grep target for the sweep: `grep -rnE 'P[0-9]{2,3}\+' crates --include='*.rs' | grep -v tests/`. Home: P97 (or the next structure-gate touch).

**STATUS:** OPEN

## 2026-07-04 21:00 | discovered-by: P91 91-05 (vision-litmus real-run) | severity: LOW

**What:** Two smaller findings from the real TokenWorld run. (a) `git-remote-reposix` emits a stray `git-remote-reposix: unknown command: feature` line after an egress-denied bus-push rejection — the helper doesn't gracefully consume git's post-rejection `feature` capability line; cosmetic but confusing next to the (good) teaching string. (b) `docs/reference/testing-targets.md` (D91-08) frames the durable fixtures as living in "the REPOSIX space ... same tenant as TokenWorld", implying two DISTINCT spaces — but both the `TokenWorld` and `REPOSIX` space keys resolve to the SAME Confluence space (id 360450, name "TokenWorld reposix demo space"). So the "never delete 7766017/7798785" constraint is load-bearing WITHIN the litmus's own mutation target, not a cross-space nicety — worth stating explicitly so a future cleanup sweep in "TokenWorld" understands it can hit the durable fixtures.

**Why out-of-scope for P91 91-05:** (a) is in `reposix-remote`; (b) is a `docs/reference/testing-targets.md` edit — both outside the litmus-only file envelope.

**Sketched resolution:** (a) have the helper consume/ignore the trailing `feature` line on the reject path (or map it to the same teaching context). (b) add one sentence to testing-targets.md: "The `TokenWorld` and `REPOSIX` space keys are two aliases for the SAME space (id 360450); the durable fixtures 7766017/7798785 live in it — any sweep of either key name must spare them." Home: P92/P97 docs+helper touch.

**STATUS:** OPEN

## 2026-07-04 22:15 | discovered-by: P91 litmus-REOPEN (repro setup + peer session) | severity: LOW

**What:** When `git-remote-reposix` is pointed at a dead origin (nothing listening on the host/port — e.g. a working tree whose `remote.origin.url` names a sim port with no live sim), a `git push`/`git fetch` HANGS indefinitely rather than failing fast with a teaching error. Observed live during the P91 mass-delete reproduction: a tree inited against `:7878` with the sim actually on `:7893` left the helper stuck for 2.5+ minutes (peer session confirmed via `ss -tlnp` that nothing listened on `:7878`). Likely cause: the helper's reqwest fetch/precheck path has no connect timeout, so a non-listening (or firewalled) origin blocks on TCP connect with no ceiling and no operator-readable diagnostic.

**Why out-of-scope for the litmus-REOPEN fix:** the fix envelope is the diff/fast_import no-commit guard; adding a connect timeout + teaching error touches the `reposix_core::http` client factory and every REST call site's error mapping — a separate robustness change.

**Sketched resolution:** set a bounded connect timeout (e.g. 10s) on `reposix_core::http::client()`, and map a connect failure to a teaching stderr line naming the unreachable origin + the likely fix ("is the sim/backend running at <host:port>? check `remote.origin.url`"). Add a test that points the helper at a closed port and asserts it exits non-zero within the timeout with the teaching string, rather than hanging. Home: a security/robustness or perf window that touches `reposix_core::http`.

**STATUS:** OPEN

## 2026-07-04 | discovered-by: P91 91-02 (deferred-items.md), reconciled during 91-06 docs wave | severity: MEDIUM

**What:** `quality/catalogs/agent-ux.json`'s two +2-reservation drain-verifier rows — `agent-ux/p87-surprises-absorption` (`last_verified: 2026-05-01T22:00:00Z`) and `agent-ux/p88-good-to-haves-drained` (`last_verified: 2026-05-01T22:30:00Z`) — neither carries `claim_vs_assertion_audit`, and reproducing `python3 quality/runners/run.py --cadence on-demand` right now (2026-07-04) shows both FAIL with the exact `_audit_field.py` schema message: `row agent-ux/p87-surprises-absorption missing claim_vs_assertion_audit (>=50 chars required for rows minted on/after 2026-05-08T00:00:00Z)` (and the p88 sibling identically). This is surprising because both rows' `last_verified` predates the 2026-05-08 cutoff and neither carries `minted_at` — the legacy-exemption heuristic in `validate_row` (`is_new = lv is None or parse_rfc3339(lv) >= cutoff`) reads, on its face, like it should treat these as pre-cutoff legacy rows exempt from the field requirement. It doesn't, live, today. Root cause NOT diagnosed here (would require stepping through `_audit_field.py`'s date-comparison path or checking for a second code path this session didn't find) — flagging the discrepancy between the code's apparent intent and its observed behavior is the finding; full root-cause is P96/P97 scope. Confirmed NOT a catalog-load-crashing SystemExit in practice: `run.py` completes its full sweep (6 PASS, 4 FAIL, 0 PARTIAL, 1 WAIVED, 1 NOT-VERIFIED, exit=1) rather than hard-aborting — so this is a per-row FAIL, not the "hard-blocks catalog load" framing in the original 91-02 deferred-items.md note (worth correcting: it degrades gracefully to FAIL, it does not crash the runner).

**Why out-of-scope for eager-resolution (91-06):** 91-06 is a docs-only wave (no cargo, no catalog-mutation authority beyond what a phase's own rows require); root-causing `_audit_field.py`'s cutoff logic plus writing genuine >=50-char audit paragraphs for two pre-P90 legacy rows is a framework-honesty fix, not a doc fix, and belongs with the other P90-era catalog-honesty carryovers this project already routes to P95-and-later windows.

**Sketched resolution:** (a) root-cause why `is_new` evaluates true for these two rows despite `last_verified` predating the cutoff and no `minted_at` present — likely candidates to check first: a stray whitespace/format difference in the stored RFC3339 string tripping `parse_rfc3339`, or a second validation call site not read during this pass; (b) once understood, either fix the legacy-exemption logic (if the bug is in `_audit_field.py`) or backfill genuine `claim_vs_assertion_audit` prose for both rows (if the two rows are legitimately being held to the new-regime bar) explaining how the P87/P88 drain-verifier assertions would fail loud if their claims were false. Either fix is small once root-caused; the root-cause step itself is the M-sized unknown.

**STATUS:** DEFERRED-P96/P97 | Filed per 91-06 Task 5(i). Home: the next framework-fixes or catalog-honesty window (P96 or P97, whichever lands the next `quality/runners/_audit_field.py`-touching phase). Not blocking today's pre-push/pre-commit (both rows are `on-demand` cadence only, per their catalog entries), but the schema drift is real and reproducible right now — don't let a future `--cadence on-demand` run's FAIL surprise the next reader who assumes these two milestone-bounded historical rows are inert.

## 2026-07-05 | TokenWorld two-writer conflict verifier does not exist — SC1 real-backend arm cannot be verified until built | discovered-by: P92 SC1 adjudication (D-P92-03) | severity: HIGH

**What:** The P97 9th probe (`pre-release-real-backend`) MUST exercise a real-backend two-writer CONFLICT (reject → pull --rebase → push), not just single-writer push, to close SC1's real-backend arm. P92 SC1 accepted this gap by design (coverage_kind: real-backend, verified at P97 only). Both P92 executors independently identified building this as a genuine new artifact unwise to rush late-session: requires git>=2.34 container + a Confluence conflict fixture on the TokenWorld space + cleanup harness.

**Why out-of-scope for P92:** SC1's designed split (sim arm GREEN now via T4 litmus; real-backend arm NOT-VERIFIED by design) matches ROADMAP coverage-kind semantics. The two-writer conflict verifier is a P97 deliverable, not a P92 scope miss — filed to chain visibility into the P97 9th probe.

**Sketched resolution:** Build a replica of the `agent-ux/t4-conflict-rebase-ancestry-real-backend` verifier that drives a REAL two-writer scenario against TokenWorld: (1) init + sync against the Confluence backend, (2) writer A edits record X, pushes (succeeds), (3) writer B performs the same edit with a stale base, pushes (correctly rejected: version mismatch), (4) writer B runs `git pull --rebase origin main` (FULL round-trip), (5) conflict resolves cleanly (ordinary textual conflict from 3-way merge proves blob fetch succeeded), (6) writer B's recovery push succeeds. Verifier script template: `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` (sim arm) extended with a Confluence fixture create/cleanup arm + real-backend URL substitution. The fixture must be durable-safe (per `docs/reference/testing-targets.md` cleanup conventions) or self-seeding (create/assert/delete within the harness like the updated `contract_confluence_live_hierarchy` test).

**Why deferred:** both P92 executors noted the new-artifact risk (schedule+complexity, not a high-confidence one-liner). The sim arm is proven GREEN; deferring the real-backend verifier to P97's final probe (which mandates it) is the OP-8 eager-resolution principle applied to sequencing: do not rush low-confidence, high-stakes shipping checks.

**STATUS:** OPEN — P97 Wave A reconciliation, 2026-07-05: CONFIRMED KEPT OPEN. The two-writer
CONFLICT-replay verifier still does not exist. The P97 milestone-close 9th probe
(`pre-release-real-backend`) will read **NOT-VERIFIED this run** — the autonomous window has
no TokenWorld creds, and OD-2's fail-closed rule is creds-missing ⇒ NOT-VERIFIED (never
skip-as-pass). Full conflict-replay coverage (reject → `git pull --rebase` → push against a
real Confluence fixture) is to be verified on a **live owner run or v0.14.0**; building it
late-session against a live tenant remains the new-artifact risk both P92 executors flagged.

## 2026-07-05 debt-drain triage

**Scope:** A tree-writer session running interleaved with the active P93 phase lane (no cargo, disjoint files from P93's `crates/`+`quality/` code) reviewed the intake backlog and recorded dispositions below. No items were silently skipped; each was either resolved, re-sequenced, or confirmed correctly routed.

- **Dependabot PR #55 + RUSTSEC** (2026-07-03 11:15 entry above): disposition updated in place on its own STATUS line — PARTIALLY RESOLVED / RE-SEQUENCED (PR #55 closed/superseded by #64/#65/#66; cargo-audit re-confirmation of the two RUSTSEC IDs deferred to a cargo-holding lane).
- **Catalog date-cutoff schema gap** (2026-07-03 21:35 / 89-07 entry, "OPEN" above): LEFT AS-IS. Already correctly routed to P95 RBF-D-06 per that entry's own Sketched resolution paragraph; no debt-drain action needed.
- **git 2.43.0 breaks single-backend push** (2026-07-05 / P92 T4 entry above): LEFT AS-IS. Already tagged for P94 (bus-push compatibility); no debt-drain action needed.
- **TokenWorld two-writer conflict verifier missing** (2026-07-05 / P92 SC1 entry above): LEFT AS-IS. Already routed to the P97 9th probe by design; no debt-drain action needed.

See the companion `GOOD-TO-HAVES.md` for the same-window disposition of the P92-filed security-gate good-to-haves (one resolved this window, one deferred, one left as-is) and the follow-on "branch hygiene + PR triage" entry appended below for owner-gated repo housekeeping.

## 2026-07-05 debt-drain: branch hygiene + PR triage (staged for owner)

**What:** A tree-writer session (same window as the triage above) ran a remote-branch + open-PR inventory as housekeeping. Findings, transcribed as verified:

- **Remote-branch inventory:** 9 branches total (NOT the ~32 previously assumed). Safe-to-delete list = EXACTLY ONE: `release-plz-2026-05-01T03-32-29Z` (its PR #32 is CLOSED/superseded by #61; 1 orphaned release-plz commit, 2 months stale). All others KEEP: `main`, `gh-pages`, the 4 dependabot branches backing OPEN PRs #62/#64/#65/#66, `release-plz-2026-07-04T05-03-11Z` (backs OPEN PR #61), and `workstream/v0.13.2` (holds 2 UNMERGED commits `cf79fd4` + `c4ed713` = P98-CONTEXT.md + P98-DISCUSSION-LOG.md, 285 lines, NOT on main — do NOT sweep; decide at P98 kickoff whether to cherry-pick or regenerate).
- **Deletion is OWNER-GATED** (external mutation per CLAUDE.md's "Uncommitted = didn't happen... External mutations need owner-named-target approval") — STAGED, not executed. Owner action: `git push origin --delete release-plz-2026-05-01T03-32-29Z` (only after confirming PR #61 supersedes #32's intent).
- **PR #62** (codecov-action 6→7): all 16 checks green, mergeable — merge proposal STAGED for owner: `gh pr merge 62 --squash --delete-branch`. Owner-gated, not executed.
- **Note:** the "quality gates (pre-pr)" FAILURE currently showing on PRs #64/#65/#66 is NOT a defect in those dependency bumps — it's the parallel P93 lane's own catalog-first rows (commit `543bfb4` added 3 `agent-ux/p93-*` catalog rows whose verifiers don't exist yet → NOT-VERIFIED → exit=1). Self-resolves once P93 ships its verifiers. PR #62 predates `543bfb4` so it reads green.
- **Note (cosmetic debt, filed below as a new GOOD-TO-HAVE):** `.git/hooks/pre-push` is a BROKEN symlink → nonexistent `scripts/hooks/pre-push` (confirmed: `ls` on the target errors ENOENT), but it's INERT because `core.hooksPath=.githooks` overrides it — the real active hook is `.githooks/pre-push`. Follow-up filed as a new low-sev entry in `GOOD-TO-HAVES.md` to delete the dead symlink for tidiness.
  **2026-07-07 corrected (shorthand `S-260707`):** this claim is STALE/WRONG — a
  pre-push hook demonstrably RAN on every push during the v0.13.1 session (the enforced
  quality-gate `55 PASS` output + a gitleaks secret-scan both fired from pre-push,
  observed live during v0.13.1 Wave F1b). The active `.githooks/pre-push` hook is NOT
  inert. Caught by v0.13.1 Wave C2-f (`RELIEF-HANDOVER-C2-wave-f.md` §4). Original text
  left in place per append-only convention — do not re-file this as a fresh
  dead-symlink surprise without re-verifying `git config core.hooksPath` and observing
  an actual push with the hook disabled/absent.

**Why staged, not executed:** branch deletion and PR merges are external mutations against the shared remote (`origin`) outside this window's file-editing charter — owner-named-target approval required per CLAUDE.md's dark-factory guardrails.

**STATUS:** OPEN (staged for owner action)

