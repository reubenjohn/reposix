# item 5 fix — DP-2 mechanism-verification review (2026-07-13)

**Commits under review:** `49666eb` (string-decode fix) + `95ed061` (mirror tooling).
**Reviewer:** fresh code-reviewer (DP-2 gate). Read-only; NO product-code edits.

## VERDICT

**MECHANISM-CORRECT** at the product-code core (the string-decode addresses the ROOT
cause, and the fail-closed guard is PRESERVED — see below), **WITH FINDINGS** on the
durable regression protection and fix-twice completeness. The core fix is merge-sound;
the findings weaken the *guarantees the commit claims*, not the fix's behavior.

## Mechanism check (#1) — PASS

`ConfBodyAdf::deserialize` (`types.rs:194-221`) matches `Value::String(s) →
serde_json::from_str(&s)`; the decoded OBJECT is stored in `ConfBodyAdf.value` and reaches
`adf_to_markdown(&adf_body.value)` at `translate.rs:117` with a real `doc` root. This is a
deserialize-shape correction (root cause), not sentinel suppression. Unit test
`translate_decodes_string_encoded_adf_value_to_real_markdown` asserts real markdown
(`# Real Title` + body text), not merely "not the sentinel".

## Fail-closed check (#2) — **PASS**

Traced every degenerate `value`:
- non-JSON string (`"not json at all"`) → `from_str` Err → kept verbatim → `adf_root_type`
  `""` → sentinel.
- empty string `""` → `from_str` Err → sentinel.
- JSON decoding to a non-object (`"123"`, `"null"`, `"\"x\""`) → `adf_root_type` `""` → sentinel.
- JSON object with wrong root (`{"type":"paragraph"}`) → `adf_to_markdown` Err → sentinel.

The ONLY Ok path requires a decoded object with `type:"doc"`. Garbage CANNOT silently pass
as a body. The item-4b guard is not weakened. Taint (#3): decode runs *inside* serde
deserialization, upstream of `Tainted::new(page)` — taint wraps the decoded page, not
stripped. serde_json's default recursion limit (128) is un-disabled (grep-confirmed), so
hostile deep JSON errors → sentinel rather than stack-overflow; `adf_to_markdown` re-caps
at 32. **PASS.**

## Findings

### HIGH — the "durable real-TokenWorld twin" is VACUOUS and MISNAMED
`agent_flow_real.rs` `get_record_real_confluence_body_is_not_unreadable_sentinel` (added by
`49666eb`) calls **`list_records`**, not `get_record`. `list_issues_impl`
(`client.rs:189`) fetches `/spaces/{id}/pages?limit=N` with **no `body-format`** → list
pages carry EMPTY bodies (`types.rs:108` documents this; `translate` yields `""`). So
`is_unreadable_adf_sentinel("")` is always false: the test **passes even against the
original buggy object-deserializer** — it never exercises the ADF decode the diagnostic's
repro (p93 `get_record(A)`) actually hit. The commit message's "the diagnostic's repro,
locked" is false, and the name promises `get_record`.
*Failure scenario:* future regression re-breaks the string decode; `get_record` sentinels
every page again; this "regression" test stays green because it reads empty list bodies.
*Fix:* for each listed id call `backend.get_record(&space, r.id)` (which requests
`body-format=atlas_doc_format`), assert the returned body is real content AND
`!is_unreadable_adf_sentinel`. Rename to match. Optionally assert at least one non-empty
markdown body so an all-empty backend can't pass vacuously.

### MED — sibling wiremock fixture still object-encodes ADF (fix-twice incomplete)
`crates/reposix-swarm/tests/mini_e2e.rs:198-211` `sample_page` still encodes
`atlas_doc_format.value` as an OBJECT (`{"type":"doc",…}`) and serves it to
`get_record` (`:249-254`). It passes only because the new deserializer's `other => other`
arm passes objects through — i.e. it still validates against a shape the live API never
sends, the exact "wiremock lies vs reality" gap the diagnosis flagged (its Noticing #1).
*Fix:* `"value": json!({...}).to_string()` here too.

### MED — mirror script's "verify matches backend" is circular + overlay is additive
`refresh-tokenworld-mirror.sh`: `BACKEND="$(all_versions "${TREE}")"` (line 134) reads the
**same overlaid clone** that was pushed, then compares it to the re-clone `TREE2` (line
136). This only proves the push round-tripped — it does NOT compare against a pristine
backend fetch, despite the "verify it now matches the backend" comment (line 130).
Separately, `git checkout FETCH_HEAD -- pages/` (line 109) is **additive**: a page present
in the mirror but absent from the backend is neither removed nor detected, so the
"byte-exact backend-current tree" claim can silently fail. Latent for the fixed 3-page
TokenWorld fixture, real if the space ever loses a page.
*Fix:* materialize a pristine backend tree separately (fresh `attach`+`fetch` into a clean
dir) and diff the pushed mirror against THAT; add a `git rm` for pages absent from the
backend (or `git read-tree`/reset to the fetched tree) so deletions propagate.

### LOW
- `refresh-tokenworld-mirror.sh` uses `set -uo pipefail` but **no `set -e`**; `git config`
  (118-119) and `git add -A pages/` (120) are unchecked — a failed `git add` would fall to
  the "already byte-identical → exit 0" branch as a false success. Critical mutating ops
  (clone/attach/fetch/checkout/push) ARE `||`-guarded, so blast radius is small.
- `git clone --quiet "${MIRROR_URL}" ...` / `attach "confluence::${SPACE}"` interpolate
  operator-env values (not remote-tainted) unquoted-of-leading-dash; add `--` argument
  terminators for defense-in-depth (LOW — values are ops-controlled, not attacker bytes).
- `translate_non_json_string_adf_value_still_sentinels` covers only ONE fail-closed input.
  For a security-critical guard, add empty-string and JSON-non-object cases (I verified
  they DO sentinel, but they are unlocked by a test).
- The fix relies on serde_json's **default** recursion limit; no explicit test or comment
  pins that dependency. A one-line comment near the `from_str` would make the DoS
  reasoning legible to the next reader.

## NOTICED (near the diff, pre-existing — not introduced by these commits)
- `ConfBodyAdf::deserialize`'s inner `Raw` requires `value` (no `#[serde(default)]`), same
  as the old derive: an `atlas_doc_format: {}` with no `value` fails the WHOLE
  `ConfPageList` deserialize (`client.rs:248`) → one malformed page DoS-fails the entire
  list. Pre-existing, but the decode rewrite was a natural place to make `value` default
  to `Value::Null` (→ sentinel) for graceful degradation, consistent with the crate's
  stated DoS posture (`translate.rs` parent-id handling).

---
_Reviewer: Claude (Opus 4.8), DP-2 mechanism-verification. Read-only; no product-code edits._
