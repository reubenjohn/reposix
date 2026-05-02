← [back to index](./index.md) · phase 30 research

## Validation Architecture

This is the Nyquist gate's dimension for Phase 30. Tests/checks the phase must produce and gate on.

### Test Framework

| Property | Value |
|----------|-------|
| Primary gates | (a) `mkdocs build --strict`, (b) `vale --config=.vale.ini docs/`, (c) `doc-clarity-review` on rendered index, (d) playwright screenshots |
| Config files | `mkdocs.yml`, `.vale.ini`, `.vale-styles/Reposix/*.yml` |
| Quick-run (author loop) | `mkdocs serve` + local `vale docs/` |
| Full suite (CI) | `.github/workflows/docs.yml` — add Vale step before build; keep mkdocs build --strict; add doc-clarity-review + playwright as phase-gate (not CI) |
| Framework install | Vale: `curl + tar` (see Code Examples §Example 3) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| DOCS-01 | Value prop lands in 10 seconds | Cold-reader verdict | `claude -p "$(cat prompt.md)" docs/index.md` (via doc-clarity-review) with custom "state value prop in one sentence" prompt | ❌ Wave 0 — create prompt + run as phase-gate script |
| DOCS-01 | "replace" banned in hero copy | Lint | `vale --config=.vale.ini docs/index.md` | ❌ Wave 0 — create `.vale-styles/Reposix/NoReplace.yml` |
| DOCS-02 | Three how-it-works pages exist | Structural | `test -f docs/how-it-works/filesystem.md && test -f docs/how-it-works/git.md && test -f docs/how-it-works/trust-model.md` | ❌ Wave 0 — scripts/check_phase_30_structure.py |
| DOCS-02 | Each has one mermaid diagram | Content | `grep -c '```mermaid' docs/how-it-works/filesystem.md` each returns 1 | ❌ Wave 0 — include in structure check script |
| DOCS-02 | Diagrams render without error | Visual | playwright screenshot + manual review checklist per user CLAUDE.md OP #1 | Manual phase-gate |
| DOCS-03 | Mental-model page exists with three conceptual keys | Structural | `grep -c '^## ' docs/mental-model.md` returns exactly 3 (one per key) AND each matches key phrasings | ❌ Wave 0 — scripts/check_phase_30_structure.py |
| DOCS-03 | vs-mcp-sdks page exists and mentions "complement" | Structural | `grep -iE '^(complement|absorb|subsume)' docs/vs-mcp-sdks.md` | Same script |
| DOCS-04 | Three guides exist | Structural | file existence test | Same script |
| DOCS-05 | Simulator page is under reference/ not how-it-works/ | Nav | `grep -A2 'Reference:' mkdocs.yml \| grep simulator.md && ! grep how-it-works.*simulator` | Same script |
| DOCS-06 | Tutorial exists and runs against simulator | Structural + manual run | file existence + `bash scripts/test_phase_30_tutorial.sh` (runs all numbered commands and asserts they exit 0) | ❌ Wave 0 — consider pytest-style harness for tutorial |
| DOCS-07 | Nav restructured | Structural | compare `grep '  -' mkdocs.yml` before/after; validate against source-of-truth IA sketch | Same script |
| DOCS-07 | Banned terms not above Layer 3 | Lint | `vale --config=.vale.ini docs/` with `[docs/how-it-works/**]` opt-out | Vale rule |
| DOCS-08 | Theme tuned | Config | `grep -c 'social' mkdocs.yml` ≥ 1; `grep -c 'navigation.footer' mkdocs.yml` ≥ 1 | Same script |
| DOCS-09 | Linter runs on every commit | Hook | `scripts/hooks/test-pre-commit-docs.sh` stages a doc with a banned word, commits, asserts reject | ❌ Wave 0 — per OP #4 promote ad-hoc bash to script |
| ALL | mkdocs build green | Build | `mkdocs build --strict` exit 0 | Existing — runs in `.github/workflows/docs.yml` |
| ALL | CI green | Build | `gh run view` green on post-push commit | Existing + augmented with Vale step |

### Sampling Rate

