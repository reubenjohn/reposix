# v0.13.1 — Real-backend friction research

**Date:** 2026-05-02
**Method:** 4 parallel dark-factory subagents, each given only user-facing docs + a goal, no prior reposix context. Findings logged sequentially as `T1..T4-*.md`.
**Scope:** validate the v0.13.0 milestone's headline UX claims against reality on real Confluence (REPOSIX space) + the simulator + the GH mirror.

## What we tested

| Test | Feature | Backend | Boxes ticked |
|---|---|---|---|
| **T1** | sim end-to-end (baseline) | simulator | 5 / 6 |
| **T2** | `reposix attach` | real Confluence | 1 / 5 |
| **T3** | bus push (`reposix::<sot>?mirror=<url>`) | real Confluence + GH mirror | 3.5 / 8 |
| **T4** | conflict recovery (`git pull --rebase`) | simulator | 5 / 7 |

## Friction headline numbers

**Total: 37 frictions logged, 16 HIGH, 13 MED, 8 LOW.**

| Test | HIGH | MED | LOW |
|---|---|---|---|
| T1 | 6 | 0 | 2 |
| T2 | 2 | 3 | 4 |
| T3 | 7 | 7 | 2 |
| T4 | 1 | 3 | 0 |

## Cross-cutting clusters (the actual story)

### CLUSTER A — `reposix attach` is unimplemented for real backends 🔴 SHOWSTOPPER

The v0.13.0 headline subcommand `reposix attach` rejects all non-sim backends with a one-line scaffold error that **leaks internal phase IDs** in production:

```
Error: attach: backend "confluence" not yet wired in P79-02 scaffold (sim only);
github/confluence/jira land alongside the integration tests in P79-03
```

- `--help` advertises full functionality (orphan policy, ignore-dirs, remote-name).
- `docs/concepts/dvcs-topology.md` Pattern C describes the documented flow.
- README has zero mention of `attach`.
- P79-03 apparently never landed despite v0.13.0 being marked complete (11/11 phases ✅ in `STATE.md`).

**Affects:** T2 (entire test blocked), T3 (Pattern C unreachable).
**Evidence:** T2-attach.md F6, F7. T3-bus-push.md F4, F5.

### CLUSTER B — `git pull --rebase` recovery doesn't work 🔴 LOAD-BEARING

The architecture's load-bearing claim — *"helper rejects with the standard git 'fetch first' error; agent recovers via `git pull --rebase && git push`"* — is half-functional:

- ✅ Rejection works exactly as documented (T4 WIN).
- ❌ Recovery fails: every helper fetch mints a NEW root commit with no ancestry to the prior tip. `git fetch` after any push fails with `fatal: error while running fast-import`. `git pull --rebase` cascades the same error.

**Affects:** T4 step 6 (recovery), step 7 (verify both pushes landed) unreachable.
**Evidence:** T4-conflict-recovery.md HIGH-1.
**Implication:** the dark-factory regression test in CI either doesn't exercise post-push fetch, or it's been broken silently. CLAUDE.md says it's the v0.9.0 architectural cornerstone.

### CLUSTER C — OP-3 audit log violated on every push 🔴 PROJECT'S OWN INVARIANT

CLAUDE.md OP-3: *"Audit log is non-optional… either schema missing a row for a network-touching action means the feature isn't done."*

Verified across T1 + T4: **every `git push` from a partial-clone working tree** prints:
```
WARN cache unavailable for push audit: ... fatal: not in a git directory
```
…and writes **zero** `helper_push_*` rows. SoT-side push works (REST shows version bump), but cache-side `cache.db` is never even created. Two consecutive pushes in T4 → 0 audit rows. The whole helper-push audit pipeline is dark.

