//! Integration tests for `reposix explain <code>` — the RPX code-lookup
//! subcommand (Phase 121 / P121, the codified half of the Rust-compiler-grade
//! UX north star).
//!
//! REGISTRY-DRIVEN by construction (W2): the per-code test iterates
//! `reposix_core::codes::REGISTRY` rather than a hardcoded count, so every entry
//! ever added (including future W3/W4 codes) is asserted automatically — the
//! suite cannot silently under-cover the registry. Every test drives the REAL
//! built `reposix` binary via `assert_cmd`; the paths are hermetic (no working
//! tree, no network, no seed — `explain` reads only the static registry).

use assert_cmd::Command;
use reposix_core::codes;

/// Drive the real `reposix explain <args>` binary and return `(success, stdout, stderr)`.
fn run_explain(args: &[&str]) -> (bool, String, String) {
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("explain")
        .args(args)
        .output()
        .expect("run `reposix explain`");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// SC2 (the codified north star): for EVERY code in the registry, `reposix
/// explain <code>` prints a non-empty title + cause + `Fix:` + copy-paste
/// `Recovery:`. Iterates the registry — NOT a hardcoded 24/31 — so a newly
/// minted code is covered the moment it lands.
#[test]
fn every_registry_code_explains_with_cause_fix_recovery() {
    assert!(
        !codes::REGISTRY.is_empty(),
        "registry must not be empty — nothing to explain"
    );
    for entry in codes::REGISTRY {
        let (ok, stdout, stderr) = run_explain(&[entry.code]);
        assert!(
            ok,
            "`reposix explain {}` must exit 0; stderr:\n{stderr}",
            entry.code
        );
        // Code header: `RPX-xxxx: <title>` (mirrors rustc's code-header line).
        let header = format!("{}: {}", entry.code, entry.title);
        assert!(
            stdout.contains(&header),
            "{}: missing code header `{header}`; got:\n{stdout}",
            entry.code
        );
        // Extended cause (registry invariant: always non-empty).
        assert!(
            !entry.cause.is_empty() && stdout.contains(entry.cause),
            "{}: extended cause missing from explain output; got:\n{stdout}",
            entry.code
        );
        // Fix limb.
        assert!(
            stdout.contains("Fix: ") && stdout.contains(entry.fix),
            "{}: missing `Fix:` teaching; got:\n{stdout}",
            entry.code
        );
        // Copy-paste recovery block + at least the first runnable line.
        assert!(
            stdout.contains("Recovery:"),
            "{}: missing `Recovery:` block; got:\n{stdout}",
            entry.code
        );
        assert!(
            !entry.recovery.is_empty(),
            "{}: registry entry has no recovery command",
            entry.code
        );
        assert!(
            stdout.contains(entry.recovery[0]),
            "{}: recovery command not rendered; got:\n{stdout}",
            entry.code
        );
        // `Alternative:` limb iff the entry has one (FLAG-1 parity).
        if entry.alternative.is_empty() {
            assert!(
                !stdout.contains("Alternative:"),
                "{}: hollow `Alternative:` line for an entry with no alternative; got:\n{stdout}",
                entry.code
            );
        } else {
            assert!(
                stdout.contains(entry.alternative),
                "{}: alternative not rendered; got:\n{stdout}",
                entry.code
            );
        }
    }
}

/// `reposix explain --list` AND the bare `reposix explain` (no arg) both
/// enumerate EVERY registered code + title, one per line. Registry-driven: the
/// assertion loops over `codes::REGISTRY`, so it never under-checks.
#[test]
fn list_and_no_arg_enumerate_every_code() {
    for args in [vec!["--list"], vec![]] {
        let (ok, stdout, stderr) = run_explain(&args);
        assert!(
            ok,
            "`reposix explain {args:?}` must exit 0; stderr:\n{stderr}"
        );
        assert!(
            stdout.contains("RPX-"),
            "enumerate output must list RPX codes; got:\n{stdout}"
        );
        for entry in codes::REGISTRY {
            assert!(
                stdout.contains(entry.code),
                "enumerate ({args:?}) omitted {}; got:\n{stdout}",
                entry.code
            );
            assert!(
                stdout.contains(entry.title),
                "enumerate ({args:?}) omitted the title for {}; got:\n{stdout}",
                entry.code
            );
        }
    }
}

/// The `--list` enumeration is sorted by code (stable, scannable output).
#[test]
fn list_output_is_sorted_by_code() {
    let (_ok, stdout, _stderr) = run_explain(&["--list"]);
    let codes_in_order: Vec<&str> = stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|tok| tok.starts_with("RPX-"))
        .collect();
    let mut sorted = codes_in_order.clone();
    sorted.sort_unstable();
    assert_eq!(
        codes_in_order, sorted,
        "`--list` must print codes in sorted order"
    );
}

