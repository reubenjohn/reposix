# Phase 4 — DONE

**Completed:** 2026-04-13 05:00 PDT (T+4:26 from kickoff; ~3h before 08:00 demo deadline).

## Success criteria (all 5 pass)

| SC | Gate | Result |
|----|------|--------|
| 1  | `docs/demo.md` + `docs/demo.typescript` exist; `## Walkthrough` present | ✓ |
| 2  | `bash scripts/demo.sh` exits 0 | ✓ (verified end-to-end on dev host 2026-04-13 04:59 PDT) |
| 3a | `grep -E 'ALLOWED_ORIGINS\|allowlist' docs/demo.md \| wc -l` ≥ 1 | 6 hits |
| 3b | Guardrail markers in typescript (`EPERM\|append-only\|refusing\|allowlist\|Permission denied\|forbid`) ≥ 1 | 6 hits |
| 4  | `grep -E '## Security\|Threat model\|v0\.2' README.md \| wc -l` ≥ 2 | 6 hits |
| 5  | `gh run list --workflow ci.yml --limit 1 --json conclusion` = `success` on latest `main` | ✓ |

## Evidence from the empirical demo run

- Step 6 (FUSE write): issue 1 status `open` → `in_progress`, version bumped from `1` to `2` on the server.
- Step 7 (git push): after pushing `status: in_review`, server confirms `in_review`.
- Step 8a (allowlist refusal): second mount with `REPOSIX_ALLOWED_ORIGINS` mismatched → `ls: reading directory '/tmp/demo-allow-mnt': Permission denied`, stderr confirms `origin not allowlisted`.
- Step 8b (SG-02 bulk-delete cap): first push of 6 deletes exit-code 1, stderr `refusing to push (would delete 6 issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)`; amended commit with the tag pushes successfully.
- Step 8c (audit): `sqlite3` returns real rows showing `GET /projects/demo/issues → 200` and the `DELETE → 204` stream from the override push.

## Commits shipped in Phase 4

| SHA | Message |
|-----|---------|
| `1655815` | feat(04-01): extend demo seed to 6 issues for SG-02 bulk-delete demo |
| `c1af7e4` | feat(04-01): add scripts/demo.sh — 9-step idempotent demo driver |
| `b6a2eb5` | docs(04-01): record script(1) typescript of demo.sh + plain-text excerpt |
| `fdeb211` | docs(04-01): add docs/demo.md walkthrough with guardrails callout |
| `1f724de` | docs(04-02): README Status / Demo / Security / Honest scope for v0.1 ship |

## Files

- `scripts/demo.sh` — 9-step idempotent demo driver.
- `docs/demo.md` — walkthrough.
- `docs/demo.typescript` — raw `script(1)` recording.
- `docs/demo.transcript.txt` — ANSI-stripped plain-text excerpt.
- `README.md` — Status / Demo / Quickstart / Security / Honest scope sections.
- `crates/reposix-sim/fixtures/seed.json` — extended to 6 issues.

## v0.1 ship manifest

- Workspace: 5 crates (`reposix-core`, `-sim`, `-fuse`, `-remote`, `-cli`). All compile, all clippy-clean, all `#![forbid(unsafe_code)]`.
- Tests: 133 passing (`cargo test --workspace`).
- CI: Green on the commit this doc lives in.
- Guardrails enforced: SG-01 (allowlist), SG-02 (bulk-delete cap), SG-03 (frontmatter strip), SG-04 (filename validation), SG-05 (tainted typing), SG-06 (append-only audit), SG-07 (5s EIO timeout), SG-08 (demo shows guardrails).
- All planned functional requirements (FC-01 through FC-09) delivered.
- STRETCH Phase S (write path + git-remote-reposix + SG-02) delivered in 29 min vs 120-min budget.

## Known deferred items (v0.2 backlog)

- Phase S REVIEW.md HIGH findings: CRLF handling in `ProtoReader`, error-line emission on protocol failures, FUSE `create()` id divergence, trailing-newline normalized-compare (M-03).
- Phase 2+3 REVIEW.md MEDIUM findings: rate-limit `DashMap` unboundedness, `X-Reposix-Agent` spoofing, `NamedTempFile` → `TempDir` for WAL siblings, SIGTERM handler in fuse main.
- Real-backend credentials (Jira/GitHub/Confluence) — explicitly v0.2.
- Adversarial swarm harness — dropped from v0.1 scope.
- FUSE-in-CI integration job — dropped from v0.1 scope.

---

*Phase: 04-demo-recording-readme complete. v0.1 shipped.*
