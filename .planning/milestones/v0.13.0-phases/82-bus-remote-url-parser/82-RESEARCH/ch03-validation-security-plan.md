# Phase 82 Research — Validation, Security, Catalog Design, Plan Splitting

← [back to index](./index.md)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `Command::new("git")` is on PATH when the helper runs | Standard Stack | LOW — git invokes the helper, so it's PATH-resolvable. Verified by existing `doctor.rs` shell-outs. |
| A2 | `git rev-parse refs/remotes/<name>/main` is the right way to read the local mirror SHA | Pattern 3 | LOW — porcelain handles packed-refs correctly. Could also use `git ls-remote .` against the local working tree but rev-parse is more direct. |
| A3 | The `url` crate's `query_pairs()` handles `mirror=git@github.com:org/repo.git` correctly without percent-encoding | Don't Hand-Roll | MEDIUM — `@` and `:` are reserved in URL contexts. Verify in a parser unit test; if `url` chokes, fall back to manual `split_once('=')` after `split_once('?')`. |
| A4 | `precheck_sot_drift_any` returning `Stable` on no-cursor (first-push) is the right policy | Pattern 2 | LOW — matches first-push behaviour in P81's `precheck_export_against_changed_set`; the inner correctness check at SoT-write time (P83) is the safety net. |
| A5 | Multiple remotes pointing at the same `mirror_url` is a real edge case worth handling | Pitfall 4 | LOW — unusual but legal. First-alphabetical pick + WARN is conservative. |

## Open Questions

### Q-A: by-remote-name vs. by-URL-match for the no-remote-configured check
**What we know:** the bus URL carries `mirror=<url>` only — no remote NAME. The user's local git config has `remote.<name>.url = <url>` keyed by name.
**What's unclear:** to find the local remote, we either (a) scan `git config --get-regexp '^remote\..+\.url$'` and match values to `mirror_url`, or (b) require the bus URL to carry a NAME (`?mirror_name=gh&mirror_url=...`). Q3.5's hint *"`git remote add <name> <mirror-url>`"* doesn't pin which one.
**Recommendation:** **(a) by-URL-match.** The user has already named the remote; making them name it again in the bus URL is friction. URL-match is the canonical UX. Pitfall 4 documents the multi-match resolution (alphabetical + WARN). Planner ratifies.

### Q-B: does the bus handler reuse `handle_export` for write fan-out (P83) or have its own write loop?
**What we know:** P82 is dispatch-only — the question's resolution doesn't block P82 success criteria. P83 ROADMAP suggests *"the existing single-backend `handle_export` whose write logic the bus wraps verbatim per architecture-sketch § Tie-back to existing helper code"*.
**What's unclear:** whether the wrapping is at function-call granularity (`bus_export()` calls `handle_export()` after prechecks) or finer (factor `handle_export`'s body into helpers and call them from a new `bus_handle_export()`).
**Recommendation:** **P82 makes no commitment.** The bus path in P82 emits prechecks then a *temporary* "P83 not yet shipped" error to keep the surface minimal. P83's planner decides reuse strategy. P82's job is to land the URL parser, prechecks, and capability branching — every line of those is independent of P83's choice.

### Q-C: should bus URLs allow query params other than `mirror=`?
**What we know:** today only `mirror=` is meaningful. v0.13.0 RATIFIED `mirror=` as the sole syntax. Future params (`priority=`, `retry=`, `mirror_name=`) are not in scope.
**What's unclear:** silently ignore unknown keys vs. reject.
**Recommendation:** **REJECT unknown keys with a clear error.** Forward-compatibility-via-silent-ignore is a footgun (a typo `?mirorr=` becomes a no-op precheck-against-mirror-less remote). Reject lets us add new keys later without ambiguity. Pattern 1 code shows this.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| `git` binary on PATH | `Command::new("git")` shell-outs | ✓ | v2.34+ (project requirement) | none — git is the runtime contract |
| `cargo` workspace | building helper + tests | ✓ | 1.82+ | none |
| Rust crates already in workspace (`anyhow`, `chrono`, `tempfile`, `assert_cmd`, `wiremock`, `tokio`) | tests | ✓ | as-pinned | none |
| `url` crate | query-string parsing in `bus_url.rs` | ✓ (transitive via `reqwest`) | confirm in Cargo.lock | hand-rolled `split_once` if missing — cheap fallback |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `url` is transitive — if a future hygiene phase trims it, fall back to manual splitting. For now, prefer `url::form_urlencoded::parse(query.as_bytes())` if `url::Url::parse` is finicky on the `reposix::` scheme.

