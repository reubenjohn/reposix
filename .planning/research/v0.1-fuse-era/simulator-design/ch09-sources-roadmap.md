# Sources and roadmap implications

← [back to index](./index.md)

## 9. Sources

Authoritative (HIGH confidence):
- axum docs and discussions: `docs.rs/axum/latest/axum/`, GitHub discussion #964 on shared SQLite state, discussion #1758 on multi-field state, `docs.rs/axum/latest/axum/extract/struct.State.html`.
- tower-governor: `docs.rs/tower_governor/latest/tower_governor/`, `github.com/benwis/tower-governor`, `lib.rs/crates/tower_governor`.
- wiremock-rs (design pattern reference, not a dependency): `github.com/LukeMathWalker/wiremock-rs`, `docs.rs/wiremock/`, `lpalmieri.com/posts/2020-04-13-wiremock-async-http-mocking-for-rust-applications/`.
- GitHub Issues REST API: `docs.github.com/en/rest/issues/issues` (API version `2026-03-10` per page header). Rate-limit and conditional-request semantics: `docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api`.
- SQLite WAL: `sqlite.org/wal.html`, `sqlite.org/walformat.html`.

MEDIUM confidence (pattern based on training data + cross-checked with public docs, not deeply re-fetched here):
- Jira workflow transitions endpoint shape (`/rest/api/3/issue/{key}/transitions`). Atlassian developer portal renders JS-heavy; the transitions list/apply shape is well-known and modeled conservatively. If the implementer wants belt-and-braces, fetch the page through playwright before hardening the sim's transitions schema.
- Stripe-style idempotency-key semantics (return cached on hit, 422 on key reuse with different body). This is a widely-replicated pattern; cite Stripe's own docs if/when adding to README.

LOW confidence flags:
- "5000/hr matches GitHub authenticated" — true today (2026-04), but real product limits drift. The simulator exposes the limit as a `--rate-limit` flag rather than baking it in.
- "tower-governor 0.6 line is the right pin" — train-data shows `tower_governor` versions in flux. The implementer should check `lib.rs/crates/tower_governor` at build time and pin to whatever current minor version supports the `KeyExtractor` trait shape used in §2.1. If the API has shifted, the migration is small and the conceptual design doesn't change.

---

## 10. Implications for the roadmap

If the orchestrator is using this report to shape phases:

1. **Phase: simulator core** — schema, AppState, /projects + /issues GET/POST/PATCH/DELETE, audit middleware, rate-limit middleware, auth middleware. ~half of the remaining build budget.
2. **Phase: workflow + RBAC + idempotency** — `/transitions`, `/permissions`, idempotency table, the conflict 409 path. The dark-factory-defining behaviors. Should land before the FUSE layer is wired so FUSE has something real to talk to.
3. **Phase: dashboard + chaos + seeder** — vibe-coded UI, chaos config endpoint, deterministic seed. Demo-facing polish.
4. **Phase: contract harness** — §8 tests. Should run in CI, including a `--ignored` real-network test that the demo runs once during walkthrough.

Phases 1 and 2 are the load-bearing ones. Phase 3 is what makes the demo *legible*; Phase 4 is what lets us claim "faithful enough."

The biggest risk to overnight delivery: getting fancy with §3.3's response-body capture or §4's SSE streaming. Both are clearly-marked "skip for v0.1" — if the implementer is tempted to do them, that's the time to stop and ship.
