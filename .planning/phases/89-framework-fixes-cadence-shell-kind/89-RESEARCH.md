# Phase 89: Framework fixes — Research

**Researched:** 2026-05-08
**Domain:** Quality Gates framework (cadence registry + verifier kind dispatch + catalog schema + structure-dim linters + milestone-close ritual)
**Confidence:** HIGH (every claim cited file:line; runner code read end-to-end; existing patterns surveyed for all 6 REQ-IDs)

## Summary

P89 ships **5 framework deliverables** that every other v0.13.0-extension phase (P90–P97) consumes:

1. **`cadence: pre-release-real-backend`** — extends `VALID_CADENCES` tuple at `quality/runners/run.py:45-47` with env-gated semantics (REPOSIX_ALLOWED_ORIGINS + per-backend creds). Requires a small new module (`_realbackend.py`) per the `_freshness.py` precedent at `run.py:34`.
2. **`kind: shell-subprocess`** — a verifier *convention* (real-subprocess invocation against real backends + transcript artifact at `quality/reports/transcripts/`). The runner already dispatches on suffix (`.sh` vs `.py`) at `run.py:226-232`; no dispatch-table rewrite needed. The kind is enforced by catalog schema doc + verifier subagent grading.
3. **Milestone-close 9th probe** — a SLOT (verifier ships at P89, returns `NOT-VERIFIED` legitimately until P91+P92+P93+P94+P95 land the substrate). Defended against C7 self-licensing-deferral via `blast_radius: P0` + missing-script preserved as `NOT-VERIFIED` at `run.py:204-221`.
4. **Two new structure-dim linters** — `banned-production-tokens.sh` (NEW; mirrors `scripts/banned-words-lint.sh:107-141` pattern, scoped to `crates/**/*.rs`) + `deferral-pointer-linter.sh` (NEW; greps three regex patterns, cross-references `.planning/phases/N-*/PLAN*.md` existence).
5. **`claim_vs_assertion_audit` field** — required string ≥50 chars on every catalog row minted ≥ 2026-05-08; runner cross-check fires at `load_catalog()` (`run.py:72-81`) per Principle B (fail loud, structured).

**Primary recommendation:** Mint catalog rows FIRST (one commit), then implement in dependency order: linters (T2+T3, disjoint files, paralellizable) → cadence + module factor (T4) → kind + transcript (T5) → audit-field cross-check (T6) → 9th-probe SLOT (T7) → CLAUDE.md + verifier dispatch (T8).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Cadence enum extension | runner core (`run.py`) | catalog schema (PROTOCOL.md table) | `VALID_CADENCES` is the canonical declaration; PROTOCOL.md table is documentation that must mirror it |
| Real-backend env gating | new sibling module `_realbackend.py` | `run_row()` short-circuit | `run.py` is over its 350-line cap (currently 375 lines per `wc -l`); the `_freshness.py:1-50` precedent shows the factoring pattern |
| Verifier-kind enforcement | catalog schema doc | verifier subagent grading at phase close | Runner dispatches on script suffix not `kind` field; kind is a convention enforced by humans/subagents |
| Transcript artifact write | new shell helper sourced by `shell-subprocess` verifiers | `run.py write_artifact()` extension for `transcript_path` field | Verifier writes its own transcript; runner copies path into JSON artifact for verifier-subagent dereference |
| Banned-tokens scan | `quality/gates/structure/banned-production-tokens.sh` (NEW) | `freshness-invariants.json` catalog row | Scoped to `crates/**/*.rs`; sibling-not-extension to docs-only `scripts/banned-words-lint.sh` |
| Deferral-pointer cross-ref | `quality/gates/structure/deferral-pointer-linter.sh` (NEW) | `freshness-invariants.json` catalog row | Two-step grep + filesystem lookup; structure dimension is the canonical home for "no broken pointers" |
| `claim_vs_assertion_audit` schema field | `catalogs/README.md` schema table | `run.py load_catalog()` cross-check | Documentation is where the contract lives; runner enforces |
| Milestone-close 9th probe | catalog row at `agent-ux.json` | verdict TEMPLATE at `quality/dispatch/milestone-close-verdict.md` (NEW per Q-LOC-1 below) | Row makes probe executable + gradable; template instructs verifier subagent to read row's artifact |

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | Python 3 stdlib (`unittest`) for runner-side; bash for verifier scripts; existing tests at `quality/runners/test_freshness.py` + `test_freshness_synth.py` |
| Config file | none — `python3 -m unittest discover quality/runners` works as-is |
| Quick run command | `python3 -m unittest discover -s quality/runners -p "test_*.py"` |
| Full suite command | `python3 quality/runners/run.py --cadence pre-push` (exercises every catalog row tagged `pre-push`) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| RBF-FW-01 | `pre-release-real-backend` in `VALID_CADENCES` + env-gate short-circuit fires | unit | `python3 -m unittest quality.runners.test_realbackend` | ❌ Wave 0 |
| RBF-FW-01 | Catalog row tagged with new cadence runs only when env set | integration (catalog row) | `python3 quality/runners/run.py --cadence pre-release-real-backend` | rows ❌ Wave 0 |
| RBF-FW-02 | `shell-subprocess` worked-example verifier produces transcript | smoke | `bash quality/gates/agent-ux/shell-subprocess-example.sh && test -f quality/reports/transcripts/*.txt` | ❌ Wave 0 |
| RBF-FW-03 | 9th-probe verifier exists + executable + returns NOT-VERIFIED until substrate | smoke | `test -x quality/gates/agent-ux/milestone-close-vision-litmus.sh` | ❌ Wave 0 |
| RBF-FW-04 | Banned-tokens regex on `crates/**/*.rs` blocks `P\d+-\d+` outside tests/allowlist | mechanical | `bash quality/gates/structure/banned-production-tokens.sh` | ❌ Wave 0 |
| RBF-FW-05 | Deferral linter cross-references named phase's PLAN files | mechanical | `bash quality/gates/structure/deferral-pointer-linter.sh` | ❌ Wave 0 |
| RBF-FW-11 | Runner refuses catalog with row missing `claim_vs_assertion_audit` (date-cutoff gated) | unit | `python3 -m unittest quality.runners.test_audit_field` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `python3 -m unittest discover -s quality/runners` (≤2s)
- **Per wave merge:** `python3 quality/runners/run.py --cadence pre-push` (full structure-dim sweep, ≤30s)
- **Phase gate:** Pre-push runner GREEN + verifier subagent verdict at `quality/reports/verdicts/p89/VERDICT.md`

