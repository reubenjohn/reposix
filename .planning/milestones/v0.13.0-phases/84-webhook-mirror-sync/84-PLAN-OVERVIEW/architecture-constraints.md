# Architecture S1–S2 + Hard constraints

← [back to index](./index.md)

## Subtle architectural points (read before T02)

The two below are flagged because they are the most likely sources
of T02 review friction. Executor must internalize them before
authoring the YAML.

### S1 — `gh repo create --add-readme` produces sub-case 4.3.a, NOT 4.3.b

CARRY-FORWARD § DVCS-MIRROR-REPO-01 line 135–137 says the mirror is
"empty except auto-generated README from `gh repo create
--add-readme`." This means the mirror's `main` ref **exists** at the
README commit's SHA. RESEARCH.md A1 confirms this assumption. So
sub-case 4.3.a (`fresh-but-readme`) is the **actual** first-run state
of `reubenjohn/reposix-tokenworld-mirror` as of P84 launch.

**Why this matters for T02 + T03.** T02's YAML must handle BOTH
sub-cases (4.3.a + 4.3.b) regardless of which one is actually the
first-run state, because (a) future mirrors created by other owners
might be 4.3.b (no `--add-readme`), and (b) the catalog row asserts
the workflow handles BOTH gracefully (DVCS-WEBHOOK-03 verbatim text
mentions "no `refs/heads/main`, no `refs/mirrors/...`" — the
truly-empty case). T03's harness exercises BOTH sub-cases.

T02's YAML branch (`if git show-ref --verify --quiet ...`) handles
both sub-cases without distinguishing them at the YAML layer —
4.3.a goes through the lease-push branch, 4.3.b goes through the
plain-push branch. The branching predicate is the SAME for the live
workflow as for T03's test harness.

**First-run impact on `reposix-tokenworld-mirror`:** the first
real workflow run (after T02 ships the YAML) REPLACES the auto-
generated README on the mirror with the SoT's content (TokenWorld
pages exported as markdown). RESEARCH.md Pitfall 5 names this:
documented in P85 as expected behavior ("the mirror repo's README is
replaced on first sync").

### S2 — `client_payload` from `repository_dispatch` is UNTRUSTED — workflow IGNORES it

The `repository_dispatch` event carries an arbitrary `client_payload`
JSON object set by the dispatching party (Confluence's webhook
config, or `gh api ... -f client_payload=...`). RESEARCH.md § "Known
Threat Patterns" flags this as untrusted input.

**Why this matters for T02.** The workflow YAML's `run:` blocks must
NEVER interpolate `${{ github.event.client_payload.* }}` into shell
commands. All values consumed by the workflow are derived from
`secrets.*` and `vars.*` (which are repo-owner-controlled). The
verbatim YAML in RESEARCH.md § "Workflow YAML Shape" follows this
discipline — there's no `${{ github.event.client_payload.* }}`
reference anywhere.

**Verifier check (T02):** `webhook-trigger-dispatch.sh` greps the
YAML for `github.event.client_payload` and asserts ZERO matches.
Defense-in-depth: even if the YAML grows in the future, a regression
that reads payload into a `run:` block fires the verifier red.

## Hard constraints (carried into the plan body)

Per the orchestrator's instructions for P84 and CLAUDE.md operating
principles:

1. **Catalog-first (QG-06).** T01 mints SIX rows + SIX verifier shells
   BEFORE T02–T06 implementation. Initial status `FAIL`. Rows are
   hand-edited per documented gap (NOT Principle A) — annotated in
   commit message referencing GOOD-TO-HAVES-01. The agent-ux
   dimension's `bind` verb is not yet implemented; rows ship as
   hand-edits matching P79/P80/P81/P82/P83 precedent.
2. **No cargo invocations.** P84 has zero new Rust integration tests
   and no compilation. Local verifiers are shell scripts and
   `gh api`/`git`/`jq` invocations only. CLAUDE.md "Build memory
   budget" is trivially satisfied; no parallel-cargo coordination
   needed.
3. **Sequential task execution.** Tasks T01 → T02 → T03 → T04 → T05
   → T06 — never parallel. T02's YAML must exist before T03/T04/T05
   verifiers can target it; T06's catalog flip + push must come last.
4. **Workflow file lands in mirror repo, NOT canonical repo (D-08 +
   CARRY-FORWARD § DVCS-MIRROR-REPO-01).** T02 ships BOTH a template
   copy in `docs/guides/dvcs-mirror-setup-template.yml` (canonical)
   AND a live copy in `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
   (mirror). The verifier checks both exist + are byte-equal modulo
   whitespace.
5. **`cargo binstall reposix-cli` (NOT `reposix`) per D-05.** Verifier
   greps the YAML for the correct crate name; wrong name fails the
   row.
6. **Literal cron `'*/30 * * * *'` per D-06.** NEVER `${{ vars.* }}`
   in the schedule field. Verifier greps for the literal string.
7. **First-run branch in the push step uses `git show-ref --verify
   --quiet refs/remotes/mirror/main` per D-07.** Handles both
   sub-cases 4.3.a + 4.3.b without distinguishing them at the YAML
   layer.
