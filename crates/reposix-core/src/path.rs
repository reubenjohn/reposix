//! Filename + path component validators (SG-04) and slug helpers.
//!
//! We centralise these checks in `reposix-core` so the cache materializer and
//! remote helper plug into the exact same grammar the simulator assumes.
//! `std::path::Path::file_name` is NOT sufficient: it normalises `..` on some
//! platforms and does not reject embedded `\0`. Hand-rolled validators are
//! 10 lines and centrally tested.
//!
//! ## Slug helpers
//!
//! [`slugify_title`], [`slug_or_fallback`], and [`dedupe_siblings`] synthesize
//! human-readable directory/file names from free-form issue/page titles for
//! the cache-materialized tree. These are deterministic pure functions; no
//! dependency on backend state.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::record::RecordId;

/// Maximum slug length in bytes.
///
/// Chosen to fit ext4 `NAME_MAX` (255) with ~190 bytes of headroom for
/// collision suffixes (`-NN`) and for the kernel's own bookkeeping.
pub const SLUG_MAX_BYTES: usize = 60;

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
/// [`RecordId`].
///
/// Rejects everything else, including paths with a directory separator,
/// hidden files, files without the `.md` extension, and trailing whitespace.
/// Leading zeros are accepted (`"00042.md"` → `RecordId(42)`).
///
/// # Errors
/// Returns [`Error::InvalidPath`] if the name is not strictly `[0-9]+\.md`.
pub fn validate_record_filename(name: &str) -> Result<RecordId> {
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
    Ok(RecordId(n))
}

/// Convert a free-form title to a filesystem-safe slug.
///
/// Algorithm (locked by Phase 13 CONTEXT.md §slug-algorithm):
/// 1. Unicode-lowercase via [`str::to_lowercase`].
/// 2. Replace every run of non-`[a-z0-9]` (including all non-ASCII bytes,
///    which necessarily fall outside `[a-z0-9]`) with a single `-`.
/// 3. Trim leading/trailing `-`.
/// 4. Byte-truncate to [`SLUG_MAX_BYTES`] on a UTF-8 char boundary (the
///    output is pure ASCII so every byte is already a boundary; the
///    check is retained defensively).
/// 5. Re-trim trailing `-` after truncation.
///
/// This function is pure and `#[must_use]`. It does NOT fall back on empty
/// results — callers that need a guaranteed-nonempty slug use
/// [`slug_or_fallback`].
///
/// # Security
///
/// Mitigates T-13-01 + T-13-02 from the Phase 13 threat model: the output
/// cannot contain `/`, `.`, `..`, `\0`, shell metacharacters, or any byte
/// outside `[a-z0-9-]`. Proven by the
/// `slug_is_ascii_alnum_dash_only_over_adversarial_inputs` test.
#[must_use]
pub fn slugify_title(title: &str) -> String {
    // IN-02: pre-cap intermediate allocation. Any input beyond ~240 chars
    // is guaranteed to be sliced off before it reaches the output (which
    // is capped at SLUG_MAX_BYTES = 60). 4x headroom covers worst-case
    // lowercase expansion (some codepoints lowercase into multi-byte
    // sequences). Defense against pathological 10MB titles.
    let trimmed_input: String = title.chars().take(SLUG_MAX_BYTES * 4).collect();
    let lower = trimmed_input.to_lowercase();
    let mut out = String::with_capacity(lower.len().min(SLUG_MAX_BYTES * 2));
    // `last_was_dash` starts true so leading non-alnum runs are elided.
    let mut last_was_dash = true;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.len() > SLUG_MAX_BYTES {
        // Output is pure ASCII; every byte is already a char boundary. The
        // is_char_boundary check is retained for defense-in-depth in case
        // the algorithm ever changes to allow non-ASCII alphanumerics.
        let mut end = SLUG_MAX_BYTES;
        while !out.is_char_boundary(end) {
            end -= 1;
        }
        out.truncate(end);
        while out.ends_with('-') {
            out.pop();
        }
    }
    out
}

/// [`slugify_title`] with a guaranteed non-empty, non-reserved result.
///
/// Falls back to `page-<11-digit-padded-id>` when the slug would be empty,
/// all dashes, `.`, or `..`. The 11-digit padding matches the existing
/// `<padded-id>.md` convention in `pages/` and `issues/`.
#[must_use]
pub fn slug_or_fallback(title: &str, id: RecordId) -> String {
    let s = slugify_title(title);
    if s.is_empty() || s == "." || s == ".." || s.chars().all(|c| c == '-') {
        format!("page-{:011}", id.0)
    } else {
        s
    }
}

