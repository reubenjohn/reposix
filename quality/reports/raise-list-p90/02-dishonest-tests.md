<!-- Shard of quality/reports/raise-list-p90.md (P90 RAISE LIST). Index has the other sections. -->
> [Back to RAISE LIST — P90 (index)](../raise-list-p90.md)

## 2. Dishonest tests — test-name-vs-asserts baseline     <!-- 90-05 fills from R2 § C -->

This section is the **human-facing twin of 90-03's committed baseline file**
(`quality/gates/agent-ux/test-name-vs-asserts.baseline`) — the two are kept
**identical, 4 entries**, confirmed by direct file comparison in this dispatch:

| # | file:line | test fn | body actually does | verdict |
|---|---|---|---|---|
| 1 | `crates/reposix-cli/tests/agent_flow.rs:103` | `dark_factory_sim_happy_path` | `#[ignore = "spawns reposix-sim child"]` (sim-child, NOT a real-backend gate); `reposix init` then asserts **only git config** (partialClone/promisor/filter/URL prefix); the test's own comment (`:117-119`) admits the trailing fetch fails and that's tolerated. No push, no fetch. | **DISHONEST** — "happy_path" implies the flow completes; body is config-only. |
| 2 | `crates/reposix-cli/tests/doctor.rs:194` | `doctor_blob_limit_zero_warns_on_real_remote` | `reposix doctor` on a config whose remote URL is a github **string**; "real remote" = URL-classification, no network, and NOT `#[ignore]`-gated. | **DISHONEST** — "real_remote" over-promises network involvement. |
| 3 | `crates/reposix-cli/tests/agent_flow.rs:174` | `dark_factory_blob_limit_teaching_string_present` | `fs::read_to_string` of `stateless_connect.rs`; source-string asserts only. | **BORDERLINE** — the `_teaching_string_present` suffix is itself honest; this RAISEs only because the `dark_factory` prefix matches the F-K8 name-token regex. |
| 4 | `crates/reposix-cli/tests/agent_flow.rs:203` | `dark_factory_conflict_teaching_string_present` | reads `main.rs`/`write_loop.rs`/`bus_handler.rs` source for strings. | **BORDERLINE** — same rationale as #3. |

**Fixes for #1/#2 route to P91** (`agent_flow_real.rs` family deepening, per
ROADMAP P91 acceptance criteria) **/ P92** (push-flow correctness, where a real
sim push-round-trip test would replace #1's config-only assertion).
#3/#4 do not need fixing — the marker convention (below) already documents
why they're exempt from further action.

### THIN-but-exempt `#[ignore]` cases (adjacent, not baseline, candidates for P91 deepening)

| file:line | test fn | why thin |
|---|---|---|
| `crates/reposix-cli/tests/agent_flow_real.rs:128,146,170` | `dark_factory_real_{github,confluence,jira}` | body = `run_init_and_assert`: `reposix init <backend>::…` then asserts `git config remote.origin.url` prefix/suffix strings only; module doc (`:29-36`) says live fetch is deferred, helper hardcodes `SimBackend`. Real creds gate a config-string assertion, not a real request. |
| `crates/reposix-cli/tests/attach.rs:548` | `attach_marks_mirror_lag_for_next_fetch` | real attach subprocess vs sim, real cache asserts; "for next fetch" is state-setup only — no fetch runs in the test. `#[ignore]` reason is sim-child, not real-backend. BORDERLINE. |

### Gate status (90-03 delivered, this section documents it)

`quality/gates/agent-ux/test-name-vs-asserts.sh` (new, pre-push cadence) walks
`crates/**/*.rs` for the F-K8 name-token regex, requires a genuine
`#[test]`/`#[tokio::test]` attribute nearby, auto-exempts
`#[ignore = "real-backend..."]`-gated tests structurally, and RAISEs any match
that is neither the 4-entry baseline above nor carries a
`// test-name-honesty: ok — <reason>` marker. 56 tests received the honest
marker (comment-only edits) in 90-03; the gate currently PASSes because the
live RAISE set exactly equals this section's 4-entry baseline.

