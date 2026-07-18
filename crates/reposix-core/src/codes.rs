//! The `RPX-xxxx` error-code registry — the single source of truth for every
//! stable reposix error code AND its EXTENDED explanation (Phase 121 / P121).
//!
//! This is the codified, queryable half of the project's Rust-compiler-grade UX
//! north star. P120 shipped the terse teaching-error bar (`Fix:` / `Alternative:`
//! / `Recovery:` limbs at each call site); P121 adds the stable identifier and a
//! `rustc --explain`-style lookup so a dev who hits `RPX-0201` on stderr can run
//! `reposix explain RPX-0201` and read a compiler-grade extended explanation.
//!
//! # Two-tier by design (rustc-faithful)
//!
//! reposix's error UX has two deliberately different surfaces, exactly like
//! rustc's compact `E0308` message vs its longer `rustc --explain E0308`:
//!
//! - **The inline error** ([`errmsg::Teach`](crate::errmsg::Teach)) prints a
//!   terse body plus a `[RPX-xxxx]` tag and an `Explain: reposix explain RPX-xxxx`
//!   nudge. Its terse `Fix:`/`Alternative:`/`Recovery:` limbs come from the P120
//!   call-site args — NOT from this registry.
//! - **`reposix explain <code>`** reads the [`ExplainEntry`] here and prints the
//!   EXTENDED cause / fix / alternative / recovery.
//!
//! Only the CODE-ID is shared across the two surfaces. The extended prose lives
//! ONCE (here); the terse prose lives ONCE (at the call site). No explanation
//! text is duplicated, so there is no single-render coherence gate — the two
//! tiers are SUPPOSED to differ.
//!
//! # Threat model (OP-2 / T-121-01)
//!
//! Every field of every [`ExplainEntry`] is `&'static str` — baked into the
//! binary, never derived from a remote byte. `explain` reads only this static
//! [`REGISTRY`]; the inline `.code(...)` slot takes only a static code id. No
//! attacker-influenced byte can reach the code slot or the explain output.
//!
//! # Adding a code
//!
//! Add a `pub const` to [`ids`] and an [`ExplainEntry`] to [`REGISTRY`], then
//! reference the code at its call site via `.code(ids::NAME)` /
//! `teach_coded(ids::NAME, …)`. The `agent-ux/rpx-codes-registry` gate
//! (`quality/gates/agent-ux/rpx_registry_check.py`) enforces that every emitted
//! code is registered, every entry teaches a non-empty cause/fix/recovery, and
//! every code is a unique four-digit `RPX-\d{4}`.

/// A single registered error code and its extended explanation.
///
/// Every field is `&'static str` (recovery a `&'static [&'static str]`) — the
/// OP-2 cut: no remote byte can reach an entry. `alternative` MAY be empty for an
/// error with no genuine alternative approach (mirrors the P120 FLAG-1 rule);
/// `title` / `cause` / `fix` / `recovery` are always non-empty (gate-enforced).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplainEntry {
    /// The stable `RPX-xxxx` identifier (four digits, zero-padded).
    pub code: &'static str,
    /// A one-line summary of what went wrong (the explain header).
    pub title: &'static str,
    /// The extended cause — one or more prose lines teaching WHY this happens
    /// and the mental model behind it (the `rustc --explain` body).
    pub cause: &'static str,
    /// What the user should change to resolve it.
    pub fix: &'static str,
    /// A different command/approach that also reaches the user's goal, or `""`
    /// when the error has no genuine alternative.
    pub alternative: &'static str,
    /// One or more copy-paste-runnable recovery command lines.
    pub recovery: &'static [&'static str],
}

