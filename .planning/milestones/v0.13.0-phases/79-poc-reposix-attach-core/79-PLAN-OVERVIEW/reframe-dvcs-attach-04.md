← [back to index](./index.md)

# Reframe of DVCS-ATTACH-04 (per checker B2)

**Original sketch acceptance** read "all materialized blobs are wrapped in
`Tainted<Vec<u8>>`" — but `Cache::build_from` per its contract does NOT
materialize blobs (lazy by design; only `Cache::read_blob` materializes,
and only when git invokes the cache via the helper). So the original
"compile-time guarantee" test in T03 was vacuously satisfied (nothing was
ever materialized during attach itself).

**Reframed acceptance for DVCS-ATTACH-04 (resolved via path b):**

> *"The cache materialization API used by `attach` (the `Cache::read_blob`
> path that git invokes lazily) returns `Tainted<Vec<u8>>` per OP-2. Verified
> by BOTH (1) a static type-system assertion in a unit test that imports
> `reposix_core::Tainted` and asserts the function signature; AND (2) an
> integration test that exercises `attach` then forces a single blob
> materialization via the helper-equivalent path and asserts the bytes are
> tainted."*

**Architectural rationale:** blobs are lazy. The `Tainted` contract belongs
to `read_blob`, not to `attach` itself. The integration test forces ONE
materialization (cheapest possible exercise of the lazy path) so the
runtime assertion has a concrete byte stream to grade.

This reframe:
- Closes the vacuity gap.
- Costs ≤ 30 lines of test code (type assertion in 79-02 T03; integration
  test in 79-03 T02).
- Does NOT require changing `Cache::read_blob` (already returns
  `Tainted<Vec<u8>>` at `crates/reposix-cache/src/builder.rs:436`).

The orchestrator updates `.planning/REQUIREMENTS.md` DVCS-ATTACH-04 row to
reflect this reframed acceptance BEFORE the verifier subagent grades P79.
This is an orchestrator-level edit (top-level coordinator action), not a
plan task.
