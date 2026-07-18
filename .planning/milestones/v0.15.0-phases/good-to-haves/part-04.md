# v0.15.0 Good-to-haves / carried-forward hardening — Part 4 of 9

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## From the owner-directive lane (post-P115-close scheduling, 2026-07-16)

### GTH-V15-35 — `docs/index.md`: nest "Build from source (advanced)" under "30-second install"
- **Source:** OWNER-DIRECTED (2026-07-16, received after commit `a1f2494`) · **Severity: LOW (docs IA)** · STATUS: **DONE — quick 260716-fmt (2026-07-16).**
- **What:** `docs/index.md`'s `<details><summary><strong>Build from source (advanced)</strong></summary>` block (currently ~L120-136, sitting after "Connector capability matrix") reads as a disconnected install path — a cold reader following the "30-second install" section (L44-67: curl/PowerShell/Homebrew/cargo-binstall tabs) has no visual link to the advanced from-source alternative several screens later. Owner directive: move/nest the "Build from source (advanced)" material under the "30-second install" section so the two install paths (package-manager fast path + from-source advanced path) read as one cohesive section.
- **Execution cautions (MUST hold, verbatim from the owner directive):**
  1. The `structure/install-leads-with-pkg-mgr-docs-index` freshness gate must stay GREEN — the install path must continue to lead with the package-manager options (curl/Homebrew/cargo-binstall/PowerShell), with "Build from source" remaining the subordinate/advanced/collapsed alternative, never promoted above or beside the primary path.
  2. Doc-alignment catalog rows bound to `docs/index.md` must be CHECKED BEFORE the edit (`reposix-quality doc-alignment status` / `walk` scoped to that file) and REFRESHED in the SAME wave as the move (moving lines will shift line-anchored citations per GTH-V15-28 — do not leave rows STALE_DOCS_DRIFT across a wave boundary).
