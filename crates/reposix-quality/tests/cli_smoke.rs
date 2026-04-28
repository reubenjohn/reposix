//! Smoke test: `--help` for top-level + `doc-alignment` subcommand.
//!
//! Asserts the surface defined by:
//!   .planning/research/v0.12.0-docs-alignment-design/02-architecture.md
//!   .claude/skills/reposix-quality-doc-alignment/prompts/extractor.md

use assert_cmd::Command;

#[test]
fn top_level_help_lists_all_subcommands() {
    let out = Command::cargo_bin("reposix-quality")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).to_string();
    for needle in ["doc-alignment", "run", "verify", "walk"] {
        assert!(
            stdout.contains(needle),
            "top-level --help missing `{needle}` -- got:\n{stdout}"
        );
    }
}

#[test]
fn doc_alignment_help_lists_all_verbs() {
    let out = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["doc-alignment", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).to_string();
    for needle in [
        "bind",
        "propose-retire",
        "confirm-retire",
        "mark-missing-test",
        "plan-refresh",
        "plan-backfill",
        "merge-shards",
        "walk",
        "status",
    ] {
        assert!(
            stdout.contains(needle),
            "doc-alignment --help missing `{needle}` -- got:\n{stdout}"
        );
    }
}

#[test]
fn version_prints() {
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}
