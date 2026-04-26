# Persona audit: Security lead writing risk assessment

> **Persona.** Security-conscious infrastructure lead, Series B fintech, ~200 engineers,
> Claude Code in active use. Evaluating reposix v0.9.0 (alpha) against our security review
> committee's standard pilot intake.
> **Method.** Walked the public docs site
> (`reubenjohn.github.io/reposix/`) plus the source tree (`crates/reposix-core/src/http.rs`,
> `crates/reposix-cache/fixtures/cache_schema.sql`, `.github/workflows/release.yml`,
> `.github/workflows/release-plz.yml`, `.github/workflows/audit.yml`).

## TL;DR

**Yellowlight.** Sandbox/tainted-by-default architecture is genuinely well-designed
and the audit-log and egress-allowlist mitigations are *implemented*, not aspirational
— but the **supply-chain story (unsigned binaries, no SBOM, no SLSA provenance) and
the absence of an org-wide policy enforcement primitive** mean we cannot let this
self-deploy across the engineering org without compensating controls. Pilot is
defensible inside an isolated team with our own build pipeline.

## Threat model coverage (where reposix's claims hold and where they don't)

reposix names the lethal trifecta correctly (private data, untrusted input,
exfiltration via `git push` / outbound REST) and structures its mitigations as three
concentric rings: a `Tainted<T>` newtype at the cache boundary, a `sanitize()` step
that strips server-controlled frontmatter fields (`id`, `created_at`, `version`,
`updated_at`), and an audit-logged egress gate.

**Claims that hold under inspection:**

1. **Frontmatter field allowlist is enforceable.** `helper_push_sanitized_field`
   audit events fire on the inbound `export` path; this is testable end-to-end via
   the agent-flow regression. Documented in `how-it-works/trust-model/`.
2. **Append-only audit log is real, not a comment.** `crates/reposix-cache/fixtures/cache_schema.sql`
   defines `audit_cache_no_update` and `audit_cache_no_delete` `BEFORE` triggers that
   `RAISE(ABORT, 'audit_events_cache is append-only')`, with a paired test
   (`audit_is_append_only.rs`) that asserts the literal error string. Same pattern
   in `reposix-core/fixtures/audit.sql` for the simulator.
3. **Single-funnel HTTP construction is enforced at compile time.** All clients
   route through `reposix_core::http::client()`; `clippy.toml`'s `disallowed-methods`
   lint blocks direct `reqwest::Client::new()` call sites; the `HttpClient`
   newtype wraps a private `inner: reqwest::Client` so callers physically cannot
   reach the unchecked send path.
4. **Lethal-trifecta cuts are *named explicitly* as cuts**, not handwaved as
   "security through architecture." The trust-model page lists the trifecta legs,
   the corresponding cut for each, and — crucially — the **explicit non-mitigations**
   (host shell access, simulator-seed taint, third-party panic backtraces,
   confused-deputy across multiple `reposix init`s, full cache-file swap). That
   honesty is what moves this from yellow to "defensible yellow."

**Claims that do not hold or need qualification:**

1. **"Egress allowlist defaults to localhost only" is *almost* true.**
   `crates/reposix-core/src/http.rs:33`: `DEFAULT_ALLOWLIST_RAW =
   "http://127.0.0.1:*,http://localhost:*"`. That is technically fail-closed
   against external origins, but it is fail-*open* to anything an attacker can
   bind on `127.0.0.1` / `localhost` — including `localhost:*` ports that a
   compromised local process might listen on (e.g., a malicious CLI tool, an
   `npm` postinstall hook, or another agent). For a multi-tenant dev box this is
   not the same as "deny by default."
2. **Token-leak surface is not uniform across backends.** `reposix-jira` and
   `reposix-confluence` implement hand-rolled `Debug` impls that print
   `api_token: "<redacted>"`, and the Confluence client routes errors through
   `redact_url()` to scrub tenant hostnames. **`reposix-github`'s
   `GithubReadOnlyBackend` derives `Debug` with `token: Option<String>` as a plain
   field** (`crates/reposix-github/src/lib.rs:106-117`). Any `tracing::error!("{:?}", backend)`,
   `dbg!()`, or panic stack-frame dump that captures this struct will print the
   GITHUB_TOKEN verbatim. The trust-model page also concedes that third-party
   panics carrying `Authorization` headers in scope are "out of reposix's hands."
