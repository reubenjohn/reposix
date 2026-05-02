← [back to index](./index.md) · phase 82 plan 01

# Objective & Architecture

<objective>
Land the read/dispatch surface of the bus remote for v0.13.0's DVCS
topology — URL parser recognizes `reposix::<sot-spec>?mirror=<mirror-url>`
per Q3.3; bus PUSH-only per Q3.4 (no `stateless-connect` advertisement);
two cheap prechecks (mirror drift via `git ls-remote`; SoT drift via P81's
`list_changed_since` substrate) bail BEFORE reading stdin. The WRITE
fan-out (steps 4–9 of the bus algorithm) is explicitly DEFERRED to P83 —
P82 ends in a clean "P83 not yet shipped" error after prechecks pass
(Q-B in this plan).

This is a **single plan, six sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 6 rows in `quality/catalogs/agent-ux.json` +
  6 TINY verifier shells (status FAIL).
- **T02** — `bus_url.rs` parser module (new file) + 4 unit tests inline.
- **T03** — Coarser SoT-drift wrapper `precheck_sot_drift_any` appended
  to `precheck.rs` + 1 unit test.
- **T04** — `bus_handler.rs` module (new file) + `main.rs` Route
  dispatch + capabilities branching + State extension.
- **T05** — 4 integration tests under `crates/reposix-remote/tests/`
  (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b).
- **T06** — Catalog flip FAIL → PASS + CLAUDE.md update + per-phase
  push.

Sequential (T01 → T02 → T03 → T04 → T05 → T06). Per CLAUDE.md "Build
memory budget" the executor holds the cargo lock sequentially across
T03 → T05. T01 is doc-only (catalog rows + verifier shell scaffolding).

**Architecture (read BEFORE diving into tasks):**

The bus URL parser lives in a NEW SIBLING module `bus_url.rs` that
wraps the existing `backend_dispatch::parse_remote_url` (RESEARCH.md
Pattern 1). The single-backend `ParsedRemote` shape stays unchanged
— the new `Route { Single(ParsedRemote) | Bus { sot: ParsedRemote,
mirror_url: String } }` enum branches at `argv[2]` parse time INSIDE
`bus_url::parse`. Strip `?<query>` segment BEFORE delegating to
`backend_dispatch::parse_remote_url(base)` (RESEARCH.md Pitfall 2 —
the existing splitter rejects `?` in the project segment).

`State` (`crates/reposix-remote/src/main.rs:48`) gains ONE new field:
`mirror_url: Option<String>`. `Some(url)` = bus invocation; `None` =
single-backend (D-05). Capability branching reads
`state.mirror_url.is_none()` to gate `stateless-connect`. The
bus_handler dispatch reads `state.mirror_url.as_deref()` to extract
the URL for STEP 0 + PRECHECK A. NO new `BusState` type-state
explosion — single Option field, minimal blast radius.

`bus_handler::handle_bus_export` is a SIBLING of the existing
`handle_export` (NOT a wrapper). Order: capabilities → list →
`export` verb received → STEP 0 (resolve local mirror remote name
by URL match per Q-A) → PRECHECK A (mirror drift) → PRECHECK B
(SoT drift via `precheck::precheck_sot_drift_any`) → emit deferred-
shipped error per Q-B (P82 stops here; P83 takes over). NO stdin
read. NO `parse_export_stream` invocation.

The coarser SoT-drift wrapper `precheck::precheck_sot_drift_any`
(NEW in T03) is a 10-line sibling of P81's
`precheck_export_against_changed_set`. It returns
`SotDriftOutcome { Drifted { changed_count: usize } | Stable }`,
reusing `cache.read_last_fetched_at()` (P81) and
`backend.list_changed_since(project, since)`. First-push (no cursor)
returns `Stable` — same policy as P81's wrapper.

PRECHECK A invokes `Command::new("git").args(["ls-remote", "--",
mirror_url, "refs/heads/main"])` — the project's existing shell-out
idiom (D-06; `crates/reposix-cli/src/doctor.rs:446-944`). The `--`
separator + reject-`-`-prefix mitigations defang argument-injection
via mirror_url (T-82-01). Local SHA read via
`git rev-parse refs/remotes/<name>/main` (handles packed-refs
correctly).

