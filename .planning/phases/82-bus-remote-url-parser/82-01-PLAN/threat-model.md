← [back to index](./index.md) · phase 82 plan 01

# Threat Model

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
</threat_model>
