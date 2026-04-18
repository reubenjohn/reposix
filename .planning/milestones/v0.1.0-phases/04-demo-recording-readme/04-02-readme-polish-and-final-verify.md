---
phase: 04-demo-recording-readme
plan: 02
type: execute
wave: 2
depends_on:
  - 04-01-demo-script-and-recording
files_modified:
  - README.md
autonomous: true
requirements:
  - FC-08
  - FC-09
  - SG-08

must_haves:
  truths:
    - "README's `## Status` block reports each phase shipped — zero 🚧 markers remaining."
    - "README has a `## Demo` section linking to `docs/demo.md` and `docs/demo.typescript`."
    - "README has a `## Security` section that names the lethal trifecta, links `.planning/research/threat-model-and-critique.md`, and names what's deferred to v0.2."
    - "README has an `## Honest scope` section stating this was built autonomously overnight and is alpha."
    - "`cargo test --workspace` is green at the final commit."
    - "`cargo clippy --workspace --all-targets -- -D warnings` is green at the final commit."
    - "`gh run list --workflow ci.yml --limit 1 --json conclusion` reports `success` for the final commit on `main`."
  artifacts:
    - path: "README.md"
      provides: "Updated README covering Status / Demo / Security / Honest scope."
      contains: "## Demo"
    - path: ".planning/phases/04-demo-recording-readme/04-02-SUMMARY.md"
      provides: "Final verification — commit SHA, CI run ID, test count."
  key_links:
    - from: "README.md ## Security"
      to: ".planning/research/threat-model-and-critique.md"
      via: "inline link"
      pattern: "threat-model-and-critique"
    - from: "README.md ## Demo"
      to: "docs/demo.md and docs/demo.typescript"
      via: "inline links"
      pattern: "docs/demo\\."
---

<objective>
Rewrite `README.md` so a first-time visitor finds accurate Status, a working Demo pointer, an honest Security section, and a scope statement that matches reality. Then run the final verification triad — `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `gh run list` — push the final commit, and close SG-08 + FC-09 + FC-08 by proving CI is green at that commit on `main`.

Purpose: Satisfies ROADMAP Phase-4 Success Criteria #4 and #5. Final commit of the overnight build.

Output: Updated `README.md`, final push, documented CI-green-at-commit.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/04-demo-recording-readme/04-CONTEXT.md
@.planning/phases/04-demo-recording-readme/04-01-SUMMARY.md
@.planning/research/threat-model-and-critique.md
@README.md
@docs/demo.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Rewrite README — Status, Demo, Security, Honest scope (commit 1)</name>
  <files>README.md</files>
  <action>
Read current `README.md` and `docs/demo.md` (written in Plan 04-01). Rewrite `README.md` in place. Preserve: title + one-liner + three badges, "Why" paragraph, "Architecture" ASCII diagram, "License" footer. REPLACE: the Status block and Quickstart, ADD: Demo, expanded Security, Honest scope.

Use these exact headings — ROADMAP SC #4 greps for them:
- `## Status`
- `## Demo`
- `## Quickstart`
- `## Security`
- `## Honest scope`

**Section bodies to write:**

`## Status` — delete the 🚧 list entirely. Replace with a table reporting every phase as ✅ Shipped with a one-line outcome:

- Phase 0 — Bootstrap ✅ 5-crate Cargo workspace, clippy-pedantic, 0 unsafe
- Phase 1 — Core types + simulator ✅ axum sim with audit log, 409 conflicts, rate limits
- Phase 2 — FUSE read path ✅ getattr/readdir/read against live sim
- Phase 3 — CI + swarm harness ✅ GitHub Actions green, FUSE-in-CI, Codecov
- Phase S — Write path + git-remote-reposix ✅ full write + git push round-trip, SG-02 cap
- Phase 4 — Demo + docs ✅ scripts/demo.sh + recorded typescript + walkthrough

Lead sentence: "v0.1 alpha. Built autonomously overnight on 2026-04-13 as an experiment in whether a single coding agent can ship a complete Rust substrate in ~7 hours. Treat as alpha per Simon Willison's 'proof of usage, not proof of concept' rule." Close with test count (~133 green) and zero clippy warnings at `-D warnings`.

`## Demo` — link to `docs/demo.md`, `docs/demo.typescript`, `docs/demo.transcript.txt`. One paragraph stating that three guardrails fire on camera (allowlist, SG-02 bulk-delete cap, server-field strip). Point at `docs/demo.md` for the walkthrough.

