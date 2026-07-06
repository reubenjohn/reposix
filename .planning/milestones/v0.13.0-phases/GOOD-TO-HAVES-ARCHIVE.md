# v0.13.0 GOOD-TO-HAVES — ARCHIVE

> **Terminal entries relocated from `GOOD-TO-HAVES.md` by P96 Wave 3a (OP-8 Slot 1 hygiene).** Pure relocation — entry bodies are byte-identical to
> their originals; only their file home changed. These are CLOSED: RESOLVED /
> WONTFIX / DEFERRED-to-a-future-milestone. The live drain queue stays in
> [`GOOD-TO-HAVES.md`](./GOOD-TO-HAVES.md).

**Archived here: 3 entries.**

## Manifest

- 2026-07-05 | Quality-gate footgun: test-name-honesty marker silently ignored outside the 6-line lookback window | dis...  → **RESOLVED**
- 2026-07-05 | Refresh stale header caveats in the two security gate scripts | discovered-by: P92 security-waiver-flip ...  → **RESOLVED**
- 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-ri...  → **RESOLVED**

---

## Entries

## 2026-07-05 | Quality-gate footgun: test-name-honesty marker silently ignored outside the 6-line lookback window | discovered-by: P92 Exec2 (SC5 test verification) | severity: MEDIUM

**What:** The `test-name-vs-asserts` agent-ux gate (`quality/gates/agent-ux/test-name-vs-asserts.sh`) only scans a 6-line window immediately after each `#[test]` or `fn test_` function signature for the honesty marker (`// HONEST: {reason}`). A correctly-present marker placed farther than 6 lines from the test declaration is silently ignored by the gate's regex scan, so the test reads as passing the gate despite carrying an honest marker — the gate's own contract is invisible once the marker drifts beyond the window. A P92 executor hit this: SC5 test read as passing until the marker was moved into range, surfacing the gate's silent distance requirement.

**Why deferred from P92:** SC5's task envelope is writing the test and embedding the marker; understanding the gate's placement-distance requirement belongs in the gate's own documentation + CLAUDE.md notes, not SC5's scope. The gate is correctly tuned (6 lines = function signature + 1-2 lines of setup is the typical case; enforcing tight placement is reasonable), but the distance constraint is undocumented.

**Sketched resolution:** (1) Document the 6-line lookback window in `quality/gates/agent-ux/test-name-vs-asserts.sh`'s header comment (state the window size, give an example of a correctly-placed marker and a mis-placed one outside the window). (2) Add a similar note to `quality/CLAUDE.md` under the test-name-honesty section (or wherever it documents the marker format) — name the placement rule so a future agent planning a honesty marker knows where to put it BEFORE writing the test. This is fix-it-twice per CLAUDE.md OD-3 meta-rule (notice is a deliverable; update the instructions to prevent recurrence).

**Default disposition:** MEDIUM — default-defer to the P95 quality-framework honesty/documentation pass; the gate itself is correct and documented at `quality/PROTOCOL.md` § "Honesty rules: test-name-asserts / marker format," but the placement-distance requirement is missing from both the gate script and CLAUDE.md.

**STATUS:** RESOLVED (P95, commit `3e9b2b2`) | Documented the 6-line lookback window in BOTH mandated homes — the gate script header (`quality/gates/agent-ux/test-name-vs-asserts.sh`: window mechanics + correctly-placed vs mis-placed example + the two silent-footgun consequences) and `quality/CLAUDE.md` § "Honesty rules" (mirror note). Ownership call = doc-only (not the robust preamble-anchored scan): a probe of all 85 test-pattern fns in `crates/` found 0 with a marker or `#[test]` beyond the 6-line window, so the footgun bites nothing today; its harm is future invisibility, which documentation resolves. Filer's own "correctly tuned / tight placement is reasonable" assessment argues against unilaterally loosening a repo-wide pre-push gate at milestone-close. The robust fix is filed as the sibling GOOD-TO-HAVE below. **Corrections vs this entry as filed:** (1) the window is a LOOKBACK (6 lines ABOVE the signature), not "immediately after" the fn; the marker format is `// test-name-honesty: ok — <reason>`, not `// HONEST: {reason}` as the What section states. (2) This entry's Default-disposition claims the marker format is "documented at `quality/PROTOCOL.md` § 'Honesty rules: test-name-asserts / marker format'" — that section does not exist; PROTOCOL.md has no test-name/marker mention at all (verified P95). The gate header + CLAUDE.md are the correct homes and now carry it.

