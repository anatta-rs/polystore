//! Core types shared across all polystore traits.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable, opaque identifier for an entity.
///
/// Backends are free to choose any string-based ID scheme (UUIDs, content hashes,
/// `kind:uuid` prefixes, etc.); polystore treats the value as opaque.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(String);

impl EntityId {
    /// Create a new `EntityId` from any string-like value.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the `EntityId` and return the underlying `String`.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for EntityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EntityId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Multi-tenant scope used to partition data along the
/// `Namespace → Repo → Branch` hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    /// Top-level org / user namespace (GitHub-orgs style).
    pub namespace: String,
    /// Repository name within the namespace.
    pub repo: String,
    /// Branch name within the repository.
    pub branch: String,
}

impl Scope {
    /// Construct a new scope from its three components.
    #[must_use]
    pub fn new(
        namespace: impl Into<String>,
        repo: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            repo: repo.into(),
            branch: branch.into(),
        }
    }

    /// Render the scope as a `namespace/repo/branch` path-like string,
    /// suitable for use as a directory key.
    #[must_use]
    pub fn as_path(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.repo, self.branch)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.namespace, self.repo, self.branch)
    }
}

/// Direction for edge traversal in graph queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Edges going FROM the entity (entity → other).
    Outgoing,
    /// Edges going TO the entity (other → entity).
    Incoming,
    /// Both directions.
    Both,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_constructs_and_displays() {
        let id = EntityId::new("foo:bar");
        assert_eq!(id.as_str(), "foo:bar");
        assert_eq!(id.to_string(), "foo:bar");
        assert_eq!(format!("{id:?}"), "EntityId(\"foo:bar\")");
    }

    #[test]
    fn entity_id_from_str_and_string() {
        let from_str: EntityId = "abc".into();
        let from_string: EntityId = String::from("abc").into();
        assert_eq!(from_str, from_string);
    }

    #[test]
    fn entity_id_into_string() {
        let id = EntityId::new("xyz");
        assert_eq!(id.into_string(), "xyz");
    }

    #[test]
    fn entity_id_clone_and_equality() {
        let a = EntityId::new("k");
        let b = a.clone();
        let c = EntityId::new("other");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn entity_id_hash_in_set() {
        use std::collections::HashSet;
        let a = EntityId::new("k");
        let b = EntityId::new("k");
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn entity_id_serde_roundtrip() {
        let id = EntityId::new("abc");
        let json = serde_json::to_string(&id).expect("serialize");
        let back: EntityId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, back);
    }

    #[test]
    fn scope_constructs_and_displays() {
        let s = Scope::new("ns", "repo", "branch");
        assert_eq!(s.namespace, "ns");
        assert_eq!(s.repo, "repo");
        assert_eq!(s.branch, "branch");
        assert_eq!(s.to_string(), "ns/repo/branch");
        assert_eq!(s.as_path(), "ns/repo/branch");
    }

    #[test]
    fn scope_clone_and_equality() {
        let a = Scope::new("n", "r", "b");
        let b = a.clone();
        let c = Scope::new("n", "r", "other");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn scope_hash_in_set() {
        use std::collections::HashSet;
        let a = Scope::new("n", "r", "b");
        let b = Scope::new("n", "r", "b");
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn scope_serde_roundtrip() {
        let s = Scope::new("a", "b", "c");
        let json = serde_json::to_string(&s).expect("serialize");
        let back: Scope = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s, back);
    }

    #[test]
    fn direction_variants_and_equality() {
        assert_eq!(Direction::Outgoing, Direction::Outgoing);
        assert_ne!(Direction::Outgoing, Direction::Incoming);
        assert_ne!(Direction::Outgoing, Direction::Both);
        assert_ne!(Direction::Incoming, Direction::Both);
    }

    #[test]
    fn direction_clone_and_copy() {
        let d = Direction::Both;
        let c = d;
        let cloned = d;
        assert_eq!(d, c);
        assert_eq!(d, cloned);
    }

    #[test]
    fn direction_serde_roundtrip() {
        for d in [Direction::Outgoing, Direction::Incoming, Direction::Both] {
            let json = serde_json::to_string(&d).expect("serialize");
            let back: Direction = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(d, back);
        }
    }

    #[test]
    fn direction_hash_in_set() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Direction::Outgoing);
        set.insert(Direction::Outgoing);
        assert_eq!(set.len(), 1);
        set.insert(Direction::Incoming);
        assert_eq!(set.len(), 2);
    }
}
