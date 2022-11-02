use crate::traversable::BfsIterable;
use crate::types::{Adjacency, AdjacencyMatrix};
use crate::types::{Gettable, IteratorHandle, MatrixGraphNode, Neighbors};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::{cmp, fmt, hash::Hasher, mem, vec};

#[derive(Debug, PartialEq, Eq)]
pub struct NodeStorage<N>
where
    N: MatrixGraphNode,
{
    nodes: Vec<Option<N>>,
    hashes: HashMap<u64, ()>,
    removed: VecDeque<usize>,
}

impl<N> Default for NodeStorage<N>
where
    N: MatrixGraphNode,
{
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            removed: VecDeque::new(),
            hashes: HashMap::new(),
        }
    }
}

impl<N> NodeStorage<N>
where
    N: MatrixGraphNode,
{
    pub fn add(&mut self, node: N) -> usize {
        let hash = Self::calculate_hash(&node);
        if self.hashes.get(&hash).is_some() {
            panic!("Nodes should be unique.");
        }

        self.hashes.insert(hash, ());

        match self.removed.pop_back() {
            Some(idx) => {
                let _ = mem::replace(&mut self.nodes[idx], Some(node));
                idx
            }
            None => {
                self.nodes.push(Some(node));
                self.nodes.len() - 1
            }
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<N> {
        let node = mem::replace(&mut self.nodes[idx], None);
        if let Some(node) = node.as_ref() {
            let hash = Self::calculate_hash(node);
            self.hashes.remove(&hash);
        }
        self.removed.push_back(idx);
        node
    }

    pub fn len(&self) -> usize {
        self.nodes.len() - self.removed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, idx: usize) -> &N {
        if idx > self.nodes.len() {
            panic!("Out of bounds");
        }

        let node = self.nodes[idx].as_ref();

        match node {
            Some(node) => node,
            None => panic!("Trying to get removed node"),
        }
    }

    pub fn get_checked(&self, idx: usize) -> Option<&N> {
        if idx > self.nodes.len() {
            return None;
        }

        self.nodes[idx].as_ref()
    }

    pub fn contains(&self, node: &N) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_i, n)| match n {
                Some(n) => n == node,
                None => false,
            })
            .map(|(i, _n)| i)
            .next()
    }

    pub fn iter(&'_ self) -> NodeStorageIterator<'_, N> {
        NodeStorageIterator::new(&self.nodes)
    }

    fn calculate_hash(node: &N) -> u64 {
        let mut s = DefaultHasher::new();
        node.hash(&mut s);
        s.finish()
    }
}

pub struct NodeStorageIterator<'a, N> {
    nodes: &'a Vec<Option<N>>,
    idx: usize,
}

impl<'a, N> NodeStorageIterator<'a, N> {
    pub fn new(nodes: &'a Vec<Option<N>>) -> Self {
        Self { nodes, idx: 0 }
    }
}

impl<'a, N> Iterator for NodeStorageIterator<'a, N> {
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx >= self.nodes.len() {
                return None;
            }

            self.idx += 1;

            if let Some(node) = &self.nodes[self.idx - 1] {
                return Some(node);
            }
        }
    }
}

/// Graph representation with adjacency matrix
///
/// Note: it's better to use for dense graph
pub struct MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    nodes: NodeStorage<N>,
    adjacency: Vec<Vec<Option<T>>>,
    edge_count: usize,
}

impl<N, T> Default for MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    fn default() -> Self {
        Self {
            nodes: NodeStorage::default(),
            adjacency: vec![vec![None]],
            edge_count: 0,
        }
    }
}

impl<N, T> MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    /// Create MatrixGraph from iterator of tuples
    /// where each element representes edge between
    /// two nodes and it's weight
    pub fn from_edges(edges: impl IntoIterator<Item = (N, N, T)>) -> Self {
        let mut g = Self::default();

        for (from, to, weight) in edges {
            let from_idx = match g.contains_node(&from) {
                true => g.get_index_of(&from).unwrap(),
                false => g.add_node(from),
            };

            let to_idx = match g.contains_node(&to) {
                true => g.get_index_of(&to).unwrap(),
                false => g.add_node(to),
            };

            g.add_edge(from_idx, to_idx, weight);
        }

        g
    }
}

