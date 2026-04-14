//! FUSE integration tests for the Phase-13 nested mount layout.
//!
//! Every test here is `#[ignore]`-gated because it mounts FUSE in a
//! tempdir, which requires `/dev/fuse` + `fusermount3` on the host.
//! Run with:
//!
//! ```bash
//! cargo test -p reposix-fuse --release -- --ignored --test-threads=1 nested_layout
//! ```
//!
//! Why `--test-threads=1`: each test mounts FUSE in a tempdir; concurrent
//! mounts race on `fusermount3 -u` and leave the kernel mount table in an
//! inconsistent state. Matches the existing `readdir.rs` + `sim_death_no_hang.rs`
//! convention.
//!
//! # Backend = wiremock-Confluence
//!
//! The wiremock `MockServer` here exposes the three Confluence v2 endpoints
//! the FUSE read path hits during a mount:
//!
//! 1. `GET /wiki/api/v2/spaces?keys=REPOSIX` → space id resolution.
//! 2. `GET /wiki/api/v2/spaces/12345/pages?limit=100` → page list with
//!    `parentId` + `parentType` populated per fixture.
//! 3. `GET /wiki/api/v2/pages/{id}?body-format=storage` → per-page body.
//!
//! `REPOSIX_ALLOWED_ORIGINS` is set per-test to include the `MockServer`
//! URL so the SG-01 allowlist admits wiremock's `127.0.0.1:*` — matching
//! the default allowlist in `reposix_core::http` (which already includes
//! `127.0.0.1:*`). Nothing to configure externally.

#![cfg(target_os = "linux")]

use std::sync::Arc;
use std::time::{Duration, Instant};

use reposix_confluence::{ConfluenceCreds, ConfluenceBackend};
use reposix_core::IssueBackend;
use reposix_fuse::{Mount, MountConfig};
use serde_json::{json, Value};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ------------------------------------------------------------------ helpers

