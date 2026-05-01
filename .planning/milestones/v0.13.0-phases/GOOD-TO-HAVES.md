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

---

> Add new entries below this line.
