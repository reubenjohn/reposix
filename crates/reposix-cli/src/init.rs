//! `reposix init <backend>::<project> <path>` — git-native partial-clone bootstrap.
//!
//! Replaces `reposix mount` (deleted in v0.9.0). Runs the six-step git
//! sequence locked in `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §5:
//!
//! 1. `git init <path>`
//! 2. `git -C <path> config extensions.partialClone origin`
//! 3. `git -C <path> config remote.origin.url <url>`
//! 4. `git -C <path> config remote.origin.promisor true`
//! 5. `git -C <path> config remote.origin.partialclonefilter blob:none`
//! 6. `git -C <path> fetch --filter=blob:none origin` *(best-effort)*
//!
//! The translation from the friendly `<backend>::<project>` form to the
//! helper-compatible `reposix::<scheme>://<host>/projects/<project>` URL is
//! [`translate_spec_to_url`].

use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use reposix_core::codes::ids;
use reposix_core::errmsg::teach_coded;
use reposix_remote::backend_dispatch::redact_userinfo;

use crate::errors::{missing_env_var_error, spec_parse_error};

/// Default sim REST origin used when the user runs `reposix init sim::<slug>`.
/// Matches the default bind in `crates/reposix-sim` (`127.0.0.1:7878`).
const DEFAULT_SIM_ORIGIN: &str = "http://127.0.0.1:7878";

/// Default GitHub API origin for `github::<owner>/<repo>` specs.
const DEFAULT_GITHUB_ORIGIN: &str = "https://api.github.com";

/// Translate a friendly `<backend>::<project>` spec into a
/// `reposix::<scheme>://<host>/projects/<project>` URL the helper accepts.
///
/// Backends:
/// - `sim::<slug>` → uses [`DEFAULT_SIM_ORIGIN`].
/// - `github::<owner>/<repo>` → uses [`DEFAULT_GITHUB_ORIGIN`]; the project
///   slug is the full `<owner>/<repo>` pair.
/// - `confluence::<space>` → requires `REPOSIX_CONFLUENCE_TENANT`;
///   constructs `https://<tenant>.atlassian.net`.
/// - `jira::<key>` → requires `REPOSIX_JIRA_INSTANCE`;
///   constructs `https://<instance>.atlassian.net`.
///
/// # Errors
/// Returns an error if the spec is missing the `::` separator, the backend
/// is unknown, or a required env var (`REPOSIX_CONFLUENCE_TENANT` /
/// `REPOSIX_JIRA_INSTANCE`) is unset for confluence/jira.
pub fn translate_spec_to_url(spec: &str) -> Result<String> {
    let (backend, project) = spec.split_once("::").ok_or_else(|| {
        spec_parse_error(
            spec,
            "expected `<backend>::<project>` form (missing `::` separator)",
        )
    })?;

    if project.is_empty() {
        return Err(spec_parse_error(spec, "empty project slug after `::`"));
    }

    match backend {
        "sim" => {
            // Honour `REPOSIX_SIM_ORIGIN` so an isolated-port sim (tests, or a
            // second local instance) can be init'd against — matching the same
            // override already honoured by `attach` (attach.rs) and `sync`
            // (sync.rs). Without this, `init` alone hardcoded 127.0.0.1:7878,
            // so there was no way to leaf-isolate an end-to-end init→fetch→
            // checkout test on a random port (every other command could).
            let origin = std::env::var("REPOSIX_SIM_ORIGIN")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| DEFAULT_SIM_ORIGIN.to_string());
            Ok(format!("reposix::{origin}/projects/{project}"))
        }
        "github" => Ok(format!(
            "reposix::{DEFAULT_GITHUB_ORIGIN}/projects/{project}"
        )),
        "confluence" => {
            let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT")
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    missing_env_var_error("REPOSIX_CONFLUENCE_TENANT", "confluence", "mycompany")
                })?;
            // Phase 36-followup: the `/confluence/` path marker
            // disambiguates the URL from JIRA at the helper's
            // backend-dispatch layer (both share the same
            // *.atlassian.net origin).
            Ok(format!(
                "reposix::https://{tenant}.atlassian.net/confluence/projects/{project}"
            ))
        }
        "jira" => {
            let instance = std::env::var("REPOSIX_JIRA_INSTANCE")
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    missing_env_var_error("REPOSIX_JIRA_INSTANCE", "jira", "mycompany")
                })?;
            // Phase 36-followup: the `/jira/` path marker
            // disambiguates the URL from Confluence at the helper's
            // backend-dispatch layer.
            Ok(format!(
                "reposix::https://{instance}.atlassian.net/jira/projects/{project}"
            ))
        }
        other => Err(spec_parse_error(
            spec,
            &format!("unknown backend `{other}` — expected one of sim, github, confluence, jira"),
        )),
    }
}

