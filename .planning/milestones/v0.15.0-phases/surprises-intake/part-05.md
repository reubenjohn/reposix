# v0.15.0 Surprises Intake — Part 5 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-17 09:35 | discovered-by: P117 W5 phase-close push executor (final-commit tree-writer) | severity: MEDIUM-HIGH

**What:** `code/ci-green-on-main` (the P0 guardrail asserting main's newest `ci.yml` run
concluded success post-push) can grade the WRONG run — a post-push race against GitHub's
Actions-list indexing. The probe queries the newest `ci.yml` run immediately after a push
and takes index-0 of a `--limit=1` list; that index-0 run can still be the PREVIOUS
commit's run, not HEAD's, if Actions has not yet indexed the just-triggered run. Live
during P117's final close: the probe graded PASS roughly 2 seconds post-push, while a
direct `gh run list` at the same moment showed the run for HEAD still `in_progress` — the
PASS was correct only because both the prior run and the (still-running, later-observed)
HEAD run were green; a red prior run or a failing HEAD run inside this same window would
have produced a FALSE PASS (a phase closing over a red main) or a FALSE FAIL, and the
guardrail would not have caught its own mis-grading either way.

**Why out-of-scope for the discovering session:** this session's charter is the single
final P117 planning-ledger append + push (no source/docs/catalog edits permitted);
hardening `quality/gates/code/ci-green-on-main.sh`'s run-selection logic is a quality-gate
script change to a P0 guardrail shared by every phase-close push — it needs its own
review/test pass, not a drive-by edit riding the last commit of a closing phase.

**Sketched resolution:** Make `ci-green-on-main.sh` assert the returned run's `headSha`
equals `git rev-parse HEAD` before accepting it as the graded run — not merely trust
index-0 of a `--limit=1` list. On a `headSha` mismatch, either poll by explicit run id
(re-list / re-fetch until a run for HEAD's sha appears) or return NOT-VERIFIED (exit 75)
and retry on a short backoff, closing the post-push indexing race window instead of
grading whatever run happens to sort first moments after push.

**STATUS:** OPEN

## 2026-07-17 10:15 | discovered-by: P118 close intake-filing (pre-existing drift surfaced, NOT a P118 defect) | severity: MEDIUM

**What:** `.planning/ROADMAP.md`'s Progress table contradicts the phase index above it for three
completed phases. The phase index (`ROADMAP.md:68-70`) marks **Phase 115 `- [x]`**, **Phase 116
`- [x]`** (completed 2026-07-16), and **Phase 117 `- [x]`** (completed 2026-07-17) as DONE — yet the
Progress table (`ROADMAP.md:293-295`) still shows `115. … 0/TBD | Not started`, `116. … 0/3 |
Planned`, and `117. … 0/7 | Planned`. Additionally, Phase 117's detail-block plan sub-bullets
(`ROADMAP.md:146-152`, 117-01…117-07) are ALL `- [ ]` (0 of 7 checked) despite the phase being
complete. A cold reader or agent consulting the SAME document gets contradictory completion signals
— the index says shipped, the Progress table + P117 detail-block checkboxes say not-started. This is
pre-existing bookkeeping drift (the Progress table was never advanced when P115/P116/P117 closed),
surfaced during P118's close; it is NOT a P118 defect.

**Why out-of-scope for the discovering session:** this session's charter is intake-filing ONLY —
file the drift, do NOT reconcile it. Reconciling the Progress-table rows + P117 detail-block
sub-bullets requires reconstructing each closed phase's true plan-complete counts (115's `TBD`
denominator, 116's 3, 117's 7) from that phase's SUMMARY/plan ledger — uncertain historical
bookkeeping that a drive-by edit riding an unrelated intake commit must not guess at. It is a
documentation-truth reconciliation, not a code fix, and belongs to a dedicated planning-doc pass.

