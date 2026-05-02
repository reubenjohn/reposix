[← index](./index.md)

# Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P82) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints SIX rows + SIX verifier shells
   BEFORE T02–T06 implementation. Initial status `FAIL`. Rows are
   hand-edited per documented gap (NOT Principle A) — annotated in
   commit message referencing GOOD-TO-HAVES-01. The agent-ux
   dimension's bind verb is not yet implemented; rows ship as
   hand-edits matching P81's precedent.
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-remote`,
   `cargo nextest run -p reposix-remote`. Pre-push hook runs the
   workspace-wide gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 → T05 → T06
   — never parallel, even though T02 (`bus_url.rs`) and T03 (precheck
   wrapper) touch different files. CLAUDE.md "Build memory budget"
   rule is "one cargo invocation at a time" — sequencing the tasks
   naturally honors this.
4. **`bus_url.rs` is a SIBLING module of `parse_remote_url`.** The
   single-backend `ParsedRemote` shape stays unchanged (per RESEARCH.md
   Pattern 1). New `Route { Single(ParsedRemote) | Bus(...) }` enum
   branches at `argv[2]` parse time INSIDE `bus_url::parse`. The
   `backend_dispatch::parse_remote_url` function is called by
   `bus_url::parse` AFTER stripping the optional query string —
   the existing parser sees only single-backend-shaped URLs.
5. **PRECHECK B uses a COARSER wrapper.** New `precheck_sot_drift_any(cache,
   backend, project, rt) -> SotDriftOutcome` returning `Drifted | Stable`.
   P81's `precheck_export_against_changed_set` is preserved verbatim
   for P83's write-time intersect-with-push-set check. Both functions
   live in `precheck.rs`; the coarser one is ~10 lines (D-01 in
   RESEARCH.md § Pattern 2).
6. **Capabilities branching is a 5-line edit at `main.rs:150-172`** (S1
   above). `if state.mirror_url.is_none() { proto.send_line("stateless-connect")?; }`
   gates DVCS-BUS-FETCH-01. The other four capability lines stay
   unchanged.
7. **PRECHECK A shells out to `git ls-remote`.** D-06 RATIFIED. Use
   `--` separator to defang argument-injection via mirror_url. Reject
   mirror URLs starting with `-` BEFORE the shell-out.
8. **No-remote lookup: by-URL-match (D-01 / Q-A).** Match against all
   configured remotes' URLs (`git config --get-regexp '^remote\..+\.url$'`),
   with multi-match alphabetical-first + WARN. Zero-match → emit
   verbatim Q3.5 hint and exit before PRECHECK A.
9. **Bus path emits clean "P83 not yet shipped" error after prechecks
   pass (D-02 / Q-B).** P82 is dispatch-only; no write fan-out. The
   error: stderr "bus write fan-out (DVCS-BUS-WRITE-01..06) is not
   yet shipped — lands in P83"; stdout `error refs/heads/main bus-write-not-yet-shipped`.
10. **Reject unknown query params (D-03 / Q-C).** Only `mirror=` is
    recognized. Typo-protection + forward compat. `bus_url::parse`
    rejects with: *"unknown query parameter `<key>` in bus URL; only
    `mirror=` is supported"*.
11. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
    codified 2026-04-30).** T06 ends with `git push origin main`;
    pre-push gate must pass; verifier subagent grades the six catalog
    rows AFTER push lands. Verifier dispatch is an orchestrator-level
    action AFTER this plan completes — NOT a plan task.
12. **CLAUDE.md update in same PR (QG-07).** T06 documents the bus URL
    scheme `reposix::<sot>?mirror=<mirror>` (§ Architecture) + the new
    push form `git push reposix main` (§ Commands). The mirror-URL
    percent-encoding requirement (RESEARCH.md Pitfall 7) is named.
13. **No new error variants.** The remote crate uses `anyhow` throughout
    (per `crates/reposix-remote/src/main.rs:18`). Bus_url + bus_handler
    + precheck wrapper all return `anyhow::Result<...>`. Reject-path
    stderr strings are passed via `.context("bus-precheck: ...")`
    annotations and emitted by the existing `fail_push(diag, ...)` shape.
14. **State extension is one Option field (D-05 / S1).** `State` gains
    `mirror_url: Option<String>` field. `Some(url)` = bus invocation;
    `None` = single-backend. Capability branching + bus-handler dispatch
    both read this single field. NO new `BusState` type.

# Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces ONE new
trifecta surface (the `git ls-remote` shell-out's argument boundary)
and reuses three existing surfaces unchanged:

| Existing surface              | What P82 changes                                                                                                                                                                                                                                                                |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — PRECHECK B's `list_changed_since` call is the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist used since v0.9.0. No new HTTP construction site introduced. |
| Cache prior-blob parse (Tainted bytes) | UNCHANGED — P82 does not touch the cache prior parse. The bus path uses the coarser `precheck_sot_drift_any` (P82's NEW wrapper) which only reads the `last_fetched_at` cursor and counts changed records — does NOT parse blobs. |
| `Tainted<T>` propagation      | UNCHANGED — no new tainted-byte sources or sinks in P82. |
| **Shell-out boundary (NEW)**  | NEW: `git ls-remote -- <mirror_url> refs/heads/main`. The `mirror_url` is user-controlled (from the bus URL in argv[2]); the threat is argument-injection. Mitigation: (a) reject mirror URLs starting with `-` BEFORE the shell-out; (b) pass `--` separator unconditionally before the URL argument; (c) `mirror_url` is byte-passed (no template expansion). STRIDE category: Tampering — mitigated by D-06 + the rejection at parse time. |
| **`git config --get-regexp` shell-out (NEW)** | NEW: STEP 0 invokes `git config --get-regexp '^remote\..+\.url$'`. The regex is helper-controlled (no user input); the helper parses stdout via whitespace-split + byte-equal comparison. STRIDE category: Tampering — mitigated by helper-controlled regex (no string concatenation with user input) + read-only config invocation. |
| **`git rev-parse` shell-out (NEW)** | NEW: PRECHECK A reads the local mirror SHA via `git rev-parse refs/remotes/<name>/main`. The `<name>` is bounded by the result of STEP 0's `git config` lookup (so it matches `^remote\..+\.url$` — the regex itself is helper-controlled). STRIDE category: Tampering — mitigated by the bounded source of `<name>`. |

`<threat_model>` STRIDE register addendum below the per-task threat
register in the plan body:

- **T-82-01 (Tampering — argument injection via `mirror_url` shell-out):**
  reject `-`-prefix + `--` separator before the URL.
- **T-82-02 (Information Disclosure — Tainted SoT bytes leaking via
  bus_handler logs):** UNCHANGED from P81 — `precheck_sot_drift_any`
  only counts records, never logs body bytes; deferred-error stub
  emits no tainted bytes.
- **T-82-03 (Denial of Service — `git ls-remote` against private
  mirrors hangs on SSH-agent prompt):** documented in CLAUDE.md
  `git ls-remote` requires SSH agent set up; tests use `file://`
  fixture exclusively per RESEARCH.md Pitfall 3.
