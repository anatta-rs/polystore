//! Error types for polystore traits.

use thiserror::Error;

/// Common error type for all polystore operations.
///
/// Backends should wrap their native errors via [`PolystoreError::Backend`].
#[derive(Debug, Error)]
pub enum PolystoreError {
    /// The requested entity was not found.
    #[error("entity not found: {0}")]
    NotFound(String),

    /// A conflict occurred (e.g. duplicate insert).
    #[error("conflict: {0}")]
    Conflict(String),

    /// The provided scope was invalid for this backend.
    #[error("invalid scope: {0}")]
    InvalidScope(String),

    /// Wraps an opaque backend-specific error.
    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// (De)serialization of an entity payload failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Convenience alias used throughout polystore.
pub type Result<T> = std::result::Result<T, PolystoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_displays() {
        let e = PolystoreError::NotFound("foo".to_owned());
        assert_eq!(e.to_string(), "entity not found: foo");
    }

    #[test]
    fn conflict_displays() {
        let e = PolystoreError::Conflict("dup".to_owned());
        assert_eq!(e.to_string(), "conflict: dup");
    }

    #[test]
    fn invalid_scope_displays() {
        let e = PolystoreError::InvalidScope("bad".to_owned());
        assert_eq!(e.to_string(), "invalid scope: bad");
    }

    #[test]
    fn backend_wraps_inner_error() {
        use std::io;
        let inner = io::Error::other("oops");
        let e = PolystoreError::Backend(Box::new(inner));
        let s = e.to_string();
        assert!(s.starts_with("backend error"), "got: {s}");
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn serialization_from_serde_json_error() {
        let bad = serde_json::from_str::<i32>("not a number").unwrap_err();
        let e: PolystoreError = bad.into();
        let s = e.to_string();
        assert!(s.starts_with("serialization error"), "got: {s}");
    }

    #[test]
    fn debug_renders() {
        let e = PolystoreError::NotFound("x".to_owned());
        assert!(format!("{e:?}").contains("NotFound"));
    }

    #[test]
    fn result_alias_carries_value_or_error() {
        fn maybe(success: bool) -> Result<i32> {
            if success {
                Ok(42)
            } else {
                Err(PolystoreError::NotFound("z".to_owned()))
            }
        }
        assert_eq!(maybe(true).expect("ok"), 42);
        assert!(maybe(false).is_err());
    }
}
