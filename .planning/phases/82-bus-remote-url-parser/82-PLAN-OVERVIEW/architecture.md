[← index](./index.md)

# Subtle architectural points (read before T04)

The two below are flagged because they are the most likely sources
of T04 review friction. Executor must internalize them before
writing the wiring code.

## S1 — Capability branching is a 5-line edit, not a refactor

The current `"capabilities"` arm in
`crates/reposix-remote/src/main.rs:150-172` emits five capability
lines unconditionally. The bus URL flips ONE of them off
(`stateless-connect`); the other four (`import`, `export`, `refspec`,
`object-format=sha1`) stay. Per Q3.4 ratified bus is PUSH-only —
fetch on a bus URL falls through to the single-backend path,
so the bus URL never advertises `stateless-connect`.

**Why this matters for T04.** A reviewer skimming the wiring may
expect the bus path to need its own capabilities arm, or a separate
`fn handle_bus_capabilities`. That would be wrong. Capabilities
are advertised once per helper invocation BEFORE any verb dispatch
— the difference between bus and single-backend is one `if`. The
existing arm becomes:

```rust
"capabilities" => {
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

DVCS-BUS-FETCH-01 is closed by this 5-line diff. The integration
test (T05's `tests/bus_capabilities.rs`) sends `capabilities\n` to
the helper with a bus URL on argv[2], reads stdout, asserts the
list contains `import`, `export`, `refspec`, `object-format=sha1`
AND does NOT contain `stateless-connect`.

## S2 — Stdin must NOT be read before either precheck fires

The whole point of the cheap-precheck design is that PRECHECK A and
PRECHECK B run BEFORE `parse_export_stream` consumes stdin. If
stdin is read first, the precheck cost-savings claim collapses
(the user has paid to upload their fast-import stream over the
pipe). For typical issue-tracker push sizes (a few KB) this is
irrelevant; for larger artifacts (image attachments, etc.) it
matters loudly.

**Why this matters for T04.** The natural place to insert prechecks
is "right where the existing one runs", but the existing
`handle_export` precheck (P81's `precheck_export_against_changed_set`)
runs AFTER `parse_export_stream`. The bus path is a SIBLING of
`handle_export`, not a wrapper — `bus_handler::handle_bus_export`
runs prechecks FIRST, then (in P82) emits the deferred-shipped
error WITHOUT reading stdin, then (in P83) buffers stdin and
proceeds.

**Test contract:** T05's `bus_precheck_a.rs` and `bus_precheck_b.rs`
each assert a fixture-state property: NO `helper_push_started` audit
row OR NO `parse_export_stream` invocation observable. The cleanest
assertion is "the bus_handler returns BEFORE `BufReader::new(ProtoReader::new(proto))`
constructs" — verifiable via test-side instrumentation OR by
observing that wiremock saw zero PATCH/PUT calls AND zero
`Cache::log_helper_push_started` rows. Use the wiremock + cache-
audit shape (lower coupling than test-side instrumentation).
