← [back to index](./index.md) · phase 80 plan 01

# Trust Boundaries & Threat Model

## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| helper → cache.repo | The helper writes refs into the cache's bare repo via `gix::Repository::edit_reference` (and `Repository::tag` if available). Trust direction: helper-controlled byte sources (`state.backend_name` enum slug + chrono::Utc::now() RFC3339 string) flow into ref names + tag-message bodies. No untrusted input from SoT propagates to ref content. |
| cache.repo → working tree (via helper advertisement) | Vanilla `git fetch` from the working tree pulls the refs across via the helper's `stateless-connect` advertisement. Trust direction: cache-side-written bytes (RFC3339 strings written by THIS helper) flow through. The bytes were never SoT-influenced. |
| reject-message stderr → user terminal | Reject stderr cites the synced-at timestamp from the tag-message body. Bytes are RFC3339 formatted by chrono — well-formed text. No raw SoT bytes flow here. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-80-01 | Tampering | Ref name composition (`format_mirror_head_ref_name`, `format_mirror_synced_at_ref_name`) | mitigate | `gix::refs::FullName::try_from` validates the formatted name; rejects `..`, `:`, control bytes per `gix_validate::reference::name`. The `sot_host` slug is `state.backend_name` — a controlled enum (`"sim" \| "github" \| "confluence" \| "jira"`), NEVER free-form user input. Unit test `mirror_ref_names_validate_via_gix` (T02) pins the validation contract to gix's enforcement. |
| T-80-02 | Information Disclosure (misleading) | Reject-message stderr citing synced-at timestamp | mitigate | Q2.2 doc-clarity contract carrier: the staleness window the refs measure IS the gap between SoT-edit and webhook-fire — NOT a "current SoT state" marker. T04 carries this prose into CLAUDE.md (one paragraph in § Architecture / Threat model). Full docs treatment defers to P85's `dvcs-topology.md`. The reject hint phrasing — "your origin (GH mirror) was last synced from <sot> at <ts> (<N> minutes ago)" — explicitly names "last synced" (not "current state"), reducing misread risk. |
| T-80-03 | Denial of Service (disk) | Reflog growth on long-lived caches (every push appends two reflog entries) | accept | Filed as a v0.14.0 operational concern. T02 adds a one-line note in `mirror_refs.rs`'s module-doc citing the deferral target. P80 ships the refs without periodic pruning; v0.14.0's OTel work + operational maturity scope handles reflog hygiene. |

No new HTTP origin in scope; no new Tainted<T> propagation path; no new
shell-out from the cache or helper; no new sanitization branch.
