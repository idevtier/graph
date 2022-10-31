# Graph library for pixelplex

![example workflow](https://github.com/usrsem/graph/actions/workflows/check-and-lint.yaml/badge.svg) ![example workflow](https://github.com/usrsem/graph/actions/workflows/test.yaml/badge.svg) [![codecov](https://codecov.io/gh/usrsem/graph/branch/main/graph/badge.svg?token=OHLZ48OAM7)](https://codecov.io/gh/usrsem/graph)

## Description

### Implementation
Choosed adjacency matrix implementation.

### Examples
Create empty graph with `u32` node and `String` edge weight:
```rust
let mut g = MatrixGraph::<u32, String>::default()
```

Or create from `IntoIterator`:
```rust
let edges = [
   (1, 2, 3),
   (3, 4, 7),
   (1, 3, 4),
];

let g = MatrxiGraph::<u32, u32>::from_edges(edges.into_iter());
```

Adding nodes and edges:
```rust
let mut g = MatrixGraph::<u32, String>::default()
let first = g.add_node(34);
let second = g.add_node(52);
g.add_edge(first, second, "First");
g.add_edge(second, first, "Second");
```

Removing nodes and edges:
```rust
g.remove_node(34);
g.remove_edge(0, 1);
```

Iterating over breadth first traverse of graph:
```rust
let start_node_idx = 0;
for entry in g.bfs_iter(start_node_idx) {
   printlnl("Node: {}, edges: {:?}", entry.node, entry.edges);
}
```

Deserialize from Trivial Graph Format:
```rust
let tgf = load_tgf_as_str();
let g = serialization::de_tgf::<u32, String>(tgf).expect("Something bad");
```

Full documentation available via `cargo doc`
