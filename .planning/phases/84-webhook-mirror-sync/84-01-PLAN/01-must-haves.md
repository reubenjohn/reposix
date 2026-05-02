← [back to index](./index.md) · phase 84 plan 01

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `repository_dispatch` event payload → workflow runtime (NEW) | The `client_payload` JSON object is set by the dispatching party (Confluence's webhook config, or anyone with a PAT to dispatch). Trust direction: untrusted external bytes flow into the workflow. Mitigation: workflow IGNORES the payload — derives ALL values from `${{ secrets.* }}` and `${{ vars.* }}` (repo-owner-controlled). NO `${{ github.event.client_payload.* }}` references in any `run:` block. |
| Confluence webhook → GitHub `dispatches` API (NEW, owner-config) | The webhook is configured on the Atlassian side with a PAT in the `Authorization: token <PAT>` header. The PAT must have `repo` scope (RESEARCH.md Pitfall 7). Trust direction: owner-controlled credential; documented in P85's setup guide. NOT a P84-implementation issue (the workflow itself does not deal with the PAT). |
| Workflow → Atlassian REST (`reposix init confluence::TokenWorld /tmp/sot`) | UNCHANGED from v0.9.0 onward. Same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist. The workflow sets `REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://${tenant}.atlassian.net'` — confluence-only. |
| Workflow → mirror repo (`git push --force-with-lease=...`) | UNCHANGED git semantics. `permissions: contents: write` on the mirror repo only; the workflow has no token to push elsewhere. |
| Atlassian REST response → cache prior-blob parse | UNCHANGED — `Tainted<T>` propagation as in v0.9.0+. The workflow's cache build uses the existing parser; no new sanitization branch in P84. |