**Sketched resolution:** in the Progress table set 115/116/117 to their real completed counts +
`Complete` + completion date (116 → `3/3 … Complete 2026-07-16`, 117 → `7/7 … Complete 2026-07-17`,
115 → its true count/date per its SUMMARY), matching the `- [x]` phase-index state; and flip P117's
detail-block plan sub-bullets (117-01…117-07) to `- [x]`. Verify each numerator/denominator against
that phase's SUMMARY — do NOT guess (especially 115's `TBD` denominator). NATURAL FIT for **Phase
119 "Docs/planning simplification" (the P112 RAISE)** — flag as a P119 candidate; a general
planning-doc-truth sweep is exactly that phase's remit. Do NOT attempt the reconciliation ahead of
that phase.

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P119 executor (docs/planning-simplification, SC-1/SC-2 audit) | severity: MEDIUM

**What:** ROADMAP.md's P119 **SC-1/SC-2** deletion criteria ("delete stale loose phase-dir artifacts
+ top-level catalog JSONs + handover transients") were written from the 5-day-stale **2026-07-12
reality-check audit**. The P119 planning audit (2026-07-17) found almost every named target is now
**LIVE or REFERENCED**: the `999.*` dirs are live backlog homes (ROADMAP §Backlog +
REQUIREMENTS.md:286-287), `MANAGER-HANDOVER.md`/`SESSION-HANDOVER.md` are live rotation state, and
BOTH top-level catalog JSONs still have live consumers/breadcrumbs (`v0.11.1-catalog.json` ← the
KEEP-AS-CANONICAL `scripts/catalog.py` orphan-scripts quality row; `docs_reproducible_catalog.json`
← 8 provenance breadcrumbs in release/docs-repro gate catalogs). So the ROADMAP's SC-1/SC-2 read as a
clean stale-purge that would in fact REGRESS live state — they are now misleading.

**Sketched resolution:** at milestone-close (OP-9 retrospective) or a ROADMAP refresh, reword P119
SC-1/SC-2 to the audited reality (clean-core simplification + a few grep-confirmed transient
deletions, NOT a big purge), or mark them SUPERSEDED with a pointer to the P119 close report. Do NOT
delete the live targets.

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P119 executor (deletion-candidate ref-eval) | severity: LOW

**What:** `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` (a DISABLED tag script from
the already-SHIPPED v0.13.0 milestone) lingers with **~9 references** across the tree (mostly
historical narrative in ROADMAP/handover/audit prose). P119 did NOT delete it — it was outside the
grep-confirmed-zero-ref set and its ~9 refs need a live-vs-narrative eval first.

**Sketched resolution:** a small ref-eval lane — classify each of the ~9 refs as live-consumer vs
historical-narrative; if all narrative → delete (git history is the archive); if any live → keep and
note why. Natural fit for a future docs/planning-hygiene pass.

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P119 executor (RAISE-2a/2b deletion gates) | severity: LOW

**What:** Two top-level catalog-JSON deletions gated by P119 were **DEFERRED** because each hit a
live reference the designed cascade does not cleanly cover:
- **`v0.11.1-catalog.json` + `scripts/catalog.py` subsystem (RAISE-2a):** `catalog.py` has no
  CI/justfile/hook INVOCATION (so "dead" by that measure) BUT is a LIVE **KEEP-AS-CANONICAL**
  orphan-scripts quality row (`quality/catalogs/orphan-scripts.json` + verifier
  `quality/reports/verifications/structure/orphan-scripts-catalog-py.json`) whose own
  `claim_vs_assertion_audit` says deleting `catalog.py` flips the row to FAIL. The P119 PLAN's
  "dead-subsystem-stale" premise missed this live row. Retiring the subsystem requires ALSO retiring
  that catalog row + verifier — a gate-sensitive change needing its own owner decision.
- **`docs_reproducible_catalog.json` (RAISE-2b):** 8 `source:` provenance breadcrumbs point INTO the
  seed — `quality/catalogs/release-assets.json` (×6, `...:install/<row-id>` form) +
  `quality/catalogs/docs-reproducible.json` (×2, line-number `...:303`/`:331` form) — with no clean
  1:1 successor location. Repointing/dropping 8 breadcrumbs across two release/docs-repro GATE
  catalogs for a 26 KB cleanup risks a catalog-integrity regression.

