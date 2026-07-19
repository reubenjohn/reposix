# P125 Verdict — Real-backend cadence & mirror-drift resilience (v0.15.0, DRAIN-02/DRAIN-12)

**Phase goal (ROADMAP.md:255):** The `pre-release-real-backend` cadence and the
milestone-close vision-litmus survive GitHub-mirror drift instead of false-negatives.

**Overall: GREEN** — all 3 success criteria verified against committed artifacts.

Verifier: fresh gsd-verifier, zero P125 authoring context. Graded against reality
(committed code/scripts/docs/tests), not the executor's word. Commit leads were treated
as leads only; every artifact was read and, where runnable, exercised.

---

## SC1 (DRAIN-02) — mirror-refresh pre-step documented for the real-backend cadence → **GREEN**

**Claim:** A mirror-refresh pre-step (`scripts/refresh-tokenworld-mirror.sh`) is documented
for the `pre-release-real-backend` cadence so a second-run vision-litmus does not
false-negative on its own prior push re-staling the GitHub mirror.

**Evidence — (a) script exists and does what its name claims:**
- `scripts/refresh-tokenworld-mirror.sh` exists, `bash -n` clean. Read in full (161 lines).
  It clones the GitHub TokenWorld mirror (`scripts/refresh-tokenworld-mirror.sh:108`),
  `reposix attach`es it to `confluence::REPOSIX` (`:112`), `git fetch reposix main`
  (`:115`) to pull the backend-materialized `pages/` tree, captures a **pristine
  non-circular** backend reference via `git archive FETCH_HEAD` BEFORE overlay
  (`:121-123`), does a wholesale `git rm -r … pages/` + `git checkout FETCH_HEAD -- pages/`
  (`:128-129`, so backend deletions propagate), fast-forward `git push origin main`
  (`:146`), then re-clones and asserts mirror-versions == backend-versions (`:152-159`).
  `set -euo pipefail` (`:62`) with per-step `|| { echo ERROR…; exit 1; }` guards. Protected
  fixtures `7766017`/`7798785` are asserted-present-and-carried-verbatim, never written
  (`:132-134`). It refreshes the GitHub mirror to backend-current — exactly what its name
  claims.

**Evidence — (b) documented + wired-by-name to the cadence:**
- `docs/reference/testing-targets.md:289-317` — a dedicated `## Mirror-refresh pre-step
  (GitHub-mirror drift)` section that names the `pre-release-real-backend` cadence and its
  runner invocation (`:291-292`), gives the copy-paste command `bash
  scripts/refresh-tokenworld-mirror.sh` (`:300`), an accurate what-it-does line, and an
  explicit "not interchangeable with `reposix sync --reconcile`" distinction (`:306-309`).
- `.planning/REQUIREMENTS.md:166-169` (DRAIN-02) frames it as a "mandatory" pre-step **with
  an explicit OR**: "run `scripts/refresh-tokenworld-mirror.sh` first (or make the litmus
  self-reconcile, see DRAIN-12/GTH-V15-09)".
- **ROADMAP.md:259 (the authoritative Success Criterion) is an explicit OR:** "A documented
  mandatory mirror-refresh pre-step (`scripts/refresh-tokenworld-mirror.sh`) — **or a
  self-reconciling litmus** — prevents a second-run false-negative…". The self-reconciling-
  litmus branch is delivered and verified under SC2 below, and its `_litmus_mirror_reconcile`
  is exactly the mechanism that prevents the false-negative.

**Why GREEN despite the "usually not needed" framing:** `testing-targets.md:311-317`
explicitly says "You usually do NOT need to run this manually" because P125/DRAIN-02 made
the litmus self-reconcile (SC2). This SOFTENS the word "mandatory," but the phase GOAL —
prevent the second-run false-negative — is provably achieved via the OR's self-reconcile
branch, AND the pre-step is genuinely documented and wired-by-name to the cadence as an
escape hatch. P116 (ADR-010) **ratified** this script as "manual op-recovery only," so
re-labeling it "you must run this" would contradict a ratified decision and produce a lying
doc. The ROADMAP contract's OR is satisfied on both branches. See NOTICED §1 for the
framing caveat surfaced for owner override.

---

## SC2 (DRAIN-12) — litmus self-heals BOTH backend drift AND mirror drift before marker push → **GREEN**

**Claim:** The milestone-close vision-litmus self-heals for BOTH (i) backend drift (trashed
protected pages) AND (ii) GitHub mirror drift, reconciling the mirror to backend-current
through the reposix bus remote BEFORE the marker push.

