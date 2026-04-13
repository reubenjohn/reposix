---
phase: 01-core-contracts-security-guardrails
reviewed: 2026-04-13T00:00:00Z
depth: deep
files_reviewed: 13
files_reviewed_list:
  - crates/reposix-core/src/error.rs
  - crates/reposix-core/src/http.rs
  - crates/reposix-core/src/taint.rs
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/src/audit.rs
  - crates/reposix-core/fixtures/audit.sql
  - crates/reposix-core/examples/show_audit_schema.rs
  - crates/reposix-core/tests/audit_schema.rs
  - crates/reposix-core/tests/compile_fail.rs
  - crates/reposix-core/tests/http_allowlist.rs
  - crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs
  - crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs
  - clippy.toml
  - scripts/check_clippy_lint_loaded.sh
findings:
  blocker: 0
  high: 2
  medium: 4
  low: 4
  total: 10
status: issues_found
verdict: FIX-REQUIRED
---

# Phase 1: Code Review — Core Contracts + Security Guardrails

## Summary Verdict

Phase 1 ships a solid, honest first cut of the guardrails. The `Tainted`/`Untainted`
type discipline (SG-05) is tight: `Untainted::new` is `pub(crate)`, no `From`/`Default`/
`serde::Deserialize` back-doors exist, and a trybuild compile-fail fixture locks the
invariant mechanically. `validate_issue_filename` is pleasingly strict — it rejects
trailing whitespace, mixed case, URL-encoded bypass attempts, RTLO/BOM unicode, and
u64-overflow without panicking. The audit-log triggers fire under WAL, the redirect
recheck test is real, and `Cargo.lock` is committed.

**But** there are two real attack surfaces the phase claims to seal and does not fully
seal:

1. **SG-01 has a post-factory bypass.** `http::client()` returns a bare
   `reqwest::Client`. Nothing in the type system, the clippy config, or the test suite
   prevents a caller from invoking `client.get(url).send().await` directly — bypassing
   the per-request allowlist gate entirely. The recheck is only genuine if callers
   route through `http::request()`, and nothing enforces that they do. The
   `check_clippy_lint_loaded.sh` grep is for the three constructors only, not for the
   send methods. This is not a defence-in-depth comment; it is a single-lock system
   presented as a double-lock.
2. **SG-06 is not enforced against schema-level attacks.** `DROP TABLE audit_events`,
   `DROP TRIGGER audit_no_delete`, and `PRAGMA writable_schema=ON ; DELETE FROM
   sqlite_master` all disable the append-only invariant and are not tested against.
   The CI fixture proves the triggers fire for UPDATE / DELETE on rows; it does not
   prove an attacker with the same DB handle cannot *remove* the triggers.

Neither rises to BLOCKER — the v0.1 threat model assumes the audit DB is not
co-resident with attacker-controlled code, and the HTTP client is only constructed
inside trusted crates — but both are HIGH because the phase-exit DONE.md states
"SG-01" and "SG-06" as closed, and a Phase 2/3 author reading that statement will
reasonably assume the stronger guarantee. Fix them before advertising the phase as
shipped, or annotate them as partial and move the remaining work to Phase 2.