/// Named constants for every registered code, so call sites reference codes by
/// name (typo-proof) instead of bare string literals. The registry-integrity
/// gate resolves `ids::NAME` back to its four-digit value.
pub mod ids {
    /// `RPX-0001` — invalid `<backend>::<project>` spec (init/attach/sync/refresh).
    pub const SPEC_PARSE: &str = "RPX-0001";
    /// `RPX-0101` — a CLI real-backend credential/tenant env var is unset.
    pub const MISSING_ENV_CLI: &str = "RPX-0101";
    /// `RPX-0102` — the git remote helper is missing backend credentials.
    pub const MISSING_ENV_HELPER: &str = "RPX-0102";
    /// `RPX-0201` — reposix could not build its local cache from the backend.
    pub const CACHE_BUILD: &str = "RPX-0201";
    /// `RPX-0202` — no synced `cache.db` yet (tokens/cost/gc/history).
    pub const NO_SYNCED_CACHE: &str = "RPX-0202";
    /// `RPX-0203` — the directory is not a reposix working tree (no remote).
    pub const NOT_A_REPOSIX_TREE: &str = "RPX-0203";
    /// `RPX-0204` — the tree's stored `reposix::` remote URL could not be parsed.
    pub const BOUND_TREE_URL_PARSE: &str = "RPX-0204";
    /// `RPX-0301` — `reposix log` currently requires `--time-travel`.
    pub const LOG_NEEDS_TIME_TRAVEL: &str = "RPX-0301";
    /// `RPX-0302` — `reposix spaces` supports only the Confluence backend.
    pub const SPACES_CONFLUENCE_ONLY: &str = "RPX-0302";
    /// `RPX-0303` — `reposix refresh --offline` is not implemented yet.
    pub const REFRESH_OFFLINE_UNIMPL: &str = "RPX-0303";
    /// `RPX-0305` — invalid `--since` value (duration shortcut or RFC-3339).
    pub const SINCE_PARSE: &str = "RPX-0305";
    /// `RPX-0306` — reposix could not invoke `git` (missing / not on PATH).
    pub const GIT_NOT_ON_PATH: &str = "RPX-0306";
    /// `RPX-0307` — the `reposix init` target path is not valid UTF-8.
    pub const INIT_PATH_NOT_UTF8: &str = "RPX-0307";
    /// `RPX-0308` — `reposix init --since` has no populated cache to rewind into.
    pub const SINCE_NO_CACHE: &str = "RPX-0308";
    /// `RPX-0309` — no sync tag exists at-or-before the `--since` timestamp.
    pub const SINCE_NO_TAG: &str = "RPX-0309";
    /// `RPX-0310` — the `--since` sync tag resolved but its commit could not be fetched.
    pub const SINCE_FETCH_FAILED: &str = "RPX-0310";
    /// `RPX-0311` — `git update-ref` failed while rewinding to the `--since` snapshot.
    pub const SINCE_UPDATE_REF_FAILED: &str = "RPX-0311";
    /// `RPX-0401` — refusing to `reposix init` over an existing repository.
    pub const INIT_EXISTING_REPO_ROOT: &str = "RPX-0401";
    /// `RPX-0402` — init configured the tree but the initial fetch failed.
    pub const INIT_FETCH_FAILED: &str = "RPX-0402";
    /// `RPX-0403` — `reposix attach` needs an existing git working tree.
    pub const ATTACH_NOT_GIT_TREE: &str = "RPX-0403";
    /// `RPX-0404` — duplicate record `id` across local files (attach aborted).
    pub const ATTACH_DUPLICATE_IDS: &str = "RPX-0404";
    /// `RPX-0405` — the tree is already attached to a different backend.
    pub const ATTACH_MULTI_SOT: &str = "RPX-0405";
    /// `RPX-0501` — the cache could not serve `git upload-pack`.
    pub const HELPER_UPLOAD_PACK: &str = "RPX-0501";
    /// `RPX-0502` — unexpected EOF mid-request (protocol desync).
    pub const HELPER_EOF: &str = "RPX-0502";
    /// `RPX-0503` — refusing to materialize more blobs than the limit.
    pub const HELPER_BLOB_LIMIT: &str = "RPX-0503";
    /// `RPX-0504` — push rejected: backend unreachable during the pre-push check.
    pub const HELPER_BACKEND_UNREACHABLE: &str = "RPX-0504";
    /// `RPX-0505` — push rejected: the record changed on the backend (fetch first).
    pub const HELPER_PUSH_CONFLICT: &str = "RPX-0505";
    /// `RPX-0506` — the lazy cache cannot serve an UNFILTERED fetch (needs `--filter=blob:none`).
    pub const HELPER_UNFILTERED_FETCH: &str = "RPX-0506";
    /// `RPX-0601` — malformed reposix bus URL (fails to PARSE — a syntax fault).
    pub const HELPER_MALFORMED_BUS_URL: &str = "RPX-0601";
    /// `RPX-0602` — the git remote helper was invoked with too few arguments.
    pub const HELPER_USAGE: &str = "RPX-0602";
    /// `RPX-0603` — the DVCS bus mirror is unreachable/misconfigured (PRECHECK A
    /// `git ls-remote` failed). DISTINCT from RPX-0601: the URL parsed, the mirror
    /// it names does not answer.
    pub const HELPER_MIRROR_UNREACHABLE: &str = "RPX-0603";
    /// `RPX-0900` — the explain-meta code (`reposix explain <unknown>`).
    pub const EXPLAIN_UNKNOWN_CODE: &str = "RPX-0900";
}

