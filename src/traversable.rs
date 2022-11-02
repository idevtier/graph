use crate::types::MatrixGraphNode;
use crate::types::{GetNodeByIndex, GraphEntry, Neighbors};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;

/// Boundary for getting iterator over breadth first traverse of graph
pub trait BfsIterable<'a, N: 'a, I, T, G>
where
    N: MatrixGraphNode,
    I: Iterator<Item = (usize, &'a N)>,
    G: Neighbors<'a, N, I> + GetNodeByIndex<N>,
{
    fn get_graph(&'a self) -> &'a G;
    fn bfs_iter(&'a self, from: usize) -> BreadthFirstTraverseIterator<'a, N, G, I> {
        BreadthFirstTraverseIterator::new(self.get_graph(), from)
    }
}

/// Iterates over breadth first traverse of graph
/// represented by adjacency list
///
/// Takes **O(n)** space and computes in **O(n + e)**
/// where n = node count, e = edge count
pub struct BreadthFirstTraverseIterator<'a, N: 'a, G, I>
where
    I: Iterator<Item = (usize, &'a N)>,
    G: Neighbors<'a, N, I> + GetNodeByIndex<N>,
{
    graph: &'a G,
    visited: HashSet<usize>,
    queue: VecDeque<usize>,
    phantom1: PhantomData<N>,
    phantom2: PhantomData<I>,
}

impl<'a, N, G, I> BreadthFirstTraverseIterator<'a, N, G, I>
where
    N: MatrixGraphNode,
    I: Iterator<Item = (usize, &'a N)>,
    G: Neighbors<'a, N, I> + GetNodeByIndex<N>,
{
    pub fn new(graph: &'a G, from: usize) -> Self {
        Self {
            graph,
            visited: HashSet::new(),
            queue: VecDeque::from([from]),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<'a, N, G, I> Iterator for BreadthFirstTraverseIterator<'a, N, G, I>
where
    N: MatrixGraphNode,
    I: Iterator<Item = (usize, &'a N)>,
    G: Neighbors<'a, N, I> + GetNodeByIndex<N>,
{
    type Item = GraphEntry<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            return None;
        }

        let cur = self.queue.pop_front()?;
        self.visited.insert(cur);

        let edges = self.graph.neighbors(cur).collect::<Vec<(usize, &N)>>();
        let node = self.graph.get_node_by_index(cur).unwrap();

        for (i, _) in edges.iter() {
            if !self.visited.contains(i) {
                self.visited.insert(*i);
                self.queue.push_back(*i);
            }
        }

        Some(GraphEntry {
            node,
            edges: edges.into_iter().map(|(_idx, node)| node).collect(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix_graph::MatrixGraph;

    fn create_graph() -> MatrixGraph<u8, ()> {
        MatrixGraph::<u8, ()>::default()
    }

    #[test]
    fn collects_all_graph() {
        let mut g = create_graph();

        let expected = vec![(1, vec![4, 2, 3]), (4, vec![1]), (2, vec![3]), (3, vec![])];

        for (info, edges) in expected.iter() {
            let info = *info;
            let from_idx = match g.contains_node(&info) {
                true => g.get_index_of(&info).unwrap(),
                false => g.add_node(info),
            };
            for edge in edges {
                let to_idx = match g.contains_node(edge) {
                    true => g.get_index_of(edge).unwrap(),
                    false => g.add_node(*edge),
                };
                g.add_edge(from_idx, to_idx, ());
            }
        }

        let iter = BreadthFirstTraverseIterator::new(&g, 0);
        let actual = iter.collect::<Vec<GraphEntry<u8>>>();
        let expected = expected
            .iter()
            .map(|info| GraphEntry {
                node: &info.0,
                edges: info.1.iter().collect(),
            })
            .collect::<Vec<GraphEntry<u8>>>();

        assert_eq!(expected, actual);
    }
}
