<!-- Shard of quality/reports/raise-list-p90.md (P90 RAISE LIST). Index has the other sections. -->
> [Back to RAISE LIST — P90 (index)](../raise-list-p90.md)

## 3. Magic-fixture hazards (path-shape schism)           <!-- 90-05 fills from R2 § E -->

Confirmed **worse than previously filed**: the path-shape schism is **triple**,
not double.

| site | shape produced | file:line |
|---|---|---|
| cache tree builder (real read path) | `issues/1.md` (unpadded, prefixed) | `crates/reposix-cache/src/builder.rs:90,135` |
| `reposix refresh` write path | `issues/00000000001.md` (11-padded) | `crates/reposix-cli/src/refresh.rs:120` |
| push diff planner + fast-import emit | `0001.md` (4-padded, NO prefix) | `crates/reposix-remote/src/diff.rs:106` + `fast_import.rs:63` |

`issue_id_from_path` (`diff.rs:74-77`) parses only bare-integer stems, so on a
real `issues/1.md` tree every prior record fails to match (spurious Creates)
**and the conflict precheck silently skips every real path**
(`precheck.rs:151-152`, `continue`) — push-time conflict detection is
bypassed on real trees, not just the in-memory diff plan. This is QL-001's
root cause (BUG-1), routed to P91 (D90-01).

### HIGH — test green depends on the bug (masks BUG-1; cannot regress-detect it)

| file:line | fixture | why |
|---|---|---|
| `crates/reposix-remote/src/diff.rs:283` | `unchanged_push_emits_no_patches` hand-builds `format!("{:04}.md", …)` | inserts the planner's own bug shape; asserts 0 actions — can never observe BUG-1 |
| `crates/reposix-remote/src/diff.rs:310` | `extra_trailing_newline_is_a_noop` inserts `"0001.md"` | same masking |
| `crates/reposix-remote/src/diff.rs:233-238` | `five_deletes_passes_cap` / `six_deletes_*` | SG-02 bulk-delete cap validated exclusively in the buggy key space |
| `crates/reposix-remote/tests/push_conflict.rs:154` | `one_file_export("0002.md", …)` | the ARCH-08 stale-base regression test only fires the precheck because the bare shape parses; with a real `issues/2.md` tree the stale write would be **silently accepted** — this regression test cannot catch the real-world bypass |
| `crates/reposix-remote/src/precheck.rs:151` | `issue_id_from_path` gate + `:152` `continue` | an entire class of "stale push slipped through" bugs is untestable by construction |
| `crates/reposix-remote/tests/bus_write_happy.rs:251`, `bus_write_sot_fail.rs:244`, `bus_write_mirror_fail.rs:213`, `bus_write_post_precheck_409.rs:227`, `bus_write_audit_completeness.rs:214` | `("0001.md", blob1)…` fixtures | the ENTIRE bus-write fan-out suite (incl. rows graded honest in § 1) rides the bug shape; fan-out/audit/lag asserts are only true in that shape |
| `crates/reposix-remote/tests/protocol.rs:85` | `M 100644 :1 0001.md` | core `export` protocol fixture, same masking |
| `crates/reposix-remote/src/fast_import.rs:63` (+ `main.rs:338` wiring) | emit side writes `{:04}.md` bare | the deprecated `import` transport genuinely emits bare paths — the origin of why bare fixtures "look real" to test authors; the documented primary path (stateless-connect/cache) does not use this shape |

### MED

| file:line | fixture | why |
|---|---|---|
| `crates/reposix-confluence/src/client.rs:1633-1656` | `update_issue_sends_put_with_version` | mock matches method+path only, asserts echoed `version==43`; name + comment (`:1649`) promise a request-body version assert that does not exist — wrong-version sends would pass |
| `crates/reposix-remote/tests/perf_l1.rs:239,303,311` | bare `0001.md` in parsed.tree | the L1 call-count economy test measures the wrong (all-Create) plan if the shape breaks matching, while the count asserts still pass |
| cross-cutting | `builder.rs:90` vs `refresh.rs:120` vs `diff.rs:106`/`fast_import.rs:63` vs attach tests' `issues/0001.md` vs `path.rs:138-140`'s documented "11-digit padded" convention | **no canonical `record_path(id)` helper exists**; four incompatible conventions are each re-derived inline. This is the structural root cause — hazards will recur without it. |

### Fix-first recommendation → P91

Introduce a canonical `record_path(id) -> "issues/<id>.md"` in `reposix-core`;
route the 4 sites (`builder.rs`, `refresh.rs`, `diff.rs`, `fast_import.rs`)
through it; re-key the diff/bus_write/push_conflict/protocol fixtures to the
canonical shape so those tests go correctly **RED until BUG-1 is fixed** —
turning today's silent test-green-masks-the-bug state into an honest
regression gate. Today there is **no push-side test using the real
`issues/<id>.md` shape** — the only real-shape coverage is cache-read-side
(`delta_sync.rs:261`, `tree_contains_all_issues.rs:51`,
`gix_api_smoke.rs:46`) and it never reaches `diff::plan`/`precheck`.

