<!-- Shard of quality/reports/raise-list-p90.md (P90 RAISE LIST). Index has the other sections. -->
> [Back to RAISE LIST — P90 (index)](../raise-list-p90.md)

## 1. Transport/perf rows — coverage_kind dispositions   <!-- 90-05 fills from R2 § A -->

Walked all 12 non-doc-alignment catalogs (`doc-alignment.json` is a distinct
schema, exempt per `quality/catalogs/README.md`). Coverage legend:
**real-backend** (non-loopback REST) / **sim** (in-process simulator,
127.0.0.1:78xx) / **localhost** (wiremock + `file://` mirror + `git init
--bare`) / **static-grep** (source/string asserts only, nothing executed) /
**vacuous** (assertion cannot fail meaningfully).

Most bus/mirror/precheck `agent-ux` rows are **honest localhost** — they
genuinely drive `git-remote-reposix` via stdin against wiremock + `file://`,
and their descriptions already say so (`coverage_kind: sim` disposition, no
text change needed): `reposix-attach-against-vanilla-clone`,
`mirror-refs-write-on-success`, `mirror-refs-readable-by-vanilla-fetch`,
`mirror-refs-cited-in-reject-hint`, `sync-reconcile-subcommand`,
`bus-precheck-a-mirror-drift-emits-fetch-first`,
`bus-precheck-b-sot-drift-emits-fetch-first`, `bus-fetch-not-advertised`,
`bus-write-sot-first-success`, `bus-write-mirror-fail-returns-ok`,
`bus-write-no-mirror-remote-still-fails`,
`bus-write-fault-injection-{mirror-fail,sot-mid-stream,post-precheck-409}`,
`bus-write-audit-completeness`, `webhook-force-with-lease-race`,
`webhook-first-run-empty-mirror`, `perf/handle-export-list-call-count`
(genuinely drives the helper via wiremock, N=200).

### Genuine overclaims (RAISE — description-soften or WAIVE)

| id | what the verifier ACTUALLY exercises | disposition |
|---|---|---|
| `agent-ux/dark-factory-sim` | **sim + static-grep.** `dark-factory/sim.sh:31-48` spawns sim + `reposix init` + git-config asserts; the "teaching strings **emit** on conflict/blob-limit paths" claim is verified by grepping *source* (`sim.sh:53,58`), not by observing a real emission. **Never pushes** (QL-001 FINDING-B). | **description-soften**: "emit" → "present in helper source"; tag `coverage_kind: sim`. |
| `agent-ux/bus-write-no-helper-retry` | **static-grep.** `bus-write-no-helper-retry.sh:31-54` greps `bus_handler.rs` for retry constructs (`for _ in 0..`, `loop {`, `sleep`, `--force`). The runtime "single push_mirror invocation" claim is NOT executed here — that's covered by the sibling wiremock row `bus-write-mirror-fail-returns-ok`. | **description-soften**: reframe as source-hygiene check; note the runtime claim lives in the sibling row. Behavioral replacement is P92 RBF-B-07 (already on ROADMAP). |
| `agent-ux/webhook-latency-floor` | **VACUOUS.** Reads the *committed* artifact `quality/reports/verifications/perf/webhook-latency.json` (n:1, method:"synthetic-dispatch", p95_seconds:5) and asserts `<=120`. The artifact's own `note` admits it measures dispatch→runner-pickup only. | **WAIVED + until_date** (already waived; description should say "dispatch-pickup latency, n=1, synthetic" not "webhook-driven sync verified"). Real n=10 measurement gated on working `cargo binstall` (now unblocked per § 5 below) + `scripts/webhook-latency-measure.sh`. |
| `agent-ux/webhook-trigger-dispatch` | **static-grep + one real `gh api` GET.** `:35` fetches live YAML, `:42` `diff -w` byte-equality, `:47-66` structural greps. **Never triggers a dispatch, never measures a sync.** | **description-soften**: "reference workflow present + byte-equal to mirror" not "webhook-driven sync verified". |
| `benchmark-claim/8ms-cached-read`, `benchmark-claim/89.1-percent-token-reduction` | **VACUOUS.** `verifier.script: None`; asserts frontmatter freshness + headline greppable. No measurement in-row. | **description-soften / WAIVE** — intentionally NOT softened per D-CONV-2 (2026-07-04): these are the reason the weekly verdict stays yellow-not-green by design; mechanizing them is GOOD-TO-HAVES-04, routed to launch-readiness. |
| `security/allowlist-enforcement`, `perf/headline-numbers-cross-check` | **Dangling verifier at time of R2.** `allowlist-enforcement.sh` confirmed absent; `headline-numbers-cross-check.py` confirmed absent. | See § 5 — `allowlist-enforcement` FIXED this dispatch (script now lands); `headline-numbers-cross-check` still dangling, flagged for P97. |

### Zero green real-backend transport coverage today

**There is zero green real-backend transport coverage in this catalog as of
2026-07-04.** The only true end-to-end `git push` in the entire framework is
`agent-ux/real-git-push-e2e` — and it is sim-only + WAIVED (QL-001, routed
P91, D90-01). The real-backend litmus row
(`agent-ux/milestone-close-vision-litmus-real-backend`) is an honest,
by-design exit-75 placeholder — correctly NOT-VERIFIED, not a hidden gap, but
still zero coverage. This is the quantified justification for P91 (attach/sync
real-backend wiring) and P92 (push-flow correctness against real backends +
OP-3 audit completeness) — both already on ROADMAP citing this exact gap.

### Legacy coverage_kind enforcement (D90-05)

90-02's `coverage_kind` validator hard-enforces on rows carrying `minted_at`
(new-regime rows, P90-onward). The ~92 legacy P78–P88 rows above carry no
`minted_at` and are **RAISE-only** here — migrating them to explicit
`coverage_kind` + honest descriptions is **P95 RBF-D-06's** chartered work
(`quality/reports/raise-list-p90.md` is P95's own stated input per ROADMAP.md
line 245/251). P90 does not hard-block them; doing so would turn pre-push RED
on nearly every catalog before the migration phase exists — the exact
deferral-loop the framework fixes are supposed to prevent (D90-05 rationale).

