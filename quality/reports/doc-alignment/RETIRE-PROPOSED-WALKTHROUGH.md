# RETIRE_PROPOSED walkthrough — 37 rows pending owner TTY confirm

> Generated 2026-04-28 PT during autonomous-mode session. Each row in
> `quality/catalogs/doc-alignment.json` with `last_verdict =
> RETIRE_PROPOSED` is grouped by *why* it's a retire candidate. The
> framing applies the W3 P67 transport-vs-feature heuristic:
> retirement requires the FEATURE to be intentionally dropped with a
> documented decision (ADR / CHANGELOG / research note); transport
> changes alone do NOT retire claims about a user-facing surface.
>
> Per HANDOVER §"What 'retire' means": archived `REQUIREMENTS.md`
> lines STAY (historical record); only the catalog row flips to
> RETIRE_CONFIRMED.
>
> If you disagree with any retirement, leave that row's catalog
> verdict unchanged (don't include it in the confirm-retire loop) —
> it'll stay RETIRE_PROPOSED and we can reclassify in v0.13.0.

## Quick summary by group

| Group | Rows | Rationale class |
|---|---:|---|
| 1. v0.9.0 FUSE pivot supersession | 14 | FUSE transport explicitly dropped per `architecture-pivot-summary.md` (2026-04-24) |
| 2. v0.9.0 superseded by current bench | 4 | v0.8 bench rows; equivalent metric now BOUND via shell verifiers |
| 3. Historical milestone facts | 4 | Past releases (v0-1, v0-2, v0-10, v0-11-active) — no test could fail. v0-11-active is also DOC_DRIFT — see Group 9 caveat |
| 4. One-time structural events | 10 | Doc moves / restructures that shipped; mkdocs gates cover loosely |
| 5. Subjective rubrics | 2 | Handled by `reposix-quality-review` skill, not docs-alignment |
| 6. Status declarations | 1 | Claim IS the status (deferral); not a test target |
| 7. Phase-28 -> Phase-29 supersession | 1 | JIRA write path shipped; Phase-28 read-only claim contradicts current code |
| 8. Forward-looking statement no longer relevant | 1 | "v0.4 will add the write path" — write path shipped Phase 22/24 |
| 9. (cross-cut) DOC_DRIFT caveat on Group 3 row | — | Discussion only; same row as Group 3 v0-11-0-active-milestone |
| **Total** | **37** | |

**Recommended action**: confirm all 37 (one TTY run). After confirm, alignment_ratio = 291 / 321 = **0.9065** (well above v0.12.1's 0.85 target). If any row gives you pause, exclude it from the loop and we can revisit.

---

## Group 1 — v0.9.0 FUSE-pivot supersession (15 rows)

**Why retire**: v0.9.0 ratified the architecture pivot from FUSE mount → git-native partial clone. Source: `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (2026-04-24). The FUSE transport (mount, page-tree symlinks, write-callback, `_INDEX.md` synthesizer) was deliberately dropped — that's a documented architectural decision, exactly what triggers retirement under the W3 heuristic.

**The user-facing capability survived** the transport pivot (write `.md` → page updates still works via `git push`), and that survives via OTHER catalog rows already BOUND (`docs/connectors/guide/backendconnector-{create,update,delete-or-close}-method` for the impl shape; v0.8 `write-01/02/03` rebound to `roundtrip.rs` for the agent UX). The 15 rows below describe FUSE-specific implementation detail that no longer exists.

| Row | Source line | What it described |
|---|---|---|
| `confluence.md/fuse_mount_symlink_tree` | `docs/reference/confluence.md:110-128` | "FUSE mount layout produces pages/, tree/ (symlink hierarchy), and .gitignore" |
| `confluence.md/fuse_daemon_role` | `docs/reference/confluence.md:6-8` | "IssueBackend trait consumed by FUSE daemon and reposix list CLI" |
| `docs/decisions/003-nested-mount-layout/fuse-architecture-retired-v0.9.0` | `docs/decisions/003-nested-mount-layout.md:1-1` | ADR-003 itself (already marked superseded at line 7) — the whole ADR is historical. |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-01` | v0.8.0 REQUIREMENTS.md:29 | `cat mount/tree/<subdir>/_INDEX.md` returns recursive sitemap (FUSE synthesizer) |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-02` | :30 | `cat mount/_INDEX.md` whole-mount overview (FUSE synthesizer) |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-01` | :33 | `ls mount/labels/<label>/` — FUSE label-symlink tree |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-02` | :34 | `ls mount/spaces/<key>/` — FUSE multi-space mount |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/cache-02` | :38 | `git diff HEAD~1 in the mount` — v0.11.0 ADR-007 sync-tags is the v0.9.0+ form (already BOUND) |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-01` | :87 | `cat mount/pages/<id>.comments/<comment-id>.md` — FUSE comments path |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-02` | :88 | `ls mount/pages/<id>.comments/` — FUSE comments-list shape |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-03` | :89 | "Comments are read-only" — was a FUSE EROFS detail; v0.9.0 surface differs |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-04` | :93 | `ls mount/whiteboards/` — FUSE whiteboards tree |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-05` | :94 | `ls mount/pages/<id>.attachments/` — FUSE attachments tree |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-06` | :95 | "Folders exposed as separate tree alongside page hierarchy" — FUSE-tree presentation |

**If you disagree**: any of these you think names a USER-FACING surface (not just FUSE transport) should not retire — flag and we'll reclassify as MISSING_TEST/IMPL_GAP. The ones I'm least sure about are `nav-02` (multi-space mount) and `conf-04` (whiteboards) — those describe shapes that COULD reappear in partial-clone form if the project decided to ship them. They're retire-eligible because there's no documented intent to ship them in v0.9.0+, but if you have plans to re-implement in git-native form, leave them as MISSING_TEST.

---

## Group 2 — v0.9.0-superseded benchmarks (4 rows)

**Why retire**: v0.8.0 benchmark rows describe FUSE-era cold-mount latency framing. The equivalent metrics now ship in `docs/benchmarks/latency.md` + `docs/benchmarks/token-economy.md` and are BOUND via shell verifiers (`quality/gates/perf/latency-bench.sh`, `bench_token_economy.py`) in scan-H. The v0.8 framing is retired; the metric is alive.

| Row | What it described | Where the metric lives now |
|---|---|---|
| `bench-01` | Cold-mount latency <10s for 100-issue sim | `docs/benchmarks/latency.md` cold-init + soft-threshold rows (BOUND) |
| `bench-02` | Read-issue latency <50ms p99 sim warm | `docs/benchmarks/latency.md` cached-read rows (BOUND) |
| `bench-03` | Token-economy MCP-baseline vs reposix | `docs/benchmarks/token-economy.md` (BOUND) |
| `bench-04` | Multi-space cold-mount (3 backends) | per-backend cold-init in `latency.md` (BOUND for sim/github/jira/confluence) |

**If you disagree**: only if you think the v0.8 framing should be revived as a separate metric (e.g. a "multi-backend benchmark" ADR exists I'm not aware of), keep these as MISSING_TEST.

---

## Group 3 — Historical milestone facts (4 rows)

**Why retire**: These describe past releases. CHANGELOG.md + git tags are the historical record. No behavioral test could fail "v0.1.0 shipped" — the git history says it shipped.

| Row | What it described |
|---|---|
| `docs-development-roadmap-md/v0-1-0-shipped` | "v0.1.0 shipped: initial release with sim+confluence+jira+github connectors" |
| `docs-development-roadmap-md/v0-2-0-alpha-shipped` | "v0.2.0-alpha shipped: alpha release with frontmatter validation + audit log" |
| `docs-development-roadmap-md/v0-10-0-shipped` | "v0.10.0 shipped: docs restructure (Diataxis nav, three-page how-it-works, etc.)" |
| `docs-development-roadmap-md/v0-11-0-active-milestone` | "v0.11.0 currently active milestone (POLISH/POLISH2 work in progress)" — see Group 9 caveat |

**If you disagree**: the only one I'd flag is `v0-11-0-active-milestone` (DOC_DRIFT — see Group 9). The other three are uncontroversial historical facts.

---

## Group 4 — One-time structural events (8 rows)

**Why retire**: These describe one-time doc moves or restructures that shipped. The files exist at their destination paths; mkdocs-strict + structure freshness gates loosely cover the post-move state. There's no behavioral assertion to make per row — the structural change is irreversible.

| Row | What it described |
|---|---|
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-01` | InitialReport.md → docs/research/initial-report.md |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-02` | AgenticEngineeringReference.md → docs/research/agentic-engineering-reference.md |
| `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-03` | Cross-references updated to new paths |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs02-three-page-howitworks` | Three-page How It Works section shipped (architecture, git-layer, simulator) |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs03-two-concept-pages` | Two concept pages shipped (mental-model, reposix-vs-mcp) |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs04-three-guides` | Three guides shipped (write-your-own-connector, troubleshooting, security) |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs05-simulator-relocated` | Simulator docs relocated under docs/how-it-works/simulator.md |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs08-theme-readme-rewrite` | mkdocs theme + README rewritten in v0.10.0 |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs11-readme-mkdocs-changelog` | README + mkdocs nav reference CHANGELOG.md |
| `planning-milestones-v0-10-0-phases-REQUIREMENTS-md/mkdocs-nav-diataxis-restructure` | mkdocs nav restructured to Diataxis |

10 rows total in this group.

**If you disagree**: any of these you think SHOULD have a behavioral test (e.g. you want a script that asserts `docs/research/initial-report.md` exists), I can write a small structure-freshness verifier. But that's net-new infrastructure. Cleaner to retire and let mkdocs-strict + nav-presence handle it.

---

## Group 5 — Subjective rubrics (2 rows)

**Why retire from docs-alignment**: These are rubric-style judgment calls (cold-reader scan, value-prop clarity) graded by the `reposix-quality-review` skill against `quality/catalogs/subjective-rubrics.json`, NOT behavioral claims that bind to a Rust fn or shell verifier. They're already in the right home; the docs-alignment row is the duplicate.

| Row | What it described |
|---|---|
| `cold-reader-16page-audit` | 16-page cold-reader audit completed (subjective rubric) |
| `docs01-value-prop-10sec` | 10-second value-prop scan: hero copy explains reposix in 10 seconds |

**If you disagree**: only flag if you want the docs-alignment dimension to ALSO track subjective rubrics (would dual-track them). Otherwise retire is clean.

---

## Group 6 — Status declaration (1 row)

| Row | What it described |
|---|---|
| `playwright-screenshots-deferred` | "Playwright screenshots deferred to a follow-up phase" |

**Why retire**: The claim IS the deferral decision — it's planning prose, not a behavioral assertion. There's nothing to test. The deferral itself is the status; if/when screenshots ship, a NEW row should be added describing the shipped behavior.

---

## Group 7 — Phase-28 → Phase-29 supersession (1 row)

| Row | What it described |
|---|---|
| `docs/reference/jira.md/read-only-phase-28` | "In Phase 28, JIRA `create_record/update_record/delete_or_close` return not supported" |

**Why retire**: Phase 29 shipped the JIRA write path (`crates/reposix-jira/src/lib.rs:8` docstring + impls at lines 197/279/334). The wiremock test `contract_jira_wiremock_write` exercises the full create/update/delete sequence and asserts each returns Ok. The Phase-28 read-only claim is now factually wrong; the impl has moved on.

**Doc follow-up captured**: `docs/reference/jira.md:96-99` prose still claims read-only. UPDATE_DOC follow-up in v0.13.0 to either rewrite as historical ("v0.11.x was read-only; v0.12 ships writes via Phase 29") or remove the §Limitations Phase-28 line.

**If you disagree**: only flag if you want to keep the "Phase 28 was read-only" historical-fact framing. But I think the doc edit + retire is cleaner.

---

## Group 8 — Forward-looking statement, no longer relevant (1 row)

| Row | What it described |
|---|---|
| `confluence.md/v0.4_write_path_claim` | "'v0.4 will add the write path' — claim is outdated, write path ships in phases 22/24 not v0.4" |

**Why retire**: The forward-looking framing ("v0.4 will add") is no longer applicable — the write path shipped in Phase 22/24. The active write path is captured by `docs/connectors/guide/backendconnector-{create,update,delete-or-close}-method` rows (all BOUND). The prose at `docs/reference/confluence.md:152-154` was rewritten in commit `b6f6dd7` (P72 doc rewrite) to describe the shipped write path.

**If you disagree**: shouldn't — the prose has already been updated. Retiring the row just closes the catalog entry that tracked the stale forward-looking claim.

---

## Group 9 — DOC_DRIFT (could be UPDATE_DOC instead) — 1 row

| Row | What it described |
|---|---|
| `docs-development-roadmap-md/v0-11-0-active-milestone` | "v0.11.0 currently active milestone (POLISH/POLISH2 work in progress)" |

**Why retire-with-caveat**: The prose says v0.11.0 is the active milestone, but v0.12.0 has shipped and v0.12.1 is active. This is DOC_DRIFT — the prose lags reality. I marked it RETIRE_PROPOSED because:
- Either the row should retire (historical milestone fact: v0.11 happened, was active at time of writing), OR
- The prose at `docs/development/roadmap.md:11` should be updated to name v0.12.1 as active, AND the row should rebind to track "the roadmap's stated active milestone matches the workspace `Cargo.toml` version".

**Action options**:
- **Confirm retire** (treat as historical): the row goes RETIRE_CONFIRMED. The prose at `docs/development/roadmap.md` STAYS pointing at v0.11; future editors update it as a separate cleanup. Lowest-friction path.
- **Skip retire + update doc**: flip the row back to MISSING_TEST, update `roadmap.md:11` to "v0.12.1 active milestone", and rebind to a verifier that compares `roadmap.md` active-milestone string with `Cargo.toml` workspace version. More work, more accurate.

I'd vote confirm-retire — the cost of the DOC_DRIFT is low (you can update the line whenever), and a "compare doc to Cargo.toml" verifier would be net-new infrastructure. But your call.

---

## Run command (one TTY)

```bash
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
```

Then commit + push:
```bash
git add quality/catalogs/doc-alignment.json
git commit -m "docs(p67): bulk-confirm 37 retirements (FUSE pivot + historical + structural + subjective)"
git push
```

The push will exercise the pre-push hook against the new catalog state (`alignment_ratio = 0.9065`). The walker is still likely to BLOCK on the residual 24 MISSING_TEST rows (per HANDOVER §3 design — per-row blockers), but the ratio gate clears decisively.

---

## To exclude rows from the loop

```bash
EXCLUDE="row-id-to-skip-1 another-row-id"
for row_id in $(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | .id' quality/catalogs/doc-alignment.json); do
  for skip in $EXCLUDE; do
    [[ "$row_id" == "$skip" ]] && continue 2
  done
  target/release/reposix-quality doc-alignment confirm-retire --row-id "$row_id"
done
```

Excluded rows stay RETIRE_PROPOSED and surface again in next session; we can reclassify in v0.13.0.