`## Quickstart` — replace the existing "when phases 2-5 land" block with: prereqs (Rust 1.82+, `fusermount3`, `jq`, `sqlite3`), three-line clone + `bash scripts/demo.sh`, link to `docs/demo.md#walkthrough` for the expanded per-step flow.

`## Security` — rewrite (currently 2 paragraphs) into:
1. One-paragraph trifecta framing + link to `.planning/research/threat-model-and-critique.md`.
2. A **"Threat model — what's enforced in v0.1"** subsection as a table (SG-01 through SG-08 with mitigation + enforcement cell).
3. A **"Deferred to v0.2"** subsection listing: M-* findings from the audit, real-backend creds, TTY confirmation on `git remote add`, signed recording attestation.
4. Closing line: "v0.1 does **not** authenticate to any real backend. Simulator-only. Treat this codebase as alpha."

`## Honest scope` — new short section. State: ~7 hours of autonomous agent work; SG-01..08 mechanically enforced; still alpha; only run against simulator; don't hand it credentials; read the threat model before considering v0.2. End with "Proof of usage, not proof of concept."

**Grep assertions that MUST pass after the rewrite:**

```
grep -c '## Demo' README.md                                   # ≥ 1
grep -c '## Security' README.md                               # ≥ 1
grep -c '## Honest scope' README.md                           # ≥ 1
grep -c 'threat-model-and-critique.md' README.md              # ≥ 1
grep -c 'docs/demo.md' README.md                              # ≥ 1
grep -cE 'Security|Threat model|v0\.2' README.md              # ≥ 2  (ROADMAP SC #4)
grep -c '🚧' README.md                                         # 0  (no future-tense markers left)
```

Commit: `docs(04-02): README Status / Demo / Security / Honest scope for v0.1 ship`

Commit message body should enumerate the sections added/changed so the commit alone documents what shipped.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; grep -q '## Demo' README.md &amp;&amp; grep -q '## Honest scope' README.md &amp;&amp; grep -q 'threat-model-and-critique.md' README.md &amp;&amp; grep -q 'docs/demo.md' README.md &amp;&amp; [ "$(grep -cE 'Security|Threat model|v0\.2' README.md)" -ge 2 ] &amp;&amp; [ "$(grep -c '🚧' README.md)" -eq 0 ]</automated>
  </verify>
  <done>README has Status / Demo / Quickstart / Security / Honest scope sections with exact heading text; no 🚧 markers remain; all grep assertions above pass; commit landed.</done>
</task>

<task type="auto">
  <name>Task 2: Final verification sweep — cargo test + clippy + fmt, then push (commit 2)</name>
  <files>(none — verification + push only; commit 2 is an empty commit if needed to trigger CI, otherwise skip and rely on the README commit)</files>
  <action>
This task runs the full verification triad and pushes the final set of commits to `origin main`. No code changes.

Step 1 — local verification, all three must exit 0:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

If any fails: STOP, fix it (fmt is auto-fixable with `cargo fmt --all` + extra commit; clippy/test failures require a real investigation — prefer landing a fix as a separate commit rather than amending). Do not proceed to push until all three are green.

Step 2 — confirm `git status` is clean on a tracked branch, then push all commits from Plans 04-01 and 04-02:

```bash
git status
git log --oneline -10
git push origin main
```

Step 3 — wait for CI to complete on the pushed HEAD, then verify it went green:

```bash
HEAD_SHA=$(git rev-parse HEAD)
# Wait for a run on this SHA.
for i in $(seq 1 30); do
  CURRENT_SHA=$(gh run list --workflow ci.yml --limit 1 --json headSha -q '.[0].headSha')
  if [ "$CURRENT_SHA" = "$HEAD_SHA" ]; then break; fi
  sleep 10
done
gh run watch "$(gh run list --workflow ci.yml --limit 1 --json databaseId -q '.[0].databaseId')" --exit-status
RESULT=$(gh run list --workflow ci.yml --limit 1 --json conclusion -q '.[0].conclusion')
echo "CI at $HEAD_SHA: $RESULT"
test "$RESULT" = "success"
```

Step 4 — write `.planning/phases/04-demo-recording-readme/04-02-SUMMARY.md` with:
- Final commit SHA.
- CI run ID + URL.
- `cargo test --workspace` test count (parse the summary line).
- The three grep assertions for SG-08 from `docs/demo.typescript` (re-run them, paste the lines they match).
- Wall-clock time for this plan.

Then commit the SUMMARY:

