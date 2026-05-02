← [back to index](./index.md)

# T3 — Fill docs/reference/simulator.md — flag table + endpoints + seeding

<task type="auto">
  <name>Task 3: Fill docs/reference/simulator.md — flag table + endpoints + seeding</name>
  <files>docs/reference/simulator.md</files>
  <read_first>
    - `docs/reference/simulator.md` (current skeleton from plan 30-02)
    - `docs/reference/cli.md` (lines 47-58 — `reposix sim` flag table)
    - `docs/reference/http-api.md` (lines 7-59 — endpoint pairs)
    - `crates/reposix-sim/fixtures/seed.json` (seed fixture — describe shape, do NOT copy content)
    - `crates/reposix-sim/src/main.rs` (or wherever the CLI flags are defined — confirm current flags match)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/reference/simulator.md
  </read_first>
  <action>
    Replace the ENTIRE contents of `docs/reference/simulator.md` with:

```markdown
# The simulator

The simulator is the default testing backend for reposix. It is an in-process axum HTTP server over SQLite WAL that speaks the same REST shape as a real tracker. Every demo, tutorial, and unit test in this repo targets the simulator unless explicitly overridden — simulator-first by project policy.

## Starting the simulator

\`\`\`bash
target/release/reposix-sim \
    --bind 127.0.0.1:7878 \
    --db /tmp/my-sim.db \
    --seed-file crates/reposix-sim/fixtures/seed.json &

curl -sf http://127.0.0.1:7878/healthz   # waits for "ok"
\`\`\`

The simulator runs until killed. For a fresh state, delete the `--db` file and restart.

## CLI flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--bind` | `127.0.0.1:7878` | Listen address. The default is localhost only, as a basic guardrail. |
| `--db` | `runtime/sim.db` | SQLite WAL file. Persists between restarts. |
| `--seed-file` | — | Path to a JSON seed (e.g. `crates/reposix-sim/fixtures/seed.json`). See "Seeding + fixtures" below. |
| `--no-seed` | off | Do not seed even if `--seed-file` is given. |
| `--ephemeral` | off | Use in-memory SQLite instead of `--db` (fastest, no persistence). |
| `--rate-limit` | `100` | Per-agent requests/sec before `429 Too Many Requests`. |

## HTTP endpoints

All paths are under `/projects/<project-id>` unless noted. The shape matches the simulator-facing subset of real tracker APIs.

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/healthz` | Readiness probe. Returns `ok`. |
| `GET` | `/projects/<p>/issues` | List issues; returns JSON array. |
| `GET` | `/projects/<p>/issues/:id` | Read one issue. Returns JSON with `version`, `status`, `assignee`, body, and custom fields. |
| `POST` | `/projects/<p>/issues` | Create issue. Accepts JSON body; returns the created issue with server-assigned `id` + `version=1`. |
| `PATCH` | `/projects/<p>/issues/:id` | Update issue. Optimistic concurrency: pass `If-Match: <version>` header — a mismatched version returns `409 Conflict`. |
| `DELETE` | `/projects/<p>/issues/:id` | Delete issue. Irreversible. Bulk-delete cap (SG-02) is enforced at the git-remote-helper layer, not here. |
| `POST` | `/projects/<p>/issues/:id/transitions` | Jira-shaped status transition endpoint. Requires transition-id in body. |
| `GET` | `/audit` | List recent audit rows (sim-only debugging). |

Full request/response JSON shapes: [HTTP API reference](http-api.md).

## Seeding + fixtures

The simulator seeds itself from a JSON fixture specified by `--seed-file`. The canonical fixture lives at `crates/reposix-sim/fixtures/seed.json` and ships 6 issues covering:

- Various `status` values (`open`, `in_progress`, `done`)
- A few custom_fields
- Comments in the markdown body
- An assignee

To author your own fixture, shape:

\`\`\`json
{
  "project": { "id": "demo" },
  "issues": [
    {
      "id": 1,
      "title": "Fix flaky test",
      "status": "open",
      "assignee": "alice@example.com",
      "labels": ["backend", "flaky"],
      "body": "## Description\n\nThe test sometimes fails on CI."
    }
  ]
}
\`\`\`

Then run with `--seed-file path/to/your.json`. The file is read once at startup; later edits do not hot-reload.

## Rate limiting + 409 behavior

The simulator models tracker realism beyond the happy path:

- **Rate limits** — exceeding `--rate-limit` returns `429 Too Many Requests` with a `Retry-After` header.
- **409 conflicts** — `PATCH` without a matching `If-Match` header returns `409`. The git-remote helper retries via `git merge` semantics; see [The git layer](../how-it-works/git.md).

## Audit log

Every mutation is logged to an append-only `audit` SQLite table. Query it directly:

\`\`\`bash
sqlite3 /tmp/my-sim.db \
    "SELECT ts, method, path, status FROM audit ORDER BY ts DESC LIMIT 10"
\`\`\`

See [The trust model](../how-it-works/trust-model.md) for the guarantees (SG-06: no UPDATE or DELETE on the audit table).
```

Verify the flag table matches the current CLI by reading `crates/reposix-sim/src/main.rs` (or wherever flags are defined). If a flag has been added/removed since Phase 29 shipped, update the table here.

Run `mkdocs build --strict` — simulator.md is already in the nav under `Reference` per plan 30-04.
  </action>
  <verify>
    <automated>test -f docs/reference/simulator.md && grep -c '^## ' docs/reference/simulator.md | awk '{exit !($1 >= 5)}' && grep -c 'Rate limiting' docs/reference/simulator.md | grep -q '^1$' && grep -c 'crates/reposix-sim/fixtures/seed.json' docs/reference/simulator.md | awk '{exit !($1 >= 1)}' && grep -c '^| `--' docs/reference/simulator.md | awk '{exit !($1 >= 5)}' && mkdocs build --strict</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c '^## ' docs/reference/simulator.md` returns `>= 5` (Starting / CLI flags / HTTP endpoints / Seeding / Rate limiting / Audit log).
    - `grep -c '^| `--' docs/reference/simulator.md` returns `>= 5` (CLI flag rows in the markdown table).
    - `grep -c '^| `GET`\\|^| `POST`\\|^| `PATCH`\\|^| `DELETE`' docs/reference/simulator.md` returns `>= 5` (endpoint rows).
    - `grep -c 'crates/reposix-sim/fixtures/seed.json' docs/reference/simulator.md` returns `>= 1`.
    - `grep -c 'http-api.md' docs/reference/simulator.md` returns `>= 1` (cross-link to endpoint reference).
    - `grep -c 'trust-model.md\|git.md' docs/reference/simulator.md` returns `>= 2` (cross-links to how-it-works).
    - `wc -l docs/reference/simulator.md` reports `>= 60`.
    - `mkdocs build --strict` exits 0.
    - Reference pages are exempt from ProgressiveDisclosure per `.vale.ini`; Vale passes.
  </acceptance_criteria>
  <done>
    simulator.md filled as dev-tooling Reference. `mkdocs build --strict` green. DOCS-05 complete.
  </done>
</task>
