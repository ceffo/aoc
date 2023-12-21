use nom::{
    bytes::complete::tag,
    character::complete::alpha1,
    character::complete::{self, line_ending, one_of},
    combinator::map_res,
    multi::{many1, separated_list1},
    sequence::{self, delimited, terminated},
};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
};

use crate::custom_error::AocError;
use nom::IResult;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Instruction {
    Left,
    Right,
}

impl TryFrom<char> for Instruction {
    type Error = AocError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Instruction::Left),
            'R' => Ok(Instruction::Right),
            _ => Err(AocError::LogicError("invalid instruction".to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node<'a> {
    tag: &'a str,
    left: &'a str,
    right: &'a str,
}

impl Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> ({},{})", self.tag, self.left, self.right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Network<'a> {
    instructions: Vec<Instruction>,
    nodes: BTreeMap<&'a str, Node<'a>>,
}

trait Visitor {
    fn visit(&mut self, node: &Node);
}

struct ClosureVisitor<F: FnMut(&Node)> {
    closure: F,
}

impl<F: FnMut(&Node)> Visitor for ClosureVisitor<F> {
    #[tracing::instrument(skip(self))]
    fn visit(&mut self, node: &Node) {
        (self.closure)(node);
    }
}

impl<'a> Network<'a> {
    #[tracing::instrument(skip(self, visitor))]
    fn walk(
        &self,
        start: &'a str,
        end: &'a str,
        visitor: &mut dyn Visitor,
    ) -> Result<&'a str, AocError> {
        let num_instructions = self.instructions.len();
        let mut visited = HashSet::<(&'a str, usize)>::with_capacity(self.nodes.len());
        let mut current = self
            .nodes
            .get(start)
            .ok_or(AocError::LogicError("node not found".to_string()))?;
        for (i, instruction) in self.instructions.iter().cycle().enumerate() {
            // safety check to make sure we don't loop forever
            let instruction_index = i % num_instructions;
            if visited.contains(&(current.tag, instruction_index)) {
                // we detect a loop by checking if we've visited this node with this instruction index before
                let msg = format!(
                    "loop detected at node [{}] with instruction # {} {:?}",
                    current.tag, instruction_index, instruction
                );
                return Err(AocError::LogicError(msg));
            }

            // visit the current node
            visitor.visit(current);

            // stop if we've the reached a terminal node
            if current.tag == end {
                break;
            }

            // otherwise, continue walking the network
            visited.insert((current.tag, instruction_index));
            current = match instruction {
                Instruction::Left => self
                    .nodes
                    .get(current.left)
                    .ok_or(AocError::LogicError("node not found".to_string()))?,
                Instruction::Right => self
                    .nodes
                    .get(current.right)
                    .ok_or(AocError::LogicError("node not found".to_string()))?,
            };
        }
        Ok(current.tag)
    }
}

fn parse_instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    let (input, instructions) = terminated(
        many1(map_res(one_of("LR"), Instruction::try_from)),
        line_ending,
    )(input)?;
    Ok((input, instructions))
}

fn parse_node(input: &str) -> IResult<&str, Node> {
    let (input, node_tag) = alpha1(input)?;
    let (input, _) = tag(" = ")(input)?;
    let (input, (left, right)) = delimited(
        complete::char('('),
        sequence::separated_pair(alpha1, tag(", "), alpha1),
        complete::char(')'),
    )(input)?;
    Ok((
        input,
        Node {
            tag: node_tag,
            left,
            right,
        },
    ))
}

fn parse_network(input: &str) -> IResult<&str, Network> {
    let (input, instructions) = parse_instructions(input)?;
    let (input, _) = many1(line_ending)(input)?;
    let (input, nodes) = separated_list1(line_ending, parse_node)(input)?;
    let nodes = nodes
        .into_iter()
        .map(|node| (node.tag, node))
        .collect::<BTreeMap<&str, Node>>();
    Ok((
        input,
        Network {
            instructions,
            nodes,
        },
    ))
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, network) = parse_network(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let mut num_visited = 0;
    let mut visited_nodes = Vec::<String>::new();
    let mut visitor = ClosureVisitor {
        closure: |node: &Node| {
            num_visited += 1;
            visited_nodes.push(node.tag.to_string());
            info!("{}: {}", num_visited, node);
        },
    };
    _ = network.walk("AAA", "ZZZ", &mut visitor)?;
    let steps = num_visited - 1;
    Ok(steps.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)",
        "2"
    )]
    #[case(
        "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)",
        "6"
    )]
    fn test_process(#[case] input: &str, #[case] expected: &str) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }

    #[test]
    #[tracing::instrument]
    fn test_walk() -> miette::Result<()> {
        let instructions = vec![Instruction::Left, Instruction::Right];
        let nodes = vec![
            Node {
                tag: "A",
                left: "B",
                right: "B",
            },
            Node {
                tag: "B",
                left: "Z",
                right: "C",
            },
            Node {
                tag: "C",
                left: "A",
                right: "Z",
            },
            Node {
                tag: "Z",
                left: "Z",
                right: "Z",
            },
        ];
        let network = Network {
            instructions,
            nodes: nodes
                .iter()
                .map(|node| (node.tag, node.clone()))
                .collect::<BTreeMap<&str, Node>>(),
        };
        let expected = vec!["A", "B", "C", "A", "B", "Z"];
        let mut visited = Vec::<String>::new();
        let mut visitor = ClosureVisitor {
            closure: |node: &Node| {
                visited.push(node.tag.to_string());
            },
        };
        let terminal = network.walk("A", "Z", &mut visitor)?;
        assert!(terminal == "Z");
        assert_eq!(expected, visited);

        Ok(())
    }
}
