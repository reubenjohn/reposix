# SESSION-HANDOVER.md — v0.13.1 SHIPPED, CI GREEN — 2026-07-11

For the incoming top-level orchestrator (L0). This is the map, not the territory —
detail lives in git and the linked files. HEAD = live state only; history is in `git
log`. No doc carries an unbounded-growth policy: bound to live state, delete
closed/superseded entries rather than appending.

## 0. Owner calibration — READ FIRST (over-ask LESS)

The owner wants **decide-and-record, not gating questions.** Pick the path the owner's
model implies, log it to `.planning/CONSULT-DECISIONS.md` with reasoning, and proceed —
the owner vetoes if you misread. Reserve owner STOPs for the genuinely-owner class only:
**irreversible/destructive moves, external-backend mutations, and credential/spend
authorization** (E1/E3) — e.g. never cut a real tag or fire a real-backend call without
the owner. When you would ask, prefer surfacing a **reversible default to veto** over a
blocking question. "Not a decision, go verify" is not an escalation.

**Owner design taste** (use to make calls autonomously): backend owns identity, client
works in **slugs** (client-side ID remapping is a smell); model multi-step client↔server
interactions as **git-native commit sequences that self-reconcile on partial fail**; big
design questions are **pivots to explore/prototype/converge**, not point-patches; **ship
honest milestones and document known limitations out loud** rather than suppress gates or
hold a green milestone hostage; **guard context aggressively** (fork, prune, lean on git,
least-complex path).

## 1. Current state (ground truth: confirm with `git rev-parse HEAD origin/main`
before trusting this section further)

- **v0.13.1 SHIPPED & VERIFIED.** `origin/main == HEAD == 632f40c`. crates.io: 9 crates
  @0.13.1. Aggregate tag `v0.13.1` cut (tag obj `ccdf404` → commit `04640d5`).
  `release.yml` GREEN; GitHub release `reposix v0.13.1` is Latest (`releases/latest`
  serves 0.13.1, no longer 404s the installer). Assets verified: 4 target archives +
  windows zip + `reposix-installer.{sh,ps1}` + `SHA256SUMS`; **git-remote-reposix
  bundled** alongside `reposix` (front-door fix confirmed via `tar tzf`); installer URL
  HTTP 200; `cargo binstall reposix-cli@0.13.1` resolves the prebuilt binary (verified
  live).
- **The post-release CI gate is now GREEN** (was the last open item). The
  `quality-post-release` workflow (`.github/workflows/quality-post-release.yml`) had been
  failing on `release/cargo-binstall-resolves`. Root cause was a BRITTLE ASSERTION, not a
  product/installer bug: prose changelog commentary had been pasted into the
  machine-checked `expected.asserts` array of that row in
  `quality/catalogs/release-assets.json`, tripping the F-K4b assertion-coverage honesty
  check. Fixed in `632f40c` by moving the prose into an underscore-prefixed
  `_provenance_note` field (runner-ignored), leaving only the real assertion. Verified
  green via `gh workflow run quality-post-release.yml --ref main` → run `29180619597`
  SUCCESS (0 FAIL). NOTE: this workflow ONLY triggers on the `release` workflow
  completing OR `workflow_dispatch` — it does NOT fire on ordinary pushes; verify it via
  `workflow_dispatch`, never by waiting for a push.
- Real-backend transport PROVEN for JIRA + Confluence (first ever, via CI). One
  documented KNOWN-LIMITATION: GitHub-v09 helper-path 404 (`S-260707-gh404`,
  continue-on-error in `ci.yml`, tracked for v0.14.0).

## 2. NEXT — wave-2 v0.14.0 hardening (led non-negotiably by D2)

1. **D2 self-safe dark factory FIRST** — reject-`t@t`-identity commit/push hook + real
   per-leaf worktree isolation + a PreToolUse hook that blocks any leaf writing
   `core.bare`/`user.email` into the shared `.git/config`. The shared config corrupted
   **4-5× this session** (origin NEVER affected — pre-push gate + identity checks held
   every time; no t@t reached published history). Leaf isolation is currently
   doctrine-only with ZERO mechanical enforcement, and even `git worktree remove --force`
   is a corruption vector — do not use it. Anchor: `S-260707-pr-08`.
2. Fix the GitHub-v09 helper-path 404 (`S-260707-gh404`).
3. RBF-LR-03 reconciliation fix (root cause of the broken `git pull --rebase`
   post-conflict recovery, currently documented as a known limitation).
4. Make waived tutorials reproduce (`docs-repro`/`tutorial-replay` + examples 01/02/04/05
   WAIVED-broken until 2026-09-15).
5. Carried HIGHs: live RUSTSEC memmap2 + quinn-proto advisories in `Cargo.lock`;
   `prune_oid_map` pagination-truncation; RBF-FW-11; quality-convergence
   write-contention.

## 3. Live nuisances / process debt (fix early wave-2)

- `doc-alignment` walk dirties the committed catalog on EVERY read (no `--persist`
  gate) — the pre-existing unstaged `quality/catalogs/doc-alignment.json` diff you may
  see in `git status` is this; workaround: `git checkout -- quality/catalogs/doc-alignment.json`.
  High nuisance, prioritize a real fix.
- Post-release verdict process gaps filed by a coordinator (`beacfb4` intake): `|| true`
  masking in the verdict path, waiver-clear promotion gap, stale artifacts. Note
  `beacfb4` also minted `release/cargo-binstall-resolves` status NOT-VERIFIED→PASS — now
  honestly earned after the `632f40c` asserts fix.
- `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` far past soft size limits (file-size waiver
  expires **2026-08-08**) — OP-8 split before then.
- `cargo-nextest` NOT installed on this box; add `cargo clippy --workspace --all-targets
  -- -D warnings` to LOCAL pre-push so CI-clippy-red can't recur silently.

## 4. Release/ops facts (settled)

crates.io publishes on MERGE-to-main via `release-plz.yml`; tag `v*` triggers
`release.yml`; `git_release_enable=false` STAYS (re-enabling stole `releases/latest` +
404'd the installers); the aggregate `v$VERSION` tag is owner-gated (L0 cuts it);
bot-authored release-plz PRs sit at `action_required` until a real-actor reopen;
release-plz auto-titles from conventional-commits (watch for unintended minor bumps —
v0.13.1 had to be forced down from an auto-computed v0.14.0).

## 5. Doctrine

Full delegation / relief / cadence / durable-state doctrine:
`.planning/ORCHESTRATION.md` §3 — relief at ~100k own-context (hard stop ~150k), a
coordinator-of-coordinators per milestone, one-cargo-invocation machine-wide, and the
Leaf Isolation HARD-STOP (leaf test setup runs in a throwaway `/tmp` clone, `cd` into it
in the SAME bash invocation — never mutate git state in the shared repo/worktree).

---

History lives in git — `git log` / `git show`, not restated here.
