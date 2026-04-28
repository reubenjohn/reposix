You are reviewing documentation files in complete isolation. You have NOT seen the codebase or any other files. Read only what is provided.

Your job: cross-check every "headline number" in the user-facing hero / overview / landing pages against the benchmark or measurement files in the same input set.

A "headline number" is any quantitative claim that appears in the hero / overview / landing copy, of these shapes:
- Percentages: "89.1% reduction", "40% faster", "99.9% uptime"
- Multipliers: "10x faster", "3x smaller"
- Absolute numbers: "1ms latency", "60 seconds", "200 lines"
- Ratios: "1:1000", "1 in 100"
- Time-bounded numbers: "in 5 minutes", "under 1 second"

For each headline number you find:

1. Where in the hero / overview does it appear? (file path + line number)
2. Is there a citation in the same paragraph (link, footnote, "(see benchmarks/foo.md)")? If yes: read the cited source.
3. Is there a benchmark / measurement file in the input set whose name suggests it covers this number? (e.g. "token-economy.md" likely covers a "token economy" claim)
4. If the source-of-truth file exists: does the number in the source-of-truth match the headline?
   - Percentages: drift <= 2 percentage points = OK; drift > 2pp = flag.
   - Multipliers: drift <= 10% = OK; drift > 10% = flag.
   - Absolute numbers: within stated confidence interval = OK; outside CI = flag.
5. If no source-of-truth file is found: flag as "uncited headline number".

Report the following sections:

## Headline numbers found

A table for every number found:

| Number | File:line in hero | Source-of-truth file | Drift | Status |
|---|---|---|---|---|
| 89.1% | README.md:23 | docs/benchmarks/token-economy.md:45 | 0.0pp | OK |
| 10x faster | docs/index.md:12 | (none found) | - | UNCITED |

## Friction points

What's confusing about how the numbers are presented? Are they cited inline? Are the source-of-truth artifacts easy to find from the citation?

## Verdict

Numeric score 1-10:
- 10: every headline number cites a source-of-truth artifact AND drift is within tolerance.
- 7-9: every headline number cites a source-of-truth AND drift is within tolerance, but 1-2 numbers have stale citations (3-5pp drift) -- fix-soon territory.
- 4-6: 1-2 headline numbers are uncited OR drift exceeds tolerance for 1+ numbers.
- 1-3: 3+ uncited numbers OR major drift (>5pp on a percentage, >25% on a multiplier).

Rate: <integer 1-10>
Verdict: <CLEAR if score >= 7 / NEEDS-WORK if 4-6 / CONFUSING if 1-3>
Rationale: <one paragraph>
