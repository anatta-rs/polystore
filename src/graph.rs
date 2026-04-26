//! `GraphStore` trait — typed nodes + edges with structural queries.

use crate::error::Result;
use crate::types::{Direction, EntityId, Scope};
use async_trait::async_trait;

/// Trait for typed graph storage (nodes + edges).
///
/// Generic over the node payload `N` and edge payload `E`. Backends may add
/// their own bounds (e.g. `Serialize + DeserializeOwned`) on their impl blocks.
///
/// Each instance is bound to a single multi-tenant [`Scope`] (namespace + repo + branch).
#[async_trait]
pub trait GraphStore<N, E>: Send + Sync
where
    N: Send + Sync,
    E: Send + Sync,
{
    /// The tenant scope this store operates on.
    fn scope(&self) -> &Scope;

    // ---- Node CRUD ----

    /// Insert or replace a node identified by `id`.
    async fn upsert_node(&self, id: &EntityId, node: N) -> Result<()>;

    /// Fetch a node by id, returning `None` if absent.
    async fn get_node(&self, id: &EntityId) -> Result<Option<N>>;

    /// Delete a node by id (idempotent: deleting a missing id is not an error).
    async fn delete_node(&self, id: &EntityId) -> Result<()>;

    // ---- Edge CRUD ----

    /// Add an edge from `from` to `to` with payload `edge`.
    async fn add_edge(&self, from: &EntityId, to: &EntityId, edge: E) -> Result<()>;

    /// List the neighbors of `id` in the given direction, with the edge payload.
    async fn neighbors(&self, id: &EntityId, direction: Direction) -> Result<Vec<(EntityId, E)>>;

    // ---- Queries ----

    /// List all entity ids whose `kind` (backend-defined) matches `kind`.
    async fn list_by_kind(&self, kind: &str) -> Result<Vec<EntityId>>;

    /// Search for nodes whose name (backend-defined) matches `query`,
    /// returning at most `top_k` results.
    async fn search_by_name(&self, query: &str, top_k: usize) -> Result<Vec<(EntityId, N)>>;

    /// Walk reverse edges (incoming) from `from` for up to `hops` steps,
    /// returning every distinct path discovered.
    async fn reverse_path(&self, from: &EntityId, hops: u8) -> Result<Vec<Vec<EntityId>>>;
}
