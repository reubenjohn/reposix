---
phase: 42
name: Tutorial + three guides + simulator relocate (DOCS-04, DOCS-05, DOCS-06)
milestone: v0.10.0
status: passed
date: 2026-04-24
---

# Phase 42 Verification — passed

All goal-backward checks from the runner brief pass.

## Files written (5 new, 1 phase scaffold)

| Path | Words | Soft cap | DOCS req |
|------|-------|----------|----------|
| `docs/reference/simulator.md` | 840 | ≤ 800 | DOCS-05 |
| `docs/tutorials/first-run.md` | 842 | ≤ 700 | DOCS-06 |
| `docs/guides/write-your-own-connector.md` | 1212 | ≤ 1500 | DOCS-04a |
| `docs/guides/integrate-with-your-agent.md` | 931 | ≤ 1200 | DOCS-04b |
| `docs/guides/troubleshooting.md` | 981 | ≤ 1000 | DOCS-04c |

Tutorial and simulator are slightly over their soft caps (842 vs 700; 840 vs 800). The tutorial's seven-step structure with copy-pastable commands and expected-output blocks is dense; trimming further would compromise the runnability claim. The simulator reference's eight sections each carry one short paragraph; further trimming would force a TOC-only stub. Soft caps treated as soft.

## Goal-backward checks

### 1. Five new files exist at expected paths — PASS

```
OK docs/tutorials/first-run.md
OK docs/guides/write-your-own-connector.md
OK docs/guides/integrate-with-your-agent.md
OK docs/guides/troubleshooting.md
OK docs/reference/simulator.md
```

### 2. Tutorial includes the seven steps — PASS

```
## 1. Build the binaries
## 2. Start the simulator
## 3. Bootstrap the working tree
## 4. Inspect the project
## 5. Edit an issue
## 6. Commit and push
## 7. Verify the audit row
```

The seven steps cover the runner brief verbatim: install / start sim / bootstrap (`reposix init`) / inspect / edit / commit+push / verify audit row. Each step has an expected-output block. Closing section frames the outcome as the dark-factory pattern ("you used `cat`, `git`, `sed`, `git push`. No reposix-specific commands except `init`").

### 3. Each guide has the section structure called out in the runner brief — PASS

`write-your-own-connector.md`: Anatomy of a BackendConnector · Walkthrough (stub Linear connector) · Audit log requirements (folded into Walkthrough Step 3) · Egress allowlist (Step 4) · Tests (Step 5) · Closing (PR bar — contract test). Five steps inside the walkthrough match the runner spec.

`integrate-with-your-agent.md`: Pattern 1 Claude Code (skill) · Pattern 2 Cursor (shell loop) · Pattern 3 Custom SDK loop · What integration is NOT · See also. Each pattern has a "Gotcha" callout (3 gotchas total). Per ROADMAP §42 SC-5, this is a pointer page — no inline recipe code; full recipes deferred to v0.12.0. Mentioned explicitly at the top.

`troubleshooting.md`: `git push` rejected → `git pull --rebase` (links git-layer §push-time conflict detection) · blob limit error → `git sparse-checkout` (links git-layer §blob limit guardrail) · audit log queries (annotated; full op vocabulary table) · `git diff --name-only origin/main` for "what changed" · real-backend creds → testing-targets · cache eviction documented as v0.13.0 coming-soon.

### 4. Cross-links present — PASS

```
tutorial → mental-model:                    2
tutorial → benchmarks:                      1
troubleshooting → git-layer:                3
troubleshooting → testing-targets:          2
connector guide → trust-model:              3
connector guide → testing-targets:          1
integrate guide → reposix-agent-flow skill: 3
integrate guide → trust-model:              3
simulator → tutorial:                       3
simulator → trust-model:                    3
```

Every required cross-link is present. The five files form a tight web rather than a star — each guide links forward to the tutorial and the trust model, and back to the others.

### 5. P1 — `replace` count = 0 in all five new files — PASS

```
0 docs/tutorials/first-run.md
0 docs/guides/write-your-own-connector.md
0 docs/guides/integrate-with-your-agent.md
0 docs/guides/troubleshooting.md
0 docs/reference/simulator.md
```

One incidental occurrence ("string-replace") in the integrate guide was caught and rewritten to "string-rewrite" pre-commit.

### 6. P2 — banned terms in tutorial + 3 guides = 0 — PASS

```
0 docs/tutorials/first-run.md
0 docs/guides/write-your-own-connector.md
0 docs/guides/integrate-with-your-agent.md
0 docs/guides/troubleshooting.md
```

Banned terms checked: `fuse`, `fusermount`, `kernel`, `syscall`, `daemon` (case-insensitive, word-boundary). The simulator reference (Layer 4) is exempt and was also checked; clean by coincidence.

## Out of scope (deferred — confirmed not regressed)

- `mkdocs build --strict` not run (Phase 45 owns it).
- `mkdocs.yml` nav not updated (Phase 43 owns it).
- `docs/connectors/guide.md` not redirected/deleted (Phase 43 owns it).
- `scripts/tutorial-runner.sh` end-to-end runner not written (deferred to Phase 45 per runner brief).
- Banned-words linter not wired to pre-commit/CI (Phase 43 owns it).
- Playwright screenshots not captured (Phase 45 owns it).

## Status

**passed** — all five files shipped, all cross-link and banned-word checks green. Phase 42 ready to hand off to Phase 43 (nav + linter).
