# item 5 RED — root-cause diagnosis (2026-07-13)

**Lane:** bounded DIAGNOSTIC (single tree-writer, NO product-code mutations). Charter:
root-cause the v0.14.0 tag-blocking coherence RED (litmus + p93 FAIL vs live TokenWorld),
classify BOUNDED-vs-ARCHITECTURAL, delete orphan `9994241`. All evidence below is
EXECUTED against live TokenWorld (space `REPOSIX`/`360450`, tenant `reuben-john`) with the
`target/debug` binaries built at commit `81f3c83` (post-item-4b `d1cc811`).

---

## TL;DR — VERDICT: **BOUNDED**

Both tag-blocking REDs share ONE bounded product bug plus one fixture-hygiene issue:

1. **Latent product bug (the real find):** `reposix-confluence` cannot parse the REAL
   Confluence v2 ADF body. The v2 API returns `body.atlas_doc_format.value` as a **JSON
   *string*** (`"{\"type\":\"doc\",…}"`), but `ConfBodyAdf.value` is typed
   `serde_json::Value` and receives it as `Value::String(…)`, so `adf_root_type()` reads
   `""` and `adf_to_markdown` ALWAYS errs `root node type must be "doc", got ""` for every
   real page. Pre-item-4b this degraded to an **empty** body (benign no-op). **Item 4b
   (`d1cc811`) changed empty → a non-empty fail-closed SENTINEL**, and the sentinel now
   breaks push round-trips.
2. **p93 RED** is DIRECTLY this: `landed_a = get_record(A)` returns the sentinel body,
   `render_verbatim` ships it in push-2, `plan()` PUTs it, and item-4b's OWN fail-closed
   guard refuses it → `some-actions-failed`. **Proven** by the refusal message naming page
   A's id (below).
3. **litmus RED** is primarily **mirror staleness** (the GitHub mirror `pages/2818063.md`
   frontmatter is pinned `version: 1` while the backend SoT is `version: 7`); the one-shot
   `git pull --rebase reposix main` recovery cannot reconcile the divergent `version:` line
   + body, and the item-4b sentinel in the fetched backend content makes the rebase
   conflict unavoidable.

The fix (decode the string-encoded ADF value + correct the lying wiremock) is a localized
correctness fix in one crate; the mirror is a fixture repair. No wire-ref/ADR/load-bearing
behavior changes. **BOUNDED.**

---

## 1. Pinned root causes (file:line + executed evidence)

### 1a. The shared latent bug — string-encoded ADF value (all real pages)

- `crates/reposix-confluence/src/types.rs:172-174` — `pub struct ConfBodyAdf { pub value:
  serde_json::Value }` expects an OBJECT.
- `crates/reposix-confluence/src/adf.rs:168-174` — `adf_to_markdown` errs when
  `adf.get("type")` != `"doc"`; a `Value::String` has no `.get("type")` → `""`.
- The wiremock fixtures encode ADF as an **object** and thus never caught this:
  `crates/reposix-confluence/src/client.rs:928-930` (`page_json_adf`) and `:2661,:2673`
  (`"value": {"type":"doc",…}`). **The mocks disagree with the real API.**

**Executed proof the real API returns a STRING** (`adf_valtype.py`):
```
page 2818063: value python-type=str   first 70: '{"type":"doc","content":[{"type":"paragraph",…'
page 7766017: value python-type=str   first 70: '{"type":"doc","content":[{"type":"paragraph",…'
```
Direct v2 read shows every page has a VALID `doc` ADF (as a string). reposix reads `""`.

**Executed proof reposix sentinels EVERY page** (git-remote-reposix stderr, push path):
```
WARN reposix_confluence::translate: adf_to_markdown failed and no storage fallback;
  substituting fail-closed unreadable-ADF sentinel
  error=adf_to_markdown: root node type must be "doc", got "" page_id=2818063 adf_root_type=
  …page_id=7766017 … page_id=7798785 … page_id=<A> … page_id=<B> … page_id=<blocker>
```
All 6 pushed pages — INCLUDING the protected pair `7766017`/`7798785` — sentinel.

### 1b. p93 FAIL — the sentinel + item-4b's own fail-closed guard

- Panic: `crates/reposix-cli/tests/agent_flow_real.rs:925` — `push 2 (recovery) must
  succeed and converge; stdout=error refs/heads/main some-actions-failed`.
- `landed_a` is captured from `get_record` at `agent_flow_real.rs:888-890` and re-shipped
  verbatim at `:918` (`render_verbatim(&landed_a)`); the test never overrides its body.
- Since `get_record(A)` returns the item-4b sentinel (1a), `plan()` PUTs page A and the
  guard `refuse_unreadable_adf_sentinel` (`crates/reposix-confluence/src/lib.rs:578-595`,
  wired into `update_record` at `:389-397`) refuses it.

