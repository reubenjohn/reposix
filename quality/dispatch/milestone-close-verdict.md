# Milestone-Close Verdict — TEMPLATE

> **Origin:** P89 RBF-FW-03 (2026-05-08). This is the verdict TEMPLATE; the
> per-milestone verdict ARTIFACT lives at `quality/reports/verdicts/milestone-<version>/VERDICT.md`.
> P97 RBF-G-04 will use this template to overwrite the existing v0.13.0 verdict.
>
> **CLI note (89-06 fix):** `quality/runners/run.py` accepts ONLY `--cadence`
> (no `--dimension`, `--row`, `--dry-run`, `--check-waivers`). Every probe
> command below was verified to actually exist/parse as of 2026-07-03 — do
> not add flags that "sound right"; check `run.py --help` first.

**Milestone:** vX.Y.Z
**Date:** YYYY-MM-DD
**Orchestrator:** <session-id or owner>
**Verifier subagent:** <author distinct from orchestrator per F-K5>

## Probe results

| # | Probe | Source / verifier | Status | Evidence (artifact + transcript) |
|---|---|---|---|---|
| 1 | Catalog rows GREEN-or-WAIVED for milestone scope | `python3 quality/runners/run.py --cadence pre-pr` | ⬜ | `quality/reports/...` |
| 2 | Dark-factory simulator arm GREEN | `bash quality/gates/agent-ux/dark-factory.sh sim` | ⬜ | |
| 3 | Dark-factory DVCS third arm GREEN (vanilla-clone + attach + bus push) | `bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm` | ⬜ | |
| 4 | All shipped REQ-IDs traceable to a catalog row + a verifier-graded artifact | per-milestone REQUIREMENTS.md walk | ⬜ | |
| 5 | RETROSPECTIVE.md milestone section distilled (OP-9) | `grep -F '## v<version>' .planning/RETROSPECTIVE.md` | ⬜ | |
| 6 | Tag-script clean-tree + signed-tag guards passing | `bash .planning/milestones/v<version>-phases/tag-v<version>.sh`[^6] | ⬜ | |
| 7 | No expired waivers without follow-up | inspect probe 1's FAIL rows + cross-reference `waiver.until` in `quality/catalogs/*.json`[^7] | ⬜ | |
| 8 | CLAUDE.md milestone-shipped subsection landed | `grep -F 'v<version> shipped' CLAUDE.md` | ⬜ | |
| 9 | **Vision litmus test against real backend** (RBF-FW-03 SLOT) | `python3 quality/runners/run.py --cadence pre-release-real-backend` | ⬜ | `quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json` + transcript |

[^6]: The tag script currently has **no `--dry-run` / non-mutating mode** —
    invoking it for real creates a local git tag (guard 1-6 must all pass
    first, or it exits non-zero before tagging). Read this probe as "run the
    guard sequence as part of the actual tag-cut" rather than a side-effect-free
    preview. `tag-v0.13.0.sh.disabled` is disabled pending P97 RBF-G-04
    re-enable; a dry-run flag is a candidate P97 improvement, not P89 scope.

[^7]: `run.py` has no `--check-waivers` flag (that command is fictional —
    do not use it). The real mechanism (`quality/PROTOCOL.md` § "Waiver
    protocol"): **expired waivers auto-flip their row to FAIL on the very
    next verify run**, so probe 7 is satisfied by re-reading probe 1-3's
    output for any FAIL row whose catalog entry carries a `waiver` block —
    cross-reference `waiver.until` against the current date. Zero such rows
    = probe passes. No dedicated waiver-audit command exists; this is
    read-the-existing-output, not a new invocation.

> **Probe 9 is non-skippable** (CLAUDE.md "Subagent delegation rules" — added P89 RBF-FW-03).
> Any milestone-close ritual that does not include `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades the milestone RED.
> The probe runs the vision litmus test against the sanctioned real backend (TokenWorld for v0.13.0); the row's `blast_radius: P0` defends against C7 self-licensing-deferral-loop.
>
> **OD-2 hard-RED (89-OWNER-DECISIONS.md, binding):** probe 9 has **no waiver
> column**. If the `pre-release-real-backend` cadence cannot EXECUTE against
> the sanctioned target at milestone-close (creds/target missing or
> unreachable), the verdict is RED — no owner-waiver, no `until_date`, no
> PASS-with-comment, no skip-counts-as-pass. This is distinct from the
> legitimate `NOT-VERIFIED` SLOT state during P89–P95 (substrate not yet
> built) — see `quality/PROTOCOL.md` § "Verifier exit-code conventions" and
> the OD-2 block under Step 6 for the full distinction.

### DRAIN-02 second-run mirror-drift regression (P125)

The P125 litmus self-heal (`quality/gates/agent-ux/lib/litmus-self-heal.sh`, wired into
`lib/litmus-flow.sh`) folds backend-drift and mirror-drift self-healing into probe 9. The
verifier MUST execute these three checks against the sanctioned real backend (TokenWorld +
the GitHub mirror): there is no scripted way to induce a trashed-fixture state or to fully
automate a "run twice" real-backend gate, so this is a documented MANUAL verification, not
new CI wiring. Wording is kept consistent with `125-VALIDATION.md`'s Manual-Only table.

1. **Run-twice proof (DRAIN-02).** Run `python3 quality/runners/run.py --cadence
   pre-release-real-backend` TWICE in immediate succession (creds + a non-default
   `REPOSIX_ALLOWED_ORIGINS` present). BOTH runs must exit 0. The SECOND run is the actual
   DRAIN-02 regression proof: the first run's own bus push re-stales the GitHub mirror, and
   the mirror pre-reconcile self-heal (`_litmus_mirror_reconcile` — a BUS-remote
   `git fetch reposix main` → `git checkout FETCH_HEAD -- pages/` overlay of the LOCAL tree
   before the marker edit, NOT `reposix sync --reconcile`, which converges only the local
   cache and leaves the external mirror head byte-identical) must prevent the second run
   from false-negativing on a stale-base rebase conflict. The external mirror head is
   refreshed by the run's own SoT-changing marker PUSH via the bus fan-out, not by this
   local overlay.

2. **Backend-drift proof (manual, non-destructive; DRAIN-12).** Confirm current fixture
   state with `python3 scripts/confluence_tokenworld.py list`; the fixture pre-flight
   (`_litmus_fixture_preflight`) restores/reparents the three known ids (2818063 / 7766017 /
   7798785) idempotently. Do NOT destructively trash the protected pair (7766017 / 7798785)
   to force the condition — the tooling refuses to delete them by design. If a genuine drift
   is observed in the wild, re-run the litmus and confirm it self-heals before the marker
   push without a manual `restore` intervention.

3. **Cold-read of the teaching string (SC3 cross-check).** Confirm the corrected mirror-lag
   helper hint (Plan 125-01) points a stale-mirror operator at `reposix sync --reconcile`
   for the cache + a remote-explicit `git pull --rebase <bus-remote> main` for the tree,
   rather than a bare `git pull --rebase` that conflicts on divergent (server-normalized)
   body content on a Pattern-C `attach` tree.

## Verifier subagent grading

<verbatim verifier subagent prompt — see quality/PROTOCOL.md § "Verifier subagent prompt template">

## Final verdict

- [ ] All 9 probes ✅
- [ ] No catalog row at NOT-VERIFIED with `blast_radius: P0`
- [ ] No waiver expired without follow-up

**Status:** ⬜ GREEN | ⬜ RED