- **Per doc commit (author loop):** `vale docs/` on changed files + `mkdocs serve` visual check.
- **Per wave merge:** `mkdocs build --strict` + full Vale lint.
- **Phase gate before `/gsd-verify-work`:** 
  - `mkdocs build --strict` green.
  - `vale --config=.vale.ini docs/` green.
  - `doc-clarity-review` on rendered `docs/index.md` returns LANDED.
  - Playwright screenshots (desktop 1280 + mobile 375) for all new/modified pages exist in `docs/screenshots/phase-30/`.
  - `gh run view` shows green CI on the milestone commit.
  - `scripts/check_phase_30_structure.py` (or equivalent) exits 0.
  - CHANGELOG entry exists for v0.9.0.

### Wave 0 Gaps (test infrastructure to ship in this phase)

- [ ] `.vale.ini` + `.vale-styles/Reposix/{ProgressiveDisclosure,NoReplace}.yml` + `.vale-styles/config/vocabularies/Reposix/accept.txt` — linter config.
- [ ] `scripts/hooks/pre-commit-docs` — pre-commit hook.
- [ ] `scripts/hooks/test-pre-commit-docs.sh` — pytest-style test that stages a bad doc, confirms hook rejects.
- [ ] `scripts/check_phase_30_structure.py` (pytest or shell) — structural invariants (pages exist, nav has them, three mermaid diagrams present, "replace" not in index.md, "FUSE" not above Layer 3).
- [ ] `scripts/test_phase_30_tutorial.sh` — runs the tutorial commands end-to-end against simulator. (Promote the ad-hoc bash per OP #4.)
- [ ] Phase-gate script that invokes `doc-clarity-review` on `docs/index.md` with the purpose-built "state the value prop in one sentence" prompt and parses LANDED/PARTIAL/MISSED verdict.
- [ ] `.github/workflows/docs.yml` — add Vale install + lint step before `mkdocs build --strict`.

Without these, "pass" is unverifiable and human-review-dependent. With them, the phase gate is mechanical and repeatable.

## Security Domain

reposix is a lethal-trifecta project and every docs change must honor the accuracy-not-overselling rule from the project CLAUDE.md: *"any trust-model diagram must be accurate — don't oversell security claims."*

### Applicable ASVS Categories

| ASVS Category | Applies to this phase | Reason / Control |
|---------------|----------------------|------------------|
| V1 Architecture & Design | YES (docs reflect architecture) | The trust-model page MUST accurately describe the eight SG guardrails. No feature may be claimed as shipped unless `security.md` already lists it. |
| V2 Authentication | N/A | Phase is docs-only. |
| V3 Session Management | N/A | Same. |
| V4 Access Control | PARTIAL | Vale + mkdocs build steps run with standard `GITHUB_TOKEN`; no elevated secrets. |
| V5 Input Validation | YES (docs lint) | Vale IgnoredScopes prevents false-positives from code blocks — a "bad" code example with malicious YAML would be prose-linted but not executed. |
| V6 Cryptography | N/A | No crypto in docs. |
| V7 Error Handling | YES (linter error path) | Vale's error format must be CI-readable; use `level: error` not `warning` to ensure CI fails. |

### Known Threat Patterns (docs-specific)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Docs claim a security property that isn't actually shipped | Information Disclosure (false sense of security) | Cross-reference every trust-model claim against `docs/security.md` shipped-items list and `SG-*` code-level evidence. The carver subagent does NOT invent new claims. |
| A contributor adds "replace" or "FUSE" via a PR from a fork | Tampering of the hero copy | Vale runs in CI on PR, not just post-merge. |
| Playwright screenshots leak local filesystem paths or env vars | Information Disclosure | Use a clean `/tmp/reposix-demo-XXXXXX` dir for tutorial commands; screenshot the browser only, never the terminal with env set. |
| Tutorial instructs the reader to set `REPOSIX_ALLOWED_ORIGINS=*` | Lethal-trifecta leg 3 enablement | Tutorial explicitly uses `http://127.0.0.1:*` default (simulator-only); never instructs widening the allowlist. |

**Key callout for the planner:** the trust-model page is the single place where the project's security story is narrativized. It is also the single easiest place to oversell. The carver subagent MUST stick to the facts in `docs/security.md` + `docs/architecture.md` + `.planning/research/threat-model-and-critique.md` and MUST NOT add superlatives. The user's global CLAUDE.md principle #6 (ground-truth obsession) applies: if a guardrail isn't in `crates/*/src/` and a test asserting it, it doesn't go in the trust-model page.