/// Refuse to `reposix init` a target that is ALREADY a git repository root.
///
/// `reposix init` runs `git init <path>` + `git config …` in place; pointed at
/// an existing checkout it RE-INITIALIZES that tree and rewrites
/// `extensions.partialClone` + `remote.origin.url` — the move that flipped
/// `core.bare` and repointed `origin` at the sim on the shared dev tree
/// (2026-07-12 D2 corruption). `init` must CREATE a new tree; adopting an
/// existing one is `reposix attach`'s job.
///
/// Rule (fail-closed, product-sensible — not a `/tmp` special-case): refuse iff
/// the target path already exists AND a `.git` entry (dir or gitfile) sits
/// directly at it, i.e. the target IS a git working-tree root. A path that does
/// not yet exist — even a fresh subdir nested INSIDE an existing working tree,
/// which is exactly the sanctioned `git clone <repo> /tmp/leaf && cd /tmp/leaf
/// && reposix init sim::demo <fresh-subdir>` flow — is allowed, as is a fresh
/// new dir anywhere on disk for a legit end-user init.
///
/// # Errors
/// Returns a teaching error naming `reposix attach` and the /tmp-clone recovery
/// when `path` is an existing git repository root.
fn refuse_existing_repo_root(path: &Path) -> Result<()> {
    // A non-existent target is always a fresh tree → allow. `.git` resolution
    // below follows symlinks / `..` via the filesystem, so a symlinked or
    // traversal-smuggled path that lands on a real repo root still refuses.
    if !path.exists() {
        return Ok(());
    }
    if !path.join(".git").exists() {
        // Exists but is NOT a repo root (empty dir, or a plain dir of files).
        // `git init` here creates a NEW tree — allowed.
        return Ok(());
    }
    // CODED (RPX-0401) but HAND-ROLLED. The `[RPX-0401]` tag rides the first headline
    // line and an `Explain: reposix explain RPX-0401` nudge trails the body — the SAME
    // render shape `teach_coded` produces — while the bespoke 3-part body is kept
    // verbatim (names the corruption shape, points at `reposix attach`, prints the
    // /tmp-clone recovery). The RPX-0401 code IS present, hand-emitted. Regression-
    // guarded by `init_refuses_existing_repo_root` +
    // `init_at_existing_repo_root_teaches_attach_and_recovery`.
    // teach-exempt: ok — hand-rolled RPX-0401; do NOT reroute through teach()/teach_coded (marker hugs the bail! for teach_scan's 2-line window; P122-W2-01 fix-twice).
    bail!(
        "reposix init: refusing to initialize `{path}` — it is already a git repository root. [RPX-0401]\n\
         `reposix init` CREATES a fresh partial-clone working tree; re-initializing an existing \
         checkout rewrites core.bare + remote.origin and corrupts the tree (the 2026-07-12 \
         shared-tree incident).\n\
         Fix: point init at a FRESH, non-existent path, e.g. a new subdir:\n  \
         reposix init <backend>::<project> {path}/reposix-clone\n\
         To adopt an EXISTING checkout into a reposix backend instead, use `reposix attach`.\n\
         For throwaway test setup, clone into /tmp first:\n  \
         git clone <repo> /tmp/leaf && cd /tmp/leaf && reposix init sim::demo sub\n\
         Explain: reposix explain RPX-0401",
        path = path.display(),
    )
}

/// Canonicalize `path` with `realpath -m` semantics — resolve symlinks through the
/// DEEPEST EXISTING ancestor, then apply the (possibly non-existent) tail
/// components LEXICALLY (`.` skipped, `..` popped) onto that real ancestor.
///
/// Unlike [`std::fs::canonicalize`] this does NOT require the leaf to exist, and
/// unlike a naive per-component `push` it collapses `..` in the tail against the
/// canonicalized ancestor — so a tail like `newdir/../../etc` cannot climb OUT of
/// the canonicalized ancestor and defeat the /tmp-safe-zone check (the classic
/// `..`-escape). Resolving symlinks in the existing prefix FIRST (before the
/// lexical `..` pass) matches GNU `realpath -m` and defeats a
/// `/tmp/link -> /elsewhere` smuggle. This MIRRORS
/// `.claude/hooks/leaf-isolation-guard.sh::is_safe`'s `realpath -m` resolution so
/// the binary and the Bash-tool hook agree on which targets are safe — a
/// divergence would let one layer pass what the other refuses.
fn canonicalize_lexical_existing(path: &Path) -> PathBuf {
    // Make absolute (join CWD for a relative path) so the ancestor walk has a root.
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    // Find the LONGEST existing prefix and canonicalize it (resolves every symlink
    // in the existing part, yields an absolute real path). Shrink from the end one
    // component at a time until a prefix canonicalizes.
    let comps: Vec<Component> = abs.components().collect();
    let mut split = comps.len();
    let (canon_prefix, tail_start) = loop {
        let prefix: PathBuf = comps[..split].iter().copied().collect();
        if let Ok(c) = std::fs::canonicalize(&prefix) {
            break (c, split);
        }
        if split == 0 {
            // Pathological (even `/` is unreadable) — fall back to a purely lexical
            // normalization of the whole absolute path below.
            break (PathBuf::from("/"), 0);
        }
        split -= 1;
    };
    // Apply the remaining (non-existent) tail LEXICALLY onto the real ancestor.
    let mut result = canon_prefix;
    for comp in &comps[tail_start..] {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(seg) => result.push(seg),
            // An absolute reset mid-tail cannot occur for a tail peeled off an
            // already-absolute path; ignore defensively.
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    result
}

/// `true` iff `canon` (an already-canonicalized path) is inside the sanctioned
/// throwaway zone — `/tmp` or `/private/tmp` (macOS). MIRRORS the
/// `/tmp|/tmp/*|/private/tmp|/private/tmp/*` safe-zone set in
/// `.claude/hooks/leaf-isolation-guard.sh::is_safe`. Component-wise `starts_with`
/// (not a string prefix) so `/tmpfoo` does NOT read as `/tmp`.
fn is_tmp_safe(canon: &Path) -> bool {
    canon.starts_with("/tmp") || canon.starts_with("/private/tmp")
}

/// Refuse to `reposix init` a fresh target that NESTS inside an existing git
/// working tree OUTSIDE the /tmp throwaway zone.
///
/// Binary-side backstop for the D2 shared-tree-corruption recurrence: the
/// Bash-tool `leaf-isolation-guard.sh` (P102) only fires on the Claude Code Bash
/// TOOL, so a `reposix init` reaching the shared checkout via a subprocess/worktree
/// bypasses it — the exact 2026-07-12 recurrence path. Only a refusal INSIDE the
/// binary cuts that vector; it runs BEFORE any git/filesystem mutation.
///
/// MIRRORS `leaf-isolation-guard.sh::is_safe`: canonicalize the effective target
/// (realpath -m via [`canonicalize_lexical_existing`]); if it is under
/// /tmp or /private/tmp → ALLOW (the sanctioned dark-factory zone, keeping the
/// `git clone /tmp/leaf && reposix init … subdir` flow working). Otherwise walk UP
/// the canonical ancestors; if any holds a `.git` (dir OR gitfile) the target nests
/// inside an existing working tree → refuse RPX-0406.
///
/// # Errors
/// Returns the RPX-0406 teaching error when the canonical target nests inside a
/// non-/tmp git working tree.
fn refuse_nested_in_worktree(path: &Path) -> Result<()> {
    let canon = canonicalize_lexical_existing(path);
    // /tmp safe zone — mirrors leaf-isolation-guard.sh::is_safe. Keeps the
    // dark-factory flow working; Test B proves a /tmp nested init still reaches the
    // fetch step rather than being refused.
    if is_tmp_safe(&canon) {
        return Ok(());
    }
    // Walk UP from the target's PARENT: if any ancestor is a git working tree (a
    // `.git` dir or gitfile), the fresh target would nest inside it.
    let mut cur = canon.parent();
    while let Some(dir) = cur {
        if dir.join(".git").exists() {
            let headline = format!(
                "reposix init: refusing to initialize `{target}` — it is nested inside \
                 an existing git working tree at `{enclosing}` (outside /tmp).",
                target = path.display(),
                enclosing = dir.display(),
            );
            bail!(
                "{}",
                teach_coded(
                    ids::INIT_NESTED_IN_REPO,
                    &headline,
                    "point init at a FRESH path that is NOT inside another git repository \
                     (a new directory of its own) — `reposix init` runs `git init` and \
                     rewrites core.bare/remote.origin, and doing that inside an enclosing \
                     repo can corrupt it (the 2026-07-12 shared-tree incident).",
                    "to adopt this existing/nested checkout into a reposix backend instead, \
                     use `reposix attach`; for a throwaway test tree, init under /tmp.",
                    &[
                        "reposix init <backend>::<project> /tmp/reposix-demo   # a throwaway tree under /tmp",
                        "reposix attach <backend>::<project>                   # adopt an existing/nested checkout",
                    ],
                )
            );
        }
        cur = dir.parent();
    }
    Ok(())
}

/// Run `git <args...>` and return a useful error on non-zero exit.
fn run_git(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    let out = cmd.output().with_context(|| {
        format!(
            "failed to spawn `git {}` (is git installed and on PATH?)",
            args.join(" ")
        )
    })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // teach-exempt: ok — internal git-subprocess wrapper; surfaces git's own stderr verbatim; user-facing setup entry errors (spec parse, existing-repo-root, unreachable fetch) teach at the call sites.
        bail!(
            "`git {}` failed with status {}: {}",
            args.join(" "),
            out.status,
            stderr.trim()
        );
    }
    Ok(())
}

