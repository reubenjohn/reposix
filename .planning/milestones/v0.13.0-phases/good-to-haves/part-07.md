# v0.13.0 GOOD-TO-HAVES — Part 7 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## 2026-07-07 | Quality gates asserting a third-party tool's exact surface string false-negative when the vendor rewords output | discovered-by: v0.13.0 post-release CI run 28839335746 investigation | severity: LOW-process

**What:** Two gates this release went RED not because the thing they check was broken, but because the verifier's PASS condition grepped a literal, exact surface string from a third-party tool's stdout, and that tool changed its wording between versions: (1) `release/cargo-binstall-resolves` grepped only `github.com/reubenjohn/reposix/releases/download/`, but current cargo-binstall prints `has been downloaded from github.com` on a successful resolve instead of echoing the download URL — the v0.13.0 tag run actually resolved the prebuilt binary in ~1.97s with rc=0, yet the gate FAILed; (2) the p94-badges gate (`quality/gates/docs-build/p94-badges-real-vs-transient.sh`) has the same class of brittleness — asserting an exact badge/response string shape from a third-party service (shields.io or similar) rather than the underlying invariant (badge resolves / is live vs. transient-404).

**Anti-pattern:** quality gates must assert the INVARIANT the row's `expected.asserts` describes (e.g. "resolved a prebuilt binary and exited 0", "badge is live not transiently 404ing"), not a single literal substring lifted from one observed run of a third-party tool's output. Vendor CLI/service wording drifts across versions with no changelog signal to the gate; a single-string match makes every such drift a false-negative RED that looks like a real regression until a human diffs the stdout.

**Fix applied to instance (1) THIS session:** `quality/gates/release/cargo-binstall-resolves.py` PASS logic broadened to a `PASS_SIGNALS` tuple (case-insensitive, accepts both the legacy URL-echo and the newer "has been downloaded from github.com" wording) AND requires the independent "will install the following binaries" line, so a third wording change fails toward PARTIAL/FAIL-investigate rather than silently matching nothing forever. Regression test: `quality/gates/release/test_cargo_binstall_resolves.py`. Catalog row `release/cargo-binstall-resolves` (quality/catalogs/release-assets.json) updated to describe the broadened contract.

**Sketched resolution for the class:** audit all `quality/gates/**/*.py` and `*.sh` verifiers for literal-substring asserts on third-party tool/service stdout or HTTP response bodies (grep for hardcoded URL fragments, exact vendor phrase matches, or single-string `in combined` checks without a fallback signal set); for each hit, either (a) broaden to a small accepted-wordings set with a REGRESSION NOTE like the binstall fix, or (b) switch to a more structural signal (exit code + a documented structural marker) that's less likely to drift. Start with `p94-badges-real-vs-transient.sh` since it's the second instance observed this release.

**Why deferred (this occurrence, instance 2 only):** instance (1) was fixed in-session (< 1h, no new dependency); instance (2) — the p94-badges gate — was only *noticed*, not reproduced or fixed, in this dispatch; fixing it requires reading that gate's current assert logic and the live shields.io/badge-service response shape, which is out of scope for this binstall-focused fix.

**Default disposition:** LOW-process — no correctness bug in the underlying feature either time, but a recurring gate-authoring anti-pattern worth a repo-wide sweep before it produces a third false-negative.

## 2026-07-07 | `SURPRISES-INTAKE.md` has outgrown its own pre-commit soft limit (~77k chars vs. 20k warn threshold) | discovered-by: v0.13.0 post-release verification pass | severity: LOW (process)

