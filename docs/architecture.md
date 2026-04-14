# Architecture

## System view

```mermaid
flowchart TB
  subgraph User["User space"]
    A["LLM agent<br/>(Claude Code / shell)"]
    G["git<br/>(push / pull)"]
    FC["FUSE mount<br/>/tmp/reposix-mnt/"]
  end
  subgraph Reposix["reposix binaries"]
    RF["reposix-fuse<br/>FUSE daemon"]
    RR["git-remote-reposix<br/>helper"]
    RS["reposix-sim<br/>axum server"]
    RC["reposix-cli<br/>orchestrator"]
  end
  subgraph Kernel["Kernel"]
    K["VFS + /dev/fuse"]
  end
  subgraph Sqlite["SQLite WAL"]
    DB["issues table<br/>audit_events (append-only)"]
  end
  A -->|"cat / sed / grep / ls"| FC
  A -->|"git commit / push"| G
  FC <--> K
  K <--> RF
  G -->|"stdin/stdout protocol"| RR
  RF -->|"HTTP / allowlist"| RS
  RR -->|"HTTP / allowlist"| RS
  RC -->|spawn| RF
  RC -->|spawn| RS
  RS --> DB
  style A fill:#6a1b9a,stroke:#fff,color:#fff
  style FC fill:#00897b,stroke:#fff,color:#fff
  style RF fill:#00897b,stroke:#fff,color:#fff
  style RR fill:#00897b,stroke:#fff,color:#fff
  style RS fill:#00897b,stroke:#fff,color:#fff
  style RC fill:#00897b,stroke:#fff,color:#fff
  style DB fill:#ef6c00,stroke:#fff,color:#fff
```

Every HTTP arrow above is mediated by a single `reposix_core::http::HttpClient` — the only legal way to construct a `reqwest::Client` in this workspace. The clippy lint `disallowed-methods` fires at compile time if any other code tries to bypass it.