**Evidence — both self-heal branches present, read in full:**
- **(i) backend drift:** `_litmus_fixture_preflight` — `litmus-self-heal.sh:39-51` restores
  (idempotently) the three known TokenWorld ids `2818063 7766017 7798785` via
  `confluence_tokenworld.py restore`, and reparents `7798785` under `7766017` when its
  parentId went null. Invoked FIRST in the flow: `litmus-flow.sh:26`.
- **(ii) mirror drift:** `_litmus_mirror_reconcile` — `litmus-self-heal.sh:60-75` overlays
  the backend-current `pages/` tree via `git -C "$tree" fetch --quiet reposix main` (the
  **reposix bus remote**, `:64`) → `git rm -r … pages/` (`:67`) → `git checkout FETCH_HEAD
  -- pages/` (`:68`) → commit. Invoked at `litmus-flow.sh:49`.

**Evidence — reconcile ordered BEFORE the marker edit/push (grep-confirmed line order):**
- `litmus-flow.sh:26` `_litmus_fixture_preflight`  (backend-drift self-heal)
- `litmus-flow.sh:49` `_litmus_mirror_reconcile "$tree"`  (mirror-drift self-heal, bus remote)
- `litmus-flow.sh:85-92` marker created + `sed`/`printf` edit + commit
- `litmus-flow.sh:106-107` `git push reposix main`  (the marker PUSH)
  → reconcile (49) strictly precedes marker edit (85) and marker push (106). ✓

**Wiring confirmed:** `milestone-close-vision-litmus.sh:110` sources `lib/litmus-flow.sh`,
which sources `lib/litmus-self-heal.sh` (`litmus-flow.sh:19`); the flow is invoked via
`write_transcript_and_artifact "$SLUG" _litmus_flow` (`:120`). All four shell files
`bash -n` clean.

**Bus-remote correctness:** the reconcile reaches backend-current through the `reposix` BUS
remote (`git fetch reposix main`), NOT `reposix sync --reconcile` (which rebuilds only the
local cache) — the helper header (`litmus-self-heal.sh:16-28`) documents this distinction
and the code honors it. It reconciles the LOCAL clone tree (the correct interpretation of
"reconciling the mirror [clone]"); the external mirror head is refreshed by the downstream
marker push's bus fan-out.

Self-heal LOGIC graded by reading (no real-backend mutation triggered, per charter). No
self-contained automated test exercises these helpers — see NOTICED §2.

---

## SC3 (DRAIN-12) — helper teaching string corrected + troubleshooting blockquote made remote-explicit → **GREEN**

**Claim:** (a) the helper's `git pull --rebase` teaching string is corrected for the
mirror-drift case; AND (b) the v0.14.0 attach-tree recovery blockquote in
`docs/guides/troubleshooting.md` is made remote-explicit. BOTH required.

**Evidence — (a) helper Rust teaching string (`crates/reposix-remote/src/write_loop.rs`):**
- `write_loop.rs:222-224` (Pitfall-2 fix, tagged `SC3/DRAIN-12`): emits `hint: run 'reposix
  sync --reconcile' to refresh your cache against the SoT, then 'git pull --rebase'` — the
  `--reconcile` flag (the form that actually rebuilds the local cache) replaces the no-op
  bare `reposix sync`.
- `write_loop.rs:225-236` (Pitfall-4 fix, tagged `SC3/DRAIN-12`): emits `hint: if this tree
  was created via 'reposix attach', a bare 'git pull'/'git rebase' reads the ORIGIN MIRROR
  by default (which may itself be stale) — rebase against the SoT-backed remote explicitly,
  e.g. 'git pull --rebase <reposix-remote-name> main'`. Static `&str`, no interpolation
  (T-125-01 credential-leak guard honored).

**Evidence — (a) test asserts the string, and it PASSES:**
- `crates/reposix-remote/tests/push_conflict.rs:237`
  `mirror_lag_reject_hint_recommends_reconcile_and_remote_explicit_rebase` (wiremock-driven,
  no real backend) asserts: mirror-lag branch fired (`:340`), `reposix sync --reconcile`
  present (`:347`), Pattern-C augmentation present (`:352-354`), pinned `git pull --rebase`
  survives (`:359`). Test name honestly matches its asserts.
- **Ran it:** `cargo test -p reposix-remote --test push_conflict
  mirror_lag_reject_hint_recommends_reconcile_and_remote_explicit_rebase` →
  `test result: ok. 1 passed; 0 failed` (0.07s).

