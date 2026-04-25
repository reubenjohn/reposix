# Agentic Engineering Reference

!!! note "About this document"
    This is **not a guide to using reposix.** It is a reference on the
    *engineering pattern* (the dark factory) that reposix was built to
    serve. Read this if you want to understand the architectural reasoning
    behind reposix or are building similar agent infrastructure yourself.
    If you just want to use reposix, start with the
    [tutorial](../tutorials/first-run.md).

Distilled from a Simon Willison interview (Lenny's Podcast, Apr 2026). Kept only the material relevant to running autonomous / semi-autonomous coding agents in production — patterns, anti-patterns, and security constraints. Not a summary of the whole conversation.

---

## 1. The dark factory pattern

The frontier question: how do you ship good software when no human reads the code?

StrongDM is the most-cited live example (security / access-management software, **not** a safe domain to vibe code). Their ruleset:

1. **Nobody writes code.** Humans prompt; agents type. Practical today — latest models are fast enough that asking for a rename/refactor beats typing it.
2. **Nobody reads the code.** This is the actually hard one. Requires replacing code review with other quality signals.

How StrongDM replaced code review:

- **Simulated QA swarm.** Thousands of agent "employees" in a simulated Slack channel, 24/7, filing requests like "give me access to Jira." ~$10k/day on tokens. Acts as a never-sleeping QA team exercising the real product.
- **Simulated external dependencies.** They didn't test against real Slack/Jira/Okta — rate limits would kill a 10k-agent swarm. Instead, they fed the public API docs + OSS client libraries to their coding agent and told it to build in-process fakes. Result: a tiny Go binary that simulates the entire integration surface, free to run, with a vibe-coded UI for observability.

Takeaways for the reposix project:
- Build simulators, not mocks. A FUSE-exposed Jira/GitHub/Confluence needs a **local fake server** that behaves like the real API (rate limits, workflow rejections, 409 conflicts). Then a swarm of agent-users can hammer the filesystem against it overnight without consuming real API quota.
- "Factory" is the wrong word for product polish — use **artisanal** for customer-facing surfaces, **factory** for invariant enforcement (tests, simulators, fuzzers).

---

## 2. Writing code is cheap — design around that

The biggest mindset shift. Implications:

- **Prototyping is free.** For any non-trivial design choice, build 3 versions and play with them. Picking between options beats arguing about them.
- **Uninterrupted focus blocks no longer matter.** You prompt for 2 minutes, then do something else while agents work. Plan for fragmented attention.
- **Estimation intuition is dead.** A task that "would take two weeks" may take 20 minutes. Throw work at agents that you think they can't do — when they succeed, you learn a new capability frontier; when they fail, you know which model / mode isn't there yet.
- **Over-testing is now cheap.** 100+ tests on a small library used to be a maintenance liability. It isn't anymore — the agent updates the tests when the code changes.

Corollary: the old "good-code signals" (thorough tests, solid docs, passing CI) no longer prove the code is *trustworthy*. They can all be generated in an hour. The new scarce signal is **proof of usage**: has the author or anyone else actually run this thing for a while? Simon marks never-used code as `alpha` even if the test suite is green.

---

## 3. Agentic engineering patterns

### 3.1 Start projects with a thin template

Agents copy existing patterns aggressively. A single example file anchors style better than paragraphs of CLAUDE.md prose.

- New project should start with: one passing test (`assert 1 + 1 == 2` is enough), preferred formatting, preferred directory layout, one representative module. Nothing more.
- Agents will pick up your indentation, naming, test style, and extend in the same shape.
- Simon keeps templates per language/runtime on GitHub and starts from them.

### 3.2 Red/green TDD as a prompt idiom

- Have the agent **write the test first, run it, watch it fail, then implement, then watch it pass.**
- This catches skipped assertions and unnecessary code.
- Shorthand: just say "use red/green TDD" — models know the jargon and will follow the discipline.
- Do not drop tests as a speed tactic. Teams that do get faster short-term and slower long-term — tests are what let the agent refactor without breaking things.

### 3.3 Hoard everything you've tried

Value comes from a large library of "I've solved something shaped like this before." Maintain it explicitly:

- Public GitHub repos as a personal knowledge base. Simon's structure: `simonw/tools` (one-off HTML/JS utilities, ~193 of them) and `simonw/research` (agent-produced markdown reports from coding-agent research tasks).
- **Research entries must involve running code**, not just "deep research" prose. A plot, a benchmark number, a working prototype. Otherwise it's LLM vomit with no signal.
- When tackling a new problem: point the agent at relevant entries (`"check out simonw/research, find the web-assembly ones, combine with…"`). Agents are now excellent at pulling and recombining context from repos.

### 3.4 YOLO mode is the unlock

Agents that ask for permission on every file edit are unusable at scale. The productivity jump comes from turning that off:

- Claude Code: `--dangerously-skip-permissions`. OpenAI: `--yolo`.
- Safest venue: **Claude Code for Web** (or equivalent). Agent runs on the provider's VM, not your laptop. Worst case, it trashes a disposable environment.
- Run 3–4 agents in parallel on separate problems. Review via GitHub PR flow, not by babysitting the terminal.
- Laptop YOLO is acceptable in a Docker container with a scoped workdir and no network access to private services. Never on your main machine with real creds mounted.

### 3.5 Mind your own exhaustion

Running 4 parallel agents well is **more** mentally demanding than writing code by hand, not less. Simon reports being wiped out by 11 a.m. Cognitive limit is on how much you can hold in your head, not on typing speed. Design for sustainable pace — the novelty of "my agents could be working right now" burns people out fast.

---

## 4. Model / tool notes (current as of April 2026)

- GPT-5.1 + Claude Opus 4.5 were the Nov 2025 inflection point. GPT-5.4 and Claude Opus 4.6 are roughly at parity; either is fine for serious coding work.
- Claude Code (both local and hosted) and OpenAI Codex are "almost indistinguishable" in capability. Choice is mostly taste and ecosystem.
- Nano Banana (Gemini image model) is the tool of choice for image generation; Simon uses only for non-serious output because models hallucinate image details.
- Turn **memory features off** if you're benchmarking or writing about model behavior — otherwise you're testing a bespoke model, not the one your readers have.
- For research/lookup, the major chat models with search are now better than Google for most questions. Verify before publishing.

---

## 5. Security — the part you cannot ignore

### 5.1 Prompt injection

- Simon coined the term in 2022; he regrets it because (a) it implies SQL-injection-style solvability (there is no such solution) and (b) people misuse it to mean jailbreaking.
- Core problem: LLMs cannot reliably distinguish instructions the developer placed in the prompt from instructions that arrived inside retrieved content (email body, scraped web page, issue description). Any of the latter can override the former.

### 5.2 The lethal trifecta

A system is in danger if an agent has **all three** of:

1. **Access to private data** (your inbox, internal docs, credentials, customer DB).
2. **Exposure to untrusted input** (anything that might contain attacker-supplied text — emails, web pages, tickets, PRs).
3. **An exfiltration channel** (ability to send data outward — reply to email, write to a public resource, make an HTTP call).

Remove at least one leg. Exfiltration is usually the easiest to cut.

### 5.3 Filter effectiveness

- "97% effective" prompt-injection filter = **failing grade**. 3% of attacks still work, and attackers iterate.
- You cannot deny-list every phrasing of an attack — it's free-form text in any human language.
- Assume: **anyone who can get text into the agent's context can make it do anything the agent is authorized to do.** Design blast radius accordingly.

### 5.4 The CaMeL pattern (Google DeepMind, the one promising direction)

Architecture that assumes prompt injection is unfixable:

- **Privileged agent**: talks to the user, can take real actions, never sees untrusted text directly.
- **Quarantined agent**: sees the untrusted content (email body, scraped page), extracts structured data only.
- Privileged agent emits a small code-like plan (`do X, then Y, then Z`) that is executed with **taint tracking** — actions touching tainted data require human approval.
- Human-in-the-loop only on high-risk, tainted steps (not on every action — click-fatigue is itself a failure mode).

### 5.5 Normalization of deviance

- Every deployment of an unsafe agent that doesn't get exploited **increases** institutional confidence in it. This is the Challenger O-ring dynamic.
- The field has been getting away with unsafe patterns because no headline-grabbing exploit has landed yet. One will. Don't rely on "it hasn't happened" as evidence of safety.

### 5.6 Reposix-specific security implications

This project is a giant lethal-trifecta machine if built naively:

- Private data: mounted FUSE is exposing Jira/Confluence/email-adjacent content.
- Untrusted input: every remote ticket/comment/PR is attacker-influenced text.
- Exfiltration: the agent can `git push` arbitrary content to arbitrary remotes.

Design consequences:
- The FUSE daemon should treat remote content as **tainted by default**. Tainted content should be readable but not re-routable — no echoing issue bodies into `git push` destinations the user didn't pre-authorize.
- Consider a quarantined-agent split: one agent reads/parses tickets, a different agent (never seeing raw ticket text) decides pushes.
- The POSIX-permission layer (RBAC → chmod) helps against honest mistakes, not against prompt injection.
- Audit log (AgentFS / SQLite WAL style) is non-optional. Every read, every translation, every outbound request.

---

## 6. What an AI-era product team looks like

- **Seniors and juniors both benefit; mid-career is most at risk.** Seniors amplify existing taste; juniors onboard in a week instead of a month. Mid-career has neither amplifier.
- **Code is no longer the bottleneck** — everything else is. Expect spec-writing, product sense, and usability testing to become the new critical path.
- **Ideation**: AI is strong for the boring first two-thirds of brainstorming. Force it past the obvious (ask for 20 more after the first 20; cross-pollinate with unrelated domains).
- **Usability**: still needs real humans. AI "playing the user" is not a substitute.
- **Ambition should go up, not down.** The productivity gain is wasted if you just do last year's roadmap faster.

---

## 7. Concrete operating rules distilled

Rules to hand to an autonomous overnight agent:

1. Start every new module from a minimal working template with one passing test.
2. Red/green TDD for every non-trivial function; "red/green TDD" is sufficient instruction.
3. Build a local simulator for every external API before wiring real credentials.
4. Run agents in YOLO mode only inside disposable environments (container, VM, hosted agent). Never against a machine with live credentials to production systems.
5. Treat any content fetched over the network as tainted. Do not route tainted content into actions that cause side effects in other systems.
6. Prefer three cheap prototypes over one deliberated design.
7. Never declare a feature working without running it end-to-end in a real (or realistic simulated) environment. Tests passing ≠ feature working.
8. Mark unused-by-author code as `alpha` regardless of test coverage.
9. Commit research artifacts (benchmark outputs, exploration notes, simulator logs) to a `research/` directory so future agent sessions can grep them.
10. Keep an audit log of every network-touching action. Structured, queryable (SQLite or JSONL).
