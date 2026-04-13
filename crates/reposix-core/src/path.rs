//! Filename + path component validators (SG-04).
//!
//! We centralise these checks in `reposix-core` so the FUSE boundary (Phase 3)
//! plugs into the exact same grammar the simulator and remote helper assume.
//! `std::path::Path::file_name` is NOT sufficient: it normalises `..` on some
//! platforms and does not reject embedded `\0`. Hand-rolled validators are
//! 10 lines and centrally tested.

use crate::error::{Error, Result};
use crate::issue::IssueId;

/// Validate a single path component.
///
/// Rejects empty, `.`, `..`, and anything containing `/` or `\0`.
///
/// # Errors
/// Returns [`Error::InvalidPath`] with the offending input echoed back.
pub fn validate_path_component(name: &str) -> Result<&str> {
    if name.is_empty() {
        return Err(Error::InvalidPath(String::from("<empty>")));
    }
    if name == "." || name == ".." {
        return Err(Error::InvalidPath(name.to_owned()));
    }
    for b in name.bytes() {
        if b == b'/' || b == 0 {
            return Err(Error::InvalidPath(name.to_owned()));
        }
    }
    Ok(name)
}

/// Validate a filename of the form `<digits>.md` and return the parsed
/// [`IssueId`].
///
/// Rejects everything else, including paths with a directory separator,
/// hidden files, files without the `.md` extension, and trailing whitespace.
/// Leading zeros are accepted (`"00042.md"` → `IssueId(42)`).
///
/// # Errors
/// Returns [`Error::InvalidPath`] if the name is not strictly `[0-9]+\.md`.
pub fn validate_issue_filename(name: &str) -> Result<IssueId> {
    validate_path_component(name)?;
    let prefix = name
        .strip_suffix(".md")
        .ok_or_else(|| Error::InvalidPath(name.to_owned()))?;
    if prefix.is_empty() {
        return Err(Error::InvalidPath(name.to_owned()));
    }
    if !prefix.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidPath(name.to_owned()));
    }
    let n = prefix
        .parse::<u64>()
        .map_err(|_| Error::InvalidPath(name.to_owned()))?;
    Ok(IssueId(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_issue_filename_accepts_digits_md() {
        assert_eq!(validate_issue_filename("0.md").unwrap().0, 0);
        assert_eq!(validate_issue_filename("123.md").unwrap().0, 123);
        assert_eq!(validate_issue_filename("00042.md").unwrap().0, 42);
    }

    #[test]
    fn filename_is_id_derived_not_title_derived() {
        // Titles-as-filenames must all be rejected — SG-04 / T-01-10.
        for bad in [
            "../etc/passwd.md",
            "my bug.md",
            "thing is broken.md",
            "readme.md",
            "abc.md",
            "12a.md",
        ] {
            let err = validate_issue_filename(bad).unwrap_err();
            assert!(
                matches!(err, Error::InvalidPath(_)),
                "{bad:?} must be rejected; got {err:?}"
            );
        }
    }

    #[test]
    fn path_with_dotdot_or_nul_is_rejected() {
        // Every way to sneak a `..`, `/`, or NUL past the validator must fail.
        for bad in [
            "..", ".", "", "a/b", "a\0b", "/", "\0", "/123.md", "123.md/", "\0.md",
        ] {
            let err = validate_path_component(bad);
            let err2 = validate_issue_filename(bad);
            assert!(
                err.is_err() || err2.is_err(),
                "{bad:?} must be rejected by at least one validator"
            );
        }
        // Specifically, `..` must fail both.
        assert!(validate_path_component("..").is_err());
        assert!(validate_issue_filename("../123.md").is_err());
    }

    #[test]
    fn validate_issue_filename_rejects_junk() {
        for bad in [
            "",
            ".md",
            "..md",
            "123",
            "123.txt",
            "123.md/",
            "/123.md",
            "\0.md",
            "123.md\n",
            ".",
            "..",
        ] {
            let err = validate_issue_filename(bad).unwrap_err();
            assert!(
                matches!(err, Error::InvalidPath(_)),
                "{bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn validate_issue_filename_rejects_overflow() {
        // Digit string larger than u64::MAX must fail cleanly with InvalidPath,
        // not panic.
        let too_big = "999999999999999999999999999999.md";
        let err = validate_issue_filename(too_big).unwrap_err();
        assert!(matches!(err, Error::InvalidPath(_)));
    }

    #[test]
    fn validate_path_component_accepts_normal() {
        assert_eq!(validate_path_component("foo").unwrap(), "foo");
        assert_eq!(validate_path_component("foo.md").unwrap(), "foo.md");
        assert_eq!(validate_path_component("issue-123").unwrap(), "issue-123");
    }

    #[test]
    fn validate_path_component_rejects_danger() {
        for bad in ["", ".", "..", "a/b", "a\0b", "/", "\0"] {
            assert!(
                validate_path_component(bad).is_err(),
                "{bad:?} must be rejected"
            );
        }
    }
}
