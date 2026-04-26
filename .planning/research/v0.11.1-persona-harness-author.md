# Persona audit: Agent harness author considering reposix integration

**Persona:** OSS maintainer of a coding-agent harness (Claude Code / Cursor / Aider / Continue / Cline) evaluating a built-in `reposix-init` recipe.
**Site walked:** home, `/concepts/*`, `/how-it-works/*`, `/guides/{integrate-with-your-agent,troubleshooting}`, `/reference/{glossary,simulator}`, `/decisions/{001,003,007,008}`, `/benchmarks/v0.9.0-latency`, `/tutorials/first-run`. Plus a quick GitHub repo side-check.

---

## TL;DR

**Maybe — leaning yes for opt-in recipe, no for default-on integration.**
The architecture is genuinely clean (one `reposix init` and the rest is upstream git), but maturity signals push back: repo self-describes as "alpha," still carries its old FUSE description, has zero stars/forks/watchers, real-backend latency cells are empty, and there is no v1.0 stability commitment. I'd ship a documented recipe, not a first-class integration.

---

## Architectural review (clean abstraction or red flags?)

**Genuinely clean.** The "three keys" framing (`clone IS a git working tree` / `frontmatter IS the schema` / `git push IS the sync verb`) survives contact with the how-it-works pages. The filesystem-layer page makes the right concrete promise: `.git/` is real, `git status`/`git diff`/`git stash` work, partial clone is upstream git ≥ 2.27 — no virtual filesystem, no daemon between agent and bytes. The git-layer page documents the three helper-protocol capabilities (`stateless-connect`, `export`, `option`) and is candid about the load-bearing `refs/heads/*:refs/reposix/*` refspec quirk — collapsing it makes `fast-export` emit empty deltas, a silent failure mode they explicitly call out.

**Yellow flags:**

1. **FUSE-era debris is still visible.** ADR-003 ("Nested mount layout") talks about `crates/reposix-fuse/` and FUSE mount roots; the filesystem-layer page says that crate was deleted in v0.9.0, but the ADR is still in the published nav with `Status: Accepted`. The GitHub repo description still reads *"Git-backed FUSE filesystem exposing REST APIs as POSIX files."* The doc set telling two different stories about the architecture is a tell.
2. **Tutorial vs. marketing inconsistency.** `/tutorials/first-run/` step 4 requires `git checkout -B main refs/reposix/origin/main`. The home page and mental-model page show `git checkout origin/main`. Exactly the kind of thing an automation script trips on.
3. **Tutorial assumes the sim is running before `reposix init sim::demo`,** but never documents what `init` does when it's not — block, retry, or fail-fast with a parseable error? Unspecified.
4. **git ≥ 2.34 requirement.** `reposix doctor` checks git version (good), but a harness shipping to 10k users will have a non-trivial tail of older-git environments — Ubuntu 20.04, RHEL 7-class hosts — and the doc story for them is missing.

---

## Stability + maintenance signals

**Mixed.** I want to ship against this; the artifacts say *don't yet*.

**Positive:** last commit today (audit date 2026-04-26). 712 commits, 8 ADRs, dense planning tree. ADR-008 (2026-04-24) shows a working ratification process. `reposix doctor` exists and exit-codes 0/1 — CI-gateable.

**Negative:**
- Home page self-describes as **alpha**.
- **Zero stars/forks/watchers** on GitHub. If I ship to 50k users, I am the entire ecosystem.
- **No v1.0 stability commitment.** I scanned `/decisions/` for an "API surface stability" or deprecation-policy ADR. None. ADR-008 itself documents that the URL form *changed shape* in v0.10.0 (added `/confluence/` and `/jira/` markers, old form hard-fails) — a breaking change to `remote.origin.url`, a load-bearing surface, inside a minor bump. No public contract that `reposix init github::owner/repo` written today will be parseable by next year's helper.
- **Real-backend latency cells empty** in `/benchmarks/v0.9.0-latency/`. Only `sim` is populated (24ms init / 9ms list / 8ms get / 8ms PATCH / 5ms capabilities). I cannot tell my users *"GitHub round-trip will be ~X ms."*
- **Bus factor of one.** Single contributor; the repo description still says *"Git-backed FUSE filesystem."*
- The site header still shows `v0.8.0` while docs reference v0.9.0 architecture and v0.11.0 features.

