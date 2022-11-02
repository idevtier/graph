use crate::types::GetNodeByIndex;
use crate::types::MatrixGraphNode;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, VecDeque},
    hash::Hasher,
    mem,
};

/// Collection for storing nodes
/// Works like indexed HashSet
#[derive(Debug, PartialEq, Eq)]
pub struct NodeStorage<N>
where
    N: MatrixGraphNode,
{
    nodes: Vec<Option<N>>,
    hashes: HashMap<u64, usize>,
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

        match self.removed.pop_back() {
            Some(idx) => {
                let _ = mem::replace(&mut self.nodes[idx], Some(node));
                self.hashes.insert(hash, idx);
                idx
            }
            None => {
                self.nodes.push(Some(node));
                let idx = self.nodes.len() - 1;
                self.hashes.insert(hash, idx);
                idx
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

    pub fn contains(&self, node: &N) -> bool {
        self.get_index_of(node).is_some()
    }

    pub fn get_index_of(&self, node: &N) -> Option<usize> {
        let hash = Self::calculate_hash(node);
        self.hashes.get(&hash).cloned()
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

impl<N> GetNodeByIndex<N> for NodeStorage<N>
where
    N: MatrixGraphNode,
{
    fn get_node_by_index(&self, node_idx: usize) -> Option<&N> {
        if node_idx > self.nodes.len() {
            return None;
        }

        self.nodes[node_idx].as_ref()
    }
}