**Executed proof — re-ran the real test (single cargo invocation):**
```
$ cargo test -p reposix-cli --test agent_flow_real partial_failure_recovery_real_confluence -- --ignored
thread '…' panicked at crates/reposix-cli/tests/agent_flow_real.rs:925:5:
push 2 (recovery) must succeed and converge; stdout=error refs/heads/main some-actions-failed
test result: FAILED. 0 passed; 1 failed … finished in 8.68s
```

**Executed proof of the FAILING ACTION** — a faithful push-2 reproduction (Python harness
driving `git-remote-reposix` and capturing the stderr the Rust test DISCARDS). With
`landed_a.body` set to the sentinel (mimicking `get_record`), push-2 flips RED with:
```
error: patch issue 9895992: confluence update_record: refusing to write page 9895992 —
  its body is the reposix unreadable-ADF placeholder, not real content.
  … Pushing it would DESTROY the real page content in the system of record. …
```
`9895992` was page **A** (first id torn down). Control run with `landed_a` from
`reposix list` (empty body, NOT sentinel) → push-2 **GREEN** (`ok refs/heads/main`). The
single variable that flips GREEN↔RED is whether A carries the item-4b sentinel. Push-1's
failure is the DESIGNED B title-collision (`400 … A page already exists with the same
TITLE`), unrelated.

### 1c. litmus FAIL — mirror staleness (+ sentinel worsens the rebase)

- Fail site: `quality/gates/agent-ux/lib/litmus-flow.sh:108` — push rejected even after
  the ONE documented recovery `git pull --rebase reposix main` (`:105`).
- The litmus clones the GitHub mirror (`:27`) and edits the first non-protected record,
  `pages/2818063.md`.

**Executed proof of the staleness** — fresh clone of the mirror:
```
$ git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git
mirror pages/2818063.md frontmatter:  version: 1      (body: old T2-marker + litmus-marker lines)
```
vs the backend (v2 API) and a FRESH reposix fetch:
```
$ reposix init confluence::REPOSIX /tmp/fresh   →   pages/2818063.md frontmatter: version: 7
```
Push precheck for 2818063: local base `v1` ≠ backend `v7` (`stale_base=true`) AND content
diverges → the lost-update guard (`crates/reposix-remote/src/precheck.rs:264-303`)
correctly rejects `fetch first`. The recovery `git pull --rebase reposix main` fetches
backend-current, whose 2818063 body is now the item-4b **sentinel** (1a) at frontmatter
`version: 7`; rebasing the mirror-`v1`-based marker edit onto it conflicts on the
`version:` line + body → hard RED at `:108`. This is the same deadlock as
`B1-mirror-reconcile-FINDINGS-2026-07-13.md`; `reposix sync --reconcile` provably does NOT
refresh the external mirror repo content (that findings file, still accurate).

---

## 2. Charter (a)/(b)/(c) verdicts

**(a) empty-ADF `adf_to_markdown` — REAL bug (broad), NOT a restore-artifact of 2818063.**
Fetched into a FRESH cache, the PROTECTED PAIR sentinel too (1a stderr: `7766017`,
`7798785` both logged). At the API every page has a valid `doc` ADF (1a `adf_valtype.py`);
reposix reads `""` for ALL of them because of the string-vs-object deserialization bug. So
it is a real, broad `reposix-confluence` translate/parse bug exposed as a sentinel by item
4b — categorically NOT specific to `2818063`'s restored body. (No page's body actually
"translates" today via `get_record`; the fresh-init tree carries EMPTY bodies for all
three — the pre-4b symptom of the same bug.)

**(b) p93 15:28-GREEN → 19:06-RED delta — item 4b (`d1cc811`), NOT the restore.** No
`crates/**` commit landed between the two runs (last was `22a7777` @11:00; `git log`).
p93's RED is DETERMINISTIC post-`d1cc811` (10:31): before it, `get_record` on an
unreadable-ADF page yielded an EMPTY body → `landed_a` round-tripped empty → push-2 no-op →
GREEN; after it, `get_record` yields the SENTINEL → push-2 PUTs A → fail-closed refusal →
RED. The "15:28 GREEN" reflects pre-4b `get_record` behavior in the tested binary. The
RESTORE of `2818063` is NOT the trigger — the proven failing action is page **A** (1b); the
restored `2818063` rides along harmlessly via `list_records` (empty body, not the sentinel).

**(c) delta-sync `last_fetched_at` cursor — REFUTED.** The P113 lost-update guard makes the
cursor / `list_changed_since` **advisory, not the conflict gate**
(`crates/reposix-remote/src/precheck.rs:128-190`): it re-fetches EVERY pushed Update via
`get_record` regardless of the `changed_set`, so a cursor advanced past a concurrent write
CANNOT drop a record from conflict detection. Neither RED involves the cursor.

---

## 3. Fix-size estimate

