---
phase: 11-confluence-adapter
plan: B
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-cli/Cargo.toml
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/mount.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-fuse/Cargo.toml
  - crates/reposix-fuse/src/main.rs
  - .github/workflows/ci.yml
autonomous: true
requirements:
  - FC-03
  - FC-05
  - SG-01
user_setup:
  - service: atlassian
    why: "Live-wire testing of --backend confluence; CI integration-contract-confluence job"
    env_vars:
      - name: ATLASSIAN_API_KEY
        source: "https://id.atlassian.com/manage-profile/security/api-tokens"
      - name: ATLASSIAN_EMAIL
        source: "The exact account email shown at id.atlassian.com top-right (MUST match the token's issuing account — see 00-CREDENTIAL-STATUS.md)"
      - name: REPOSIX_CONFLUENCE_TENANT
        source: "Subdomain of your Atlassian Cloud tenant: `<tenant>.atlassian.net`"
    dashboard_config:
      - task: "gh secret set ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT"
        location: "GitHub repo → Settings → Secrets and variables → Actions (or `gh secret set <NAME>` CLI). Do NOT attempt this in autonomous mode — HANDOFF §4 forbids it."

must_haves:
  truths:
    - "`reposix list --backend confluence --project KEY` exits 2 with a clear error listing all three required env vars when any are missing"
    - "`reposix list --backend confluence --project KEY` constructs a `ConfluenceReadOnlyBackend` and calls `list_issues(KEY)` when all env vars are set"
    - "`reposix mount /path --backend confluence --project KEY` spawns the fuse daemon with `--backend-kind confluence`"
    - "`reposix-fuse --backend-kind confluence` refuses to start if `REPOSIX_ALLOWED_ORIGINS` does not contain `<tenant>.atlassian.net`"
    - "CI job `integration-contract-confluence` exists and gates on all three secrets being non-empty"
    - "`cargo test --workspace --locked` stays green (no CLI regressions)"
    - "`bash scripts/demos/smoke.sh` stays 4/4 green (Tier 1 demos untouched)"
  artifacts:
    - path: "crates/reposix-cli/src/list.rs"
      provides: "`ListBackend::Confluence` variant + dispatch arm constructing `ConfluenceReadOnlyBackend`"
      contains: "ListBackend::Confluence"
    - path: "crates/reposix-cli/src/mount.rs"
      provides: "`ListBackend::Confluence` arm in `MountProcess::spawn`, fail-fast allowlist guard + env-var check"
      contains: "ListBackend::Confluence"
    - path: "crates/reposix-fuse/src/main.rs"
      provides: "`BackendKind::Confluence` variant in the fuse binary; `build_backend` arm constructing `ConfluenceReadOnlyBackend`"
      contains: "BackendKind::Confluence"
    - path: ".github/workflows/ci.yml"
      provides: "`integration-contract-confluence` job mirroring `integration-contract` shape"
      contains: "integration-contract-confluence"
    - path: "crates/reposix-cli/Cargo.toml"
      provides: "`reposix-confluence = { path = \"../reposix-confluence\" }` under `[dependencies]`"
    - path: "crates/reposix-fuse/Cargo.toml"
      provides: "`reposix-confluence = { path = \"../reposix-confluence\" }` under `[dependencies]`"
  key_links:
    - from: "crates/reposix-cli/src/list.rs"
      to: "reposix_confluence::ConfluenceReadOnlyBackend::new"
      via: "`ListBackend::Confluence` arm in `run()`"
      pattern: "ConfluenceReadOnlyBackend::new\\("
    - from: "crates/reposix-fuse/src/main.rs"
      to: "reposix_confluence::ConfluenceReadOnlyBackend::new"
      via: "`build_backend` arm for `BackendKind::Confluence`"
      pattern: "ConfluenceReadOnlyBackend::new\\("
    - from: ".github/workflows/ci.yml `integration-contract-confluence`"
      to: "crates/reposix-confluence/tests/contract.rs `contract_confluence_live`"
      via: "`cargo test -p reposix-confluence -- --ignored`"
      pattern: "cargo test -p reposix-confluence.*--ignored"
---

