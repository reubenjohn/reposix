//! Prints the audit-log schema DDL to stdout.
//!
//! Used by ROADMAP phase-1 SC #3:
//!   `cargo run -q -p reposix-core --example show_audit_schema`
//! must emit DDL containing the `audit_no_update` / `audit_no_delete` triggers.

fn main() {
    print!("{}", reposix_core::audit::SCHEMA_SQL);
}
