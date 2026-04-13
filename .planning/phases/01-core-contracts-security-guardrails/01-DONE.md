# Phase 1 — DONE

Shipped (2026-04-13): `reposix_core` now publishes the locked-down HTTP client factory (5s timeout, no redirects, env-driven origin allowlist + per-request recheck, clippy.toml ban on direct reqwest construction), the `Tainted<T>`/`Untainted<T>` + `sanitize()` type discipline with trybuild compile-fail locks, the `validate_issue_filename` / `validate_path_component` validators (SG-04), and the append-only `audit_events` SQLite schema fixture with `BEFORE UPDATE` / `BEFORE DELETE` triggers proven by integration tests.

Commits:
- `fa95f03` feat(01-00): add InvalidOrigin, InvalidPath, Http error variants
- `56719af` feat(01-01): add ClientOpts, OriginGlob, allowlist parser, client() factory, request() gate
- `4552b1a` feat(01-01): add workspace clippy.toml disallowed-methods + load-proof script (FIX 3)
- `c0c3c55` test(01-01): http_allowlist integration tests (7 live + 1 ignored)
- `6c35efc` feat(01-02): add Tainted/Untainted newtypes + sanitize() + ServerMetadata
- `b5df598` feat(01-02): add validate_issue_filename + validate_path_component (SG-04)
- `1a696bf` test(01-02): trybuild compile-fail fixtures for SG-05 type discipline + FIX 4
- `a166c22` feat(01-03): add audit_events schema fixture + SCHEMA_SQL / load_schema (SG-06)
- `92d1675` test(01-03): show_audit_schema example + audit_schema integration tests
- `5f26860` chore(01): cargo fmt + Cargo.lock for CI (rustfmt check + --locked)

Phase-exit verification: all named tests (`egress_to_non_allowlisted_host_is_rejected`, `server_controlled_frontmatter_fields_are_stripped`, `filename_is_id_derived_not_title_derived`, `path_with_dotdot_or_nul_is_rejected`, `tainted_cannot_be_used_where_untainted_required`, `untainted_new_is_pub_crate_only`, `redirect_target_is_rechecked_against_allowlist`, `allowlist_default_and_env`), `audit_schema` integration tests, `show_audit_schema` example emits both triggers on one line, zero direct reqwest constructors outside `http.rs`, `clippy.toml` loaded (`scripts/check_clippy_lint_loaded.sh` exits 0), `cargo clippy -p reposix-core --all-targets -- -D warnings` clean.

Workspace test totals after phase 1: 36 unit + 5 audit_schema + 2 compile_fail + 7 http_allowlist (1 ignored timeout test, run via `--ignored`) = 50 passing tests.