**Sketched resolution:** a dedicated catalog-hygiene lane that (a) for RAISE-2a decides whether to
retire the KEEP-AS-CANONICAL orphan-scripts row + `catalog.py` + `v0.11.1-catalog.json` +
`CATALOG-v3.md` together, or keep the subsystem; (b) for RAISE-2b drops or repoints the 8 breadcrumbs
in ONE commit and re-runs the full release + docs-repro cadence, then deletes the seed. Do NOT force
either ahead of that lane.

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P119 close (phase-close push cadence) | severity: MEDIUM (velocity/health regression)

**What:** The git `pre-push` hook took **103s vs the ~60s budget** (`quality/CLAUDE.md § Cadences`)
on the P119 close push; the hook itself WARNed "check for a new whole-repo gate before assuming diff
size is the cause." P119 added NO gate, so this is pre-existing drift, not a defect introduced by
this phase. Corroborates the earlier `2026-07-15 06:35` entry (manager amendment 4: ~91.7s / ~101s,
kcov shell-coverage the prime suspect) — a stable three-observation creep (101s -> 91.7s -> 103s).

**Sketched resolution:** P120 candidate — profile the pre-push cadence, identify what pushed it past
budget (kcov shell-coverage / full-workspace clippy / mkdocs are the suspects), and either optimize
the slow gate or re-baseline the documented ~60s budget in `quality/CLAUDE.md § Cadences`.
Consolidate with the `2026-07-15 06:35` entry so P120 treats this as ONE recurring regression.

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P119 phase-close tree-writer (executor-dispatch race NOTICED) | severity: LOW (no damage — orchestration-process candidate)

**What:** During the P119 phase-close, **TWO `gsd-executor` instances were dispatched for the same
STATE-advance + ROADMAP-finalize work** and raced the same files (`.planning/STATE.md`,
`.planning/ROADMAP.md`). Root cause: a premature "sub-executor completed" relay led the coordinator
to dispatch a follow-up writer while the original writer was **still running**. It was race-SAFE this
time ONLY because each executor grep-confirmed the target text before editing, detected the
already-applied edits, and no-op'd/reverted — so there was no duplicate commit and no double-increment
of any counter. That safety was a property of the executors' verify-before-edit discipline, NOT of the
dispatch, which had no dedup guard.

**Sketched resolution (dedup guard):** Before dispatching a writer that targets the same files as a
still-live or recently-dispatched writer, re-confirm the prior writer has **TERMINATED** via
ground-truth — agent status + `git log` showing its commit landed — not merely that it was
*relayed-as-done*. Never dispatch a second writer for work a running writer may still complete. This is
an **orchestration-process / doctrine candidate** for P120 (or a `.planning/ORCHESTRATION.md` update):
codify "one live tree-writer per file-set; confirm termination by artifact before re-dispatch."

**STATUS:** OPEN

## 2026-07-17 | discovered-by: P120 CLOSE Wave A (machine-wide build-mutex hazard, EXECUTED evidence) | severity: HIGH

**What:** `.claude/hooks/cargo-mutex.sh` gated concurrent builds with
`pgrep -f 'cargo (check|build|test|clippy|nextest|run)|rustc '` — `-f` matches the FULL
argv of ANY process. Any shell wait-loop, `pgrep`/`grep`/`rg`, or editor whose command
line merely CONTAINED `cargo build` or `rustc …/target/debug` FALSE-MATCHED, so the hook
exit-2'd and BLOCKED every subsequent cargo/rustc Bash **machine-wide** until that
unrelated process died. This is not hypothetical: it ACTUALLY deadlocked another live
session and burned ~180k tokens (executed evidence — no repro needed). The failure mode
is self-amplifying: a session's own `pgrep -f rustc` diagnostic (run to investigate the
block) is itself a false-match, so the block persists as long as anyone looks at it.

