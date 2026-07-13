# B1 litmus self-heal — PROVEN INSUFFICIENT (2026-07-12)

**Lane:** v0.14.0 tag-remediation, STEP 1 (B1), post-D1. **Status: escalated to
OWNER** — the manager-blessed genuine fix (DECISION-1) is implemented, proven to
fire correctly against live TokenWorld, and proven **unable to green the
litmus**. Two real substrate gaps, both empirically reproduced. Do NOT improvise
past this point per charter — this needs an owner call (see
`.planning/CONSULT-DECISIONS.md` OPEN entry filed alongside this document).

Raw evidence: `quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`
(the `shell-subprocess` kind's mandatory transcript, argv/env_keys/cwd/exit_code +
full stdout/stderr, `ts: 2026-07-13T03:58:51Z`, `exit_code: 1`).

## What was done

Self-heal added to `quality/gates/agent-ux/lib/litmus-flow.sh`'s push block
(commit `d413432`, pushed, 60/60 pre-push gates PASS). On a `git push reposix
main` rejection it runs the documented recovery **exactly once**: `git pull
--rebase reposix main` (the BUS remote) followed by a retry `git push reposix
main` — **never `origin`** (the stale mirror; `attach` wires `fetch`/`pull` to
read from `origin`, so a naive `git pull --rebase` with no remote argument
would silently re-pull the same stale mirror and never see backend-current).
Bounded to a single fetch-rebase-retry; a rejection persisting past it fails
closed to RED — treated as a real coherence bug, not mirror lag. The
`sync --reconcile` mirror-refresh doc-lie in root `CLAUDE.md` (§ "Mirror-head
refresh promise") was fixed in the same commit.

## Proven-exercised against live TokenWorld

First `git push reposix main` was rejected — the standard "fetch first" error
(page 2818063, local base v1 vs backend v7):

```
issue 2818063 modified on backend at 2026-07-06T06:28:06.103+00:00 since last fetch
(local base version: 1, backend version: 7). Run: git pull --rebase
 ! [rejected]        main -> main (fetch first)
```

The self-heal branch fired — transcript marker `running documented recovery`
appears **exactly once**:

```
↻ push rejected — running documented recovery: git pull --rebase reposix main + retry
(out-of-band SoT drift; troubleshooting.md § DVCS push/pull)
```

It rebased against the bus (`reposix/main`), retried, and the retry also
failed → the bounded self-heal exhausted → **exit 1 (RED)**. NOT a green.

## Root cause — the self-heal STRUCTURALLY cannot green the litmus

Two independent blockers, both empirically reproduced in the same transcript:

1. **Lineage gap.** Fetching `reposix` shows `* [new branch] main ->
   reposix/main` — the bus's history is a **freshly reconstructed SYNTHETIC**
   history (rebase replays from "Initial commit"), unrelated to the local
   `main`'s ancestry (the GitHub-mirror's fan-out history). Unrelated ancestry
   turns every shared file into an `add/add` conflict on rebase:
   ```
   CONFLICT (add/add): Merge conflict in pages/7798785.md
   Auto-merging pages/7798785.md
   CONFLICT (add/add): Merge conflict in pages/2818063.md
   Auto-merging pages/2818063.md
   Patch failed at 0003 reposix refresh: confluence/TokenWorld — 3 issues at 2026-07-04T20:48:13Z
   ```
2. **Unparseable ADF fixture.** Page 2818063 ("demo space Home", the litmus's
   only non-protected editable target) has an ADF body reposix-confluence
   cannot parse:
   ```
   adf_to_markdown failed; using empty body error=adf_to_markdown: root node
   type must be "doc", got ""
   ```
   The bus reads an empty body for this page while the mirror carries real
   content markers, so even a lineage-clean rebase would hit a content
   conflict on this file specifically (observed as the `add/add` on
   `pages/2818063.md` above, compounding with the lineage gap rather than
   being cleanly separable from it in this run).

## Implication

The documented recovery `git pull --rebase && git push` does **NOT** recover
for the canonical vanilla-clone+attach topology — it only recovers for trees
already tracking a bus ref with shared ancestry. This is a real product/docs
gap, not merely a litmus artifact (filed to SURPRISES-INTAKE, see below). The
ADF-unparse path also silently empties a real page's body on every clean push
through this fixture — a data-loss risk (also filed).

## Proper fix (v0.15.0-class)

Litmus substrate redesign: (a) check out the working tree from the bus
tracking ref instead of the mirror clone, so lineage is shared and rebase is a
clean fast-forward/no-op-diff by construction; AND (b) replace the
unparseable-ADF home page with a parseable, durable, editable fixture. This is
architecture-shaping (E2-class) and mid-tag-sequence defect work — which the
manager's standing tag-authority explicitly reserves for arc-gated defect-lane
work, not an in-sequence self-decide.

## TokenWorld state after D1

Net-zero footprint from this investigation: exactly 2 durable pages confirmed
(parent `7766017` + child `7798785`, `child.parentId=7766017`); page `2818063`
untouched by this run (the litmus's local edit never landed — the push failed
both attempts). CI-safe; no orphan pages created by this lane.

## The conflict this creates

DECISION-1 (manager, `.planning/MANAGER-HANDOVER.md`) blessed the self-heal as
B1's genuine fix under an explicit **non-waivable, NO caveat escape**
constraint for the litmus's `blast_radius: P0` row. The self-heal is now
proven to fire correctly but cannot green the litmus, and the only sanctioned
mid-sequence path (this self-heal) is exhausted. Escalated to the owner — see
`.planning/CONSULT-DECISIONS.md` OPEN entry filed alongside this document.