```bash
git add .planning/phases/04-demo-recording-readme/04-02-SUMMARY.md
git commit -m "docs(04-02): phase 4 ship summary — CI green at <SHA>"
git push origin main
```

(The SUMMARY commit is the very last commit. It does not need to trigger another CI re-verify — the prior commit was already confirmed green, and the SUMMARY is pure documentation. But if the user prefers, push and let CI run once more; either is acceptable. Default: push the SUMMARY and let CI confirm it once more. That SECOND run being green closes the SG-08 + FC-09 + FC-08 loop.)

Commit: `docs(04-02): phase 4 ship summary — CI green at <SHA>` (substantial body summarising what shipped across all 4 phases + Phase S, to satisfy the CONTEXT directive that the final commit message be substantive).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets -- -D warnings &amp;&amp; cargo test --workspace --quiet &amp;&amp; [ "$(gh run list --workflow ci.yml --limit 1 --json conclusion -q '.[0].conclusion')" = "success" ]</automated>
  </verify>
  <done>Three local commands exit 0; final commits pushed to `origin main`; CI run on the final SHA is `success`; `04-02-SUMMARY.md` committed with CI run ID + commit SHA.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| local repo → GitHub `main` | The push publishes the README, demo script, and recording to the world. Nothing in this plan adds new HTTP-egress code paths; all edits are to documentation files. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-05 | I (Info disclosure) | README links to `.planning/research/threat-model-and-critique.md` | accept | The threat-model file is already committed and public; linking it from README doesn't add disclosure. The research doc is the acknowledged red-team artifact. |
| T-04-06 | T (Tamper) | Pushing to `main` | mitigate | We push via `git push origin main` only after local `cargo test` + `cargo clippy` are green. We rely on GitHub branch protection (if configured) + CI check on the pushed SHA. No force-push. |
| T-04-07 | D (Denial of service) | A still-running `scripts/demo.sh` from Plan 04-01 leaving state on the dev host | mitigate | Plan 04-01 Task 2 requires the script to be idempotent. No action needed here beyond running `fusermount3 -u /tmp/demo-mnt 2>/dev/null; pkill -f reposix-cli 2>/dev/null` before starting if the agent's session is fresh. |
| T-04-08 | R (Repudiation) | "CI was green when we shipped" claim | mitigate | `04-02-SUMMARY.md` records the CI run ID + commit SHA, which are verifiable long after the fact via `gh run view <ID>`. |
</threat_model>

<verification>
After both tasks land, all five ROADMAP Phase-4 Success Criteria must hold:

1. **SC #1:** `test -f docs/demo.md && test -f docs/demo.typescript` and `docs/demo.md` has `## Walkthrough`. (Delivered by Plan 04-01.)
2. **SC #2:** `bash scripts/demo.sh` exits 0. (Delivered by Plan 04-01; re-verify once here as a spot-check.)
3. **SC #3:** `grep -E 'ALLOWED_ORIGINS|allowlist' docs/demo.md | wc -l` ≥ 1 AND the typescript contains the allowlist refusal marker. (Delivered by Plan 04-01.)
4. **SC #4:** `grep -E '### Security|Threat model|v0\.2' README.md | wc -l` ≥ 2. (Delivered by Task 1.)
5. **SC #5:** `gh run list --workflow ci.yml --limit 1 --json conclusion -q '.[0].conclusion'` prints `success`. (Delivered by Task 2.)
</verification>

<success_criteria>
- [ ] README.md rewritten; Status / Demo / Quickstart / Security / Honest scope sections all present with exact heading text.
- [ ] Zero 🚧 markers remaining in README.
- [ ] `cargo fmt --all --check`, `cargo clippy -- -D warnings`, `cargo test --workspace` all exit 0 locally.
- [ ] Final commits pushed to `origin main`.
- [ ] CI run on the final SHA is `success` (recorded in SUMMARY with run ID).
- [ ] `04-02-SUMMARY.md` committed with commit SHA + CI run URL + test count.
</success_criteria>

<output>
After completion, `.planning/phases/04-demo-recording-readme/04-02-SUMMARY.md` MUST contain:
- Final commit SHA (the one CI ran against).
- CI run ID + run URL.
- Output of `gh run list --workflow ci.yml --limit 1 --json conclusion,headSha -q '.[]'`.
- `cargo test --workspace` final test count (e.g. "133 passed, 3 ignored").
- The three grep assertions from `docs/demo.typescript` that prove SG-08 (re-run here, paste matching lines).
- Wall-clock time for this plan.
- Any deviations from this plan.
</output>
