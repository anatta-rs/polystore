//! `VectorStore` trait — embeddings + similarity k-NN.

use crate::error::Result;
use crate::types::{EntityId, Scope};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// One result from a similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHit {
    /// The id of the matched entity.
    pub id: EntityId,
    /// The similarity score (backend-defined; typically cosine in `[-1, 1]`
    /// or distance in `[0, ∞)`).
    pub score: f32,
    /// Arbitrary backend-stored payload (filterable / projectable).
    pub payload: serde_json::Value,
}

/// Trait for vector storage with similarity search.
///
/// Each instance is bound to a single multi-tenant [`Scope`] (namespace + repo + branch).
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// The tenant scope this store operates on.
    fn scope(&self) -> &Scope;

    /// Insert or replace the vector for `id`, with attached `payload`.
    async fn upsert(
        &self,
        id: &EntityId,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> Result<()>;

    /// Insert or replace a batch of vectors. Default impl loops
    /// [`Self::upsert`] — fine for in-memory / test backends. Production
    /// backends (sigil-api HTTP, Qdrant client, sigil engine direct)
    /// override with a real batch (single HTTP body, single rebuild
    /// trigger at the tail) for ~10-100× throughput on bulk ingest.
    /// Returns the number of items persisted.
    async fn upsert_batch(
        &self,
        items: Vec<(EntityId, Vec<f32>, serde_json::Value)>,
    ) -> Result<usize> {
        let n = items.len();
        for (id, vector, payload) in items {
            self.upsert(&id, vector, payload).await?;
        }
        Ok(n)
    }

    /// Search for the `top_k` nearest vectors to `vector`. An optional
    /// `filter` (backend-specific JSON predicate) restricts the candidates.
    async fn search(
        &self,
        vector: &[f32],
        top_k: usize,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<VectorHit>>;

    /// Delete the vector for `id` (idempotent).
    async fn delete(&self, id: &EntityId) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_hit_constructs_and_displays() {
        let hit = VectorHit {
            id: EntityId::new("e:1"),
            score: 0.95,
            payload: serde_json::json!({"name": "foo"}),
        };
        assert_eq!(hit.id.as_str(), "e:1");
        assert!((hit.score - 0.95).abs() < f32::EPSILON);
        assert_eq!(hit.payload["name"], "foo");
        let dbg = format!("{hit:?}");
        assert!(dbg.contains("VectorHit"));
    }

    #[test]
    fn vector_hit_clones() {
        let a = VectorHit {
            id: EntityId::new("k"),
            score: 0.5,
            payload: serde_json::json!(null),
        };
        let b = a.clone();
        assert_eq!(a.id, b.id);
        assert!((a.score - b.score).abs() < f32::EPSILON);
        assert_eq!(a.payload, b.payload);
    }

    #[test]
    fn vector_hit_serde_roundtrip() {
        let hit = VectorHit {
            id: EntityId::new("e:1"),
            score: 0.123,
            payload: serde_json::json!({"a": 1, "b": "two"}),
        };
        let json = serde_json::to_string(&hit).expect("serialize");
        let back: VectorHit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, hit.id);
        assert!((back.score - hit.score).abs() < f32::EPSILON);
        assert_eq!(back.payload, hit.payload);
    }
}
