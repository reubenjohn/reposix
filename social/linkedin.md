# LinkedIn post

Had a genuine "whoa, coding agents are cool" moment last night.

**The idea:** expose REST-based SaaS systems — GitHub Issues, Confluence, Google Keep, Jira — as POSIX file systems. Just folders and files an agent can `cat`, `grep`, `sed`, and `git diff` over.

Why it matters: coding agents are *dramatically* more capable and efficient when they're working with files and folders than when they're juggling tool schemas and JSON payloads. Fewer tokens burned on tool definitions, fewer brittle API-shape mismatches, and APIs become the *escape hatch* for genuinely complex operations instead of the default interface for everything.

**The experiment:** I wanted to see how far a coding agent could take the idea on its own. Last night I handed Claude Code three inputs and said "you have until 8 AM. Go wild.":

1. A Gemini Deep Research report on the architecture — https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/InitialReport.md
2. Context on the agentic-engineering patterns I wanted it to follow — https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/AgenticEngineeringReference.md
3. GSD, the planning/execution workflow framework — https://github.com/gsd-build/get-shit-done

**What I woke up to:**

**reposix** — a working FUSE filesystem + `git-remote-helper` for issue trackers, with a measured **92.3% token-economy reduction vs. MCP** on an equivalent task (~12.9× more usable context for the agent).

Shipped overnight:
• Rust workspace across 5 crates, `#![forbid(unsafe_code)]`, `clippy::pedantic` clean
• **139 passing tests**, CI green across rustfmt / clippy / test / coverage / integration
• **8 security guardrails** enforced and demo-visible (outbound-origin allowlist, append-only audit log, frontmatter field allowlist, bulk-delete cap, rate limits, 409 conflict handling, CRLF handling, deterministic blobs)
• A `scripts/demo.sh` that runs the full end-to-end story in under 2 minutes
• A live MkDocs site with 11 mermaid architecture diagrams, published on GitHub Pages
• A reproducible token-economy benchmark with an auditable fixture

And a full paper trail: per-phase plans, adversarial code reviews, threat model, goal-backward verification — all committed, nothing hand-waved.

**The takeaway** isn't just "Claude built a thing overnight." It's that with the right grounding — a clear architectural north star, a disciplined planning framework, and guardrails that are themselves committed artifacts — an agent can run an entire SDLC loop (research → plan → execute → review → verify → ship) without a human in the chair.

There are still plenty of kinks to iron out: macOS support, a real GitHub Issues adapter, an adversarial swarm harness for v0.2. But the central thesis — that "APIs as filesystems" is a real unlock for autonomous agents — has measured evidence behind it now.

The morning brief I woke up to: https://github.com/reubenjohn/reposix/blob/main/MORNING-BRIEF.md
The repo: https://github.com/reubenjohn/reposix

#AIAgents #ClaudeCode #DeveloperTools #Rust #FUSE #AgenticEngineering
