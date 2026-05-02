[index](./index.md)

# Task 01-T07 — Workspace-wide gate: clippy + tests

<read_first>
- Every file edited in this plan.
</read_first>

<action>
Run from repo root:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If any new lint fires, fix it inline (do not blanket-allow). Typical: `cast_possible_wrap`, `module_name_repetitions`, `too_many_lines` — allow with a `#[allow(clippy::X)]` ONLY if the full `-- -D warnings` run shows it's structural and unavoidable after minor refactor.
</action>

<acceptance_criteria>
- All three commands exit 0.
- Test count increase: at least +6 tests over the Phase 32 baseline (1 default-impl + 3 sim route + 2 SimBackend override + 1 each for github/confluence/jira = 7 minimum).
</acceptance_criteria>

<threat_model>
N/A (verification task). The tests themselves provide the regression proof for the egress + injection-defense claims made in earlier tasks.
</threat_model>
