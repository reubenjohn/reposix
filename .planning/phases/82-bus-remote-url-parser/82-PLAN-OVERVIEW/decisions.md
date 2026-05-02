[← index](./index.md)

# Decisions ratified at plan time

The three open questions surfaced by RESEARCH.md § "Open Questions"
are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Each decision
references the source artifact and the rationale.

## D-01 — Q-A: by-URL-match for the no-remote-configured check (RATIFIED)

**Decision:** the bus handler resolves the local mirror remote NAME
by **scanning all configured remotes' URLs** and matching against
`mirror_url`. NOT by requiring a NAME in the bus URL. NOT by
auto-mutating git config.

**Implementation:** STEP 0 of `bus_handler::handle_bus_export` runs
`git config --get-regexp '^remote\..+\.url$'`, splits each line on
whitespace into `(config_key, value)`, reverse-looks-up the remotes
whose value byte-equals `mirror_url`. If zero matches → emit the
verbatim Q3.5 hint (`"configure the mirror remote first: git remote
add <name> <mirror-url>"`) and exit non-zero BEFORE PRECHECK A. If
one match → use that remote's name. If multiple matches → pick the
**first alphabetically**, emit a stderr WARNING naming the chosen
remote per RESEARCH.md Pitfall 4, proceed.

**Why URL-match (not require-name-in-URL):** the user has already
NAMED the remote (via `git remote add`). Making them also encode
the name into the bus URL is friction — a single `mirror=` carries
the same information, and the URL is the canonical UX. The `+`-form
that initially encoded `<sot>+<mirror_name>+<mirror_url>` was
explicitly dropped per Q3.3.

**Why first-alphabetical-with-WARN (not error) on multi-match:**
multi-match is a legitimate (rare) case — a user with a fork +
upstream both pointing at the same URL. Erroring would be
unfriendly; picking deterministically + naming the choice in stderr
gives the user a way to disambiguate (rename or remove the duplicate
remote).

**Source:** RESEARCH.md § "Open Questions" Q-A; Pitfall 4;
`.planning/research/v0.13.0-dvcs/decisions.md` Q3.5 (no-auto-mutate
ratified).

## D-02 — Q-B: P82 emits clean "P83 not yet shipped" error after prechecks pass (RATIFIED)

**Decision:** after PRECHECK A and PRECHECK B both succeed, the bus
handler does NOT proceed to read stdin. It emits a CLEAN diagnostic
to stderr (`"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet
shipped — lands in P83"`) AND a protocol error to stdout (`"error
refs/heads/main bus-write-not-yet-shipped"`), sets
`state.push_failed = true`, and returns cleanly. NO stdin read. NO
SoT writes. NO mirror writes.

**Why this shape (and not: silently fall through to `handle_export`):**
P82 is dispatch-only by ROADMAP definition. Falling through to
`handle_export` would route the bus URL through the single-backend
write path (because `handle_export` has no concept of a mirror —
SoT-only), corrupting the "SoT-first, mirror-best-effort" contract
P83 will land. Emitting a clear deferred-shipped error preserves
the "PRECHECK-pass means BOTH prechecks succeeded" contract for
P82's tests AND signals to a user who tries `git push reposix main`
in v0.13.0-dev that the write path is forthcoming.

**Why not: silently emit `ok refs/heads/main`:** that would lie to
git, which would conclude the push succeeded — confusing the user
when their mirror + SoT both remain unchanged.