**What:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` is now ~77k chars; the
pre-commit hook already warns (non-blocking) on any commit touching it past 20k chars. It
needs a distill/split pass (progressive disclosure — move resolved/dated history into
`.planning/RETROSPECTIVE.md` or an archive file, keep only live OPEN entries in the working
file per the "bound-to-live-state" rule in SESSION-HANDOVER §0).

**Why out-of-scope for eager-resolution:** distilling ~700 lines of entries requires
per-entry triage (which are truly resolved/superseded vs. still live) — that's exactly the
milestone-close OP-9 distillation work, not a drive-by edit mid-verification-pass.

**Sketched resolution:** at the next milestone-close (OP-9 distillation into
`.planning/RETROSPECTIVE.md`), split this file: archive terminal/RESOLVED/superseded
entries out (git is the archive per the file's own carry-forward-banner convention), leave
only live OPEN/DEFERRED entries in the working file, and consider a per-quarter or
per-milestone rotation so the file doesn't recross the 20k soft limit again.

**Default disposition:** LOW — recommended at milestone-close; do not distill now (mid
verification pass is not the venue).

**STATUS:** OPEN

## 2026-07-07 | Ship a bundled default seed inside the release binary so getting-started needs no `--seed-file` / no network fetch | discovered-by: v0.13.1 Wave E (README quick-start doc-lie fix) | severity: MEDIUM

**What:** `reposix sim` with no `--seed-file` seeds 0 issues (confirmed by a real run:
`reposix-sim: listening on http://127.0.0.1:7878 (seed: none (no --seed-file), 0 issues)`).
The only fixture that seeds real data, `crates/reposix-sim/fixtures/seed.json`, is a
source-tree file NOT bundled into any release archive (Homebrew, cargo-binstall,
curl-installer, PowerShell) — so every prebuilt-binary install path is stuck at 0 issues
unless the user separately fetches that fixture. Wave D (`docs/tutorials/first-run.md`,
`docs/index.md`) and Wave E (`README.md`) both worked around this by having the
getting-started flow `curl` the fixture from `raw.githubusercontent.com` before starting
the sim. That workaround is honest and verified, but it introduces a hard network
dependency into the first 60 seconds of onboarding (an air-gapped or firewalled dev, or
anyone hitting a GitHub outage, cannot complete the tutorial) and adds one more moving
part a copy-pasting dev can get wrong (wrong URL, `curl` not installed, corporate proxy
blocking raw.githubusercontent.com).

**Why out-of-scope for eager-resolution:** the real fix is embedding the fixture into the
`reposix-sim` (or `reposix-cli`) binary at compile time (e.g. `include_str!` pointing at
`crates/reposix-sim/fixtures/seed.json`, wired behind a `--seed-file` fallback so an
explicit `--seed-file` still overrides the embedded default) and deciding the UX contract
for "seed on first run with no flag" vs. "stay explicit, but ship the embedded default
under a new flag like `--seed-default`". That's an intentional behavior change to the sim
CLI plus a release-asset verification loop (rebuild all 8 crates' release binaries,
confirm the embedded bytes survive `cargo binstall` / Homebrew / curl-installer / choco
paths) — real cross-cutting work, not a docs-only drive-by.

**Sketched resolution:** add a `const DEFAULT_SEED: &str = include_str!("../fixtures/seed.json")` (or equivalent embed macro) to `reposix-sim`; when `reposix sim` starts with
no `--seed-file`, seed from the embedded default instead of seeding nothing (or add an
explicit `--seed-default` flag if "no flag = empty" is intentionally load-bearing
elsewhere and shouldn't silently change). Once shipped, revert the Wave D/E curl-fixture
workaround in `README.md` + `docs/tutorials/first-run.md` + `docs/index.md` back to a
plain `reposix sim --bind 127.0.0.1:7878 &` with no curl step and no `--seed-file` flag —
removing the network dependency from onboarding entirely. Verify against a real prebuilt
binary (not `cargo run`) so the embed genuinely survives packaging.

**Default disposition:** MEDIUM — this is the real root-cause fix underneath two doc
workarounds (Wave D + Wave E); closing it removes a network dependency from the
zero-shot-gate onboarding path and lets both docs surfaces simplify. Target: v0.14.0.

**STATUS:** OPEN

## 2026-07-07 | doc-alignment catalog carries a ~180-row backlog of un-rebound drift outside this milestone's edited docs | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM

**What:** Running `reposix-quality doc-alignment walk` against the current committed
catalog surfaces roughly 180 rows across `quality/catalogs/doc-alignment.json` in
`STALE_TEST_DRIFT` / `STALE_DOCS_DRIFT` that are unrelated to any file this milestone
(v0.13.1) touched — benchmark pages, the glossary, connector-guide docs, and older
sessions' bindings that drifted from source/test changes in prior milestones and were
never re-bound. Wave E1b rebound exactly the 8 rows whose staleness this milestone's
`docs/**`/`README.md` edits freshly introduced (`docs/tutorials/first-run.md`,
`docs/guides/troubleshooting.md`, `docs/index.md`); the pre-existing ~180-row backlog
was deliberately left untouched (git-diff-verified: only the 8 in-scope rows changed
in the commit). This means the catalog's `alignment_ratio` (0.789) and `coverage_ratio`
(0.181) summary numbers are currently propped up by a large silently-stale substrate —
neither ratio can be trusted as "everything reachable from HEAD is actually bound"
until the backlog is drained.

**Why out-of-scope for eager-resolution:** re-binding ~180 rows requires per-row
citation work (reading each doc's current content, re-deriving accurate line ranges
and claims, confirming the bound test still asserts what the claim says) — this is
exactly the `/reposix-quality-backfill` full-extraction workflow's job, not a
drive-by merge inside a scoped docs-rebind lane. Several of the backlog rows are also
flagged `coverage: row ... cites out-of-eligible file ...` (e.g. rows citing
`crates/reposix-core/src/backend.rs`, `docs/architecture.md`, `docs/demo.md`) which is
a distinct structural cleanup (retire or re-point the citation), not a hash refresh.

**Sketched resolution:** dispatch a dedicated `/reposix-quality-refresh <doc>` pass per
stale doc (or `/reposix-quality-backfill` for a full extraction sweep) before the next
milestone trusts `alignment_ratio`/`coverage_ratio` as a release gate; triage the
`cites out-of-eligible file` rows separately (retire via `propose-retire` +
`confirm-retire`, or re-point the citation to the file that actually moved).

**Default disposition:** MEDIUM — doesn't block v0.13.1 (this milestone's own edits are
now honestly bound), but the backlog's size means the dimension's headline ratios are
not yet trustworthy signal. Target: v0.14.0 scoping session.