STEP 0's name lookup: `git config --get-regexp '^remote\..+\.url$'`,
parse stdout into `(config_key, value)` pairs, filter where value
byte-equals `mirror_url` (with trailing-slash normalization), sort
matched names alphabetically, pick first + WARN if multiple per
Pitfall 4. Zero matches → emit Q3.5 hint and exit BEFORE PRECHECK A
(D-01 / Q-A).

P82 emits the deferred-shipped error after prechecks pass:
- stderr: `"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83"`
- stdout: `error refs/heads/main bus-write-not-yet-shipped`
- `state.push_failed = true`; return `Ok(())`.

The unknown-query-key rejection (D-03 / Q-C) lives inside
`bus_url::parse`: after `parse_query`, iterate keys; if any key !=
`"mirror"`, return `Err(...)` with verbatim message *"unknown query
parameter `<key>` in bus URL; only `mirror=` is supported"*. The
`+`-delimited rejection lives BEFORE the query-string split — if
`stripped.contains('+')` and the URL has no `?`, return error citing
the canonical form.

**Best-effort vs hard-error semantics:**

- **STEP 0 zero-match:** hard error (refuses to push; emits Q3.5 hint).
- **STEP 0 multi-match:** WARN to stderr, proceed with first
  alphabetical (D-01).
- **PRECHECK A `git ls-remote` failure:** hard error → reject path
  (`fail_push(diag, "ls-remote-failed", ...)`); the user sees the
  failure reason, not a silent succeed.
- **PRECHECK A drift:** reject path (`fail_push(diag, "fetch first",
  "your GH mirror has new commits...")`).
- **PRECHECK A empty mirror:** treat as `Stable` (no drift possible);
  P84 webhook sync handles first-push-to-empty-mirror via separate
  code path.
- **PRECHECK B failure (REST unreachable):** hard error → reject path
  (`fail_push(diag, "backend-unreachable", ...)`).
- **PRECHECK B `Drifted`:** reject path (`fail_push(diag, "fetch first",
  "<sot> has changes since your last fetch...")`); cite mirror-lag
  refs hint when `read_mirror_synced_at` is populated (P80).
