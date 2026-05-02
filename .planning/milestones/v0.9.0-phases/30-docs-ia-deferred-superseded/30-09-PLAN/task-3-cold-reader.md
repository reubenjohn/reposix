← [back to index](./index.md)

# Task 3: Cold-reader review — does the value prop LAND in 10 seconds?

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Cold-reader review — does the value prop LAND in 10 seconds?</name>
  <what-built>
    docs/index.md (rewritten in plan 30-03) is the Layer-1 hero. Plan 30-09 Task 1 confirmed Vale + mkdocs + structure + tutorial all green, and Task 2 captured 14 screenshots. The last gate is the subjective one: does a cold reader, with zero context, state reposix's value proposition in one sentence after reading the page?

    Automated proxy for this: the `doc-clarity-review` skill, invoked with a purpose-built prompt that asks for a VERDICT (LANDED / PARTIAL / MISSED) + the single-sentence statement of reposix's value.
  </what-built>
  <how-to-verify>
    **Step 1 — Run doc-clarity-review against the rendered landing page:**

    ```bash
    VERDICT_DIR=$(mktemp -d /tmp/phase-30-value-prop-XXXXXX)
    cp docs/index.md "$VERDICT_DIR/"

    cat > "$VERDICT_DIR/_prompt.md" <<'EOF'
    You are reading this page for the first time, completely cold. You have NOT
    seen the repo, any other documentation, or this project's GitHub.

    Your job: after reading the page, tell me in EXACTLY ONE SENTENCE what
    reposix is and what problem it solves. Then rate:

    - LANDED — you got it from the content alone
    - PARTIAL — you got a rough idea but are missing the pivotal point
    - MISSED — you'd need to read more to answer

    Then: identify the single sentence or image on this page that did the most work.
    If you can't identify one, that's itself a verdict.
    EOF

    claude -p "$(cat $VERDICT_DIR/_prompt.md)" "$VERDICT_DIR/index.md" 2>&1 \
        | tee "$VERDICT_DIR/_feedback.md"
    ```

    **Step 2 — Parse the verdict:**

    ```bash
    grep -iE '^(LANDED|PARTIAL|MISSED)' "$VERDICT_DIR/_feedback.md" | head -1
    # If LANDED: phase passes; proceed to CHANGELOG + SUMMARY.
    # If PARTIAL: return to plan 30-03 for targeted revision; re-run this gate.
    # If MISSED: return to plan 30-03 for full rewrite; re-run this gate.
    ```

    **Step 3 — User confirms the verdict aligns with intent:**

    1. Read `_feedback.md` end-to-end.
    2. Decide: does the single-sentence statement match what reposix actually does?
    3. If YES + verdict = LANDED → approve.
    4. If verdict = LANDED but the single-sentence statement is WRONG (e.g. "reposix replaces your Jira" — explicit P1 violation landed in summary) → this is a calibration false-positive; return to plan 30-03 for clarification. The cold-reader subagent's summary must reflect P1.
    5. If verdict is PARTIAL or MISSED → decide whether this phase ships as-is (gap captured as Phase 30.1 follow-up) OR returns to plan 30-03. User discretion.

    **Step 4 — Copy the verdict into the phase SUMMARY:**

    The `_feedback.md` file is preserved in the phase directory as evidence:

    ```bash
    cp "$VERDICT_DIR/_feedback.md" \
        .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-09-doc-clarity-feedback.md
    ```
  </how-to-verify>
  <resume-signal>
    Type `approved` to continue to CHANGELOG + SUMMARY composition (Task 4).
    Type `revise plan 30-03: <reason>` to loop back — the orchestrator will re-plan 30-03 with the cold-reader feedback as input, then re-run this gate.
    Type `ship partial: <rationale>` to ship with a PARTIAL verdict documented in SUMMARY as a known gap.
  </resume-signal>
</task>

