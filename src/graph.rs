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

    // ---- Bulk reads (default impls; backends override for one-query speedups) ----

    /// Fetch many nodes at once, preserving input order.
    ///
    /// Missing ids yield `None` slots so callers can correlate by index.
    /// The default implementation calls [`Self::get_node`] sequentially —
    /// backends with bulk fetch (e.g. Cypher `MATCH WHERE id IN [...]`) should
    /// override for a single round-trip.
    async fn get_nodes_bulk(&self, ids: &[EntityId]) -> Result<Vec<Option<N>>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push(self.get_node(id).await?);
        }
        Ok(out)
    }

    /// List all entity ids whose `kind` is one of `kinds`, grouped by kind.
    ///
    /// The default implementation calls [`Self::list_by_kind`] per kind —
    /// backends should override with one query per call (`WHERE kind IN [...]`).
    async fn list_by_kinds(&self, kinds: &[&str]) -> Result<Vec<(String, Vec<EntityId>)>> {
        let mut out = Vec::with_capacity(kinds.len());
        for k in kinds {
            out.push(((*k).to_owned(), self.list_by_kind(k).await?));
        }
        Ok(out)
    }

    /// Walk neighbors of many nodes at once, returning per-source results in input order.
    ///
    /// The default implementation calls [`Self::neighbors`] sequentially —
    /// backends should override with a single graph traversal that filters
    /// by the source-id set.
    async fn neighbors_bulk(
        &self,
        ids: &[EntityId],
        direction: Direction,
    ) -> Result<Vec<(EntityId, Vec<(EntityId, E)>)>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push((id.clone(), self.neighbors(id, direction).await?));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Minimal in-test `GraphStore` that exercises the default bulk impls.
    /// Counts each base call so tests can assert the default implementations
    /// dispatch through the singular methods as documented.
    struct CountingStore {
        scope: Scope,
        nodes: Mutex<HashMap<EntityId, String>>,
        out_edges: Mutex<HashMap<EntityId, Vec<(EntityId, ())>>>,
        kinds: Mutex<HashMap<String, Vec<EntityId>>>,
        get_node_calls: Mutex<usize>,
        list_by_kind_calls: Mutex<usize>,
        neighbors_calls: Mutex<usize>,
    }

    impl CountingStore {
        fn new() -> Self {
            Self {
                scope: Scope::new("ns", "repo", "branch"),
                nodes: Mutex::new(HashMap::new()),
                out_edges: Mutex::new(HashMap::new()),
                kinds: Mutex::new(HashMap::new()),
                get_node_calls: Mutex::new(0),
                list_by_kind_calls: Mutex::new(0),
                neighbors_calls: Mutex::new(0),
            }
        }
    }

    #[async_trait]
    impl GraphStore<String, ()> for CountingStore {
        fn scope(&self) -> &Scope {
            &self.scope
        }

        async fn upsert_node(&self, id: &EntityId, node: String) -> Result<()> {
            self.nodes.lock().expect("lock").insert(id.clone(), node);
            Ok(())
        }

        async fn get_node(&self, id: &EntityId) -> Result<Option<String>> {
            *self.get_node_calls.lock().expect("lock") += 1;
            Ok(self.nodes.lock().expect("lock").get(id).cloned())
        }

        async fn delete_node(&self, id: &EntityId) -> Result<()> {
            self.nodes.lock().expect("lock").remove(id);
            Ok(())
        }

        async fn add_edge(&self, from: &EntityId, to: &EntityId, edge: ()) -> Result<()> {
            self.out_edges
                .lock()
                .expect("lock")
                .entry(from.clone())
                .or_default()
                .push((to.clone(), edge));
            Ok(())
        }

        async fn neighbors(
            &self,
            id: &EntityId,
            _direction: Direction,
        ) -> Result<Vec<(EntityId, ())>> {
            *self.neighbors_calls.lock().expect("lock") += 1;
            Ok(self
                .out_edges
                .lock()
                .expect("lock")
                .get(id)
                .cloned()
                .unwrap_or_default())
        }

        async fn list_by_kind(&self, kind: &str) -> Result<Vec<EntityId>> {
            *self.list_by_kind_calls.lock().expect("lock") += 1;
            Ok(self
                .kinds
                .lock()
                .expect("lock")
                .get(kind)
                .cloned()
                .unwrap_or_default())
        }

        async fn search_by_name(
            &self,
            _query: &str,
            _top_k: usize,
        ) -> Result<Vec<(EntityId, String)>> {
            Ok(vec![])
        }

        async fn reverse_path(&self, _from: &EntityId, _hops: u8) -> Result<Vec<Vec<EntityId>>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn get_nodes_bulk_preserves_order_and_marks_missing() {
        let store = CountingStore::new();
        let a = EntityId::new("a");
        let b = EntityId::new("b");
        let missing = EntityId::new("ghost");
        store.upsert_node(&a, "A".into()).await.expect("upsert");
        store.upsert_node(&b, "B".into()).await.expect("upsert");

        let got = store
            .get_nodes_bulk(&[a.clone(), missing.clone(), b.clone()])
            .await
            .expect("bulk");

        assert_eq!(got, vec![Some("A".into()), None, Some("B".into())]);
        assert_eq!(*store.get_node_calls.lock().expect("lock"), 3);
    }

    #[tokio::test]
    async fn list_by_kinds_groups_per_kind_in_input_order() {
        let store = CountingStore::new();
        store.kinds.lock().expect("lock").insert(
            "function".into(),
            vec![EntityId::new("f1"), EntityId::new("f2")],
        );
        store
            .kinds
            .lock()
            .expect("lock")
            .insert("module".into(), vec![EntityId::new("m1")]);

        let got = store
            .list_by_kinds(&["function", "module", "absent"])
            .await
            .expect("kinds");

        assert_eq!(got.len(), 3);
        assert_eq!(got[0].0, "function");
        assert_eq!(got[0].1.len(), 2);
        assert_eq!(got[1].0, "module");
        assert_eq!(got[1].1.len(), 1);
        assert_eq!(got[2].0, "absent");
        assert!(got[2].1.is_empty());
        assert_eq!(*store.list_by_kind_calls.lock().expect("lock"), 3);
    }

    #[tokio::test]
    async fn neighbors_bulk_returns_per_source_in_input_order() {
        let store = CountingStore::new();
        let a = EntityId::new("a");
        let b = EntityId::new("b");
        let c = EntityId::new("c");
        store.upsert_node(&a, "A".into()).await.expect("upsert");
        store.upsert_node(&b, "B".into()).await.expect("upsert");
        store.upsert_node(&c, "C".into()).await.expect("upsert");
        store.add_edge(&a, &b, ()).await.expect("edge");
        store.add_edge(&a, &c, ()).await.expect("edge");
        store.add_edge(&b, &c, ()).await.expect("edge");

        let got = store
            .neighbors_bulk(&[a.clone(), b.clone(), c.clone()], Direction::Outgoing)
            .await
            .expect("bulk");

        assert_eq!(got.len(), 3);
        assert_eq!(got[0].0, a);
        assert_eq!(got[0].1.len(), 2);
        assert_eq!(got[1].0, b);
        assert_eq!(got[1].1.len(), 1);
        assert_eq!(got[2].0, c);
        assert!(got[2].1.is_empty());
        assert_eq!(*store.neighbors_calls.lock().expect("lock"), 3);
    }
}
