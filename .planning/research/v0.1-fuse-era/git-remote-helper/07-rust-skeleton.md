← [back to index](./index.md)

# 7. Concrete Rust Skeleton

### 7.1 Crate layout (within the existing `reposix` workspace)

```
crates/reposix-remote/
├── Cargo.toml
└── src/
    ├── main.rs           # binary: git-remote-reposix
    ├── protocol.rs       # line protocol I/O (Protocol struct)
    ├── caps.rs           # capability advertisement
    ├── fast_import.rs    # parser + emitter for fast-import streams
    ├── state.rs          # marks file + state.json persistence
    ├── diff.rs           # tree-diff → REST-call planner
    ├── client.rs         # async HTTP client (reqwest + rustls-tls)
    └── error.rs          # ApiError, exit-code mapping
```

### 7.2 Cargo.toml (essentials)

```toml
[package]
name = "reposix-remote"
edition = "2021"

[[bin]]
name = "git-remote-reposix"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["rt", "macros", "net", "time"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
anyhow = "1"
thiserror = "1"
url = "2"
sha1 = "0.10"
hex = "0.4"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 7.3 main.rs — the dispatch loop

```rust
use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};

mod protocol;
mod caps;
mod fast_import;
mod state;
mod diff;
mod client;
mod error;

use protocol::Protocol;

fn main() -> Result<()> {
    // Diagnostics to stderr only. NEVER stdout.
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("REPOSIX_LOG")
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("usage: git-remote-reposix <alias> <url>");
    }
    let alias = args[1].clone();
    let url = args[2].clone();
    let git_dir = std::env::var("GIT_DIR")
        .context("GIT_DIR not set; this binary must be invoked by git")?;

    // One Tokio runtime for the lifetime of the process. All async calls
    // funnel through `runtime.block_on`. This is the same bridge pattern
    // FUSE uses (see crates/reposix-fuse/src/lib.rs).
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let creds = resolve_credentials(&alias)?;
    let client = client::Client::new(&url, creds)?;
    let mut state = state::State::open(&git_dir, &alias)?;
    let mut proto = Protocol::new(io::stdin().lock(), io::stdout().lock());

    loop {
        let line = match proto.read_line()? {
            Some(l) => l,
            None => break, // EOF — git is done with us
        };

        let cmd = line.split_whitespace().next().unwrap_or("");
        match cmd {
            "capabilities" => caps::handle(&mut proto, &state)?,
            "list" => {
                let for_push = line.contains("for-push");
                let refs = runtime.block_on(client.list_refs())?;
                proto.send_list(&refs)?;
            }
            "option" => {
                let resp = handle_option(&line, &mut state);
                proto.send_line(&resp)?;
            }
            "import" => {
                // Drain the batch (multiple consecutive `import <ref>` lines
                // terminated by blank line).
                let mut refs = vec![parse_ref(&line)];
                while let Some(more) = proto.peek_line()? {
                    if more.starts_with("import ") {
                        refs.push(parse_ref(&proto.read_line()?.unwrap()));
                    } else {
                        break;
                    }
                }
                // Consume the terminating blank line.
                proto.expect_blank()?;
                runtime.block_on(handle_import(&mut proto, &client, &mut state, &refs))?;
            }
            "export" => {
                proto.expect_blank()?;
                // Now stdin is a fast-import stream until `done`.
                runtime.block_on(handle_export(&mut proto, &client, &mut state))?;
            }
            "" => continue, // blank line between commands
            other => anyhow::bail!("unknown command: {}", other),
        }
        proto.flush()?;
    }

    state.persist()?;
    Ok(())
}

fn handle_option(line: &str, state: &mut state::State) -> String {
    let mut parts = line.splitn(3, ' ');
    let _ = parts.next(); // "option"
    let key = parts.next().unwrap_or("");
    let val = parts.next().unwrap_or("");
    match key {
        "verbosity" => { state.verbosity = val.parse().unwrap_or(1); "ok".into() }
        "dry-run"   => { state.dry_run = val == "true"; "ok".into() }
        "progress"  => "unsupported".into(),
        _           => "unsupported".into(),
    }
}

fn parse_ref(line: &str) -> String {
    // "import refs/heads/main" → "refs/heads/main"
    line.splitn(2, ' ').nth(1).unwrap_or("").to_string()
}

fn resolve_credentials(alias: &str) -> Result<client::Creds> {
    // Priority order:
    // 1. Env var REPOSIX_TOKEN_<ALIAS_UPPERCASE>
    // 2. Env var REPOSIX_TOKEN
    // 3. `git config --get remote.<alias>.reposixToken`
    // 4. `git config --get reposix.token`
    let alias_upper = alias.to_ascii_uppercase().replace('-', "_");
    if let Ok(t) = std::env::var(format!("REPOSIX_TOKEN_{}", alias_upper)) {
        return Ok(client::Creds::Bearer(t));
    }
    if let Ok(t) = std::env::var("REPOSIX_TOKEN") {
        return Ok(client::Creds::Bearer(t));
    }
    if let Some(t) = git_config(&format!("remote.{}.reposixToken", alias))? {
        return Ok(client::Creds::Bearer(t));
    }
    if let Some(t) = git_config("reposix.token")? {
        return Ok(client::Creds::Bearer(t));
    }
    Ok(client::Creds::None)
}

