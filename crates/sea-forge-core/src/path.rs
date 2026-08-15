//! Canonical workspace-relative path validation and normalization.
//!
//! One shared primitive for the whole "raw-string path vs normalized
//! filesystem semantics" defect class: every enforcing function (authority
//! deny/allow matching, sandbox materialization, planner validation, and the
//! generated-zone guard) must agree on what a canonical relative path is, so
//! `a//b`, `a/./b`, and `a/` can never alias `a/b`.

use std::path::{Component, Path};

use crate::errors::ForgeError;

/// Validate a workspace-relative path as a single, unambiguous spelling.
///
/// Rejects:
/// - absolute paths and the empty path;
/// - characters outside `[A-Za-z0-9._/-@]` (`@` is used by legitimate
///   template filenames and cannot form an escape);
/// - `..`, root, and prefix components;
/// - `.` segments (e.g. `a/./b`) and empty segments (e.g. `a//b`, leading
///   `/`, trailing `/`) — each is a second spelling of an already-reachable
///   path.
///
/// The single bare `.` is accepted: it is the unambiguous "workspace root"
/// marker used by `ExecuteCommand.cwd`, and it names the root itself rather
/// than an alias of another path.
pub fn validate_relative_path(path: &str) -> Result<(), ForgeError> {
    let value = Path::new(path);
    if value.is_absolute()
        || path.is_empty()
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._/-@".contains(c))
        || value.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ForgeError::UnsafePath(format!(
            "unsafe workspace-relative path: {path}"
        )));
    }
    // `Path::components()` collapses `//` and strips `.` segments, so inspect
    // the raw spelling for the aliasing forms it hides.
    if path != "."
        && path
            .split('/')
            .any(|segment| segment.is_empty() || segment == ".")
    {
        return Err(ForgeError::UnsafePath(format!(
            "ambiguous workspace-relative path: {path}"
        )));
    }
    Ok(())
}

/// Collapse a relative path to its canonical lexical spelling without
/// validating it.
///
/// `a//b`, `a/./b`, `./a`, `a/`, and `/a` all render as `a/b` (or `a`). This
/// is the normalization deny-glob matching must apply so a hard-boundary
/// pattern sees the same spelling the filesystem would resolve to. Callers
/// that need the enforcing-side rejection should use
/// [`validate_relative_path`] first; this function deliberately stays total so
/// matching can normalize any input.
pub fn normalize_relative_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut first = true;
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if !first {
            out.push('/');
        }
        out.push_str(segment);
        first = false;
    }
    out
}

/// True when `prefix` is a path-segment prefix of `path` (not a raw string
/// prefix). `docs` matches `docs` and `docs/x` but not `docs-private/x`.
pub fn path_prefix_matches(path: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if path == prefix {
        return true;
    }
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.starts_with('/'))
}

/// True when `id` is a single safe path segment: non-empty, at most `max_len`
/// bytes, and composed only of `[A-Za-z0-9_-]`. This is the grammar for any
/// caller-supplied identifier that is joined into a filesystem path (ledger
/// ids, case/run ids, template names/versions, draft ids, plan-item ids) so a
/// traversal-shaped id is rejected at the enforcing boundary.
pub fn valid_id_segment(id: &str, max_len: usize) -> bool {
    !id.is_empty()
        && id.len() <= max_len
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_and_bare_dot() {
        assert!(validate_relative_path("ok/model.sea").is_ok());
        assert!(validate_relative_path("name@version.yaml").is_ok());
        assert!(validate_relative_path(".").is_ok());
    }

    #[test]
    fn rejects_traversal_and_ambiguous_spellings() {
        for bad in [
            "../x", "/x", "", "bad path", "a/../b", "a//b", "a/./b", "a/", "/a", "./a",
        ] {
            assert!(
                validate_relative_path(bad).is_err(),
                "{bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn normalization_collapses_aliases() {
        assert_eq!(normalize_relative_path("a//b"), "a/b");
        assert_eq!(normalize_relative_path("a/./b"), "a/b");
        assert_eq!(normalize_relative_path("./a"), "a");
        assert_eq!(normalize_relative_path("a/"), "a");
        assert_eq!(
            normalize_relative_path("src//gen//model.rs"),
            "src/gen/model.rs"
        );
        assert_eq!(normalize_relative_path("subdir/.env"), "subdir/.env");
    }

    #[test]
    fn prefix_is_segment_aware() {
        assert!(path_prefix_matches("docs", "docs"));
        assert!(path_prefix_matches("docs/x", "docs"));
        assert!(!path_prefix_matches("docs-private/x", "docs"));
        assert!(!path_prefix_matches("x", "docs"));
        assert!(path_prefix_matches("anything", ""));
    }

    #[test]
    fn id_segment_grammar() {
        assert!(valid_id_segment("case_20260710T120000Z_ab12cd34", 128));
        assert!(valid_id_segment("self-model", 128));
        assert!(valid_id_segment("name@version", 128) == false);
        assert!(!valid_id_segment("", 128));
        assert!(!valid_id_segment("a/b", 128));
        assert!(!valid_id_segment("../x", 128));
        assert!(!valid_id_segment("a.b", 128));
        assert!(!valid_id_segment("toolong", 3));
    }
}