### Wave 0 Gaps

- [ ] `quality/runners/test_realbackend.py` — covers RBF-FW-01 env-gate semantics
- [ ] `quality/runners/test_audit_field.py` — covers RBF-FW-11 cross-check
- [ ] `quality/gates/structure/banned-production-tokens.sh` — RBF-FW-04 verifier
- [ ] `quality/gates/structure/deferral-pointer-linter.sh` — RBF-FW-05 verifier
- [ ] `quality/gates/agent-ux/shell-subprocess-example.sh` — RBF-FW-02 worked example
- [ ] `quality/gates/agent-ux/milestone-close-vision-litmus.sh` — RBF-FW-03 SLOT verifier
- [ ] `quality/dispatch/milestone-close-verdict.md` — verdict template (NEW per Q-LOC-1 below)
- [ ] 6 catalog rows minted NOT-VERIFIED (T1)
- [ ] No framework install needed — Python stdlib + bash already established

## Open Questions Resolved

### Q-LOC-1 — Milestone-close verdict TEMPLATE location
**Resolved:** Template does NOT exist today. Searched: `find /home/reuben/workspace/reposix -name "milestone-close*" -o -name "milestone-verdict*"` returned no template files. The closest existing artifact is `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (an artifact, not a template — see file lines 1-67; it documents 8 probes in the `## Re-verification probe results` table at lines 12-23). **Action:** P89 CREATES `quality/dispatch/milestone-close-verdict.md` (new file; parallels the future `quality/dispatch/milestone-adversarial.md` that P90 RBF-FW-12 will land per REMEDIATION-PLAN.md:191). The template is a markdown skeleton with 9 probe rows; P97's RBF-G-04 fills probe-9 results when it overwrites `milestone-v0.13.0/VERDICT.md` per Decision 4 option (b). The CONTEXT.md default in Q-LOC-1 was correct.

### Q-CADENCE-1 — Runner cadence model
**Resolved:** Flat `tuple` registry + `argparse choices=` validation. See `quality/runners/run.py:45-47`:
```python
VALID_CADENCES = (
    "pre-commit", "pre-push", "pre-pr", "weekly", "pre-release", "post-release", "on-demand",
)
```
…and `run.py:310`: `parser.add_argument("--cadence", required=True, choices=VALID_CADENCES)`. Extension is one tuple-line edit + module-docstring update at `run.py:11`. **No enum class, no string registry; CONTEXT D-01a is correct.** Confirms CD-02 from CONTEXT (keep flat enum).

### Q-SHELL-1 — Existing shell verifier interface
**Resolved:** Existing shell verifiers do NOT write JSON artifacts and do NOT emit structured transcripts. They print to stdout/stderr; the runner synthesizes the JSON artifact from `result.stdout` / `result.stderr` / `result.returncode` at `run.py:259-274`. Examples:
- `quality/gates/agent-ux/mirror-refs-write-on-success.sh:23-31` — runs `cargo test … | tail -20` then `echo "PASS: …" && exit 0`. No artifact write, no transcript.
- `quality/gates/agent-ux/reposix-attach.sh:38-53` — runs the binary, asserts via shell, exits 0/1.
- `quality/gates/structure/no-loose-top-level-planning-audits.sh:14-26` — pure stdlib bash; exits 0/1.

The runner WILL respect a pre-existing artifact JSON if the verifier wrote one (`run.py:260-264`: "If verifier wrote it, we keep its body and only annotate top-level metadata"). So **the transcript-artifact convention for `kind: shell-subprocess` is genuinely new** — verifiers MUST write the JSON artifact themselves AND a sibling transcript at `quality/reports/transcripts/<row-slug>-<ts>.txt`. Recommend factoring the transcript-write into a shared helper at `quality/gates/agent-ux/lib/transcript.sh` (mirrors `quality/gates/agent-ux/dark-factory/lib.sh` precedent).

