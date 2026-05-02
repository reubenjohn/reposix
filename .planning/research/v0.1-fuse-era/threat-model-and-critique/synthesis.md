← [back to index](./index.md)

## Synthesis

The PROJECT.md threat model is one paragraph and zero requirements. This document is the missing requirements. The fastest way to internalize them is to add the bullets in PART D.1 to PROJECT.md `### Active` *now* (before the build agent starts on FUSE/remote/CLI), so they show up as checkboxes the build agent feels obligated to satisfy.

If only one thing is added: **the egress allowlist (D.1 first bullet, D.3 first test).** It cuts the exfiltration leg of the trifecta for every attack scenario in PART A simultaneously, including ones not enumerated here. It is ~30 lines of Rust. It survives every other architectural decision.

If only two things: add **bulk-delete guard** (D.1 fifth bullet, D.3 bulk-delete test). The "agent runs `rm -rf` on the mount" failure mode is the most-likely-to-appear-in-a-blog-post-headline incident for this project. Costs ~50 lines of Rust to prevent.

The "demo by 8am" plan is achievable for read-only with audit log and a green CI. Write + remote-helper + swarm + FUSE-in-CI in 7 hours is not credible; the agent should pre-commit to the read-only fallback at hour 3 and not litigate it at hour 6.

The single most important sentence in `docs/research/agentic-engineering-reference.md` for this project is §5.5:

> "Every deployment of an unsafe agent that doesn't get exploited increases institutional confidence in it. This is the Challenger O-ring dynamic."

A reposix v0.1 that demos beautifully but lacks egress allowlisting is exactly such a deployment. Ship the constraint with the feature, or don't ship the feature.