**Cause** (subagent's hypothesis, not verified): wrong cwd / gitdir resolution when helper opens cache to write audit row. Manual `git config --add transfer.hideRefs` in either the worktree or the cache bare repo works fine; only the helper's invocation path is broken.

**Evidence:** T1-sim-baseline.md F6. T4-conflict-recovery.md MED-2 (subagent flagged this should be HIGH).

### CLUSTER D — Bus push has structural conflicts with mirror setup 🔴 ARCHITECTURAL

The helper's frontmatter-only export validator **rejects exactly the files the documented mirror setup tells you to commit**:

- `docs/guides/dvcs-mirror-setup.md` step 4 instructs the user to `git commit` a `.github/workflows/reposix-mirror-sync.yml` workflow file into the mirror repo.
- The helper's export path rejects `.github/workflows/*.yml` and `README.md` as "invalid records" (no frontmatter).
- `reposix refresh` itself writes `.reposix/.gitignore` and `.reposix/fetched_at.txt` into the working tree → helper export rejects these too.

Result: bus push **cannot succeed** against any mirror that follows the docs.

**Evidence:** T3-bus-push.md F12, F13.

### CLUSTER E — Init UX broken on first contact ⚠️ COMPOUND

Three independent issues in `reposix init`:

1. **Wrong "Next:" hint.** Init prints `Next: cd … && git checkout origin/main` — that fails (`pathspec 'origin/main' did not match`). The README form `git checkout -B main refs/reposix/origin/main` works but contradicts the binary. (T1 F1, T4 MED-1, prior session env-pollution finding.)
2. **WARN noise on success path.** `WARN: git fetch --filter=blob:none failed … fatal: could not read ref refs/reposix/main` printed even when the fetch DID populate `refs/reposix/origin/main`. No way for fresh user to know if it's broken or not. (T1 F2.)
3. **Bare `git push` fails.** README + tutorial both use bare `git push`; needs `--set-upstream origin main` first. (T1 F5.)

### CLUSTER F — Tutorial expected output is stale 🟡 DOCS

- README + `first-run.md` step 5 say issues live at `issues/0001.md`. Reality (default sim seed): `0001.md` at repo root.
- Tutorial expected `cat` output ("Add user avatar upload" / `assignee: alice@acme.com`) is fictional — actual seed issue 1 is "database connection drops under load."
- Five additional copy-pasteable steps fail verbatim.

**Evidence:** T1-sim-baseline.md F3, F4.

### CLUSTER G — Quality framework structurally exempted real-backend flows ⚠️ META

(From the prior forensics-subagent investigation, not the dark-factory tests, but same milestone.)

- P86 verdict (DVCS dark-factory third arm) GREEN despite TokenWorld arm being **untested** — explicitly deferred under "substrate gap" framing.
- The `dark_factory_real_confluence` test (`crates/reposix-cli/tests/agent_flow_real.rs:146-164`) **stops at "URL has the right shape"** — never runs `git fetch` or `git push`.
- Agent-ux dimension verifies sim + cargo-test wire paths only. Shell subprocess flows against real backends have **zero verifier coverage**.
- The deferral was internal (catalog comment + SURPRISES-INTAKE) — public docs ("pure git after init") never qualified.

This is *why* clusters A–F shipped silently: **no automation ever tries the docs against a real backend end-to-end.**

### CLUSTER H — Smaller doc / env issues 🟡

- `docs/reference/testing-targets.md` references "TokenWorld" Confluence space — that space does NOT exist on the configured tenant. Real space is `REPOSIX`.
- README has no mention of `reposix attach`.
- No tutorial for Pattern C (round-tripper / bus push).
- Required git version (2.34+) not surfaced in install docs — Ubuntu 20.04's default 2.25.1 silently breaks init fetch.
- Mirror README says "do not commit directly" but Pattern C's documented flow includes `git commit -am` in the mirror tree.
- Bus URL literal example missing from all user-facing docs (cold reader must mine CLAUDE.md).

## Recommended v0.13.1 phase shape (4 work + 2 reservation = 6)

> User pre-approved this shape. Roadmapper in next session should validate the breakdown against current planning conventions.

**P89 — `reposix attach` real-backend implementation** (Cluster A)
- Land confluence/github/jira backends for `attach`. The work P79-03 was supposed to ship.
- Acceptance: `reposix attach confluence::REPOSIX` against a vanilla mirror clone configures git correctly + reconciles by frontmatter id (5 cases per architecture sketch).
- Blocks: most of T2, T3 will pass once this lands.

**P90 — Push-flow correctness fixes** (Clusters B + C)
- Fix `git pull --rebase` recovery: helper-side fetch must preserve ancestry across post-push refetches (no fresh root commits per fetch).
- Fix push audit log silence: helper must open cache.db with correct gitdir/cwd resolution; `helper_push_*` rows MUST land for OP-3 compliance.
- Acceptance: two-writer conflict scenario in T4 completes step 6 + step 7. Audit table has rows for every push.

**P91 — Bus-push compatibility with documented mirror setup** (Cluster D)
- Helper export must accept (or skip) non-frontmatter files in the mirror tree (`.github/workflows/*`, `README.md`, `.reposix/*`).
- Decide: skip-list, allowlist, or path-filter (e.g. only export `pages/*.md` / `issues/*.md`)? Trade-off doc + ADR.
- Acceptance: bus push against a mirror with `.github/workflows/reposix-mirror-sync.yml` succeeds.

**P92 — Quality framework upgrade + doc fixes** (Clusters E + F + G + H)
- Add 6 catalog rows from prior forensics (real-backend post-init flow, headline-promise qualifier, env-propagation audit, TokenWorld-vs-real-space binding, etc.).
- Add `kind: shell-subprocess` verifier type with explicit env-control assertions.
- Fix init's "Next:" hint, WARN-noise, bare-`git push` UX nits.
- Refresh tutorial expected output (issues/, sample bodies, `--set-upstream`).
- Add git ≥2.34 requirement to install docs. Add Pattern C tutorial.
- Fix `testing-targets.md` to say REPOSIX (not TokenWorld) and bind a verifier that probes the configured tenant.
- Drop "P79-02 scaffold" / "P79-03" leak from `attach` error messages.

**P93 — Surprises absorption** (+2 reservation slot 1, OP-8)
- Drains anything P89–P92 surfaced but couldn't fix without doubling scope.

**P94 — Good-to-haves polish + milestone close** (+2 reservation slot 2, OP-9 retrospective ritual)
- Cold-reader pass on revised docs.
- RETROSPECTIVE.md distillation.
- Tag `v0.13.1`.

## Per-test file pointers

- `T1-sim-baseline.md` — sim end-to-end (baseline; the simulator path is the cleanest but still has 6 HIGH frictions including OP-3 violation).
- `T2-attach.md` — `reposix attach confluence::REPOSIX` blocked at the binary; thesis path of v0.13.0 unimplemented.
- `T3-bus-push.md` — bus URL form + mirror-lag refs; structural conflicts with documented mirror setup.
- `T4-conflict-recovery.md` — `git pull --rebase` recovery broken; rejection-detection works perfectly.

## Open questions for the new session

1. Is `reposix attach` for real backends a *bug* (P79-03 silently dropped) or a *deliberate scope cut* (the docs lie)? Either way, v0.13.1 closes the gap.
2. Should bus push fix (P91) include a runtime `path-filter` config so users can choose which paths are SoT records vs mirror-only files?
3. Is the cache-.db creation bug (OP-3) v0.13.1 patch material or v0.14.0 cache-coherence redesign material? Recommend patch.
4. Should v0.13.0 be re-tagged with a known-issues changelog amendment, or should v0.13.1 be the user-facing release with v0.13.0's tag staying as-is?