/// Group siblings under one parent by slug, then assign unique suffixes.
///
/// - Input: vec of `(RecordId, slug)` pairs. Input order is not assumed.
/// - Output: same length as input. For each group of colliding slugs,
///   entries are sorted ascending by [`RecordId`]; the first keeps the bare
///   slug, the `N`th (`N >= 2`) gets suffix `-N`.
/// - Deterministic: same input always produces same output across mounts.
/// - Preserves all input entries (no silent drops).
///
/// The returned vec is in ascending [`RecordId`] order so callers who want
/// a stable render order get it for free.
#[must_use]
pub fn dedupe_siblings(mut siblings: Vec<(RecordId, String)>) -> Vec<(RecordId, String)> {
    // Ascending RecordId is the tie-break: smaller ids keep the bare slug.
    siblings.sort_by_key(|(id, _)| *id);
    let mut seen: HashMap<String, u32> = HashMap::new();
    let mut out = Vec::with_capacity(siblings.len());
    for (id, slug) in siblings {
        let n = seen.entry(slug.clone()).or_insert(0);
        *n += 1;
        let final_slug = if *n == 1 { slug } else { format!("{slug}-{n}") };
        out.push((id, final_slug));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_issue_filename_accepts_digits_md() {
        assert_eq!(validate_record_filename("0.md").unwrap().0, 0);
        assert_eq!(validate_record_filename("123.md").unwrap().0, 123);
        assert_eq!(validate_record_filename("00042.md").unwrap().0, 42);
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
            let err = validate_record_filename(bad).unwrap_err();
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
            let err2 = validate_record_filename(bad);
            assert!(
                err.is_err() || err2.is_err(),
                "{bad:?} must be rejected by at least one validator"
            );
        }
        // Specifically, `..` must fail both.
        assert!(validate_path_component("..").is_err());
        assert!(validate_record_filename("../123.md").is_err());
    }

    #[test]
    fn validate_issue_filename_rejects_junk() {
        for bad in [
            "", ".md", "..md", "123", "123.txt", "123.md/", "/123.md", "\0.md", "123.md\n", ".",
            "..",
        ] {
            let err = validate_record_filename(bad).unwrap_err();
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
        let err = validate_record_filename(too_big).unwrap_err();
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

    // --- Phase 13: slugify_title --------------------------------------------

    #[test]
    fn slug_simple_ascii() {
        assert_eq!(slugify_title("Hello, World!"), "hello-world");
    }

    #[test]
    fn slug_strips_leading_and_trailing_whitespace() {
        assert_eq!(
            slugify_title("  leading and trailing  "),
            "leading-and-trailing"
        );
    }

    #[test]
    fn slug_collapses_multiple_spaces() {
        assert_eq!(slugify_title("multiple   spaces"), "multiple-spaces");
    }

    #[test]
    fn slug_welcome_to_reposix() {
        assert_eq!(slugify_title("Welcome to reposix"), "welcome-to-reposix");
    }

    #[test]
    fn slug_empty_input_is_empty() {
        assert_eq!(slugify_title(""), "");
    }

    #[test]
    fn slug_full_multibyte_is_empty() {
        // CJK codepoints are outside [a-z0-9] → all stripped.
        assert_eq!(slugify_title("日本語"), "");
    }

    #[test]
    fn slug_emoji_stripped_alnum_preserved() {
        assert_eq!(slugify_title("🚀 Rocket"), "rocket");
    }

    #[test]
    fn slug_all_dashes_is_empty() {
        assert_eq!(slugify_title("---"), "");
    }

    #[test]
    fn slug_single_dot_is_empty() {
        assert_eq!(slugify_title("."), "");
    }

    #[test]
    fn slug_double_dot_is_empty() {
        assert_eq!(slugify_title(".."), "");
    }

    #[test]
    fn slug_respects_max_bytes_and_const_is_60() {
        assert_eq!(SLUG_MAX_BYTES, 60);
        let long = "A".repeat(100);
        let s = slugify_title(&long);
        assert!(
            s.len() <= SLUG_MAX_BYTES,
            "slug length {} exceeded SLUG_MAX_BYTES {}",
            s.len(),
            SLUG_MAX_BYTES
        );
        // All-alpha slug with no embedded separators: the whole prefix is a
        // run of 'a', so truncation yields exactly SLUG_MAX_BYTES 'a's with
        // no trailing dash to trim.
        assert_eq!(s.len(), SLUG_MAX_BYTES);
        assert!(s.chars().all(|c| c == 'a'));
    }

    #[test]
    fn slug_non_ascii_alphanumeric_becomes_separator() {
        // `é` (U+00E9) is non-ASCII → run of `-` → collapses to one `-`,
        // which then trims at the edges. Expected: `"a-b"`.
        assert_eq!(slugify_title("a-\u{00e9}-b"), "a-b");
    }

    #[test]
    fn slug_truncation_is_char_boundary_safe_on_long_alpha() {
        // Pure ASCII input; this is mostly a "does not panic" check under
        // the is_char_boundary tightening loop.
        let long = "A".repeat(100);
        let _ = slugify_title(&long);
    }

    #[test]
    fn slug_truncation_trims_trailing_dash_after_cut() {
        // Construct an input where byte 60 would land on a dash: 59 'a's +
        // '-' + more chars → truncate to 60 puts '-' at index 59, post-trim
        // removes it.
        let input = format!("{}-tail", "a".repeat(59));
        let s = slugify_title(&input);
        assert!(!s.ends_with('-'), "slug must not end with '-': {s:?}");
        assert!(s.len() <= SLUG_MAX_BYTES);
    }

    #[test]
    fn slug_is_ascii_alnum_dash_only_over_adversarial_inputs() {
        // T-13-01 + T-13-02 mitigation proof. Any output byte outside
        // [a-z0-9-] would be a tampering success; any slug equal to "." or
        // ".." would let a caller escape the mount root.
        let adversarial = [
            "../../../etc/passwd",
            "foo/bar",
            "foo\0bar",
            "$(rm -rf /)",
            "`whoami`",
            "hello;ls",
            "\u{202e}reverse", // Unicode right-to-left override
            "tab\there",
        ];
        for input in adversarial {
            let s = slugify_title(input);
            assert!(
                s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "slugify_title({input:?}) = {s:?} contains forbidden char"
            );
            assert!(
                s != "." && s != "..",
                "slug must not be '.' or '..'; got {s:?} from {input:?}"
            );
            assert!(!s.contains('/'), "slug must not contain '/'; got {s:?}");
            assert!(!s.contains('\0'), "slug must not contain NUL; got {s:?}");
        }
    }

    // --- Phase 13: slug_or_fallback -----------------------------------------

    #[test]
    fn fallback_on_empty_input() {
        assert_eq!(slug_or_fallback("", RecordId(42)), "page-00000000042");
    }

    #[test]
    fn fallback_on_all_dashes() {
        assert_eq!(slug_or_fallback("---", RecordId(7)), "page-00000000007");
    }

    #[test]
    fn fallback_on_double_dot() {
        assert_eq!(slug_or_fallback("..", RecordId(3)), "page-00000000003");
    }

    #[test]
    fn fallback_on_all_multibyte() {
        assert_eq!(
            slug_or_fallback("日本語", RecordId(100)),
            "page-00000000100"
        );
    }

    #[test]
    fn fallback_passthrough_for_nonempty_slug() {
        assert_eq!(slug_or_fallback("Welcome", RecordId(1)), "welcome");
    }

    // --- Phase 13: dedupe_siblings -----------------------------------------

    #[test]
    fn dedupe_assigns_suffix_to_lower_id_first() {
        let input = vec![
            (RecordId(5), "foo".to_owned()),
            (RecordId(3), "foo".to_owned()),
            (RecordId(4), "bar".to_owned()),
        ];
        let got = dedupe_siblings(input);
        assert_eq!(
            got,
            vec![
                (RecordId(3), "foo".to_owned()),
                (RecordId(4), "bar".to_owned()),
                (RecordId(5), "foo-2".to_owned()),
            ]
        );
    }

    #[test]
    fn dedupe_three_colliders_get_ascending_suffixes() {
        let input = vec![
            (RecordId(30), "same".to_owned()),
            (RecordId(10), "same".to_owned()),
            (RecordId(20), "same".to_owned()),
        ];
        let got = dedupe_siblings(input);
        assert_eq!(
            got,
            vec![
                (RecordId(10), "same".to_owned()),
                (RecordId(20), "same-2".to_owned()),
                (RecordId(30), "same-3".to_owned()),
            ]
        );
    }

    #[test]
    fn dedupe_empty_input_is_empty() {
        let got = dedupe_siblings(Vec::new());
        assert!(got.is_empty());
    }

    #[test]
    fn dedupe_preserves_all_entries() {
        let input: Vec<(RecordId, String)> = (1..=10)
            .map(|i| (RecordId(i), format!("slug-{}", i % 3)))
            .collect();
        let got = dedupe_siblings(input.clone());
        assert_eq!(got.len(), input.len(), "no entries may be dropped");
        // Every input id must appear in output exactly once.
        let mut ids: Vec<u64> = got.iter().map(|(id, _)| id.0).collect();
        ids.sort_unstable();
        assert_eq!(ids, (1..=10).collect::<Vec<_>>());
    }

    #[test]
    fn dedupe_is_deterministic() {
        let input = vec![
            (RecordId(2), "foo".to_owned()),
            (RecordId(1), "foo".to_owned()),
            (RecordId(4), "bar".to_owned()),
            (RecordId(3), "foo".to_owned()),
        ];
        let a = dedupe_siblings(input.clone());
        let b = dedupe_siblings(input);
        assert_eq!(a, b, "dedupe must be deterministic");
    }
}
