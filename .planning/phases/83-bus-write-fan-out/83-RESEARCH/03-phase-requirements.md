← [back to index](./index.md) · phase 83 research

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DVCS-BUS-WRITE-01 | SoT-first write — buffer fast-import; apply REST; on success, audit BOTH tables + update `last_fetched_at`; on failure, bail (mirror unchanged). | § Pattern 1 (`apply_writes` refactor) lifts existing `handle_export` loop verbatim; only the surface is reshaped. |
| DVCS-BUS-WRITE-02 | Mirror write — `git push` to mirror after SoT success; on mirror-fail, write mirror-lag audit row, update `head` ref but NOT `synced-at`, stderr warn, return ok to git. | § Pattern 2 (`push_mirror` subprocess) + § "Mirror-write algorithm" exact state transitions. |
| DVCS-BUS-WRITE-03 | On mirror-write success: update `synced-at` ref to now; send `ok refs/heads/main` to git. | § Pattern 2; ref-write helpers `Cache::write_mirror_synced_at` already shipped P80. |
| DVCS-BUS-WRITE-04 | No helper-side retry on transient mirror failure (Q3.6). | § Pitfall 4; `push_mirror` returns `Err` on first non-zero exit. |
| DVCS-BUS-WRITE-05 | Bus URL with no local `git remote` for the mirror fails with P82's verbatim hint (no auto-mutation). | Already shipped P82 (`bus_handler::resolve_mirror_remote_name`); P83 adds a regression integration test. |
| DVCS-BUS-WRITE-06 | Fault-injection tests cover (a) mirror-push fail, (b) SoT-write mid-stream fail, (c) post-precheck SoT 409. Each → correct audit + recoverable state. | § "Fault-injection test infrastructure" section 4. |

