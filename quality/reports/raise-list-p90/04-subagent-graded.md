<!-- Shard of quality/reports/raise-list-p90.md (P90 RAISE LIST). Index has the other sections. -->
> [Back to RAISE LIST — P90 (index)](../raise-list-p90.md)

## 4. Subagent-graded migration record                    <!-- 90-05 fills from 90-03 -->

Per D90-10, R2 § B confirmed exactly 5 `kind: subagent-graded` rows workspace-
wide; two needed action, landed by 90-03 (commits `fb01c2f`, `520d222`):

| Row | Before | After |
|---|---|---|
| `agent-ux/dvcs-third-arm` | `kind: subagent-graded` (but the verifier — `dark-factory/dvcs-third-arm.sh` — is pure deterministic shell: source-string greps, `--help` greps, real `reposix attach` subprocess vs sim, git-config asserts, sqlite3 audit-count query, and a test-fn-existence grep for the wire path; zero `claude`/Task/rubric calls anywhere in its call chain; header self-discloses "No real LLM in CI") | **`kind: mechanical`** — matches D90-10's "deterministic assert script with no grading step → mechanical" rule (closes p86 F7). Provenance note appended to the row citing this migration. Header over-claim in `dvcs-third-arm.sh` ("bus-push end-to-end" vs its own `:20-23` disclosure that push is NOT exercised) eager-fixed same commit, comment-only. |
| `subjective/dvcs-cold-reader` | `kind: subagent-graded`, but `dispatch.sh`'s `--rubric` case statement (`:58-69`) had no entry for `dvcs-cold-reader` — it fell through to the Path-B **stub** (`:70-76`), which raises `KeyError` on `catalog.find_row` for an id it doesn't recognize. This was the real decorative-kind instance (intent WAS real subjective grading; the wiring, not the kind, was missing). | **Wired for real.** `.claude/skills/reposix-quality-review/dispatch.sh` gained a `"subjective/dvcs-cold-reader"` case invoking new `lib/dispatch_dvcs_cold_reader.sh`, which `exec`s `claude /doc-clarity-review` against the 3 DVCS docs (`docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, `docs/guides/troubleshooting.md`) via an inline rubric heredoc, parses `Rate:`/`Rationale:`, and persists to `quality/reports/verifications/subjective/dvcs-cold-reader.json`. `kind` unchanged (correctly still `subagent-graded` — the fix was wiring, not reclassification). Catalog row content itself was NOT edited (no lie to fix there). |

The remaining 3 `subjective-rubrics.json` rows (`cold-reader-hero-clarity`,
`install-positioning`, `headline-numbers-sanity`) were already correctly
dispatch-wired (`dispatch.sh:59-68`) — no migration needed; their waiver-cliff
disposition is handled in § 5 below (NOT mooted by this migration, per D90-02b
— the waiver covers a different, still-unresolved gap: the runner
subprocess's dispatch-and-preserve invariant).

**Noticed during 90-03 (carried forward here, not yet resolved):** all 4
subjective rows' `verifier.args` pass the BARE rubric slug (e.g.
`["--rubric", "cold-reader-hero-clarity"]`), but `dispatch.sh`'s case-statement
keys use the FULL `subjective/<slug>` id. If the runner subprocess invokes
`bash dispatch.sh --rubric cold-reader-hero-clarity` verbatim (matching the
row's own `verifier.args`), the case statement never matches and all 4 rubrics
fall through to the Path-B stub, which then raises `KeyError` on the bare
slug. This is a real, pre-existing, framework-wide wiring gap — **not fixed in
P90** (M-sized: touches `catalog.py`, `dispatch.sh`'s CLI parsing, and/or all
4 rows' `verifier.args`; not a <1h fix safely landed inside a framework
dispatch). Flagged as an intake candidate for 90-07 to route (see this
report's closing TLDR).