- **PRECHECK B `Stable`:** proceed to deferred-shipped error stub
  (Q-B).

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo check -p reposix-remote`,
`cargo nextest run -p reposix-remote`) used instead of workspace-wide.

This plan terminates with `git push origin main` (per CLAUDE.md push
cadence) with pre-push GREEN. The catalog rows' initial FAIL status
is acceptable through T01–T05 because the rows are `pre-pr` cadence
(NOT `pre-push`); the runner re-grades to PASS during T06 BEFORE the
push commits.

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| helper → `git ls-remote` shell-out (NEW) | `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"])`. The `mirror_url` is user-controlled (from argv[2]'s bus URL). Trust direction: argv[2]-derived bytes flow as command argument. Mitigations: (a) reject `mirror_url` starting with `-` (T-82-01), (b) `--` separator unconditionally before the URL, (c) byte-pass (no template expansion / shell interpretation). |
| helper → `git config --get-regexp` shell-out (NEW) | The regex `^remote\..+\.url$` is HELPER-controlled (no user input flows to the regex). The output (config keys + values) is parsed via whitespace-split; values are byte-equal-compared to `mirror_url`. Trust direction: helper-controlled command, user-controlled output. Mitigation: regex is helper-controlled; no string concatenation with user input. |
| helper → `git rev-parse refs/remotes/<name>/main` shell-out (NEW) | `<name>` comes from STEP 0's name resolution, which is bounded by config keys matching `^remote\.([^.]+)\.url$`. The middle group is the remote name — git's own validation prevents weird characters. Trust direction: helper-controlled call site, value bounded by git's own remote-name rules. Mitigation: `<name>` extracted from config key (not from user URL), so it's already validated by git when the user ran `git remote add`. |
| helper → SoT (`list_changed_since` REST call, PRECHECK B) | UNCHANGED from P81 — same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist. The `since` parameter is helper-generated (`Cache::read_last_fetched_at()` written by THIS helper on prior push). |
| SoT bytes → bus handler (PRECHECK B response) | UNCHANGED — `precheck_sot_drift_any` only counts records (`changed.len()`), never parses blobs. NO Tainted byte propagation in P82. |
| Bus URL argv[2] → `bus_url::parse` | argv[2] is user-controlled. The parser produces a `Route` enum; the `mirror_url` field is propagated to the shell-out (mitigated above). The `sot: ParsedRemote` flows to the existing `instantiate` path which already handles malicious origin URLs via `REPOSIX_ALLOWED_ORIGINS`. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-82-01 | Tampering | `git ls-remote` shell-out (PRECHECK A) — argument injection via `mirror_url` (e.g. `--upload-pack=evil`, `-c protocol.file.allow=user`) | mitigate | TWO-fold defense: (a) `bus_handler` rejects `mirror_url` whose first byte is `-` (returns error before shell-out — `fail_push(..., "bad-mirror-url", "mirror URL cannot start with `-`: <mirror_url>")`); (b) `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"])` passes `--` as positional separator. The `--` tells `git ls-remote` "every following argument is positional, not a flag." Combined, an attacker cannot smuggle a flag-shaped URL through the bus URL parser into the shell-out. Verifier: T05's `bus_precheck_a.rs` includes a `rejects_dash_prefixed_mirror_url` test that asserts the helper exits non-zero with the verbatim "mirror URL cannot start with `-`" message. Code review checkpoint: `crates/reposix-remote/src/bus_handler.rs` is grepped for the `--` literal at the `args(["ls-remote", "--", ...])` site BEFORE merge. |
| T-82-02 | Information Disclosure | Tainted SoT bytes leaking via bus_handler logs | accept | UNCHANGED from P81. `precheck_sot_drift_any` counts records via `changed.len()` — never extracts body bytes. The deferred-shipped error stub emits a static stderr string (`"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83"`). NO Tainted byte sinks introduced in P82. P83 will introduce the SoT-write logs and need its own analysis. |
| T-82-03 | Denial of Service | `git ls-remote` against private mirrors hangs on SSH-agent prompt (RESEARCH.md Pitfall 3) | accept | Documentation-only mitigation: CLAUDE.md § Architecture (new paragraph in T06) names the production requirement that users have SSH agent set up before bus push to private mirrors. Tests use `file://` fixture exclusively per RESEARCH.md Pitfall 3. If the prompt becomes a real production issue, future work could pass `GIT_TERMINAL_PROMPT=0` env var (forces non-interactive failure with clear error) — filed as v0.13.0 GOOD-TO-HAVE candidate, not P82 scope. |
| T-82-04 | Tampering | `git config --get-regexp` shell-out parsing — config-value injection (e.g. embedded newlines, whitespace, control bytes) | mitigate | Regex `^remote\..+\.url$` is helper-controlled (no string concatenation with user input). Output parsing uses `splitn(2, char::is_whitespace)` per line — robust against extra whitespace; multi-line values would be a `git config` bug, not a bus_handler bug. The matched `mirror_url` is byte-equal-compared (with trailing-slash normalization) to the parsed value — no further interpretation. Verifier: T05's `bus_precheck_a.rs` includes a `multi-match` fixture where two remotes point at the same URL; assertion is "first alphabetical chosen + WARN". |
| T-82-05 | Tampering | `git rev-parse refs/remotes/<name>/main` shell-out — `<name>` injection | mitigate | `<name>` is extracted from the matched config key via `key.strip_prefix("remote.").and_then(|k| k.strip_suffix(".url"))`. The middle is bounded by git's own remote-name validation (`git remote add` rejects weird characters). The shell-out command is `git rev-parse refs/remotes/<name>/main` — no `--` separator needed because `<name>` is guaranteed safe by construction. Code review checkpoint: the strip_prefix/strip_suffix shape is grep-able. |

No new HTTP origin in scope (PRECHECK B reuses the existing
`BackendConnector` allowlist). NEW `Tainted<T>` propagation path
introduced in P82 (mirror SHA from `git ls-remote` is byte-compared,
not parsed/executed/committed — bounded). Three new shell-out sites
all mitigated via D-06 + T-82-01/04/05. No new sanitization branch.