**P83's planner inherits a clean seam:** the `bus_handler.rs` body's
deferred-error stub is a single `return Ok(())` site. P83 replaces
it with the SoT-write + mirror-write fan-out + audit + ref-update
logic. P83's test surface re-uses P82's PRECHECK A + PRECHECK B
fixtures (file:// mirror, wiremock SoT) without modification — only
the post-precheck behavior changes.

**Source:** RESEARCH.md § "Open Questions" Q-B; ROADMAP P82 SC1
("Bus URL parser is dispatch-only; WRITE fan-out is P83").

## D-03 — Q-C: reject unknown query parameters (RATIFIED)

**Decision:** `bus_url::parse` rejects bus URLs containing query
keys other than `mirror=`. Only `mirror=` is recognized. Unknown
keys produce an error: *"unknown query parameter `<key>` in bus
URL; only `mirror=` is supported"*.

**Why reject (not silently ignore):** silent-ignore is a footgun. A
typo `?mirorr=git@github.com:org/repo.git` becomes a no-op (the
parser sees no `mirror=`, falls through to `Route::Single`, and the
push hits the single-backend path with NO mirror integration) —
silently violating the user's intent. Rejection forces typos to
surface immediately.

**Forward compatibility:** rejecting today is cheaper than reclaiming
namespace later. If v0.14.0 adds `?priority=`, `?retry=`, etc., the
parser opts those keys in explicitly. No legacy-key compatibility
debt accrues.

**Empty-query-string boundary case:** `reposix::sim::demo?` (no
key=value pairs) produces an error citing missing `mirror=`.
`reposix::sim::demo` (no `?` at all) is `Route::Single` and
unaffected.

**Mirror URL with embedded `?`** (RESEARCH.md Pitfall 7): the mirror
URL must be percent-encoded in that case. The `url` crate's
`form_urlencoded::parse` handles the encoding cleanly. Document
the requirement in `bus_url.rs`'s module-doc and in CLAUDE.md
§ Architecture.

**Source:** RESEARCH.md § "Open Questions" Q-C; Pitfall 7;
`.planning/research/v0.13.0-dvcs/decisions.md` Q3.3 (`?mirror=` as
sole syntax).

## D-04 — `agent-ux.json` is the catalog home (NOT a new `bus-remote.json`)

**Decision:** add the 6 new rows to the existing
`quality/catalogs/agent-ux.json` (joining `agent-ux/dark-factory-sim`,
`agent-ux/reposix-attach-against-vanilla-clone`,
`agent-ux/mirror-refs-write-on-success`, `agent-ux/mirror-refs-read-by-vanilla-clone`,
`agent-ux/mirror-refs-reject-message-cites-refs`,
`agent-ux/sync-reconcile-subcommand`). NOT a new `bus-remote.json`.

**Why:** dimension catalogs are routed to `quality/gates/<dim>/`
runner discovery — `agent-ux` is the existing dimension. Splitting
it into two catalog files would force the runner to discover both
via tag, adding indirection for no benefit. The existing
`agent-ux.json` already has 6 rows; adding 6 more keeps the
single-file shape readable. P81's `agent-ux/sync-reconcile-subcommand`
is the immediate-prior precedent.

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P80 and P81 catalog-home precedents.

## D-05 — Bus path uses the SAME `BackendConnector` pipeline as the single-backend path for the SoT side

**Decision:** when `Route::Bus { sot, mirror_url }` is dispatched,
the SoT side (the `sot: ParsedRemote`) is consumed by the existing
`backend_dispatch::instantiate(&sot)` to produce the same
`Arc<dyn BackendConnector>` the single-backend path uses. The
`mirror_url` is held alongside in a NEW `BusState { sot_state:
State, mirror_url: String }` shape (or, more minimally, two extra
fields on `State`; see implementation note below).

**Why one BackendConnector pipeline:** Q3.4 RATIFIED bus is
PUSH-only; bus's SoT reads happen during PRECHECK B (calls
`backend.list_changed_since`), and in P83 SoT writes happen via
`backend.create_record / update_record / delete_record_or_close`.
All three paths go through `BackendConnector` — there's no separate
"bus backend" trait. The mirror is plain git (shell-out to `git
ls-remote` / `git push`); it's NOT a `BackendConnector`.

**Implementation note (T04):** the simplest shape is to extend
`State` with `Option<String> mirror_url`. `Some(url)` means "this
is a bus invocation"; `None` means single-backend. Capability
branching reads `state.mirror_url.is_none()` to decide whether to
advertise `stateless-connect`. The bus_handler dispatch reads
`state.mirror_url.as_deref()` to extract the URL for STEP 0 +
PRECHECK A. This avoids a `BusState` type-state explosion with
minimal blast radius (one `Option` field).

**Source:** RESEARCH.md § "Architecture Patterns" Pattern 1
(`Route::Bus { sot, mirror_url }` carries the SoT as a
`ParsedRemote`); decisions.md Q3.4 RATIFIED PUSH-only.

## D-06 — `git ls-remote` shell-out (NOT gix-native) for PRECHECK A

**Decision:** PRECHECK A invokes `Command::new("git").args(["ls-remote",
"--", mirror_url, "refs/heads/main"])` via `std::process::Command`.
NOT gix's native `Repository::find_remote(...).connect(direction)`.

**Why shell-out:** the project's existing idiom is shell-out for
porcelain calls (`crates/reposix-cli/src/doctor.rs:446` for `git
--version`; lines 859/909/935/944 for `git rev-parse / for-each-ref
/ rev-list`). gix-native ls-remote requires ~50 lines of
refspec/connection-state management for what shell-out does in 5
lines. The helper runs in a context where `git push` already
invoked `git`, so `git` is on PATH (Assumption A1).

**Security: `--` separator + reject `-`-prefixed mirror URLs.**
RESEARCH.md § Security flagged argument-injection via `mirror_url`
(e.g., `--upload-pack=evil`). Mitigation: BEFORE the shell-out,
reject any `mirror_url` that starts with `-`; ALWAYS pass `--` as
a positional separator before the URL. The reject error: *"mirror
URL cannot start with `-`: <mirror_url>"*. Documented in
`bus_handler.rs`'s module-doc.

**Local SHA read:** `git rev-parse refs/remotes/<name>/main`
(another shell-out, same idiom). Handles packed-refs correctly;
raw fs reads of `.git/refs/remotes/<name>/main` would miss them.

**Source:** RESEARCH.md § "Don't Hand-Roll", Pattern 3, Pitfall 3;
`crates/reposix-cli/src/doctor.rs` (donor pattern).
