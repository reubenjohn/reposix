← [back to index](./index.md) · phase 30 research

## Architectural Responsibility Map

Docs phase — "tiers" are documentation categories rather than runtime architectural tiers. Using Diátaxis.

| Capability | Primary Tier (Diátaxis) | Secondary Tier | Rationale |
|------------|------------------------|----------------|-----------|
| Landing-page hero + value prop | **Narrative/Marketing (sits above Diátaxis)** | Explanation (below fold) | Hero is framing; Diátaxis doesn't cover it — source-of-truth note adds this as "Layer 1/2" ahead of Layer 3 which IS Diátaxis Explanation |
| Mental model in 60 seconds | **Explanation** | — | Propositional knowledge; study-mode, not work-mode |
| reposix vs MCP / SDKs | **Explanation** | — | Comparison grounding; same category as "Why" — explain by contrast |
| How-it-works pages (filesystem / git / trust) | **Explanation** | — | Architecture explanation with diagrams — study-mode |
| Write your own connector | **How-to** | Reference (trait signature) | Work-mode: "I want to add backend X" |
| Integrate with your agent | **How-to** | — | Work-mode: "I want to wire reposix into Claude Code" |
| Troubleshooting | **How-to** | — | Work-mode: "my mount is broken, what now?" |
| 5-minute tutorial | **Tutorial** | — | Guided encounter; learn-by-doing |
| Simulator page | **Reference** | — | Facts about a tool; read when you need to look something up |
| CLI / HTTP API / git-remote / frontmatter schema | **Reference** | — | Unchanged from Phase 26 |
| ADRs | **Reference** (decision records) | — | Unchanged |
| Research docs | **Explanation** | — | Unchanged |

**Key tier-correctness checks for the planner:**
- The tutorial must NOT be a reference. Every step must be an action the reader performs, not a description of a thing.
- How-it-works pages must NOT be how-tos. They describe; they don't prescribe. ("This is how reposix works" — not "here's how to run a FUSE daemon.")
- The connector guide is a How-to that links to Reference (`crates/reposix-core/src/backend.rs`) — the source-of-truth clause in the existing page (`Do NOT read the above and copy it into your adapter's docs — link to ... as the single source of truth`) is already correct and must be preserved.
- Mental model page must NOT become a hero. It's Explanation; it's read in study-mode after the reader has already committed to caring.