<objective>
Wire `--backend confluence` into the CLI (`reposix list`, `reposix mount`) and the `reposix-fuse` binary. Add the `integration-contract-confluence` CI job gated on all three Atlassian secrets being set. Keep all existing CLI/fuse behavior unchanged — this is a purely additive `enum` variant + dispatch arm.

Purpose: Without this plan, the core crate (11-A) is library-only. After this plan, a user with the right env vars can invoke `reposix list --backend confluence --project REPOSIX` end-to-end. Runs in Wave 1 in parallel with 11-A because the only symbol this plan consumes from 11-A is `ConfluenceReadOnlyBackend::new` and `ConfluenceCreds` — both of which 11-A introduces in its Task 1 scaffold. (If 11-A's scaffold hasn't landed yet at execution time, this plan's cargo build will fail and the wave gate will hold it back.)

Output: Patches to the two CLI files, the fuse binary, their Cargo.toml manifests, and one new CI job.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/11-RESEARCH.md
@CLAUDE.md
@crates/reposix-cli/src/list.rs
@crates/reposix-cli/src/mount.rs
@crates/reposix-cli/src/main.rs
@crates/reposix-cli/Cargo.toml
@crates/reposix-fuse/src/main.rs
@crates/reposix-fuse/Cargo.toml
@.github/workflows/ci.yml

<interfaces>
<!-- From 11-A's Task 1 scaffold (MUST be present at Wave 1 build time): -->

```rust
// crates/reposix-confluence/src/lib.rs
pub struct ConfluenceCreds { pub email: String, pub api_token: String }
pub struct ConfluenceReadOnlyBackend { /* ... */ }
impl ConfluenceReadOnlyBackend {
    pub fn new(creds: ConfluenceCreds, tenant: &str) -> Result<Self>;
    pub fn new_with_base_url(creds: ConfluenceCreds, base_url: String) -> Result<Self>;
}
```

<!-- From CLAUDE.md / HANDOFF §4: -->
Env var names (LOCKED by CONTEXT):
- `ATLASSIAN_API_KEY` — API token
- `ATLASSIAN_EMAIL` — account email
- `REPOSIX_CONFLUENCE_TENANT` — tenant subdomain (e.g. `mycompany`)

The CLI MUST fail fast with ONE error listing all three if ANY is missing.

<!-- Pattern template from `crates/reposix-cli/src/mount.rs` (Github arm): -->
```rust
if backend == ListBackend::Github {
    let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
    if !raw.contains("api.github.com") {
        bail!("REPOSIX_ALLOWED_ORIGINS must include https://api.github.com for --backend github (got {raw:?})");
    }
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend `ListBackend` enum + `reposix list` dispatch</name>
  <files>
    crates/reposix-cli/Cargo.toml,
    crates/reposix-cli/src/list.rs
  </files>
  <behavior>
    - `reposix list --backend confluence --project REPOSIX` with ALL three env vars unset prints on stderr one error that mentions ALL of `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` in a single message, and exits with code 1 (anyhow's default bail code) or 2.
    - `reposix list --backend confluence --project REPOSIX` with ALL three env vars set and a `REPOSIX_ALLOWED_ORIGINS` that does not include the tenant proceeds to the backend construction, at which point `ConfluenceReadOnlyBackend::new(...)` succeeds (it validates tenant format, not allowlist). The `list_issues` call will then fail at the HttpClient SG-01 check with a clean error referencing the allowlist. That's the expected failure mode — we do NOT pre-validate the allowlist in `list.rs` because `list` is test/inspect surface and the error from HttpClient is already descriptive enough.
    - Unit test in `list.rs` `#[cfg(test)] mod tests`: `confluence_requires_all_three_env_vars` — clears the three env vars with `std::env::remove_var` (inside a `tokio::test` — serial-test attribute if multiple env-touching tests exist, else just document the hazard), calls `run("KEY".into(), "".into(), ListBackend::Confluence, ListFormat::Json).await`, asserts `Err(...)` with a message containing all three env-var names.
  </behavior>
  <action>
    In `crates/reposix-cli/Cargo.toml`, under `[dependencies]`:
    ```toml
    reposix-confluence = { path = "../reposix-confluence" }
    ```
    Place it alphabetically near `reposix-github`.

    In `crates/reposix-cli/src/list.rs`:

    1. Add import: `use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};`
    2. Extend the enum:
       ```rust
       #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
       pub enum ListBackend {
           Sim,
           Github,
           /// Real Atlassian Confluence Cloud REST v2. `--project` is the
           /// space key (e.g. `REPOSIX`). Requires `ATLASSIAN_API_KEY`,
           /// `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars plus
           /// `REPOSIX_ALLOWED_ORIGINS` that includes the tenant origin.
           Confluence,
       }
       ```
    3. Add a new arm in the `match backend` block inside `run`:
       ```rust
       ListBackend::Confluence => {
           let (email, token, tenant) = read_confluence_env()
               .context("confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, and REPOSIX_CONFLUENCE_TENANT env vars")?;
           let creds = ConfluenceCreds { email, api_token: token };
           let b = ConfluenceReadOnlyBackend::new(creds, &tenant)
               .context("build ConfluenceReadOnlyBackend")?;
           b.list_issues(&project).await.with_context(|| {
               format!(
                   "confluence list_issues space_key={project} (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)"
               )
           })?
       }
       ```
    4. Add the helper `read_confluence_env`:
       ```rust
       /// Read the three Atlassian env vars in one shot. On partial-set, returns
       /// an anyhow::Error whose message lists ALL three names and indicates
       /// which were empty — so the user fixes them in one round-trip instead of
       /// N error / edit / re-run cycles.
       fn read_confluence_env() -> anyhow::Result<(String, String, String)> {
           let email = std::env::var("ATLASSIAN_EMAIL").unwrap_or_default();
           let token = std::env::var("ATLASSIAN_API_KEY").unwrap_or_default();
           let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
           let mut missing: Vec<&'static str> = Vec::new();
           if email.is_empty()  { missing.push("ATLASSIAN_EMAIL"); }
           if token.is_empty()  { missing.push("ATLASSIAN_API_KEY"); }
           if tenant.is_empty() { missing.push("REPOSIX_CONFLUENCE_TENANT"); }
           if !missing.is_empty() {
               anyhow::bail!(
                   "confluence backend requires these env vars; currently unset: {}. \
                    Required: ATLASSIAN_EMAIL (your Atlassian account email), \
                    ATLASSIAN_API_KEY (token from id.atlassian.com/manage-profile/security/api-tokens), \
                    REPOSIX_CONFLUENCE_TENANT (your `<tenant>.atlassian.net` subdomain).",
                   missing.join(", ")
               );
           }
           Ok((email, token, tenant))
       }
       ```
    5. Add the unit test described in <behavior>. NOTE: mutating env vars in parallel tests is unsound in Rust. Guard with the `serial_test` crate, OR (simpler) use a pure-fn shape: extract a `read_confluence_env_from(get: impl Fn(&str) -> String)` that the unit test can call with a closure returning `""`. Prefer the pure-fn refactor — it's trivial and avoids adding a dev-dep.

    Run `cargo test -p reposix-cli --locked` and `cargo clippy -p reposix-cli --all-targets --locked -- -D warnings`.
  </action>
  <verify>
    <automated>cargo test -p reposix-cli --locked confluence &amp;&amp; cargo clippy -p reposix-cli --all-targets --locked -- -D warnings</automated>
  </verify>
  <done>
    `ListBackend::Confluence` dispatches to the new backend. Error messages cite all three env vars when any missing. Unit test proves it. Commit: `feat(11-B-1): reposix list --backend confluence dispatch`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Extend `MountProcess::spawn` + fuse binary for `--backend confluence`</name>
  <files>
    crates/reposix-cli/src/mount.rs,
    crates/reposix-fuse/Cargo.toml,
    crates/reposix-fuse/src/main.rs
  </files>
  <action>
    In `crates/reposix-cli/src/mount.rs`:

    1. Extend the `ListBackend::Github` allowlist guard to also handle `Confluence`:
       ```rust
       match backend {
           ListBackend::Sim => {}
           ListBackend::Github => {
               let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
               if !raw.contains("api.github.com") {
                   bail!("REPOSIX_ALLOWED_ORIGINS must include https://api.github.com for --backend github (got {raw:?})");
               }
           }
           ListBackend::Confluence => {
               let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
               let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
               if tenant.is_empty() {
                   bail!("REPOSIX_CONFLUENCE_TENANT must be set for --backend confluence");
               }
               let expected = format!("{tenant}.atlassian.net");
               if !raw.contains(&expected) {
                   bail!("REPOSIX_ALLOWED_ORIGINS must include https://{expected} for --backend confluence (got {raw:?})");
               }
               // Also assert the other two so we fail before spawning fuse.
               if std::env::var("ATLASSIAN_EMAIL").unwrap_or_default().is_empty()
                   || std::env::var("ATLASSIAN_API_KEY").unwrap_or_default().is_empty() {
                   bail!("confluence backend requires ATLASSIAN_EMAIL and ATLASSIAN_API_KEY env vars");
               }
           }
       }
       ```
    2. Extend the `backend_kind` match:
       ```rust
       let backend_kind = match backend {
           ListBackend::Sim => "sim",
           ListBackend::Github => "github",
           ListBackend::Confluence => "confluence",
       };
       ```

    In `crates/reposix-fuse/Cargo.toml`, add under `[dependencies]`:
    ```toml
    reposix-confluence = { path = "../reposix-confluence" }
    ```

    In `crates/reposix-fuse/src/main.rs`:

    1. Add import: `use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};`
    2. Extend `BackendKind`:
       ```rust
       #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
       enum BackendKind {
           Sim,
           Github,
           /// Real Atlassian Confluence Cloud REST v2 at `https://{tenant}.atlassian.net`.
           /// Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`
           /// env vars and an allowlist that contains the tenant origin.
           Confluence,
       }
       ```
    3. Extend `build_backend`:
       ```rust
       BackendKind::Confluence => {
           let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
           let email  = std::env::var("ATLASSIAN_EMAIL").unwrap_or_default();
           let token  = std::env::var("ATLASSIAN_API_KEY").unwrap_or_default();
           let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
           if tenant.is_empty() || email.is_empty() || token.is_empty() {
               bail!("confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT env vars");
           }
           let expected = format!("{tenant}.atlassian.net");
           if !raw.contains(&expected) {
               bail!(
                   "REPOSIX_ALLOWED_ORIGINS must include https://{expected} for --backend-kind confluence (got {raw:?})"
               );
           }
           let creds = ConfluenceCreds { email, api_token: token };
           let b = ConfluenceReadOnlyBackend::new(creds, &tenant)?;
           Ok(Arc::new(b))
       }
       ```
    4. Update the module-level doc comment's usage line to list the three variants.

    Run `cargo build --workspace --locked` and `cargo clippy --workspace --all-targets --locked -- -D warnings`.
  </action>
  <verify>
    <automated>cargo build --workspace --locked &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test -p reposix-cli -p reposix-fuse --locked</automated>
  </verify>
  <done>
    `reposix mount --backend confluence` and `reposix-fuse --backend-kind confluence` both compile, both fail fast when env vars incomplete. Commit: `feat(11-B-2): --backend confluence in mount + reposix-fuse binary`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Add `integration-contract-confluence` CI job</name>
  <files>
    .github/workflows/ci.yml
  </files>
  <action>
    In `.github/workflows/ci.yml`, add a new job after `integration-contract` (line ~111), structurally mirroring it:

    ```yaml
    integration-contract-confluence:
      # Hits real Atlassian Confluence via reposix-confluence's contract test.
      # Gated on ALL THREE secrets being present so fork PRs don't fail and
      # so an incomplete config surfaces a SKIP rather than a false failure.
      # The secrets MUST be set by the repo owner via:
      #   gh secret set ATLASSIAN_API_KEY
      #   gh secret set ATLASSIAN_EMAIL
      #   gh secret set REPOSIX_CONFLUENCE_TENANT
      #   gh secret set REPOSIX_CONFLUENCE_SPACE      # space key for the contract test
      # (see MORNING-BRIEF-v0.3.md for the one-shot command).
      name: integration (contract, real confluence)
      runs-on: ubuntu-latest
      needs: [test]
      if: ${{ secrets.ATLASSIAN_API_KEY != '' && secrets.ATLASSIAN_EMAIL != '' && secrets.REPOSIX_CONFLUENCE_TENANT != '' && secrets.REPOSIX_CONFLUENCE_SPACE != '' }}
      timeout-minutes: 5
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - uses: Swatinem/rust-cache@v2
        - name: Run reposix-confluence contract test against real Atlassian
          env:
            REPOSIX_ALLOWED_ORIGINS: http://127.0.0.1:*,https://${{ secrets.REPOSIX_CONFLUENCE_TENANT }}.atlassian.net
            ATLASSIAN_API_KEY: ${{ secrets.ATLASSIAN_API_KEY }}
            ATLASSIAN_EMAIL: ${{ secrets.ATLASSIAN_EMAIL }}
            REPOSIX_CONFLUENCE_TENANT: ${{ secrets.REPOSIX_CONFLUENCE_TENANT }}
            REPOSIX_CONFLUENCE_SPACE: ${{ secrets.REPOSIX_CONFLUENCE_SPACE }}
          run: cargo test -p reposix-confluence -- --ignored
    ```

    Validate YAML syntax locally:
    ```bash
    python -c "import yaml, sys; yaml.safe_load(open('.github/workflows/ci.yml'))"
    ```

    Do NOT set any secrets yourself — the user will do that per HANDOFF §4. MORNING-BRIEF-v0.3.md (Phase 11-F) will tell them exactly which commands to run.
  </action>
  <verify>
    <automated>python -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" &amp;&amp; grep -q 'integration-contract-confluence' .github/workflows/ci.yml &amp;&amp; grep -q "secrets.ATLASSIAN_API_KEY != ''" .github/workflows/ci.yml</automated>
  </verify>
  <done>
    CI YAML parses. Job exists, gated on all four secrets, references `cargo test -p reposix-confluence -- --ignored`. Commit: `ci(11-B-3): integration-contract-confluence job gated on atlassian secrets`.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| env vars → process | `ATLASSIAN_API_KEY` and `ATLASSIAN_EMAIL` are both authenticators; the process must not leak them into logs, error strings, or child-process argv. |
| CLI args → fuse binary | `--project` (space key) is user-supplied; must not be used in any shell construction. |
| `REPOSIX_ALLOWED_ORIGINS` → egress | SG-01 enforcement remains the HttpClient's job, but the CLI does a courtesy pre-check so users get a loud error instead of an opaque EIO. |
| CI secrets → logs | GitHub Actions redacts configured secrets in logs; confirm with the `cargo test -- --ignored` run that no `println!` or tracing span echoes the credentials. (reposix-confluence's manual Debug redact, T-11-01, handles this.) |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11B-01 | Information disclosure | Error messages in `list.rs` / `mount.rs` / fuse `main.rs` | mitigate | Error-bail strings name env-var NAMES but never their VALUES. Verified by grep in Task 1's unit test (the error message contains `"ATLASSIAN_API_KEY"` but NEVER any caller-supplied token). |
| T-11B-02 | Tampering / SSRF | `REPOSIX_CONFLUENCE_TENANT` in allowlist check | mitigate | `format!("{tenant}.atlassian.net")` is used for allowlist substring matching, NOT for URL construction. SSRF mitigation is in 11-A Task 2 (`ConfluenceReadOnlyBackend::new` tenant validation). The CLI trusts the tenant string only for human-readable error messages here. |
| T-11B-03 | Elevation of privilege | CI job running against real Atlassian | mitigate | Gated on ALL three secrets being present (`if:` clause). Fork PRs can't trigger the job because forks don't have the secrets. `needs: [test]` means a broken build can't burn an Atlassian API quota. |
| T-11B-04 | Repudiation | CI job leaking secrets into test output | accept | GitHub Actions redacts configured secrets by name from logs. The contract test (11-C) does not `println!` credentials. reposix-confluence's Debug redact (T-11-01) covers the backend's own trace spans. |
| T-11B-05 | Denial of service | CI job hitting real Atlassian on every push | mitigate | `timeout-minutes: 5` caps the job. The contract test hits ≤3 endpoints per run. Atlassian's soft cap is ~1000 req/min; per-push usage is negligible. |

Block-on-high: T-11B-01 mitigation is verified in Task 1's unit test.
</threat_model>

<verification>
Nyquist coverage:
- **Unit:** `confluence_requires_all_three_env_vars` in `list.rs` (pure-fn, no env mutation).
- **Compile-time:** `ValueEnum` derive for `Confluence` variants; `cargo build --workspace --locked` green.
- **Clippy:** `-D warnings` green.
- **Regression:** `cargo test --workspace --locked` stays at ≥180 tests, 0 failures. `bash scripts/demos/smoke.sh` stays 4/4 green (this plan does NOT modify any demo script).
- **YAML:** Python yaml.safe_load parses the workflow file.
- **Contract (live-wire):** deferred to 11-C. This plan's CI job is the harness, 11-C's `contract_confluence_live` is the payload.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -q 'Confluence,' crates/reposix-cli/src/list.rs` returns 0.
2. `grep -q 'Confluence,' crates/reposix-cli/src/mount.rs` returns 0.
3. `grep -q 'Confluence,' crates/reposix-fuse/src/main.rs` returns 0.
4. `grep -q 'reposix-confluence = { path = "../reposix-confluence" }' crates/reposix-cli/Cargo.toml` returns 0.
5. `grep -q 'reposix-confluence = { path = "../reposix-confluence" }' crates/reposix-fuse/Cargo.toml` returns 0.
6. `grep -q 'integration-contract-confluence' .github/workflows/ci.yml` returns 0.
7. `grep -q 'cargo test -p reposix-confluence -- --ignored' .github/workflows/ci.yml` returns 0.
8. `python -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` exits 0.
9. `cargo build --workspace --locked` exits 0.
10. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
11. `cargo test --workspace --locked` exits 0 with ≥180 tests passed.
12. `cargo test -p reposix-cli --locked confluence 2>&1 | grep -q 'test result: ok'` returns 0.
13. `env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT cargo run -q -p reposix-cli --bin reposix --locked -- list --backend confluence --project X 2>&1 | grep -q 'ATLASSIAN_API_KEY.*ATLASSIAN_EMAIL.*REPOSIX_CONFLUENCE_TENANT\|ATLASSIAN_EMAIL.*ATLASSIAN_API_KEY.*REPOSIX_CONFLUENCE_TENANT'` — exits 0 (error message lists all three in a single line, order-insensitive). If this pipeline is too finicky, relax to three separate `grep -q` calls on the same captured stderr.
14. `bash scripts/demos/smoke.sh` exits 0 (unchanged; regression check).
</success_criteria>

