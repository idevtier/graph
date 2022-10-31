use crate::matrix_graph::MatrixGraph;
use crate::types::{AdjacencyMatrix, MatrixGraphNode};
use core::str::FromStr;
use std::error::Error;
use std::fmt;

pub fn ser_tgf<N, T>(input: &AdjacencyMatrix<N, T>) -> String
where
    N: MatrixGraphNode + fmt::Display,
    T: fmt::Display,
{
    let mut output = "".to_string();

    for (i, node) in input.nodes.iter().enumerate() {
        output += &format!("{} {}\n", i + 1, node);
    }

    output += "#\n";

    for i in 0..input.nodes.len() {
        for j in 0..input.nodes.len() {
            if let Some(weight) = &input.edges[i][j] {
                output += &format!("{} {} {}\n", i + 1, j + 1, weight);
            }
        }
    }

    output
}

pub fn de_tgf<N, T>(input: &str) -> Result<MatrixGraph<N, T>, Box<dyn Error>>
where
    N: MatrixGraphNode + FromStr,
    T: FromStr,
{
    let mut g = MatrixGraph::<N, T>::default();
    let (nodes, edges) = input.split_once('#').unwrap();

    for node_str in nodes[..nodes.len() - 1].split('\n') {
        let (_id, node) = node_str.split_once(' ').unwrap();
        match node.parse() {
            Ok(node) => g.add_node(node),
            _ => panic!("Can't parse node"),
        };
    }

    for edge_str in edges.lines().filter(|s| !s.is_empty()) {
        let (from, to_weight) = edge_str.split_once(' ').unwrap();
        let (to, weight) = to_weight.split_once(' ').unwrap();

        g.add_edge(
            from.parse::<usize>()? - 1,
            to.parse::<usize>()? - 1,
            weight.parse().ok().unwrap(),
        );
    }

    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Adjacency, Gettable};

    #[test]
    fn test_ser_to_tgf() {
        let tgf = "1 1
2 2
3 3
4 4
5 5
#
1 2 3
1 3 4
1 4 5
1 5 6
3 1 4
3 2 5
3 4 7
5 2 7
";

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
        let actual = ser_tgf(&g.get_adjacency_matrix());
        assert_eq!(tgf, actual);
    }

    #[test]
    fn test_de_from_tgf() {
        let tgf = "1 54
2 2
3 3
4 4
5 5
#
1 2 3
1 3 4
1 4 5
1 5 6
3 1 4
3 2 5
3 4 7
5 2 7
";

        let edges = [
            (54, 2, 3),
            (3, 4, 7),
            (54, 3, 4),
            (3, 2, 5),
            (5, 2, 7),
            (54, 4, 5),
            (54, 5, 6),
            (3, 54, 4),
        ];

        let actual = de_tgf::<u32, u8>(tgf).unwrap();

        for (from, to, weight) in edges {
            let from = actual.get_index_of(&from).unwrap();
            let to = actual.get_index_of(&to).unwrap();
            let actual = actual.get_edge_by_index(from, to);
            assert!(actual.is_some());
            assert_eq!(actual.unwrap(), &weight);
        }
    }

    #[test]
    fn ser_wiki_example() {
        let tgf = "1 First node
2 Second node
#
1 2 Edge between the two
";

        let g: MatrixGraph<String, String> = de_tgf(tgf).unwrap();
        let edges = vec![("First node", "Second node", "Edge between the two")];

        for (from, to, weight) in edges {
            let from = g.get_index_of(&from.to_string()).unwrap();
            let to = g.get_index_of(&to.to_string()).unwrap();
            assert_eq!(g.get_edge_by_index(from, to).unwrap(), weight);
        }
    }
}