/// Run `git -C <path> <args...>`, returning the raw [`std::process::Output`]
/// so the caller can inspect status + stderr itself.
///
/// Used for the trailing `git fetch` step, where the caller turns a non-zero
/// git exit into a teaching `reposix init` error (an unreachable backend is a
/// hard failure, not a warning — see [`run_with_since`]).
fn run_git_in(path: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).args(args);
    cmd.output()
}

/// Return `true` iff the working tree at `path` has at least one
/// `refs/reposix/origin/*` tracking ref — the honest "the initial fetch
/// actually synced something" signal.
///
/// This exists as defence-in-depth: the process exit code alone is a weak
/// success signal for a fetch through a remote helper. Historically the
/// helper advertised `refspec refs/heads/*:refs/reposix/*` while the
/// fast-import stream wrote `refs/reposix/origin/main`, so git exited 128
/// with a benign `could not read ref refs/reposix/main` even on a fully
/// successful sync (v0.13.1 CHECKOUT-BREAK closed that mismatch). As of
/// RBF-LR-03 layer-2 the helper advertises a PRIVATE import namespace
/// (`refs/heads/*:refs/reposix-import/*`) as the fast-import write target;
/// git then maps that into the tracking namespace via the
/// `remote.origin.fetch` refspec configured below, so `refs/reposix/origin/*`
/// is written by git fetch (its sole writer), not by the helper stream. We
/// keep the ref-reality check regardless: a reachable backend leaves at least
/// one `refs/reposix/origin/*` ref; an unreachable one leaves none.
fn repo_has_synced_refs(path: &Path) -> bool {
    run_git_in(
        path,
        &[
            "for-each-ref",
            "--count=1",
            "--format=%(refname)",
            "refs/reposix/origin/",
        ],
    )
    .is_ok_and(|o| o.status.success() && !o.stdout.is_empty())
}

/// `reposix init` entry point.
///
/// `since` is an optional RFC-3339 timestamp. When set, after the normal
/// `git fetch` completes, the working tree's HEAD is rewound to the
/// closest cache sync tag at-or-before the timestamp. Errors clearly
/// when no sync tag exists at-or-before the target.
///
/// # Errors
/// Returns an error if `spec` cannot be translated, if any of `git init`
/// or the four `git config` invocations fail, if `git` is not on PATH, or
/// if the initial `git fetch` from the backend fails (an unreachable backend
/// exits non-zero with a teaching error — it is NOT masked as success).
/// When `since` is set and no matching sync tag exists, `init` errors with a
/// non-zero exit (after configuring the working tree).
pub fn run(spec: String, path: PathBuf) -> Result<()> {
    run_with_since(spec, path, None)
}