## Validation Architecture

### Test Framework
| Property | Value |
|---|---|
| Framework | `cargo test` (unit + integration); `cargo nextest run` for full-workspace runs (CLAUDE.md memory budget) |
| Config file | none — uses workspace `Cargo.toml` |
| Quick run command | `cargo test -p reposix-remote --test bus_url --test bus_precheck_a --test bus_precheck_b --test bus_capabilities` |
| Full suite command | `cargo nextest run -p reposix-remote` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| DVCS-BUS-URL-01 | URL parses `?mirror=` form; rejects `+` form | unit | `cargo test -p reposix-remote --test bus_url` | ❌ Wave 0 |
| DVCS-BUS-PRECHECK-01 | mirror drift emits `fetch first` + hint | integration | `cargo test -p reposix-remote --test bus_precheck_a` | ❌ Wave 0 |
| DVCS-BUS-PRECHECK-02 | SoT drift emits `fetch first` + cites mirror-lag refs | integration | `cargo test -p reposix-remote --test bus_precheck_b` | ❌ Wave 0 |
| DVCS-BUS-FETCH-01 | bus URL omits `stateless-connect` from capabilities | unit | `cargo test -p reposix-remote --test bus_capabilities` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-remote` (per-crate, per CLAUDE.md memory budget)
- **Per wave merge:** `cargo nextest run -p reposix-remote`
- **Phase gate:** Pre-push hook runs workspace-wide; verifier subagent runs catalog gates.

### Wave 0 Gaps
- [ ] `crates/reposix-remote/src/bus_url.rs` — new module
- [ ] `crates/reposix-remote/src/bus_handler.rs` — new module
- [ ] `crates/reposix-remote/tests/bus_url.rs` — new test file
- [ ] `crates/reposix-remote/tests/bus_precheck_a.rs` — new test file (with file:// bare-repo fixture)
- [ ] `crates/reposix-remote/tests/bus_precheck_b.rs` — new test file (wiremock-backed; mirror precheck_a.rs's wiremock idiom)
- [ ] `crates/reposix-remote/tests/bus_capabilities.rs` — new test file
- [ ] `quality/catalogs/agent-ux.json` — 5 new rows minted BEFORE implementation per QG-06
- [ ] `quality/gates/agent-ux/bus-*.sh` — 5 new verifier scripts (one per row)
- [ ] CLAUDE.md update — § Architecture (mention bus URL form) + § Commands (the new push form)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | yes (transitively) | `git ls-remote` against private mirrors uses SSH agent / git creds. The helper does NOT store or transmit credentials directly — relies on the user's existing git config. |
| V3 Session Management | no | helper is per-invocation. |
| V4 Access Control | yes | `REPOSIX_ALLOWED_ORIGINS` allowlist already gates SoT egress. PRECHECK B's `list_changed_since` call goes through the same allowlisted client. |
| V5 Input Validation | yes | URL parsing must reject malformed forms; the `?mirror=` value is bound for SHELL-OUT (`git ls-remote <mirror_url>`) so injection is the canonical risk. **MUST** treat `mirror_url` as untrusted; reject characters that could be interpreted as flags by `git ls-remote` (leading `--`). |
| V6 Cryptography | no | helper does no crypto directly. |

### Known Threat Patterns for the bus path

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Argument-injection via `mirror_url` (e.g., `--upload-pack=evil`) | Tampering | Reject `mirror_url` values starting with `-`; pass `--` separator before arguments to `git ls-remote`. Idiom: `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"])`. |
| Tainted `mirror_url` from a malicious git config | Tampering | The user controls their own git config; treating it as trusted is in keeping with the project threat model. But the BUS URL is in `argv` from `git push` — same trust origin (user). Document explicitly in `bus_url.rs` module-doc that `mirror_url` is treated as user-controlled, not attacker-controlled. |
| Mirror SHA from `git ls-remote` is attacker-influenced | Tampering | The SHA is only compared (byte-equal) to the local SHA; it's not parsed, executed, or committed. Tainted-bytes flow is bounded — no `Tainted<T>` wrapper needed for SHAs. |
| Allowlist bypass via SoT URL change | Tampering | The SoT URL is parsed and the SoT BackendConnector is built before any precheck runs. Existing allowlist enforcement (`reposix_core::http::client()`) covers PRECHECK B's `list_changed_since` call. |

