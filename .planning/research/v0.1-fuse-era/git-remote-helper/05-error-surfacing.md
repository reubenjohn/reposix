← [back to index](./index.md)

# 5. Surfacing API Errors Back to the Agent

The agent runs in a shell loop. It sees:
- **stdout** of `git push`: usually summarized refs.
- **stderr** of `git push`: progress, warnings, errors. **This is where humans look. This is where our errors must go.**

### 5.1 Two channels, two purposes

Per the protocol:
- **stdout** (helper → git) is *protocol*. Per-ref status: `ok refs/heads/main` or `error refs/heads/main <one-line-reason>`. Anything not protocol-valid here breaks git.
- **stderr** (helper → user, passed through by git) is *free-form*. Use this for the human-readable explanation.

### 5.2 Pattern

```rust
fn handle_push_error(refname: &str, err: &ApiError) {
    // Free-form, agent-readable: goes to terminal stderr.
    eprintln!("reposix: push of {} failed", refname);
    match err {
        ApiError::Conflict { remote_etag, local_etag, field } => {
            eprintln!("  HTTP 409 Conflict on field `{}`", field);
            eprintln!("  Remote ETag: {}, local last-known: {}", remote_etag, local_etag);
            eprintln!("  Hint: run `git pull` and resolve conflict markers, then push again.");
        }
        ApiError::WorkflowViolation { from, to, allowed } => {
            eprintln!("  HTTP 400: cannot transition `{}` → `{}`", from, to);
            eprintln!("  Allowed transitions: {}", allowed.join(", "));
        }
        ApiError::RateLimit { retry_after } => {
            eprintln!("  HTTP 429: rate limited; retry after {}s", retry_after.as_secs());
        }
        _ => eprintln!("  {}", err),
    }

    // Machine-readable, on-protocol: goes to git, then `git push` exits non-zero.
    println!("error {} {}", refname, err.short_summary().replace('\n', " "));
}
```

The agent's shell sees both:
```
$ git push reposix
To reposix::http://localhost:7777/projects/demo
 ! [remote rejected] main -> main (remote diverged; run 'git pull' first)
error: failed to push some refs to 'reposix::...'
reposix: push of refs/heads/main failed
  HTTP 409 Conflict on field `status`
  Remote ETag: W/"7", local last-known: W/"4"
  Hint: run `git pull` and resolve conflict markers, then push again.
```

LLM agents are extremely good at parsing this format because it's dense in their training data (every developer-facing CLI emits something like it).

### 5.3 Pitfall: never write non-protocol bytes to stdout

```rust
println!("DEBUG: about to PATCH issue PROJ-123");  // BUG! Breaks the protocol.
eprintln!("DEBUG: about to PATCH issue PROJ-123"); // Correct.
```
This is the #1 footgun for new helper authors. Wrap stdout in a `Mutex<BufWriter<Stdout>>` and only write through a `Protocol::send_line()` API. Make stderr the default for everything else.
