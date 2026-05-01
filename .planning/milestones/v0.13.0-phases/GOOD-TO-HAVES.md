# v0.13.0 GOOD-TO-HAVES

> **Purpose.** OP-8 +2 reservation slot 2 — improvements (clarity, perf, consistency, grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items always close; M items default-defer to next milestone. Drained by P88 (good-to-haves polish + milestone close).

## GOOD-TO-HAVES-01 — extend `reposix-quality bind` to support all catalog dimensions

**Discovered during:** P79-02 (2026-05-01)

**Size:** S (~30-50 lines Rust)

**Source:** `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools validate and mint" requires catalog rows to be MINTED via the `reposix-quality bind` verb (NOT hand-written into JSON). Today, `bind` only supports the `docs-alignment` dimension. P79-02 needed to mint a row in `quality/catalogs/agent-ux.json` (`agent-ux/reposix-attach-against-vanilla-clone`), but had to hand-edit JSON because `bind --dimension agent-ux` is not implemented. The hand-edit was annotated as "Hand-edit per documented gap (NOT Principle A)" in commit `1812647`'s message and in the row's `_provenance_note` field. Without this extension, every future agent-ux / release / code / structure / etc. catalog row will continue bypassing Principle A.

**Acceptance:**

- `reposix-quality bind --dimension <agent-ux|release|code|structure|docs-build|docs-repro|perf|security>` validates citations against the live filesystem (where applicable) and mints a row into the dimension's catalog at `quality/catalogs/<dimension>.json`.
- Refuses on invalid dimension or invalid citation.
- Test fixture in `crates/reposix-quality/tests/` covers at least one non-`docs-alignment` dimension end-to-end.
- The P79-02 hand-edit of `agent-ux/reposix-attach-against-vanilla-clone` is rebound via the new code path (provenance flag retained or auto-cleared).

**Why deferred from P79:** the gap is not load-bearing for `reposix attach` correctness — the row exists, the verifier runs, the catalog row reads PASS at phase close. Extending `bind` would have doubled P79-02's scope. Tracked here so the next milestone can close the Principle A gap cleanly.

**Default disposition for P88:** Size S; close in P88 if budget permits, else default-defer to v0.14.0 per OP-8 (M items default-defer; S items can either go either way).

**STATUS:** DEFERRED to v0.14.0 (P88 close 2026-05-01)

**Rationale.** P88's scope is docs + catalog + shell only — no Rust code changes per the milestone-close charter (CLAUDE.md OP-9 + ROADMAP P88). Extending `reposix-quality bind` to all 8 catalog dimensions requires (a) ~30-50 lines of Rust spanning the `bind` verb's dispatch + per-dimension validation paths, (b) cross-dimension schema design (each catalog has its own row shape — agent-ux uses `command/expected.asserts/verifier.script` while doc-alignment uses `source/test/source_hash`), and (c) a new test fixture in `crates/reposix-quality/tests/` covering at least one non-`docs-alignment` dimension end-to-end. The work is well-scoped at S size but doesn't fit P88's pure-docs envelope; doing it here would double the phase's scope per the OP-8 "scope-creep-to-fit-the-finding" anti-pattern. v0.14.0 (`observability-and-multi-repo`) carries the gap forward with the existing GOOD-TO-HAVES-01 acceptance criteria intact. Until then, hand-edits to `agent-ux/release/code/structure/docs-build/docs-repro/perf/security` catalog JSON files continue to carry the `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"` field documented at v0.13.0 P79 + reused throughout P80–P88.

If v0.14.0 budget tightens, can move to v0.14.x polish slot — the gap is operationally tolerable (every row that needs hand-editing carries the provenance flag, audit trail intact) and only blocks the cleaner provenance story, not any substantive Principle A invariant.

---

> Add new entries below this line.
