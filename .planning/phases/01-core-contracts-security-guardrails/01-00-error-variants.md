---
phase: 01-core-contracts-security-guardrails
plan: 00
type: execute
wave: 0
depends_on: []
files_modified:
  - crates/reposix-core/src/error.rs
  - crates/reposix-core/Cargo.toml
autonomous: true
requirements:
  - SG-01
  - SG-04
  - SG-07
user_setup: []

must_haves:
  truths:
    - "`crates/reposix-core/src/error.rs` defines `Error::InvalidOrigin(String)`, `Error::InvalidPath(String)`, and `Error::Http(#[from] reqwest::Error)` after this plan."
    - "`cargo check -p reposix-core` succeeds with the new variants present."
    - "`grep -E 'InvalidOrigin|InvalidPath|Http\\(' crates/reposix-core/src/error.rs` prints all three variants."
    - "`reqwest` is present in `[dependencies]` of `reposix-core/Cargo.toml` so `Error::Http(#[from] reqwest::Error)` resolves."
  artifacts:
    - path: "crates/reposix-core/src/error.rs"
      provides: "Error enum extended with InvalidOrigin, InvalidPath, Http variants — single-commit merge-collision avoidance for Wave-1 plans 01-01 and 01-02"
      contains: "InvalidOrigin"
    - path: "crates/reposix-core/Cargo.toml"
      provides: "reqwest in [dependencies] so Http variant compiles"
      contains: "reqwest"
  key_links:
    - from: "crates/reposix-core/src/error.rs"
      to: "crates/reposix-core/Cargo.toml"
      via: "Error::Http(#[from] reqwest::Error) requires reqwest in [dependencies]"
      pattern: "reqwest::Error"
---

<objective>
Wave-0 micro-plan that lands all three new `Error` variants (`InvalidOrigin`, `InvalidPath`, `Http`) in a single commit so Wave-1 plans 01-01 and 01-02 can run in parallel without colliding on `crates/reposix-core/src/error.rs`.

Purpose: eliminate the merge-collision blocker (plan-checker FIX 1) flagged because 01-01 and 01-02 both edit the same file in the same wave. By front-loading all variant additions here, both downstream plans only consume `Error::*` and never edit it.

Output:
  - `crates/reposix-core/src/error.rs` extended with three variants.
  - `crates/reposix-core/Cargo.toml` declaring `reqwest = { workspace = true }` in `[dependencies]` so the `#[from] reqwest::Error` impl resolves.
  - One commit, one task, no test code (the variants are exercised by 01-01 and 01-02 tests).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-core-contracts-security-guardrails/01-CONTEXT.md
@CLAUDE.md
@Cargo.toml
@crates/reposix-core/Cargo.toml
@crates/reposix-core/src/error.rs

<interfaces>
Existing `Error` enum (from `crates/reposix-core/src/error.rs`) — this plan ADDS variants, does not rewrite:

```rust
pub enum Error {
    Frontmatter(String),
    InvalidIssue(String),
    InvalidRemote(String),
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    Yaml(#[from] serde_yaml::Error),
    Other(String),
}
pub type Result<T> = std::result::Result<T, Error>;
```

Variants this plan MUST add (preserve existing variants and `#[derive]`s; append at the end of the enum):

```rust
/// URL rejected by the egress allowlist (SG-01).
#[error("blocked origin: {0}")]
InvalidOrigin(String),

/// Path/filename rejected by the path validator (SG-04).
#[error("invalid path: {0}")]
InvalidPath(String),

/// Underlying HTTP/transport error from reqwest.
#[error(transparent)]
Http(#[from] reqwest::Error),
```
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Wave-1 parallel executors → `error.rs` | Two plans editing the same file in the same wave is the merge-collision boundary; this plan removes that boundary by performing all edits in Wave 0. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-00 | Tampering (process integrity) | Wave-1 parallel writes to `crates/reposix-core/src/error.rs` could lose a variant on merge | mitigate | Land all three variants in Wave 0; downstream plans only read `Error::*`, never write `error.rs`. |
</threat_model>

<tasks>

<task type="auto">
  <name>Task 1: Add InvalidOrigin, InvalidPath, Http variants to Error</name>
  <files>
    crates/reposix-core/src/error.rs
    crates/reposix-core/Cargo.toml
  </files>
  <action>
    1. Edit `crates/reposix-core/Cargo.toml`:
       - Under `[dependencies]`, add `reqwest = { workspace = true }`. This is required so `Error::Http(#[from] reqwest::Error)` compiles. Do NOT add `[dev-dependencies]` here — plan 01-01 owns those (`tokio`, `wiremock`).
    2. Edit `crates/reposix-core/src/error.rs`: append the three variants at the end of the `Error` enum (do NOT alphabetize — preserve existing variant order to minimise diff noise):
       ```rust
       /// URL rejected by the egress allowlist (SG-01).
       #[error("blocked origin: {0}")]
       InvalidOrigin(String),
       /// Path/filename rejected by the path validator (SG-04).
       #[error("invalid path: {0}")]
       InvalidPath(String),
       /// Underlying HTTP/transport error from reqwest.
       #[error(transparent)]
       Http(#[from] reqwest::Error),
       ```
       Keep all existing `#[derive]`s on the enum (`Debug`, `thiserror::Error`, etc.). Do not introduce any other change.
    3. Run `cargo check -p reposix-core` to confirm the file still compiles.

    AVOID: adding any test code (Wave-1 plans own the tests). AVOID touching `lib.rs` (no re-export changes here). AVOID alphabetising existing variants. AVOID introducing `wiremock`, `tokio`, or `trybuild` here — they belong to 01-01 and 01-02 respectively.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo check -p reposix-core &amp;&amp; grep -q 'InvalidOrigin' crates/reposix-core/src/error.rs &amp;&amp; grep -q 'InvalidPath' crates/reposix-core/src/error.rs &amp;&amp; grep -q 'Http(#\[from\] reqwest::Error)' crates/reposix-core/src/error.rs &amp;&amp; grep -q 'reqwest = { workspace = true }' crates/reposix-core/Cargo.toml</automated>
  </verify>
  <done>
    All three variants land in one commit; `cargo check -p reposix-core` is green; Wave-1 plans 01-01 and 01-02 can now run in parallel without touching `error.rs`.
  </done>
</task>

</tasks>

<verification>
Phase-level checks this plan contributes to:

1. Unblocks Wave-1 parallelism by collapsing the `error.rs` write-set to a single commit.
2. `cargo check -p reposix-core` is green.
3. No semantic feature changes — purely structural; observable behaviour comes from 01-01/01-02.
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

    cd /home/reuben/workspace/reposix && \
      cargo check -p reposix-core && \
      grep -q 'InvalidOrigin\|InvalidPath\|Http' crates/reposix-core/src/error.rs

…then Wave 0 is complete and Wave 1 may begin. Plans 01-01 and 01-02 MUST NOT include `crates/reposix-core/src/error.rs` in their `files_modified` after this plan lands.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-00-SUMMARY.md` per the summary template. Must include: the line numbers where each new variant landed, confirmation that `reqwest` is in `[dependencies]`, and a forward note to plans 01-01/01-02 that `error.rs` is now off-limits to them.
</output>
