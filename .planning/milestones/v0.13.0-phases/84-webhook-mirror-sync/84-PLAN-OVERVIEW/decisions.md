# Decisions ratified at plan time

← [back to index](./index.md)

The four open questions surfaced by RESEARCH.md § "Open Questions"
are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Decisions D-01..D-08
correspond to RESEARCH.md OQ#1..#4 plus four planner-discretion
calls.

### D-01 — Q1 (concurrency block): YES, add `concurrency` block to the workflow (RATIFIED)

**Decision:** the workflow YAML adds a top-level
`concurrency: { group: reposix-mirror-sync, cancel-in-progress: false }`
block. NOT cancel-in-progress (per below).

**Implementation (T02):** add to YAML at top-level (sibling of `on:`,
`permissions:`, `env:`, `jobs:`):

```yaml
concurrency:
  group: reposix-mirror-sync
  cancel-in-progress: false
```

**Why YES (vs leaving it off):** cron + `repository_dispatch` could
fire 2× near a 30-min boundary if a webhook fires within seconds of
a cron tick. While `--force-with-lease` makes the second push a
no-op (the first run's push advanced `main`; the second run's lease
check fails cleanly per the race walk-through in RESEARCH.md), the
cost-of-protecting is 5 lines of YAML; the cost-of-being-bitten is
twin runs racing through `reposix init` + cache build (~2-3 min of
runner time) and producing duplicate `audit_events_cache` blob-mat
rows. Idempotent at the mirror level, but wasteful at the runner
level.

**Why `cancel-in-progress: false` (NOT true):** `cancel-in-progress:
true` would mean a cron tick that fires while a webhook-dispatched
run is in flight CANCELS the in-flight run. That's the wrong
direction — the in-flight run is already syncing the mirror; killing
it mid-sync produces a partially-applied state (some blobs
materialized, no push). Better to QUEUE the second run behind the
first; the second run sees `main` already in sync and exits cleanly.

**Precedent:** `ci.yml:16-18` uses `concurrency: { group: ${{
github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`.
The cancellation semantics are different there (CI is OK to cancel
because a new push supersedes the old one); for sync workflows
queue-don't-cancel is the idiomatic choice.

