use crate::matrix_graph::NodeStorage;
use crate::serialization;
use std::fmt;
use std::hash::Hash;

/// Base type for graph node
pub trait MatrixGraphNode: Eq + Hash {}
impl<N> MatrixGraphNode for N where N: Eq + Hash {}

/// Structure for returning while traversing graph
#[derive(Debug, PartialEq, Eq)]
pub struct GraphEntry<'a, N>
where
    N: MatrixGraphNode,
{
    pub node: &'a N,
    pub edges: Vec<&'a N>,
}

/// Boundary for getting neighbors by graph node index
pub trait Neighbors<'a, N: 'a, I>
where
    I: Iterator<Item = (usize, &'a N)>,
{
    fn neighbors(&'a self, node: usize) -> IteratorHandle<'a, N, I>;
}

/// Generic iterator wrapper
pub struct IteratorHandle<'a, N: 'a, I>
where
    I: Iterator<Item = (usize, &'a N)>,
{
    pub iterator: I,
}

impl<'a, N: 'a, I> IteratorHandle<'a, N, I>
where
    I: Iterator<Item = (usize, &'a N)>,
{
    pub fn new(iterator: I) -> Self {
        Self { iterator }
    }
}

impl<'a, N: 'a, I> Iterator for IteratorHandle<'a, N, I>
where
    I: Iterator<Item = (usize, &'a N)>,
{
    type Item = (usize, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// Boundary for getting nodes and edges by index
/// Can be changed to defalt Index after stable GAT
pub trait Gettable<N, T> {
    fn get_node_by_index(&self, node_idx: usize) -> Option<&N>;
    fn get_edge_by_index(&self, from: usize, to: usize) -> Option<&T>;
}

/// Boundary for representing graph as adjacency matrix
pub trait Adjacency<N, T>
where
    N: MatrixGraphNode,
{
    fn get_adjacency_matrix(&self) -> AdjacencyMatrix<N, T>;
}

/// Structure for representing graph as adjacency matrix
#[derive(Debug, PartialEq, Eq)]
pub struct AdjacencyMatrix<'a, N, T>
where
    N: MatrixGraphNode,
{
    pub nodes: &'a NodeStorage<N>,
    pub edges: &'a Vec<Vec<Option<T>>>,
}

impl<'a, N, T> fmt::Display for AdjacencyMatrix<'a, N, T>
where
    N: fmt::Display + MatrixGraphNode,
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serialization::ser_tgf(self))?;
        Ok(())
    }
}
