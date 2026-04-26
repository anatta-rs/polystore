# polystore

Triad storage abstraction for Rust — three orthogonal traits for substitutable backends.

```rust
trait GraphStore<N, E>     // typed nodes + edges + structural queries
trait KvStore              // bytes in / bytes out
trait VectorStore          // embeddings + similarity k-NN
```

Zero impl, zero backend. Consumers code against the traits; backends slot in via dependency injection.

## Why

Storage choices for a single application often don't reduce to one technology:

- A **graph DB** for typed entities + edges (call graphs, knowledge graphs, social graphs).
- A **KV store** for opaque content blobs indexed by hash.
- A **vector DB** for embedding similarity search.

Polystore declares the contract for all three so consumer code never depends on
the chosen backend. Swap from in-memory → embedded SurrealDB → distributed
Neo4j+Qdrant without changing a line of consumer code.

## Multi-tenant by design

Every trait carries a `Scope` (`namespace / repo / branch`) so backends
naturally partition data along the GitHub-orgs hierarchy.

## Usage

```rust
use polystore::{GraphStore, KvStore, VectorStore, Scope, EntityId};

async fn analyze<G, K, V>(graph: &G, kv: &K, vec: &V) -> polystore::Result<()>
where
    G: GraphStore<MyNode, MyEdge>,
    K: KvStore,
    V: VectorStore,
{
    let scope = graph.scope();
    println!("operating on {scope}");

    let id = EntityId::new("fn:foo");
    graph.upsert_node(&id, MyNode::default()).await?;
    kv.put(&format!("content:{id}"), b"...").await?;
    vec.upsert(&id, vec![0.1, 0.2, 0.3], serde_json::json!({})).await?;

    Ok(())
}

#[derive(Default)]
struct MyNode;
struct MyEdge;
```

## Quality gates

- `make check` → fmt + clippy + test
- `make coverage-gate` → enforces ≥ 95% line coverage
- CI runs all of the above on every PR.

## Status

`v0.1.x` — API exploration. Expect breaking changes until `v1.0`.

## License

[Apache-2.0](./LICENSE)