3. **"Org-wide policy enforcement"** does not exist as a primitive. The allowlist
   is an env var. Nothing prevents a developer from clearing it, expanding it, or
   running a binary that ignores it. There is no signed policy file, no
   centralized denylist, no exit-via-proxy mode. The integration-with-your-agent
   guide is explicit: "Not a sandbox. An agent on the host can still `curl` the
   backend with the same token." We can enforce only via egress-network controls
   we already own (egress firewall, DNS denylist, mTLS proxy).

## Credentials & secrets

Tokens are read from process env: `GITHUB_TOKEN`, `ATLASSIAN_API_KEY` +
`ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`, `JIRA_API_TOKEN` + `JIRA_EMAIL`
+ `REPOSIX_JIRA_INSTANCE`. There is **no documented integration with a secrets
manager** (Vault, AWS Secrets Manager, 1Password CLI, sops). For our org, that
means tokens live in shells, `.env` files, or our internal credential helper —
i.e., wherever the developer puts them.

| Concern | Status |
| --- | --- |
| Tokens redacted in `Debug` (Confluence, JIRA) | Yes, hand-rolled + tested |
| Tokens redacted in `Debug` (GitHub) | **No — derives `Debug` with `token: Option<String>` exposed** |
| Tokens redacted in error messages | Confluence routes via `redact_url`; GitHub/JIRA partial |
| Tokens redacted in audit log | Yes (audit module comment: "responsible for hashing / redacting sensitive content before insert") |
| Tokens never written to disk by reposix | Believed yes (no on-disk creds path found); not formally claimed |
| Token rotation guidance | **None** in docs |
| Least-privilege scope guidance per backend | **None** in docs (we'd have to derive: GH issues:write only, Atlassian "Read/write Confluence pages", JIRA project-scoped) |

## Audit & compliance posture

For SOC 2 / fintech audit needs the audit log is **better than I expected** for an
alpha tool. Every materialize, fetch, push (accept/reject/sanitize), egress denial,
and blob-limit hit writes a row to a per-project SQLite DB at
`<XDG_CACHE_HOME>/reposix/<backend>-<project>.git/cache.db`. Operations
documented in the trust model: `materialize`, `egress_denied`, `delta_sync`,
`helper_connect`, `helper_advertise`, `helper_fetch`, `helper_fetch_error`,
`helper_push_started`, `helper_push_accepted`, `helper_push_rejected_conflict`,
`helper_push_sanitized_field`, `blob_limit_exceeded`.

**What works for compliance:**

- Append-only is enforced *in the database*, not in application code, so even an
  agent with file-write access to the DB can't `UPDATE` or `DELETE` rows without
  tripping the SQLite trigger and producing an error trail.
- Schema is in-tree (`crates/reposix-cache/fixtures/cache_schema.sql`), reviewable.
- "Did agent X read issue Y?" *is* answerable via the `materialize` event.

**What does not work for compliance out of the box:**

- The audit DB is **per-clone, on the developer's laptop**. There is no
  forwarding to a central log sink (Splunk, Datadog, CloudWatch). If the laptop
  is wiped, the trail is gone. The append-only protection only defends the live
  segment — the trust-model page concedes "full file swap undetectable."
- No documented retention policy, no documented schema-version pinning for
  long-term forensic queries.
- The audit log captures *helper* events but does not by itself capture *agent
  prompt context*. Linking "Claude Code session 12345 read issue 17" requires
  correlating reposix audit timestamps with the IDE/CLI's own session log. We'd
  need to build that correlator.

## Supply-chain story

This is where the assessment turns yellow.

**What `release.yml` does:**

- Builds five-platform binaries (`x86_64-musl`, `aarch64-musl`, two darwin,
  windows-msvc) on tag push.
- Computes `SHA256SUMS` of each tarball/zip.
- Publishes to GitHub Releases plus an installer script
  (`reposix-installer.sh`, `reposix-installer.ps1`) and a Homebrew tap.
- Installer is the classic `curl --proto '=https' --tlsv1.2 -LsSf URL | sh`
  pattern — TLS-pinned to https, but downloads + executes a shell script, then
  downloads and runs an unverified binary.

**What `release.yml` does *not* do:**

- **No code signing.** No cosign, no sigstore/Rekor, no Apple notarization, no
  Authenticode (Windows binary will SmartScreen-warn).
- **No SLSA provenance attestation.** No `actions/attest-build-provenance@v1`.
- **No SBOM.** No CycloneDX/SPDX output, no `syft`/`cargo-cyclonedx` step.
- **No reproducible-build claim.** Build is not deterministic by construction.

**Crates.io exposure.** `release-plz` runs on every push to main with
`CARGO_REGISTRY_TOKEN` and auto-publishes crates. There is no manual hold-back.
A compromised maintainer GitHub account → push to main → auto-publish to
crates.io. Mitigated only by the maintainer's account hygiene (GitHub 2FA, no
phished PAT). For a fintech, that is single-factor in the wrong direction.

