# LinkedIn post

> LinkedIn strips markdown (bold/italics/code-ticks) but auto-shortens URLs. Paste as-is.

---

💡 Had a genuine "whoa, coding agents are cool" moment last night.

🗂️ The idea: expose REST-based SaaS systems — GitHub Issues, Confluence, Google Keep, Jira — as POSIX file systems. Just folders and files an agent can read, grep, edit, and git-diff over.

🧠 Why it matters: agents are dramatically more efficient on files and folders than on tool schemas and JSON payloads. Fewer tokens burned on tool definitions, fewer brittle API-shape mismatches, and APIs become the escape hatch for complex operations instead of the default interface.

🌙 The experiment: last night I handed Claude Code three inputs and said "you have until 8 AM. Go wild.":

📄 A Gemini Deep Research report on the architecture: https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/InitialReport.md
📄 Context on the agentic-engineering patterns I wanted followed: https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/AgenticEngineeringReference.md
🧰 GSD, the planning/execution workflow framework: https://github.com/gsd-build/get-shit-done

☀️ What I woke up to:

🚀 reposix — a working git-native partial clone + git-remote-helper for REST issue trackers. In a simulated benchmark (a representative 35-tool Jira-shaped MCP catalog vs a reposix shell session for the same task) it used 📉 89.1% fewer tokens. Real-world numbers are still TBD — the simulator isn't the real GitHub API yet — but the direction is clear and the fixture is auditable in the repo.

📦 Shipped overnight:

✅ Rust workspace across 5 crates, unsafe-code forbidden, clippy pedantic clean
✅ 139 passing tests, CI green across rustfmt / clippy / test / coverage / integration
🔒 8 security guardrails enforced and demo-visible (outbound-origin allowlist, append-only audit log, frontmatter field allowlist, bulk-delete cap, rate limits, and more)
🎬 A demo script that runs the full end-to-end story in under 2 minutes
📚 A live MkDocs site with 11 mermaid architecture diagrams on GitHub Pages
📊 A reproducible token-economy benchmark with an auditable fixture

📝 And a full paper trail: per-phase plans, adversarial code reviews, threat model, goal-backward verification — all committed, nothing hand-waved.

🎯 The takeaway isn't just "Claude built a thing overnight." It's that with the right grounding — a clear architectural north star, a disciplined planning framework, and guardrails that are themselves committed artifacts — an agent can run an entire SDLC loop (research → plan → execute → review → verify → ship) without a human in the chair.

🧪 Plenty of kinks left: macOS support, OP-2 dynamic INDEX.md per dir, OP-3 `git pull` cache refresh, OP-9 Confluence comments. The central thesis — "APIs as filesystems" is a real unlock for autonomous agents — has real-backend evidence behind it now (v0.2 real-GitHub, v0.3 real-Confluence, v0.4 nested `pages/`+`tree/` mount layout — all autonomously built).

🔗 Latest handoff Claude wrote for me: https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md
🔗 Repo: https://github.com/reubenjohn/reposix

#AIAgents #ClaudeCode #DeveloperTools #Rust #AgenticEngineering
