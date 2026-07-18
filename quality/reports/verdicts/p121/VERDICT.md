---
phase: 121-error-codes-explain
verdict: GREEN
verified: 2026-07-18T05:15:00Z
verifier: gsd-verifier (opus tier), unbiased phase-close gate
graded_state: 9b80538c (origin/main == HEAD)
ci: run 29631441150 @ 9b80538c concluded success (verified via gh)
score: 6/6 (SC1-SC5 + OP-2) PASS
methodology: goal-backward — built the real binaries, triggered live CLI + helper
  error paths in /tmp, ran every registry code through `reposix explain`, ran the gate
  + self-test, ran all three crates' test suites, and read the emission sites for the
  protocol-line-no-leak honesty check. SUMMARY/HANDOVER claims were not taken on faith.
---

# Phase 121: RPX error-code namespace + `reposix explain` — VERDICT

**Phase goal:** Every user-facing error carries a stable, documented code; `reposix
explain <code>` looks it up, mirroring `rustc --explain E0308`.

**Verdict: GREEN.** All six criteria (SC1–SC5 + the OP-2 threat cut) are genuinely met
on the current pushed tree, and every load-bearing honesty-mandate claim holds against
reality.

---

## Per-SC grade

### SC1 — every user-facing CLI **and** helper error emits an `[RPX-xxxx]` code — **PASS**

Verified against reality (not the registry). Built `target/debug/{reposix,git-remote-reposix}`
and triggered real error paths from a `/tmp` clone:

| Path | Rendered |
|---|---|
| `reposix init 'not-a-spec' …` | `… [RPX-0001]` + `Explain: reposix explain RPX-0001` |
| `reposix init sim::demo <existing-git-root>` | `… [RPX-0401]` + nudge |
| `reposix log` (no `--time-travel`) | `… [RPX-0301]` + nudge |
| `reposix tokens` (non-reposix tree) | `… [RPX-0203]` + nudge |
| `reposix spaces --backend sim` | `… [RPX-0302]` + nudge |
| `git-remote-reposix` (too few args) | `… [RPX-0602]` + nudge |
| `git-remote-reposix origin 'reposix::sim::demo?notmirror=foo'` | `… [RPX-0601]` + nudge |

The full surface is mechanically enforced, not spot-check-only: the gate's leg-3
(converse/M2) asserts every teaching site in teach_scan's 21-file CLI_SCOPE+HELPER_SCOPE
carries a code, and leg-5 (reverse-completeness) asserts all 33 registered codes are
emitted in production `src/`. `RPX_CODE_ALLOWLIST` is empty and there are **zero** inline
`rpx-code-exempt: ok` markers — no user-facing teaching site is exempted. The late-fixed
helper const-string sites (RPX-0503 `BLOB_LIMIT_EXCEEDED_FMT`, RPX-0506
`UNFILTERED_FETCH_HINT`) carry their tag on the `eprintln!` stderr string; the diag sites
(RPX-0504/0505 in write_loop.rs, RPX-0507 in main.rs) carry it on the `diag()`/`fail_push`
stderr line. `cargo test -p reposix-cli --test errors_teach_recovery` = 14/14 green;
`cargo test -p reposix-remote` (bare, covers bin-target tests) = all green incl.
`import_unreachable_detail_renders_rpx0507_tag_and_explain_nudge`.

### SC2 — `reposix explain <code>` prints non-empty cause + fix + copy-paste recovery for EVERY code — **PASS**

Live sweep over all 33 registered codes: **33 checked, 0 thin/broken** — each prints a
`RPX-xxxx: <title>` header, a non-empty extended cause, a `Fix:` line, and a `Recovery:`
block with ≥1 runnable command. Corroborated by the registry-driven Rust test
`every_registry_code_explains_with_cause_fix_recovery` (iterates `codes::REGISTRY`, not a
hardcoded count) and `codes::tests::every_entry_teaches_nonempty_cause_fix_recovery`
(137/137 reposix-core tests green). Bodies are genuinely rustc-grade (spot-read
RPX-0507/0603/0401/0402 — each teaches cause + fix + alternative + copy-paste recovery).

### SC3 — output SHAPE matches `rustc --explain E0308` — **PASS**

`rustc_parity_of_shape` test green: asserts reposix's own output is a code-header line +
a ≥5-line non-empty body + a `Fix:` section (the hard gate), and best-effort captures
`rustc --explain E0308` to assert the shared multi-line shape (honestly skippable if
rustc absent — not byte-diffed). Live output confirms the shape.

### SC4 — the `agent-ux/rpx-codes-registry` gate + `--list`/no-arg enumeration — **PASS**

