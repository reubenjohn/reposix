← [back to index](./index.md)

# New GOOD-TO-HAVES entry (per checker B3)

The original 79-02 T01 cited `quality/PROTOCOL.md` § Principle A
("Subagents propose; tools validate and mint") as the catalog-mint path,
but immediately admitted that `reposix-quality bind` only supports the
`docs-alignment` dimension — leaving agent-ux dim mints as hand-edited
JSON, which violates Principle A while citing it.

**Resolution: option (b) — acknowledge the gap.** The 79-02 T01 task
hand-edits `quality/catalogs/agent-ux.json` with an explicit annotation
that this is a documented gap (NOT a Principle A application), and the
orchestrator files a GOOD-TO-HAVES entry tracking the bind-extension work
for a future polish slot (P88 or later).

**GOOD-TO-HAVES entry to be filed at
`.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` (created lazily
when first entry surfaces):**

> ### GOOD-TO-HAVES-01 — `reposix-quality bind` agent-ux dim support
>
> - **Discovered by:** Phase 79 plan-checker (B3).
> - **Size:** S (~30-50 lines of Rust in `crates/reposix-quality/src/`).
> - **Impact:** consistency — closes Principle A bypass for agent-ux dim
>   mints. Today, agent-ux + perf + security + agent-ux dims are
>   hand-edited JSON; only docs-alignment dim has `bind` verb support.
> - **Why deferred:** keeping P79 scope tight; mint extension is its own
>   work that fits cleanly in a polish slot.
> - **Eager-resolution gate:** would have eaten >1h into P79's scope and
>   introduced a new dependency (the quality binary's verb dispatch path).
>   Defers cleanly to P88 (good-to-haves polish slot of v0.13.0) per OP-8.
> - **Acceptance:** `reposix-quality bind --dimension agent-ux <id> --command <cmd> --verifier <path>`
>   exists and mints the row with required schema fields. Unit test in
>   `crates/reposix-quality/`. Once shipped, the existing hand-edited
>   agent-ux row from P79 can be retroactively re-bound via the verb
>   (no row content change; provenance trail tightens).

The 79-02 T01 task does NOT cite Principle A; instead it annotates:
"Hand-edit per documented gap; tracked in GOOD-TO-HAVES-01 until
`reposix-quality bind` supports agent-ux dim."

The orchestrator (top-level) files the GOOD-TO-HAVES.md entry as part of
the phase-close ritual — this is NOT a plan task.
