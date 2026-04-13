---
phase: S-stretch-write-path-and-remote-helper
plan: B
type: execute
wave: 2
depends_on: [A]
files_modified:
  - crates/reposix-remote/Cargo.toml
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/protocol.rs
  - crates/reposix-remote/src/fast_import.rs
  - crates/reposix-remote/src/diff.rs
  - crates/reposix-remote/src/client.rs
  - crates/reposix-remote/tests/protocol.rs
  - crates/reposix-remote/tests/export.rs
  - crates/reposix-remote/tests/bulk_delete_cap.rs
autonomous: true
requirements:
  - FC-04
  - SG-02
  - SG-03
  - SG-07
user_setup: []

# BUDGET: 60 min wall clock. SKIPPED if S-A overran by >15 min.
# Tasks 1 + 2 = MINIMUM VIABLE (capabilities + import + bulk-delete cap on a stub
# export). Task 3 (full export tree-diff → PATCH/POST/DELETE) is NICE-TO-HAVE.
# Even with only Tasks 1+2, the demo can show capabilities round-trip + the
# bulk-delete cap firing on a synthetic 6-delete commit.

must_haves:
  truths:
    - "`git-remote-reposix origin reposix::http://127.0.0.1:7878/projects/demo` reads `capabilities` from stdin and writes `import\\nexport\\nrefspec refs/heads/*:refs/reposix/*\\n\\n` to stdout, then `option <key> <value>` lines reply `unsupported` (never `error`)."
    - "`import refs/heads/main` followed by blank line emits a `feature done` line, then a deterministic fast-import stream: one `blob`/`data`/<bytes> entry per issue (sorted by id ASC), then a `commit refs/reposix/origin/main` with one `M 100644 :N <id>.md` entry per issue, terminated by `done`."
    - "Blob bytes are produced by `reposix_core::issue::frontmatter::render(&issue)` — the SAME function the FUSE read path uses. There is NO second renderer in this crate."
    - "`export` reads a fast-import stream from stdin, parses the new tree, fetches the prior tree from the simulator (no marks file in v0.1), and translates each path delta: new path → POST, removed path → DELETE candidate, modified path → PATCH with `If-Match: <prior version>`."
    - "Bulk-delete cap (SG-02): if the new tree-diff would yield > 5 deletes, the helper prints `error: refusing to push (would delete N issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)` to stderr, writes `error refs/heads/main bulk-delete` to stdout, and exits non-zero WITHOUT making any HTTP call. If the commit message contains the literal `[allow-bulk-delete]`, the cap is bypassed."
    - "All HTTP via `reposix_core::http::client(ClientOpts::default())?` and `HttpClient::request_with_headers_and_body` — no direct `reqwest::Client` construction."
    - "All outbound PATCH/POST bodies pass through `Tainted::new(...).then(sanitize).inner()` to strip server-controlled fields (defense in depth even though the source is the user's own commit)."
    - "All diagnostics on stderr ONLY. Stdout is reserved for protocol output. A dedicated `stderr!()` macro or a `Protocol::diag(&str)` helper enforces this — `println!` is banned in this crate (clippy-disallowed)."
  artifacts:
    - path: "crates/reposix-remote/src/protocol.rs"
      provides: "`Protocol` struct wrapping `Stdin`/`Stdout` with `read_line()`, `peek_line()`, `expect_blank()`, `send_line()`, `send_blob(bytes)`, `flush()`. The ONLY type that writes to stdout. `diag(msg)` writes to stderr."
    - path: "crates/reposix-remote/src/fast_import.rs"
      provides: "`render_blob(&Issue) -> String` (delegates to `reposix_core::issue::frontmatter::render`), `emit_import_stream(W, &[Issue], parent_mark: Option<u64>) -> Result<()>`, and `parse_export_stream(R) -> Result<ExportBatch>` where `ExportBatch { commits: Vec<ParsedCommit>, blobs: HashMap<u64, Vec<u8>>, message: String }`."
    - path: "crates/reposix-remote/src/diff.rs"
      provides: "`PlannedAction { Create(Issue), Update { id, version, new: Issue }, Delete { id, version } }`. `plan(prior: &[Issue], new_tree: &BTreeMap<String, Vec<u8>>, msg: &str) -> Result<Vec<PlannedAction>, BulkDeleteError>` enforces the SG-02 cap unless `msg.contains(\"[allow-bulk-delete]\")`."
    - path: "crates/reposix-remote/src/client.rs"
      provides: "Thin async wrappers `list_issues(http, origin, project)`, `patch_issue(...)`, `post_issue(...)`, `delete_issue(...)` — each goes through `HttpClient::request_with_headers_and_body`. Bodies are JSON-serialized `Untainted<Issue>` payloads (limited keys via the same `EgressPayload` shape from S-A)."
    - path: "crates/reposix-remote/src/main.rs"
      provides: "REWRITE: dispatch loop matching `capabilities | option | list | import | export`. Owns the Tokio runtime + `HttpClient` + `Protocol`. argv = `<alias> <url>`. Parses url via `reposix_core::parse_remote_url`."
    - path: "crates/reposix-remote/tests/protocol.rs"
      provides: "Spawns the binary via `std::process::Command`, feeds `capabilities\\n` on stdin, asserts stdout matches the exact 4-line + blank capability advertisement. Also tests `option foo bar\\n` → `unsupported`."
    - path: "crates/reposix-remote/tests/export.rs"
      provides: "wiremock-backed: feeds a synthetic export stream representing `M 100644 :1 0001.md` (modified blob with `status: closed`) on stdin; asserts the wiremock saw exactly one PATCH /projects/demo/issues/1 with `If-Match: 1` and body containing `\\\"status\\\":\\\"closed\\\"`. (Skip if Task 3 not done — gate behind `#[ignore]`.)"
    - path: "crates/reposix-remote/tests/bulk_delete_cap.rs"
      provides: "Synthetic export deleting 6 issues → asserts exit code != 0, asserts wiremock saw zero DELETE requests, asserts stderr contains `refusing to push`. SECOND test: same fixture but commit message `[allow-bulk-delete] cleanup` → asserts wiremock saw exactly 6 DELETE requests."
  key_links:
    - from: "crates/reposix-remote/src/fast_import.rs :: render_blob"
      to: "reposix_core::issue::frontmatter::render"
      via: "direct delegate — single source of truth for blob rendering across FUSE read + helper import"
      pattern: "frontmatter::render"
    - from: "crates/reposix-remote/src/diff.rs :: plan"
      to: "SG-02 bulk-delete cap"
      via: "if delete_count > 5 && !msg.contains(\"[allow-bulk-delete]\") -> Err(BulkDeleteError { count })"
      pattern: "\\[allow-bulk-delete\\]|delete_count > 5|BulkDelete"
    - from: "crates/reposix-remote/src/client.rs :: patch_issue/post_issue"
      to: "reposix_core::sanitize"
      via: "Tainted::new(parsed).then(sanitize_with_server_meta).into_inner() before serialization"
      pattern: "sanitize\\("
    - from: "crates/reposix-remote/src/main.rs"
      to: "reposix_core::http::{client, HttpClient::request_with_headers_and_body}"
      via: "sole HTTP path; allowlist enforced per call"
      pattern: "request_with_headers_and_body"
    - from: "crates/reposix-remote/src/protocol.rs"
      to: "stderr-only diagnostics"
      via: "All `eprintln!`/`tracing` writes go to stderr; stdout writes only via `Protocol::send_*`"
      pattern: "io::stderr|eprintln!"
