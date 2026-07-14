# make_latest hardening — EXECUTED proof (prove-before-fix, DP-2)

**Date:** 2026-07-14 · **Probe target (sanctioned):** `reubenjohn/reposix` ·
**gh version:** 2.62.0 (2024-11-14)

**Question.** `.github/workflows/release.yml` runs `gh release create/edit` with NO
explicit `--latest` flag. Does a **back-tag** (an older version, e.g. a future v0.13.2
hotfix, tagged now while **v0.14.0 is Latest**) STEAL `releases/latest` — 404-ing every
installer URL that points at `releases/latest`?

**Verdict: THE HAZARD IS REAL.** Under the OLD workflow a back-tag becomes `releases/latest`.

---

## E1 guardrails honored

- Live `releases/latest` = `v0.14.0` **never touched** (re-confirmed after every step).
- No release that could become latest was ever published on `reubenjohn/reposix`.
- All scratch tags used non-`v*` names (`probe-makelatest-DELETEME`,
  `probe-payload-DELETEME`) so they can NOT trigger `release.yml` (trigger is `v*` /
  `reposix-cli-v*` only).
- The disposable-scratch-repo steal-demo (charter option 2) was **NOT executed**: the
  active gh token lacks the `delete_repo` scope (verified — 403), so it could not be
  cleaned up. Refused rather than leave residue. See "Residue" below.

## Method 1 — draft probe (executed; inconclusive on the steal, by design)

```
$ gh release create probe-makelatest-DELETEME --repo reubenjohn/reposix --draft \
    --title "PROBE make_latest DELETE ME" --notes "probe"
https://github.com/reubenjohn/reposix/releases/tag/untagged-889082df4d17dc2dd468

$ gh api repos/reubenjohn/reposix/releases \
    --jq '.[] | select(.draft==true) | {id,tag:.tag_name,draft,prerelease,make_latest}'
{"draft":true,"id":353559038,"make_latest":null,"name":"PROBE make_latest DELETE ME",
 "prerelease":false,"tag":"probe-makelatest-DELETEME"}
```

`make_latest` reads **`null`** on the GET response — it is a **write-only** create/edit
parameter, not a stored/returned field. A draft therefore cannot demonstrate the steal
(drafts can never be latest) and cannot resolve `make_latest`. Draft deleted; 0 residue;
latest still `v0.14.0`.

## Method 2 — observe the ACTUAL request body gh sends (executed, residue-free)

```
$ GH_DEBUG=api gh release create probe-payload-DELETEME --repo reubenjohn/reposix --draft \
    --title "PROBE payload DELETE ME" --notes "payload probe"  2>&1 | grep '"..."'
> POST /repos/reubenjohn/reposix/releases HTTP/1.1
  "tag_name": "probe-payload-DELETEME",
  "name": "PROBE payload DELETE ME",
  "draft": true,
```

The POST body contains `tag_name`, `name`, `draft` — and **no `make_latest` key**. gh
does not send `make_latest` unless `--latest` is passed. Draft deleted; 0 residue.

## Method 3 — authoritative source confirmation (residue-free)

**gh CLI v2.62.0 `pkg/cmd/release/create/create.go` (L194, L427-429):**
```
--latest  "Mark this release as \"Latest\" (default [automatic based on date and version]).
           --latest=false to explicitly NOT set as latest"
...
if opts.IsLatest != nil {                          // ONLY when --latest is explicit
    params["make_latest"] = fmt.Sprintf("%v", *opts.IsLatest)
}
```
gh adds `make_latest` to the payload **only** when `--latest` is explicitly given;
otherwise it is **omitted**, and GitHub applies its REST default.

**GitHub REST OpenAPI — `POST /repos/{owner}/{repo}/releases`, `make_latest`:**
```json
{ "type": "string", "enum": ["true","false","legacy"], "default": "true",
  "description": "... Defaults to `true` for newly published releases. `legacy`
   specifies that the latest release should be determined based on the release
   creation date and higher semantic version." }
```
The omit-default is **`"true"` = set THIS release as latest** — NOT the date/version-aware
`legacy`. gh's help text "automatic based on date and version" describes `legacy`, which
gh **never sends** — a misleading discrepancy that plausibly fooled the original author
into believing the default was version-aware and safe.

## Chained conclusion

Workflow passes no `--latest` (executed) → gh omits `make_latest` (executed GH_DEBUG +
source L427) → GitHub applies `make_latest="true"` (OpenAPI default) → the created/edited
release is set as `releases/latest` **regardless of whether its tag is a back-tag**.

**A future back-tag (e.g. v0.13.2) pushed now would STEAL `releases/latest` from v0.14.0,
404-ing installer URLs that point at `releases/latest`. Hazard confirmed REAL → fix warranted.**

## Fix (see release.yml "Create / update release")

Compute whether this version is the highest published app-release semver (`sort -V`) and
pass `--latest=true|false` **explicitly** on both the create and edit paths — deterministic,
independent of gh/API default drift. Newest release still wins latest; back-tags never steal it.

**Fix verified against the REAL live release list** (read-only, residue-free):

```
Normalized published app versions:
0.1.0 0.2.0 0.2.0-alpha 0.3.0 0.4.1 0.5.0 0.8.0 0.11.0 0.11.1 0.11.2 0.11.3 0.12.0 0.13.0 0.13.1 0.14.0

  version=0.15.0                     highest=0.14.0->0.15.0  -> --latest=true    (genuinely-newest: wins Latest)
  version=0.13.2                     highest=0.14.0          -> --latest=false   (BACK-TAG: never steals Latest)
  version=0.14.0                     highest=0.14.0          -> --latest=true    (re-run of current Latest: correct)
  version=0.0.0-dev-20260714...      highest=0.14.0          -> --latest=false   (dispatch synthetic dev: not Latest)
```

## Residue

- `reubenjohn/reposix`: **0 residue** — no drafts, no scratch tags; `releases/latest` = `v0.14.0` untouched.
- One incidental repo `reubenjohn/reposix-scope-test-DELETEME` was created while
  verifying the `delete_repo` scope (403 → scope absent). It could not be deleted for
  the same reason; **archived** (inert, read-only) as the strongest available neutralization.
  **Owner cleanup (one command, needs the scope):**
  `gh auth refresh -h github.com -s delete_repo && gh repo delete reubenjohn/reposix-scope-test-DELETEME --yes`
