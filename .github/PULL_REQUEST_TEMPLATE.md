## Summary

(1-2 lines, why-focused)

## What changed

- 

## Phase reference

(e.g. "Closes Phase 47" / "Quick — no phase / "Hot fix")

## Testing

- [ ] `cargo test --workspace` green
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` green
- [ ] `cargo fmt --all` clean
- [ ] `bash scripts/banned-words-lint.sh` clean (if doc changes)
- [ ] Manual run / screenshot (UI):

## Threat-model impact

| Path | Touched? | Mitigation |
|---|---|---|
| New egress endpoint |  |  |
| New tainted-byte path |  |  |
| New audit log op |  |  |
| Frontmatter schema |  |  |

## Checklist

- [ ] CHANGELOG entry (if user-facing)
- [ ] Docs updated (`docs/`, README, CLAUDE.md)
- [ ] Atomic conventional commits