**Will `reposix init <backend>::<project>` still work in 6 months?** Invocation form probably yes; ADR-008 already moved URL-form goalposts and nothing publicly commits against another move. **My bet: 12-month survival yes; flag-stability for harness automation, 60–70%.**

---

## Automation-friendly?

**Partially.**

**What works:**
- `reposix doctor` exits 1 on ERROR, 0 otherwise — CI-gateable.
- `reposix init` is a one-shot bootstrap (`git init` + `extensions.partialClone=origin` + `remote.origin.url` + `git fetch --filter=blob:none`).
- Errors are *intentionally* parseable as standard git errors (`fetch first`, `refusing to fetch N blobs`). Dark-factory framing means recovery is literally what git would tell a human.
- Audit log is a plain SQLite file at `<XDG_CACHE_HOME>/reposix/<backend>-<project>.git/cache.db` with a documented `audit_events_cache` schema — queryable post-run.

**What breaks at scale (1000 invocations):**
1. **No documented exit codes for `reposix init`.** Integrate guide, first-run tutorial, troubleshooting — none enumerate them. I want to differentiate sim-not-running, network-down, bad-URL-form, partialClone-not-supported (old git), already-initialized-directory.
2. **No machine-readable output.** No `--json`, no NDJSON stderr. The integrate guide explicitly says *"surface helper stderr to the model verbatim."* Fine for an in-loop LLM, hostile to programmatic recovery.
3. **Error strings are not specified as stable.** Troubleshooting shows `error: refusing to fetch 487 blobs (limit: 200).` but doesn't commit to that punctuation. Regex-matching that today is fragile.
4. **Env vars are split across pages.** `REPOSIX_BLOB_LIMIT`, `REPOSIX_ALLOWED_ORIGINS`, `REPOSIX_CACHE_DIR`, `GITHUB_TOKEN`, `ATLASSIAN_API_KEY`/`_EMAIL`/`REPOSIX_CONFLUENCE_TENANT`, `JIRA_EMAIL`/`_API_TOKEN`/`REPOSIX_JIRA_INSTANCE`, `JIRA_TEST_PROJECT`/`REPOSIX_JIRA_PROJECT` — no single `/reference/env-vars/` page.
5. **Concurrency unaddressed.** 50 simultaneous `reposix init` against one sim, or two users sharing a host both pointing at `<XDG_CACHE_HOME>/reposix/github-owner-repo.git/` — cache stampede behavior is not specified.
6. **Already-initialized directory case is undocumented.** What does `reposix init` do when `<path>/.git/` exists — refuse, re-init, update remote URL?

---

## Security story for end-users

**Strongest section of the docs.** The trust-model page is unusually honest: concentric-rings diagram, explicit lethal-trifecta framing (Simon Willison citation), `Tainted<T>` newtype with a trybuild compile-fail test, append-only SQLite triggers (`BEFORE UPDATE/DELETE RAISE`), single egress chokepoint (`reposix_core::http::client()`, `clippy::disallowed_methods` enforces no direct `reqwest::Client::new()`).

**Talking points I could give users with a straight face:** allowlisted egress (`REPOSIX_ALLOWED_ORIGINS` defaults loopback-only); server-controlled fields (`id`, `created_at`, `version`, `updated_at`) stripped on push so a poisoned body can't rewrite metadata; append-only audit table with documented `op` vocabulary; push-time conflict detection rejects stale-base writes with the standard git `fetch first` error.

**Caveats I'd have to own:** the trust-model page's *"What's NOT mitigated"* section is forthright — shell access bypasses every cut; *"reposix is a substrate for safer agent loops, not a sandbox"*; the simulator is itself attacker-influenced; confused-deputy across multiple backends; cache compromise (full file swap) defeats append-only triggers.

This is the reason I'd consider an opt-in recipe rather than declining outright.

---

## The "integrate with your agent" guide — completeness audit

**It's a pointer page, not a recipe.** Quote: *"Full vetted recipes (Claude Code, Cursor, Aider, Continue, Devin, SWE-agent CI fixtures) ship in v0.12.0; this is the pointer page."* The page knows it isn't done.

**Does well:** three named patterns (Claude Code skill / Cursor shell loop / custom SDK loop) with correct architectural advice ("don't register reposix as a tool with the model — the substrate is the point"); concrete gotchas (don't truncate stderr, don't add `reposix list` to the allow-list, watch for `egress_denied` audit rows); honest "What integration is NOT" disambiguation from MCP servers, custom CLIs, sandboxes.

