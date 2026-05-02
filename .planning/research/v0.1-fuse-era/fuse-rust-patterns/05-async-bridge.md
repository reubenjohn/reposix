# Async bridge: calling reqwest from sync FUSE callbacks

## 5. Async bridge: calling reqwest from sync FUSE callbacks

**The core tension:** `Filesystem` methods are sync `&self` functions. Our backend (simulator, eventually Jira/GitHub) is HTTP, ideally async (reqwest). We need to resolve this without blocking the FUSE worker thread pool indefinitely and without a deadlock.

### 5.1 Why NOT use `fuser::experimental::AsyncFilesystem`

The `experimental` module exists and can be enabled with `features = ["experimental"]`. It provides `AsyncFilesystem` with `async fn` methods and a `TokioAdapter`. Tempting. But:

1. **It's experimental** — API churned between 0.15, 0.16, 0.17. `DirEntListBuilder`, `RequestContext`, `LookupResponse`, `GetAttrResponse` are new names every release. Pinning a version works, but upgrading is painful.
2. **It forces tokio into the public surface of our fuse crate.** Crate users (other reposix workspace members testing with a fake FS) pay for tokio even if they don't need it.
3. **Internally, `TokioAdapter` does exactly what we'd do manually:** wraps the filesystem in `Arc<RwLock<...>>` + pins a tokio runtime and runs each request on it. No magic win.

**Verdict:** implement the pattern ourselves in ~40 lines. It's more robust across fuser upgrades.

### 5.2 The pattern: dedicated multi-thread runtime owned by the FS

```rust
use tokio::runtime::{Builder, Runtime, Handle};
use tokio::sync::oneshot;

pub struct ReposixFs {
    inodes:   Arc<RwLock<DashMap<u64, Node>>>,
    next_ino: AtomicU64,
    uid: u32, gid: u32,

    /// Dedicated async runtime. Owned by the FS so it lives as long as we do.
    rt:       Arc<Runtime>,
    /// HTTP client — reqwest uses tokio internally, so it's tied to `rt`.
    http:     Arc<reqwest::Client>,
    backend:  reqwest::Url,
}

impl ReposixFs {
    pub fn new(backend: reqwest::Url) -> Self {
        let rt = Arc::new(
            Builder::new_multi_thread()
                .worker_threads(2)           // small — most calls are I/O bound
                .enable_all()
                .thread_name("reposix-async")
                .build()
                .expect("tokio runtime"),
        );
        let http = Arc::new(
            rt.block_on(async { reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap()
            }),
        );
        // ... seed root, etc.
        ReposixFs { /* ... */ rt, http, backend }
    }

    /// Bridge helper: run an async future to completion from sync code.
    fn block<F, T>(&self, fut: F) -> T
    where F: std::future::Future<Output = T> + Send + 'static,
          T: Send + 'static,
    {
        // CRITICAL: we are on a FUSE worker thread (not the tokio runtime).
        // So block_on is safe — it doesn't deadlock the runtime.
        self.rt.block_on(fut)
    }
}
```

**Why `block_on` is safe here:** FUSE spawns OS threads to serve kernel requests. Those threads are *not* tokio runtime worker threads. So when one of them calls `self.rt.block_on(fut)`, it's parking a non-runtime thread — the tokio worker threads are free to poll `fut` to completion. This is the canonical pattern from the [tokio bridging docs](https://tokio.rs/tokio/topics/bridging).

**Deadlock trap to avoid:** if you accidentally called `block_on` from *inside* an async context (e.g. from a callback driven by tokio itself), you'd block a worker thread that might be needed to poll your own future. Since FUSE callbacks are never inside tokio, we're safe — but guard with a newtype to prevent future regressions.

### 5.3 Using the bridge — example: fetch an issue lazily

```rust
impl Filesystem for ReposixFs {
    fn read(&self, _req: &Request, ino: INodeNo, _fh: FileHandle,
            offset: u64, size: u32, _flags: OpenFlags,
            _lock: Option<LockOwner>, reply: ReplyData) {
        // Fast path: content already cached.
        {
            let inodes = self.inodes.read();
            if let Some(n) = inodes.get(&ino.0) {
                if let NodeKind::File { bytes } = &n.kind {
                    if !bytes.is_empty() || n.marked_fetched {
                        let start = (offset as usize).min(bytes.len());
                        let end   = (start + size as usize).min(bytes.len());
                        return reply.data(&bytes[start..end]);
                    }
                }
            }
        }
        // Slow path: fetch from backend synchronously.
        let key = match self.inode_to_key(ino.0) {
            Some(k) => k,
            None => return reply.error(Errno::ENOENT),
        };
        let http = self.http.clone();
        let url  = self.backend.join(&format!("issues/{key}")).unwrap();

        let result = self.block(async move {
            http.get(url).send().await?.error_for_status()?.bytes().await
        });

        let bytes = match result {
            Ok(b) => b.to_vec(),
            Err(e) => {
                log::warn!("fetch {key} failed: {e}");
                return reply.error(Errno::EIO);
            }
        };

        // Cache in place + slice for reply.
        {
            let inodes = self.inodes.read();
            if let Some(mut n) = inodes.get_mut(&ino.0) {
                if let NodeKind::File { bytes: b } = &mut n.kind {
                    *b = bytes.clone();
                    n.mtime = SystemTime::now();
                }
            }
        }
        let start = (offset as usize).min(bytes.len());
        let end   = (start + size as usize).min(bytes.len());
        reply.data(&bytes[start..end]);
    }
}
```

### 5.4 Alternative: spawn + oneshot for fire-and-forget writes

For `write`, we usually want to buffer and ack the kernel quickly (writes shouldn't block on the network). Ack locally, push to the backend in the background:

```rust
fn write(&self, ...) {
    // ... update local bytes first, ack kernel immediately
    let bytes_written = data.len() as u32;
    let http = self.http.clone();
    let url  = self.backend.join(&format!("issues/{key}")).unwrap();
    let data_owned = new_content.clone();
    self.rt.spawn(async move {
        if let Err(e) = http.put(url).body(data_owned).send().await {
            log::warn!("background push failed: {e}");
            // TODO: surface via audit log + retry queue
        }
    });
    reply.written(bytes_written);
}
```

**This matches the `docs/research/initial-report.md` architecture** — writes do not go directly to the upstream. In the real design, writes go to local state and are pushed via `git-remote-reposix`. The spawned PUT above is only a hot-cache write-through; the durable write path is git.

### 5.5 When oneshot channels help

You rarely need them. The main use case is if you want timeout-with-fallback:

```rust
let (tx, rx) = tokio::sync::oneshot::channel();
self.rt.spawn(async move {
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        http.get(url).send()
    ).await;
    let _ = tx.send(result);
});
// Don't block_on — poll synchronously with a deadline:
match rx.blocking_recv() {
    Ok(Ok(Ok(resp))) => { /* got response */ }
    _ => { /* timeout or error — serve from cache or EIO */ }
}
```

But `rt.block_on(tokio::time::timeout(...))` is equivalent and simpler.