- **Fix-sketch:** relocate the `<details markdown>…Build from source (advanced)…</details>` block to sit immediately after (or nested as a collapsed `<details>` within) the "30-second install" `=== "curl…"` / Homebrew / cargo-binstall tab set (L44-67), before "After — one commit" (L69); re-run `structure/install-leads-with-pkg-mgr-docs-index` + the doc-alignment refresh for the shifted rows in the same commit/wave.
- **Effort:** small (<1h), single-file docs move; execute via `/gsd-quick` per the owner's scheduling.
- **Addendum (2026-07-16, P115 cold-reader pass):** nesting the block alone is NOT
  sufficient — the `reposix sim &` + `reposix init sim::demo` bootstrap commands live ONLY
  inside the collapsed "Build from source (advanced)" block (`docs/index.md` ~L129-130)
  yet are required by EVERY install method, so the visible "After — one commit" demo
  (~L69-76) references `/tmp/reposix-demo` with no visible step creating it. When
  executing this row, also surface those two bootstrap lines in visible prose independent
  of the source-build block (README's Quick Start L56-77 is a good template). Minor,
  same-wave note: L19's "5-line install" count only holds summed across all 4 methods.
- **Addendum 2 (2026-07-16, cold-reader eager-fix BLOCKED into this row):** also rewrite
  the STALE claim on `docs/index.md:93` — "Real-backend cells fill in once CI secret
  packs are wired (Phase 36)" — reality: `docs/benchmarks/latency.md:42` already carries
  real GitHub 320 ms / Confluence 202 ms figures (verified 2026-07-16). The eager-fix
  leaf fail-closed correctly: line 93 is a TWO-CLAIM line whose other clause ("Sim cold
  init is 278 ms…") is hash-bound by catalog row `docs/index/soft-threshold-24ms`
  (line_start/end 93, enforced by `headline-numbers-cross-check.py`), so ANY edit to the
  line needs that row rebound in the same wave. While there, consider splitting the
  two-claim line so future single-claim edits stop requiring unrelated rebinds.

### GTH-V15-36 — Docs site as a "furnished product": owner furnished-product quality bar for P117/P119
- **Source:** OWNER MANDATE (2026-07-16, received after commit `a1f2494`) · **Severity: MEDIUM-HIGH (broad quality bar, P117/P119 shaping input)** · STATUS: OPEN — feeds P117 (doc-truth purge) and P119 (docs simplification) planning.
- **Owner quote (verbatim):** *"Its good, but we can do so much better!"*
- **What:** Broad owner mandate that the docs site (mkdocs-built, `docs/`) should read as a **FURNISHED PRODUCT** with streamlined documentation — not merely factually correct (P117's bar) or merely destaled (P119's bar), but polished as a cohesive, professional surface. This is explicit quality-bar shaping input for BOTH P117 and P119, covering:
  1. **Information architecture** — sections in the order a first-time reader / agent actually needs them; no orphaned or oddly-sequenced material (see GTH-V15-35 for a concrete instance of this class).
  2. **Progressive disclosure** — the 30-second path stays first and uncluttered; advanced/edge-case material (build-from-source, ADR blockquotes, superseded-figure history, deep config) is pushed down/collapsed, never competing with the fast path for attention.
  3. **Visual polish of the mkdocs site** — consistent use of admonitions/tabs/details, no walls of undifferentiated prose, mermaid diagrams that render cleanly (per global CLAUDE.md Operating Principle 1), badges/links that resolve.
  4. **Cold-reader rubric pass over every landing surface** — `docs/index.md`, `README.md`, and other first-touch pages graded via `/doc-clarity-review` / `/reposix-quality-review --rubric <id>`, not just mechanically gated.
- **Fix-sketch:** P117 planner and P119 planner both take this mandate as an explicit acceptance-bar input (not just their existing success criteria) — plan a cold-reader/IA pass as a first-class task in each phase's PLAN.md, not a leftover. Concretely: (a) inventory every landing/tutorial page against the progressive-disclosure decision table (global CLAUDE.md § Self-improving infrastructure), (b) push advanced/edge material below the fold or into collapsed `<details>` blocks consistently (GTH-V15-35 is one instance; audit for siblings), (c) run `/doc-clarity-review` + the cold-reader rubric on `docs/index.md` + `README.md` + any other first-touch page BEFORE declaring either phase done, (d) treat "good, but so much better" as a directive to over-deliver polish, not just clear the stated Success Criteria checkboxes.
- **Effort:** shaping input, not a standalone task — folds into P117 + P119 planning; each phase's plan should size its own cold-reader/polish pass.

### GTH-V15-37 — Embed the owner's 80s launch animation on the mkdocs home page
- **Source:** OWNER-APPROVED P117 LANE ADDITION (2026-07-16); manager feasibility spike VERIFIED via headless playwright (`~/workspace/reposix-animation-pitch`, client-side React/JSX, 7 scenes) — renders flawlessly, scales to iframe container, zero JS errors · **Severity: MEDIUM (owner-approved scope addition)** · STATUS: OPEN — P117-shaping input.
  MANAGER-DEFERRED 2026-07-17 (standing doctrine: outward publishing = owner-only);
  OWNER APPROVAL PENDING — the gh-release-upload + animation-renders live verify E1 ask
  remains OPEN/surfaced to owner; mp4 baseline embed already shipped (644763a).
- **What:** Embed the owner's 7-scene launch animation on the mkdocs home page (`docs/index.md`), productionized per the owner-approved checklist below (verbatim).
- **Productionization checklist (MUST hold, verbatim):**
  1. Pre-compile the JSX offline to a plain JS bundle — removes the `unpkg.com` Babel-standalone/React CDN dependency AND the ~2.8s in-browser-compile blank; self-host React or inline it.
  2. Self-host the two Google Fonts (Space Grotesk, JetBrains Mono).
  3. Embed mode: `TWEAK_DEFAULTS.motionEditor=false`, neutralize the `localStorage` `animstage:t` playhead persistence (returning visitors currently get a frozen end frame), poster + click-to-play rather than autoplay.
  4. Owner's 7MB mp4 export = video fallback + Show-HN/social asset, host as a GitHub release attachment NOT committed to the repo (file-size gates).
  5. Docs gates: assets under `docs/assets/animation/`, strip Windows `Zone.Identifier` files from uploads/, mkdocs-strict + playwright-walk coverage.
- **Addendum (2026-07-16, L0 #44):** the checklist-item-4 mp4 export now EXISTS at
  `/home/reuben/workspace/reposix-animation-pitch/Reposix Launch Animation.mp4` (7.1MB,
  verified on disk 2026-07-16) — video fallback + Show-HN/social asset; attach to a
  GitHub release, never commit to the repo.
- **Fix-sketch:** P117 planner adds a dedicated task implementing the 5-item checklist above in order (bundle precompile → font self-host → embed-mode config → video-fallback hosting → docs-gate coverage), each item independently verifiable; feasibility already de-risked by the manager's headless-playwright spike, so this is an implementation lane, not a research spike.
- **Effort:** medium — new static asset pipeline + mkdocs page wiring; fits inside P117 as an owner-approved scope addition alongside the doc-truth purge.

### GTH-V15-38 — ADR-01 Option C: post-write snapshot fan-out (NOT sanctioned for v0.15; pull-forward gated)
- **Source:** MANAGER ruling 2026-07-16 on
  `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
  Decision 1(iv) (`[MANAGER]` entry in `CONSULT-DECISIONS.md`) · **Severity: LOW
  (deferred design option)** · STATUS: OPEN — pull-forward gated, do not schedule.
- **What:** re-materialize the external mirror tree post-write so mirror == SoT
  immediately after every SoT-changing push (self-idempotent litmus), replacing the
  sanctioned lag-until-convergence model.
- **Pull-forward trigger (verbatim from the ruling):** *"a real incident or recurring
  operational friction from the litmus pre-step"* — until that trigger fires, webhook +
  30-min cron remain the AUTHORITATIVE external-mirror convergence mechanism per the
  same ruling.
- **Fix-sketch:** packet § Decision 1, Option C. If triggered, propose as a phase at the
  then-current milestone boundary; do not fold into an unrelated lane.
- **Effort:** medium-large (fan-out write path + litmus rework) — sized only if triggered.

### GTH-V15-39 — Catalog row-id prefixes inconsistent for the same doc (README-md/ vs README/)
- **Source:** MANAGER-ROUTED noticing (2026-07-16, via #49→#50 handover §5 item 6); cost the manager a false-negative grep · **Severity: LOW** · STATUS: OPEN — tag P126 (doc-alignment polish).
- **What:** `quality/catalogs/doc-alignment.json` uses two different row-id prefix conventions for the SAME underlying doc (`README.md`). Legacy rows use `README-md/` (e.g. `README-md/token-89-percent`, live at `doc-alignment.json:4450`); the hero row minted this milestone uses `README/` (e.g. `README/hero-token-economy-94-75`, live at line 9581). A grep keyed to one prefix silently misses rows under the other — this already produced a false-negative lookup for the manager.
- **Fix-sketch:** pick ONE canonical prefix convention (proposal: derive deterministically from the doc path — e.g. `README.md` → `README-md/`, matching the legacy majority and the slug-safe `.`→`-` encoding used elsewhere) and either (a) migrate the divergent new rows to it, or (b) add a `reposix-quality` linter check that BLOCKS a catalog row whose prefix doesn't match the canonical encoding of its bound file path. Option (b) is the durable fix (prevents recurrence); option (a) alone would re-drift.
- **Effort:** small — either a one-time row-id rename (touch the two `README/`-prefixed rows) or a ~1 linter rule in the catalog validator; no new dependency. Natural home: P126 (doc-alignment polish lane).

### GTH-V15-40 — `.source` is sometimes an ARRAY, crashing naive `jq '.source.file'` catalog-wide filters
- **Source:** NOTICED by `gsd-pattern-mapper` (2026-07-16), carried via #50→#51 handover §5 item 7b; **independently verified against reality by L0 #51 before filing** (was hearsay in the handover) · **Severity: LOW** · STATUS: OPEN — tag P126 (doc-alignment polish, same lane as GTH-V15-39).
- **What:** `quality/catalogs/doc-alignment.json` mixes `.source` field shapes — a type-safe probe (`jq '[.. | objects | select(has("source")) | .source | type] | group_by(.) | map({type:.[0], count:length})'`) confirms **2 array-typed `.source` rows** alongside 397 object-typed ones. A naive catalog-wide `jq '.source.file'` (no type guard) aborts with `Cannot index array with "file"` the moment it hits one of the 2 array rows. Bites any executor doing jq-based doc-alignment rebind-risk verification (e.g. P116 execution, P126 polish) — the filter dies mid-scan rather than skipping the row.
- **Fix-sketch:** workaround is a one-line type guard in any catalog-wide jq that dereferences `.source` — e.g. `.source | (if type=="array" then .[] else . end) | .file`. Durable fix: normalize the catalog schema so `.source` is always the same shape (object, or array-of-objects everywhere), backed by a `reposix-quality` schema-lint that BLOCKS a mixed-type `.source` — prevents recurrence rather than papering over it downstream.
- **Effort:** small — one-line jq guard for the workaround, or a schema-normalization + lint rule for the durable fix; no new dependency. Natural home: P126 (doc-alignment polish lane).

### GTH-V15-41 — Banned-words docs gate does not cover `docs/decisions/**` (ADRs)
- **Source:** NOTICED during P116-02 (2026-07-16) · **Severity: LOW** · STATUS: OPEN — tag P126 (doc-alignment polish, same lane as GTH-V15-39/40).
- **What:** the P1 bare-"replace" ban enforced by `quality/gates/docs-alignment/banned-words*.sh` scans only index/concepts/tutorials/guides/how-it-works; `docs/decisions/**` (ADRs) is entirely out of scope, so the "replace" ban is NOT enforced on ADRs — a pre-existing "replaced" at `docs/decisions/010-l2-l3-cache-coherence.md:215` passes the gate untouched.
- **Fix-sketch:** decide whether ADRs should be in-scope (extend the banned-words glob to include `docs/decisions/**`) or the exclusion should be documented as intentional (ADRs = Layer-3 reference prose, exempt from the reader-facing banned-words ban by design). Either outcome is a small, contained change — a glob edit or a one-line scope comment in the gate script.
- **Effort:** small — glob extension or scope-documentation comment; no new dependency. Natural home: P126 (doc-alignment polish lane).

### GTH-V15-42 — `docs/decisions/010-l2-l3-cache-coherence.md` at 155% of the file-size soft ceiling
- **Source:** NOTICED during P116-02 (2026-07-16) · **Severity: LOW** · STATUS: OPEN — tag P126 (doc-alignment polish, same lane as GTH-V15-41).
- **What:** `docs/decisions/010-l2-l3-cache-coherence.md` is ~30,959 chars = 155% of the 20KB progressive-disclosure soft ceiling; the file-size hook is warn-only until the 2026-08-08 waiver expiry.
- **Fix-sketch:** a progressive-disclosure split before the waiver expires — extract the superseded-options history / decision matrix to a child page, leaving the ADR's live decision + rationale in the parent doc under the ceiling.
- **Effort:** small-medium — one doc-split edit + updating any inbound line-anchored doc-alignment citations that point into the extracted section (cross-reference GTH-V15-28's line-anchored-citation sharp edge). Natural home: P126 (doc-alignment polish lane).

## From P117-01 SC4 Option-B ratification (2026-07-16)

### GTH-V15-43 — Real `reposix detach` subcommand (SC4 Option A; deferred by L0 in favor of Option B)
- **Source:** P117-01 SC4 (2026-07-16); L0-directed filing after ratifying Option B (reword-only) for the attach multi-SoT-conflict error · **Severity: LOW** · STATUS: OPEN — tag CLI-surface. **Needs owner/manager sign-off — DRIFTS the decision-009 LOCKED CLI-surface row.**
- **What:** P117-01 rewrote `crates/reposix-cli/src/attach.rs`'s multi-SoT-conflict error (Option B) to teach a manual recovery — `git remote remove <remote>` then re-attach/re-init — instead of the phantom `reposix detach` subcommand it used to promise. Option A would make that promise real: a first-class `reposix detach` subcommand that unbinds a working tree from its system of record.
- **Fix-sketch:** unbind the SoT — one `Cmd::Detach { path: Option<PathBuf> }` enum arm (`main.rs:40`+) + one dispatch arm + a new ~80-120-line `detach.rs` reusing `worktree_helpers::cache_path_from_worktree` + `doctor.rs`'s `git_config_get/set` pattern: unset `extensions.partialClone`, remove the reposix remote, optionally delete the cache dir; mirror `attach.rs`'s `.git/`-exists guard + idempotency. Adds CLI surface, so it DRIFTS the decision-009 LOCKED CLI-surface row and requires owner/manager sign-off before scheduling.
- **Effort:** small-medium (~3-5h) — one new module + enum/dispatch arm, no new external deps; gated on owner/manager sign-off (CLI-surface lock).

## From P117-01 W1 intake triage (2026-07-16, L0 #55)

### GTH-V15-44 — Split `attach.rs` / `list.rs` (both over the 20k soft ceiling)
- **Source:** NOTICED during P117-01 W1; L0-directed filing after triaging the W1 intake · **Severity: LOW** · STATUS: OPEN — tag CLI-crate size; same split lane as GTH-V15-08 / GTH-V15-42.
- **What:** `crates/reposix-cli/src/attach.rs` (26,021 bytes) and `crates/reposix-cli/src/list.rs` (21,015 bytes) both exceed the `structure/file-size-limits` 20,000-byte soft ceiling, currently masked by the GTH-V15-21 `--warn-only` waiver (expires 2026-08-08). When that waiver lapses the size warnings become push-blocking.
- **Fix-sketch:** extract a shared `backend_errors` sibling module (`crates/reposix-cli/src/backend_errors.rs`) holding the connection-refused / auth error-teaching arms both files reference; mirrors the GTH-V15-08 / GTH-V15-42 split pattern. Dovetails with GTH-V15-45 (the non-sim error-teaching arms would naturally live in that shared module).
- **Effort:** small — one-pass code extraction, no new external deps.

### GTH-V15-45 — github / confluence / jira connection-refused arms lack copy-paste recovery (only the sim arm was upgraded in W1)
- **Source:** NOTICED during P117-01 W1; L0-directed filing after triaging the W1 intake · **Severity: LOW** · STATUS: OPEN — tag CLI-UX / north-star error-teaching.
- **What:** P117-01 W1 raised the SIM connection-refused arms of `reposix list` / `refresh` to the root `CLAUDE.md` § Ownership-charter item-5 north-star bar (teach the fix + suggest the alternative + give a copy-paste recovery command), matching the `crates/reposix-cli/src/init.rs` exemplar. The GitHub, Confluence, and JIRA arms on the same surfaces still emit bare errors with no recovery line.
- **Fix-sketch:** bring the three real-backend arms up to the same `init.rs` exemplar bar — each teaches what failed, the likely cause (missing/expired creds vs unreachable host vs allowlist), and a one-line copy-paste check. Lands naturally alongside GTH-V15-44's shared `backend_errors` module.
- **Effort:** small — ~3 error arms following the established exemplar pattern.