**Missing for me to ship:**
- **No post-init detection sequence.** The page never says how to verify a healthy working tree post-init. (Presumably `reposix doctor`, but the integrate page doesn't link it.)
- **Pseudo-code, not runnable code.** The custom-SDK-loop section is `init := subprocess.run([...])`, not a compilable example.
- **No teardown story.** Cache location, growth bounds, cleanup procedure on harness uninstall. (`reposix gc` exists in troubleshooting but isn't linked.)
- **No version-pinning advice.** `>= X.Y, < Z`? `cargo binstall`? Vendor binary? Installer? Unanswered.
- **No "is reposix installed" check in-harness.** `which reposix` works, but it's not mentioned.

---

## What's missing from the docs that would block integration

In rough priority order:

1. **Documented exit codes for `reposix init` and `reposix doctor`.** Table form: code → meaning → recommended retry strategy. Without this I can't write reliable automation.
2. **`--format=json` / NDJSON stderr** for `reposix init`, `reposix doctor`, and helper protocol error events. Stable schema, versioned. The dark-factory teaching strings can stay; add a parallel structured stream.
3. **Stability commitment / SemVer policy ADR.** "Through v1.0, the following surfaces are frozen: …" — at minimum: `reposix init <backend>::<project> <path>` invocation form, `remote.origin.url` written shape, `audit_events_cache` schema, env var names, doctor exit codes. ADR-008's URL-shape change is a cautionary tale.
4. **Single canonical env-var reference page.** `/reference/env-vars/` enumerating every `REPOSIX_*` and credential var.
5. **Concurrency contract.** What's safe to do in parallel? What about per-user cache isolation on shared hosts?
6. **Real-backend latency numbers.** The empty cells in `/benchmarks/v0.9.0-latency/` directly undercut the claim that the architecture works against real backends. Even one populated row would help.
7. **A version-stability badge** on the home page and in the GitHub README. The repo description still says *"Git-backed FUSE filesystem"* — the v0.9.0 pivot has not propagated to the front door.
8. **A `reposix --version` invocation note in the integrate guide** so harness preflight checks know the minimum version they need.
9. **Tutorial / mental-model coherence.** Either the tutorial uses `git checkout -B main refs/reposix/origin/main` and the home page should too, or both should standardize on `git checkout origin/main`. Pick one.
10. **A `reposix doctor --json` mode** that emits findings as structured records — feed straight into a harness's UI.

---

## What the persona would request from upstream

A short note I'd file as a GitHub issue under `reubenjohn/reposix`:

> **Title:** Harness-author integration: stability commitments and machine-readable interfaces
>
> Hi — auditing reposix as a built-in `reposix init` recipe for our agent harness. The architecture is excellent and the trust model is the most thoughtful I've seen in this category. Three asks before we'd ship:
>
> 1. **Publish a stability ADR.** Pin the `<backend>::<project>` invocation form, the URL shape written into `remote.origin.url`, the `audit_events_cache` schema, and the doctor/init exit codes through v1.0. ADR-008 already moved the URL shape once inside a minor; we need a contract before we lock our user-facing recipe to a specific call form.
> 2. **Add `--format=json` to `reposix init` and `reposix doctor`.** We surface raw stderr to the LLM in agent loops (great), but we also want programmatic recovery in our orchestrator (e.g. detect missing-creds → prompt the user to set env vars). A versioned JSON schema solves it.
> 3. **Fill the real-backend latency cells.** The `/benchmarks/v0.9.0-latency/` page advertises an envelope that's blank for GitHub/Confluence/JIRA. Without a single real number, we can't tell our users what to expect.
>
> Bonus asks: a single `/reference/env-vars/` page; explicit concurrency/per-user-cache semantics for shared hosts; bring the GitHub repo description in line with the v0.9.0 pivot (it still says "Git-backed FUSE filesystem").
>
> Happy to be a design partner on any of these.

---

**Bottom line:** I'd ship a contributed recipe in `~/recipes/reposix.md` that says *"experimental, here's how to wire it up against the simulator, real backends are advanced-mode."* I would not make `reposix init` a default-installed verb in my harness until items 1–3 above are upstream.
