← [back to index](./index.md)

# Task 02-T06 — Workspace gate

<read_first>
- All files edited across both Plans 01 and 02.
</read_first>

<action>
Run:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Fix any residual lint or test failure. Do not allow-list lints without a rationale comment at the allow site.
</action>

<acceptance_criteria>
- All three commands exit 0.
- Workspace test count increases by the combined additions from Plan 01 and Plan 02 (roughly +10 net tests).
- No `todo!` / `unimplemented!` / `FIXME(33)` remain in the shipped files (`grep -rn 'todo!\\|unimplemented!' crates/reposix-cache/src/ crates/reposix-remote/src/` has no new matches beyond pre-existing unrelated ones).
</acceptance_criteria>

<threat_model>
N/A (verification task). The preceding tasks' threat-model discipline is regression-guarded by the test suite.
</threat_model>