| Piece | Scope | Est. |
|---|---|---|
| Decode string-encoded `atlas_doc_format.value` before `adf_to_markdown` (custom `Deserialize` on `ConfBodyAdf`, or a normalize step in the get/list body extraction) | 1 crate (`reposix-confluence`), `types.rs`/`translate.rs`/`client.rs` | ~15–40 LoC |
| **Fix-twice:** correct the lying wiremock fixtures to string-encode the ADF value (`page_json_adf` + contract tests) so the bug can't regress | same crate, tests | ~20–50 LoC |
| Real-backend regression assert: `get_record`/list body is real content, never the sentinel, against TokenWorld | 1 test | ~20 LoC |
| litmus: refresh the mirror fixture `pages/2818063.md` to backend-current v7 (fixture repair, no code) and/or land GTH-V15-09 (litmus self-reconciles the mirror before editing) | fixture/harness | ~0 code / small harness |

Total product change: **one crate, well under ~100 LoC including tests.**

---

## 4. BOUNDED-vs-ARCHITECTURAL

**BOUNDED.** The core defect is a localized deserialization mismatch: the real Confluence
v2 API string-encodes the ADF `value` and the adapter never decoded it, so the Confluence
body-read path has silently produced empty bodies since it was written — item 4b merely
converted the silent empty into a loud, push-blocking sentinel. Fixing it is a
string-decode in `reposix-confluence` plus correcting the unfaithful mock; it changes no
wire-ref format, no refspec, no ADR, and no load-bearing DVCS behavior. Item 4b's
fail-closed guard is CORRECT and stays — once ADF-parse works, the sentinel only fires for
genuinely-unreadable ADF (rare), which is exactly its intended job. The litmus's remaining
blocker is fixture staleness (external mirror content), a repair, not a code change. Both
fixes pass sim-first + corrected wiremock + a real-backend regression test and would survive
review without altering a load-bearing behavior. Nothing here forces a schema, refspec, or
ADR change — hence bounded, not architectural.

---

## 5. Owed cleanup — DONE

Orphan p93 teardown-500 leftover `9994241` deleted (`confluence_tokenworld.py delete 9994241`
→ HTTP 204). TokenWorld end state verified EXACTLY the 3 fixture pages:
```
2818063  reposix demo space Home
7766017  … parent … [PROTECTED]
7798785  … child  … [PROTECTED]   (parent=7766017)
--- 3 current pages in space 360450 ---
```
Before: 4 pages (incl. `9994241`). After: 3. All p93-repro pages self-torn-down (204s).

---

## 6. Noticing (owner mandate OD-3)

1. **The wiremock ADF fixtures LIE.** `page_json_adf` (`client.rs:928`) and the contract
   tests encode `atlas_doc_format.value` as a JSON object; the real v2 API string-encodes
   it. Every ADF test is green against a shape the backend never sends — the entire
   Confluence body-read path was never validated against reality. This is the "verify
   against reality" gap that let item 4b ship a change that sentinels 100% of real pages.
2. **Item 4b was validated sim/wiremock-only.** Its 7 new tests all use parseable ADF, so
   none exercised the real-API string-encoding it would trip on. A real-TokenWorld
   `get_record`-returns-real-body assertion would have caught it before the tag lane.
3. **p93's `run_helper_export_real` discards the helper stderr** (`agent_flow_real.rs:720-724`
   returns only `(status, stdout)`), so `some-actions-failed` surfaces with ZERO per-action
   detail — the real refusal (`refusing to write page …`) is thrown away. A failing
   real-backend push test that hides *which* action failed and why is a debuggability hole;
   capture+print stderr on assert failure.
4. **`reposix refresh` silently defaults to the sim** even inside an attached confluence
   tree — `reposix refresh <tree>` hit `127.0.0.1:7878/projects/demo` and errored
   `Connection refused` instead of using the tree's configured `reposix::confluence`
   remote. Teaching-free for an agent who just attached a real backend.
5. **`reposix init` prints a `Next:` checkout hint but exits 2**, and the working tree has
   no `pages/` until the manual `git checkout -B main refs/reposix/origin/main` — the exit
   code reads like failure for a successful bootstrap.

---

## Appendix — reproduction commands (all executed 2026-07-13)

- `python3 scripts/confluence_tokenworld.py delete 9994241 && … list` (cleanup + verify).
- `adf_valtype.py` — real API returns `value` as `str`.
- `reposix init confluence::REPOSIX /tmp/fresh` + checkout — fresh fetch = v7, empty bodies,
  no sentinel on disk (pre-4b symptom).
- `cargo test -p reposix-cli --test agent_flow_real partial_failure_recovery_real_confluence
  -- --ignored` — reproduces the RED at `:925` in 8.68s.
- `p93_repro.py` — faithful push-1/push-2 with helper-stderr capture; GREEN with empty
  `landed_a`, RED (`refusing to write page <A>`) with sentinel `landed_a`.
- `git clone …reposix-tokenworld-mirror.git` — mirror `pages/2818063.md` pinned `version: 1`.
