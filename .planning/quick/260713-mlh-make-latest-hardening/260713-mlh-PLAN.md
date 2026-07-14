---
quick_id: 260713-mlh
title: "release.yml make_latest preventive hardening (+ 2 planning de-stale riders)"
status: ready
created: 2026-07-13
---

# Quick Task 260713-mlh — make_latest hardening + planning de-stale

Post-tag queue **item 1** (re-scoped, manager-ratified). The original "v0.13.0 tag
sequence" is DROPPED — v0.13.0/v0.13.1 are already released and v0.14.0 is Latest
(2026-07-14). **No tag is cut in this item.** Scope = a preventive `release.yml`
`make_latest` hardening (prove-before-fix) + 2 planning de-stale riders in the same lane.
Workflow YAML + planning docs only. NO cargo, NO crates.

## The hazard

`.github/workflows/release.yml` runs `gh release create/edit` with NO `--latest` flag.
gh omits `make_latest` when the flag is absent → GitHub's REST omit-default `make_latest="true"`
applies → a future **back-tag** (an older version tagged after a newer release, e.g. a
v0.13.2 hotfix pushed while v0.14.0 is Latest) would STEAL `releases/latest`, 404-ing every
installer URL that points at `releases/latest`.

## Checklist

### A. PROVE actual behavior FIRST (executed; DP-2 prove-before-fix)
- [ ] Executed probe on the sanctioned target `reubenjohn/reposix` (drafts/scratch only,
      full cleanup, live v0.14.0 Latest untouched, scratch tags NOT matching `v*`).
- [ ] Corroborate with `gh release create --help` `--latest` default + REST API default.
- [ ] Capture commands + outputs to `make-latest-proof.md`. Plain REAL/not-real verdict.

### B. FIX release.yml (informed by the proof)
- [ ] Confirm trigger is tag-based (editing + pushing to main must NOT trigger a release).
- [ ] Fix `gh release create` + sibling `edit` so a back-tag does NOT become latest while
      the genuinely-newest release still does. Explicit + deterministic over API defaults.
- [ ] Fix-it-twice comment at the edit site. YAML parses. Do NOT trigger a real release.

### C. Planning de-stale riders (same lane)
- [ ] STATE.md § Workstream C body prose: de-stale "TAG BLOCKED" / "READY-TO-TAG: NO" /
      B1–B5 tag-remediation cursor to current reality (v0.14.0 shipped + Latest; nothing
      tag-blocked). Terse; invent no new claims.
- [ ] PROJECT.md: add a short dated truth banner at the TOP (OP-8 eager-fix, NOT a
      wholesale reconcile — full re-anchor deferred to `/gsd-new-milestone`).

## Acceptance

- Executed proof artifact committed with a plain REAL/not-real verdict.
- release.yml back-tag path yields `--latest=false`; newest-release path yields `--latest=true`;
  YAML parses; no real release triggered.
- STATE.md Workstream C de-staled; PROJECT.md banner verbatim.
- Pushed to origin/main; `HEAD...origin/main` = `0 0`.

## Constraints

- NO cargo (workflow YAML + planning docs only).
- External mutations ONLY on `reubenjohn/reposix` per the sanction; drafts/scratch only;
  full cleanup; live v0.14.0 Latest untouched.
- No `--no-verify`. Hook BLOCK → STOP + report verbatim. Uncommitted = didn't happen.