---

<objective>
Replace the capabilities-only stub at `crates/reposix-remote/src/main.rs` with
a working `git-remote-reposix` helper speaking the git remote helper protocol
on stdin/stdout, with the SG-02 bulk-delete cap and SG-03 sanitize-on-egress
both enforced.

Purpose: closes ROADMAP Phase S success criteria #2 (git push round-trip
with PATCH per changed issue) and #3 (contrived 6-delete commit refused with
SG-02 message) — the two assertions the Phase 4 demo recording needs to fire
on camera.

Output: `cargo install --path crates/reposix-remote` produces a
`git-remote-reposix` binary. Inside a git repo with
`git remote add origin reposix::http://127.0.0.1:7878/projects/demo`, a
`git push origin main` translates the diff into HTTP calls; a 6-delete push
is refused before any DELETE fires.

BUDGET: 60 min wall clock. Minimum viable = Tasks 1 + 2 (capabilities + import +
SG-02 cap on a stub-export path that just counts deletes from the fast-import
stream). Task 3 (full export → PATCH/POST/DELETE translation) is the stretch.
SKIP THIS PLAN ENTIRELY if S-A overran by >15 minutes.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/S-stretch-write-path-and-remote-helper/S-CONTEXT.md
@.planning/research/git-remote-helper.md
@CLAUDE.md
@crates/reposix-core/src/http.rs
@crates/reposix-core/src/taint.rs
@crates/reposix-core/src/remote.rs
@crates/reposix-core/src/issue.rs
@crates/reposix-core/src/path.rs
@crates/reposix-remote/src/main.rs
@crates/reposix-remote/Cargo.toml