**Root cause:** `pgrep -f` matches command-line text, not the process identity; build-tool
tokens appear in the argv of many non-build processes.

**Fix (eager, this phase):** switched to `pgrep -x 'cargo|cargo-nextest|rustc|cross|clippy-driver'`
— exact whole-COMM match. A shell/editor/grep comm is `bash`/`zsh`/`nvim`/`pgrep`, which
`-x` structurally cannot false-match, so no argv mention can trip the gate; a genuine
second concurrent build always shows one of those build comms, so the OOM protection
(the whole point of the mutex) is preserved. Belt-and-suspenders: excludes the hook's own
`$$`/`$PPID`. Added a committed regression guard
`.claude/hooks/tests/cargo-mutex-no-false-match.sh` (spawns a decoy `bash` whose argv
carries the exact poison `rustc /home/x/target/debug/foo`, pipes a `cargo check` payload
into the hook, asserts exit 0 + allow-decision JSON) — proven PASS locally. A positive
"second build blocks" test is impractical to fake (comm can't be spoofed without a real
cargo/rustc, which would trip the mutex itself); that path is covered by the OOM doctrine
+ the unchanged exit-2 contract, cited in a code comment.

**STATUS:** RESOLVED (P120 CLOSE Wave A — see this commit; hook + regression test + this entry landed together)

## 2026-07-17 | discovered-by: P120 phase-close verifier (OD-3 adversarial credential-flow sweep) | severity: MEDIUM

**What:** `reposix doctor` echoes the RAW `remote.*.url` — including any embedded
`?mirror=https://user:token@…` bus credentials — into three `DoctorFinding` messages
(`crates/reposix-cli/src/doctor.rs:440`, `:444`, `:454`, and the recovery hint at `:456`).
`strip_bus_query(url)` strips the mirror half only for the `sot` that feeds
`parse_remote_url`; the echoed `format!("reposix remote={url}")` / `format!("remote url={url} …")`
still interpolates the FULL raw URL. This is the SAME leak class WR-02 (`sync.rs`) and WR-03
(`worktree_helpers.rs`) closed with `backend_dispatch::redact_userinfo`, but `doctor.rs`
was left untouched — its findings surface the user's own mirror token to stdout/stderr
(and thence any captured `reposix doctor` log, CI artifact, or screenshot).

**Why out-of-scope for the discovering session:** `doctor.rs` is a DELIBERATE exception to
P120's `teach_scan.py` scope (it emits a structured `DoctorReport`, not a `bail!`, so the
3-part teaching bar / scanner does not apply — documented in `crates/CLAUDE.md` §
"doctor.rs is exempt from the teach-scan by scope"). It is NOT in the enumerated retrofit
surface nor the WR-01/02/03 set the code reviewer found, so it falls outside P120's charter.
The teach-scan exemption is about the TEACHING bar, not the credential-redaction invariant —
so the leak was never in the scanner's field of view. Severity is MEDIUM not HIGH because the
credential is the user's OWN mirror token rendered in their OWN `reposix doctor` output
(lower exfil surface than WR-01's git-triggered append-only audit row or the git-triggered
helper stderr WR-02/WR-03 closed) — but it is a real raw-URL echo symmetric to the three
just fixed, and doctor output is routinely captured.

**Sketched resolution:** Wrap each `{url}` interpolation at `doctor.rs:440/444/454/456` in
`reposix_remote::backend_dispatch::redact_userinfo(url)` (NOT `strip_url_userinfo` — the
outer `reposix::` scheme defeats `Url::parse` and passes the token through, the exact
footgun documented in `crates/CLAUDE.md`). Add a regression test mirroring
`wr03_cache_path_error_redacts_mirror_credentials_in_malformed_bus_url` that runs `doctor`
on a tree with a credentialed `?mirror=` remote and asserts the token is absent from the
`DoctorReport`. ~30 min, no new dependency. Consider whether `doctor.rs` should be added to
the redaction-regression exemplar list in `crates/CLAUDE.md`.