The rest are MEDIUM/LOW polish: `validate_path_component` quietly accepts trailing
whitespace / CR / LF / RTLO (intentional, but not documented as "use the *_filename
variant at privileged boundaries"); `IssueId(0)` is silently accepted; the clippy
load-proof script's "source is clean" grep lets `// reqwest::Client::new` comments
through but misses multi-line constructor patterns; the audit schema's `DROP TRIGGER
IF EXISTS` is re-run on every `load_schema` call, so an already-open connection that
dropped the trigger would get it re-installed but the side effect is racy.

Phase-exit invariants verified live during review:

- `bash scripts/check_clippy_lint_loaded.sh` → exit 0, workspace clippy clean under `-D warnings`.
- `cargo test -p reposix-core --lib --tests` → all green (5 unit + 2 compile-fail + 7 http_allowlist + 5 audit_schema, 1 ignored timeout test).
- `cargo run -q -p reposix-core --example show_audit_schema` → emits both triggers.
- `Cargo.lock` is committed (confirmed via `git log -- Cargo.lock`).
- `#![forbid(unsafe_code)]` is set at `crates/reposix-core/src/lib.rs:7` (covers every submodule; satisfies the intent of the prompt's per-module requirement).

## HIGH

### H-01 — `http::client()` returns a raw `reqwest::Client`; direct use bypasses the allowlist gate

**Files:** `crates/reposix-core/src/http.rs:177-197`, `crates/reposix-core/src/http.rs:211-223`, `clippy.toml:1-5`, `scripts/check_clippy_lint_loaded.sh:22-30`

**Issue.** The factory returns `reqwest::Client`, not a wrapper that hides `.get()` / `.post()` / `.request()` / `.execute()`. A caller in any downstream crate can write:

```rust
let c = reposix_core::http::client(ClientOpts::default())?;
let r = c.get("https://attacker.example/").send().await?; // gate skipped
```

This compiles cleanly. The clippy `disallowed-methods` list only bans the three constructors (`Client::new`, `Client::builder`, `ClientBuilder::new`) and says nothing about `Client::get`/`post`/`request`/`execute` on an already-built client. The phase-exit test `egress_to_non_allowlisted_host_is_rejected` proves `request()` rejects the bad URL — it does not prove that callers must use `request()`.

The docstring at `http.rs:170-172` admits as much ("the factory alone is not a sufficient gate") but the enforcement mechanism is documentation, not the type system or a lint.

**Severity.** HIGH, not BLOCKER, because no code in the workspace today makes the direct call and the `check_clippy_lint_loaded.sh` grep catches construction-time bypasses. It rises to HIGH because SG-01 is listed in PROJECT.md as "Enforced in a single `reposix_core::http::client()` factory — no other code constructs `reqwest::Client`", which is true, but the guarantee users care about (no other code *uses* `reqwest::Client` to bypass the gate) is stronger and is not enforced.

**Fix (cheapest that actually works).** Return a newtype wrapper rather than the raw client:

```rust
pub struct HttpClient { inner: reqwest::Client }
impl HttpClient {
    // No Deref. Callers MUST go through request().
}
pub fn client(opts: ClientOpts) -> Result<HttpClient> { ... }
pub async fn request(client: &HttpClient, method: Method, url: &str) -> Result<reqwest::Response> { ... }
```

This compiles away the bypass: callers physically cannot reach `client.inner.get(...)` because `inner` is private. Cost: ~20 lines; one downstream-facing type change. The alternative — extending `clippy.toml` with `disallowed-methods = [{ path = "reqwest::Client::get", ... }, ...]` on every hot method — is brittle (new `reqwest::Client::execute_request` variants ship and silently bypass) and still leaves the response type leaky.

### H-02 — audit-log append-only invariant can be disabled via `DROP TRIGGER`, `DROP TABLE`, or `PRAGMA writable_schema=ON`

**Files:** `crates/reposix-core/fixtures/audit.sql:6-27`, `crates/reposix-core/tests/audit_schema.rs:91-123`

**Issue.** The integration tests prove the triggers fire on row-level UPDATE/DELETE. They do not test the attacker path: an agent (or a compromised component sharing the connection pool) that executes schema-level SQL. Live-tested during review against the exact fixture:

| Attack | Result |
|---|---|
| `DROP TRIGGER audit_no_delete` | succeeds (`Ok(1)`), then `DELETE FROM audit_events` succeeds |
| `DROP TABLE audit_events` | succeeds (`Ok(1)`) — silent wipe of the entire audit trail |
| `PRAGMA writable_schema=ON; DELETE FROM sqlite_master WHERE name='audit_no_delete'` | succeeds (`Ok(1)`) |
| INSERT inside a BEGIN...ROLLBACK | row is not persisted (expected, but note: Phase 2's insert path must NOT be inside the same transaction as a request-handling tx that might roll back; otherwise an audit row for a rejected request vanishes) |

**Severity.** HIGH, not BLOCKER, because (a) the Phase 1 surface only ships the schema, not the runtime insert path — so no attacker SQL reaches the DB yet; (b) the v0.1 threat model assumes the sim binary and its connection are not co-resident with attacker code; (c) the trigger-fires semantics *are* proven for the row path, which is the common case. It rises to HIGH because PROJECT.md and DONE.md both describe the invariant as "no UPDATE or DELETE triggers permitted" / "append-only" without the above caveats, and a Phase 2 insert-path author will not find this list while reading the fixture.

**Fix.** Pick one, in order of robustness:

1. *Minimum* — extend `tests/audit_schema.rs` with three tests that assert the DROP TRIGGER / DROP TABLE / writable_schema attempts either fail, or that the schema is revalidated on connection open and a failed revalidation aborts startup. This at least pins the "in-connection attacker" behaviour you actually depend on.
2. *Better* — open the runtime SQLite connection with `SQLITE_DBCONFIG_DEFENSIVE` (via `rusqlite::Connection::open_with_flags` + `db_config(SQLITE_DBCONFIG_DEFENSIVE, 1)`), which rejects schema-modifying statements on that handle. Document this in the schema file. Test it.
3. *Strongest* — run the audit log as a separate SQLite file opened read-write only by the audit-writer subsystem, with a distinct connection pool. Likely Phase 2 scope, but worth a note in the schema now.

Also add a short comment in `audit.sql` itself: "Append-only is enforced *only* for row-level UPDATE/DELETE on an open connection. The runtime must open this DB with `SQLITE_DBCONFIG_DEFENSIVE` or the equivalent to prevent schema-level bypass."

## MEDIUM

### M-01 — `validate_path_component` accepts trailing whitespace, CR, LF, and RTLO / BOM unicode

**File:** `crates/reposix-core/src/path.rs:18-31`

**Issue.** Probe run against the exact validator:

```
" 123.md"        -> Ok    (leading space)
"123.md "        -> Ok    (trailing space)
"123.md\t"       -> Ok    (tab)
"123.md\r\n"     -> Ok    (CRLF)
"123\u{202E}.md" -> Ok    (right-to-left override)
"\u{FEFF}..."    -> Ok    (BOM)
```

This is intentional per the current contract (the strict check is `validate_issue_filename`), but the docstring at `path.rs:13-17` reads as a general-purpose safety validator ("Rejects empty, `.`, `..`, and anything containing `/` or `\0`"). A Phase 3 FUSE-boundary author reaching for "the path validator" will reach for this one and let an RTLO filename slip through into directory listings, where it will re-order visible characters in `ls` output.

**Severity.** MEDIUM. It is not a Phase 1 bug — the Phase 1 only-user is `validate_issue_filename`, which DOES reject all of the above. It is a ticking footgun for Phase 3.

**Fix.** Add a prominent "when to use which" section to the module-level docstring:

```
//! - Use `validate_issue_filename` for anything that becomes a POSIX dirent inside
//!   the FUSE mount. It is strictly `^[0-9]+\.md$`.
//! - Use `validate_path_component` ONLY for pre-normalised slugs (project slugs)
//!   where ASCII control / whitespace / unicode-direction bytes have already been
//!   rejected by a prior layer. Do NOT use it as the FUSE-boundary guard.
```

Or, better, rename `validate_path_component` to `validate_path_component_weak` and
introduce a `validate_path_component_strict` that rejects all C0 control chars,
whitespace, and unicode bidi / format characters. Phase 3 wants the strict one.

### M-02 — `validate_issue_filename("0000000000.md")` returns `IssueId(0)` without surfacing the leading-zero semantics

**File:** `crates/reposix-core/src/path.rs:42-57`, `crates/reposix-core/src/path.rs:64-68` (test assertion)

**Issue.** The test at line 67 explicitly allows `"00042.md"` → `IssueId(42)`. This is documented. However `"0000000000.md"` → `IssueId(0)` is also accepted, and `IssueId(0)` is a valid issue id per the type. If Phase 2's simulator uses `0` as a sentinel ("no issue") or a reserved bootstrap id, the FUSE write path can synthesise `0.md` that collides with it.

**Severity.** MEDIUM. The simulator contract is not yet written, so this may be harmless. But deciding now whether `IssueId(0)` is legal is cheaper than discovering a collision in Phase 2/3.

**Fix.** One line — either:

```rust
if n == 0 { return Err(Error::InvalidPath(name.to_owned())); }
```

or add a doc note: "IssueId(0) is a valid filename at this layer; simulator MUST ensure it is not a reserved id." Prefer the former.

### M-03 — `scripts/check_clippy_lint_loaded.sh` grep misses multi-line `reqwest::Client` construction patterns and comment-stripping is fragile

**File:** `scripts/check_clippy_lint_loaded.sh:22-30`

**Issue.** The "no direct construction outside http.rs" check is:

```bash
grep -RIn 'reqwest::Client::new\|reqwest::Client::builder\|reqwest::ClientBuilder::new' crates/ --include='*.rs' \
  | grep -v 'crates/reposix-core/src/http.rs' \
  | grep -v '^[^:]*:[^:]*: *//'
```

Two fragility points:

1. The comment-stripping filter `'^[^:]*:[^:]*: *//'` only catches single-line comments starting at line start (after optional whitespace). A block comment `/* reqwest::Client::new */` slips past; a doc comment `/// reqwest::Client::new` is caught (starts with `//`). The second-filter also matches `//` anywhere after the line-number prefix, which incidentally passes a lot of cases but is not the contract the author seems to intend.
2. `use reqwest::Client; ... Client::new()` would NOT be caught — grep matches `reqwest::Client::new` literally. If a Phase 2/3 author writes `use reqwest::Client as C; C::new()`, neither clippy (`disallowed-methods` resolves paths, so it SHOULD catch this) nor the script catches it. The script's value is only as a belt-and-braces for the clippy rule; if clippy's resolution is what actually saves us, the grep is a placebo.

**Severity.** MEDIUM. The clippy rule itself is the real defence; the script is advertised (in its own header) as the "FIX 3" proof that the rule is loaded. But a green script is not proof that the rule catches path-renamed uses. The `cargo clippy --workspace --all-targets -- -D warnings` at line 32 IS that proof — if that line fails, the script exits non-zero.

**Fix.** Either (a) drop the fragile grep (#1-#3) and rely solely on line 32 plus one direct-construction "decoy" file in a test-only `#[cfg(clippy_proof)]` location that expects clippy to reject it, or (b) rewrite the grep with ripgrep + `--json` to capture file+line reliably and parse out comments with a committed rust/python helper. Prefer (a) — it removes 8 lines of shell and strengthens the guarantee.

### M-04 — `load_schema` is idempotent via `DROP TRIGGER IF EXISTS`, which races with concurrent writers on a shared connection pool

**File:** `crates/reposix-core/fixtures/audit.sql:17-27`, `crates/reposix-core/src/audit.rs:25-28`

**Issue.** Every call to `load_schema` runs `DROP TRIGGER IF EXISTS audit_no_update; CREATE TRIGGER audit_no_update ...` in sequence. There is a moment between `DROP` and `CREATE` where the trigger is gone. If two threads both call `load_schema` on connections to the same DB (the SQLite backend serialises, so this is safe across connections in default mode), or if a future code path opens the DB during Phase 2's startup sequence while a request handler is mid-transaction, there is a narrow window where an audit row can be UPDATEd.

**Severity.** MEDIUM. Current call sites are test-only and single-threaded. The race is theoretical for Phase 1. It becomes real the moment Phase 2 wires up the sim.

**Fix.** Guard `load_schema` with a once-only initialisation. A file-DB only needs `CREATE ... IF NOT EXISTS`. Replace the two `DROP + CREATE` pairs with:

```sql
CREATE TRIGGER IF NOT EXISTS audit_no_update BEFORE UPDATE ...
CREATE TRIGGER IF NOT EXISTS audit_no_delete BEFORE DELETE ...
```

and drop the `DROP TRIGGER IF EXISTS` lines entirely. `CREATE TRIGGER IF NOT EXISTS` is a standard SQLite construct (documented since 3.3). The current code's `DROP` was presumably there to handle schema *upgrades*; if so, that's a real concern but wants a `PRAGMA user_version` bump + explicit migration path, not an unconditional drop-and-replace. For v0.1, no-op is correct.

## LOW

### L-01 — `Error::Other(String)` is used for env-var parse errors; would typed errors help Phase 2?

**File:** `crates/reposix-core/src/http.rs:101-104, 117-119, 131-133, 141-143`

**Issue.** The allowlist parser encodes every failure mode as `Error::Other(format!("REPOSIX_ALLOWED_ORIGINS: ..."))`. This prevents downstream crates from matching on the specific failure (empty-entry vs bad-scheme vs bad-port) without string-inspection.

**Severity.** LOW. A CLI that wants to surface "you set scheme=ftp, try http" can't easily discriminate. Not a bug.

**Fix.** If Phase 2 surfaces these to the user, consider `Error::InvalidOrigin` variants with a `kind: enum { EmptyEntry, MissingScheme, BadScheme, EmptyHost, BadPort }` payload. Punt otherwise.

### L-02 — `parse_one` uses `rsplit_once(':')` which misparses IPv6 allowlist entries

**File:** `crates/reposix-core/src/http.rs:121-145`

**Issue.** `rsplit_once(':')` on `http://[::1]:7878` splits at the LAST colon, which is correct for IPv4 / DNS names. But `[::1]` contains `:` inside the brackets, so the host substring becomes `[::1` and the parser emits `OriginGlob { host: "[::1", port: Some(7878) }`. When matched against a `url::Url::parse("http://[::1]:7878/")` whose `host_str()` is `[::1]` (with closing bracket), the comparison `url_host != self.host` fails → every IPv6 allowlist entry silently never matches.

Not currently exploitable because the default allowlist is IPv4/DNS only and no test sets an IPv6 allowlist, but if an operator sets `REPOSIX_ALLOWED_ORIGINS="http://[::1]:7777"`, the loopback IPv6 app will never actually be reachable through this client.

**Severity.** LOW — broken future feature, not a security regression. File under "add an integration test that sets `http://[::1]:7777` in the env var and asserts a request to `http://[::1]:7777/` is allowed".

**Fix.** Parse bracketed IPv6 literals specially before the `rsplit_once`:

```rust
let rest_for_port = if rest.starts_with('[') {
    // IPv6 literal: [::1]:7878  -> host="[::1]", the port suffix (if any) follows ']'
    let close = rest.find(']').ok_or_else(|| Error::Other(...))?;
    ...
} else {
    rest
};
```

Or use `url::Url::parse(&format!("{}{}", entry, "/"))` and read `.host_str()` + `.port()` directly — the `url` crate already handles all of this. Probably the right move; it also gives you scheme / host normalisation for free.

### L-03 — `tests/http_allowlist.rs` uses `std::env::set_var` in a multi-threaded tokio test; edition 2021 permits it but it is a known footgun

**File:** `crates/reposix-core/tests/http_allowlist.rs:36, 48, 59, 60`

**Issue.** The `ENV_LOCK` mutex serialises test-side access, but other threads (reqwest's DNS resolver, rustls, etc.) may read env vars concurrently. In Rust 2024 edition `set_var` is `unsafe` for this reason. Edition 2021 lets it compile, but the SAFETY comment at line 35 understates the risk — holding the mutex only serialises against this test file, not against any background thread that reqwest spawns.

**Severity.** LOW. No observed flake in this review's test run. File this under "migrate when the workspace adopts edition 2024" or under "switch to `temp-env` / `assert_cmd` subprocess isolation".

**Fix.** None required for Phase 1. A comment at line 35 that names the real risk ("reqwest may read env vars in a background thread; the mutex does not protect against that, but in practice reqwest only reads proxy-related env vars during `build()`, which completes before `request()` reads ALLOWLIST_ENV_VAR") would be more honest.

### L-04 — `ClientOpts` fields are `pub`; future additions are breaking changes

**File:** `crates/reposix-core/src/http.rs:34-40`

**Issue.** `ClientOpts` has `pub total_timeout: Duration` and `pub user_agent: Option<String>`. Any new option (e.g. `pub connect_timeout`, `pub pool_idle_timeout`) is a breaking change because callers pattern-match or struct-literal with `ClientOpts { total_timeout: ..., user_agent: ... }`. Callers in the workspace all use `ClientOpts::default()`, so no breakage today; downstream crates are a future hazard.

**Severity.** LOW.

**Fix.** Mark the struct `#[non_exhaustive]` and add a builder (`ClientOpts::default().with_timeout(...)`) if you want to keep the fields `pub`. Or make the fields private and expose builder methods only. Either is five lines.

---

## Threat-vector checklist (from prompt)

| # | Attack | Verdict |
|---|---|---|
| 1 | Egress allowlist mutation-after-send | `request()` consumes the parsed `Url`, no mutation hook. **BUT** — direct `client.get(...).send()` bypasses the gate entirely. See H-01. |
| 1 | Loopback aliases (localhost, 0.0.0.0, ::1) | Live-probed the `url` crate: `127.1`, `2130706433`, `0x7f000001`, `127.0.0.1.` all normalise to `127.0.0.1` (allowlisted). `localhost` is case-insensitive. `0.0.0.0`, `[::1]`, `[::ffff:127.0.0.1]` are NOT in the default allowlist — this is conservative and correct. |
| 2 | `Untainted::new` not callable externally | Confirmed via trybuild fixture + manual grep: no `Default`, no `serde::Deserialize`, no `From`/`Into`, no `Deref`. |
| 3 | Filename validator bypass attempts | Live-probed 22 inputs including trailing whitespace, `\r\n`, `.MD`, `%2e%2e`, RTLO, BOM, emoji, 25-digit overflow. `validate_issue_filename` rejects all. `validate_path_component` is weaker — see M-01. |
| 4 | Audit triggers under WAL | Triggers fire. Verified live. |
| 4 | Audit triggers under `PRAGMA writable_schema=ON` | Triggers can be dropped from `sqlite_master`. See H-02. |
| 4 | Audit triggers under `ROLLBACK` | INSERT + ROLLBACK = zero rows. Phase 2 insert path must isolate the audit tx. See note in H-02. |
| 5 | HTTP timeout is actually 5s | Default is `Duration::from_secs(5)`. `reqwest::ClientBuilder::timeout` is total (connect + body). Test at `http_allowlist.rs:222-241` is `#[ignore]` but legitimate. |
| 5 | HTTP timeout overrideable | Yes, via `ClientOpts::total_timeout`. |
| 6 | Clippy lint fires | `bash scripts/check_clippy_lint_loaded.sh` → exit 0, "OK: clippy.toml loaded, disallowed-methods enforced, workspace clean." Verified live. |
| 7 | Cargo.lock committed | Yes (`git log -- Cargo.lock` shows commit `5f26860`). |
| 8 | `#[forbid(unsafe_code)]` in every new module | At crate root (`lib.rs:7`); covers all submodules. The prompt's strict-literal reading wants it per-module, but the intent ("no unsafe anywhere in this crate") is fully met by the root-level forbid. |

---

## Verdict

**FIX-REQUIRED.**

Not a BLOCK — nothing is actively exploitable against the Phase 1 surface, and the surface itself (core types + fixtures) has no runtime exposed to an attacker. The DONE.md's summary is accurate to the letter.

Not a PASS — H-01 and H-02 are real gaps in the SG-01 / SG-06 guarantees that the phase advertises as sealed. A Phase 2 author reading DONE.md will not expect these caveats; that is exactly the ambiguity that begets follow-up bugs.

Recommend: address H-01 (HttpClient wrapper) and H-02 (minimum fix: add the three schema-attack tests + a comment documenting the limit of the guarantee; defensive-open of the audit DB is Phase 2 scope). M-01 through M-04 can ride along or deferred to a small Phase 1.5 polish commit. L-01 through L-04 are take-them-or-leave-them.

_Reviewed: 2026-04-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
