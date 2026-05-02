← [back to index](./index.md)

# Task 02-T04 — Catalog flip: WAIVED → PASS for all three rows (catalog-first commit)

<read_first>
- `quality/catalogs/freshness-invariants.json:324-435` (the three rows being edited).
- A non-WAIVED row in the same file (e.g., `structure/no-version-pinned-filenames` lines 6-37) — to confirm the precise PASS-row shape: `"status": "PASS"`, `"last_verified": "<ISO>"`, `"waiver": null`.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose with citations; tools validate and mint" (3 sub-points).
- `quality/runners/run.py` (just the relevant dispatch path: confirm the runner reads `verifier.script` + invokes via shell when the script ends in `.sh`).
</read_first>

<action>
Edit `quality/catalogs/freshness-invariants.json`. For EACH of the three rows
(lines 324-360, 361-397, 398-435 approximately — line numbers are pre-edit;
re-locate by `id` field):

1. Update `verifier.script`:
   - `no-loose-top-level-planning-audits`: change to `quality/gates/structure/no-loose-top-level-planning-audits.sh`
   - `no-pre-pivot-doc-stubs`: change to `quality/gates/structure/no-pre-pivot-doc-stubs.sh`
   - `repo-org-audit-artifact-present`: change to `quality/gates/structure/repo-org-audit-artifact-present.sh`
2. Update `verifier.args` — change from `["--row-id", "<row-id>"]` to `[]`. The .sh implements the row directly; no row-id dispatch arg.
3. Update `status` — change `"WAIVED"` to `"PASS"`.
4. Update `last_verified` — set to the current ISO-8601 UTC timestamp at the
   moment of the edit. Use `date -u +"%Y-%m-%dT%H:%M:%SZ"` to generate.
5. Update `waiver` — change from the multi-line `{"until": "...", "reason":
   "...", "dimension_owner": "structure", "tracked_in": "..."}` block to
   `null`. Match the precedent in PASS rows in the same file.

Do NOT alter:
- `id`, `dimension`, `cadence`, `kind`, `sources`, `command`, `expected.asserts`, `artifact`, `freshness_ttl`, `blast_radius`, `owner_hint`.

After editing, validate the JSON parses:

```bash
python3 -c 'import json; json.load(open("quality/catalogs/freshness-invariants.json"))'
```

Then run the runner end-to-end (it dispatches the .sh per `verifier.script`):

```bash
python3 quality/runners/run.py --cadence pre-push
```

This MUST exit 0 with the three rows graded PASS. If the runner is
script-extension-aware and dispatches `bash <path>` for `.sh` and `python3
<path>` for `.py`, this works as-is. If the runner has no extension dispatch
and always invokes `python3`, the runner needs a one-line dispatch update —
inspect `quality/runners/run.py` first and append a SURPRISES-INTAKE.md entry
+ either Eager-resolve (if < 30 min) or DEFER to P87.

Inspecting the runner BEFORE the catalog flip avoids the failure mode where
the catalog says PASS but the runner can't verify it — that's a
"self-invalidating gate" per CLAUDE.md "grep gate hygiene" lessons.
</action>

<acceptance_criteria>
- `python3 -c 'import json; json.load(open("quality/catalogs/freshness-invariants.json"))'` exits 0.
- `jq -r '.rows[] | select(.id | startswith("structure/no-loose-top-level-planning-audits") or startswith("structure/no-pre-pivot-doc-stubs") or startswith("structure/repo-org-audit-artifact-present")) | "\(.id) status=\(.status) verifier=\(.verifier.script) waiver=\(.waiver)"' quality/catalogs/freshness-invariants.json` shows all three rows: status=PASS, verifier ends in `.sh`, waiver=null.
- `python3 quality/runners/run.py --cadence pre-push` exits 0 (runner dispatches the new .sh wrappers + grades PASS).
- Three artifact files appear under `quality/reports/verifications/structure/`: `no-loose-top-level-planning-audits.json`, `no-pre-pivot-doc-stubs.json`, `repo-org-audit-artifact-present.json` — each with `"exit_code": 0`.
- The schema-shape match: each flipped row's JSON shape is identical to the existing PASS rows (e.g., `structure/no-version-pinned-filenames`).
</acceptance_criteria>