fn git_config(key: &str) -> Result<Option<String>> {
    let out = std::process::Command::new("git")
        .args(["config", "--get", key])
        .output()?;
    if out.status.success() {
        Ok(Some(String::from_utf8(out.stdout)?.trim().to_string()))
    } else {
        Ok(None)
    }
}
```

### 7.4 protocol.rs — disciplined I/O

```rust
use anyhow::Result;
use std::io::{BufRead, Write};

pub struct Protocol<R: BufRead, W: Write> {
    reader: R,
    writer: W,
    peeked: Option<String>,
}

impl<R: BufRead, W: Write> Protocol<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer, peeked: None }
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        if let Some(p) = self.peeked.take() {
            return Ok(Some(p));
        }
        let mut buf = String::new();
        let n = self.reader.read_line(&mut buf)?;
        if n == 0 { return Ok(None); }
        // Strip trailing \n only; preserve internal whitespace.
        if buf.ends_with('\n') { buf.pop(); }
        if buf.ends_with('\r') { buf.pop(); }
        Ok(Some(buf))
    }

    pub fn peek_line(&mut self) -> Result<Option<&str>> {
        if self.peeked.is_none() {
            self.peeked = self.read_line()?;
        }
        Ok(self.peeked.as_deref())
    }

    pub fn expect_blank(&mut self) -> Result<()> {
        match self.read_line()? {
            Some(s) if s.is_empty() => Ok(()),
            Some(other) => anyhow::bail!("expected blank, got: {:?}", other),
            None => Ok(()),
        }
    }

    pub fn send_line(&mut self, s: &str) -> Result<()> {
        // ENFORCE: no embedded \n in protocol lines.
        debug_assert!(!s.contains('\n'), "protocol line contains LF: {:?}", s);
        writeln!(self.writer, "{}", s)?;
        Ok(())
    }

    pub fn send_blank(&mut self) -> Result<()> {
        writeln!(self.writer)?;
        Ok(())
    }

    pub fn send_list(&mut self, refs: &[(String, String)]) -> Result<()> {
        for (sha, name) in refs {
            self.send_line(&format!("{} {}", sha, name))?;
        }
        self.send_blank()?;
        self.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Pass through stdin to a callback (for fast-import stream parsing).
    pub fn drain_until_done<F: FnMut(&str) -> Result<()>>(
        &mut self,
        mut cb: F,
    ) -> Result<()> {
        loop {
            let line = self.read_line()?
                .ok_or_else(|| anyhow::anyhow!("unexpected EOF in fast-import"))?;
            if line == "done" { return Ok(()); }
            cb(&line)?;
        }
    }
}
```

### 7.5 caps.rs — the static handshake

```rust
use anyhow::Result;
use crate::protocol::Protocol;
use crate::state::State;

pub fn handle<R, W>(proto: &mut Protocol<R, W>, state: &State) -> Result<()>
where R: std::io::BufRead, W: std::io::Write
{
    proto.send_line("import")?;
    proto.send_line("export")?;
    // Private namespace: keep our synthetic commits out of refs/remotes.
    proto.send_line("refspec refs/heads/*:refs/reposix/{alias}/*"
        .replace("{alias}", &state.alias))?;

    // Only advertise *import-marks if the file already exists; git treats
    // a missing mandatory marks file as a fatal error.
    let marks = state.marks_path();
    if marks.exists() {
        proto.send_line(&format!("*import-marks {}", marks.display()))?;
    }
    proto.send_line(&format!("*export-marks {}", marks.display()))?;

    proto.send_line("option")?;
    proto.send_blank()?;
    proto.flush()
}
```

### 7.6 The async-from-sync bridge

The whole protocol loop is sync — git speaks to us line-at-a-time over pipes and blocking reads are fine. But `reqwest` is async, and we want connection pooling, retries, and concurrent fan-out for batch operations.

**Pattern (also used by reposix-fuse):**

```rust
// At startup, build one current-thread runtime. This runs on the main thread;
// no extra threads are spawned unless we explicitly use `multi_thread`.
let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;

// Each command handler is sync. It calls block_on at the boundary.
fn handle_command(rt: &Runtime, client: &Client, ...) -> Result<()> {
    let result = rt.block_on(async {
        client.do_async_thing().await
    });
    result
}
```

For batch fan-out (e.g., applying 10 PATCH calls from a single push), use `futures::future::try_join_all` *inside* the `block_on`:

```rust
runtime.block_on(async {
    let futures: Vec<_> = changes.into_iter()
        .map(|c| client.apply_change(c))
        .collect();
    futures::future::try_join_all(futures).await
})?;
```

**Pitfalls:**
- Do not call `block_on` from within an async context — that deadlocks the current-thread runtime. Keep async/sync boundaries crisp.
- Do not spawn a multi-threaded runtime for one binary that processes one push: it adds threads, introduces nondeterminism in test logs, and offers nothing here.
- `reqwest::Client` is `Clone` and `Arc`-internal; create one per process and clone freely.