/// `reposix init --since=<RFC3339>` entry point.
///
/// Same as [`run`] except that, after the normal `git fetch` completes,
/// `since` (if `Some`) selects the closest cache sync tag at-or-before
/// the target and rewinds the working tree's HEAD + `refs/remotes/origin/main`
/// to that historical commit.
///
/// # Errors
/// Same as [`run`], plus:
/// - `since` is not a valid RFC-3339 timestamp.
/// - No sync tag exists at-or-before `since` in the cache.
/// - The local `git fetch <cache-path> <oid>` to bring the historical
///   commit into the working tree fails.
pub fn run_with_since(spec: String, path: PathBuf, since: Option<String>) -> Result<()> {
    // FAIL-CLOSED binary-side refusal (D2 re-seal, v0.14.0 Wave 2). This runs
    // BEFORE any filesystem/git mutation so it also stops a SUBPROCESS bypass
    // of the Bash-tool leaf-isolation hook — the exact recurrence path that
    // corrupted the shared dev tree on 2026-07-12 (a `reposix init` reached the
    // shared checkout via a worktree/subprocess that never touched the
    // PreToolUse guard). Only a refusal INSIDE the binary can cut that vector.
    refuse_existing_repo_root(&path)?;
    // Latch 1 (P122 / RPX-0406, DRAIN-09): refuse a FRESH target that nests inside
    // an existing NON-/tmp git working tree — `refuse_existing_repo_root` only
    // catches a target that IS a repo root; this closes the residual "fresh subdir
    // inside the shared tree" vector. Canonicalized + /tmp-safe so it mirrors
    // leaf-isolation-guard.sh::is_safe and never breaks the dark-factory flow.
    refuse_nested_in_worktree(&path)?;

    let url = translate_spec_to_url(&spec)?;

    // Ensure parent dir exists for `git init`. `git init` creates the leaf
    // dir but not intermediate parents.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create parent dir for {path}", path = path.display()))?;
        }
    }

    let path_str = path.to_str().ok_or_else(|| {
        anyhow!(
            "{}",
            teach_coded(
                ids::INIT_PATH_NOT_UTF8,
                &format!("the target path is not valid UTF-8: {}", path.display()),
                "reposix shells out to git, which needs a UTF-8 path — rename the directory to \
                 plain UTF-8 (ASCII is safest).",
                "",
                &["reposix init <backend>::<project> /tmp/reposix-demo"],
            )
        )
    })?;

    // 1. git init <path>
    run_git(&["init", path_str])?;
    // 2-5. configure partial clone + remote.
    run_git(&[
        "-C",
        path_str,
        "config",
        "extensions.partialClone",
        "origin",
    ])?;
    run_git(&["-C", path_str, "config", "remote.origin.url", &url])?;
    run_git(&["-C", path_str, "config", "remote.origin.promisor", "true"])?;
    run_git(&[
        "-C",
        path_str,
        "config",
        "remote.origin.partialclonefilter",
        "blob:none",
    ])?;
    // Explicit fetch refspec. WITHOUT this, `git fetch origin` maps nothing
    // into a persistent ref (FETCH_HEAD only), so `git checkout
    // refs/reposix/origin/main` — the documented next step (docs/index.md,
    // mental-model-in-60-seconds) — has nothing to resolve. The
    // `refs/reposix/origin/*` namespace (not git's default
    // `refs/remotes/origin/*`) keeps helper-side refs out of the agent's
    // `refs/heads/*`. This refspec is THE mechanism by which git fetch becomes
    // the SOLE writer of the tracking ns: the helper advertises a disjoint
    // PRIVATE namespace `refs/heads/*:refs/reposix-import/*` (its fast-import
    // write target), and git fetch maps the remote `refs/heads/*` it read back
    // from there into `refs/reposix/origin/*` HERE. Collapsing both onto
    // `refs/reposix/origin/*` made the helper stream AND git fetch race on one
    // ref → fetch-time `cannot lock ref … is at T1 but expected T0`, aborting
    // `git pull --rebase` (RBF-LR-03 layer-2). The leading `+` force-updates
    // the tracking ref on drift.
    //
    // KNOWN GAP (filed for v0.14.0, SURPRISES-INTAKE): because this maps to
    // `refs/reposix/origin/*` and NOT `refs/remotes/origin/*`, the pure-git
    // `git checkout origin/main` still fails to resolve — agents must use the
    // fully-named `git checkout -B main refs/reposix/origin/main` (printed in
    // the success banner below). A verified-safe additive second refspec
    // (`+refs/heads/*:refs/remotes/origin/*`) makes `git checkout origin/main`
    // resolve, but lands in detached HEAD, so the edit→commit→push ergonomics
    // need a design pass + verification on the git >= 2.34 stateless-connect
    // fetch path (untestable on this VM's git 2.25) before shipping.
    run_git(&[
        "-C",
        path_str,
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/reposix/origin/*",
    ])?;

    // 6. git fetch --filter=blob:none origin.
    //
    // An unreachable backend is a HARD ERROR, not a warning (v0.13.1
    // onboarding hotfix B4). Previously this step downgraded EVERY failure to
    // `tracing::warn!` and returned `Ok(())`, so `reposix init` against a
    // dead backend exited 0 with the SAME success message as a real sync — a
    // silent lie that leaves an empty repo whose next `git checkout` fails
    // with a confusing "pathspec did not match".
    //
    // Subtlety: git's EXIT CODE is a weak success signal for a helper fetch,
    // so we still cross-check against ref reality. The historical
    // `could not read ref refs/reposix/main` git-128 (advertised refspec
    // `refs/heads/*:refs/reposix/*` vs the fast-import write target
    // `refs/reposix/origin/main`) was closed in v0.13.1 CHECKOUT-BREAK. As of
    // RBF-LR-03 layer-2 the helper's write target is the private
    // `refs/reposix-import/*`, mapped into `refs/reposix/origin/*` by the
    // `remote.origin.fetch` refspec above — git fetch is the sole writer.
    // We keep the ref-reality cross-check regardless: the ground truth of a
    // successful sync is whether the destination refs actually materialized —
    // a reachable backend leaves at least one `refs/reposix/origin/*` ref; an
    // unreachable one leaves none. We honour the ref reality and mirror
    // `Cmd::Doctor`'s honest non-zero exit only when nothing synced.
    let fetch_out = run_git_in(&path, &["fetch", "--filter=blob:none", "origin"]);
    let synced = repo_has_synced_refs(&path);
    match &fetch_out {
        Ok(o) if o.status.success() => {
            tracing::debug!("git fetch --filter=blob:none succeeded");
        }
        _ if synced => {
            // Sync landed `refs/reposix/origin/*` despite git's spurious
            // non-zero exit — this is the happy path. Stay quiet; the success
            // summary is printed below. No misleading warning here.
            tracing::debug!(
                "git fetch exited non-zero but refs/reposix/origin/* is present; sync succeeded"
            );
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            // RPX-0402 (INIT_FETCH_FAILED): the remote was configured but the initial
            // fetch brought nothing back. Routed through `teach_coded` so the
            // `[RPX-0402]` tag + `Explain:` nudge render (matching every other coded
            // site). git's own stderr is SURFACED (not swallowed) but REDACTED first
            // with `redact_userinfo` — the crates/CLAUDE.md rule is "never interpolate
            // raw git stderr", and this makes the no-leak guarantee independent of
            // git's version (modern git strips userinfo from its "unable to access
            // '<url>'" line; an older git could echo it). Matches the sibling redaction
            // in `bus_handler::precheck_mirror_drift`. Regression-guarded by
            // `init_errors_nonzero_when_initial_fetch_fails`.
            let safe_stderr = redact_userinfo(stderr.trim());
            let headline = format!(
                "reposix init: could not sync `{path_str}` from backend `{url}` — the repo was \
                 configured but nothing was fetched, so it has no commits yet.\n\
                 git stderr:\n  {safe_stderr}",
            );
            let fetch_in_place = format!(
                "git -C {path_str} fetch --filter=blob:none origin   # sync the configured tree in place"
            );
            bail!(
                "{}",
                teach_coded(
                    ids::INIT_FETCH_FAILED,
                    &headline,
                    "confirm the backend is running and reachable — for the simulator, start it in \
                     another terminal with `reposix sim` — then re-run `reposix init`.",
                    "the remote is already configured, so you can fetch in place instead of \
                     re-running init.",
                    &[
                        "reposix sim                                        # start the simulator, if you meant sim::…",
                        fetch_in_place.as_str(),
                    ],
                )
            );
        }
        Err(e) => {
            let retry = format!("reposix init <backend>::<project> {path_str}");
            bail!(
                "{}",
                teach_coded(
                    ids::GIT_NOT_ON_PATH,
                    &format!("reposix init: could not invoke `git fetch` for `{path_str}`: {e}"),
                    "`git` must be installed and on PATH (>= 2.34 for reliable partial-clone \
                     fetches) — the fetch subprocess could not be spawned.",
                    "",
                    &[
                        "git --version   # confirm git is installed and on PATH",
                        retry.as_str(),
                    ],
                )
            );
        }
    }

    // The onboarding command MUST be verbatim-runnable. `git checkout
    // origin/main` resolves via `refs/remotes/origin/main`, which this init
    // path deliberately does NOT populate — the fetch refspec lands the
    // synced ref under `refs/reposix/origin/*` (kept out of the agent's
    // `refs/heads/*` and matching the helper's push namespace). So the
    // honest, tested next step is a checkout of that ref by full name. The
    // pure-git `git checkout origin/main` ergonomic is filed for v0.14.0
    // (SURPRISES-INTAKE): populating `refs/remotes/origin/*` additively must
    // be designed + tested on the supported git floor (>= 2.34, which fetches
    // via stateless-connect), not silently bolted on in a hotfix.
    println!(
        "reposix init: configured `{path_str}` with remote.origin.url = {url}\nNext: cd {path_str} && git checkout -B main refs/reposix/origin/main (or git sparse-checkout set <pathspec> first)"
    );

    // --since=<RFC3339> handling — rewind the working tree to a historical
    // sync tag from the cache. Runs AFTER the normal fetch so the cache is
    // populated and contains the tag refs.
    if let Some(ts) = since {
        rewind_to_since(&spec, &path, &ts)?;
    }

    // Record the working-tree path in the cache's meta table so
    // `reposix gc --orphans` can detect caches whose owning working tree
    // has since been deleted. Best-effort: if the cache doesn't exist yet
    // (network-less init, fetch failed), this silently no-ops — the next
    // successful fetch will trigger Cache::open and the same recording
    // call from a subsequent `reposix init` will fix it up.
    record_worktree_in_cache(&spec, &path);

    Ok(())
}

/// Append the absolute working-tree path to the cache's `meta.worktrees`
/// row. Best-effort: a missing cache, an unparseable path, or a SQL error
/// downgrades to a tracing warning. Used by `reposix gc --orphans` to
/// detect caches whose owning working trees have been deleted.
///
/// `meta.worktrees` is stored as a newline-separated list of absolute
/// paths. Duplicates are deduped on insert; ordering is insertion-order
/// for forensics.
fn record_worktree_in_cache(spec: &str, path: &Path) {
    let Some((backend, project)) = spec.split_once("::") else {
        return;
    };
    // S-260707-gh404: pass the RAW project slug — `resolve_cache_path` is the
    // single sanitization site (collapses `owner/repo` → the flat cache dir).
    let Ok(cache_path) = reposix_cache::resolve_cache_path(backend, project) else {
        return;
    };
    let db = cache_path.join("cache.db");
    if !db.exists() {
        return;
    }
    let abs_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let abs_str = match abs_path.to_str() {
        Some(s) => s.to_string(),
        None => return,
    };

    let conn = match rusqlite::Connection::open(&db) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "could not open {db} to record worktree: {e}",
                db = db.display()
            );
            return;
        }
    };
    let existing: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'worktrees'", [], |r| {
            r.get::<_, String>(0)
        })
        .ok();
    let mut entries: Vec<String> = existing
        .as_deref()
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default();
    if !entries.iter().any(|e| e == &abs_str) {
        entries.push(abs_str);
    }
    let value = entries.join("\n");
    let now = chrono::Utc::now().to_rfc3339();
    if let Err(e) = conn.execute(
        "INSERT INTO meta (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        rusqlite::params!["worktrees", value, now],
    ) {
        tracing::warn!("could not write meta.worktrees in {}: {e}", db.display());
    }
}

/// Resolve the cache path for `spec`, look up the closest sync tag at-or-before
/// `target_rfc3339`, and rewind the working tree's `refs/heads/main` +
/// `refs/remotes/origin/main` to that commit. Errors clearly if no
/// matching tag is found.
// Sequential `--since` rewind: parse timestamp → resolve cache → list tags → pick
// tag → fetch commit → update refs, each with a full 3-part teaching error (P120).
// The teach() bodies push it just past the 100-line lint; splitting the linear
// step sequence across helpers would obscure the ordering (same rationale as
// `attach::run`'s allow).
#[allow(clippy::too_many_lines)]
fn rewind_to_since(spec: &str, path: &Path, target_rfc3339: &str) -> Result<()> {
    use chrono::{DateTime, Utc};

    let target: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(target_rfc3339)
        .with_context(|| {
            format!(
                "invalid --since timestamp `{target_rfc3339}` — expected RFC-3339 (e.g. 2026-04-25T01:00:00Z)"
            )
        })?
        .with_timezone(&Utc);

    // Map spec → (backend, project) for the cache resolver. We re-derive
    // here rather than calling translate_spec_to_url + parse_remote_url
    // because the cache path keying uses the friendly slug directly.
    let (backend, project) = spec.split_once("::").ok_or_else(|| {
        spec_parse_error(
            spec,
            "expected `<backend>::<project>` form (missing `::` separator)",
        )
    })?;
    // S-260707-gh404: pass the RAW project slug — `resolve_cache_path` is the
    // single sanitization site (collapses GitHub's `owner/repo` → the flat
    // `github-owner-repo.git` cache dir).
    let cache_path = reposix_cache::resolve_cache_path(backend, project)
        .with_context(|| format!("resolve cache path for {backend}::{project}"))?;
    if !cache_path.exists() {
        let recovery = format!(
            "reposix init {spec} <path>   # populate the cache first, then re-run with --since"
        );
        bail!(
            "{}",
            teach_coded(
                ids::SINCE_NO_CACHE,
                &format!(
                    "no cache at {} for `--since` to rewind into.",
                    cache_path.display()
                ),
                "`--since` rewinds to a historical sync tag, but the cache has not been populated \
                 yet — run a normal `reposix init` (no `--since`) first.",
                "want the latest state instead of a historical snapshot? drop `--since` entirely.",
                &[recovery.as_str()],
            )
        );
    }

    let tags = reposix_cache::list_sync_tags_at(&cache_path)
        .with_context(|| format!("list sync tags from {}", cache_path.display()))?;
    let chosen = tags.into_iter().rev().find(|t| t.timestamp <= target);
    let tag = chosen.ok_or_else(|| {
        let recovery = format!("reposix init {spec} <path>   # (no --since) for the latest state");
        anyhow!(
            "{}",
            teach_coded(
                ids::SINCE_NO_TAG,
                &format!(
                    "no sync tag at-or-before `{target_rfc3339}` in {}.",
                    cache_path.display()
                ),
                "`--since` selects the newest sync tag at-or-before your timestamp; none exists \
                 that early — pick a later timestamp.",
                "want the latest state? omit `--since` and `reposix init` takes the most recent sync.",
                &[recovery.as_str()],
            )
        )
    })?;

    let oid_hex = tag.commit.to_hex().to_string();

    // Bring the historical commit into the working tree's object store.
    // Local-path fetch by SHA works against the cache's bare repo regardless
    // of `transfer.hideRefs` because we name the OID, not the hidden ref.
    let cache_str = cache_path
        .to_str()
        // teach-exempt: ok — machine-derived cache path (from `resolve_cache_path`); a non-UTF8
        // cache path is pathological on supported platforms.
        .ok_or_else(|| anyhow!("cache path is not valid UTF-8: {}", cache_path.display()))?;
    let path_str = path
        .to_str()
        // teach-exempt: ok — the same user `<path>` was already UTF-8-validated at the top of
        // `run_with_since` before the `--since` rewind runs; unreachable here.
        .ok_or_else(|| anyhow!("working-tree path is not valid UTF-8: {}", path.display()))?;
    let fetch_out = Command::new("git")
        .arg("-C")
        .arg(path_str)
        .args(["fetch", "--filter=blob:none", cache_str, &oid_hex])
        .output()
        .with_context(|| {
            format!("invoke `git fetch --filter=blob:none {cache_str} {oid_hex}` from {path_str}")
        })?;
    if !fetch_out.status.success() {
        // Redact any `scheme://user:secret@` userinfo git might echo before it
        // reaches a user-facing error (crates/CLAUDE.md § "never interpolate raw
        // git stderr"; defense-in-depth, version-independent).
        let git_stderr = redact_userinfo(String::from_utf8_lossy(&fetch_out.stderr).trim());
        bail!(
            "{}",
            teach_coded(
                ids::SINCE_FETCH_FAILED,
                &format!(
                    "could not bring historical commit {oid_hex} into the working tree from cache \
                     {cache}.\ngit stderr:\n  {git_stderr}",
                    cache = cache_path.display(),
                ),
                "the sync tag resolved but its commit could not be fetched from the local cache — \
                 the cache may be incomplete or have been gc'd.",
                "want the latest state instead? re-run `reposix init` without `--since`.",
                &["reposix init <backend>::<project> <path>   # (no --since) repopulates, then retry --since"],
            )
        );
    }

    // Update the working tree's main + origin/main refs to the historical
    // commit so `git checkout main` puts the agent at the snapshot.
    for refname in ["refs/heads/main", "refs/remotes/origin/main"] {
        let out = Command::new("git")
            .arg("-C")
            .arg(path_str)
            .args(["update-ref", refname, &oid_hex])
            .output()
            .with_context(|| format!("update-ref {refname} -> {oid_hex}"))?;
        if !out.status.success() {
            // Redact any `scheme://user:secret@` userinfo git might echo before it
            // reaches a user-facing error (crates/CLAUDE.md § "never interpolate raw
            // git stderr"; defense-in-depth, version-independent).
            let git_stderr = redact_userinfo(String::from_utf8_lossy(&out.stderr).trim());
            bail!(
                "{}",
                teach_coded(
                    ids::SINCE_UPDATE_REF_FAILED,
                    &format!(
                        "could not move `{refname}` to the historical snapshot {oid_hex}.\n\
                         git stderr:\n  {git_stderr}"
                    ),
                    "git update-ref failed while rewinding the working tree to the `--since` \
                     snapshot — the ref may be locked by a concurrent git process.",
                    "",
                    &["reposix init <backend>::<project> <path>   # (no --since) for the latest state, then retry"],
                )
            );
        }
    }

    println!(
        "reposix init --since={target_rfc3339}: rewound to sync tag {tag} (commit {oid_short})\n      cache: {cache}",
        tag = tag.name,
        oid_short = oid_hex.chars().take(12).collect::<String>(),
        cache = cache_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests that mutate process-wide env vars must run serially; cargo test
    // spawns one thread per test, so concurrent set_var/remove_var races.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn translate_sim_spec() {
        let url = translate_spec_to_url("sim::demo").unwrap();
        assert_eq!(url, "reposix::http://127.0.0.1:7878/projects/demo");
    }

    #[test]
    fn translate_github_spec() {
        let url = translate_spec_to_url("github::reubenjohn/reposix").unwrap();
        assert_eq!(
            url,
            "reposix::https://api.github.com/projects/reubenjohn/reposix"
        );
    }

    #[test]
    fn translate_confluence_emits_path_marker() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Phase 36-followup: the `/confluence/` path marker is what
        // the helper's URL-scheme dispatcher uses to disambiguate
        // between Confluence and JIRA on the shared *.atlassian.net
        // origin. Pin it here so init/helper stay in sync.
        let saved = std::env::var("REPOSIX_CONFLUENCE_TENANT").ok();
        std::env::set_var("REPOSIX_CONFLUENCE_TENANT", "reuben-john");
        let url = translate_spec_to_url("confluence::TokenWorld").unwrap();
        assert_eq!(
            url,
            "reposix::https://reuben-john.atlassian.net/confluence/projects/TokenWorld"
        );
        match saved {
            Some(v) => std::env::set_var("REPOSIX_CONFLUENCE_TENANT", v),
            None => std::env::remove_var("REPOSIX_CONFLUENCE_TENANT"),
        }
    }

    #[test]
    fn translate_jira_emits_path_marker() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = std::env::var("REPOSIX_JIRA_INSTANCE").ok();
        std::env::set_var("REPOSIX_JIRA_INSTANCE", "reuben-john");
        let url = translate_spec_to_url("jira::TEST").unwrap();
        assert_eq!(
            url,
            "reposix::https://reuben-john.atlassian.net/jira/projects/TEST"
        );
        match saved {
            Some(v) => std::env::set_var("REPOSIX_JIRA_INSTANCE", v),
            None => std::env::remove_var("REPOSIX_JIRA_INSTANCE"),
        }
    }

    #[test]
    fn translate_confluence_requires_tenant() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Save and clear the env var to ensure this test is deterministic.
        let saved = std::env::var("REPOSIX_CONFLUENCE_TENANT").ok();
        std::env::remove_var("REPOSIX_CONFLUENCE_TENANT");
        let err = translate_spec_to_url("confluence::TokenWorld").unwrap_err();
        assert!(
            err.to_string().contains("REPOSIX_CONFLUENCE_TENANT"),
            "expected error to name env var, got: {err}"
        );
        if let Some(v) = saved {
            std::env::set_var("REPOSIX_CONFLUENCE_TENANT", v);
        }
    }

    #[test]
    fn translate_jira_requires_instance() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = std::env::var("REPOSIX_JIRA_INSTANCE").ok();
        std::env::remove_var("REPOSIX_JIRA_INSTANCE");
        let err = translate_spec_to_url("jira::TEST").unwrap_err();
        assert!(
            err.to_string().contains("REPOSIX_JIRA_INSTANCE"),
            "expected error to name env var, got: {err}"
        );
        if let Some(v) = saved {
            std::env::set_var("REPOSIX_JIRA_INSTANCE", v);
        }
    }

    #[test]
    fn translate_rejects_missing_separator() {
        let err = translate_spec_to_url("sim").unwrap_err();
        assert!(
            err.to_string().contains("expected `<backend>::<project>`"),
            "got: {err}"
        );
    }

    #[test]
    fn translate_rejects_unknown_backend() {
        let err = translate_spec_to_url("foo::bar").unwrap_err();
        assert!(
            err.to_string().contains("unknown backend `foo`"),
            "got: {err}"
        );
    }

    #[test]
    fn translate_rejects_empty_project() {
        let err = translate_spec_to_url("sim::").unwrap_err();
        assert!(err.to_string().contains("empty project"), "got: {err}");
    }

    /// B4 (v0.13.1 onboarding hotfix): a failed initial `git fetch` — because
    /// the backend is unreachable (nothing listening on the sim's default
    /// `127.0.0.1:7878`) or the `reposix` remote helper cannot be resolved —
    /// MUST make `reposix init` exit non-zero with a teaching error, NOT be
    /// downgraded to a warning + `Ok(())` that prints the same success banner
    /// as a real sync. The whole invocation is confined to an isolated
    /// tempdir (`git init`/`config`/`fetch` all run via `git -C <tempdir>`),
    /// so nothing touches the shared repo's `.git/config` or object store.
    #[test]
    // test-name-honesty: ok — attempts a genuine `git fetch` against an unreachable
    // sim origin (nothing listening) and asserts the resulting error/exit-code
    // teaches recovery; not a round-trip success path, only the failure contract.
    fn init_errors_nonzero_when_initial_fetch_fails() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        let err = run("sim::demo".to_string(), repo.clone())
            .expect_err("init must return Err when the initial fetch cannot complete");
        let msg = err.to_string();
        assert!(
            msg.contains("could not sync") && msg.contains("Fix:"),
            "error must report the failed sync and teach recovery, got: {msg}"
        );
        // SC1 (P121): the fetch-failure teaching carries the RPX-0402 code tag +
        // the `reposix explain` nudge, so a dev who hits it can look up the extended
        // cause/fix/recovery.
        assert!(
            msg.contains("[RPX-0402]") && msg.contains("reposix explain RPX-0402"),
            "fetch-failure error must carry the [RPX-0402] tag + explain nudge, got: {msg}"
        );
        // The tree was configured (git init + config ran) — the failure is at
        // the sync step — but `init` did NOT report success.
        assert!(
            repo.join(".git").exists(),
            "git init should have run in the isolated tempdir before the fetch"
        );
    }

    /// D2 re-seal (v0.14.0 Wave 2): `reposix init` MUST fail-closed when its
    /// target is ALREADY a git working-tree root — the corruption shape that
    /// re-initialized (and flipped core.bare on) the shared dev tree. The
    /// refusal runs BEFORE any git/filesystem mutation, so a subprocess bypass
    /// of the Bash-tool hook is still cut. The whole test is confined to a
    /// tempdir; nothing touches the shared repo.
    #[test]
    // test-name-honesty: ok — builds a real repo-root marker (`.git/`) at the target
    // and asserts init REFUSES with a teaching error naming `attach`; the failure
    // contract, not a success round-trip.
    fn init_refuses_existing_repo_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("existing");
        std::fs::create_dir_all(repo.join(".git")).expect("make a git repo-root marker");
        let err = run("sim::demo".to_string(), repo.clone())
            .expect_err("init must refuse an existing git repository root");
        let msg = err.to_string();
        assert!(
            msg.contains("already a git repository root") && msg.contains("reposix attach"),
            "refusal must explain the corruption shape and name `reposix attach`, got: {msg}"
        );
        // SC1 (P121): the exemplar refusal carries the RPX-0401 code tag + the
        // `reposix explain` nudge (hand-emitted, matching the teach_coded render).
        assert!(
            msg.contains("[RPX-0401]") && msg.contains("reposix explain RPX-0401"),
            "refusal must carry the [RPX-0401] tag + explain nudge, got: {msg}"
        );
        // Fail-closed: the refusal fired before `translate_spec_to_url` /
        // `git init`, so no `remote.origin.url` config was written into the
        // pre-existing `.git` — the tree is byte-unchanged.
        assert!(
            !repo.join(".git/config").exists(),
            "refusal must run BEFORE any git config write (no re-init happened)"
        );
    }

    /// D2 re-seal (v0.14.0 Wave 2): the refusal MUST NOT fire on a FRESH subdir
    /// nested inside an existing working tree — the sanctioned
    /// `git clone <repo> /tmp/leaf && cd /tmp/leaf && reposix init sim::demo
    /// <fresh-subdir>` flow (and the dark-factory gate, which inits
    /// `$RUN_DIR/repo`). The only failure here is the expected unreachable-sim
    /// fetch error, proving init got PAST the refusal to the sync step.
    #[test]
    // test-name-honesty: ok — creates a `.git/` repo-root marker then inits a fresh
    // nested subdir and asserts the refusal does NOT fire (error is the fetch error,
    // not the refusal); proves the sanctioned /tmp-clone flow stays green.
    fn init_allows_fresh_subdir_inside_existing_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Make the tempdir itself a git repo root, mirroring a /tmp clone.
        std::fs::create_dir_all(tmp.path().join(".git")).expect("repo-root marker");
        let fresh = tmp.path().join("fresh-subdir");
        // No sim listening on the default origin → the fetch step fails. That
        // is the ONLY thing that should fail: the refusal must have allowed us
        // through.
        let err = run("sim::demo".to_string(), fresh)
            .expect_err("unreachable sim → the initial fetch fails");
        let msg = err.to_string();
        assert!(
            msg.contains("could not sync"),
            "init must reach the fetch step on a fresh subdir (refusal did NOT fire), got: {msg}"
        );
        assert!(
            !msg.contains("already a git repository root"),
            "the existing-repo-root refusal must NOT fire on a fresh nested subdir, got: {msg}"
        );
    }
}