### Q-BANNED-1 — Existing banned-words scope
**Resolved:** `quality/gates/structure/banned-words.sh:1-7` is a 7-line wrapper that delegates to `scripts/banned-words-lint.sh --all`. The canonical impl at `scripts/banned-words-lint.sh:32-75` hardcodes `DOCS_ROOT="${REPO_ROOT}/docs"` and three layered glob arrays (LAYER1_PATHS=`index.md`; LAYER2_GLOBS=`concepts tutorials guides`; LAYER3_GLOBS=`how-it-works`). Allowlist marker is `<!-- banned-words: ok -->` (line 29; markdown comment shape — won't work in Rust source). **Conclusion:** CONTEXT D-04a is correct — extending the docs-layered linter to `crates/` would conflate scopes and break its model. **Mint a NEW sibling script** `quality/gates/structure/banned-production-tokens.sh` with a Rust-native allowlist marker (`// banned-words: ok` per CONTEXT D-04b).

### Q-DEFERRAL-1 — Existing `not yet wired in PNN` strings in crates/
**Resolved:** Sample search confirmed two production-source matches:

```
crates/reposix-cli/src/attach.rs:163  "attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); \
crates/reposix-cli/src/sync.rs:42     /// - The backend slug isn't `sim` (real-backend wiring lands in P82+).
```

**Both legitimately exist today** — they're the very strings P91 (RBF-A-03) is committed to scrubbing. The deferral-pointer linter MUST resolve `P79-02` → `.planning/phases/79-*/` (exists; PLAN files present) → PASS. After P91 ships and the strings are removed, the linter has nothing to scan; the gate stays GREEN. **False-positive risk is LOW** because the linter only fires on the named pattern AND requires the named phase dir to be MISSING. Sample size (2) is small enough to manually verify both during P89 implementation.

Additional matches with `P\d+-\d+` shape (banned-production-tokens scope, NOT deferral-pointer scope):
- `crates/reposix-remote/src/main.rs:439` — `// Narrow-deps signature (P83-01 T02 refactor):` — comment, would BLOCK under default RBF-FW-04 unless allowlist marker added. **Owner choice:** either move historical refactor markers to CHANGELOG.md or add `// banned-words: ok` allowlist comment. Recommend the allowlist in P89 implementation since these are post-hoc archaeology, not active deferrals.
- `crates/reposix-core/src/error.rs:54,81` — `code-quality audit P1-1` — false positive on `P1-1`. The regex `\bP[0-9]+-[0-9]+\b` matches. **Mitigation:** scope the regex more tightly to `P\d{2,3}-\d+` (require ≥2 digits in phase number) OR add allowlist markers. Recommend the digit-count tightening since `P1-1` / `P0-2` / `P1-5` etc. are code-quality audit IDs that legitimately persist.
- `crates/reposix-cache/src/db.rs:97` + `fixtures/cache_schema.sql:22,24,26,76,89,91` + `crates/reposix-remote/src/bus_handler.rs:25,112,222` — **all comments, all P79-02/P80-01/P83-01-style.** Same call: tighten regex to `P\d{2,3}-\d+` or add allowlist markers.

**Strong recommendation:** Tighten RBF-FW-04 regex to `\bP\d{2,3}-\d+\b` (matches `P79-02`, `P83-01`; misses `P1-1`, `P0-2`). The original CONTEXT D-04c regex `\bP[0-9]+-[0-9]+\b` would block ~10 legitimate code-quality audit ID references on first run; the planner should override CONTEXT D-04c here.

CHANGELOG.md files in `crates/*/CHANGELOG.md` (e.g., `crates/reposix-confluence/CHANGELOG.md:65,66`) also contain matches. **Recommend excluding `**/CHANGELOG.md` from the linter scope** — these are historical records by definition.

### Q-AUDIT-1 — Catalog schema validation today
**Resolved:** The runner does NOT validate row schemas beyond wrapper structure (`run.py:72-81` only checks `dimension` + `rows` keys exist + JSON parses). There's NO JSON Schema validator; rows are duck-typed. The `$schema` field at every catalog file's top (e.g., `agent-ux.json:2`) is `"https://json-schema.org/draft-07/schema#"` but no actual schema is referenced. **Implementation path for RBF-FW-11:** extend `load_catalog()` at `run.py:72-81` (or factor into `_audit_field.py` per CONTEXT D-11c). Stdlib-only; check `last_verified` ISO date >= `2026-05-08T00:00:00Z` (or null) AND `claim_vs_assertion_audit` is str + non-empty + `len(s) >= 50`. Raise `SystemExit(f"FAIL: {path}: row {row.id} missing claim_vs_assertion_audit (≥50 chars required for rows minted on/after 2026-05-08)")`.

### Q-VISION-1 — Vision litmus test definition
**Resolved:** The runnable scenario lives at `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh` (P86 invariant; dispatched via `quality/gates/agent-ux/dark-factory.sh:40-42`). The vision-and-mental-model document at `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` is the definitional source; the executable shape lives in dark-factory. **The 9th-probe verifier MUST mirror dvcs-third-arm's shape** (vanilla-clone + `reposix attach` + edit + `git push`) **but fire against TokenWorld** (Confluence) instead of in-process sim. Until P91 ships real-backend `attach`, the verifier legitimately exits with status NOT-VERIFIED. CONTEXT D-03b assertions list (4 invariants) is correct.

### Q-CATALOG-DIM-1 — `framework.json` vs existing dimensions
**Resolved:** CLAUDE.md "Quality Gates" § "9 dimensions" table lists exactly: `code, docs-alignment, docs-build, docs-repro, release, structure, agent-ux, perf, security`. There is NO `framework` dimension. The `quality/catalogs/` directory has these dimension files: `agent-ux.json, code.json, cross-platform.json, doc-alignment.json, docs-build.json, docs-reproducible.json, freshness-invariants.json, perf-targets.json, release-assets.json, security-gates.json, subjective-rubrics.json`. **Note:** `freshness-invariants.json` is the `structure`-dimension catalog (its wrapper has `"dimension": "structure"`); there is no `structure.json`. **Conclusion:** CONTEXT D-CAT-03 is correct — DO NOT create `framework.json`. Mint:
- 3 rows (RBF-FW-01, RBF-FW-02, RBF-FW-03) → `quality/catalogs/agent-ux.json`
- 3 rows (RBF-FW-04, RBF-FW-05, RBF-FW-11) → `quality/catalogs/freshness-invariants.json`

The ROADMAP § Phase 89 SC #5 mention of `{agent-ux,framework}.json` is conventional shorthand; literal compliance would require a schema migration that bloats P89's scope. The planner should override the ROADMAP wording here with the dimension-respecting layout.

### Q-VERDICT-OVERLAY — How does milestone-v0.13.0/VERDICT.md get extended?
**Resolved:** The existing verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is **free-form prose with two markdown tables** (probe results lines 12-23 + per-phase verdicts lines 27-39). It is NOT a structured/sectioned format. **For RBF-FW-03's 9th probe to be addable structurally**, the new verdict TEMPLATE at `quality/dispatch/milestone-close-verdict.md` (per Q-LOC-1 resolution) should DEFINE the structure: a 9-row probe table that any milestone-close ritual fills in. P97's RBF-G-04 will use this template to overwrite the existing VERDICT.md per Decision 4 option (b) (per `_archive/DECISIONS-NEEDED.md:42-44`). For P95's RBF-S-05, the existing verdict gets a 2026-05-08 banner overlay; the structural extension waits for P97. **P89 ships ONLY the template; it does NOT touch the existing VERDICT.md.**

### Q-RUNNER-CROSS-CHECK — When does claim_vs_assertion cross-check fire?
**Resolved:** **At catalog-load time**, NOT at row-grade time. Rationale:
1. The runner's "fail loud" principle (`PROTOCOL.md` § Principle B, line 47) says preconditions assert before work — load-time validation matches that.
2. `load_catalog()` at `run.py:72-81` already raises `SystemExit` on schema violations — the new check fits there as one more validation.
3. Failing at row-grade time means partial-success runs (some rows graded, some failing schema check) — bad UX.
4. Per-row gating gives the operator a clear error: `FAIL: {path}: row {row.id} missing claim_vs_assertion_audit`.

**Cleanest hook:** add a function `_validate_row_audit_field(row, path)` called from `load_catalog()` after the wrapper-validation lines (after `run.py:81`). For each row in `data["rows"]`, check the cutoff-date condition + field presence + length. Use `parse_rfc3339()` (already exists at `run.py:55-59`) to compare `last_verified` against `2026-05-08T00:00:00Z`. Rows with `last_verified is None` are also subject to the check (newly-minted rows). Pre-existing P78–P88 rows with `last_verified` from before the cutoff PASS the check unconditionally — backfill is P95 RBF-D-06's job per CONTEXT spec_lock.

## File Anchors

### RBF-FW-01 — `cadence: pre-release-real-backend`
| Touch point | Evidence |
|---|---|
| `quality/runners/run.py:11` | Module docstring "The 7 cadences are…" — bump to 8 |
| `quality/runners/run.py:45-47` | `VALID_CADENCES` tuple — add `"pre-release-real-backend"` |
| `quality/runners/_realbackend.py` (NEW) | Env-gate logic per `_freshness.py:1-50` precedent; ~30-50 lines |
| `quality/runners/run.py:153-199` | `run_row()` — call `_realbackend.is_skipped(row, env)` BEFORE the WAIVED case (line 163); mirror the `_stale=True` transient flag pattern (line 197) using `_skipped_real_backend=True` |
| `quality/runners/run.py:297-305` | `print_row_summary()` — extend the `_stale` label branch to also recognize `_skipped_real_backend` |
| `quality/PROTOCOL.md:140-148` | Latency budgets table — add row for `pre-release-real-backend` |
| `CLAUDE.md` § "Quality Gates" 7 cadences table | Extend to 8 cadences (per D-CLM-01) |

### RBF-FW-02 — `kind: shell-subprocess`
| Touch point | Evidence |
|---|---|
| `quality/catalogs/README.md:27` | Schema kind enum — add `shell-subprocess` to the list |
| `quality/gates/agent-ux/lib/transcript.sh` (NEW, recommended) | Shared bash helper for transcript-write; mirrors `quality/gates/agent-ux/dark-factory/lib.sh` factoring shape |
| `quality/gates/agent-ux/shell-subprocess-example.sh` (NEW) | Worked example: invokes `reposix --version` as subprocess; writes JSON artifact + transcript |
| `quality/reports/transcripts/` (NEW dir) | gitkeep; transcript files named `<row-slug>-<RFC3339>.txt` |
| `quality/runners/run.py:259-274` | `run_row()` — extend artifact synthesis to read `transcript_path` from verifier-written JSON if present, copy into top-level artifact field |
| `quality/PROTOCOL.md` § "Verifier subagent prompt template" (line 267) | Add bullet: "if row.kind == 'shell-subprocess', dereference artifact.transcript_path and verify subprocess argv + env_keys + exit_code" |
| `CLAUDE.md` § "Quality Gates" 5 kinds table | Extend to 6 kinds (per D-CLM-01) |

### RBF-FW-03 — Milestone-close 9th probe
| Touch point | Evidence |
|---|---|
| `quality/dispatch/milestone-close-verdict.md` (NEW; per Q-LOC-1) | Verdict template skeleton: 9-probe table; instructs verifier subagent to read row's artifact + transcript |
| `quality/gates/agent-ux/milestone-close-vision-litmus.sh` (NEW) | SLOT verifier — exits 0 with NOT-VERIFIED status until P91+ substrate lands; writes its own artifact JSON with `status: NOT-VERIFIED, reason: substrate_not_landed` |
| `quality/catalogs/agent-ux.json` | NEW row `agent-ux/milestone-close-vision-litmus-real-backend` tagged `cadences: ["pre-release-real-backend"], kind: shell-subprocess, blast_radius: P0` |
| `quality/PROTOCOL.md` § "Per-phase protocol" Step 6 | Add note: "milestone-close ritual MUST also invoke `--cadence pre-release-real-backend` and require exit 0" |
| `CLAUDE.md` § "Subagent delegation rules" | New bullet per D-CLM-03 |

### RBF-FW-04 — Banned production-error tokens
| Touch point | Evidence |
|---|---|
| `quality/gates/structure/banned-production-tokens.sh` (NEW) | Mirrors `scripts/banned-words-lint.sh:107-141` ERE alternation + allowlist marker pattern; scoped to `crates/**/*.rs`; allowlist marker `// banned-words: ok` |
| `quality/catalogs/freshness-invariants.json` | NEW row `structure/banned-production-tokens`, `cadences: ["pre-commit", "pre-push", "pre-pr"], blast_radius: P1` |
| `crates/reposix-cli/src/attach.rs:163-164` (FUTURE — P91 RBF-A-03) | Where the production violation lives today; documented worked-example for the gate |
| `crates/reposix-cli/src/sync.rs:42` (FUTURE — P91) | Second production violation for worked-example |

### RBF-FW-05 — Deferral-pointer linter
| Touch point | Evidence |
|---|---|
| `quality/gates/structure/deferral-pointer-linter.sh` (NEW) | Three regex patterns per F-K6 verbatim: `not yet wired in P\d+`, `lands? (alongside\|in) P\d+`, `substrate-gap-deferred` |
| `quality/catalogs/freshness-invariants.json` | NEW row `structure/deferral-pointer-linter`, `cadences: ["pre-push"], blast_radius: P1` |
| `.planning/phases/N-*/PLAN*.md` (resolved at runtime) | Linter resolves named phase number against this glob; fails if no PLAN files exist |
| `crates/reposix-cli/src/attach.rs:163` | Today's only "not yet wired in P79-02" string; PASSES because P79's PLAN files exist |

### RBF-FW-11 — `claim_vs_assertion_audit` field
| Touch point | Evidence |
|---|---|
| `quality/catalogs/README.md:22-41` | Schema table — add `claim_vs_assertion_audit` row |
| `quality/runners/_audit_field.py` (NEW) | `validate_row_audit_field(row, path)` — date-cutoff + non-empty + ≥50 chars |
| `quality/runners/run.py:72-81` | `load_catalog()` — call `_audit_field.validate_row_audit_field()` for each row in `data["rows"]` after wrapper validation |
| `quality/runners/run.py:148-150` (`write_artifact`) | Optionally compute `sha256(claim_vs_assertion_audit)` and include in artifact as `claim_vs_assertion_audit_hash` (D-11d, "ship in P89") |

### Catalog-first commit (T1)
| Catalog file | Rows added (count) |
|---|---|
| `quality/catalogs/agent-ux.json` | 3: cadence row, kind row, 9th-probe row |
| `quality/catalogs/freshness-invariants.json` | 3: banned-tokens row, deferral-pointer row, audit-field row |

## Implementation Patterns

### RBF-FW-01 — Cadence extension pattern

**Mirror:** `quality/runners/_freshness.py:1-50` (sibling-module factoring) + `run.py:181-199` (STALE-case short-circuit shape).

```python
# quality/runners/_realbackend.py (NEW)
import os
import re
from typing import Iterable

_CRED_SETS = {
    "confluence": ("ATLASSIAN_API_KEY", "ATLASSIAN_EMAIL", "REPOSIX_CONFLUENCE_TENANT"),
    "github": ("GITHUB_TOKEN",),
    "jira": ("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE"),
}
_NON_LOCAL_RE = re.compile(r"^https?://(?!127\.0\.0\.1)")

def is_skipped(row: dict, env: os._Environ) -> bool:
    """True iff row is tagged pre-release-real-backend AND env not configured."""
    if "pre-release-real-backend" not in row.get("cadences", []):
        return False
    origins = env.get("REPOSIX_ALLOWED_ORIGINS", "")
    if not origins or not _NON_LOCAL_RE.search(origins):
        return True
    return not any(all(env.get(k) for k in keys) for keys in _CRED_SETS.values())
```

In `run.py:163` (before WAIVED case), after `started = time.monotonic()`:
```python
if _realbackend.is_skipped(row, os.environ):
    artifact = {"ts": now_iso(), "row_id": row["id"], "exit_code": None,
                "skipped_real_backend": True, "asserts_passed": [],
                "asserts_failed": ["env not set: need REPOSIX_ALLOWED_ORIGINS + creds"]}
    if artifact_path: write_artifact(artifact_path, artifact)
    row["status"] = "NOT-VERIFIED"
    row["_skipped_real_backend"] = True
    row["last_verified"] = artifact["ts"]
    return row, time.monotonic() - started
```

### RBF-FW-02 — Shell-subprocess + transcript pattern

**Mirror:** `quality/gates/agent-ux/dark-factory/lib.sh` (shared helper factoring) + `quality/gates/agent-ux/reposix-attach.sh:38-53` (subprocess invocation).

```bash
# quality/gates/agent-ux/lib/transcript.sh (NEW)
# write_transcript_and_artifact <row-slug> <argv...>
write_transcript_and_artifact() {
  local slug="$1"; shift
  local ts=$(date -u +%Y-%m-%dT%H-%M-%SZ)
  local transcript="quality/reports/transcripts/${slug}-${ts}.txt"
  local artifact="quality/reports/verifications/agent-ux/${slug}.json"
  mkdir -p "$(dirname "$transcript")" "$(dirname "$artifact")"

  local stdout stderr exit_code env_keys
  env_keys=$(env | cut -d= -f1 | sort | tr '\n' ',' | sed 's/,$//')
  stdout=$(mktemp); stderr=$(mktemp)
  set +e
  "$@" >"$stdout" 2>"$stderr"
  exit_code=$?
  set -e

  {
    printf 'argv: %s\n' "$*"
    printf 'env_keys: %s\n' "$env_keys"
    printf 'cwd: %s\n' "$(pwd)"
    printf 'exit_code: %s\n' "$exit_code"
    printf -- '--- STDOUT ---\n'
    cat "$stdout"
    printf -- '--- STDERR ---\n'
    cat "$stderr"
  } > "$transcript"

  cat > "$artifact" <<EOF
{"ts":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","row_id":"agent-ux/${slug}","exit_code":${exit_code},"transcript_path":"${transcript}","asserts_passed":[],"asserts_failed":[]}
EOF
  rm -f "$stdout" "$stderr"
  return $exit_code
}
```

### RBF-FW-03 — SLOT verifier pattern

**Mirror:** CONTEXT.md `<specifics>` section worked example + `quality/gates/structure/no-loose-top-level-planning-audits.sh:14-26` (structured failure exit). The runner does NOT support exit-75 → NOT-VERIFIED mapping (per Q-EXIT-1 / `run.py:276-283`); the verifier MUST write the artifact directly.

```bash
#!/usr/bin/env bash
# quality/gates/agent-ux/milestone-close-vision-litmus.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json"
mkdir -p "$(dirname "$ARTIFACT")"

if [[ -z "${REPOSIX_ALLOWED_ORIGINS:-}" || -z "${ATLASSIAN_API_KEY:-}" ]]; then
  cat > "$ARTIFACT" <<EOF
{"ts":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","row_id":"agent-ux/milestone-close-vision-litmus-real-backend","exit_code":null,"skipped_real_backend":true,"asserts_passed":[],"asserts_failed":["env not set"]}
EOF
  echo "SKIP: real-backend env not set" >&2
  exit 1  # runner maps to FAIL → NOT-VERIFIED via blast_radius gating
fi
# TODO P91-P95: invoke real-backend dvcs-third-arm against TokenWorld
echo "NOT-VERIFIED: substrate not landed (depends on P91+P92+P93+P94+P95)" >&2
exit 1
```

**Important deviation from CONTEXT specifics:** The runner does not have an exit-75 mapping; exit ≠ 0 maps to FAIL (`run.py:283`). Combine with the env-gate skip from RBF-FW-01 (which DOES set NOT-VERIFIED via the new `_realbackend.is_skipped` short-circuit) and the verifier reaches the runner only when env IS set. So the SLOT shape is: tagged with `cadences: ["pre-release-real-backend"]` → without env, the new RBF-FW-01 short-circuit returns NOT-VERIFIED before invoking the script; with env, the script runs and (until P91+) returns FAIL with explanatory artifact. Either way, milestone-close grading of this row produces "not green," which is the desired SLOT semantics.

### RBF-FW-04 — Banned tokens (Rust-source-aware)

**Mirror:** `scripts/banned-words-lint.sh:107-141` `scan_files()` ERE alternation + allowlist filter pattern, but scoped to `crates/` and Rust-style allowlist marker.

```bash
#!/usr/bin/env bash
# quality/gates/structure/banned-production-tokens.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ALLOWLIST_MARKER='// banned-words: ok'
PATTERN='\bP[0-9]{2,3}-[0-9]+\b'  # tightened from CONTEXT D-04c per Q-DEFERRAL-1 finding

violations=0
while IFS= read -r -d '' file; do
  while IFS=: read -r path lineno content; do
    [[ "$content" == *"$ALLOWLIST_MARKER"* ]] && continue
    printf '✖ banned production token: %s:%s: %s\n' "$path" "$lineno" "$content" >&2
    violations=1
  done < <(grep -nHE "$PATTERN" "$file" 2>/dev/null || true)
done < <(find crates -type f -name '*.rs' \
  ! -path '*/tests/*' ! -path '*/target/*' ! -name 'CHANGELOG.md' -print0)

[[ $violations -eq 1 ]] && {
  echo "owner_hint: rename or remove the phase-id token; or add '$ALLOWLIST_MARKER' to the line" >&2
  exit 1
}
echo "PASS: no banned production-error tokens in crates/**/*.rs"
```

**Deviation from CONTEXT D-04c:** Tightened regex to `P\d{2,3}-\d+` to avoid blocking `P1-1` / `P0-2` code-quality audit IDs. CHANGELOG.md files explicitly excluded (historical record). This is a planner override of a CONTEXT default that would generate ~10 false positives on first run.

### RBF-FW-05 — Deferral-pointer cross-reference

```bash
#!/usr/bin/env bash
# quality/gates/structure/deferral-pointer-linter.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

PATTERNS=('not yet wired in P[0-9]+' 'lands? (alongside|in) P[0-9]+' 'substrate-gap-deferred')
violations=0

for pat in "${PATTERNS[@]}"; do
  while IFS=: read -r path lineno content; do
    # Extract phase number(s) from the matched line
    while IFS= read -r N; do
      [[ -z "$N" ]] && continue
      if ! ls .planning/phases/${N}-*/*PLAN*.md >/dev/null 2>&1; then
        printf '✖ %s:%s references P%s but no .planning/phases/%s-*/*PLAN*.md exists\n' "$path" "$lineno" "$N" "$N" >&2
        violations=1
      fi
    done < <(echo "$content" | grep -oE 'P[0-9]+' | sed 's/^P//')
  done < <(grep -rnHE "$pat" crates/ 2>/dev/null || true)
done

[[ $violations -eq 1 ]] && exit 1
echo "PASS: every deferral-pointer in crates/ resolves to an existing phase dir with PLAN files"
```

### RBF-FW-11 — Claim-vs-assertion audit field

**Mirror:** `_freshness.py` factoring + `load_catalog()` SystemExit pattern (`run.py:77`).

```python
# quality/runners/_audit_field.py (NEW)
from datetime import datetime, timezone

CUTOFF_ISO = "2026-05-08T00:00:00+00:00"
MIN_LEN = 50

def validate_row(row: dict, catalog_path: str, parse_rfc3339) -> None:
    """Raise SystemExit if row minted ≥ cutoff lacks claim_vs_assertion_audit."""
    lv = row.get("last_verified")
    cutoff = parse_rfc3339(CUTOFF_ISO)
    # Rows with last_verified=None (newly minted) AND rows verified on/after cutoff
    is_new = lv is None or parse_rfc3339(lv) >= cutoff
    if not is_new:
        return
    audit = row.get("claim_vs_assertion_audit")
    if not isinstance(audit, str) or len(audit.strip()) < MIN_LEN:
        raise SystemExit(
            f"FAIL: {catalog_path}: row {row.get('id', '?')} missing "
            f"claim_vs_assertion_audit (≥{MIN_LEN} chars required for rows minted on/after {CUTOFF_ISO})"
        )
```

Hooked into `load_catalog()` at `run.py:81` (after wrapper-key check):
```python
from _audit_field import validate_row as _validate_audit_field
for row in data.get("rows", []):
    _validate_audit_field(row, str(path), parse_rfc3339)
```

## Risks and Watchouts

### False-positive risk: RBF-FW-04 banned-tokens
**Severity:** HIGH if regex is `P\d+-\d+` as-written in CONTEXT D-04c.
**Cause:** Existing legitimate `P1-1`, `P0-2`, `P1-5` code-quality audit IDs in `crates/reposix-core/src/error.rs:54,81` and `crates/reposix-core/src/backend/sim.rs:610`.
**Mitigation:** Tighten regex to `P\d{2,3}-\d+` (planner override of CONTEXT default per Q-DEFERRAL-1 finding). Also exclude `**/CHANGELOG.md` and `**/tests/**`.
**Residual:** Even with tightening, `P79-02 scaffold` strings in `attach.rs:163` and `bus_handler.rs:25,112,222` (P83-01 refactor markers) remain. The attach.rs string is the intended catch (P91 fixes it). The bus_handler.rs strings are post-hoc archaeology — recommend allowlist markers OR scrub them in P89's same PR if owner agrees.

### False-positive risk: RBF-FW-05 deferral-pointer linter
**Severity:** LOW.
**Cause:** Only 2 production-source matches today (verified by `grep -rnHE 'not yet wired in P\|substrate-gap-deferred\|lands? (alongside|in) P' crates/`); both resolve to existing phase dirs.
**Watchout:** If phase numbers get renumbered (e.g., during a milestone re-org), existing references could break en masse. Recommend documenting "phase-number-renaming requires updating any deferral-pointer references in crates/" in CLAUDE.md or PROTOCOL.md.

### Cache-warming risk: RBF-FW-01 new cadence
**Severity:** MEDIUM.
**Cause:** First operator to run `--cadence pre-release-real-backend` against TokenWorld will pay the cache-warming cost (full `list_records` walk + blob materialization). Per CLAUDE.md "Build memory budget", this could push memory pressure on a machine simultaneously running cargo work.
**Mitigation:** Document in PROTOCOL.md latency budget row that this cadence is "first-run heavy; subsequent runs benefit from cache." Recommend running it in a dedicated session window with no concurrent cargo.

### Backward-compat risk: RBF-FW-11 schema field
**Severity:** LOW (mitigated by date-cutoff design).
**Cause:** Adding a required field would normally break the 388 existing P78–P88 rows (per CLAUDE.md `freshness-invariants` references — "388-row catalog was migrated in place").
**Mitigation:** The cutoff-date condition (`last_verified >= 2026-05-08T00:00:00Z` OR `last_verified is None`) means legacy rows are exempt. P95 RBF-D-06 backfills them later. The runner cross-check is implemented as a pre-existing-rows-tolerant check.
**Watchout:** Any P89 catalog row whose `last_verified` is hand-set to a pre-cutoff date to dodge the check would constitute a process violation; the verifier subagent should spot-check this at phase close. Consider adding a structure-dim catalog row in P89 itself to assert: "no row has `last_verified < 2026-05-08T00:00:00Z` AND `_minted_in: P89` " — but this borders on over-engineering; defer to P95 if the failure mode actually surfaces.

### Self-licensing-deferral-loop (PATTERNS C7) risk: RBF-FW-03 SLOT
**Severity:** MEDIUM (called out in CONTEXT D-03c but worth re-flagging).
**Cause:** A SLOT verifier that legitimately returns NOT-VERIFIED could become a "WAIVED-by-default" anti-pattern if not policed.
**Mitigation:** (a) `blast_radius: P0` on the row so any milestone-close grading attempt with the row NOT-VERIFIED returns exit 1; (b) NEVER use a `waiver` block on this row (waivers are the explicit rejection of SLOT); (c) verifier subagent at P89-close should explicitly check that the row's claim_vs_assertion_audit paragraph names "this row WILL flip to PASS only when P91+P92+P93+P94+P95 all ship their substrate" — making the deferral self-falsifying.

### Scope-creep risk: RBF-FW-02 `kind` enforcement
**Severity:** MEDIUM.
**Cause:** "kind" is currently a documentation convention, not runtime-enforced. Adding `shell-subprocess` to the kind enum doesn't actually prevent a row from declaring `kind: shell-subprocess` while pointing at a `.py` script. Real enforcement is a P90 concern (RBF-FW-08 wires `dispatch.sh` requirement; could similarly add `kind: shell-subprocess` requires script suffix `.sh` AND transcript-path emission).
**Mitigation:** P89's worked-example verifier proves the kind works end-to-end; structural enforcement deferred to P90. Document this boundary in CLAUDE.md per CONTEXT D-CLM-01..04.

## Recommended Task Decomposition

| Order | Task | REQ-IDs | Effort | Dependencies | Notes |
|---|---|---|---|---|---|
| **T1** | Catalog-first commit: mint 6 NOT-VERIFIED rows in `agent-ux.json` (3) + `freshness-invariants.json` (3); each row carries `claim_vs_assertion_audit` ≥50 chars | All 6 | XS (1-2h) | — | First commit per `PROTOCOL.md` Step 3. All subsequent commits cite a row id. Eats own dogfood (RBF-FW-11 rows include the field). |
| **T2** | RBF-FW-04 — Banned-production-tokens linter: NEW `quality/gates/structure/banned-production-tokens.sh` with tightened regex `P\d{2,3}-\d+` and `**/CHANGELOG.md` exclusion | RBF-FW-04 | XS (2h) | T1 | Quick-win; validates the catalog-first contract. Disjoint from T3. Do NOT extend `scripts/banned-words-lint.sh` per Q-BANNED-1. |
| **T3** | RBF-FW-05 — Deferral-pointer linter: NEW `quality/gates/structure/deferral-pointer-linter.sh` with three regex patterns + phase-dir existence check | RBF-FW-05 | S (3h) | T1 | Disjoint from T2 — can run in parallel as separate sub-subagent dispatch. Existence-only check per Q-DEFLINK-1 default. |
| **T4** | RBF-FW-01 — `pre-release-real-backend` cadence: extend `VALID_CADENCES`, factor env-gate into `_realbackend.py`, mirror `_freshness.py:1-50` factoring, extend `print_row_summary`, update PROTOCOL.md latency table | RBF-FW-01 | M (4-5h) | T1 | Touches `run.py` — sequence after T2/T3 to avoid merge conflicts on the catalog file. Add `quality/runners/test_realbackend.py` unit test. |
| **T5** | RBF-FW-02 — `kind: shell-subprocess` + transcript convention: extend `catalogs/README.md` schema, ship `quality/gates/agent-ux/lib/transcript.sh` shared helper, ship `quality/gates/agent-ux/shell-subprocess-example.sh` worked example, extend `run_row()` to copy `transcript_path` into top-level artifact | RBF-FW-02 | M (4-5h) | T4 | Touches same `run.py` extension surface; sequence after T4. Worked example MUST exercise the convention against the local binary (sim path; works in CI). |
| **T6** | RBF-FW-11 — `claim_vs_assertion_audit` field + runner cross-check: extend `catalogs/README.md` schema table, ship `quality/runners/_audit_field.py`, hook into `load_catalog()`, add `claim_vs_assertion_audit_hash` artifact field, add `quality/runners/test_audit_field.py` | RBF-FW-11 | S (3h) | T5 | Touches `load_catalog()` — sequence after T4/T5. Verifier subagent at phase close MUST grade T1's 6 rows against the new check (eating dogfood). |
| **T7** | RBF-FW-03 — Milestone-close 9th-probe SLOT: NEW `quality/dispatch/milestone-close-verdict.md` template, NEW `quality/gates/agent-ux/milestone-close-vision-litmus.sh` SLOT verifier (returns NOT-VERIFIED honestly), document P97 future-coupling in PROTOCOL.md | RBF-FW-03 | S (3h) | T1, T4 (cadence must exist for the row to validate) | Independent of T2/T3/T5/T6 file surfaces; can run in parallel with T6. NEVER add a `waiver` block to this row (anti-C7). |
| **T8** | CLAUDE.md update + verifier subagent dispatch: extend "8 cadences" + "6 kinds" tables; add subagent-rule bullet for 9th probe; add deferral-pointer + banned-production-tokens entries; per PROTOCOL.md Step 7, dispatch unbiased verifier subagent against T1's 6 rows | All 6 | S (2-3h) | T1–T7 | Final commit. `git push origin main` BEFORE verifier dispatch per CLAUDE.md "Push cadence". Verdict at `quality/reports/verdicts/p89/VERDICT.md`. |

**Effort total:** 18–25h (within 5-6 day envelope per CONTEXT spec_lock).

**Parallelism (per CLAUDE.md Build memory budget — no cargo work in P89, so no RAM pressure):**
- After T1: T2 + T3 + T7 can run in parallel (3 disjoint file surfaces).
- T4 → T5 → T6 sequenced (all touch `run.py`).
- T8 is the wrap-up.

**Verifier subagent prompt for T8** uses verbatim template from `quality/PROTOCOL.md:267-296` § "Verifier subagent prompt template". Subagent grades 6 catalog rows + CLAUDE.md QG-07 update.

## Sources

### Primary (HIGH confidence — read end-to-end)
- `quality/runners/run.py:1-376` — runner contract surface
- `quality/runners/_freshness.py:1-50` — sibling-module factoring precedent
- `quality/PROTOCOL.md:1-307` — runtime contract (especially Steps 3, 6, 7; Principles A+B; Verifier subagent prompt template)
- `quality/catalogs/README.md:1-207` — schema spec
- `quality/catalogs/agent-ux.json` — sample row shapes (read first 100 lines)
- `quality/catalogs/freshness-invariants.json` — structure-dim catalog (read first 200 lines)
- `quality/gates/agent-ux/dark-factory.sh:1-59` + `quality/gates/agent-ux/dark-factory/{lib.sh,sim.sh,dvcs-third-arm.sh}` (listed; lib.sh factoring confirmed)
- `quality/gates/agent-ux/reposix-attach.sh:1-53` — verifier pattern
- `quality/gates/agent-ux/mirror-refs-write-on-success.sh:1-31` — verifier-without-artifact-write evidence
- `quality/gates/structure/no-loose-top-level-planning-audits.sh:1-26` — structure-dim verifier shape
- `scripts/banned-words-lint.sh:1-200` — banned-tokens canonical pattern (DOCS-only scope confirmed)
- `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:1-67` — existing 8-probe verdict structure
- `.githooks/pre-push:1-71` — pre-push wiring (no script edit needed; runner discovery handles new rows)

### Secondary (referenced)
- `.planning/phases/89-framework-fixes-cadence-shell-kind/89-CONTEXT.md` — locked decisions + open questions
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md:120-167` — F-K1..F-K7 sources + P89 verbatim
- `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md:30-44` — S1 chicken-and-egg + Decision 3 reference
- `.planning/research/v0.13.0-real-backend-frictions/_archive/DECISIONS-NEEDED.md:25-46` — Decision 3 + Decision 4 verbatim
- `.planning/milestones/v0.13.0-phases/ROADMAP.md:135-167` — Phase 89 success criteria
- `CLAUDE.md` § "Quality Gates — dimension/cadence/kind taxonomy" + § "Subagent delegation rules" + § "Push cadence — per-phase"
- `crates/reposix-cli/src/attach.rs:163-164` — production violation worked example
- `crates/reposix-cli/src/sync.rs:42` — second production violation
- `crates/reposix-core/src/error.rs:54,81` — `P1-1` false-positive evidence (drives regex tightening)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every framework component already exists in the codebase; this phase EXTENDS them
- Architecture: HIGH — runner code read end-to-end; existing verifier scripts surveyed; transcript convention is the only genuinely-new shape
- Pitfalls: HIGH — false-positive risk for banned-tokens regex empirically measured (10+ legitimate `P1-1`-style audit IDs in production source)
- Open questions: HIGH — all 10 resolved with file:line evidence

**Research date:** 2026-05-08
**Valid until:** 2026-06-07 (30 days; quality framework is stable; risk of source drift before P89 ships is LOW)

## RESEARCH COMPLETE
