← [back to index](./index.md)

# Verification, Success Criteria, and Output

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Playwright screenshot target URL | Must be `http://127.0.0.1:8000` (localhost mkdocs serve). Never external. |
| doc-clarity-review subprocess | Must receive ONLY the markdown file; no repo context, no planning notes. |
| CHANGELOG.md edit | Written once per phase; should NOT be rewritten on revision — append to `[Unreleased]` if verdict comes back PARTIAL. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-09-01 | Information Disclosure | Playwright captures external URL by accident | mitigate | `scripts/screenshot-docs.sh` rejects non-`http://127.0.0.1:*` URLs; orchestrator uses that script to generate the manifest. |
| T-30-09-02 | Tampering | doc-clarity-review verdict gamed by including "LANDED" in the prompt | mitigate | The custom prompt (RESEARCH.md §Example 4) asks for a VERDICT after reading; the skill writes a JSON-structured `_feedback.md` the orchestrator parses for the literal token. Document the prompt in 30-SUMMARY. |
| T-30-09-03 | Repudiation | Screenshots stale after post-review copy tweaks | mitigate | Capture screenshots AFTER the final commit of Wave 3; if any Wave-4 revision lands, re-capture. |
</threat_model>

<verification>
1. All four automated gates (mkdocs, vale, structure, tutorial-e2e) green.
2. 14 Playwright screenshots committed under `.planning/phases/30-.../screenshots/`.
3. doc-clarity-review verdict LANDED (documented in 30-09-doc-clarity-feedback.md).
4. 30-SUMMARY.md exists and consolidates every per-plan SUMMARY.
5. CHANGELOG.md `[v0.9.0]` entry present.
6. `git log --oneline -1` shows the phase-grade commit referencing SUMMARY path.
</verification>

<success_criteria>
- All Validation Sign-Off checkboxes from 30-VALIDATION.md are ticked.
- Every DOCS-01..09 requirement has evidence (screenshot, grep output, or test run) cited in SUMMARY.
- Cold-reader verdict LANDED.
- CHANGELOG ready for tag.
- No auto-push of v0.9.0 tag — user gates that via `scripts/tag-v0.9.0.sh` (separate follow-up).
</success_criteria>

<output>
The phase SUMMARY itself IS the output artifact. Additionally, the orchestrator should print:

```
## PHASE 30 SHIPPED

**Requirements closed:** DOCS-01..09 (all 9)
**Plans:** 9 plans, 5 waves
**Automated gates:** all green
**Cold-reader verdict:** LANDED
**Next user action:** review 30-SUMMARY.md; when satisfied, author scripts/tag-v0.9.0.sh and push.
```
</output>