**STATUS:** OPEN

## 2026-07-07 | `doc-alignment walk` mutates the committed catalog in place with no `--persist` gate, unlike `run.py`'s GRADE/PERSIST split | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM

**What:** `reposix-quality doc-alignment walk` (help text: "Hash drift walker --
updates `last_verdict` only") writes its recomputed `last_verdict` (and summary block:
`claims_bound`, `alignment_ratio`, `coverage_ratio`, `last_walked`, etc.) straight back
to `quality/catalogs/doc-alignment.json` on every invocation — there is no `--persist`
flag to gate this, unlike `quality/runners/run.py`, which (per the D-P96-01 GRADE/
PERSIST split documented in `quality/PROTOCOL.md`) is validate-only by default and
requires an explicit `--persist` to mutate `quality/catalogs/`. A diagnostic-only
invocation of `walk` (e.g. "let me see what's stale before I decide what to rebind")
silently dirties the tree with a full-catalog re-verdict — confirmed directly in this
lane: running `walk` once flipped `claims_bound` 261→265 and touched ~180 rows'
`last_verdict` fields, none of which this lane intended to commit. The lane recovered
via `git checkout -- quality/catalogs/doc-alignment.json` before the real (surgical,
`bind`-based) rebind work, and Wave E1 (`4f1e0f0`) hit the identical side effect and
used the same recovery.

**Why out-of-scope for eager-resolution:** changing `walk`'s persistence contract is a
runner-semantics change to a load-bearing quality-gate tool (touches the pre-push gate
at `gates/docs-alignment/walk.sh`, the `status` verb's read path, and every doc in
`quality/catalogs/README.md` describing the docs-alignment dimension) — it needs the
same design care as the P96 `run.py` GRADE/PERSIST split, not a same-session
drive-by patch mid docs-rebind.

**Sketched resolution:** add a `--persist` flag to `doc-alignment walk` mirroring
`run.py`'s contract: default invocation computes and prints the stale-row report
without writing `quality/catalogs/doc-alignment.json`; `--persist` writes it back.
Update `gates/docs-alignment/walk.sh` (the pre-push gate) to pass `--persist`
explicitly since that gate's job IS to mint the walked state. Document the flag in
`quality/catalogs/README.md` § docs-alignment dimension and cross-reference the
D-P96-01 precedent in `quality/PROTOCOL.md`.

**Default disposition:** MEDIUM — tooling-hygiene; a silent-dirty diagnostic tool is a
recurring trap (this is the second lane in two Waves to hit it) but not correctness-
blocking since every lane so far has caught it via `git status` before committing.
Target: v0.14.0 scoping session.

**STATUS:** OPEN

---

## 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-risk executor | severity: MEDIUM

