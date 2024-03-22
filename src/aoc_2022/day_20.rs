use aoc::prelude::*;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "1
2
-3
3
-2
0
4";
            answers = signed![3];
        }
        actual_answers = signed![123];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use bare_metal_modulo::{MNum, ModNum};
    use itertools::Itertools;
    use nom::Finish;
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::{Rc, Weak},
    };

    struct CircularListNode<T> {
        value: T,
        previous: Option<NodeRefRaw<T>>,
        next: Option<NodeRefRaw<T>>,
    }
    impl<T> CircularListNode<T> {
        pub fn previous<'a>(&self, list: &'a CircularList<T>) -> Option<NodeRef<'a, T>> {
            self.previous.as_ref().map(|nr| NodeRef {
                list,
                raw: nr.clone(),
            })
        }

        pub fn next<'a>(&self, list: &'a CircularList<T>) -> Option<NodeRef<'a, T>> {
            self.next.as_ref().map(|nr| NodeRef {
                list,
                raw: nr.clone(),
            })
        }
    }

    type NodeRefRaw<T> = Weak<RefCell<CircularListNode<T>>>;

    #[derive(Clone)]
    struct NodeRef<'a, T> {
        list: &'a CircularList<T>,
        raw: NodeRefRaw<T>,
    }
    impl<T> PartialEq for NodeRef<'_, T> {
        fn eq(&self, other: &Self) -> bool {
            self.raw.ptr_eq(&other.raw)
        }
    }
    impl<T> Eq for NodeRef<'_, T> {}
    impl<T: std::fmt::Debug + Clone> std::fmt::Debug for NodeRef<'_, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.with_ref(|n| write!(f, "{:?}", n.value))
        }
    }
    impl<T: Copy> NodeRef<'_, T> {
        pub fn value(&self) -> T {
            self.with_ref(|n| n.value)
        }
    }
    impl<'a, T: Clone> NodeRef<'a, T> {
        fn with_ref<R>(&self, f: impl FnOnce(&CircularListNode<T>) -> R) -> R {
            f(self.raw.upgrade().unwrap().as_ref().borrow().deref())
        }

        fn with_mut<R>(&self, f: impl FnOnce(&mut CircularListNode<T>) -> R) -> R {
            f(self
                .raw
                .upgrade()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .deref_mut())
        }

        pub fn iter(&self, once: bool) -> CircularListNodeIterator<'a, T> {
            CircularListNodeIterator {
                list: self.list,
                next: Some(self.clone()),
                stop: once.then(|| self.clone()),
            }
        }

        pub fn remove(&self) {
            self.with_mut(|node| {
                assert!(
                    node.previous.is_some() && node.next.is_some(),
                    "the node has already been removed"
                );

                let node_next = node.next(self.list).unwrap();

                node.previous(self.list)
                    .unwrap()
                    .with_mut(|p| p.next = node.next.take());

                node_next.with_mut(|n| n.previous = node.previous.take());
            })
        }

        pub fn insert_after(&self, node: NodeRef<'a, T>) {
            assert!(
                std::ptr::eq(self.list, node.list),
                "the nodes are for different lists!"
            );
            assert!(
                node.with_ref(|n| n.previous.is_none() && n.next.is_none()),
                "cannot insert node because it is still linked"
            );

            let next = self.with_ref(|n| n.next(self.list)).unwrap();

            node.with_mut(|n| {
                n.previous = Some(self.raw.clone());
                n.next = Some(next.raw.clone());
            });

            self.with_mut(|n| n.next = Some(node.raw.clone()));
            next.with_mut(|n| n.previous = Some(node.raw.clone()));
        }

        pub fn shift(&self, relative_position: isize) {
            let relative_position = relative_position
                .rem_euclid(isize::try_from(self.list.nodes.len() - 1).unwrap())
                .try_into()
                .unwrap();

            let insert_node = if relative_position > 0 {
                self.iter(false).nth(relative_position).unwrap()
            } else {
                return;
            };

            self.remove();
            insert_node.insert_after(self.clone());
        }

        pub fn node_at(&self, index: isize) -> NodeRef<'a, T> {
            let index =
                usize::try_from(index.rem_euclid(isize::try_from(self.list.nodes.len()).unwrap()))
                    .unwrap();

            self.iter(false).nth(index).unwrap()
        }
    }

    #[derive(Clone)]
    struct CircularListNodeIterator<'a, T: Clone> {
        list: &'a CircularList<T>,
        next: Option<NodeRef<'a, T>>,
        stop: Option<NodeRef<'a, T>>,
    }
    impl<T: std::fmt::Debug + Clone> std::fmt::Debug for CircularListNodeIterator<'_, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut iter = self.clone();
            iter.stop = self.next.clone();

            write!(f, "[{}]", iter.map(|nr| format!("{nr:?}")).join(", "))
        }
    }
    impl<'a, T: Clone> CircularListNodeIterator<'a, T> {
        fn fetch_next(
            &mut self,
            which: impl FnOnce(&CircularListNode<T>) -> Option<NodeRef<'a, T>>,
        ) -> Option<NodeRef<'a, T>> {
            let ret = self.next.clone();

            if let Some(current_node) = ret.as_ref() {
                self.next = current_node.with_ref(which).and_then(|next_node| {
                    if let Some(stop_node) = self.stop.as_ref()
                        && *stop_node == next_node
                    {
                        None
                    } else {
                        Some(next_node)
                    }
                })
            }

            ret
        }
    }
    impl<'a, T: Clone> Iterator for CircularListNodeIterator<'a, T> {
        type Item = NodeRef<'a, T>;

        fn next(&mut self) -> Option<Self::Item> {
            self.fetch_next(|n| n.next(self.list))
        }
    }
    impl<T: Clone> DoubleEndedIterator for CircularListNodeIterator<'_, T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.fetch_next(|n| n.previous(self.list))
        }
    }

    struct CircularList<T> {
        nodes: Vec<Rc<RefCell<CircularListNode<T>>>>,
    }
    impl<T> CircularList<T> {
        pub fn new(iter: impl Iterator<Item = T>) -> Option<Self> {
            // Create initial list of nodes
            let nodes = iter
                .map(|value| {
                    Rc::new(RefCell::new(CircularListNode {
                        value,
                        previous: None,
                        next: None,
                    }))
                })
                .collect_vec();

            // Now add linked list references
            for (idx, node) in nodes.iter().enumerate() {
                let idx = ModNum::new(idx, nodes.len());

                let mut node = node.as_ref().borrow_mut();
                node.previous = Some(Rc::downgrade(&nodes[(idx - 1).a()]));
                node.next = Some(Rc::downgrade(&nodes[(idx + 1).a()]));
            }

            (!nodes.is_empty()).then_some(Self { nodes })
        }

        pub fn iter_const(&self) -> impl Iterator<Item = NodeRef<T>> + '_ {
            self.nodes.iter().map(|n| NodeRef {
                list: self,
                raw: Rc::downgrade(n),
            })
        }
    }

    pub struct File {
        data: Vec<i16>,
    }
    impl FromStr for File {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let data = s
                .lines()
                .map(|line| {
                    nom::character::complete::i16::<_, NomParseError>(line)
                        .finish()
                        .discard_input()
                })
                .collect::<Result<Vec<_>, _>>()?;

            if data.is_empty() {
                return Err(AocError::InvalidInput("Input contains no numbers!".into()));
            }

            Ok(Self { data })
        }
    }
    impl File {
        fn mix(&self) -> CircularList<i16> {
            let buffer = CircularList::new(self.data.iter().copied()).unwrap();

            for node in buffer.iter_const() {
                let shift = node.value().into();

                node.shift(shift);
            }

            buffer
        }

        pub fn grove_coordinate_sum(&self) -> i64 {
            let buffer = self.mix();
            let zero_node = buffer.iter_const().find(|n| n.value() == 0).unwrap();

            (1..=3)
                .map(|i| i64::from(zero_node.node_at(1000 * i).value()))
                .sum()
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 20,
    name: "Grove Positioning System",
    preprocessor: Some(|input| Ok(Box::new(File::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input.expect_data::<File>()?.grove_coordinate_sum().into())
        },
    ],
};