**Evidence — (b) troubleshooting.md v0.14.0 blockquote made remote-explicit:**
- `docs/guides/troubleshooting.md:274-282` — the v0.14.0 "Resolved in v0.14.0 (Phase 105,
  RBF-LR-03)" blockquote now says for a Pattern-C attach tree: "name the bus remote
  explicitly (`git pull --rebase <reposix-remote-name> main && git push <reposix-remote-name>
  main`) … because a bare `git pull --rebase` on an attach tree reads the stale `origin`
  mirror, not the backend."
- `docs/guides/troubleshooting.md:284-290` — a dedicated follow-on note "Pattern-C (`reposix
  attach`) trees — name the bus remote explicitly," cross-referencing the litmus's
  remote-explicit recovery (`litmus-flow.sh:94-99`).

Both (a) and (b) present in committed files, test green. GREEN.

---

## NOTICED (ownership deliverable — near P125's surfaces)

1. **SC1 "mandatory" wording softened to "usually not needed."**
   `docs/reference/testing-targets.md:311` — "You usually do NOT need to run this manually."
   This diverges from the literal charter word "mandatory," but MATCHES the ROADMAP OR-clause
   (`ROADMAP.md:259`) and the P116-ratified "manual op-recovery only" decision
   (`CONSULT-DECISIONS.md:124`, `docs/decisions/010-l2-l3-cache-coherence.md:235`). Graded
   GREEN because the goal (prevent false-negative) is achieved via the self-reconciling-litmus
   branch. Surfaced so the coordinator can override to RED if they read the charter's
   "mandatory" strictly — but reopening to inject a "you must run this" claim would create a
   lying doc, so GREEN is the recommended call.

2. **SC2 self-heal has NO self-contained automated test (coverage gap).** Zero references to
   `_litmus_mirror_reconcile` / `_litmus_fixture_preflight` exist outside `lib/litmus-*.sh`.
   The self-heal LOGIC is exercised ONLY by the env-gated real-backend milestone-close litmus
   (which itself runs only at tag-time). A regression in the reconcile ordering, the
   `git rm`+`checkout FETCH_HEAD` overlay, or the bus-remote fetch would slip past every
   pre-push/CI gate and surface only in a live TokenWorld run. Suggest filing a GOOD-TO-HAVE:
   a local git-fixture harness that exercises the overlay ordering (fetch → rm → checkout →
   marker) without a real backend, the same way `refresh-tokenworld-mirror.sh`'s mechanism
   could be fixture-tested.

3. **Dead `PROTECTED_IDS` variable in `scripts/refresh-tokenworld-mirror.sh:66`.**
   `PROTECTED_IDS=" 7766017 7798785 "` is set but never referenced — the actual protected
   guard (`:132`) hardcodes `for pid in 7766017 7798785`. Already filed in v0.14.0
   SURPRISES-INTAKE part-04:202 and still unresolved. Trivial dead code; a name-honesty/lint
   pass should drop it or wire the guard to read it. Not P125-introduced (script was
   documented, not modified, in P125).

4. **Weakened assertion in `push_conflict.rs:352-354`.** The Pattern-C assert is an OR:
   `stderr.contains("git pull --rebase <reposix-remote-name> main") || stderr.contains("reposix
   attach")`. The `|| "reposix attach"` fallback means a future regression that dropped the
   actual remote-explicit COMMAND while keeping the "reposix attach" prose preamble would
   still pass — despite the test name promising `remote_explicit_rebase` coverage. Currently
   the primary clause matches (both strings emitted at `write_loop.rs:233-235`), so it is not
   a live failure, but the guard is looser than its name implies. Tightening to require the
   command substring (drop the OR) would harden it.

5. **Env-var name/default divergence for the same space.**
   `refresh-tokenworld-mirror.sh:65` reads `REPOSIX_CONFLUENCE_SPACE_OVERRIDE` (default
   `REPOSIX`); `milestone-close-vision-litmus.sh:53` reads `REPOSIX_CONFLUENCE_SPACE` (default
   `TokenWorld`). Comments assert both resolve to space 360450, so default behavior is
   consistent — but an operator overriding `REPOSIX_CONFLUENCE_SPACE` to retarget the litmus
   would NOT retarget the refresh script (different var name), a latent surprise. Cosmetic;
   worth a one-line consistency note if the two are meant to be steered together.

---

_Verified: 2026-07-19T04:31:57Z · Verifier: fresh gsd-verifier (zero P125 authoring context)_
_Cargo: one foreground `-p reposix-remote --test push_conflict` invocation (mutex respected). No real-backend mutation triggered._
