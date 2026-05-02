# Observability dashboard — vibe-coded, working

← [back to index](./index.md)

Following Simon Willison's "tiny Go binary with vibe-coded UI" pattern (`docs/research/agentic-engineering-reference.md` §1). Goals:

- Single embedded HTML file, no build step, no node_modules.
- Polls `/_audit?since={cursor}` every 2s.
- Shows: live request stream, conflict frequency, rate-limit hits, chaos events.
- Useful enough that a human glancing at it during the demo *immediately* sees the swarm working.

`src/ui/index.html` is roughly 200 lines: a single `<table>` for the request log, two Chart.js sparklines (loaded from CDN — fine for a localhost dev tool), and a controls bar to toggle chaos and reset the rate limiter. Embedded via `include_str!` and served at `/`. The widget shape:

```
+---------------------------------------------------------------+
| reposix-sim   [agents: 8]  [rps: 47]  [conflict-rate: 4.2%]   |
| chaos: latency [0..50ms] errors [0%/0%]   [reset]  [pause]    |
+---------------------------------------------------------------+
| sparkline: req/s last 5min     | sparkline: 409s last 5min    |
+---------------------------------------------------------------+
| time     agent          method path                  status   |
| 03:14:15 agent-alice    PATCH  /projects/reposix/... 409 (CONFLICT)
| 03:14:15 agent-bob      POST   /projects/reposix/... 201
| 03:14:14 agent-cara     GET    /projects/reposix/... 304
+---------------------------------------------------------------+
```

The dashboard is read-only over the network from outside the host — there is no auth on `/`, but the `/_audit` endpoint requires admin auth. That keeps a casual `curl 127.0.0.1:7878/` safe while gating the actual data. For a swarm demo we run the UI behind the same `--bind 127.0.0.1` as the API.

`tokio::sync::broadcast` channel is the optional upgrade for "live tail" — the audit middleware sends `AuditRow` over the channel; the dashboard subscribes via `GET /_audit/stream` (SSE). Skip for v0.1 unless polling proves laggy. Polling every 2s is fine for a dev tool.