/// SC4 / gate leg-d: `reposix explain RPX-9999` (unknown) TEACHES — it names
/// `reposix explain --list`, exits non-zero, and never panics or prints a bare
/// "not found". Dogfoods the 3-part bar via the explain-meta code RPX-0900.
#[test]
fn unknown_code_teaches_and_exits_nonzero() {
    let (ok, stdout, stderr) = run_explain(&["RPX-9999"]);
    assert!(
        !ok,
        "unknown code must exit non-zero; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.to_lowercase().contains("panicked"),
        "unknown code must not panic; got:\n{combined}"
    );
    assert!(
        combined.contains("RPX-9999"),
        "unknown-code error must name the code the user asked for; got:\n{combined}"
    );
    assert!(
        combined.contains("reposix explain --list"),
        "unknown-code error must name `reposix explain --list` as recovery; got:\n{combined}"
    );
    // It teaches via the explain-meta code RPX-0900 (its `[RPX-0900]` tag rides
    // the headline through the shared `teach_coded` render).
    assert!(
        combined.contains("RPX-0900"),
        "unknown-code teaching should carry the explain-meta code RPX-0900; got:\n{combined}"
    );
}

/// SC3 — `reposix explain` output SHAPE matches `rustc --explain E0308`: a
/// code-header line + a non-empty multi-line body + a fix section. The rustc leg
/// is BEST-EFFORT (skipped when `rustc --explain` is unavailable); the shape
/// invariants on reposix's OWN output are the hard gate.
#[test]
fn rustc_parity_of_shape() {
    // --- Hard gate: reposix's own output shape ---
    let (ok, stdout, stderr) = run_explain(&["RPX-0900"]);
    assert!(
        ok,
        "`reposix explain RPX-0900` must exit 0; stderr:\n{stderr}"
    );
    let lines: Vec<&str> = stdout.lines().collect();

    // (1) A code-header line CONTAINING the code.
    let header = lines
        .first()
        .expect("explain output must have a header line");
    assert!(
        header.contains("RPX-0900"),
        "first line must be a code header containing the code; got: `{header}`"
    );
    // (2) A non-empty multi-line body (header + blank + cause + fix + recovery).
    assert!(
        lines.len() >= 5,
        "explain output must be a multi-line body; got {} lines:\n{stdout}",
        lines.len()
    );
    let body_nonempty = lines
        .iter()
        .skip(1)
        .any(|l| !l.trim().is_empty() && !l.starts_with("Fix:"));
    assert!(
        body_nonempty,
        "explain output must have a non-empty extended body; got:\n{stdout}"
    );
    // (3) A fix section.
    assert!(
        stdout.contains("Fix: "),
        "explain output must have a fix section; got:\n{stdout}"
    );

    // --- Best-effort leg: rustc --explain E0308 shares the shape ---
    let Ok(rustc) = std::process::Command::new("rustc")
        .args(["--explain", "E0308"])
        .output()
    else {
        eprintln!("SKIP rustc parity leg: `rustc` not invokable");
        return;
    };
    if !rustc.status.success() {
        eprintln!("SKIP rustc parity leg: `rustc --explain E0308` exited non-zero");
        return;
    }
    let rustc_out = String::from_utf8_lossy(&rustc.stdout);
    // rustc --explain prints a multi-line prose explanation — the same
    // "code header + extended body" shape reposix mirrors. We assert the shared
    // SHAPE (multi-line, non-empty), never byte-diff (brittle across versions).
    assert!(
        rustc_out.lines().filter(|l| !l.trim().is_empty()).count() >= 5,
        "rustc --explain E0308 should be a multi-line explanation (shared shape)"
    );
}