## Catalog Row Design

Per QG-06 (catalog-first), the FIRST commit of the implementing phase mints these rows in `quality/catalogs/agent-ux.json` (or a new `bus-remote.json` — recommend keeping in `agent-ux.json` to match the existing `agent-ux/dark-factory-sim`, `agent-ux/reposix-attach-against-vanilla-clone`, `agent-ux/mirror-refs-*` neighbours).

**Five proposed rows** (verifier scripts under `quality/gates/agent-ux/`):

1. **`agent-ux/bus-url-parses-query-param-form`**
   - `verifier`: `quality/gates/agent-ux/bus-url-parse.sh` — runs `cargo test -p reposix-remote --test bus_url -- --exact bus_url_parses_query_param_form`
   - `kind`: mechanical
   - `cadence`: pre-pr
   - asserts: parser returns `Route::Bus { sot: <expected>, mirror_url: <expected> }` for `reposix::sim::demo?mirror=git@github.com:org/r.git`
2. **`agent-ux/bus-url-rejects-plus-delimited`**
   - `verifier`: `quality/gates/agent-ux/bus-url-reject-plus.sh`
   - asserts: helper exits non-zero with stderr containing `"+`-delimited"` AND `"?mirror="`
3. **`agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`**
   - `verifier`: `quality/gates/agent-ux/bus-precheck-a-mirror-drift.sh`
   - kind: mechanical (uses file:// fixture, sim, assert_cmd-style invocation)
   - asserts: stdout contains `error refs/heads/main fetch first`; stderr contains `your GH mirror has new commits`; helper exited before any stdin was read (`fast-import` parsing not invoked — assertable via cache audit row count `helper_push_started=1, helper_push_accepted=0`)
4. **`agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`**
   - `verifier`: `quality/gates/agent-ux/bus-precheck-b-sot-drift.sh`
   - kind: mechanical
   - asserts: stdout `error refs/heads/main fetch first`; stderr cites `refs/mirrors/<sot>-synced-at` (when populated) — uses the existing `read_mirror_synced_at` helper from P80
5. **`agent-ux/bus-fetch-not-advertised`**
   - `verifier`: `quality/gates/agent-ux/bus-fetch-not-advertised.sh`
   - kind: mechanical
   - asserts: capability list emitted on stdout for a bus URL contains `import`, `export`, `refspec`, `object-format=sha1` but NOT `stateless-connect` (DVCS-BUS-FETCH-01)

**Sixth row (recommended, not required by phase requirements):**

6. **`agent-ux/bus-no-mirror-remote-configured-error`** — covers Q3.5 / DVCS-BUS-WRITE-05's P82 portion (success criterion 5 of P82 ROADMAP).
   - asserts: bus URL referencing a `mirror_url` not in any local `remote.<name>.url` fails with verbatim hint `"configure the mirror remote first: git remote add <name> <mirror-url>"`. NO auto-mutation of git config.

The roadmap explicitly lists 5 + the no-remote case as success criteria. Rows 1-5 + row 6 = full coverage. Plan to mint 6 rows.

## Test Fixture Strategy (PRECHECK A — drifting mirror)

**Two options:**

**(a) Two local bare repos.** `mktemp -d`; `git init --bare mirror.git`; seed a commit; `git init --bare working-copy.git`; clone mirror; commit + push to mirror with `--force` to make it diverge from working-copy's `refs/remotes/mirror/main`. Bus URL points at `file:///tmp/.../mirror.git`. **No network. No SSH agent. No wiremock.** ~30 lines of bash.

**(b) wiremock-backed mirror via reposix-sim.** sim doesn't speak smart-HTTP; this would require a brand-new mock server speaking the git smart-HTTP protocol. Out of scope; non-trivial.

**Recommend (a).** It's the project's existing idiom (`scripts/dark-factory-test.sh` uses local bare repos for the same reason). The fixture also makes the test fast (<2s) and offline.

For PRECHECK B, **wiremock** is the right answer — `precheck.rs` already uses wiremock in `crates/reposix-remote/tests/perf_l1.rs`. The bus_precheck_b test mirrors that idiom: spawn wiremock, mock `list_changed_since` to return `[id 5]`, set `last_fetched_at` in the cache, run the helper against the bus URL, assert the rejection message + that NO writes hit wiremock (`Mock::expect(0)` on PATCH).

## Plan Splitting

**Recommend: SINGLE plan.** Phase 82 has 7 success criteria but they all sit in `crates/reposix-remote/`. Cargo-heavy task count:

| Task | Cargo work | Notes |
|---|---|---|
| T1: catalog rows + CLAUDE.md update | none | doc/JSON only |
| T2: `bus_url.rs` + unit tests | per-crate `cargo test -p reposix-remote --test bus_url` | small |
| T3: `precheck_sot_drift_any` wrapper + unit test | per-crate `cargo test -p reposix-remote --lib precheck` | small |
| T4: `bus_handler.rs` (STEP 0 + PRECHECK A + PRECHECK B + capability branching) + integration tests | per-crate `cargo test -p reposix-remote --test bus_*` | medium — 4 integration tests bundled |
| T5: verifier scripts (5+1) + catalog refresh | shell only | doc/script work |
| T6: phase close — `git push origin main`; verifier subagent | none | gate |

Six tasks; four are doc/JSON/shell. Two cargo-heavy tasks (T2 + T4) — well under the ≤4 cargo-heavy ceiling per CLAUDE.md memory budget. **Single plan stands.**

The `bus_handler.rs` body (T4) is where complexity sits — but the prechecks are independent (a fail in PRECHECK A short-circuits before B even runs), so the test surface can be parallelized across `bus_precheck_a.rs` and `bus_precheck_b.rs` without sharing state.

## Sources

### Primary (HIGH confidence)
- `crates/reposix-remote/src/main.rs` (full body, 687 lines) — current dispatch loop, capabilities arm, `handle_export`, lazy cache, ProtoReader.
- `crates/reposix-remote/src/precheck.rs` (302 lines) — P81's L1 precheck function; M1 narrow-deps signature.
- `crates/reposix-remote/src/backend_dispatch.rs` (537 lines) — `parse_remote_url`, `BackendKind`, `instantiate`, env-var-keyed credential checks.
- `crates/reposix-cache/src/mirror_refs.rs` (371 lines) — P80's mirror-lag refs, `read_mirror_synced_at`.
- `crates/reposix-core/src/backend.rs:253` — `BackendConnector::list_changed_since` signature.
- `crates/reposix-core/src/remote.rs:43` — `split_reposix_url`.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § 3 — the bus algorithm.
- `.planning/research/v0.13.0-dvcs/decisions.md` § Phase-N+2 — Q3.1–Q3.6.
- `.planning/ROADMAP.md` lines 124–143 — phase scope + 7 success criteria.
- `.planning/REQUIREMENTS.md` lines 71–80 — DVCS-BUS-* IDs.
- `quality/gates/agent-ux/reposix-attach.sh` — verifier idiom for new agent-ux rows.

### Secondary (MEDIUM confidence)
- `crates/reposix-cli/src/doctor.rs:446-944` — existing `Command::new("git")` shell-out idiom.
- `crates/reposix-remote/tests/perf_l1.rs` (lines 1–90) — wiremock idiom for SoT-side tests; `NoSinceQueryParam` / `HasSinceQueryParam` matchers.

### Tertiary (LOW confidence)
- None — every claim above ties to a verified source.

## Metadata

**Confidence breakdown:**
- URL parser design: HIGH — every API call cited to a verified source line.
- PRECHECK A choice (shell-out): HIGH — matches project idiom (doctor.rs).
- PRECHECK B reuse strategy: HIGH — wrapper pattern preserves P81 behaviour; uses existing `last_fetched_at` cursor.
- Capabilities advertisement branching: HIGH — current code at `main.rs:150-172` is a 5-line edit.
- No-remote-configured lookup (URL-match vs name): MEDIUM — Q-A is real, recommendation is conservative.
- Catalog row design: HIGH — mirrors P80 / P81 row shape verbatim.
- Test fixture strategy: HIGH — file:// fixture + wiremock are both pre-existing idioms.

**Research date:** 2026-05-01
**Valid until:** 2026-05-31 (30 days; bus URL spec is RATIFIED — only invalidates if Q3.x decisions reopen)