**Source:** RESEARCH.md § "Open Questions" Q2 ("Does the workflow
need `concurrency:` to prevent overlapping runs?"); `ci.yml:16-18`
precedent.

### D-02 — Q2 (latency measurement cadence): one-shot artifact with `cadence: pre-release` (RATIFIED)

**Decision:** the `webhook-latency-floor` catalog row has `cadence:
pre-release` (NOT `pre-pr`, NOT `weekly`). The `webhook-latency.json`
artifact is one-shot per release; T05 lands the v0.13.0 measurement;
re-measurement happens at v0.14.0 P0 or whenever a perf-related code
path changes.

**Why pre-release (NOT pre-pr):** the latency measurement is real
network I/O against TokenWorld + the GH Actions runner. Running it on
every PR is (a) flaky (TokenWorld rate limits, GH Actions runner
cold-start variance, atlas API hiccups), (b) costly (10 manual edits
per measurement; can't be automated against TokenWorld without
mock-edit infrastructure that doesn't exist), and (c) overkill —
the substrate doesn't change PR-to-PR. Tracking a per-release
number in `webhook-latency.json` as an asset-exists check is the
matching cadence.

**Why NOT `weekly`:** `weekly` cadence would fire `webhook-latency-floor`
in a cron context where the verifier needs to RE-RUN the measurement
to refresh the artifact. The synthetic-dispatch path (`gh api repos/
.../dispatches`) could in principle do this, but introduces noise
in the GH Actions tab and doesn't add signal — pre-release is when
the reader cares.

**Falsifiable threshold:** the verifier asserts `p95_seconds ≤ 120`
(per ROADMAP SC4 "if p95 > 120s, P85 docs document the constraint").
Aspirational target is `p95 ≤ 60`; failure threshold is `p95 ≤ 120`;
between-bands triggers a documentation entry but not a row failure.

**Open follow-up:** if owner load post-v0.13.0 surfaces value in
recurring measurement, a v0.14.0 GOOD-TO-HAVE row could change
cadence to `weekly` with a CI runner-side synthetic dispatch driver.
Not in scope for v0.13.0.

**Source:** RESEARCH.md § "Open Questions" Q1 ("Should the latency
measurement be a recurring CI check or one-shot artifact?"); ROADMAP
SC4 falsifiable threshold.

### D-03 — Q3 (workflow's setup-guide reference): inline 5-line tldr in YAML header + forward-link to P85 (RATIFIED)

**Decision:** the workflow YAML's header (top-of-file comment block)
includes a 5-line tldr summarizing what the workflow does, what
secrets it needs, and how to override the cron cadence — followed by
a forward-link to `docs/guides/dvcs-mirror-setup.md` (the P85
deliverable). The forward-link is fine because the docs file lands
before milestone close (P85 explicitly depends on P84 GREEN).

**Implementation (T02):** the YAML header block:

```yaml
# reposix-mirror-sync — v0.13.0 webhook-driven mirror sync
#
# What: sync the GH mirror with confluence-side edits (the pull
# direction; bus-remote handles the push direction).
# Triggers: repository_dispatch (event_type=reposix-mirror-sync) +
# cron */30min safety net + workflow_dispatch (manual).
# Secrets needed: ATLASSIAN_API_KEY, ATLASSIAN_EMAIL,
# REPOSIX_CONFLUENCE_TENANT (set via `gh secret set` on this repo).
# Cron override: edit this YAML directly (the schedule field cannot
# read ${{ vars.* }} — see Pitfall 3 in P84 RESEARCH).
# Full setup walk-through: docs/guides/dvcs-mirror-setup.md (P85).
```

**Why inline-tldr-plus-link (vs link-only or full-docs-inline):**
link-only assumes the reader has the canonical repo handy; an owner
hitting the workflow file from the mirror repo's GitHub UI may not
have that context — 5 lines of inline orientation is enough to
answer "what is this and what env does it need?". Full-docs-inline
duplicates information that gets stale; the tldr is the irreducible
minimum.

**Forward-reference acceptability:** P85 explicitly depends on P84
GREEN (ROADMAP P85 `Depends on: P79 + P80 + P81 + P82 + P83 + P84
ALL GREEN`). The link `docs/guides/dvcs-mirror-setup.md` will be
live by the time milestone v0.13.0 ships. If P85 slips, the link is
a 404 in the interim — acceptable because the workflow is owner-only
during the dev window; readers hitting the mirror repo UI in that
window are the owner who knows the doc is forthcoming.

**Source:** RESEARCH.md § "Open Questions" Q4 ("Should P84's plan
reference forward to P85's docs or vice-versa?"); ROADMAP P85
dependency.

### D-04 — `agent-ux.json` is the catalog home (NOT a new `webhook-sync.json`)

**Decision:** add the 6 new rows to the existing
`quality/catalogs/agent-ux.json` (joining the P79–P83 row family).
NOT a new `webhook-sync.json` or `release.json`.

**Why:** `agent-ux` is the existing dimension that P79–P83 already
populated; adding 6 more keeps the single-file shape readable.
P82's D-04 set the precedent ("dimension catalogs are routed to
`quality/gates/<dim>/` runner discovery — `agent-ux` is the existing
dimension"); P84 inherits.

**Why NOT a release-dimension row for `webhook-latency-floor`:** the
latency artifact is a perf claim about an agent-ux substrate (the
webhook sync is part of the agent UX — it's how the mirror stays
current with what the agent sees). Releases handle CHANGELOG +
asset-bytes claims; they don't track substrate-quality numbers.
`agent-ux` is the right dimension; `pre-release` is the right
cadence (D-02).

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P82 D-04 precedent.

### D-05 — `cargo binstall reposix-cli` (NOT `reposix`) — verbatim

**Decision:** the workflow YAML's install step uses `cargo binstall
reposix-cli`. NEVER `cargo binstall reposix`.

**Why:** the published crate name is `reposix-cli` (verified in
`crates/reposix-cli/Cargo.toml` `[package]` block). `reposix` is the
**workspace name**, not a crate name. The architecture-sketch's
"`cargo binstall reposix`" at line 223 is shorthand; it is incorrect
when literally executed in a workflow.

**Verification at T02:** add a comment in the YAML at the binstall
step pointing at the binstall metadata location (`crates/reposix-cli/Cargo.toml`
`[package.metadata.binstall]`). The verifier `webhook-trigger-dispatch.sh`
greps for `cargo binstall reposix-cli` (NOT `reposix`) in the
workflow YAML — failing the build if the wrong crate name appears.

**Source:** RESEARCH.md § "Standard Stack" + § "Common Pitfalls"
Pitfall 2; `crates/reposix-cli/Cargo.toml:19-25`.

### D-06 — Cron expression is a literal `'*/30 * * * *'` in the schedule field — NEVER `${{ vars.* }}`

**Decision:** the workflow `schedule:` block uses a literal
`'*/30 * * * *'` cron expression. The Q4.1 RATIFIED "configurable
via `vars.MIRROR_SYNC_CRON`" is satisfied by **editing the YAML
directly** when the owner wants a different cadence. NOT by
templating `${{ vars.MIRROR_SYNC_CRON }}` into the cron field.

**Why:** GitHub Actions parses the `schedule:` block at workflow-load
time, BEFORE the `${{ vars.* }}` context is evaluated. The cron
expression CANNOT be templated via `vars` — the templating is parsed
literally and the resulting cron is INVALID. RESEARCH.md Pitfall 3
documents this explicitly with the bench-cron precedent (which
hardcodes `'0 13 * * 1'`). The Q4.1 intent ("configurable") is
preserved by documenting the edit-YAML flow in P85's setup guide.

**Verification at T02:** the YAML's literal cron expression is
asserted by `webhook-cron-fallback.sh` via grep (`grep -F "*/30 * * *
*"` on the YAML file).

**Documentation (T06 + P85):** CLAUDE.md's P84 entry names the
constraint; P85's `dvcs-mirror-setup.md` provides the edit-the-YAML
instructions for owners who want a different cadence.

**Source:** RESEARCH.md § "Workflow YAML Shape" Note + Pitfall 3 +
Assumption A2; bench-latency-cron.yml precedent (line 18 hardcodes
its cron).

### D-07 — First-run handling: branch on `git show-ref --verify --quiet refs/remotes/mirror/main`

**Decision:** the workflow's "Push to mirror" step branches on the
LOCAL state of `refs/remotes/mirror/main` (set by the prior `git
fetch mirror` step). If the ref EXISTS locally → use `--force-with-lease`
push. If the ref is ABSENT locally → use plain `git push mirror main`
(no lease).

**Why this exact predicate (vs e.g. checking the remote directly):**
two reasons: (a) the `git fetch mirror` step IMMEDIATELY precedes
this branch — so `refs/remotes/mirror/main`'s presence is the
freshest possible signal of the remote's `main` existing or not.
(b) `git show-ref --verify --quiet` is a non-network local query —
zero round-trip cost, deterministic exit code (0 = present, 1 =
absent). Compare to a `git ls-remote` re-query at this point (one
extra network round-trip per workflow run, for no information gain
over what `git fetch` just told us).

**Race property:** between `git fetch mirror` and the push, a
concurrent bus push could change `mirror/main` from "present at
SHA-A" to "present at SHA-B". This is exactly the race that
`--force-with-lease=refs/heads/main:SHA-A` detects and rejects. The
race CANNOT, however, change `mirror/main` from absent → present in
this window UNLESS the bus push is itself the first-ever push — and
in that case the workflow's plain push will fail with "non-fast-forward"
and the next cron tick will see the present-state and lease-push.
Acceptable.

**Sub-case 4.3.a (fresh-but-readme):** `gh repo create --add-readme`
seeded the mirror with one README commit. After `git fetch mirror`,
`refs/remotes/mirror/main` IS present (pointing at the README
commit's SHA). The lease push branch fires; the workflow's `main`
(SoT content) replaces the README. Owner sees this on first
workflow run and is documented in P85.

**Sub-case 4.3.b (truly-empty):** `gh repo create` (no `--add-readme`)
or a `gh repo create` followed by an emptying push. After `git fetch
mirror`, `refs/remotes/mirror/main` is ABSENT (no remote `main`
exists to fetch). The plain-push branch fires; `git push mirror
main` creates the ref. Subsequent runs go through 4.3.a.

**Source:** RESEARCH.md § "First-run Handling (Q4.3)" + § "Workflow
YAML Shape" verbatim YAML; `git show-ref` man page for the predicate
semantics.

### D-08 — Live workflow + template copy are byte-equal modulo whitespace; T02 ships both

**Decision:** the workflow YAML exists in TWO places: (a)
**LIVE COPY** at `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
(in `reubenjohn/reposix-tokenworld-mirror`); (b) **TEMPLATE COPY**
at `docs/guides/dvcs-mirror-setup-template.yml` (in `reubenjohn/reposix`).
T02 ships both atomically.

**Why both (vs only-live or only-template):**

- **Only-live:** P85's setup guide can't reference a file that lives
  in another repo without leaving a permanently dangling link or
  requiring readers to clone the mirror repo to read the YAML.
  Worse: catalog-first principle says we land the contract in this
  repo's catalog first; if the live copy is the only artifact and
  the verifier needs to see it, we'd need cross-repo verifier
  invocations (heavy; stateful in a way the rest of the catalog isn't).
- **Only-template:** T01–T05 work would land cleanly, but the actual
  GH Action would never run. The mirror would never sync. Phase
  ships catalog-green but the substrate is dead.

**Drift prevention:** T02's commit ships BOTH copies in the same
working session. The template path is `docs/guides/dvcs-mirror-setup-template.yml`
(NOT `.github/workflows/...`) so it doesn't accidentally activate as
a workflow in the canonical repo. The verifier
`webhook-trigger-dispatch.sh` asserts:

1. `docs/guides/dvcs-mirror-setup-template.yml` exists in the canonical
   repo and parses as YAML.
2. `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/
   workflows/reposix-mirror-sync.yml` returns 200 (live copy exists in
   mirror repo).
3. The two copies are byte-equal modulo whitespace (a `diff -w
   <template> <(gh api ... | jq -r .content | base64 -d)` check).

If the diff is non-zero modulo whitespace, the verifier fails — drift
is caught at the row level, NOT at runtime when the live workflow
breaks.

**Why-modulo-whitespace and not bit-for-bit:** YAML parsers tolerate
trailing-newline / indent-tab/space variations. Forcing bit-for-bit
would create false-positive drift on cosmetic diffs that don't
affect behavior. `diff -w` (ignore whitespace) is the standard
right-cardinality check for "logically identical".

**T02 cross-repo push protocol:** the executor performs the live-copy
push to the mirror repo via:

```bash
# Working tree is in canonical repo. Author template copy.
$EDITOR docs/guides/dvcs-mirror-setup-template.yml
git add docs/guides/dvcs-mirror-setup-template.yml
git commit -m "..."  # part of T02's atomic commit

# Push live copy to mirror repo via temp clone.
TMPDIR=$(mktemp -d); trap "rm -rf $TMPDIR" EXIT
git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git "$TMPDIR/mirror"
mkdir -p "$TMPDIR/mirror/.github/workflows"
cp docs/guides/dvcs-mirror-setup-template.yml \
   "$TMPDIR/mirror/.github/workflows/reposix-mirror-sync.yml"
cd "$TMPDIR/mirror"
git add .github/workflows/reposix-mirror-sync.yml
git commit -m "feat(workflow): add reposix-mirror-sync.yml (P84 / DVCS-WEBHOOK-01)"
git push origin main
```

This is a SEPARATE git operation from T02's commit-into-canonical-repo
flow. The two commits are NOT atomic across repos (no cross-repo
two-phase commit exists in git). If the canonical-repo commit lands
but the mirror-repo push fails, the verifier catches the drift on
next run; the executor fixes by retrying the mirror-repo push. This
is acceptable because the mirror-repo push is idempotent.

**Source:** RESEARCH.md § "Claude's Discretion" "Whether to ship the
workflow as a template..."; CARRY-FORWARD § DVCS-MIRROR-REPO-01
("workflow YAML lives in `reubenjohn/reposix-tokenworld-mirror`,
NOT in `reubenjohn/reposix`").