**STATUS:** OPEN

## From P121 W1 (2026-07-17, registry authoring)

### P121-W1-01 — the `<code_allocation>` table UNDERCOUNTS the checker's flagged sites; W3/W4 must reconcile
- **Source:** NOTICED during P121 W1 while authoring the `reposix-core::codes` registry against the AUTHORITATIVE flagged-site list from `rpx_registry_check.py` (30 M2-converse sites), not just the plan's ~26-row `<code_allocation>` table. · **Severity: MED (W3/W4 blocker if not reconciled — the gate stays red until every flagged site carries a code or an exempt marker)** · STATUS: OPEN — tag planning-accuracy / P121-W3 / P121-W4.
- **What W1 AUTHORED (24 entries, all rustc-`--explain`-grade, committed `30c518da`):** RPX-0001 (spec-parse), 0101 (CLI missing-env), 0102 (helper missing-env), 0201 (cache-build), 0202 (no-synced-cache), 0203 (not-a-reposix-tree), 0301 (log needs --time-travel), 0302 (spaces confluence-only), 0303 (refresh --offline unimpl), 0305 (--since parse), 0306 (git-not-on-PATH), 0401 (refuse-existing-repo-root), 0402 (init fetch-failed), 0403 (attach not-git-tree), 0404 (attach dup-ids), 0405 (attach multi-SoT), 0501 (upload-pack), 0502 (EOF), 0503 (blob-limit), 0504 (push backend-unreachable), 0505 (push conflict/fetch-first), 0601 (malformed bus URL), 0602 (helper too-few-args), 0900 (explain-meta). Site→code mapping is in the entry doc-comments + the codes.rs module header.
- **RPX-0304 has NO home:** the plan's `gc --strategy unimplemented` site no longer exists — `gc.rs` `strategy_arg` is a working clap `value_enum` and the checker did NOT flag it. RPX-0304 was deliberately NOT minted (a registered-but-unemitted code is pointless; the north star is real error paths). W3 should drop RPX-0304 from scope or re-target it.
- **UNDERCOUNT — flagged sites with NO dedicated W1 code (W3/W4 must decide: map onto an existing code, or mint a new one):**
  - `init.rs:277` — target path not valid UTF-8 (git needs a UTF-8 path). Candidate: fold into RPX-0306 (git-related) or mint RPX-0307.
  - `init.rs:542 / :562 / :604 / :631` — the four `--since` rewind scenarios (no-cache-for-since / no-tag-before-timestamp / since-fetch-failed / since-update-ref-failed). NONE has a plan code. Candidate: a new RPX-03xx `--since` family, or fold the fetch/ref ones into RPX-0402/0201.
  - `sync.rs:123` — the tree's `remote.*.url` could not be parsed (a bound-tree URL parse, distinct from RPX-0001's spec parse). Candidate: RPX-0001 vs a new code.
  - `stateless_connect.rs` `UNFILTERED_FETCH_HINT` (const ~L61) — an unfiltered-fetch-can't-be-served hint, distinct from the RPX-0503 blob-LIMIT path. Candidate: fold into RPX-0503 or mint RPX-0506.
  - `list.rs:276/:340` + `refresh.rs:294/:319` — confluence/jira missing-env variants; these map cleanly onto RPX-0101 (CLI missing-env) — W3 just wires `.code(ids::MISSING_ENV_CLI)`.
  - `bus_handler.rs:461` — bad-mirror arm; maps onto RPX-0601 (malformed bus URL) per the plan.
- **Fix-sketch:** W3 (CLI) + W4 (helper) each, per site, either `.code(ids::…)` an existing entry or append a new `ExplainEntry` to `codes.rs` for the genuinely-new scenarios above; the `agent-ux/rpx-codes-registry` gate stays red until every flagged site is coded or `// rpx-code-exempt: ok — <reason>`-marked. Effort: small per site (the render + registry machinery is done), but the reconciliation DECISIONS (merge vs mint) are the real content and belong to the wave that owns each file.

**STATUS:** OPEN

