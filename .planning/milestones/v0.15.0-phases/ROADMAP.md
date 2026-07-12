## v0.15.0 Rust-compiler-grade UX (PLANNING)

> **Status:** stub scheduled 2026-07-12 by `[SELF]` decision (D6 in
> `.planning/CONSULT-DECISIONS.md`) — REVERSIBLE. This is a forward-looking scaffold, not
> a discovery/research pass; the owner (or the C2 closing v0.14.0) MAY pull this phase
> forward into v0.14.0 if the milestone has room. Phase numbering is left UNASSIGNED
> because v0.14.0's live numbers (P102–P113, plus untracked phase dirs 21/22) are owned by
> the concurrent milestone-C2 — a hard-assigned number here would collide. The number is
> assigned at `/gsd-plan-phase`. This scaffold edits NO v0.14.0 file.

**Thesis.** The end-user experience is the north star all reposix tooling serves (root
`CLAUDE.md` § Ownership charter, item 5, tightened 2026-07-12). Today the exemplar
`reposix-cli/src/init.rs::refuse_existing_repo_root` is the ONLY error surface that meets
the Rust-compiler-grade bar — it refuses fail-closed, names the corruption shape, points at
`reposix attach` as the alternative, and prints copy-paste recovery lines. The rest of the
CLI and the `reposix-remote` git helper still emit bare `bail!("usage: …")` and terse
`.context(...)` strings that state a fault without teaching the fix. v0.15.0 brings every
user-facing error to the exemplar's standard, as a first-class lane rather than a leftover.

### Phase TBD: UX error-message audit — Rust-compiler-grade CLI + helper surface
*(number assigned at `/gsd-plan-phase`)*

**Scope.** Audit every `reposix` CLI subcommand error surface (`crates/reposix-cli/src/`:
init / attach / list / sync / doctor / spaces / refresh / etc.) AND the `reposix-remote`
git helper (`crates/reposix-remote/src/main.rs`, `stateless_connect.rs`) error messages,
bringing each to the `init.rs::refuse_existing_repo_root` standard.

**Acceptance bar.** Three-part Rust-compiler-grade UX on EVERY user-facing error:
(1) **teach the fix**, (2) **suggest the alternative**, (3) **give a copy-paste recovery
command**. `init.rs::refuse_existing_repo_root` is the reference implementation every other
surface is measured against. Dimension: **agent-ux / docs-alignment** — route audits to
`quality/gates/agent-ux/`; the phase's first commit writes the catalog contract rows
(catalog-first, per `quality/CLAUDE.md`) before any implementation commit.

**Also on the v0.15.0 roadmap.** Tutorials / onboarding-friction reduction — the remaining
tutorial work carried out of v0.14.0 P106 inherits this three-part error-message bar
immediately (the mandate is active now, not only when this phase runs). Onboarding friction
is a first-class UX surface, not a docs afterthought.

**Deferred-into-v0.15.0 note (coordination guard).** `GTH-09` is being deferred into
v0.15.0 by the concurrent v0.14.0 milestone-C2. The intent is recorded HERE only — this
scaffold does NOT edit the v0.14.0 `GOOD-TO-HAVES.md` or `ROADMAP.md` to effect the
deferral (two-writer conflict guard; C2 owns those live files). The v0.15.0 planner should
reconcile `GTH-09` into this milestone's scope at `/gsd-plan-phase`.

**Seed candidate audit targets** *(candidates from a read-only scan, verify at plan time —
line numbers drift):*

- `crates/reposix-remote/src/main.rs:115` — bare `bail!("usage: git-remote-reposix
  <alias> <url>")`: states the shape but teaches no fix, suggests no alternative, gives no
  recovery command.
- `crates/reposix-remote/src/stateless_connect.rs:331` — terse `bail!("unexpected EOF
  mid-request")`: a raw protocol error with no recovery guidance for the user.
- `crates/reposix-cli/src/attach.rs:113` — `bail!("not a git working tree: {} (.git/
  missing)")`: states the fault but suggests no fix (e.g. `git init` first, or `reposix
  init` for a fresh tree) and gives no copy-paste recovery.
- `crates/reposix-cli/src/list.rs` + `sync.rs` — `bail!`s that do not match the teaching
  pattern; sweep both for terse fault-only messages.

### Phase TBD: Error codes + `reposix explain <code>` — Rust-compiler-grade code namespace
*(number assigned at `/gsd-plan-phase`; folds into or runs alongside the UX
error-message audit phase above — same "Rust-compiler-grade UX" thesis, HEADLINE scope,
committed by the owner 2026-07-12)*

**Scope.** Give every user-facing reposix error a **structured, stable code** in a
dedicated namespace (e.g. `RPX-xxxx`) and ship **`reposix explain <code>`** — a subcommand
that prints the detailed cause, the fix, and copy-paste recovery for that code, mirroring
`rustc --explain E0308`. Applies across the same surface as the audit phase above: every
`reposix` CLI subcommand (init / attach / list / sync / doctor / spaces / refresh / etc.)
and the `reposix-remote` git helper.

**Acceptance intent.** Every user-facing error carries a stable, documented code in its
output; `reposix explain <code>` exists and, for every code emitted anywhere in the CLI or
helper, prints a non-empty cause + fix + copy-paste recovery. This is HEADLINE scope for
v0.15.0, not a nice-to-have — the codified, queryable half of the Rust-compiler-grade UX
north star (§ Thesis above), with the UX-audit phase supplying the prose bar and this
phase supplying the stable-identifier + lookup mechanism.

**Reversibility.** Fully reversible — planning-scaffold only; no code or v0.14.0 file
touched. The owner or the v0.14.0-closing C2 may pull this phase forward into v0.14.0.
