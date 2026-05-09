# v0.13.2 GOOD-TO-HAVES

> **Purpose.** OP-8 +2 reservation slot — improvements (clarity, perf, consistency, grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items always close; M items default-defer to next milestone. Drained by P107 (good-to-haves polish + milestone close, absorbing both reservation duties at v0.13.2's close).

## GOOD-TO-HAVES-01 — `XLINK-SANITIZE-DOLLAR-VAR-REJECT` — reject `${VAR}` syntax in `grading_context`

**Discovered during:** P102 seeding (Q6 ratification — owner picked cred-hygiene regex only at v1, declined the `${...}` reject)

**Size:** M (~30-50 lines Rust + new regex match-and-block in pre-commit hook + author-discipline doc updates)

**Source:** `.planning/research/v0.13.2-cross-link-fidelity/08-open-questions.md` § "Owner ratification" Q6 — owner ratified cred-hygiene regex pre-commit ONLY at v1; declined the `${VAR}` reject and the 2KB cap. Two leak vectors are knowingly left open at v1: (a) authors templating secrets via `${VAR}` syntax, (b) accidental large log dumps in `grading_context`. Both rely on author discipline + post-leak audit-log forensics. The 2KB cap and `${...}` reject move to **DEFERRED v0.13.3 candidates**. This entry seeds the dollar-var-reject vector.

**Acceptance:**

- Pre-commit hook rejects any `grading_context` block containing `${VAR}` or `${...}` substitution syntax with a clear "templated secret rejected" error message naming the offending file + line.
- Test fixture in `crates/reposix-quality/tests/` covers (a) plain `${VAR}` reject, (b) literal `\${VAR}` escape (allowed), (c) edge case where `${...}` appears inside a code-fenced block (still rejected — frontmatter is structured data, not prose).
- CLAUDE.md + `docs/concepts/cross-link-fidelity.md` author-guide warning updated to remove the "left open at v1" caveat for this vector.
- Re-evaluate after a real leak or a dogfood near-miss surfaces the cost.

**Why deferred from P102:** Owner ratified cred-hygiene regex ONLY at v1 (Q6). Implementing `${VAR}` reject at v1 would expand P102's scope beyond the owner-ratified Q6 envelope. Tracked here so v0.13.3 can close the gap cleanly after a real leak or near-miss provides evidence.

**Default disposition for P107:** Size M; **default-defer to v0.13.3** per OP-8 (M items default-defer; carry-forward target named in CHANGELOG `[v0.13.2]`).

**STATUS:** OPEN

---

## GOOD-TO-HAVES-02 — `XLINK-SANITIZE-2KB-CAP` — 2KB length cap on `grading_context` blocks

**Discovered during:** P102 seeding (Q6 ratification — owner picked cred-hygiene regex only at v1, declined the 2KB length cap)

**Size:** S (~10-20 lines Rust — single length-check in the pre-commit hook + error message + test fixture)

**Source:** `.planning/research/v0.13.2-cross-link-fidelity/08-open-questions.md` § "Owner ratification" Q6 — owner ratified cred-hygiene regex pre-commit ONLY at v1; the 2KB length cap is the second deferred sanitization tier. Accidental large log dumps in `grading_context` (e.g., a stack trace pasted into frontmatter for L3 grading context that includes log lines containing tokens) pass through at v1 and rely on author discipline + post-leak audit-log forensics.

**Acceptance:**

- Pre-commit hook rejects any `grading_context` frontmatter block exceeding 2048 bytes with a clear "grading_context block exceeds 2KB cap" error citing the offending file + byte count.
- Test fixture covers (a) under-cap block (allowed), (b) over-cap block (rejected), (c) edge case where the byte count is computed against the YAML-deserialized payload (not the source frontmatter bytes) so that comments + whitespace don't pad the count.
- CLAUDE.md + `docs/concepts/cross-link-fidelity.md` author-guide warning updated to remove the "left open at v1" caveat for this vector.

**Why deferred from P102:** Owner ratified cred-hygiene regex ONLY at v1 (Q6). The 2KB cap would expand P102's scope beyond the Q6 envelope. Tracked here so P107 can close it if budget permits, else default-defer.

**Default disposition for P107:** Size S; **defer-to-budget at P107** per OP-8 (S items can either close or defer; tighter than M default-defer). If P107's intake is light AND no surprises pulled budget away, close it in-phase. Otherwise defer to v0.13.3 with carry-forward target named in CHANGELOG `[v0.13.2]`.

**STATUS:** OPEN

---

> Add new entries below this line.