![architecture poster](https://raw.githubusercontent.com/reubenjohn/reposix/main/docs/social/assets/architecture.png){ .no-lightbox width="100%" }

## Crate topology

```mermaid
flowchart LR
  CORE["reposix-core<br/>types + contracts"]
  SIM["reposix-sim<br/>axum REST sim"]
  GH["reposix-github<br/>GitHub REST v3 adapter"]
  CONF["reposix-confluence<br/>Confluence REST v2 adapter"]
  FUSE["reposix-fuse<br/>FUSE daemon"]
  REM["reposix-remote<br/>git-remote-reposix"]
  CLI["reposix-cli<br/>orchestrator"]
  CORE --> SIM
  CORE --> GH
  CORE --> CONF
  CORE --> FUSE
  CORE --> REM
  CORE --> CLI
  FUSE -.spawns via.-> CLI
  SIM -.spawns via.-> CLI
  style CORE fill:#6a1b9a,stroke:#fff,color:#fff
  style GH fill:#00897b,stroke:#fff,color:#fff
  style CONF fill:#00897b,stroke:#fff,color:#fff
```

`reposix-github` and `reposix-confluence` are sibling `IssueBackend`
implementations. Both follow the same pattern — `HttpClient` for SG-01
allowlist enforcement, `Tainted<T>` ingress wrapping for SG-05, a shared
`rate_limit_gate: Arc<Mutex<Option<Instant>>>` for per-token throttling,
manual-redact `Debug` on credential structs. `reposix-confluence` follows
the same SG-01 allowlist / SG-05 tainted-ingress discipline as
`reposix-github`; adding a third backend is mechanical (see
[`docs/connectors/guide.md`](connectors/guide.md)).

`reposix-core` is the seam. Every other crate depends on it; it depends on nothing internal. Ships: `Issue`, `IssueId`, `IssueStatus`, `ProjectSlug`, `Project`, `RemoteSpec`, `Tainted<T>`, `Untainted<T>`, `HttpClient`, `validate_issue_filename`, `frontmatter::{render, parse}`, the audit-log schema fixture, the `sanitize` function.

## Read path: cat /mnt/reposix/issues/00000000001.md

```mermaid
sequenceDiagram
  autonumber
  participant A as Agent (shell)
  participant K as Kernel VFS
  participant F as reposix-fuse
  participant S as reposix-sim
  participant D as SQLite WAL
  A->>K: read("/mnt/reposix/issues/00000000001.md")
  K->>F: FUSE_READ(ino)
  Note over F: validate_issue_filename("00000000001.md") — SG-04
  F->>F: HttpClient::request_with_headers<br/>5s timeout (SG-07)
  F->>S: GET /projects/demo/issues/1<br/>X-Reposix-Agent: reposix-fuse-{pid}
  S->>D: INSERT audit_events (ts,agent,method,path,...)
  S->>D: SELECT * FROM issues WHERE id=1
  S-->>F: 200 {frontmatter, body}
  F->>F: frontmatter::render(&issue)
  F-->>K: bytes
  K-->>A: bytes
```

Key points:

- Filename validation happens at the FUSE boundary. `../../etc/passwd.md` is rejected with `EINVAL` before any HTTP call.
- The 5-second timeout means a dead backend cannot hang the kernel indefinitely; `ls` returns within 5s with `EIO`.
- The audit insert is an outermost axum middleware layer — every request is recorded, including rate-limited ones.
- `frontmatter::render` is the same function the `git-remote-reposix` helper uses for fast-import blobs, guaranteeing deterministic SHAs across read/push.

## Write path: sed -i 's/status: open/status: done/' /mnt/reposix/issues/00000000001.md

```mermaid
sequenceDiagram
  autonumber
  participant A as Agent
  participant K as Kernel
  participant F as reposix-fuse
  participant S as reposix-sim
  A->>K: open(issues/00000000001.md, O_WRONLY)
  K->>F: OPEN, GETATTR
  A->>K: write(...)
  K->>F: WRITE(ino, offset, bytes)
  Note over F: append to per-ino DashMap buffer
  F-->>K: bytes written (ack)
  A->>K: close(fd)
  K->>F: FLUSH, RELEASE
  Note over F: parse buffer as Issue<br/>Tainted::new(issue).then(sanitize)<br/>(strips id/version/created_at/updated_at — SG-03)
  F->>S: PATCH /projects/demo/issues/1<br/>If-Match: "<version>"
  alt 200 OK
    S-->>F: 200 { new version }
    F-->>K: release OK
  else 409 stale
    S-->>F: 409 { current version }
    F-->>K: EIO (agent resolves via git pull later)
  else timeout
    F-->>K: EIO
  end
```

## git push: the central value prop

```mermaid
sequenceDiagram
  autonumber
  participant A as Agent
  participant G as git
  participant H as git-remote-reposix
  participant S as reposix-sim
  A->>G: git commit -am "..."
  A->>G: git push origin main
  G->>H: (spawn) + capabilities
  H-->>G: import / export / refspec
  G->>H: export<br/>(fast-export stream of new tree)
  H->>S: GET /projects/demo/issues (fetch prior tree)
  H->>H: diff::plan(prior, new)
  Note over H: Count deletes > 5? → SG-02 BulkDeleteRefused
  alt plan has ≤5 deletes OR [allow-bulk-delete] tag
    H->>S: POST / PATCH / DELETE per delta<br/>(each body sanitized — SG-03)
    S-->>H: 200 per call
    H-->>G: ok refs/heads/main
  else plan has >5 deletes, no override
    H-->>G: error refs/heads/main bulk-delete
    Note over G: exit code 1, stderr shows message
  end
```

## Optimistic concurrency as git merge

This is the most load-bearing design choice in reposix — and it falls out of the architecture for free.

```mermaid
flowchart TB
  A1["Agent A edits 00000000001.md<br/>status: open → in_progress"]
  A2["Agent A: git commit + push"]
  A3["Server version 1 → 2"]
  B1["Agent B edits 00000000001.md<br/>status: open → done"]
  B2["Agent B: git commit + push"]
  B3["git-remote-reposix: PATCH If-Match: \"1\""]
  B4["sim returns 409<br/>current version: 2"]
  B5["git-remote-reposix: git pull"]
  B6["Local merge produces &lt;&lt;&lt;&lt;&lt;&lt;&lt; markers<br/>inside 00000000001.md"]
  B7["Agent B resolves via sed<br/>git commit + push"]
  A1 --> A2 --> A3
  B1 --> B2 --> B3 --> B4 --> B5 --> B6 --> B7
  style A1 fill:#6a1b9a,stroke:#fff,color:#fff
  style B1 fill:#6a1b9a,stroke:#fff,color:#fff
  style B6 fill:#ef6c00,stroke:#fff,color:#fff
```

The agent resolving the conflict never has to parse a JSON `409` error. It never has to hold two versions of the issue in context and synthesize a merge. It uses `sed` on a text file with unambiguous markers — a flow it has seen in every merge-conflict-resolution corpus it was trained on.

## The async bridge

FUSE callbacks are synchronous by kernel contract. `git-remote-reposix` speaks a synchronous stdin/stdout protocol. But all our HTTP is async (tokio + reqwest). The bridge is deliberately tiny:

```rust
pub struct ReposixFs {
    rt: Arc<tokio::runtime::Runtime>,
    http: Arc<HttpClient>,
    // ...
}

impl fuser::Filesystem for ReposixFs {
    fn read(&mut self, _req: &Request, ino: u64, /* ... */) {
        let bytes = self.rt.block_on(async {
            self.http.request_with_headers(
                Method::GET, &url, &[("X-Reposix-Agent", &agent_id)],
            ).await
        });
        // ... respond to kernel
    }
}
```

The FUSE thread is not a tokio worker, so `block_on` from inside the callback is deadlock-safe. The same pattern applies in `git-remote-reposix` for its dispatch loop.

## Security perimeter

```mermaid
flowchart LR
  subgraph Trust["Trusted"]
    CORE["reposix-core<br/>sealed HttpClient"]
    SIM["reposix-sim"]
    FUSE["reposix-fuse"]
    REM["reposix-remote"]
    CLI["reposix-cli"]
  end
  subgraph Tainted["Tainted"]
    BODY["Issue bodies<br/>(attacker-authored)"]
    TITLE["Issue titles"]
    COMMENT["Comments (v0.2)"]
  end
  subgraph Egress["Egress"]
    NET["Any HTTP call"]
    DISK["File writes outside mount"]
  end
  BODY -.->|Tainted&lt;T&gt;| CORE
  TITLE -.->|Tainted&lt;T&gt;| CORE
  CORE -->|"sanitize()<br/>strips id/version/created_at"| Egress
  CORE -->|"HttpClient<br/>REPOSIX_ALLOWED_ORIGINS"| NET
  style Tainted fill:#d32f2f,stroke:#fff,color:#fff
  style Egress fill:#ef6c00,stroke:#fff,color:#fff
  style Trust fill:#00897b,stroke:#fff,color:#fff
```

See the [security page](security.md) for the full guardrails table, threat model, and what's deferred to v0.2.