**What:** The `badges-resolve` check (README/docs badge URLs must resolve — shields.io,
Codecov, CI status) was FAILing on pre-push. Root cause was unconfirmed at filing time:
transient upstream flake vs. genuinely-broken badge URL.

**Resolution (P94 D3, 2026-07-05):** Determined **TRANSIENT** via ≥2 spaced isolated
re-runs (3 runs across ~8 min, all 10 badge URLs returned HTTP 200 + correct
content-type on the first attempt; no deterministic 404/wrong-type ever observed).
Full evidence + verdict: `.planning/phases/94-real-backend-frictions/94-D3-badges-determination.md`.
Fix applied (TRANSIENT branch of the catalog contract): `badges-resolve.py` `head_url()`
now retries a transient failure (network error, or HTTP 408/425/429/5xx) up to
`MAX_ATTEMPTS = 3` with `BACKOFF_S = (1.0, 2.0)` spacing; a deterministic failure
(404/403/other-4xx or wrong content-type) still fails fast on the first attempt, so the
retry cannot mask a genuinely-dead badge. Net: `python3 quality/gates/docs-build/badges-resolve.py`
exits 0 reliably instead of flaking RED on pre-push.

**Note (2026-07-07):** This entry was accidentally pruned by `1b37350` ("prune v0.13.0
intakes to open-only") and restored here — the `docs-build/p94-badges-real-vs-transient`
verifier mechanically reads this RESOLVED entry as its assert-2 precondition, so it must
persist in `GOOD-TO-HAVES.md` (do not re-prune while that gate is live). Surfaced by
S-260707-pr-07.

**STATUS:** RESOLVED

---

## 2026-07-07 | `git-version-requirement-documented.sh` is a bare `grep -F '2.34'`, cannot detect a hard-vs-recommended regression | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** The structure/docs gate `quality/gates/.../git-version-requirement-documented.sh`
(exact path TBD by the next touching phase — locate via `grep -rl git-version-requirement
quality/gates/`) passes as long as the literal string "2.34" survives anywhere in the
target doc. It cannot distinguish "git >= 2.34 is a HARD requirement" from "git 2.34+ is
RECOMMENDED" — so a future regression from the now-softened recommended-not-hard framing
back to a hard floor (or vice versa) would sail through the gate silently.

**Acceptance:** tighten the verifier to assert the "recommended"/"WARN not ERROR" framing
specifically (e.g. grep for the phrase pattern that distinguishes recommended-vs-required,
not just the bare version literal), so the softened git-floor story is test-enforced, not
just prose.

**Why deferred:** gate-touching change with its own false-positive review across whatever
docs it currently scans; out of this lane's mechanical-filing envelope (no `docs/**`
edits, no gate edits).

**Default disposition:** LOW — always closes; fold into the next `quality/gates/`-touching
phase or the git-floor drift fix (SURPRISES-INTAKE.md 2026-07-07 entry) landing together.

**STATUS:** OPEN

## 2026-07-07 | doc-alignment `walk` mutates the catalog with no `--persist` gate — dirties the tree on every validate-only run | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Waves E1/E1b/F1b) | severity: MEDIUM

**What:** Duplicate/companion filing to the "2026-07-07 | `doc-alignment walk` mutates
the committed catalog in place" entry already in this file (search for that header) —
confirming here for visibility since it bit every push this session. `reposix-quality
doc-alignment walk` writes its recomputed verdict straight back to
`quality/catalogs/doc-alignment.json` with no `--persist` gate, unlike `run.py`'s
GRADE/PERSIST split. Bitten Waves E1, E1b, and F1b, plus every push this v0.13.1 session
— each lane had to `git checkout -- quality/catalogs/doc-alignment.json` before
committing.

**Acceptance:** add a read-only/`--persist`-gated mode so validate-only walks don't dirty
the catalog, mirroring the `run.py` D-P96-01 GRADE/PERSIST split precedent.

**Why deferred:** runner-semantics change to a load-bearing quality-gate tool (see the
existing fuller entry in this file for the full rationale) — not a mechanical-filing
action.

**Default disposition:** MEDIUM — recurring trap (3+ lanes hit it), not yet
correctness-blocking. Target: v0.14.0 scoping session. This entry is intentionally
lightweight since the full write-up already exists in this file; kept as a second
pointer so a future dedupe pass can merge them.

**STATUS:** OPEN

