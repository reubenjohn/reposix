# v0.13.0 Surprises Intake (P87 source-of-truth)

> **Append-only intake for surprises discovered during P78-P86 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. P87 drains this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P88).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P87 should resolve.

**STATUS:** OPEN  (← P87 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## 2026-05-01 09:30 | discovered-by: P80 | severity: LOW

**What:** P80's verifier subagent (verdict GREEN) flagged that the three `agent-ux/mirror-refs-*` verifier shells underwent a shape change vs. the planned design. The plan called for `reposix init <sim-spec>` + `git fetch` + `git push` end-to-end scenarios (mirroring the P79 `reposix-attach.sh` precedent); the executor rewired all three as thin wrappers around the integration tests (`cargo test -p reposix-remote --test mirror_refs <name>`). The change is reasonable — deterministic, faster to run, no `reposix init` flakiness — but it bypasses the dark-factory `reposix init` → `git fetch` end-to-end surface that the P79-style verifier-shell pattern was specifically designed to exercise.

**Why out-of-scope for P80:** Eager-resolution at T04 time forced the shape change because the original `reposix init`-based shell hit a `fatal: could not read ref refs/reposix/main` snag (Q6 from the planner's open-questions). The executor already exceeded the verifier's TINY budget in service of getting one-shape-of-coverage shipped; reverting to the dark-factory shape would have required a deeper dive into the helper's first-push ref-advertisement contract that's already P82's territory.

**Sketched resolution:** P87 evaluates whether the `cargo test`-as-verifier shape is the new house pattern OR a one-off P80 deviation. If house: update CLAUDE.md "Subagent delegation rules" + the verifier-shell convention in `quality/PROTOCOL.md` to name `cargo test` as a sanctioned verifier kind (or document the restriction). If one-off: P86's dark-factory third-arm regression (which DOES exercise `reposix init` + `git fetch` + bus-push end-to-end against a real GH mirror) covers the same surface; P87 confirms by reading P86's verdict and either closes this entry as RESOLVED or files a P88 GOOD-TO-HAVE to reshape the P80 verifiers post-hoc.

**STATUS:** OPEN
