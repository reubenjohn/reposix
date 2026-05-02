← [back to index](./index.md) · phase 84 plan 01

## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `repository_dispatch` event payload → workflow runtime (NEW) | The `client_payload` JSON object is set by the dispatching party (Confluence's webhook config, or anyone with a PAT to dispatch). Trust direction: untrusted external bytes flow into the workflow. Mitigation: workflow IGNORES the payload — derives ALL values from `${{ secrets.* }}` and `${{ vars.* }}` (repo-owner-controlled). NO `${{ github.event.client_payload.* }}` references in any `run:` block. |
| Confluence webhook → GitHub `dispatches` API (NEW, owner-config) | The webhook is configured on the Atlassian side with a PAT in the `Authorization: token <PAT>` header. The PAT must have `repo` scope (RESEARCH.md Pitfall 7). Trust direction: owner-controlled credential; documented in P85's setup guide. NOT a P84-implementation issue (the workflow itself does not deal with the PAT). |
| Workflow → Atlassian REST (`reposix init confluence::TokenWorld /tmp/sot`) | UNCHANGED from v0.9.0 onward. Same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist. The workflow sets `REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://${tenant}.atlassian.net'` — confluence-only. |
| Workflow → mirror repo (`git push --force-with-lease=...`) | UNCHANGED git semantics. `permissions: contents: write` on the mirror repo only; the workflow has no token to push elsewhere. |
| Atlassian REST response → cache prior-blob parse | UNCHANGED — `Tainted<T>` propagation as in v0.9.0+. The workflow's cache build uses the existing parser; no new sanitization branch in P84. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-84-01 | Tampering | `repository_dispatch` `client_payload` injection — payload-into-`run:`-block → arbitrary code execution on runner | mitigate | Workflow IGNORES `client_payload`. The verbatim YAML in T02's `<must_haves>` derives ALL values from `${{ secrets.* }}` and `${{ vars.* }}`. The verifier `webhook-trigger-dispatch.sh` greps the YAML for `github.event.client_payload` and asserts ZERO matches — defense-in-depth against future regressions. Code review checkpoint: T02's diff is grepped for `client_payload` BEFORE the mirror-repo push lands. |
| T-84-02 | Information Disclosure | Secrets in workflow logs (`set -x` in `run:` block leaks `${{ secrets.* }}`) | mitigate | GH Actions auto-redacts `${{ secrets.* }}` when the literal byte sequence appears in step output. The workflow YAML avoids `set -x` in any `run:` block (only `set -euo pipefail`). The verifier `webhook-trigger-dispatch.sh` greps for `set -x` and asserts ZERO matches. |
| T-84-03 | Denial of Service | Workflow-trigger amplification — cron + dispatch fire 2× per 30min near a boundary | mitigate | `concurrency: { group: reposix-mirror-sync, cancel-in-progress: false }` (D-01) queues the second run; the second run sees `main` already in sync via `--force-with-lease` and exits cleanly (the lease check fails → exit 1). Wasted runner time bounded by the 10-min job timeout; for a typical 5-min sync, near-zero risk. |
| T-84-04 | Elevation of Privilege | Non-owner pushes to mirror via dispatching the workflow | mitigate | `repository_dispatch` requires a PAT on the dispatching side. The mirror repo's settings restrict who can configure secrets / dispatch. Documented in P85: mirror repo permissions should be `Maintain` for owner only; PAT for cross-repo dispatch is owner-controlled. NOT a P84-implementation issue (the workflow validates nothing on the dispatch side; it just runs). |
| T-84-05 | Tampering | `mirror_url` derived from `${{ github.server_url }}/${{ github.repository }}.git` could be tampered with via repository-rename or fork-spoofing | accept | The values come from GH-controlled context — `github.server_url` is `https://github.com` for github.com (or the GH Enterprise URL for self-hosted), and `github.repository` is the canonical repo path of the workflow's HOST repo. An attacker would need write access to the mirror repo to rename it; if they have that, the mirror is already compromised. NOT a new threat surface. |
| T-84-06 | Information Disclosure | Atlassian credentials leaked via the cache built into `/tmp/sot` AND uploaded as a build artifact | accept | The workflow does NOT call `actions/upload-artifact` for `/tmp/sot`. The runner is destroyed post-job. The cache is ephemeral. NOT a new threat surface; included for completeness because the cache contains tainted bytes. |

No new HTTP origin in scope (the `reposix init` REST call goes
through the existing `BackendConnector` allowlist). NEW
`Tainted<T>` propagation paths are NOT introduced in P84 (the
existing cache-build path is reused). New shell-out boundaries are
NOT introduced in P84 (the workflow's `git` invocations are inside
a GH Actions runner; the args come from `${{ github.* }}` /
`${{ secrets.* }}` / `${{ vars.* }}` — all owner-controlled).

The single new threat surface is the `repository_dispatch` event
payload, mitigated by S2 of the OVERVIEW (workflow ignores
`client_payload`; verifier grep check).
</threat_model>

---

