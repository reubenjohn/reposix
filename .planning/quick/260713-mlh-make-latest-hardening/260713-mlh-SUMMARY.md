---
quick_id: 260713-mlh
title: "release.yml make_latest preventive hardening (+ 2 planning de-stale riders)"
status: complete
completed: 2026-07-14
---

# Quick Task 260713-mlh — SUMMARY (make_latest hardening + planning de-stale)

Post-tag queue item 1 (re-scoped). Proved the `make_latest` back-tag hazard is REAL
(executed probe + authoritative source), fixed `release.yml` with explicit deterministic
`--latest`, and de-staled two planning docs. Workflow YAML + planning docs only — NO cargo,
NO crates. No tag cut; no real release triggered; live v0.14.0 Latest untouched.

## A. The proof (executed; prove-before-fix) — VERDICT: hazard is REAL

Artifact: `make-latest-proof.md`. Executed on the sanctioned target `reubenjohn/reposix`
with drafts/scratch only (scratch tags NOT matching `v*`, so they can't trigger `release.yml`):

- **Draft probe** — `make_latest` reads `null` on GET (it is a write-only create/edit
  parameter); a draft can't demonstrate the steal, by design.
- **`GH_DEBUG=api` payload probe** — gh's create POST body contains `tag_name`/`name`/`draft`
  but **no `make_latest` key** when `--latest` is absent.
- **Authoritative confirmation** — gh v2.62.0 `create.go` L427: `make_latest` is added ONLY
  when `--latest` is explicit; otherwise omitted. GitHub REST OpenAPI: omitted `make_latest`
  **defaults to `"true"`** = "set THIS release as Latest" (NOT the date/version-aware `legacy`
  its help text misleadingly implies).

**Chain:** no `--latest` → gh omits `make_latest` → GitHub applies `make_latest="true"` → the
release becomes Latest regardless of tag → **a future back-tag would STEAL `releases/latest`,
404-ing installer URLs. Hazard REAL → fix warranted.**

## B. The fix (`.github/workflows/release.yml`, "Create / update release")

Trigger confirmed tag-based only (`push.tags` + `workflow_dispatch`; NO `push.branches`) —
editing the file + pushing to main does NOT trigger a release.

Before → after of the release-create/edit invocation:

```
# BEFORE
gh release edit   "$TAG" --notes-file body.md
gh release create "$TAG" --title "reposix ${TAG}" --notes-file body.md

# AFTER (compute highest published app-release semver; pass --latest explicitly)
EXISTING=$(gh api ".../releases" --paginate --jq 'select(non-draft,non-prerelease).tag_name'
           | sed -nE 's/^(reposix-cli-)?v([0-9.…])$/\2/p')
HIGHEST=$(printf '%s\n%s\n' "$EXISTING" "$VERSION" | grep -E '^[0-9]' | sort -V | tail -1)
[[ "$HIGHEST" == "$VERSION" ]] && IS_LATEST=true || IS_LATEST=false
gh release edit   "$TAG" --notes-file body.md --latest="$IS_LATEST"
gh release create "$TAG" --title "reposix ${TAG}" --notes-file body.md --latest="$IS_LATEST"
```

**Mechanism:** highest-published-semver comparison via `sort -V`, applied to both the create
and edit paths (release-plz creates the object first, so `edit` is the common path). Explicit
+ deterministic, independent of gh/API default drift. Fix-it-twice comment added at the site.

**Newest-release path stays correct** — verified against the LIVE release list:
`0.15.0 → --latest=true` (newest wins), `0.13.2 → false` (back-tag never steals),
`0.14.0 → true` (re-run of current Latest), `0.0.0-dev-… → false` (dispatch synthetic).
YAML parses (`yaml.safe_load` OK); `actionlint` not installed locally (skipped).

## C. Planning de-stale riders

1. **STATE.md § Workstream C** — de-staled the frontmatter `workstream_c` cursor
   (`status`/`next_phase`), the `###` header, the READY-TO-TAG blockquote, the B1–B5
   tag-remediation lane block, the Current Focus row, and the resume cursor. All now read
   "v0.14.0 SHIPPED + Latest 2026-07-14; b773c04 RED-main CLOSED @ 8e2aae5; nothing
   tag-blocked". B1–B5 detail retained as historical record + v0.15.0-intake pointers. Terse;
   no new claims.
2. **PROJECT.md** — added the verbatim dated truth banner at the top (blockquote, before the
   stale 2026-05-01 footer). OP-8 eager-fix only; wholesale re-anchor deferred to
   `/gsd-new-milestone`.

## Commits

- `370310d` — `fix(release): pin make_latest so back-tags never steal releases/latest`
  (release.yml + make-latest-proof.md + PLAN.md)
- `a5081a1` — `docs(planning): de-stale STATE.md Workstream C + add PROJECT.md truth banner`
- (this) — `docs(260713-mlh): complete make_latest hardening quick` (SUMMARY)

## Noticing (OD-3 ownership charter — REPORTED)

- **gh help text is misleading** — `gh release create --help` says the `--latest` default is
  "automatic based on date and version" (= `legacy`), but gh never sends `legacy`; it omits
  `make_latest` → GitHub applies `true`. This discrepancy plausibly fooled the original author
  into thinking the default was version-aware/safe. Filed here as the root cause of the hazard.
- **`gh` token lacks `delete_repo` scope** — the charter's scratch-repo steal-demo (option 2)
  couldn't run without leaving an undeletable repo, so it was refused (draft + authoritative
  source proof used instead). One incidental repo `reubenjohn/reposix-scope-test-DELETEME`
  from the scope check could not be deleted; **archived** (inert). Owner cleanup (needs scope):
  `gh auth refresh -h github.com -s delete_repo && gh repo delete reubenjohn/reposix-scope-test-DELETEME --yes`.
- **PROJECT.md is over the 20k soft limit** (22.6k; non-blocking WARN — was already over before
  the ~250-char banner). Wholesale reconcile / progressive-disclosure split is the deferred
  `/gsd-new-milestone` job, not this quick.

## Residue on reubenjohn/reposix

0 — no drafts, no scratch tags; `releases/latest` = `v0.14.0` untouched throughout.

## Self-Check: PASSED

- Proof artifact committed with plain REAL verdict.
- release.yml fix applied (both create + edit paths); YAML parses; trigger tag-only; no real release triggered.
- STATE.md Workstream C de-staled; PROJECT.md banner verbatim.
- Quick record under `.planning/quick/260713-mlh-make-latest-hardening/` (PLAN + SUMMARY + proof).
