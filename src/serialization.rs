use crate::matrix_graph::MatrixGraph;
use crate::types::{AdjacencyMatrix, MatrixGraphNode};
use anyhow::Result;
use core::str::FromStr;
use std::error::Error;
use std::fmt;
use std::str::Lines;

#[derive(Debug, PartialEq, Eq)]
pub enum TgfParseError {
    WrongFileFormat,
    WrongNodeValue,
    WrongEdgeValue,
}

impl Error for TgfParseError {}

impl fmt::Display for TgfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TgfParseError::*;

        let output = match self {
            WrongNodeValue => "Wrong node value.".to_string(),
            WrongEdgeValue => "Wrong edge value.".to_string(),
            WrongFileFormat => "Wrong file format.".to_string(),
        };

        write!(f, "{}", output)?;

        Ok(())
    }
}

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

pub struct TgfParser<'a, N, T>
where
    N: MatrixGraphNode + FromStr,
    T: FromStr,
{
    input: Lines<'a>,
    result: MatrixGraph<N, T>,
}

impl<'a, N, T> TgfParser<'a, N, T>
where
    N: MatrixGraphNode + FromStr,
    T: FromStr,
{
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.lines(),
            result: MatrixGraph::<N, T>::default(),
        }
    }

    pub fn parse(mut self) -> Result<MatrixGraph<N, T>> {
        self.parse_nodes()?;
        self.parse_delimiter()?;
        self.parse_edges()?;
        Ok(self.result)
    }

    fn parse_nodes(&mut self) -> Result<()> {
        loop {
            let line = self.input.next().ok_or(TgfParseError::WrongNodeValue)?;
            if line == "#" {
                return Ok(());
            }
            let value = Self::parse_node(&line)?;
            self.result.add_node(value);
        }
    }

    fn parse_delimiter(&mut self) -> Result<()> {
        Ok(())
    }

    fn parse_edges(&mut self) -> Result<()> {
        loop {
            match self.input.next() {
                Some(line) => {
                    let (from, to, value) = Self::parse_edge(&line)?;
                    self.result.add_edge(from, to, value);
                }
                None => break Ok(()),
            }
        }
    }

    fn parse_node(line: &str) -> Result<N> {
        let (_, node) = line.split_once(' ').ok_or(TgfParseError::WrongNodeValue)?;
        let node = node
            .parse::<N>()
            .map_err(|_| TgfParseError::WrongNodeValue)?;
        Ok(node)
    }

    fn parse_edge(line: &str) -> Result<(usize, usize, T)> {
        let (from, to_and_value) = line.split_once(' ').ok_or(TgfParseError::WrongEdgeValue)?;
        let (to, value) = to_and_value
            .split_once(' ')
            .ok_or(TgfParseError::WrongEdgeValue)?;

        let from = from
            .parse::<usize>()
            .map_err(|_| TgfParseError::WrongNodeValue)?;
        let to = to
            .parse::<usize>()
            .map_err(|_| TgfParseError::WrongNodeValue)?;
        let value = value.parse().map_err(|_| TgfParseError::WrongNodeValue)?;

        Ok((from - 1, to - 1, value))
    }
}

pub fn de_tgf<N, T>(input: &str) -> Result<MatrixGraph<N, T>>
where
    N: MatrixGraphNode + FromStr,
    T: FromStr,
{
    TgfParser::new(input).parse()
}

fn _de_nodes_and_edges(input: &str) -> Result<(&str, &str)> {
    input
        .split_once(r"^#\n")
        .ok_or(TgfParseError::WrongFileFormat.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Adjacency, GetEdgeByIndex};

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
    fn test_de_tgf_returns_error_on_wrong_node_value() {
        let wrong_tgf = "1 Some bad value\r2 3\r#";
        let g = de_tgf::<u32, u32>(wrong_tgf);
        assert!(g.err().is_some());
    }

    #[test]
    fn test_de_tgf_returns_error_on_wrong_edge_value() {
        let wrong_tgf = "1 2
2 3
#
Wrong value";
        let g = de_tgf::<u32, u32>(wrong_tgf);
        assert!(g.err().is_some());
    }
}
