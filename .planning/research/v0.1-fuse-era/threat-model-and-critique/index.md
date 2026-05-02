# reposix Threat Model and Adversarial Critique

**Reviewer:** adversarial security subagent
**Date:** 2026-04-13
**Mandate:** poke holes in the design before shipping a 7-hour overnight demo
**Frame:** Simon Willison lethal-trifecta (`docs/research/agentic-engineering-reference.md` §5)

> "Anyone who can get text into the agent's context can make it do anything the agent is authorized to do." — `docs/research/agentic-engineering-reference.md` §5.3
>
> "This project is a giant lethal-trifecta machine if built naively." — `docs/research/agentic-engineering-reference.md` §5.6

The PROJECT.md threat-model paragraph (line 45) acknowledges this in one sentence:

> "This project is a textbook lethal trifecta: private remote data + untrusted ticket text + git-push exfiltration. Mitigations are first-class: tainted-content marking, audit log, no auto-push to unauthorized remotes, RBAC → POSIX permission translation."

That paragraph is a promissory note. The "Active" requirements list (lines 21–29) does not contain a single explicit security requirement — every checkbox is a feature. **Security is not a feature; it is a constraint on every other feature, and right now reposix has no committed artifact that enforces any of the four mitigations promised in line 45.** This document is the gap analysis.

---

## Chapters

- [PART A — Lethal Trifecta Audit](./part-a.md)
- [PART B — Architectural Holes](./part-b.md)
- [PART C — Unbiased Critique of "Demo by 8am"](./part-c.md)
- [PART D — Recommended Mitigations (Concrete)](./part-d.md)
- [Synthesis](./synthesis.md)