<interfaces>
<!-- Contracts the executor consumes directly — no codebase spelunking needed. -->

From `reposix_core::remote`:
```rust
pub struct RemoteSpec { pub origin: String, pub project: ProjectSlug }
pub fn parse_remote_url(url: &str) -> Result<RemoteSpec>;
// Accepts both `reposix::http://...` and `http://...` forms.
// `RemoteSpec::project.as_str()` for slug; `origin` is e.g. "http://127.0.0.1:7878".
```

From `reposix_core::issue::frontmatter`:
```rust
pub fn render(issue: &Issue) -> Result<String>;  // deterministic blob bytes
pub fn parse(text: &str) -> Result<Issue>;       // reverses render
// CRITICAL: this crate MUST NOT introduce a second renderer/parser. Use these
// directly. The FUSE read path uses these too — same SHAs across FUSE and helper
// is the property that makes `git status` clean after a roundtrip.
```

From `reposix_core::http` (sealed newtype, SG-01):
```rust
pub fn client(opts: ClientOpts) -> Result<HttpClient>;
impl HttpClient {
    pub async fn request_with_headers_and_body<U, B>(
        &self, method: Method, url: U,
        headers: &[(&str, &str)],
        body: Option<B>,
    ) -> Result<reqwest::Response>;
}
// Use Method::GET (list), Method::POST (create), Method::PATCH (update),
// Method::DELETE (remove). Allowlist re-checked per call.
```

From `reposix_core::taint` (SG-03):
```rust
pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>;
// Same usage pattern as S-A. Tainted::new(parsed_issue).then(sanitize).into_inner()
// → JSON serialize → request body. The point: even if a malicious issue body
// somehow contained `version: 999999`, sanitize discards it.
```

Git remote helper protocol (verbatim from `.planning/research/git-remote-helper.md`):
```
> capabilities                       # git → us, on stdin
< import\n
< export\n
< refspec refs/heads/*:refs/reposix/*\n
< \n                                 # blank line terminates response

> option dry-run true                # zero-or-more option lines
< unsupported\n                      # always reply unsupported for v0.1

> list                               # git asks for refs
< ? refs/heads/main\n                # we report `?` (unknown SHA — fast-import will compute)
< @refs/heads/main HEAD\n            # symref
< \n

> import refs/heads/main             # git asks us to fetch
> \n                                 # blank line ends import batch
< feature done\n                     # MUST be first
< blob\n
< mark :1\n
< data 234\n
< <bytes>\n
< (...repeat per issue...)
< commit refs/reposix/origin/main\n
< mark :N\n
< committer reposix-helper <bot@reposix> 0 +0000\n
< data 25\n
< Sync from REST snapshot
< from refs/reposix/origin/main^0\n   # OR omit on first import
< M 100644 :1 0001.md\n
< (...repeat...)
< done\n

> export                             # git asks us to push
> \n
> (...fast-export stream from git, terminated by `done`...)
< ok refs/heads/main\n               # OR `error refs/heads/main <reason>`
< \n
```
For v0.1 we DO NOT advertise `*export-marks` / `*import-marks` (no marks file;
each pull recomputes the world). Per CONTEXT.md this is acceptable for the demo.

Sample target HTTP shapes (already shipped in Phase 2):
```
GET    /projects/{slug}/issues             -> [Issue, ...]
POST   /projects/{slug}/issues             -> 201 + Issue + Location
PATCH  /projects/{slug}/issues/{id}        -> 200 + Issue   (If-Match required)
                                           -> 409 + {error,current} on mismatch
DELETE /projects/{slug}/issues/{id}        -> 204
```
</interfaces>

</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 [MIN-VIABLE]: Crate skeleton — Protocol, capabilities, list, option, dispatch loop</name>
  <files>crates/reposix-remote/Cargo.toml, crates/reposix-remote/src/main.rs, crates/reposix-remote/src/protocol.rs, crates/reposix-remote/tests/protocol.rs</files>
  <behavior>
    - Cargo.toml: ensure `dashmap`, `wiremock` (dev), `assert_cmd` (dev) are present;
      keep existing deps. Add `clippy.toml` lint or per-file `#[deny(clippy::print_stdout)]`
      so accidental `println!` outside `Protocol` is a compile error.
    - `protocol.rs`: `pub struct Protocol<R, W> { reader: BufReader<R>, writer: BufWriter<W>,
      peeked: Option<String> }` with methods:
      - `new(r, w) -> Self`
      - `read_line(&mut self) -> io::Result<Option<String>>` — returns None at EOF; trims
        trailing `\n` but preserves internal whitespace.
      - `peek_line(&mut self) -> io::Result<Option<&str>>` — uses the `peeked` cache.
      - `expect_blank(&mut self) -> io::Result<()>` — read_line; error if not empty/EOF.
      - `send_line(&mut self, s: &str) -> io::Result<()>` — write `s + \n` to stdout buffer.
      - `send_blank(&mut self) -> io::Result<()>` — write `\n`.
      - `send_raw(&mut self, bytes: &[u8])` — for blob payloads in import.
      - `flush(&mut self) -> io::Result<()>`.
      - `diag(msg: &str)` — `eprintln!("{msg}")`. THE SOLE non-protocol output channel.
    - `main.rs`: REWRITE the existing 44-line stub.
      - `fn main() -> Result<()>`:
        - tracing-subscriber to stderr (already present — preserve).
        - argv parse: `git-remote-reposix <alias> <url>` (>=3 args; otherwise bail).
        - `let spec = reposix_core::parse_remote_url(&url)?;`
        - Build Tokio current-thread runtime (matches research §1).
        - Build `HttpClient` via `reposix_core::http::client(ClientOpts::default())?`.
        - `let mut proto = Protocol::new(io::stdin().lock(), io::stdout().lock());`
        - Loop: `read_line` → match first whitespace-split token:
          - `"capabilities"` → emit `import\n export\n refspec refs/heads/*:refs/reposix/*\n \n`.
          - `"option"` → reply `unsupported\n`. Never `error`.
          - `"list"` → block_on `list_issues(http, ...)`. Reply `? refs/heads/main\n
            @refs/heads/main HEAD\n \n` (the SHA can be `?` since we use fast-import).
          - `"import"` → drain the batch (collect refnames until blank), call
            `handle_import` (Task 2 fills it; for now stub returning `done\n` only —
            this is enough for Task 1 to pass).
          - `"export"` → call `handle_export` (Task 2/3 fills it; stub for now: read
            until `done`, reply `ok refs/heads/main\n \n`).
          - `""` → continue (blank between commands).
          - other → `proto.diag(&format!("unknown command: {tok}")); break;`
        - `proto.flush()` after every iteration.
    - Tests in `tests/protocol.rs` using `assert_cmd` (or plain `std::process::Command`):
      - `capabilities_advertises_import_export_refspec` — feed `capabilities\n` on stdin,
        assert stdout starts with `import\nexport\nrefspec refs/heads/*:refs/reposix/*\n\n`.
      - `option_replies_unsupported` — feed `option dry-run true\n`, assert stdout is
        `unsupported\n`.
      - `unknown_command_writes_to_stderr_not_stdout` — feed `floofle\n`, assert stdout
        is empty (or just blank), stderr contains `unknown command`.
      - `binary_accepts_alias_and_url_args` — invoke `git-remote-reposix origin reposix::http://127.0.0.1:7878/projects/demo`
        with empty stdin → exit 0.
  </behavior>
  <action>
    1. Read the current `crates/reposix-remote/src/main.rs` and `Cargo.toml`.
    2. Add dev-deps: `wiremock = "0.6"`, `assert_cmd = "2"`, `tempfile = "3"` if not
       already inherited from workspace. (Reuse workspace deps where possible.)
    3. Create `src/protocol.rs` with the `Protocol<R, W>` struct as described.
    4. REWRITE `src/main.rs` to the dispatch loop in `<behavior>`. For Task 1 the
       `import` and `export` arms can be stubs (`proto.send_line("done")?` for import;
       `proto.send_line("ok refs/heads/main")?; proto.send_blank()?` for export).
    5. Add `mod protocol;` to `main.rs` (binary crate, so it's `mod`, not `use`).
    6. Add `#![forbid(unsafe_code)]` at top of `main.rs` and `protocol.rs`.
    7. Add `#![deny(clippy::print_stdout, clippy::print_stderr)]` to `main.rs` AND
       `protocol.rs` — then sprinkle `#[allow(...)]` only on the two methods inside
       `Protocol` that legitimately write to stdout. This is the mechanical lock.
       (For `eprintln!` inside `diag`, use `#[allow(clippy::print_stderr)]` on the
       method.)
    8. Write the four tests in `tests/protocol.rs`.
    9. COMMIT: `feat(reposix-remote): protocol skeleton + capabilities/list/option`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-remote 2>&1 | tail -30 && cargo clippy -p reposix-remote --all-targets -- -D warnings 2>&1 | tail -10</automated>
  </verify>
  <done>
    All four protocol tests pass; clippy clean; `print_stdout` lint denies any
    accidental future `println!` outside the `Protocol` struct.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2 [MIN-VIABLE]: Import (fast-export emit) + bulk-delete cap on a synthetic export</name>
  <files>crates/reposix-remote/src/fast_import.rs, crates/reposix-remote/src/diff.rs, crates/reposix-remote/src/client.rs, crates/reposix-remote/src/main.rs, crates/reposix-remote/tests/bulk_delete_cap.rs</files>
  <behavior>
    - `client.rs`: `pub async fn list_issues(http, origin, project) -> Result<Vec<Issue>>`
      mirroring `crates/reposix-fuse/src/fetch.rs::fetch_issues` shape (5s timeout,
      `X-Reposix-Agent: git-remote-reposix-{pid}` header, JSON parse). Reuse the same
      pattern; do NOT depend on `reposix-fuse` (keep `reposix-remote` decoupled).
    - `fast_import.rs`:
      - `pub fn render_blob(issue: &Issue) -> Result<String>` — delegates to
        `reposix_core::issue::frontmatter::render(issue)`. Single-line wrapper
        documents intent; ensures any future renderer change ripples here.
      - `pub fn emit_import_stream<W: Write>(w: &mut W, issues: &[Issue], parent: Option<&str>) -> Result<()>`:
        - Sort `issues` by `id` ASC (deterministic).
        - For each issue: `writeln!(w, "blob")?; writeln!(w, "mark :{N}")?;`
          `let bytes = render_blob(issue)?; writeln!(w, "data {}", bytes.len())?;`
          `w.write_all(bytes.as_bytes())?; writeln!(w)?;`
          where `N` is a per-emit counter starting at 1.
        - After all blobs: `writeln!(w, "commit refs/reposix/origin/main")?;`
          `writeln!(w, "mark :{commit_mark}")?;`
          `writeln!(w, "committer reposix-helper <bot@reposix> 0 +0000")?;`
          a fixed `data 25\nSync from REST snapshot` block,
          `if let Some(p) = parent { writeln!(w, "from {p}")?; }`
          `for each issue: writeln!(w, "M 100644 :{N} {:04}.md", issue.id.0)?;`
          `writeln!(w)?;`
        - Final `writeln!(w, "done")?;`.
      - `pub struct ParsedExport { pub commit_message: String, pub blobs: HashMap<u64, Vec<u8>>, pub tree: BTreeMap<String, u64> }`
      - `pub fn parse_export_stream<R: BufRead>(r: &mut R) -> Result<ParsedExport>`:
        - Parse `blob` / `mark :N` / `data <len>` / `<bytes>` blocks → fill `blobs`.
        - Parse `commit refs/heads/main` / `mark :N` / `author` / `committer` / `data <len>`
          (capture as `commit_message`) / optional `from ...` / `M 100644 :N <path>`
          entries → fill `tree`.
        - Stop at literal `done` line. Tolerate `feature ...` lines (ignore).
        - This is a NARROW parser: only handles what `git fast-export` from a flat
          one-file-per-issue tree emits. Not a general fast-import parser.
    - `diff.rs`:
      - `pub enum PlannedAction { Create(Issue), Update { id: IssueId, prior_version: u64, new: Issue }, Delete { id: IssueId, prior_version: u64 } }`
      - `pub enum BulkDeleteError { Refused { count: usize } }` (for SG-02)
      - `pub fn plan(prior: &[Issue], parsed: &ParsedExport) -> Result<Vec<PlannedAction>, BulkDeleteError>`:
        - Build `prior_by_id: HashMap<IssueId, &Issue>` and `prior_by_path: BTreeMap<String, IssueId>` from `prior`.
        - Build `new_by_path: BTreeMap<String, &Vec<u8>>` resolving each tree entry's mark to bytes.
        - Walk both: path-in-new-only → `Create(parse blob bytes via frontmatter::parse)`;
          path-in-old-only → `Delete { id, prior_version }`;
          both → `parse new blob`; if parsed != prior issue (compare relevant fields):
          `Update { id, prior_version, new }`.
        - Count deletes. If `> 5` and `!parsed.commit_message.contains("[allow-bulk-delete]")`,
          return `Err(BulkDeleteError::Refused { count })`. Otherwise return `Ok(actions)`.
        - For Task 2 MIN-VIABLE we do NOT execute the actions yet (Task 3 wires the HTTP).
          The export handler in `main.rs` for Task 2 calls `plan(...)`, on error prints
          the SG-02 message and `error refs/heads/main bulk-delete`, exits non-zero
          (after the helper loop returns).
    - `main.rs::handle_export`:
      - Read until `done` via `parse_export_stream`.
      - `runtime.block_on(client::list_issues(http, &origin, &project))?` to get prior state.
      - `match diff::plan(&prior, &parsed)`:
        - `Ok(_actions)` → for Task 2: just `proto.send_line("ok refs/heads/main")?; proto.send_blank()?;`
          (Task 3 will execute the actions). LOG a `proto.diag(&format!("would apply {} actions", _actions.len()))`.
        - `Err(BulkDeleteError::Refused { count })` →
          `proto.diag(&format!("error: refusing to push (would delete {count} issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)"));`
          `proto.send_line("error refs/heads/main bulk-delete")?; proto.send_blank()?;`
          set an exit-code flag; the `main` function returns `anyhow::bail!("bulk delete refused")` AFTER the loop.
    - `main.rs::handle_import`:
      - For each ref in the batch (only `refs/heads/main` for v0.1):
        `runtime.block_on(client::list_issues(http, &origin, &project))?`
        emit via `fast_import::emit_import_stream` (parent = None for v0.1 — first
        import is orphan; subsequent imports re-orphan since we have no marks).
    - Tests:
      - `bulk_delete_cap.rs`:
        - `six_deletes_refuses_and_calls_no_delete` — wiremock with `GET /projects/demo/issues`
          returning 6 issues; feed an export stream that deletes all 6 (empty tree);
          assert exit code != 0, stderr contains `refusing to push`, wiremock saw
          ZERO DELETE requests.
        - `six_deletes_with_allow_tag_passes_plan` — same setup but commit message
          `[allow-bulk-delete] cleanup`. For Task 2 MIN-VIABLE: assert stdout has
          `ok refs/heads/main` (we don't execute the deletes yet). The Task 3 version
          of this test asserts wiremock saw 6 DELETEs.
        - `five_deletes_passes_plan` — boundary check: 5 deletes is allowed (cap is `>5`).
  </behavior>
  <action>
    1. Create `src/client.rs`, `src/fast_import.rs`, `src/diff.rs` per `<behavior>`.
    2. Wire `mod` declarations in `main.rs`.
    3. Implement the `import` and `export` arms in the dispatch loop. The export arm
       for Task 2 only enforces the cap; it does NOT yet call PATCH/POST/DELETE on
       success (Task 3).
    4. Write the three `bulk_delete_cap.rs` tests.
    5. Write a `tests/import.rs` smoke test: spin wiremock with 3 issues, drive the
       binary with `import refs/heads/main\n\n`, assert stdout starts with `feature done`
       and contains `M 100644 :1 0001.md`.
    6. `# Errors` doc on every Result-returning fn.
    7. COMMIT: `feat(reposix-remote): import emit + SG-02 bulk-delete cap on export`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-remote 2>&1 | tail -40 && cargo clippy -p reposix-remote --all-targets -- -D warnings 2>&1 | tail -10</automated>
  </verify>
  <done>
    `bulk_delete_cap::six_deletes_refuses_and_calls_no_delete`,
    `bulk_delete_cap::six_deletes_with_allow_tag_passes_plan`,
    `bulk_delete_cap::five_deletes_passes_plan`, and
    `import::*` smoke test all pass. `git-remote-reposix` binary builds. Clippy clean.
    **Decision point:** if elapsed time at end of this task is >T+45min OR the
    Phase S hard cut (06:00 PDT) is within 20 minutes, STOP and skip Task 3.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3 [STRETCH — skip if budget tight]: Wire export PATCH/POST/DELETE with sanitize-on-egress</name>
  <files>crates/reposix-remote/src/client.rs, crates/reposix-remote/src/main.rs, crates/reposix-remote/tests/export.rs</files>
  <behavior>
    - `client.rs`: add `patch_issue`, `post_issue`, `delete_issue` mirroring S-A's
      shape. PATCH attaches `If-Match: <prior_version>`. All three use 5s timeout.
      All bodies for PATCH/POST go through `Tainted::new(parsed_issue).then(|t|
      sanitize(t, server_meta_from_prior_or_placeholder)).into_inner()` → JSON
      serialize → request body. Use the same `EgressPayload` shape from S-A
      (`title, status, assignee, labels, body` only — `deny_unknown_fields` on the
      sim side enforces this).
    - `main.rs::handle_export`:
      - On `Ok(actions)` from `diff::plan`:
        - For each action in order (Creates, then Updates, then Deletes — so a
          rename-shaped delete-then-create still works):
          - `Create(issue)` → `runtime.block_on(client::post_issue(...))`. Track outcome.
          - `Update { id, prior_version, new }` → `runtime.block_on(client::patch_issue(http, ..., id, prior_version, new))`.
            On `Conflict { current }`: print to stderr, write `error refs/heads/main "remote diverged on issue {id}; run git pull"`, set failure flag.
          - `Delete { id, prior_version }` → `runtime.block_on(client::delete_issue(...))`.
        - If all succeed: `proto.send_line("ok refs/heads/main")?;`
        - If any failed: the per-failure `error refs/heads/main <reason>` already went
          out; just `proto.send_blank()?;` and bail at end of `main`.
    - Tests:
      - `export.rs`:
        - `single_modify_emits_patch_with_if_match` — wiremock GET returns 1 issue
          (version 1, status open). Synthetic export modifies it to status closed.
          Assert wiremock saw exactly 1 PATCH /projects/demo/issues/1 with
          `If-Match: 1` and body `{"status":"closed",...}` (the body MUST NOT
          contain `version`/`id`/`created_at`/`updated_at` keys → SG-03 proof).
        - `conflict_409_writes_error_and_exits_nonzero` — wiremock returns 409 on
          PATCH; assert exit != 0, stderr mentions `409`/`conflict`, stdout has
          `error refs/heads/main`.
        - `bulk_delete_with_allow_tag_actually_deletes` — wiremock GET returns 6
          issues. Synthetic export deletes all 6 with commit message
          `[allow-bulk-delete] cleanup`. Assert wiremock saw 6 DELETE requests
          and exit code 0.
  </behavior>
  <action>
    1. Implement `patch_issue`/`post_issue`/`delete_issue` in `src/client.rs`. For
       sanitize: build `ServerMetadata` from the prior `Issue` (or a placeholder for
       Create where no prior exists — the POST response will overwrite anyway).
    2. Wire the action-execution loop into `handle_export`.
    3. Write the three `tests/export.rs` tests.
    4. COMMIT: `feat(reposix-remote): export PATCH/POST/DELETE with sanitize-on-egress`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-remote 2>&1 | tail -30</automated>
  </verify>
  <done>
    All `export::*` tests pass. SG-03 proof test
    (`single_modify_emits_patch_with_if_match`) confirms egress payload omits
    server-controlled fields. Bulk-delete-with-allow-tag test confirms cap can be
    overridden.
  </done>
</task>

</tasks>

<verification>
- `cargo fmt --all --check` exits 0.
- `cargo clippy -p reposix-remote --all-targets -- -D warnings` exits 0.
- `cargo test -p reposix-remote` exits 0 for all non-ignored tests.
- `grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-remote/`
  returns nothing.
- `grep -RIn 'println!' crates/reposix-remote/src/main.rs crates/reposix-remote/src/protocol.rs`
  returns no hits OUTSIDE the two `Protocol` methods that legitimately write to
  stdout (which carry `#[allow(clippy::print_stdout)]` justifying themselves).
- `grep -q 'frontmatter::render' crates/reposix-remote/src/fast_import.rs` — confirms
  blob renderer is the shared one, not a re-implementation.
- `grep -q '\\[allow-bulk-delete\\]' crates/reposix-remote/src/diff.rs` — confirms
  SG-02 override tag is implemented.
- `grep -q 'sanitize(' crates/reposix-remote/src/client.rs` — confirms egress
  sanitization is wired (Task 3 only — skip this check if Task 3 was skipped).
- `cargo build --release -p reposix-remote && ls target/release/git-remote-reposix`
  produces the binary.
</verification>

<success_criteria>
**Minimum viable (Tasks 1+2 only):**
- Capabilities round-trip works (test passes).
- `import refs/heads/main` produces a valid fast-import stream with deterministic
  blob ordering and uses `frontmatter::render` (single source of truth).
- A synthetic 6-delete export is REFUSED with the SG-02 message; a 5-delete export
  is allowed; commit message containing `[allow-bulk-delete]` bypasses the cap.
- The Phase 4 demo can show `git push` failing on the cap (without Task 3 the cap
  fires before any HTTP would have been made anyway, so this is the load-bearing
  half of demo SC #3).

**Full plan (all three tasks):**
- Above plus the export path actually executes PATCH/POST/DELETE. The Phase 4 demo
  can show the central `sed && git commit && git push` round-trip (demo SC #2).
- SG-03 sanitize-on-egress is mechanically proven: a test writes `version: 999999`
  in YAML, asserts the captured PATCH body omits `version` entirely.

**Plan exit gate:**
```
cargo fmt --all --check && \
cargo clippy -p reposix-remote --all-targets -- -D warnings && \
cargo test -p reposix-remote && \
grep -q 'frontmatter::render' crates/reposix-remote/src/fast_import.rs && \
grep -q '\[allow-bulk-delete\]' crates/reposix-remote/src/diff.rs && \
test -x ./target/debug/git-remote-reposix
```
(If Task 3 was skipped, drop the `sanitize(` check from verification and the
`bulk_delete_with_allow_tag_actually_deletes` test from `<success_criteria>`.)
</success_criteria>

<output>
After completion, create
`.planning/phases/S-stretch-write-path-and-remote-helper/S-B-SUMMARY.md`
recording: tasks completed, elapsed wall time, any deviations, and whether
the Phase 4 demo can use the full `git push` flow (Task 3 done) or only the
SG-02-firing flow (Tasks 1+2 done).
</output>
