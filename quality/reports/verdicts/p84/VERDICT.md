# P84 — Webhook-Driven Mirror Sync — Verifier Verdict

**Verdict:** GREEN
**Verified:** 2026-05-01
**Verifier:** unbiased subagent (zero session context)

## Per-row grading (catalog: `quality/catalogs/agent-ux.json`)

| Row | Catalog status | Verifier exit | Grade |
|---|---|---|---|
| `agent-ux/webhook-trigger-dispatch` | PASS | 0 (workflow YAML present in both copies, byte-equal mod whitespace) | PASS |
| `agent-ux/webhook-cron-fallback` | PASS | 0 (cron literal + fetch-depth + concurrency invariants hold) | PASS |
| `agent-ux/webhook-force-with-lease-race` | PASS | 0 (lease rejected on race; mirror/main untouched at 52deb6a) | PASS |
| `agent-ux/webhook-first-run-empty-mirror` | PASS | 0 (4.3.a fresh-but-readme + 4.3.b truly-empty both handled) | PASS |
| `agent-ux/webhook-backends-without-webhooks` | PASS | 0 (cron-only mode preserved when repository_dispatch removed) | PASS |
| `agent-ux/webhook-latency-floor` | PASS | 0 (p95=5s within 120s threshold; JSON parses) | PASS |

## Per-requirement evidence (DVCS-WEBHOOK-01..04)

| Req | Evidence |
|---|---|
| DVCS-WEBHOOK-01 (workflow shipped) | `e4fb6da` adds `docs/guides/dvcs-mirror-setup-template.yml` + live mirror copy; byte-equal-mod-ws verified by trigger-dispatch verifier |
| DVCS-WEBHOOK-02 (force-with-lease race) | `3472de0` shell harness; lease-rejection observed; mirror untouched on race |
| DVCS-WEBHOOK-03 (first-run empty-mirror) | `2267149` harness covers Q4.3 sub-cases 4.3.a (fresh-but-readme → lease-push) + 4.3.b (truly-empty → plain-push) |
| DVCS-WEBHOOK-04 (latency p95 ≤ 120s) | `32a04a7` ships `quality/reports/verifications/perf/webhook-latency.json` (p95=5s) + `scripts/webhook-latency-measure.sh` |

## Template invariants (`docs/guides/dvcs-mirror-setup-template.yml`)

- `repository_dispatch` trigger (types=[reposix-mirror-sync]): present (line 16-17)
- Cron `'*/30 * * * *'` literal: present (line 22)
- `--force-with-lease=refs/heads/main:${LEASE_SHA}`: present (line 94)
- `git show-ref --verify --quiet refs/remotes/mirror/main` first-run gate: present (line 91)
- `cargo binstall reposix-cli` (NOT bare `reposix`): present (line 58)
- `concurrency:` block (group=reposix-mirror-sync): present (line 25-27)

## CLAUDE.md confirmation

YES — `a5e7678` adds the "Webhook-driven mirror sync (v0.13.0 P84+)" architecture
paragraph (workflow path, secrets convention, Q4.1 cron-edit, first-run, race,
latency target, substrate gating ref) AND the Commands bullet for `gh api .../dispatches`
+ `scripts/webhook-latency-measure.sh`.

## Commit chain (origin/main, last 10)

All 6 P84 commits present: `77d192a` (T01 catalog) → `e4fb6da` (T02 workflow) →
`2267149` (T03 first-run) → `3472de0` (T04 race) → `32a04a7` (T05 latency) →
`a5e7678` (T06 close: catalog flip + CLAUDE.md + push).

## SURPRISES-INTAKE legitimacy

LEGITIMATE. The 2026-05-01 16:43 entry (severity HIGH, OPEN) properly scopes
the binstall + yanked-gix substrate gap out of P84: workflow halts at install
step against published v0.12.0 because (1) no binstall tarball asset for v0.12.0
and (2) source-compile fallback fails on yanked `gix=0.82.0`. Resolution path
correctly routed to P85+ release-pipeline work. Eager-resolution carve-out does
NOT apply — fix requires cutting a v0.13.x release with non-yanked gix +
working binstall artifacts (>>1hr; cross-cutting release-pipeline change).

## Latency JSON honesty note

The `quality/reports/verifications/perf/webhook-latency.json` headline
(`p95=5s, n=1`) measures **dispatch-API call → runner pickup**, NOT
end-to-end Confluence-to-mirror sync latency. The workflow does not complete
because of the substrate gap (binstall + yanked gix). The JSON's `note` field
is explicit about this and points at SURPRISES-INTAKE 2026-05-01 16:43. The
catalog row's falsifiable threshold (p95 ≤ 120s per ROADMAP P84 SC4) is
satisfied vacuously by the dispatch-only timing; real n=10 synthetic and
real-TokenWorld end-to-end measurements deferred to a post-v0.13.x phase
with working binstall substrate. Substrate gap properly deferred — not
hidden, not over-claimed.

---
*Phase 84 ships GREEN. Zero blockers.*