fn wait_for<F: FnMut() -> bool>(mut pred: F, budget: Duration) -> bool {
    let t0 = Instant::now();
    loop {
        if pred() {
            return true;
        }
        if t0.elapsed() >= budget {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// One Confluence-v2-shaped page object.
fn conf_page(id: u64, title: &str, parent: Option<u64>, body_xhtml: &str) -> Value {
    let mut page = json!({
        "id": id.to_string(),
        "status": "current",
        "title": title,
        "createdAt": "2024-01-15T10:30:00.000Z",
        "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
        "ownerId": null,
        "body": {"storage": {"value": body_xhtml, "representation": "storage"}}
    });
    if let Some(pid) = parent {
        page["parentId"] = json!(pid.to_string());
        page["parentType"] = json!("page");
    }
    page
}

/// Register the Confluence v2 fixtures on `server` for a given pages list.
/// Mounts the space-key resolver + list + per-page get endpoints. The
/// "any other page id → 404" wildcard is mounted last so specific ids
/// match first.
async fn install_confluence_fixtures(server: &MockServer, pages: &[Value]) {
    // 1. Space-key resolver.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}]
        })))
        .mount(server)
        .await;

    // 2. List pages. Single-shot, no _links.next pagination.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": pages,
            "_links": {}
        })))
        .mount(server)
        .await;

    // 3. Per-page GET — one mount per page.
    for page in pages {
        let id = page["id"].as_str().expect("page id").to_owned();
        let p = format!("/wiki/api/v2/pages/{id}");
        Mock::given(method("GET"))
            .and(path(p))
            .and(query_param("body-format", "storage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page))
            .mount(server)
            .await;
    }
}

/// Boot a `ConfluenceBackend` + FUSE mount against the wiremock
/// `server`. Returns `(mount, mount_path, tempdir)`. Drop `tempdir` last.
fn boot_mount(server_uri: String) -> (reposix_fuse::Mount, std::path::PathBuf, tempfile::TempDir) {
    let td = tempfile::Builder::new()
        .prefix("reposix-nested-")
        .tempdir()
        .expect("tempdir");
    let mount_path = td.path().to_path_buf();

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend: Arc<dyn IssueBackend> = Arc::new(
        ConfluenceBackend::new_with_base_url(creds, server_uri.clone())
            .expect("confluence backend"),
    );
    let mount = Mount::open(
        &MountConfig {
            mount_point: mount_path.clone(),
            origin: server_uri,
            project: "REPOSIX".to_owned(),
            read_only: true,
        },
        backend,
    )
    .expect("mount opened");

    // Wait for `.gitignore` + `pages` + `tree` to surface.
    let mp = mount_path.clone();
    let ok = wait_for(
        move || {
            std::fs::read_dir(&mp)
                .map(|it| {
                    let names: Vec<_> = it
                        .flatten()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                        .collect();
                    names.iter().any(|n| n == ".gitignore")
                        && names.iter().any(|n| n == "pages")
                        && names.iter().any(|n| n == "tree")
                })
                .unwrap_or(false)
        },
        Duration::from_secs(5),
    );
    assert!(
        ok,
        "mount did not expose .gitignore + pages + tree within 5s"
    );
    (mount, mount_path, td)
}

/// Unmount `mount` (via Drop) and wait up to 3s for `mount_path` to
/// either be empty (tempdir view) or non-existent. Asserts cleanly.
fn unmount_and_wait(mount: reposix_fuse::Mount, mount_path: &std::path::Path) {
    drop(mount);
    let mp = mount_path.to_path_buf();
    let unmounted = wait_for(
        move || {
            std::fs::read_dir(&mp)
                .map(|it| it.flatten().count() == 0)
                .unwrap_or(true)
        },
        Duration::from_secs(3),
    );
    assert!(unmounted, "mount did not unmount within 3s");
}

// ------------------------------------------------------------------ 3-level

/// Demo-space fixture: the 4 pages from the REPOSIX Confluence demo.
///
/// ```text
/// 360556 "reposix demo space Home"        (root, no parent)
/// ├── 131192 "Welcome to reposix"         (parent=360556)
/// ├── 65916  "Architecture notes"         (parent=360556)
/// └── 425985 "Demo plan"                  (parent=360556)
/// ```
fn demo_space_fixture() -> Vec<Value> {
    vec![
        conf_page(360_556, "reposix demo space Home", None, "<p>home body</p>"),
        conf_page(
            131_192,
            "Welcome to reposix",
            Some(360_556),
            "<p>welcome body</p>",
        ),
        conf_page(
            65916,
            "Architecture notes",
            Some(360_556),
            "<p>arch body</p>",
        ),
        conf_page(425_985, "Demo plan", Some(360_556), "<p>plan body</p>"),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires FUSE + fusermount3; run with --ignored --test-threads=1"]
async fn nested_layout_three_level_hierarchy() {
    let server = MockServer::start().await;
    install_confluence_fixtures(&server, &demo_space_fixture()).await;

    let (mount, mount_path, td) = boot_mount(server.uri());

    // (1) root listing: exactly `.gitignore`, `pages`, `tree`.
    let mut root: Vec<String> = std::fs::read_dir(&mount_path)
        .expect("read_dir root")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    root.sort();
    assert_eq!(
        root,
        vec![
            ".gitignore".to_owned(),
            "pages".to_owned(),
            "tree".to_owned()
        ],
        "root listing"
    );

    // (2) pages/ contains 4 padded-id entries plus the synthesized
    // `_INDEX.md` summary (Phase 15 Wave A, LD-15-01).
    let mut pages: Vec<String> = std::fs::read_dir(mount_path.join("pages"))
        .expect("read_dir pages")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    pages.sort();
    assert_eq!(
        pages,
        vec![
            "00000065916.md".to_owned(),
            "00000131192.md".to_owned(),
            "00000360556.md".to_owned(),
            "00000425985.md".to_owned(),
            "_INDEX.md".to_owned(),
        ],
        "pages/ listing"
    );

    // (3) tree/ contains exactly one directory (the homepage has children).
    let mut tree_root: Vec<String> = std::fs::read_dir(mount_path.join("tree"))
        .expect("read_dir tree")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    tree_root.sort();
    assert_eq!(
        tree_root,
        vec!["reposix-demo-space-home".to_owned()],
        "tree root listing"
    );

    // (4) Inside the hierarchy dir: _self.md + 3 children.
    let mut tree_home: Vec<String> =
        std::fs::read_dir(mount_path.join("tree/reposix-demo-space-home"))
            .expect("read_dir tree/home")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
    tree_home.sort();
    assert_eq!(
        tree_home,
        vec![
            "_self.md".to_owned(),
            "architecture-notes.md".to_owned(),
            "demo-plan.md".to_owned(),
            "welcome-to-reposix.md".to_owned(),
        ],
        "tree/homepage children"
    );

    // (5) readlink returns exact depth-correct targets.
    let self_target = std::fs::read_link(mount_path.join("tree/reposix-demo-space-home/_self.md"))
        .expect("readlink _self.md");
    assert_eq!(
        self_target.to_string_lossy(),
        "../../pages/00000360556.md",
        "_self.md target"
    );

    let welcome_target =
        std::fs::read_link(mount_path.join("tree/reposix-demo-space-home/welcome-to-reposix.md"))
            .expect("readlink welcome");
    assert_eq!(
        welcome_target.to_string_lossy(),
        "../../pages/00000131192.md",
        "welcome-to-reposix.md target"
    );

    // (6) The kernel should transparently follow the symlink through our
    // FUSE lookup/read pipeline. `read_to_string` on the symlink path
    // must return the rendered frontmatter+body of the target page.
    let welcome_body = std::fs::read_to_string(
        mount_path.join("tree/reposix-demo-space-home/welcome-to-reposix.md"),
    )
    .expect("read_to_string welcome via symlink");
    assert!(
        welcome_body.starts_with("---\n"),
        "symlink-followed read should begin with frontmatter fence: {welcome_body:?}"
    );
    assert!(
        welcome_body.contains("id: 131192"),
        "symlink-followed read missing id line: {welcome_body:?}"
    );

    // Cross-check: the same bytes appear when reading pages/<id>.md directly.
    let direct_body =
        std::fs::read_to_string(mount_path.join("pages/00000131192.md")).expect("read pages");
    assert_eq!(
        welcome_body, direct_body,
        "symlink and direct path should return identical bytes"
    );

    // (7) `.gitignore` content is `/tree/\n` exactly (7 bytes).
    let gi = std::fs::read(mount_path.join(".gitignore")).expect("read .gitignore");
    assert_eq!(gi, b"/tree/\n", "gitignore must be /tree/\\n; got {gi:?}");
    assert_eq!(gi.len(), 7, ".gitignore must be 7 bytes");

    unmount_and_wait(mount, &mount_path);
    drop(td);
}

// ------------------------------------------------------------------ collision

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires FUSE + fusermount3; run with --ignored --test-threads=1"]
async fn nested_layout_collision_gets_suffixed() {
    // Parent with 3 children of identical title. Wave-A's `dedupe_siblings`
    // contract: ascending-IssueId keeps the bare slug; the next two get
    // `-2` and `-3` suffixes.
    let pages = vec![
        conf_page(10, "Parent", None, "<p>parent body</p>"),
        conf_page(11, "Same Title", Some(10), "<p>child a</p>"),
        conf_page(12, "Same Title", Some(10), "<p>child b</p>"),
        conf_page(13, "Same Title", Some(10), "<p>child c</p>"),
    ];
    let server = MockServer::start().await;
    install_confluence_fixtures(&server, &pages).await;

    let (mount, mount_path, td) = boot_mount(server.uri());

    let mut children: Vec<String> = std::fs::read_dir(mount_path.join("tree/parent"))
        .expect("read_dir tree/parent")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    children.sort();
    assert_eq!(
        children,
        vec![
            "_self.md".to_owned(),
            "same-title-2.md".to_owned(),
            "same-title-3.md".to_owned(),
            "same-title.md".to_owned(),
        ],
        "collision dedup should yield bare + -2 + -3"
    );

    // All three symlinks must resolve to different page ids.
    let t11 = std::fs::read_link(mount_path.join("tree/parent/same-title.md"))
        .expect("readlink same-title.md");
    let t12 = std::fs::read_link(mount_path.join("tree/parent/same-title-2.md"))
        .expect("readlink same-title-2.md");
    let t13 = std::fs::read_link(mount_path.join("tree/parent/same-title-3.md"))
        .expect("readlink same-title-3.md");
    assert_eq!(
        t11.to_string_lossy(),
        "../../pages/00000000011.md",
        "bare slug points at smallest id (11)"
    );
    assert_eq!(
        t12.to_string_lossy(),
        "../../pages/00000000012.md",
        "-2 suffix points at next id (12)"
    );
    assert_eq!(
        t13.to_string_lossy(),
        "../../pages/00000000013.md",
        "-3 suffix points at largest id (13)"
    );

    unmount_and_wait(mount, &mount_path);
    drop(td);
}

// ------------------------------------------------------------------ cycle

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires FUSE + fusermount3; run with --ignored --test-threads=1"]
async fn nested_layout_cycle_does_not_hang() {
    // Two-page cycle: A.parent=B, B.parent=A. Wave-B2's cycle detector
    // must break the cycle and expose both pages as tree roots; the mount
    // must stay responsive within a bounded wall-clock budget.
    let pages = vec![
        conf_page(1, "Page A", Some(2), "<p>a</p>"),
        conf_page(2, "Page B", Some(1), "<p>b</p>"),
    ];
    let server = MockServer::start().await;
    install_confluence_fixtures(&server, &pages).await;

    let t0 = Instant::now();
    let (mount, mount_path, td) = boot_mount(server.uri());
    assert!(
        t0.elapsed() < Duration::from_secs(5),
        "mount open with cycle took >5s (likely infinite recursion): {:?}",
        t0.elapsed()
    );

    // Both pages appear under pages/ (bucket is not affected by cycle),
    // alongside the synthesized `_INDEX.md` summary (Phase 15, LD-15-01).
    let mut pages_names: Vec<String> = std::fs::read_dir(mount_path.join("pages"))
        .expect("read_dir pages")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    pages_names.sort();
    assert_eq!(
        pages_names,
        vec![
            "00000000001.md".to_owned(),
            "00000000002.md".to_owned(),
            "_INDEX.md".to_owned(),
        ],
        "pages/ should list both cycle members"
    );

    // tree/ should list both as roots (both broken out of the cycle).
    let readdir_start = Instant::now();
    let mut tree: Vec<String> = std::fs::read_dir(mount_path.join("tree"))
        .expect("read_dir tree")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        readdir_start.elapsed() < Duration::from_secs(3),
        "readdir tree/ took >3s — mount might be stuck"
    );
    tree.sort();
    assert_eq!(
        tree,
        vec!["page-a.md".to_owned(), "page-b.md".to_owned()],
        "both cycle members should surface as tree roots"
    );

    unmount_and_wait(mount, &mount_path);
    drop(td);
}

// ------------------------------------------------------------------ gitignore content

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires FUSE + fusermount3; run with --ignored --test-threads=1"]
async fn nested_layout_gitignore_content_exact() {
    // Minimal fixture: 1 root page (no children needed). We only care
    // about `.gitignore` here, but the mount still needs the backend to
    // resolve the space key and list pages.
    let pages = vec![conf_page(1, "solo", None, "<p>solo</p>")];
    let server = MockServer::start().await;
    install_confluence_fixtures(&server, &pages).await;

    let (mount, mount_path, td) = boot_mount(server.uri());

    let gi = std::fs::read(mount_path.join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        gi, b"/tree/\n",
        "gitignore bytes must be exactly /tree/\\n; got {gi:?}"
    );
    assert_eq!(gi.len(), 7, ".gitignore must be 7 bytes");

    // Sanity — metadata claims the right size too (otherwise ls -l
    // shows wrong byte count).
    let md = std::fs::metadata(mount_path.join(".gitignore")).expect("stat .gitignore");
    assert_eq!(md.len(), 7, "stat size must match read length");
    assert!(md.is_file(), ".gitignore must be a regular file");

    unmount_and_wait(mount, &mount_path);
    drop(td);
}

// ------------------------------------------------------------------ depth

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires FUSE + fusermount3; run with --ignored --test-threads=1"]
async fn nested_layout_readlink_target_depth_is_correct() {
    // 3-level chain: 1 → 2 → 3. Each non-leaf becomes a dir; the leaf
    // is a symlink at depth 2.
    //
    // Expected shape:
    // tree/
    // └── grandparent/
    //     ├── _self.md           -> ../../pages/00000000001.md   (depth 1)
    //     └── parent/
    //         ├── _self.md       -> ../../../pages/00000000002.md   (depth 2)
    //         └── child.md       -> ../../../pages/00000000003.md   (depth 2)
    //
    // Target `../` count = depth + 1 (the +1 hops out of tree/).
    let pages = vec![
        conf_page(1, "grandparent", None, "<p>gp</p>"),
        conf_page(2, "parent", Some(1), "<p>p</p>"),
        conf_page(3, "child", Some(2), "<p>c</p>"),
    ];
    let server = MockServer::start().await;
    install_confluence_fixtures(&server, &pages).await;

    let (mount, mount_path, td) = boot_mount(server.uri());

    // Helper: count leading `../` in a target string.
    let count_dotdots = |t: &str| t.split('/').take_while(|s| *s == "..").count();

    // Depth 0 → target has 1 `../` (none here — grandparent is a dir, no
    // leaf symlink at depth 0 because it has children).

    // Depth 1 (direct child of grandparent/): `_self.md` of the parent
    // dir AND the grandparent's own `_self.md`.
    let gp_self = std::fs::read_link(mount_path.join("tree/grandparent/_self.md"))
        .expect("readlink gp _self");
    assert_eq!(
        count_dotdots(&gp_self.to_string_lossy()),
        2,
        "grandparent _self: 1 level deep → 2 ../ segments; got {gp_self:?}"
    );
    assert_eq!(
        gp_self.to_string_lossy(),
        "../../pages/00000000001.md",
        "grandparent _self target"
    );

    // Depth 2: parent/_self.md and parent/child.md.
    let p_self = std::fs::read_link(mount_path.join("tree/grandparent/parent/_self.md"))
        .expect("readlink p _self");
    assert_eq!(
        count_dotdots(&p_self.to_string_lossy()),
        3,
        "parent _self: 2 levels deep → 3 ../ segments; got {p_self:?}"
    );
    assert_eq!(
        p_self.to_string_lossy(),
        "../../../pages/00000000002.md",
        "parent _self target"
    );

    let child = std::fs::read_link(mount_path.join("tree/grandparent/parent/child.md"))
        .expect("readlink child");
    assert_eq!(
        count_dotdots(&child.to_string_lossy()),
        3,
        "child leaf: 2 levels deep → 3 ../ segments; got {child:?}"
    );
    assert_eq!(
        child.to_string_lossy(),
        "../../../pages/00000000003.md",
        "child target"
    );

    // End-to-end symlink traversal proof: `cat tree/gp/parent/child.md`
    // must return the rendered bytes of pages/00000000003.md.
    let via_symlink = std::fs::read_to_string(mount_path.join("tree/grandparent/parent/child.md"))
        .expect("read via deep symlink");
    let direct =
        std::fs::read_to_string(mount_path.join("pages/00000000003.md")).expect("read direct");
    assert_eq!(
        via_symlink, direct,
        "deep symlink traversal must yield identical bytes"
    );

    unmount_and_wait(mount, &mount_path);
    drop(td);
}
