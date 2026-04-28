# quality/gates/security/ -- security-dimension verifiers

**v0.12.0 status: stub home.** All security-dim catalog rows ship as v0.12.1 carry-forward stubs (NOT-VERIFIED + waiver until 2026-07-26 tracked_in v0.12.1 SEC-* placeholders). v0.12.1 MIGRATE-03 ships the verifier scripts + flips status to PASS.

## Verifiers (v0.12.1 PERF: stub paths only)

| Script (planned v0.12.1) | Catalog row | Cadence |
|---|---|---|
| `allowlist-enforcement.sh` | security/allowlist-enforcement | pre-pr |
| `audit-immutability.sh` | security/audit-immutability | pre-pr |

## Conventions

- Bash for thin test wrappers; Rust integration tests live next to the module under test (e.g., `crates/reposix-core/src/http.rs`).
- Exit 0 = PASS, 1 = FAIL.
- Artifacts at `quality/reports/verifications/security/<row-slug>.json`.

## Threat-model anchor

Two P0 cuts drive this dimension:

- **Outbound HTTP allowlist** -- `crates/reposix-core/src/http.rs` is the single factory; `REPOSIX_ALLOWED_ORIGINS` env var enforces the deny-by-default rule. CLAUDE.md threat-model section lifts the contract.
- **Audit log immutability** -- `audit_events_cache` (in `reposix-cache::audit`) + `audit_events` (in `reposix-core::audit`) are append-only by SQLite schema; v0.12.1 SEC-02 verifies that UPDATE/DELETE attempts fail.

## Cross-references

- `quality/catalogs/security-gates.json` -- 2-row catalog (P63 stubs).
- `quality/PROTOCOL.md` -- runtime contract.
- `CLAUDE.md` § "Threat model" -- the two-leg cut anchored here.
- `.planning/REQUIREMENTS.md` § MIGRATE-03 -- v0.12.1 carry-forward.

P63 cohesion pass: trimmed to verifier-table + conventions; runtime detail at `quality/PROTOCOL.md`.
