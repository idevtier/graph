use crate::matrix_graph::MatrixGraph;
use crate::types::{AdjacencyMatrix, MatrixGraphNode};
use core::str::FromStr;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum TgfParseError {
    WrongNodeValue,
    WrongEdgeValue,
}

impl fmt::Display for TgfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TgfParseError::*;

        let output = match self {
            WrongNodeValue => "Wrong node value".to_string(),
            WrongEdgeValue => "Wrong edge value".to_string(),
        };

        write!(f, "{}", output)?;

        Ok(())
    }
}

impl Error for TgfParseError {}

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

pub fn de_tgf<N, T>(input: &str) -> Result<MatrixGraph<N, T>, TgfParseError>
where
    N: MatrixGraphNode + FromStr,
    T: FromStr,
{
    let mut g = MatrixGraph::<N, T>::default();
    let (nodes, edges) = input.split_once('#').unwrap();

    for node_str in nodes[..nodes.len() - 1].split('\n') {
        let (_id, node) = node_str.split_once(' ').unwrap();
        if let Ok(node) = node.parse() {
            g.add_node(node);
            continue;
        };

        return Err(TgfParseError::WrongNodeValue);
    }

    for edge_str in edges.lines().filter(|s| !s.is_empty()) {
        let (from, to_weight) = edge_str
            .split_once(' ')
            .ok_or(TgfParseError::WrongEdgeValue)?;

        let (to, weight) = to_weight
            .split_once(' ')
            .ok_or(TgfParseError::WrongEdgeValue)?;

        let weight = weight
            .parse::<T>()
            .map_err(|_| TgfParseError::WrongEdgeValue)?;
        let from = from
            .parse::<usize>()
            .map_err(|_| TgfParseError::WrongEdgeValue)?;
        let to = to
            .parse::<usize>()
            .map_err(|_| TgfParseError::WrongEdgeValue)?;

        g.add_edge(from - 1, to - 1, weight);
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
    fn test_de_tgf_returns_error_on_wrang_node_value() {
        let wrong_tgf = "1 Some bad value\r2 3\r#";
        let g = de_tgf::<u32, u32>(wrong_tgf);
        assert_eq!(g.err().unwrap(), TgfParseError::WrongNodeValue);
    }

    #[test]
    fn test_de_tgf_returns_error_on_wrang_edge_value() {
        let wrong_tgf = "1 2
2 3
#
Wrong value";
        let g = de_tgf::<u32, u32>(wrong_tgf);
        assert_eq!(g.err().unwrap(), TgfParseError::WrongEdgeValue);
    }
}