`bash quality/gates/agent-ux/rpx-codes-registry.sh` exits **0**;
`python3 quality/gates/agent-ux/rpx_registry_check.py` exits **0** ("clean — 33 registered
codes; all emitted codes registered; all teaching sites coded"); `--self-test` exits **0**
(all 20 checks green — forward + M3 + integrity + M2 + reverse-completeness). `reposix
explain --list` and no-arg both enumerate all 33 codes sorted; `reposix explain RPX-9999`
teaches (`[RPX-0900]`, names `reposix explain --list`) and exits 1. Catalog row status
`PASS`, kind/coverage_kind `mechanical`, audit 1317 chars. Catalog-first ordering verified:
the checker commit `cab82d15` is an ancestor of the first impl commit `30c518da` (codes.rs).

### SC5 — docs + fix-twice — **PASS**

`docs/reference/error-codes.md` present (in mkdocs nav L139); `docs/reference/cli.md` has
an `## reposix explain` section (L326); `crates/CLAUDE.md` revised with the RPX registry
convention (L139, "how to add a code" + the gate). `bash quality/gates/docs-build/mkdocs-strict.sh`
exit 0; `bash quality/gates/structure/banned-words.sh` exit 0.

### OP-2 threat cut — registry/code-slot/explain are `&'static str`-only — **PASS**

`codes::REGISTRY` is 100% `&'static str` (recovery `&'static [&'static str]`). All 33
`teach_coded` code args and every `.code(...)` call site are static `ids::*` consts (or a
static literal) — no `format!`/variable/remote byte reaches any code slot (grep-confirmed:
0 dynamic code args). `errmsg.rs` Display interpolates only `self.code`. `explain` reads
only the static REGISTRY. No tainted byte can reach the code slot or explain output.

---

## Honesty-mandate checks (load-bearing) — ALL CONFIRMED

- **Protocol-line no-leak — CONFIRMED.** `grep` for `send_line`/`send_raw` carrying `RPX`
  = **NONE**. `diag()`/`Protocol::diag` are `eprintln!` (stderr); `send_line`/`send_blank`
  write to the protocol stdout writer. The `error refs/heads/main backend-unreachable` and
  `error refs/heads/main fetch first` status lines stay verbatim on stdout; the
  `[RPX-0504]`/`[RPX-0505]`/`[RPX-0507]` tags ride the accompanying stderr diag only. No
  bracketed code can reach a line git parses as protocol.
- **RPX-0601 not used for reachability — CONFIRMED.** RPX-0601 = "malformed reposix bus URL"
  (a PARSE/syntax fault). The mirror-UNREACHABLE fault (bus_handler.rs:463) is coded
  RPX-0603 ("mirror unreachable/misconfigured — PRECHECK A ls-remote failed"), a distinct
  registry entry; a bin-target test pins that the path carries `[RPX-0603]` and NOT RPX-0601.
- **RPX-0507 not a reuse of push-specific RPX-0504 — CONFIRMED.** Distinct `ids` const
  (`HELPER_IMPORT_UNREACHABLE`) + distinct ExplainEntry; emitted on the fetch/import path
  (main.rs `import_unreachable_detail`), while RPX-0504 is the pre-push precheck path.
- **Gate `EMISSION_EXEMPT` genuinely empty — CONFIRMED.** `EMISSION_EXEMPT = {}` and
  `RPX_CODE_ALLOWLIST = set()` in rpx_registry_check.py; leg-5 (all 33 codes emitted in
  src) passes with no exemptions — every registered code is actually emitted.

---

## NOTICING (OD-3)

- **GTH-V15-72 (filed, LOW, not a blocker).** `run_git`'s local-git-subprocess failures
  (`git init`/`config`, init.rs:170-190) `bail!` raw git stderr with no RPX code — it
  carries a valid `// teach-exempt: ok` marker (P120 decision, local-only, no remote
  bytes). This is consistent with the PLAN's explicit SC1 coverage floor (P120 teach-exempt
  sites are out of scope by the same reasoning). Transparently filed as a GOOD-TO-HAVE, not
  swept under the rug. Does not RED SC1.
- **Tests assert their names.** `every_registry_code_explains_*` iterates the real registry
  (not a hardcoded count); `rustc_parity_of_shape` makes real shape assertions with an
  honestly-skippable rustc leg; `unknown_code_teaches_*` asserts non-zero exit + the
  teaching content. No name-lying found.
- **Minor (not a defect):** `reposix cost --since gibberish` in a non-reposix tree surfaces
  RPX-0203 (the tree check correctly precedes `--since` parse) rather than RPX-0305;
  RPX-0305 is covered by the Rust suite. Correct precedence.
- **Clock note:** the catalog row `minted_at` (2026-07-18T01:51:13Z) and CI timestamps run
  ahead of the stated local date (2026-07-17) — a VM-clock/TZ artifact, not a defect.

---

## OVERALL: GREEN — phase P121 may close.

All SC1–SC5 + OP-2 verified against reality; all honesty-mandate claims confirmed; CI
green on main at the graded SHA. No remediation required.

_Verified: 2026-07-18 · Verifier: Claude (gsd-verifier, opus tier)_
