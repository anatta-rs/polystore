//! `KvStore` trait — bytes in / bytes out.

use crate::error::Result;
use crate::types::Scope;
use async_trait::async_trait;

/// Trait for content-addressable byte storage.
///
/// Each instance is bound to a single multi-tenant [`Scope`] (namespace + repo + branch).
/// Keys are arbitrary strings (typically content hashes or namespaced keys);
/// values are raw byte buffers.
#[async_trait]
pub trait KvStore: Send + Sync {
    /// The tenant scope this store operates on.
    fn scope(&self) -> &Scope;

    /// Insert or replace the value at `key`.
    async fn put(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Fetch the value at `key`, returning `None` if absent.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete the value at `key` (idempotent: deleting a missing key is not an error).
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check whether `key` is present without fetching the value.
    async fn exists(&self, key: &str) -> Result<bool>;
}
