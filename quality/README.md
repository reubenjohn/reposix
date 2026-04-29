# quality/ — Quality Gates framework

Automated regression prevention for reposix. Every claim the project makes
(docs accuracy, install works, latency meets budget, security invariants hold)
is backed by a verifiable catalog row with a deterministic or subagent-graded
check.

## Directory map

```
quality/
├── README.md            ← you are here
├── PROTOCOL.md          # Runtime contract — read at the start of every phase
├── SURPRISES.md         # Pivot journal — dead ends earlier phases hit
├── catalogs/            # Data layer: JSON rows defining what GREEN means
│   └── README.md        # Schema spec
├── gates/               # Verifier scripts organized by dimension
│   └── <dim>/README.md  # Per-dimension docs
├── runners/             # Orchestration: run gates, produce verdicts
│   └── README.md        # Usage and exit-code contract
└── reports/             # Output artifacts: verdicts, audits, badge
    └── README.md        # What gets written here and when
```

## Key concepts (one sentence each)

- **Dimension** — a regression class (code, docs-alignment, security, etc.). Nine total.
- **Catalog row** — one `(gate, verifier, expected-outcome)` triple in `catalogs/<dim>.json`.
- **Cadence** — when a gate runs: pre-push, pre-pr, weekly, pre-release, post-release, on-demand.
- **Kind** — how it's verified: mechanical, container, asset-exists, subagent-graded, manual.
- **Runner** — `runners/run.py` reads catalogs, executes verifiers, writes artifacts.
- **Verdict** — `runners/verdict.py` collates artifacts into a GREEN/RED markdown report + badge JSON.
- **Verifier subagent** — unbiased agent that grades catalog rows at phase close with zero session context.

## Where to go next

| Goal | Read |
|---|---|
| Execute a phase correctly | `PROTOCOL.md` (the full runtime contract) |
| Understand catalog row schema | `catalogs/README.md` |
| Add a new gate | `gates/README.md` then the relevant `gates/<dim>/README.md` |
| Run gates or read verdicts | `runners/README.md` |
| Understand report artifacts | `reports/README.md` |
| See what went wrong before | `SURPRISES.md` |