---

## 2026-07-05 | Refresh stale header caveats in the two security gate scripts | discovered-by: P92 security-waiver-flip executor | severity: LOW

**What:** `quality/gates/security/allowlist-enforcement.sh` and `quality/gates/security/audit-immutability.sh` both carry header comments stating the gate "has NOT been executed via a real cargo run" — now false. Both gates ran green (real cargo, full pre-commit/pre-push sweep) on 2026-07-05; their catalog rows are PASS (commit 99b57b4). The header comments are lying to the next reader.

**Acceptance:** Update both scripts' header comments to reflect that they have been executed and passed (remove or update the "NOT been executed" caveat). Optionally note the execution date or the commit at which they first passed.

**Why deferred:** pure documentation update; the gates themselves are correct and running. This is fix-it-twice grounding (notice + update instructions so the next reader isn't misled).

**Default disposition:** LOW — fold into the next CLAUDE.md or `quality/gates/security/` documentation pass, or into whichever phase next touches these scripts.

**STATUS:** RESOLVED — 2026-07-05 debt-drain triage, commit `ae93cfb` (`docs(security-gates): refresh stale "not executed" caveats in gate script headers (P92 GTH)`). Both scripts' header comments now state the real 2026-07-05 execution (12/13 + 8/8 + 1/1 tests passing) and the P92 CI run 28735908764 confirmation, matching `quality/catalogs/security-gates.json`'s owner_hint fields for both rows. Comment-only change; no shell logic touched.

---

## 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-risk executor | severity: MEDIUM

**What:** The `badges-resolve` check (spanning the `docs-build` and `structure` quality
dimensions — README/docs badge URLs must resolve, e.g. shields.io, Codecov, CI status)
is FAILing on pre-push. Most likely a shields.io or Codecov transient network flake
rather than a genuine broken badge, but this has not yet been confirmed either way.

**Acceptance:** During the P94–P97 debt-drain window, re-run `badges-resolve` in
isolation (ideally on more than one occasion, spaced apart) to distinguish a real broken
badge URL from a transient upstream flake. If transient: consider whether the gate needs
a retry/backoff before failing, or a documented waiver note. If real: fix the underlying
badge URL/config.

**Why deferred:** confirming real-vs-transient requires multiple isolated re-runs over
time, which doesn't fit Wave 1's single-pass de-risk window; the fix (if any) is also
docs-build/structure-gate territory, orthogonal to this wave's durable-record + push-risk
scope.

**Default disposition:** MEDIUM — confirm real-vs-transient in the P94–P97 debt window,
then fix (if real) or waive with a documented reason (if transient/flaky-upstream).

**STATUS:** RESOLVED (P94 D3, 2026-07-05) — **verdict: TRANSIENT** (upstream flake, not a
broken badge). `badges-resolve.py` was re-run in isolation on 3 spaced occasions
(23:48:04Z, 23:48:37Z, 23:56:36Z, ~8 min apart) — all 10 badge URLs from README.md +
docs/index.md returned HTTP 200 + correct content-type on the first attempt every time;
no URL was ever a deterministic 404/wrong-type. The recurring pre-push RED was a
single-shot HEAD hitting a transient shields.io/Codecov/GitHub badge-endpoint hiccup
(timeout or 5xx/429 under load). **Fix (transient branch of the acceptance criteria):**
bounded retry/backoff added to `quality/gates/docs-build/badges-resolve.py` `head_url()`
(`MAX_ATTEMPTS=3`, `BACKOFF_S=(1.0, 2.0)`, retries only 408/425/429/5xx + network errors;
a real 404/403/wrong-type still fails fast so a genuine breakage can't be masked). Chosen
over a waiver: a waiver would blanket-suppress a future real breakage and expire,
re-surfacing the flake. Full evidence:
`.planning/phases/94-real-backend-frictions/94-D3-badges-determination.md`. Net:
`badges-resolve.py` exits 0 reliably.

---

