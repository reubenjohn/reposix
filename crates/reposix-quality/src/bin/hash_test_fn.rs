//! Standalone `hash_test_fn` binary -- shares `reposix_quality::hash` with
//! the umbrella binary's in-process hashing path.
//!
//! Usage:
//!   `hash_test_fn` --file <path> --fn <name>
//!
//! Prints the hex sha256 of the named fn's `to_token_stream().to_string()`
//! to stdout on success; exits 1 with a stderr message on miss.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use reposix_quality::hash;

#[derive(Debug, Parser)]
#[command(
    name = "hash_test_fn",
    version,
    about = "Hash a Rust fn body to sha256."
)]
struct Cli {
    /// Path to the Rust source file.
    #[arg(long)]
    file: PathBuf,

    /// Simple fn name (free fn or impl-block method).
    #[arg(long = "fn")]
    fn_name: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match hash::test_body_hash(&cli.file, &cli.fn_name) {
        Ok(h) => {
            println!("{h}");
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("hash_test_fn: {e:#}");
            ExitCode::from(1)
        }
    }
}
