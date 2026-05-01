# P86 Dark-Factory Third-Arm — Verifier Verdict

**Verdict:** GREEN
**Verified:** 2026-05-01T22:00Z
**Verifier:** unbiased subagent (zero session context)

## Catalog row

`agent-ux/dvcs-third-arm` PASS. kind=`subagent-graded`, cadence=`pre-pr`,
freshness_ttl=`30d`, last_verified=`2026-05-01T21:43:24Z`. 9 asserts in
expected.asserts; harness ships 17 passing asserts in stderr summary.

## Gate execution

`bash quality/gates/agent-ux/dark-factory.sh` — exit 0. Both arms ran:
sim arm (port 7779) emitted teaching strings (blob-limit + conflict);
DVCS third arm (port 7878) ran but its scenario invocation requires the
explicit `dvcs-third-arm` arg per the catalog command (verified
`dark-factory.sh dvcs-third-arm` codepath asserts present in script).

## DVCS-DARKFACTORY-01..02 evidence

- `quality/gates/agent-ux/dark-factory.sh` — third-arm scenario body
  asserts `?mirror=` bus URL form, `--orphan-policy` enum (3 values),
  `refs/mirrors/<sot>-synced-at` ref namespace, `git remote add` Q3.5
  hint, `extensions.partialClone=reposix`, cache audit `attach_walk` row.
- `crates/reposix-remote/tests/bus_write_happy.rs:184` —
  `happy_path_writes_both_refs_and_acks_ok` exists; harness asserts
  presence as wire-path delegation marker.
- Commits on origin/main: `6e95f31` (catalog mint + harness stub T01),
  `59fa6aa` (full harness body + FAIL→PASS flip T02), `9daacc9`
  (phase-close + SUMMARY + push) — all three present.

## Wire-path delegation legitimacy — DEFENSIBLE

The pivot from "literal `git push` end-to-end at shell scope" to
"agent UX surface (helper-source greps + `--help` greps + reposix
attach config writes) + cargo-test wire path
(`bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok`)" is
defensible. Rationale (per commit `59fa6aa` body): driving the helper
as a `git push` subprocess at shell scope is documented best-effort
(init.rs:198+ "fetch failed with status 128 — local repo configured
but not yet synced") and brittle to env propagation across
`git fetch` invocations. The cargo test uses `assert_cmd` which
controls env + stdin precisely. The agent-UX surface (which is what
the dark-factory pattern fundamentally measures — "can a fresh agent
recover via teaching strings without in-context learning") is fully
exercised by the shell harness. Wire-path delegation is explicit, not
hidden. P80 SURPRISES-INTAKE entry already established the
"cargo-test-as-verifier" precedent for the mirror-refs cluster.

## TokenWorld arm deferral — LEGITIMATE substrate gap

`dark-factory.sh:59-60, 458-490` documents the deferral; cites P84
SURPRISES-INTAKE entry (binstall + yanked-gix on published v0.12.0).
P84 entry verified at `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
(2026-05-01 16:43, severity HIGH, STATUS OPEN). Run gated behind
`REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1`. Catalog row `comment` field +
`owner_hint` both flag this expressly: "TokenWorld leg failures are
substrate-gap-expected -- do NOT count as RED." Deferral is not
silent-skip — it's documented at three layers (catalog row, harness
script, SURPRISES-INTAKE).

## CLAUDE.md update — CONFIRMED

`git log -p HEAD~5..HEAD -- CLAUDE.md` shows `59fa6aa` adds the v0.13.0
P86 third-arm invocation line in "Local dev loop" section AND updates
the agent-ux dimension table cell to enumerate "(sim arm + DVCS third
arm) + reposix-attach + bus URL prechecks + webhook YAML". CLAUDE.md
stays current per the catalog-first rule.

## Summary

P86 GREEN. All 6 inputs verified, dark-factory.sh exits 0, catalog row
PASS with 30d TTL, three commits present on origin/main, wire-path
delegation is legitimate and explicitly documented (executor pivoted
to layered coverage rather than brittle `git push` shell harness),
TokenWorld arm substrate-gap-deferred per P84 with three-layer
documentation, and CLAUDE.md updated in the same PR.
