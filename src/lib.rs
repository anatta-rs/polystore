//! Triad storage abstraction — `GraphStore` + `KvStore` + `VectorStore` traits.
//!
//! Polystore defines three orthogonal storage traits intended to be implemented
//! by substitutable backends (in-memory, embedded `SurrealDB`, `Neo4j`+`KV`+`Qdrant`…).
//! Consumers code against the traits; backends slot in via dependency injection.
//!
//! # Example
//!
//! ```no_run
//! use polystore::{GraphStore, KvStore, VectorStore, Scope};
//!
//! async fn process<G, K, V>(graph: &G, kv: &K, vec: &V) -> polystore::Result<()>
//! where
//!     G: GraphStore<String, String>,
//!     K: KvStore,
//!     V: VectorStore,
//! {
//!     let _ = graph.scope();
//!     let _ = kv.scope();
//!     let _ = vec.scope();
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod error;
pub mod graph;
pub mod kv;
pub mod types;
pub mod vector;

pub use error::{PolystoreError, Result};
pub use graph::GraphStore;
pub use kv::KvStore;
pub use types::{Direction, EntityId, Scope};
pub use vector::{VectorHit, VectorStore};