8. **Concurrency block YES, `cancel-in-progress: false` per D-01.**
   Defends against duplicate runs near a cron-vs-dispatch boundary;
   queues rather than cancels.
9. **Latency artifact at `quality/reports/verifications/perf/webhook-latency.json`
   per D-02 + ROADMAP SC4.** T05 ships the synthetic-method JSON
   with `verdict: "PASS"` if `p95_seconds ≤ 120` (falsifiable
   threshold). The catalog row's cadence is `pre-release`.
10. **Workflow YAML's `client_payload` MUST NOT be interpolated into
    `run:` blocks per S2.** Verifier greps for `github.event.client_payload`
    and asserts zero matches.
11. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
    codified 2026-04-30).** T06 ends with `git push origin main`
    against the canonical repo; pre-push gate must pass; verifier
    subagent grades the six catalog rows AFTER push lands. Verifier
    dispatch is an orchestrator-level action AFTER this plan
    completes — NOT a plan task. The mirror-repo push (T02 step 2)
    is SEPARATE and follows its own pre-push (no pre-push hook on
    the mirror repo since it has no source code).
12. **CLAUDE.md update in same PR (QG-07).** T06 documents the
    workflow path (§ Architecture) + secrets convention + the
    `gh api ... dispatches` invocation form (§ Commands or new
    "Mirror sync" sub-section).
13. **Six-row catalog set in `agent-ux.json` (D-04).** NOT a new
    `webhook-sync.json`. Rows joining the existing
    `agent-ux/dark-factory-sim`, `agent-ux/reposix-attach-*`,
    `agent-ux/mirror-refs-*`, `agent-ux/sync-reconcile-*`,
    `agent-ux/bus-*` family.
14. **`--` separator + `-` rejection NOT in scope here.** P82's
    `git ls-remote` shell-out used these as Tampering mitigations
    for argument-injection via `mirror_url`. P84's workflow runs in
    a GH Actions context with `mirror_url` derived from
    `${{ github.server_url }}/${{ github.repository }}.git` — server-
    side fixed values, no user-input attack surface. The mitigation
    is unnecessary at the YAML layer. (P82's PRECHECK A still uses
    them; P84's workflow is a different threat model.)

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces ONE new
trifecta surface (the workflow's `repository_dispatch` event-payload
boundary) and inherits two existing surfaces.

| Existing/new surface                      | What P84 changes                                                                                                                                                                                                                                                                                |
|-------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP (cache → confluence) | UNCHANGED — workflow's `reposix init` step uses the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist. The only delta is that `REPOSIX_ALLOWED_ORIGINS` is set to confluence-tenant-only in the workflow env. No new HTTP construction site introduced. |
| Cache prior-blob parse (Tainted bytes)    | UNCHANGED — workflow does not introduce new tainted-byte sources. The cache built by `reposix init` is ephemeral (deleted post-runner-shutdown).                                                                                                                                                |
| **`repository_dispatch` `client_payload` (NEW)** | NEW: the workflow receives a JSON `client_payload` from the dispatching party (Confluence webhook or any party with a PAT to dispatch). Threat: payload-injection into `run:` shell commands → arbitrary code execution on the runner. Mitigation: workflow IGNORES `client_payload` — derives all values from `secrets.*` and `vars.*` (per RESEARCH.md "Known Threat Patterns" § Untrusted client_payload injection). STRIDE: Tampering — mitigated by S2 + verifier grep check. |
| **GH PAT for cross-repo webhook dispatch (NEW)** | NEW: Confluence's outbound webhook needs a GH PAT with `repo` scope to dispatch into the mirror repo. The PAT is configured on the Atlassian webhook side (NOT in the workflow). RESEARCH.md Pitfall 7 documents the requirement; P85's setup guide walks through it. STRIDE: Authentication / Spoofing — mitigated by user-controlled PAT scope; not exposed to the workflow. |

`<threat_model>` STRIDE register addendum below the per-task threat
register in the plan body:

- **T-84-01 (Tampering — `client_payload` injection):** workflow
  IGNORES the payload (no `${{ github.event.client_payload.* }}`
  references); verifier grep check.
- **T-84-02 (Information Disclosure — secrets in workflow logs):**
  GH Actions auto-redacts `${{ secrets.* }}` in step output;
  workflow avoids `set -x` in `run:` blocks.
- **T-84-03 (Denial of Service — workflow-trigger amplification):**
  cron + dispatch combined could fire 2× per 30min; the `concurrency:
  cancel-in-progress: false` block (D-01) queues the second run; the
  second run sees `main` in sync via `--force-with-lease` and exits
  cleanly. Wasted runner time bounded by GH's 6-hour job limit per
  workflow run; for a 5-min sync, near-zero risk.
- **T-84-04 (Elevation of Privilege — non-owner pushes to mirror via
  workflow):** workflow's `permissions: contents: write` is scoped
  to the mirror repo only; only repo-write users can dispatch; PAT
  for cross-repo dispatch is owner-controlled. Documented in P85.
