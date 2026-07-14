← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T04 — `bus_handler.rs` module + `main.rs` Route dispatch + capabilities branching + State extension

<read_first>
- `crates/reposix-remote/src/main.rs` (entire file ~700 lines —
  understand the existing dispatch loop, `State` field shape,
  `capabilities`/`list`/`export` arms, and `fail_push`).
  Specifically:
  - lines 24-32 (mod declarations)
  - lines 48-77 (`State` struct)
  - lines 103-140 (`real_main` URL parsing + State init)
  - lines 142-211 (dispatch loop + capabilities/list/export arms)
  - lines 246-258 (`fail_push` helper)
  - lines 305-549 (`handle_export` — sibling pattern; bus_handler
    reuses the diag/fail_push idiom but does NOT call
    parse_export_stream).
- `crates/reposix-remote/src/bus_url.rs` (post-T02; confirm `Route`
  enum + `parse` signature).
- `crates/reposix-remote/src/precheck.rs` (post-T03; confirm
  `SotDriftOutcome` + `precheck_sot_drift_any` signature).
- `crates/reposix-cli/src/doctor.rs:446-944` (DONOR pattern for
  `Command::new("git")` shell-outs — same idiom for ls-remote +
  config + rev-parse).
- `crates/reposix-cache/src/mirror_refs.rs:227`
  (`Cache::read_mirror_synced_at` — used by PRECHECK B's hint
  composition when populated).
- `crates/reposix-remote/src/protocol.rs` — `Protocol` API for
  `send_line`, `send_blank`, `flush`.
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md`
  § Pattern 1 (capability branching) + Pattern 3 (PRECHECK A
  shell-out) + Pitfall 4 (multi-match) + § Security (T-82-01).
</read_first>

<action>
Five concerns in this task; keep ordering: State extension → main.rs
URL dispatch → main.rs capabilities + export branching →
bus_handler.rs body → cargo check + commit.

### 4a. State extension — `crates/reposix-remote/src/main.rs:48`

Add ONE new field to the existing `State` struct:

```rust
pub(crate) struct State {
    pub(crate) rt: Runtime,
    pub(crate) backend: Arc<dyn BackendConnector>,
    backend_name: String,
    pub(crate) project: String,
    cache_project: String,
    push_failed: bool,
    #[allow(dead_code)]
    last_fetch_want_count: u32,
    pub(crate) cache: Option<Cache>,
    /// Bus-mode mirror URL (DVCS-BUS-URL-01). `Some(url)` when the
    /// helper was invoked with a `reposix::<sot>?mirror=<url>` URL
    /// per Q3.3; `None` for single-backend `reposix::<sot>` URLs.
    /// The capabilities arm gates `stateless-connect` on
    /// `mirror_url.is_none()` (DVCS-BUS-FETCH-01 / Q3.4); the export
    /// arm dispatches to `bus_handler::handle_bus_export` when
    /// `Some` and to `handle_export` when `None`.
    pub(crate) mirror_url: Option<String>,
}
```

Update the `State` initializer in `real_main` (line 127-136):

```rust
    let mut state = State {
        rt,
        backend,
        backend_name,
        project: project_for_backend,
        cache_project: project_for_cache,
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
        mirror_url: None,  // NEW: defaults to None; set to Some(url) below for Route::Bus
    };
```

### 4b. URL dispatch via `bus_url::parse` — `crates/reposix-remote/src/main.rs:103-126`

Replace the existing URL-parse block. Current:

```rust
    let url = &argv[2];
    let parsed = parse_dispatch_url(url).context("parse remote url")?;
    let backend = instantiate(&parsed).context("instantiate backend")?;
    let backend_name = parsed.kind.slug().to_owned();
    let project_for_cache = sanitize_project_for_cache(&parsed.project);
    let project_for_backend = parsed.project;
```

New shape:

```rust
    let url = &argv[2];
    let route = bus_url::parse(url).context("parse remote url")?;
    // For both Route::Single and Route::Bus, the SoT side is consumed
    // by the existing `instantiate` path. The mirror_url (if any) is
    // captured into `mirror_url_opt` and stamped onto `State` after
    // initialization (D-05 — single Option field, not a new BusState
    // type-state).
    let (parsed, mirror_url_opt): (ParsedRemote, Option<String>) = match route {
        bus_url::Route::Single(p) => (p, None),
        bus_url::Route::Bus { sot, mirror_url } => (sot, Some(mirror_url)),
    };
    let backend = instantiate(&parsed).context("instantiate backend")?;
    let backend_name = parsed.kind.slug().to_owned();
    let project_for_cache = sanitize_project_for_cache(&parsed.project);
    let project_for_backend = parsed.project;
```

Update the `State` initializer to consume `mirror_url_opt`:

```rust
    let mut state = State {
        rt,
        backend,
        backend_name,
        project: project_for_backend,
        cache_project: project_for_cache,
        push_failed: false,
        last_fetch_want_count: 0,
        cache: None,
        mirror_url: mirror_url_opt,  // CHANGED: from `None` to `mirror_url_opt`
    };
```

Add `ParsedRemote` to the existing `use crate::backend_dispatch::...`
import line if it isn't already imported.

### 4c. Capabilities branching — `crates/reposix-remote/src/main.rs:150-172`

Wrap the existing `proto.send_line("stateless-connect")?;` line (S1
/ Pitfall 6 / DVCS-BUS-FETCH-01). Current arm:

```rust
"capabilities" => {
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    proto.send_line("stateless-connect")?;
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

New shape (5-line edit — the wrapping `if`):

```rust
"capabilities" => {
    // DVCS-BUS-FETCH-01 / Q3.4 — bus URL is PUSH-only; fetch falls
    // through to the single-backend code path. We omit
    // `stateless-connect` for bus URLs; single-backend URLs continue
    // to advertise it. Per `transport-helper.c::process_connect_service`,
    // `stateless-connect` is dispatched only for git-upload-pack /
    // git-upload-archive — push (`git-receive-pack`) falls through
    // to `export` regardless.
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    if state.mirror_url.is_none() {
        proto.send_line("stateless-connect")?;
    }
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

### 4d. Export-verb branching — `crates/reposix-remote/src/main.rs:186-188`

Replace the existing single-line export arm:

```rust
"export" => {
    handle_export(&mut state, &mut proto)?;
}
```

with the two-branch dispatch:

```rust
"export" => {
    if state.mirror_url.is_some() {
        bus_handler::handle_bus_export(&mut state, &mut proto)?;
    } else {
        handle_export(&mut state, &mut proto)?;
    }
}
```

Add `mod bus_handler;` to the top-of-file mod declarations
(alphabetical placement — between `mod bus_url;` (added in T02) and
`mod diff;`):

```rust
mod backend_dispatch;
mod bus_handler;      // NEW
mod bus_url;
mod diff;
mod fast_import;
mod pktline;
mod precheck;
mod protocol;
mod stateless_connect;
```

Continue to [T04 step 2](./T04-step-2.md) for the full `bus_handler.rs`
module source (step 4e), HARD-BLOCKs, cargo check, and commit.
</action>