/// The complete RPX registry — one [`ExplainEntry`] per distinct error scenario
/// across the CLI and the git remote helper, plus the explain-meta code.
pub const REGISTRY: &[ExplainEntry] = &[
    ExplainEntry {
        code: ids::SPEC_PARSE,
        title: "invalid backend spec",
        cause: "The `reposix` commands that bind a working tree to a backend — \
                `init`, `attach`, `sync`, `refresh` — take a spec of the form \
                `<backend>::<project>`. The spec you gave could not be parsed: \
                the `::` separator is missing, the project half is empty, or the \
                backend name is not one reposix knows. The four backends are \
                `sim` (the credential-free simulator), `github` \
                (`github::<owner>/<repo>`), `confluence` (`confluence::<space-key>`), \
                and `jira` (`jira::<project-key>`).",
        fix: "Write the spec as `<backend>::<project>` using one of the four known \
              backends, e.g. `sim::demo`, `github::octocat/hello-world`, \
              `confluence::ENG`, or `jira::PROJ`.",
        alternative: "Not sure a real backend is reachable yet? Start with the \
                      simulator — it needs no credentials and no network.",
        recovery: &[
            "reposix sim                              # start the simulator in another terminal",
            "reposix init sim::demo /tmp/reposix-demo # bootstrap a tree against it",
        ],
    },
    ExplainEntry {
        code: ids::MISSING_ENV_CLI,
        title: "a real-backend credential or tenant environment variable is unset",
        cause: "The Confluence and JIRA backends read your Atlassian Cloud \
                credentials and tenant from the environment, never from disk. One \
                or more required variable is unset, so reposix cannot \
                authenticate. Confluence needs ATLASSIAN_EMAIL, ATLASSIAN_API_KEY, \
                and REPOSIX_CONFLUENCE_TENANT; JIRA needs JIRA_EMAIL, \
                JIRA_API_TOKEN, and REPOSIX_JIRA_INSTANCE. The tenant/instance is \
                the `<x>` in `https://<x>.atlassian.net`.",
        fix: "`export` every variable the error listed, then re-run the same \
              command. Mint an API token at \
              id.atlassian.com/manage-profile/security/api-tokens.",
        alternative: "No Atlassian account handy? The simulator backend needs no \
                      credentials — target `sim::demo` instead.",
        recovery: &[
            "export ATLASSIAN_EMAIL=you@example.com",
            "export ATLASSIAN_API_KEY=<api-token>",
            "export REPOSIX_CONFLUENCE_TENANT=<subdomain>   # JIRA: JIRA_EMAIL / JIRA_API_TOKEN / REPOSIX_JIRA_INSTANCE",
            "# then re-run the same reposix command",
        ],
    },
    ExplainEntry {
        code: ids::MISSING_ENV_HELPER,
        title: "the git remote helper is missing backend credentials",
        cause: "`git-remote-reposix` — the helper git runs for a `reposix::` \
                remote on every fetch and push — reads the backend's credentials \
                from the environment git invokes it in. One or more required \
                variable is unset, so the helper cannot reach the system of \
                record. This is the same credential set the CLI uses, but \
                surfaced from inside a `git fetch`/`git push` rather than a \
                `reposix` subcommand.",
        fix: "`export` each listed variable in the shell you run `git` from (a git \
              credential helper does NOT supply these — they are reposix backend \
              env vars), then retry the git command; git re-invokes the helper \
              automatically.",
        alternative: "No real-backend credentials handy? Re-init against the \
                      simulator, which needs none — a `sim::` remote never reaches \
                      this code path.",
        recovery: &[
            "export <VAR>=<value>   # each variable the error named; matrix: docs/reference/testing-targets.md",
            "git push               # git re-runs the helper with the new environment",
        ],
    },
    ExplainEntry {
        code: ids::CACHE_BUILD,
        title: "reposix could not build its local cache from the backend",
        cause: "reposix serves a backend as a git partial clone by first \
                materializing a local bare-repo cache from the backend's REST \
                API. That materialization failed: reposix could not reach or read \
                the backend. The usual causes are a backend that is down or \
                unreachable, credentials that are unset or wrong, or an origin the \
                REPOSIX_ALLOWED_ORIGINS egress allowlist does not permit. The \
                underlying connector error is preserved on the `(underlying: …)` \
                line of the inline message — read it for the specific fault.",
        fix: "Confirm the backend is running and reachable and that its \
              credentials + allowlist are set, then re-run. `reposix doctor` \
              checks reachability and credentials for the backend.",
        alternative: "For a no-network smoke test, use the simulator: start it \
                      with `reposix sim`, then target `sim::<slug>`.",
        recovery: &[
            "reposix sim      # start the simulator, if you meant sim::…",
            "reposix doctor   # check backend reachability + credentials",
        ],
    },
    ExplainEntry {
        code: ids::NO_SYNCED_CACHE,
        title: "no synced reposix cache yet — nothing to read",
        cause: "`tokens`, `cost`, `gc`, and `history` read a per-tree cache and \
                its token/audit ledger that reposix builds from the backend on the \
                FIRST fetch. This working tree is valid, but that cache (its \
                `cache.db` ledger) has never been populated, so there is nothing \
                to read. This is expected on a freshly-`init`'d tree before any \
                `git fetch`.",
        fix: "Run one fetch from inside the working tree to materialize the cache \
              + audit ledger, then re-run the command. `reposix refresh` rebuilds \
              the whole tree + cache from the backend.",
        alternative: "Already synced in another checkout? Re-run the command from \
                      that working tree instead.",
        recovery: &[
            "git fetch         # from the working tree — materializes the cache + audit ledger",
            "reposix refresh   # or rebuild the whole tree + cache from the backend",
        ],
    },
    ExplainEntry {
        code: ids::NOT_A_REPOSIX_TREE,
        title: "this directory is not a reposix working tree",
        cause: "The subcommand resolves its cache through the working tree's \
                `reposix::` git remote — the binding that `reposix init` or \
                `reposix attach` writes. This directory has no such remote, so it \
                is bound to no backend and there is no cache to resolve. You are \
                probably running from the wrong directory, or the tree was never \
                bound.",
        fix: "`cd` into a reposix-bound tree, or create/adopt one: `reposix init` \
              bootstraps a fresh partial-clone tree, `reposix attach` binds an \
              existing checkout to a backend.",
        alternative: "Start with the credential-free simulator backend: \
                      `sim::demo`.",
        recovery: &[
            "reposix init <backend>::<project> <path>   # bootstrap a new tree",
            "reposix attach <backend>::<project>        # adopt an existing checkout",
        ],
    },
    ExplainEntry {
        code: ids::BOUND_TREE_URL_PARSE,
        title: "this tree's stored reposix remote URL could not be parsed",
        cause: "This working tree IS bound to a backend — it has a `reposix::` git \
                remote — but the URL stored in its `remote.<name>.url` is not a \
                valid `reposix::<backend>::<project>` URL (nor the \
                `?mirror=<url>` bus form). The binding was probably hand-edited or \
                written by an incompatible reposix version. This is DISTINCT from \
                RPX-0001, which parses a `<backend>::<project>` spec you typed on \
                the command line; here the unparseable URL already lives in the \
                tree's git config. Any embedded credential is redacted before the \
                URL is echoed.",
        fix: "Re-create the binding from scratch: `reposix init` for a fresh tree, \
              or `reposix attach` to rebind this existing checkout.",
        alternative: "Inspect the current remote first with `git remote -v` to see \
                      the malformed URL before you rebind.",
        recovery: &[
            "git remote -v                             # inspect the current remote URL",
            "reposix attach <backend>::<project> .     # rebind THIS checkout",
        ],
    },
    ExplainEntry {
        code: ids::LOG_NEEDS_TIME_TRAVEL,
        title: "`reposix log` currently requires the `--time-travel` flag",
        cause: "`reposix log` today lists the `refs/reposix/sync/<timestamp>` sync \
                history and requires `--time-travel` to do so. The bare \
                `reposix log` form is reserved for a future commit-graph view and \
                is refused rather than silently doing something different.",
        fix: "Pass `--time-travel` to list the sync history.",
        alternative: "`reposix history` shows the same sync-tag listing with no \
                      flag.",
        recovery: &["reposix log --time-travel", "reposix history"],
    },
    ExplainEntry {
        code: ids::SPACES_CONFLUENCE_ONLY,
        title: "`reposix spaces` supports only the Confluence backend",
        cause: "`reposix spaces` enumerates Confluence spaces — the wiki space \
                directories that are a Confluence-specific concept. The simulator, \
                GitHub, and JIRA backends have no notion of a space, so there is \
                nothing for `spaces` to list against them.",
        fix: "Run `reposix spaces` against Confluence, or list the other \
              backend's records by project with `reposix list --backend <backend> \
              --project <KEY>`.",
        alternative: "To browse the requested backend's issues instead, list them \
                      by project key rather than by space.",
        recovery: &[
            "reposix spaces --backend confluence                 # list your Confluence spaces",
            "reposix list --backend <backend> --project <KEY>    # per-project listing for any backend",
        ],
    },
    ExplainEntry {
        code: ids::REFRESH_OFFLINE_UNIMPL,
        title: "`reposix refresh --offline` is not implemented yet",
        cause: "`reposix refresh` always fetches a fresh snapshot from the backend \
                today — there is no offline read path. The working tree already \
                holds the last-fetched `.md` record files, so an offline read is \
                just reading those files directly; there is nothing for \
                `--offline` to add, and it is refused rather than silently \
                ignored.",
        fix: "Drop `--offline` and read the already-fetched records straight from \
              the working tree with `cat` / `grep` / `ls`.",
        alternative: "When you DO want a fresh backend snapshot, run \
                      `reposix refresh` without `--offline`.",
        recovery: &[
            "ls issues/                # already-fetched records (pages/ for confluence)",
            "grep -rl TODO issues/     # search the last snapshot offline",
            "reposix refresh <path>    # fetch a fresh backend snapshot when you want one",
        ],
    },
    ExplainEntry {
        code: ids::SINCE_PARSE,
        title: "invalid `--since` value",
        cause: "`--since` accepts either a duration shortcut — `7d`, `30d`, `1m`, \
                `1y`, `12h`, `30min` — or a full RFC-3339 timestamp such as \
                `2026-04-25T01:00:00Z`. The value you gave matched neither shape, \
                so reposix cannot compute the window start.",
        fix: "Pass a duration shortcut or a full RFC-3339 timestamp.",
        alternative: "Omit `--since` entirely to aggregate the whole ledger \
                      (all-time).",
        recovery: &[
            "reposix cost --since 7d",
            "reposix cost --since 2026-04-25T01:00:00Z",
        ],
    },
    ExplainEntry {
        code: ids::GIT_NOT_ON_PATH,
        title: "reposix could not invoke `git`",
        cause: "reposix drives git as a subprocess for the fetch/clone steps of \
                `init`. The `git` subprocess could not be spawned — usually git is \
                not installed or not on `PATH`. Partial-clone fetches also need \
                git 2.34+ for reliable `stateless-connect`.",
        fix: "Install git (2.34+ recommended) and make sure it is on `PATH`, then \
              re-run.",
        alternative: "",
        recovery: &[
            "git --version                              # confirm git is installed and on PATH",
            "reposix init <backend>::<project> <path>   # then retry",
        ],
    },
    ExplainEntry {
        code: ids::INIT_PATH_NOT_UTF8,
        title: "the `reposix init` target path is not valid UTF-8",
        cause: "`reposix init` drives `git` as a subprocess and passes it the \
                target `<path>`. git needs a UTF-8 path, and the path you gave \
                contains bytes that are not valid UTF-8 (an unusual locale \
                encoding, a stray byte from a shell glob, or a filename copied \
                from another system). reposix refuses fail-closed before touching \
                the filesystem rather than hand git a path it cannot use.",
        fix: "Point `init` at a plain UTF-8 path — rename the directory to ASCII \
              (safest) or a valid UTF-8 name, then re-run.",
        alternative: "",
        recovery: &[
            "reposix init <backend>::<project> /tmp/reposix-demo   # a plain-ASCII path",
        ],
    },
    ExplainEntry {
        code: ids::SINCE_NO_CACHE,
        title: "`reposix init --since` has no cache to rewind into",
        cause: "`--since` rewinds a fresh tree to a HISTORICAL sync tag recorded \
                in reposix's local cache for this backend. That cache has never \
                been populated, so there is no sync history to rewind through. \
                This is expected the very first time you touch a backend: the \
                history `--since` reads is built up by ordinary (non-`--since`) \
                syncs.",
        fix: "Run a normal `reposix init` (no `--since`) first to populate the \
              cache and record a sync tag, then re-run with `--since`.",
        alternative: "Only want the latest state, not a historical snapshot? Drop \
                      `--since` entirely.",
        recovery: &[
            "reposix init <backend>::<project> <path>            # populate the cache first",
            "reposix init <backend>::<project> <path> --since 7d # then rewind",
        ],
    },
    ExplainEntry {
        code: ids::SINCE_NO_TAG,
        title: "no sync tag at-or-before the `--since` timestamp",
        cause: "`--since` selects the NEWEST sync tag recorded at-or-before your \
                timestamp. The cache holds sync tags, but none is that early — \
                your timestamp (or the start of your duration window) predates the \
                first recorded sync for this backend. There is simply no snapshot \
                from that far back to rewind to.",
        fix: "Pass a later `--since` timestamp, or a shorter duration shortcut \
              (`7d` rather than `1y`). `reposix log --time-travel` lists the sync \
              tags that DO exist with their timestamps.",
        alternative: "Want the latest state? Omit `--since` and `reposix init` \
                      takes the most recent sync.",
        recovery: &[
            "reposix log --time-travel                            # list available sync tags + timestamps",
            "reposix init <backend>::<project> <path> --since 7d  # pick a window a tag falls inside",
        ],
    },
    ExplainEntry {
        code: ids::SINCE_FETCH_FAILED,
        title: "the `--since` sync tag resolved but its commit could not be fetched",
        cause: "`--since` matched a historical sync tag, but bringing that tag's \
                commit into the working tree from the local cache failed. The \
                cache is incomplete or was garbage-collected after the tag was \
                recorded, so the commit the tag names is no longer present. git's \
                own stderr is preserved on the inline message — read it for the \
                exact fault.",
        fix: "Repopulate the cache with a plain `reposix init` (no `--since`), \
              which re-fetches from the backend, then retry `--since`.",
        alternative: "Want the latest state instead of a historical snapshot? \
                      Re-run `reposix init` without `--since`.",
        recovery: &[
            "reposix init <backend>::<project> <path>             # (no --since) repopulates the cache",
            "reposix init <backend>::<project> <path> --since <ts># then retry the rewind",
        ],
    },
    ExplainEntry {
        code: ids::SINCE_UPDATE_REF_FAILED,
        title: "could not move the working tree's refs to the `--since` snapshot",
        cause: "The historical commit fetched successfully, but `git update-ref` \
                failed while pointing `main` / `origin/main` at it — so the tree \
                could not be rewound. The usual cause is a concurrent git process \
                holding a ref lock (another `git` running against the same tree). \
                git's own stderr is preserved on the inline message.",
        fix: "Make sure no other git process is operating on this tree (wait for \
              it to finish or stop it), then re-run.",
        alternative: "",
        recovery: &[
            "reposix init <backend>::<project> <path>             # (no --since) for the latest state, then retry",
        ],
    },
    ExplainEntry {
        code: ids::INIT_EXISTING_REPO_ROOT,
        title: "refusing to `reposix init` over an existing git repository",
        cause: "`reposix init` CREATES a fresh partial-clone working tree — it \
                runs `git init` and writes `core.bare` + `remote.origin`. The \
                target path is already a git repository root, and re-initializing \
                it would rewrite those settings and corrupt the existing tree (the \
                failure mode behind the 2026-07-12 shared-tree incident). reposix \
                refuses fail-closed rather than clobber your repo.",
        fix: "Point `init` at a FRESH, non-existent path — e.g. a new \
              subdirectory.",
        alternative: "To adopt an EXISTING checkout into a reposix backend instead \
                      of creating one, use `reposix attach`.",
        recovery: &[
            "reposix init <backend>::<project> <path>/reposix-clone   # a fresh subdir",
            "reposix attach <backend>::<project>                      # adopt this existing checkout",
        ],
    },
    ExplainEntry {
        code: ids::INIT_FETCH_FAILED,
        title: "`reposix init` configured the tree but the initial fetch failed",
        cause: "`reposix init` configured the partial-clone remote successfully, \
                but the first `git fetch` from the backend brought nothing back — \
                so the tree has no commits yet. The backend was almost certainly \
                unreachable or not running when the fetch ran (for the simulator, \
                it was not started).",
        fix: "Confirm the backend is running and reachable — for the simulator, \
              start it in another terminal with `reposix sim` — then re-run \
              `reposix init`, or sync in place with a filtered `git fetch`.",
        alternative: "The remote is already configured, so you can fetch in place \
                      instead of re-running init.",
        recovery: &[
            "reposix sim                                       # start the simulator, if you meant sim::…",
            "git -C <path> fetch --filter=blob:none origin     # sync the already-configured tree in place",
        ],
    },
    ExplainEntry {
        code: ids::ATTACH_NOT_GIT_TREE,
        title: "`reposix attach` needs an existing git working tree",
        cause: "`reposix attach` ADOPTS an existing checkout — it binds a \
                directory that is already a git repository to a reposix backend. \
                The directory you pointed it at has no `.git/`, so there is no \
                checkout to adopt.",
        fix: "`cd` into (or pass) a directory that is already a git repository.",
        alternative: "Starting from scratch with no checkout yet? Use \
                      `reposix init <backend>::<project> <path>` to bootstrap one \
                      instead.",
        recovery: &[
            "git init            # if this dir should become a repo",
            "git clone <url> .   # or clone your mirror first, then re-run reposix attach",
        ],
    },
    ExplainEntry {
        code: ids::ATTACH_DUPLICATE_IDS,
        title: "duplicate record `id` across local files — attach aborted",
        cause: "During `reposix attach`, reconciliation matches your local record \
                files to backend records by their frontmatter `id:`. Two or more \
                local files claim the SAME `id`, so reposix cannot decide which \
                one maps to the backend record. Reconciliation aborts before \
                committing any rows — your tree is unchanged.",
        fix: "Give each record a UNIQUE `id:` in its frontmatter — edit or remove \
              the duplicate — then re-run `reposix attach`.",
        alternative: "Meant to keep both as NEW records rather than matching them \
                      to existing backend records? Re-run with `--orphan-policy \
                      fork-as-new`.",
        recovery: &[
            "grep -rn '^id:' <the-duplicate-files>   # find the clashing ids",
            "reposix attach <backend>::<project>     # re-run once the ids are unique",
        ],
    },
    ExplainEntry {
        code: ids::ATTACH_MULTI_SOT,
        title: "this tree is already attached to a different system of record",
        cause: "reposix binds one working tree to exactly ONE system of record. \
                This tree is already bound to a different backend than the one you \
                are attaching, and silently re-pointing it would orphan the \
                existing binding. reposix refuses until you unbind explicitly.",
        fix: "Remove the current reposix remote, then re-attach. If the tree was \
              `reposix init`-bootstrapped, also unset `extensions.partialClone` \
              and delete the cache dir.",
        alternative: "Want a second system of record? Attach it in a SEPARATE \
                      checkout instead of re-pointing this one.",
        recovery: &[
            "git remote remove <reposix-remote-name>",
            "reposix attach <backend>::<project>",
            "# if init-bootstrapped, also: git config --unset extensions.partialClone (then delete the cache dir)",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_UPLOAD_PACK,
        title: "the reposix cache could not serve `git upload-pack`",
        cause: "On a fetch, `git-remote-reposix` tunnels git's protocol-v2 request \
                to a `git upload-pack` process running against the cache's bare \
                repo. That subprocess exited non-zero. The usual causes are an \
                incompatible git (partial-clone reads need git 2.34+) or a \
                missing/corrupt cache. git's own stderr is preserved in the inline \
                headline — read it for the specific fault.",
        fix: "Verify git is 2.34+ and the cache is healthy, then retry the fetch. \
              `reposix doctor` checks both.",
        alternative: "",
        recovery: &[
            "reposix doctor    # verify git 2.34+ and cache health",
            "git fetch origin  # retry once doctor is clean",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_EOF,
        title: "unexpected EOF mid-request (protocol desync)",
        cause: "The git client closed the connection partway through a pkt-line \
                request, so `git-remote-reposix` read an unexpected end-of-file \
                mid-request. This is a protocol desync, not a data error — the \
                usual trigger is a killed or backgrounded git process, or a broken \
                pipe.",
        fix: "Re-run the git operation from a clean state on a fresh connection.",
        alternative: "",
        recovery: &["git fetch origin   # re-drive the fetch on a fresh connection"],
    },
    ExplainEntry {
        code: ids::HELPER_BLOB_LIMIT,
        title: "refusing to materialize more blobs than the configured limit",
        cause: "A `command=fetch` asked `git-remote-reposix` to materialize more \
                blobs than REPOSIX_BLOB_LIMIT allows (default 200). The limit is a \
                guardrail against a single fetch pulling the entire backend into \
                the cache. It usually means the fetch is unscoped — it is walking \
                far more of the tree than you intend.",
        fix: "Narrow the fetch scope with `git sparse-checkout set <pathspec>` and \
              retry, or raise the ceiling with REPOSIX_BLOB_LIMIT if you really \
              need a wider fetch.",
        alternative: "Set REPOSIX_BLOB_LIMIT=0 to disable the guardrail entirely \
                      (only for a deliberate full materialization).",
        recovery: &[
            "git sparse-checkout set <pathspec>   # scope the fetch, then retry",
            "REPOSIX_BLOB_LIMIT=500 git fetch     # or raise the ceiling for this fetch",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_BACKEND_UNREACHABLE,
        title: "push rejected — the backend was unreachable during the pre-push check",
        cause: "Before writing your push through to the system of record, \
                `git-remote-reposix` runs an L1 precheck that reads current \
                backend state to detect conflicts. That precheck could not reach \
                the backend, so the push is rejected fail-closed rather than \
                written blind. git prints the protocol-standard `backend-unreachable` \
                status; the accompanying diag line carries this code. Your local \
                commits are intact.",
        fix: "Confirm the backend is running and reachable (and credentials + \
              allowlist are set), then re-drive the push.",
        alternative: "For the simulator, make sure `reposix sim` is running in \
                      another terminal before pushing.",
        recovery: &[
            "reposix doctor   # check backend reachability + credentials",
            "git push         # re-drive once the backend is reachable",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_PUSH_CONFLICT,
        title: "push rejected — the record changed on the backend since your last fetch",
        cause: "A record you are pushing was modified on the backend after your \
                last fetch, so your push would overwrite a newer version. \
                `git-remote-reposix` rejects with git's standard `fetch first` \
                status to protect the remote change; the accompanying diag names \
                the conflicting record, both versions, and a mirror-lag hint. This \
                is ordinary distributed-VCS drift, not a reposix fault.",
        fix: "Fetch the newer backend state, rebase your change on top, then push \
              again.",
        alternative: "Run `reposix sync` to update the local cache from the \
                      backend directly, then `git rebase`.",
        recovery: &[
            "git pull --rebase   # bring in the newer backend state, replay your change",
            "git push            # re-drive the push once rebased",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_UNFILTERED_FETCH,
        title: "the reposix cache cannot serve an unfiltered fetch",
        cause: "The reposix cache materializes blobs LAZILY — it holds only the \
                blob objects that were explicitly fetched, never the full backend. \
                An UNFILTERED `git fetch` asked `upload-pack` to pack the entire \
                reachable blob closure, which the cache does not hold, so \
                upload-pack died (often with a misleading `possible repository \
                corruption` message). This is DISTINCT from RPX-0503, which \
                refuses a FILTERED fetch that would exceed the blob LIMIT — here \
                the fetch carries no `--filter` at all, which the lazy cache can \
                never satisfy.",
        fix: "Re-run the fetch with `--filter=blob:none`. `reposix init` sets this \
              on the remote automatically; a hand-run `git fetch` must pass it \
              explicitly.",
        alternative: "",
        recovery: &[
            "git fetch --filter=blob:none origin        # the partial-clone fetch the cache serves",
            "reposix init <backend>::<project> <path>   # bootstraps a tree with the filter preset",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_MALFORMED_BUS_URL,
        title: "malformed reposix bus URL",
        cause: "A `reposix::` remote URL is either `reposix::<sot-spec>` (single \
                backend) or `reposix::<sot-spec>?mirror=<mirror-url>` (with a DVCS \
                mirror fan-out). The URL git handed the helper parses as neither — \
                common causes are the dropped `+`-delimited form, a query string \
                with no `mirror=` parameter, or an unescaped `?` inside the mirror \
                URL. The offending URL is echoed with any embedded credentials \
                redacted.",
        fix: "Rewrite the remote as `reposix::<sot-spec>` or \
              `reposix::<sot-spec>?mirror=<mirror-url>`; percent-encode any literal \
              `?` inside the mirror URL. Only `mirror=` is supported.",
        alternative: "For a single-backend remote with no mirror fan-out, drop the \
                      whole `?mirror=…` query.",
        recovery: &[
            "git remote set-url <name> 'reposix::http://127.0.0.1:7878/projects/demo?mirror=file:///tmp/mirror.git'",
            "# never embed credentials in the mirror URL — use a git credential helper or ssh keys",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_USAGE,
        title: "`git-remote-reposix` was invoked with too few arguments",
        cause: "`git-remote-reposix` is a git REMOTE HELPER — git runs it \
                automatically, passing a remote `<alias>` and `<url>`, whenever it \
                drives a `reposix::` remote. It was invoked with fewer than those \
                two arguments, which almost always means it was run by hand. You \
                normally never call it directly.",
        fix: "Don't invoke the helper by hand — use git against a reposix remote \
              (`git fetch` / `git push`) and git will run the helper for you.",
        alternative: "To create a reposix remote git can drive, run \
                      `reposix init <backend>::<project> <path>`.",
        recovery: &[
            "reposix init sim::demo /tmp/demo   # creates a reposix:: remote git can drive",
            "git -C /tmp/demo fetch origin",
        ],
    },
    ExplainEntry {
        code: ids::HELPER_MIRROR_UNREACHABLE,
        title: "the reposix bus mirror is unreachable or misconfigured",
        cause: "A `reposix::<sot-spec>?mirror=<mirror-url>` bus remote fans each \
                push out to a DVCS mirror after the system-of-record write. Before \
                pushing, `git-remote-reposix` runs PRECHECK A — a `git ls-remote` \
                against the mirror URL to detect drift — and that `git ls-remote` \
                failed: the mirror host could not be reached, the URL is wrong, or \
                the mirror repository does not exist. This is DISTINCT from RPX-0601 \
                (a MALFORMED bus URL that fails to PARSE); here the URL parsed fine \
                but the mirror it names does not answer. Any credentials embedded in \
                the mirror URL are redacted before it is echoed.",
        fix: "Confirm the mirror URL is correct and its host is reachable, then \
              re-drive the push. `git ls-remote <mirror-remote-name>` reproduces the \
              exact PRECHECK A probe against the configured remote (creds-free and \
              runnable — never the redacted URL).",
        alternative: "Push to the system of record alone by dropping `?mirror=<url>` \
                      from the remote URL — the mirror fan-out is best-effort, so \
                      the SoT write still lands without it.",
        recovery: &[
            "git ls-remote <mirror-remote-name>                    # reproduce the PRECHECK A probe",
            "git remote set-url origin 'reposix::<sot-spec>'       # drop the mirror fan-out",
            "# mirror recovery playbook: docs/guides/dvcs-mirror-setup.md",
        ],
    },
    ExplainEntry {
        code: ids::EXPLAIN_UNKNOWN_CODE,
        title: "no such reposix error code",
        cause: "`reposix explain` looks up an `RPX-xxxx` error code in the \
                built-in registry and prints its extended explanation. The code \
                you asked about is not registered — it may be mistyped, from a \
                newer reposix version, or not an RPX code at all. Codes are always \
                four digits, e.g. `RPX-0201`.",
        fix: "Check the code spelling, then look it up. `reposix explain --list` \
              prints every code reposix knows about with its title.",
        alternative: "Browse the full code index in the docs at \
                      docs/reference/error-codes.md.",
        recovery: &[
            "reposix explain --list     # every defined RPX code + title",
            "reposix explain RPX-0201   # look up a specific code",
        ],
    },
];

/// Look up an `RPX-xxxx` code in the [`REGISTRY`].
///
/// Returns the matching [`ExplainEntry`] or `None` for an unregistered code
/// (the caller — `reposix explain` — teaches the unknown-code path via
/// [`ids::EXPLAIN_UNKNOWN_CODE`]). Linear scan; the registry is small, so O(n) is
/// intentional.
#[must_use]
pub fn explain(code: &str) -> Option<&'static ExplainEntry> {
    REGISTRY.iter().find(|entry| entry.code == code)
}

#[cfg(test)]
mod tests {
    use super::{explain, ids, ExplainEntry, REGISTRY};

    /// A code is exactly `RPX-` followed by four ASCII digits.
    fn is_rpx_code(code: &str) -> bool {
        code.len() == 8 && code.starts_with("RPX-") && code[4..].chars().all(|c| c.is_ascii_digit())
    }

    #[test]
    fn every_code_is_wellformed_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for entry in REGISTRY {
            assert!(is_rpx_code(entry.code), "malformed code: {}", entry.code);
            assert!(
                seen.insert(entry.code),
                "duplicate code in REGISTRY: {}",
                entry.code
            );
        }
    }

    #[test]
    fn every_entry_teaches_nonempty_cause_fix_recovery() {
        // The codified north-star bar: NO terse one-liner entries. Every code must
        // teach a non-empty title + cause + fix + copy-paste recovery (alternative
        // MAY be empty for an error with no genuine alternative — FLAG-1 parity).
        for entry in REGISTRY {
            assert!(!entry.title.is_empty(), "{}: empty title", entry.code);
            assert!(!entry.cause.is_empty(), "{}: empty cause", entry.code);
            assert!(!entry.fix.is_empty(), "{}: empty fix", entry.code);
            assert!(
                !entry.recovery.is_empty(),
                "{}: empty recovery (no copy-paste command)",
                entry.code
            );
            for line in entry.recovery {
                assert!(!line.is_empty(), "{}: blank recovery line", entry.code);
            }
        }
    }

    #[test]
    fn explain_hits_registered_and_misses_unknown() {
        let hit = explain(ids::CACHE_BUILD).expect("RPX-0201 is registered");
        assert_eq!(hit.code, "RPX-0201");
        assert!(!hit.cause.is_empty() && !hit.fix.is_empty() && !hit.recovery.is_empty());
        assert!(explain("RPX-9999").is_none(), "unknown code must miss");
        assert!(explain("not-a-code").is_none());
    }

    #[test]
    fn explain_meta_code_exists_for_unknown_code_path() {
        // Leg 4 of the registry-integrity gate: `reposix explain <unknown>` needs a
        // teaching home. RPX-0900 must resolve.
        let meta = explain(ids::EXPLAIN_UNKNOWN_CODE).expect("RPX-0900 must exist");
        assert_eq!(meta.code, "RPX-0900");
    }

    #[test]
    fn every_ids_const_resolves_to_a_registered_entry() {
        // Guards against an `ids` const with no matching ExplainEntry (a code that
        // a call site could reference but explain could never resolve).
        for code in [
            ids::SPEC_PARSE,
            ids::MISSING_ENV_CLI,
            ids::MISSING_ENV_HELPER,
            ids::CACHE_BUILD,
            ids::NO_SYNCED_CACHE,
            ids::NOT_A_REPOSIX_TREE,
            ids::BOUND_TREE_URL_PARSE,
            ids::LOG_NEEDS_TIME_TRAVEL,
            ids::SPACES_CONFLUENCE_ONLY,
            ids::REFRESH_OFFLINE_UNIMPL,
            ids::SINCE_PARSE,
            ids::GIT_NOT_ON_PATH,
            ids::INIT_PATH_NOT_UTF8,
            ids::SINCE_NO_CACHE,
            ids::SINCE_NO_TAG,
            ids::SINCE_FETCH_FAILED,
            ids::SINCE_UPDATE_REF_FAILED,
            ids::INIT_EXISTING_REPO_ROOT,
            ids::INIT_FETCH_FAILED,
            ids::ATTACH_NOT_GIT_TREE,
            ids::ATTACH_DUPLICATE_IDS,
            ids::ATTACH_MULTI_SOT,
            ids::HELPER_UPLOAD_PACK,
            ids::HELPER_EOF,
            ids::HELPER_BLOB_LIMIT,
            ids::HELPER_BACKEND_UNREACHABLE,
            ids::HELPER_PUSH_CONFLICT,
            ids::HELPER_UNFILTERED_FETCH,
            ids::HELPER_MALFORMED_BUS_URL,
            ids::HELPER_USAGE,
            ids::HELPER_MIRROR_UNREACHABLE,
            ids::EXPLAIN_UNKNOWN_CODE,
        ] {
            assert!(
                explain(code).is_some(),
                "ids const {code} has no ExplainEntry"
            );
        }
    }

    #[test]
    fn entry_is_copy_for_cheap_lookup_returns() {
        // ExplainEntry is a small Copy struct of &'static str — confirm lookups are
        // reference returns into the static REGISTRY (no allocation).
        let a: &'static ExplainEntry = explain(ids::SPEC_PARSE).unwrap();
        let b: &'static ExplainEntry = explain(ids::SPEC_PARSE).unwrap();
        assert!(
            std::ptr::eq(a, b),
            "explain must return the same static reference"
        );
    }
}
