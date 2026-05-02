# Validating fidelity — contract harness vs. real GitHub

← [back to index](./index.md)

**The point:** if our simulator drifts from the real API, the swarm catches phantom bugs. We need a small, cheap, network-tolerant harness that asserts our simulator matches GitHub on the **shape** of the GET-issue path. We cannot match full semantics (we deliberately differ on workflow), but the data shapes for read endpoints should round-trip.

### 8.1 Strategy

For each test:
1. Pick a public, stable issue on a well-known repo. Suggestion: `octocat/Hello-World` issue #1, or one of the never-closed `rust-lang/rust` historical issues. They've existed for years and won't disappear before the demo.
2. `GET https://api.github.com/repos/octocat/Hello-World/issues/1` (no auth — public, stays under unauthenticated 60/hr limit).
3. `GET http://127.0.0.1:7878/projects/octocat-hello-world/issues/1` against a sim seeded with one issue copied from the GitHub fixture.
4. Assert: same set of top-level keys, same value types, same enum domains for `state`. Allow our extra keys (`etag`, `version`, `project`) and missing GitHub-only keys (`node_id`, `repository_url`, etc.) — we don't pretend to be wire-compatible, only schema-shaped.

### 8.2 Test code

```rust
// tests/contract.rs
use serde_json::Value;

const GITHUB_KEYS: &[&str] = &[
    "id", "number", "title", "body", "state", "labels",
    "assignees", "user", "created_at", "updated_at",
];
const SIM_RENAMES: &[(&str, &str)] = &[
    ("user", "author"),  // we call it author
];

#[tokio::test]
async fn github_issue_shape_matches_simulator() {
    // 1. fetch from real GitHub (skip if offline)
    let gh: Value = match reqwest::get(
        "https://api.github.com/repos/octocat/Hello-World/issues/1"
    ).await {
        Ok(r) if r.status().is_success() => r.json().await.unwrap(),
        _ => { eprintln!("skipping: GitHub unreachable"); return; }
    };

    // 2. spin up sim with that issue seeded
    let state = test_state_with_seed_from(&gh).await;
    let app = reposix_sim::build_router(state);
    let server = axum_test::TestServer::new(app).unwrap();

    let sim: Value = server
        .get("/projects/octocat-hello-world/issues/1")
        .add_header("authorization", "Bearer test-admin-token")
        .await
        .json();

    // 3. shape assertions
    for key in GITHUB_KEYS {
        let sim_key = SIM_RENAMES.iter()
            .find(|(g,_)| g == key).map(|(_,s)| *s).unwrap_or(key);
        let gh_val = &gh[key];
        let sim_val = &sim[sim_key];
        assert!(
            same_kind(gh_val, sim_val),
            "key {key}: github={gh_val:?} sim={sim_val:?}"
        );
    }
    // state enum
    assert!(matches!(
        sim["state"].as_str().unwrap(),
        "open" | "closed" | "in_progress" | "in_review" | "done"
    ));
    assert!(matches!(gh["state"].as_str().unwrap(), "open" | "closed"));
}

fn same_kind(a: &Value, b: &Value) -> bool {
    use Value::*;
    matches!((a, b),
        (Null, Null) | (Bool(_), Bool(_)) | (Number(_), Number(_)) |
        (String(_), String(_)) | (Array(_), Array(_)) | (Object(_), Object(_))
    )
}
```

### 8.3 Property-test fold-in

For workflow safety:

```rust
// tests/workflow.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn no_random_transition_sequence_reaches_done_without_in_review(
        seq in proptest::collection::vec(
            "(start|review|complete|close|reopen|drop)", 0..40
        )
    ) {
        let mut state = "open".to_string();
        let mut visited_in_review = false;
        for tid in &seq {
            if let Ok(t) = WORKFLOW.validate(&state, tid) {
                state = t.to.clone();
                if state == "in_review" { visited_in_review = true; }
            }
        }
        if state == "done" {
            prop_assert!(visited_in_review,
                "reached done without going through in_review: {seq:?}");
        }
    }
}
```

This is the kind of test that earns its keep — it asserts an invariant that humans will *believe* but might not test, and it generates inputs faster than humans can think them up. Cite Luke Palmieri's testing material at `lpalmieri.com` for the broader pattern of using property tests to lock down API invariants.

### 8.4 What this harness explicitly does NOT cover

- Authentication (we don't have GitHub credentials in autonomous mode).
- Pagination (next-link headers are GitHub-specific; we keep it simple).
- Comments, reactions, sub-issues — out of scope for v0.1.
- Markdown rendering — `body` is opaque text in both APIs.

If the implementer has spare cycles, the next contract test to add is "POST → GET round-trip preserves field types" against the simulator alone, which doesn't need network and runs in CI.
