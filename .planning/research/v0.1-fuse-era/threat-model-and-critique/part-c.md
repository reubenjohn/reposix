← [back to index](./index.md)

## PART C — Unbiased Critique of "Demo by 8am"

PROJECT.md L29: **"Demo-ready by 2026-04-13 morning."** L50: **"Demo by 2026-04-13 ~08:00 PDT. Hard limit. Project kicked off 2026-04-13 ~00:30 PDT. ~7 hours of autonomous build time."**

Active scope (lines 21–29):
1. Simulator-first architecture (HTTP fake with rate limits, 409s, workflow rules, RBAC).
2. Issues as Markdown + YAML.
3. FUSE mount with full r/w (`getattr`, `readdir`, `read`, `write`, `create`, `unlink`).
4. `git-remote-reposix` helper (full git remote helper protocol with conflict surfacing).
5. CLI orchestrator (`sim`, `mount`, `demo`).
6. SQLite audit log.
7. Adversarial swarm harness.
8. CI on GHA: lint, test, integration test that **mounts FUSE in the runner**, codecov.
9. Demo recording + walkthrough doc.

### C1. Where this is most likely to fail

**FUSE in GitHub Actions.** GHA `ubuntu-latest` runners do support FUSE in some configurations, but `fusermount` requires `/dev/fuse` to be present and the runner user to be in the `fuse` group, or `--privileged` for the container. Hosted GHA runners run rootless inside a Docker-on-LXC sandwich — `/dev/fuse` may exist but may not be mountable without special action. **This is the single most likely cause of "CI is red at 7am".** Estimated 1–2 hours of yak-shaving alone, often more.

**`git-remote-helper` protocol depth.** This is not a small protocol. Implementing `capabilities`, `list`, `fetch`, `push` (with refspec parsing, force/delete handling, atomic mode), and the streamed pkt-line responses is a multi-day task even for someone who has done it before. The `docs/research/initial-report.md` description (§"Internal Mechanics of Git Remote Helpers") is glossing over the wire format. A naive implementation will work for `git fetch` of a single ref and break on anything else.

**Conflict resolution end-to-end.** The `docs/research/initial-report.md` §"Native Git Conflict Resolution" claim that conflicts surface as standard git markers requires the remote helper to fabricate a "remote tree" that diverged from the local tree, with content chosen so that `git merge` produces the right markers. This is real engineering. Estimated 2–4 hours minimum to get a single demo case working; not generalizable in 7 hours.

**Adversarial swarm harness.** Listed as "Active". Implementing a load generator that "spawns N agent-shaped clients" is a half-day on its own. With a 7-hour total budget and 8 other things on the list, this is the most cuttable.

**Audit log queryability.** Just "log to SQLite" is fine. "Queryable" with documented schema, with redaction (per A5 / PART D #12), is half a day.

**Demo recording.** Asciinema recordings need to be re-shot when anything breaks. Plan for 30 minutes of recording + edit even if everything works. If anything breaks during recording, add another hour.

### C2. Minimum viable demo that's still credible

**Cut to this scope. Anything beyond is gravy.**

| Keep | Cut | Defer to v0.2 |
|------|-----|---------------|
| Simulator with at least: list, get, create, update issue. **No** rate limits, **no** 409s, **no** workflow validators, **no** RBAC. | Adversarial swarm harness | Real backends |
| Read-only FUSE mount: `getattr`, `readdir`, `read`. **No write.** | Full `git-remote-helper` protocol | Write path through `git push` |
| Issues as Markdown + YAML — read direction only | CI integration test that mounts FUSE in GHA | RBAC → POSIX mapping |
| CLI: `reposix sim`, `reposix mount`, `reposix demo` | Codecov | git-bug-style Lamport timestamps |
| Audit log of every read (append-only JSONL is fine, SQLite is gravy) | Conflict resolution via git semantics | Confluence draft → branch mapping |
| One golden-path demo: start sim, mount, `cat /mnt/reposix/PROJ-1.md`, `grep -r database /mnt/reposix`, show audit log | Asciinema if time permits; static screenshots if not | |
| README + 5-minute walkthrough doc | | |
| CI: just `cargo test` and `cargo clippy` on GHA, no FUSE mount in CI | | |

**Why this is still credible:**

- It demonstrates the central thesis (PROJECT.md L9): "An LLM agent can `ls`, `cat`, `grep` issues in a remote tracker without ever seeing a JSON schema or REST endpoint" — read path only is enough for the value prop.
- It has the audit log (OP #6 ground truth).
- It has CI that's actually green (vs. CI that's red because of FUSE-in-Docker issues, which would undermine the whole project's credibility).
- It honestly acknowledges write is harder and defers it. **Honesty about scope is a credibility multiplier; over-promising and under-delivering at 8am is a credibility killer.**

**What the agent should do at hour 4 if write is not working:** rip it out, ship read-only, write a "Roadmap" section in README that promises write in v0.2 with a candid explanation of the lethal-trifecta engineering it requires.

### C3. Schedule risk register

| Hour | Planned | Realistic risk |
|------|---------|----------------|
| 0–1 | Workspace + simulator skeleton | OK |
| 1–2 | FUSE read path + simulator GET/list | FUSE crate compile errors (`fuser` features) eat 30 min |
| 2–3 | FUSE write path + simulator POST/PUT | High risk of write semantics rabbit-hole; this is where you cut |
| 3–4 | git-remote-helper | **Highest single risk.** 50/50 it doesn't ship |
| 4–5 | CLI + audit log | OK |
| 5–6 | CI | FUSE-in-GHA may eat the whole hour |
| 6–7 | Demo recording + README | If anything earlier ran over, this slips and the demo is unrehearsed |

**Recommendation:** at hour 3, hard-stop. If write or remote-helper aren't both demo-able, cut them both and ship read-only. Don't decide at hour 5; you'll be too sunk-cost-deep.

### C4. Things the plan doesn't budget for

- Writing a PR description, release notes, or CHANGELOG.
- Verifying the asciinema recording renders correctly on GitHub README (it often doesn't).
- Verifying badges resolve (per CLAUDE.md OP #1).
- Re-recording the demo after a typo is found.
- Sleep.