impl<N, T> MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    /// Adds new node in graph
    ///
    /// Returns index of new node
    ///
    /// Computes in **O(1)** (average amortized)
    /// Worse case **O(n)** where n is nodes count
    ///
    /// **Panics** if node already exists
    pub fn add_node(&mut self, node: N) -> usize {
        self.nodes.add(node)
    }

    /// Removes node and all edges for it
    ///
    /// Returns removed node or None, if node not found
    ///
    /// Computes in **O(e)** (average) where e = node's edges count
    pub fn remove_node(&mut self, node_index: usize) -> Option<N> {
        if node_index >= self.nodes.len() || node_index >= self.adjacency.len() {
            return None;
        }

        for i in 0..self.nodes.len() {
            if i >= self.adjacency.len() {
                break;
            }

            let positions = [(i, node_index), (node_index, i)];

            positions
                .iter()
                .for_each(|(from, to)| self.adjacency[*from][*to] = None);
        }

        self.nodes.remove(node_index)
    }

    /// Adds edge between two nodes
    ///
    /// Computes in **O(1)** (average)
    /// Worst case **O(n ^ 2)** where n = nodes count
    ///
    /// **Panics** if some of nodes not exists or edge already exists
    pub fn add_edge(&mut self, from: usize, to: usize, weight: T) {
        let max_idx = cmp::max(from, to);
        if max_idx >= self.nodes.len() {
            panic!(
                "Can't add edge for not existing node with index {}",
                max_idx
            );
        }

        if self.update_edge(from, to, weight).is_some() {
            panic!("Edge from {} to {} already exists", from, to);
        }
    }

    /// Removes edge between two nodes
    ///
    /// Returns edge's weight if removed else None
    ///
    /// Computes in **O(1)**
    pub fn remove_edge(&mut self, from_node: usize, to_node: usize) -> Option<T> {
        if cmp::max(from_node, to_node) >= self.nodes.len() {
            return None;
        }

        let old_edge = mem::replace(&mut self.adjacency[from_node][to_node], None);

        if old_edge.is_some() {
            self.edge_count -= 1;
        }

        old_edge
    }

    /// Returns count of nodes
    ///
    /// Computes in **O(1)**
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns count of edges
    ///
    /// Computes in **O(1)**
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// Returns iterator over nodes
    #[inline]
    pub fn node_iter(&mut self) -> NodeIterator<N> {
        NodeIterator::new(&self.nodes)
    }

    /// Checks if node exists in graph
    ///
    /// Computes in **O(n)**
    #[inline]
    pub fn contains_node(&self, node: &N) -> bool {
        self.nodes.contains(node).is_some()
    }

    /// Checks if edge between two nodes exists
    ///
    /// Computes in **O(1)**
    #[inline]
    pub fn contains_edge(&self, from: usize, to: usize) -> bool {
        if cmp::max(from, to) >= self.adjacency.len() {
            return false;
        }
        self.adjacency[from][to].is_some()
    }

    /// Returns index of node or None if not found
    ///
    /// Computes in **O(n)**
    #[inline]
    pub fn get_index_of(&self, node: &N) -> Option<usize> {
        self.nodes.contains(node)
    }

    fn update_edge(&mut self, from: usize, to: usize, weight: T) -> Option<T> {
        self.extend_capacity_if_needed(from, to);
        let last_edge = mem::replace(&mut self.adjacency[from][to], Some(weight));

        if last_edge.is_none() {
            self.edge_count += 1;
        }

        last_edge
    }

    fn extend_capacity_if_needed(&mut self, from: usize, to: usize) {
        let p = cmp::max(from, to);

        if p < self.adjacency.len() {
            return;
        }

        let new_capacity = cmp::max(4, p + 1).next_power_of_two().pow(2);
        let diff = new_capacity - self.adjacency.len();
        self.adjacency
            .extend((0..diff).map(|_| (0..new_capacity).map(|_| None).collect()));

        for i in 0..diff {
            self.adjacency[i].extend((0..diff).map(|_| None));
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////

impl<N, T> Gettable<N, T> for MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    #[inline]
    fn get_node_by_index(&self, node_idx: usize) -> Option<&N> {
        self.nodes.get_checked(node_idx)
    }

    #[inline]
    fn get_edge_by_index(&self, from: usize, to: usize) -> Option<&T> {
        if cmp::max(from, to) >= self.adjacency.len() {
            return None;
        }
        self.adjacency[from][to].as_ref()
    }
}

/////////////////////////////////////////////////////////////////////////////////////

pub struct NodeIterator<'a, N>
where
    N: MatrixGraphNode,
{
    nodes: &'a NodeStorage<N>,
    index: usize,
}

impl<'a, N> NodeIterator<'a, N>
where
    N: MatrixGraphNode,
{
    pub fn new(nodes: &'a NodeStorage<N>) -> Self {
        Self { nodes, index: 0 }
    }
}

impl<'a, N> Iterator for NodeIterator<'a, N>
where
    N: MatrixGraphNode,
{
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.nodes.len() {
                return None;
            }
            self.index += 1;
            let node = self.nodes.get_checked(self.index - 1);
            if node.is_some() {
                return node;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct MatrixGraphNeighborsIterator<'a, N, T>
where
    N: MatrixGraphNode,
{
    column: usize,
    nodes: &'a NodeStorage<N>,
    adjacency: &'a Vec<Option<T>>,
}

impl<'a, N, T> Iterator for MatrixGraphNeighborsIterator<'a, N, T>
where
    N: MatrixGraphNode,
{
    type Item = (usize, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.column >= self.adjacency.len() {
                return None;
            }

            let node_exists = &self.adjacency[self.column];
            self.column += 1;
            if node_exists.is_some() {
                return Some((self.column - 1, self.nodes.get_checked(self.column - 1)?));
            }
        }
    }
}

impl<'a, N: 'a, T> Neighbors<'a, N, MatrixGraphNeighborsIterator<'a, N, T>> for MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    fn neighbors(
        &'a self,
        node: usize,
    ) -> IteratorHandle<'a, N, MatrixGraphNeighborsIterator<'a, N, T>> {
        if node >= self.nodes.len() {
            panic!("Node with index {} not found", node);
        }

        let iterator = MatrixGraphNeighborsIterator {
            column: 0,
            nodes: &self.nodes,
            adjacency: &self.adjacency[node],
        };

        IteratorHandle { iterator }
    }
}

impl<N, T> Adjacency<N, T> for MatrixGraph<N, T>
where
    N: MatrixGraphNode + Clone,
{
    fn get_adjacency_matrix(&self) -> AdjacencyMatrix<N, T> {
        AdjacencyMatrix {
            nodes: &self.nodes,
            edges: &self.adjacency,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'a, N: 'a, T> BfsIterable<'a, N, MatrixGraphNeighborsIterator<'a, N, T>, T, MatrixGraph<N, T>>
    for MatrixGraph<N, T>
where
    N: MatrixGraphNode,
{
    fn get_graph(&'a self) -> &'a MatrixGraph<N, T> {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<N, T> fmt::Display for MatrixGraph<N, T>
where
    N: MatrixGraphNode + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in self.bfs_iter(0) {
            let neighs = if entry.edges.is_empty() {
                "[ ]".to_string()
            } else {
                entry.edges.iter().fold("[ ".to_string(), |s, node| {
                    s + &(self.get_index_of(node).unwrap() + 1).to_string() + " "
                }) + "]"
            };

            writeln!(
                f,
                "Id: {}, neighbors: {}, Value: {}",
                self.get_index_of(entry.node).unwrap() + 1,
                neighs,
                entry.node
            )?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn create_graph() -> MatrixGraph<u32, ()> {
        MatrixGraph::<u32, ()>::default()
    }

    #[test]
    fn test_creates_default_empty() {
        let g = create_graph();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_creates_from_edges() {
        let edges = [
            (1, 2, 3),
            (3, 4, 7),
            (1, 3, 4),
            (3, 2, 5),
            (5, 2, 7),
            (1, 4, 5),
            (1, 5, 6),
            (3, 1, 4),
        ];

        let g = MatrixGraph::<u32, u8>::from_edges(edges.into_iter());
        assert_eq!(g.node_count(), 5, "Nodes: {}", g.node_count());
        assert_eq!(g.edge_count(), edges.len(), "Nodes: {}", g.edge_count());

        for (from, to, weight) in edges {
            let from_idx = g.get_index_of(&from).unwrap();
            let to_idx = g.get_index_of(&to).unwrap();
            assert_eq!(g.get_edge_by_index(from_idx, to_idx).unwrap(), &weight);
        }
    }

    #[test]
    fn test_adds_new_node() {
        let mut g = create_graph();
        let node_idx = g.add_node(34);
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.edge_count(), 0);
        assert_eq!(node_idx, 0);
    }

    #[test]
    #[should_panic(expected = "Nodes should be unique.")]
    fn test_panics_on_adding_existing_node() {
        let mut g = create_graph();
        g.add_node(34);
        g.add_node(34);
    }

    #[test]
    fn test_returns_none_on_removing_not_existing_node() {
        let mut g = create_graph();
        let node = g.remove_node(0);
        assert!(node.is_none());
    }

    #[test]
    fn test_removes_existing_node() {
        let mut g = create_graph();
        let i = g.add_node(32);
        let node = g.remove_node(i).unwrap();
        assert_eq!(node, 32);
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn test_saves_correct_edges_after_node_remove() {
        let mut g = MatrixGraph::<u32, u32>::default();
        let a_idx = g.add_node(1);
        let b_idx = g.add_node(2);
        let c_idx = g.add_node(3);

        g.add_edge(b_idx, c_idx, 1);

        g.remove_node(a_idx);

        let weight = g.get_edge_by_index(b_idx, c_idx);

        assert!(weight.is_some());
    }

    #[test]
    fn test_adds_edge() {
        let mut g = create_graph();
        let first = g.add_node(34);
        let second = g.add_node(52);
        g.add_edge(first, second, ());
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    #[should_panic(expected = "Edge from 0 to 1 already exists")]
    fn test_panics_on_creating_existing_edge() {
        let mut g = create_graph();
        let first = g.add_node(34);
        let second = g.add_node(52);
        g.add_edge(first, second, ());
        g.add_edge(first, second, ());
    }

    #[test]
    #[should_panic(expected = "Can't add edge for not existing node with index 1")]
    fn test_panics_on_create_edge_for_not_existing_node() {
        let mut g = create_graph();
        g.add_edge(0, 1, ());
    }

    #[test]
    fn test_removes_existing_edge() {
        let mut g = create_graph();
        let a = g.add_node(12);
        let b = g.add_node(54);
        g.add_edge(a, b, ());
        g.remove_edge(a, b);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_indexes_not_shifted_after_removing_middle_node() {
        let mut g = create_graph();
        let a_idx = g.add_node(13);
        let b_idx = g.add_node(43);
        let c_idx = g.add_node(89);
        g.remove_node(b_idx);
        assert_eq!(g.get_index_of(&13).unwrap(), a_idx);
        assert_eq!(g.get_index_of(&89).unwrap(), c_idx);
    }

    #[test]
    fn test_returns_false_on_removing_not_existing_edge() {
        let mut g = create_graph();
        let is_removed = g.remove_edge(0, 1);
        let expected = None;
        assert_eq!(expected, is_removed);
    }

    #[test]
    fn test_not_removes_flipped_edge() {
        let mut g = create_graph();
        let a = g.add_node(12);
        let b = g.add_node(54);
        g.add_edge(a, b, ());
        let actual = g.remove_edge(b, a);
        let expected = None;
        assert_eq!(expected, actual);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn test_return_node_by_index() {
        let mut g = create_graph();
        g.add_node(12);
        g.add_node(54);
        assert_eq!(g.get_node_by_index(0).unwrap(), &12);
        assert_eq!(g.get_node_by_index(1).unwrap(), &54);
    }

    #[test]
    fn test_returns_valid_nodes_iter() {
        let mut g = create_graph();
        let expected: Vec<u32> = vec![12, 34, 56, 89];
        for node in expected.iter() {
            g.add_node(*node);
        }
        let actual: Vec<_> = g.node_iter().copied().collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_adds_incoming_and_outgoing_edges() {
        let mut g = create_graph();
        let a_idx = g.add_node(5);
        let b_idx = g.add_node(7);
        g.add_edge(a_idx, b_idx, ());
        g.add_edge(b_idx, a_idx, ());
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_contains_edge_returns_true() {
        let mut g = create_graph();
        let a_idx = g.add_node(1);
        let b_idx = g.add_node(3);
        g.add_edge(a_idx, b_idx, ());
        assert!(g.contains_edge(a_idx, b_idx));
    }

    #[test]
    fn test_contains_edge_returns_false() {
        let mut g = create_graph();
        let a_idx = g.add_node(1);
        assert!(!g.contains_edge(a_idx, 5));
    }

    #[test]
    fn test_get_edge_by_index_return_none_if_not_foun() {
        let g = create_graph();
        let weight = g.get_edge_by_index(0, 2);
        assert!(weight.is_none());
    }

    #[test]
    #[should_panic(expected = "Node with index 6 not found")]
    fn test_panics_on_getting_neighbors_for_not_existed_node() {
        let g = create_graph();
        g.neighbors(6);
    }

    fn create_closure() -> fn(u32) {
        |x| println!("This is x: {}", x)
    }

    #[test]
    fn test_can_hold_fn_as_edge_weight() {
        let edges = vec![
            (1, 2, create_closure()),
            (2, 3, create_closure()),
            (3, 4, create_closure()),
        ];
        MatrixGraph::<u32, fn(u32)>::from_edges(edges.into_iter());
    }
}
