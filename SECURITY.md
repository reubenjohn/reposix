# Security policy

## Threat model summary

reposix is, by construction, a textbook **lethal-trifecta** machine: it puts (1) **private data** (issue bodies, custom fields, attachments) on the same path as (2) **untrusted input** (every byte that came from the network is attacker-influenced text) and (3) **exfiltration** (`git push` is a side-effecting verb; the helper makes outbound HTTP). The design does not pretend any leg is missing — it cuts the path between them at every boundary.

The full analysis lives in [`docs/how-it-works/trust-model.md`](docs/how-it-works/trust-model.md) (user-facing) and [`.planning/research/v0.1-fuse-era/threat-model-and-critique.md`](.planning/research/v0.1-fuse-era/threat-model-and-critique.md) (full historical red-team report). Read those before proposing changes that touch egress, audit, or sanitization.

## Hard guardrails (test-enforced)

These are not aspirational. Each is wired into the type system, a clippy lint, or a runtime check, and each has at least one regression test.

| Guardrail | Where |
|---|---|
| **Egress allowlist** — every HTTP client built via `reposix_core::http::client()`; raw `reqwest::Client::new()` is banned by clippy `disallowed-methods`. Default origin allowlist: `http://127.0.0.1:*`. Override with `REPOSIX_ALLOWED_ORIGINS`. | `crates/reposix-core/src/http.rs` |
| **Frontmatter field allowlist** — server-controlled fields (`id`, `created_at`, `version`, `updated_at`) are stripped from inbound writes before the REST call. An attacker-authored body cannot poison server metadata. | helper push handler; audited as `helper_push_sanitized_field` |
| **`Tainted<T>` / `Untainted<T>` newtypes** — the cache returns `Tainted<Vec<u8>>`; `sanitize()` is the only safe path to a side-effecting call. A trybuild compile-fail test asserts you cannot send `Tainted<T>` to an egress sink without `sanitize`. | `crates/reposix-core/src/tainted.rs` |
| **Append-only audit log** — `BEFORE UPDATE` / `BEFORE DELETE` triggers on `audit_events_cache` raise an error. SQLite WAL mode. | `crates/reposix-cache/src/cache_schema.sql` |
| **Blob-fetch limit** — helper refuses any `command=fetch` carrying more `want` lines than `REPOSIX_BLOB_LIMIT` (default 200). Error message tells the agent to narrow scope via `git sparse-checkout`. | helper; audited as `blob_limit_exceeded` |
| **Push-time conflict detection** — helper checks backend version at push time; rejects stale-base pushes with `error refs/heads/main fetch first`. Prevents blind overwrite of writes that landed between clone and push. | helper; audited as `helper_push_rejected_conflict` |

The audit-log ops vocabulary is fixed and documented in [`docs/how-it-works/trust-model.md`](docs/how-it-works/trust-model.md#audit-log).

## Reporting a vulnerability

**Please do not open a public GitHub issue for security reports.**

Preferred channel:
- **GitHub Security Advisories** — open a private advisory at <https://github.com/reubenjohn/reposix/security/advisories/new>. This routes directly to the maintainer with no public visibility.

Fallback channel:
- **Email** — `reubenvjohn@gmail.com`. Subject line: `[reposix security] <one-line summary>`.

A PGP key is not currently published; if you need one, request it via the email above and we'll set one up. Use a GitHub Security Advisory in the meantime.

### What to include

- A description of the issue, including the leg(s) of the lethal trifecta it crosses.
- Steps to reproduce (a minimal `cargo test` or `bash` reproducer is ideal).
- The affected version (`cargo pkgid` output or git commit SHA).
- Your assessment of impact — read access? write access? credential leakage? audit-log tampering?

### Response SLA

We aim to acknowledge security reports within **7 days**. For confirmed vulnerabilities, we'll work with you on a coordinated-disclosure timeline (typical: fix released before public details, 30–90 days depending on severity). Reporters who wish to be credited will be listed in [Acknowledgments](#acknowledgments) once the fix ships.

## Security regression kit

A `scripts/security-regression.sh` script that exercises every guardrail end-to-end is **planned** (not yet implemented) and will be linked here once it lands. In the meantime, the existing `bash scripts/dark-factory-test.sh sim` exercises the egress-allowlist + audit-log guardrails as part of the dark-factory regression suite, and the unit tests in `crates/reposix-core/src/tainted.rs` + `crates/reposix-cache/` cover the type-system and SQL invariants.

## Supply chain

Cargo dependencies are pinned in `Cargo.lock` (committed). `cargo-deny` and `cargo-audit` integration is **planned / in-flight** in parallel work — once enabled in CI they will gate merges on advisory-database hits and license violations. Until then, `cargo audit` should be run manually before a release.

GitHub Actions versions are pinned via Dependabot configuration ([`.github/dependabot.yml`](.github/dependabot.yml)) so a workflow update is always a reviewable PR.

## Known limitations

Honesty about the threat model is a feature, not a footnote. The following are **not** mitigated by reposix and are not bugs to file:

- **Shell access bypasses every cut.** An attacker on the dev host can `curl` the backend with the same token. reposix is a substrate for safer agent loops, not a sandbox. The egress allowlist guards the helper and the cache; it does not guard the rest of the host.
- **The simulator is itself attacker-influenced.** Seed data is authored by an agent (or by a fixture written by an agent), so simulator runs are *also* tainted. The lethal-trifecta mitigations apply against the simulator just as hard as against a real backend.
- **Token leakage via crash logs.** A panicking helper that includes auth headers in its `RUST_BACKTRACE` output can leak credentials. Known credential headers are scrubbed before logging, but third-party crates panicking with a header in scope are out of reposix's hands.
- **Confused-deputy across backends.** A user with credentials for two backends and one allowlist entry can be tricked by a tainted issue body into directing writes at the wrong backend. The allowlist constrains *origin*; it does not constrain *intent*. Multi-backend egress is high-friction by design — the agent must run a separate `reposix init` per backend.
- **Cache compromise.** An attacker with write access to `cache.db` can replay or hide audit rows from older WAL segments. Append-only triggers prevent in-place tampering on the live segment but cannot defend against the file being swapped wholesale.

If you find a way around one of the *intended* mitigations, that's the report we want. If you find a way around something this list explicitly cedes, that's expected behaviour — though we're still happy to hear about novel attack chains.

## Acknowledgments

Security reporters who have helped harden reposix will be listed here. (None yet — be the first.)