**Dependency hygiene.** `cargo-audit` runs weekly via
`actions-rust-lang/audit@v1` (`audit.yml`). Good. But there is **no
`cargo-deny`** (license + duplicate + advisory + sources policy) and **no
GitHub `dependency-review-action`** on PRs. Advisories arriving after a release
are surfaced once a week, not at install time.

**Threat-model-and-critique research doc.** Referenced as
`research/threat-model-and-critique.md` in the in-repo `CLAUDE.md`, but does
not appear to be published to the docs site under `/research/`. The site lists
`initial-report` and `agentic-engineering-reference` only. **Either it's not
actually written yet, or it's not in the public-doc TOC.** We should ask before
pilot.

## What I'd require before pilot

1. **Signed releases (P0).** Cosign + Sigstore Rekor transparency log, or at
   minimum GitHub Actions `attest-build-provenance@v1` so we can verify the
   binary came from the published `release.yml` on the published commit.
   Unsigned `curl | sh` is a non-starter for any fintech production path,
   pilot or not.
2. **SBOM at release (P0).** CycloneDX JSON attached to the GitHub release.
   We need this for our existing dependency-review process.
3. **`cargo-deny` in CI (P0).** Sources policy locked to crates.io,
   advisory denylist, license allowlist. `cargo-audit` weekly is too slow.
4. **Hand-rolled redacting `Debug` on `GithubReadOnlyBackend` (P0).**
   Match the `reposix-jira` / `reposix-confluence` pattern with
   `api_token: "<redacted>"`. File a security issue on the public repo;
   this is a one-line fix with a regression test.
5. **Egress-allowlist enforcement at the network, not just env-var (P1).**
   We will route the pilot host's outbound through our existing
   egress proxy; allowlist `https://api.github.com` and `https://*.atlassian.net`
   only. We do not rely on `REPOSIX_ALLOWED_ORIGINS` as the sole cut.
6. **Audit-log forwarder (P1).** Tail `cache.db` `audit_events_cache` and
   ship to our SIEM. If the project doesn't ship a forwarder, we write a
   ~50-line Rust binary; the schema is stable enough.
7. **Token vault integration (P1).** Tokens from our credential helper, not
   `.env`. No `GITHUB_TOKEN` in shell history.
8. **Published threat-model-and-critique doc (P2).** Either confirm it
   exists or treat its absence as a planning gap. The CLAUDE.md references
   it; the site does not surface it.
9. **Documented least-privilege scopes per backend (P2).** The docs imply
   broad write scope; we'd commit to issues:write / project-scoped JIRA /
   space-scoped Confluence and verify reposix actually works under that
   ceiling.
10. **A pinned version, not "latest."** Lockfile-committed via `Cargo.lock`
    (already done) plus a hash-pinned binary URL in our deployment manifest.

## What's well-handled

Crediting where credit is due — this is unusually thoughtful for an alpha:

- **Tainted-by-default at the type level** with a single `sanitize()` conversion
  point and trybuild compile-fail tests for direct tainted egress. That's the
  CaMeL pattern executed seriously, not as a slogan.
- **One-funnel HTTP construction** with clippy-enforced ban on
  `reqwest::Client::new()` is the kind of *structural* control reviewers can
  actually verify in a code review.
- **Append-only audit at the database layer** with a tested literal error
  string (`'audit_events_cache is append-only'`). Defense in depth — even
  application-level bugs can't soft-delete.
- **The trust model page tells you what the cuts *don't* cover** —
  shell access, simulator-seed taint, third-party panics, full DB swap.
  That candor is rare and is the single biggest reason I'd open a pilot
  rather than reject outright.
- **Push-time conflict detection** uses standard git error semantics
  (`fetch first`), which means our own pre-existing git auditing/CI
  hooks compose naturally. No bespoke protocol to learn or audit.
- **Dark-factory regression test** (`scripts/dark-factory-test.sh`) gives us
  a single end-to-end command to re-validate that the agent-UX assumption
  ("agent uses pure git, no in-context learning") holds across upgrades. We
  can wire it into our own CI.

— Security Lead, Pilot Intake Committee