<rollback_plan>
If the enum variant addition triggers a clap `ValueEnum` build error:
1. Check that `#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]` is intact on both enums.
2. clap-derive cheats: variant name `Confluence` becomes CLI value `confluence` by default — no explicit rename needed.

If the fuse binary's `build_backend` fails to resolve `ConfluenceReadOnlyBackend`:
1. Check that `reposix-confluence = { path = "../reposix-confluence" }` is in `crates/reposix-fuse/Cargo.toml`.
2. Check that 11-A Task 1 (scaffold) has landed — use `cargo metadata | jq '.workspace_members[]' | grep reposix-confluence`.

If CI YAML is rejected by GitHub Actions:
1. Validate `if:` syntax — use `&&` between clauses, not `and`.
2. Use the GitHub Actions schema on VS Code / `actionlint` CLI if available.
3. Test with `gh workflow run ci.yml` manually (don't push to main blindly).

If the workspace test count drops below 180 due to inadvertent test-suite breakage:
1. `cargo test --workspace --locked 2>&1 | grep 'test result:'` to find which crate lost tests.
2. Bisect with `git diff HEAD~1` to isolate the regression.
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-B-SUMMARY.md` with:
- Any CLI help-text changes (document them verbatim so the README update in 11-E knows what to copy).
- Confirmation that `smoke.sh` stayed 4/4 green (paste the final line of output).
- Confirmation that env-var-missing error lists all three names.
- Workspace test count before/after.
</output>
