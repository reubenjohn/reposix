#!/usr/bin/env bash
# Phase 3 exit check — union of ROADMAP Phase 3 SC #1–#5 plus the ambient
# CLAUDE.md constraints (no unsafe, no direct reqwest ctor, AllowOther OFF,
# filename validator on the FUSE boundary). Exits 0 iff all pass.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> cargo fmt --all --check"
cargo fmt --all --check

echo "==> cargo clippy -p reposix-core -p reposix-fuse -p reposix-cli --all-targets -- -D warnings"
cargo clippy -p reposix-core -p reposix-fuse -p reposix-cli --all-targets -- -D warnings

echo "==> cargo test -p reposix-core -p reposix-fuse -p reposix-cli"
cargo test -p reposix-core -p reposix-fuse -p reposix-cli

echo "==> cargo test --release -- --ignored --test-threads=1 (sim_death + demo)"
cargo test -p reposix-fuse -p reposix-cli --release -- --ignored --test-threads=1

echo "==> guard: no direct reqwest ctor in fuse/cli"
test "$(grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' \
        crates/reposix-fuse/ crates/reposix-cli/ --include='*.rs' | \
        grep -v 'crates/reposix-core/src/http.rs' | wc -l)" = "0"

echo "==> guard: validate_issue_filename used in fuse fs.rs"
grep -q 'validate_issue_filename' crates/reposix-fuse/src/fs.rs

echo "==> guard: AllowOther not present in fuse"
test "$(grep -RIn 'AllowOther' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0"

echo "==> guard: reposix --help lists sim/mount/demo"
./target/debug/reposix --help | grep -q '\bsim\b'
./target/debug/reposix --help | grep -q '\bmount\b'
./target/debug/reposix --help | grep -q '\bdemo\b'

echo "ALL PASS"
