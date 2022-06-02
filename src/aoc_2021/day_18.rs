use std::fmt::Write;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, separated_pair},
};
use trees::{Node, Tree};

use crate::aoc::{parse::trim, prelude::*};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]",
    vec![11u64].answer_vec()
    }
}

enum Side {
    Left,
    Right,
    Root,
}
enum NodeType {
    Branch,
    Leaf(u8),
}
#[derive(new)]
struct NodeData {
    side: Side,
    node_type: NodeType,
}
#[derive(new)]
struct Number {
    tree: Tree<NodeData>,
}
impl Parseable<'_> for Number {
    fn parser(input: &str) -> NomParseResult<&str, Self> {
        alt((
            map(nom::character::complete::u8, |v| {
                Number::new(Tree::new(NodeData::new(Side::Root, NodeType::Leaf(v))))
            }),
            map(
                delimited(
                    tag("["),
                    separated_pair(Self::parser, trim(tag(",")), Self::parser),
                    tag("]"),
                ),
                |(mut tl, mut tr)| {
                    tl.tree.root_mut().data_mut().side = Side::Left;
                    tr.tree.root_mut().data_mut().side = Side::Right;
                    let mut root = Tree::new(NodeData::new(Side::Root, NodeType::Branch));
                    root.push_back(tl.tree);
                    root.push_back(tr.tree);
                    Self::new(root)
                },
            ),
        ))(input)
    }
}
impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_node(f: &mut std::fmt::Formatter<'_>, node: &Node<NodeData>) -> std::fmt::Result {
            match node.data().node_type {
                NodeType::Branch => {
                    f.write_char('[')?;
                    format_node(f, node.front().unwrap())?;
                    f.write_char(',')?;
                    format_node(f, node.back().unwrap())?;
                    f.write_char(']')
                }
                NodeType::Leaf(v) => write!(f, "{}", v),
            }
        }
        format_node(f, self.tree.root())?;
        f.write_char('\n')
    }
}
impl Number {
    fn reduce(&mut self) {
        // First find any sufficiently deep nodes with two leaves
        for node in self.tree.bfs_children_mut().iter {}
    }
}

trait NodeUtils {
    fn next_left(&self) -> Option<&mut Self>;
    fn next_right(&self) -> Option<&mut Self>;
    fn depth(&self) -> usize;
}
impl NodeUtils for Node<NodeData> {
    fn next_left(&self) -> Option<&mut Self> {
        todo!()
    }

    fn next_right(&self) -> Option<&mut Self> {
        todo!()
    }

    fn depth(&self) -> usize {
        let mut depth = 0;
        let mut node = self;
        loop {
            match node.parent() {
                Some(p) => {
                    node = p;
                    depth += 1;
                }
                None => break,
            }
        }
        depth
    }
}

pub const SOLUTION: Solution = Solution {
    day: 18,
    name: "Snailfish",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            let numbers = Number::gather(input.lines())?;

            println!("TODO: {:?}", numbers[0]);

            // Process
            Ok(0u64.into())
        },
    ],
};
