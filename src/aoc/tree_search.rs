//! Potentially exhaustive search of a tree structure.
//!
//! Provides traits that can be implemented by tree nodes to enable recursive
//! searches of the tree. The different traits have different goals and provide
//! different results when searching the tree. Tree search capability is added
//! by implementing one of the node traits on tree node items, which can then
//! spawn child nodes into which the search algorithm will recurse.
//!
//! Examples of problems amenable to tree structures including one or
//! multiplayer game trees, optimally solving a problem with a particular goal
//! using a brute force search, etc. Refer to AOC problem solutions that utilize
//! this module for more examples.

use crate::error::{AocError, AocResult};
use derive_more::{Add, From};
use derive_new::new;
use std::{collections::HashMap, fmt::Debug};

/// Action to take by a tree search algorithm after processing a particular
/// node.
pub enum NodeAction<N> {
    /// This is a terminal node, so do not recurse.
    Stop,
    /// The is a terminal node, and the search should immediately stop.
    Complete,
    /// Recurse to one or more child nodes.
    Continue(Vec<N>),
}

/// Implemented by a tree node, for which the tree search runs until stopped by
/// a node or the entire tree is searched.
///
/// The nodes have access to a single, mutable global state.
///
/// # Examples
/// For examples of the usage of this tree search method, see the
/// [2020 day 20
/// problem](../../advent_of_code/aoc_2020/day_20/solution/index.html)
/// or the
/// [2021 day 12
/// problem](../../advent_of_code/aoc_2021/day_12/solution/index.html).
pub trait GlobalStateTreeNode: Sized {
    /// The type of the global state.
    type GlobalState;

    /// Determines the action to take by the search algorithm from the current
    /// node.
    fn recurse_action(self, global_state: &mut Self::GlobalState) -> NodeAction<Self>;

    /// Searches the tree until the whole tree is searched, or a node stops the
    /// search by returning [`NodeAction::Complete`].
    ///
    /// The initial global state is passed in and the final state after the
    /// search is returned.
    ///
    /// # Panics
    /// This will panic if any node returns an empty array of children.
    fn traverse_tree(self, mut initial_state: Self::GlobalState) -> Self::GlobalState {
        /// This is an internal recursive function of
        /// [`GlobalStateTreeNode::traverse_tree`].
        ///
        /// Recursive performs the tree search.
        /// The return value is whether to terminate the search immediately.
        fn rec_traverse<N: GlobalStateTreeNode>(
            global_state: &mut N::GlobalState,
            current_node: N,
        ) -> bool {
            match current_node.recurse_action(global_state) {
                NodeAction::Stop => false,
                NodeAction::Continue(children) => {
                    if children.is_empty() {
                        panic!("node returned an empty child list");
                    }

                    for child in children {
                        if rec_traverse(global_state, child) {
                            return true;
                        }
                    }
                    false
                }
                NodeAction::Complete => true,
            }
        }

        rec_traverse(&mut initial_state, self);
        initial_state
    }
}

/// A metric, primarily for use with a [`BestCostTreeNode`] tree search, but can
/// be used more generally.
///
/// Typically metrics are numeric, representing a cost to be minimized.
pub trait Metric: Sized + Debug {
    /// Returns whether this metric is better than some `other` metric.
    fn is_better(&self, other: &Self) -> bool;

    /// Sets this metric value to some other metric if the other is better.
    fn update_if_better(&mut self, other: Self) {
        if other.is_better(self) {
            *self = other;
        }
    }
}
impl<T: Metric> Metric for Option<T> {
    fn is_better(&self, other: &Self) -> bool {
        match self {
            Some(s) => match other {
                Some(o) => s.is_better(o),
                None => true,
            },
            None => false,
        }
    }
}

/// Action to take by a tree search algorithm after processing a particular
/// node.
pub enum ApplyNodeAction<C> {
    /// This is a terminal node, and whether this node/path should should count
    /// or not.
    Stop(bool),
    /// This is a terminal node and the search should be immediately stopped,
    /// and whether this node/path should should count or not.
    Complete(bool),
    /// Recurse to one or more child nodes.
    Continue(Vec<C>),
}

/// State that stores a singular solution.
pub struct BasicSolutionState<T> {
    /// The current solution, if one has been set.
    solution: Option<T>,
}
impl<T> Default for BasicSolutionState<T> {
    fn default() -> Self {
        Self { solution: None }
    }
}
impl<T> BasicSolutionState<T> {
    /// Sets the solution.
    pub fn set_solution(&mut self, sol: T) {
        self.solution = Some(sol);
    }

    /// Returns the current solution, or [`AocError::NoSolution`] if none has
    /// been set.
    pub fn solution(self) -> AocResult<T> {
        self.solution.ok_or(AocError::NoSolution)
    }
}

/// The global state for a [`BestCostTreeNode`] search.
struct BestCostState<N: BestCostTreeNode> {
    /// The overall best cost, if one has been set.
    best_cost: Option<N::Metric>,
    /// Optimization table where the key is a node, and the value is the best
    /// cost of the node's sub-tree, that is, the best cost if starting at
    /// the node.
    ///
    /// A value of `None` means that the goal state cannot be reached by the
    /// node.
    node_best_costs: HashMap<N, Option<N::Metric>>,
}
impl<N: BestCostTreeNode> BestCostState<N> {
    /// Updates the overall best cost if `other` is better.
    pub fn update_if_better(&mut self, other: N::Metric) {
        self.best_cost.update_if_better(Some(other));
    }
}

/// A child node to which to recurse for a [`BestCostTreeNode`].
#[derive(new)]
pub struct BestCostChild<N: BestCostTreeNode> {
    /// The child node to which to recurse.
    node: N,
    /// The cost to go from the parent node to this child.
    cost: N::Metric,
}

/// A tree node in a [`BestCostTreeNode`] search.
struct BestCostNode<N: BestCostTreeNode> {
    /// The actual [`BestCostTreeNode`].
    node: N,
    /// The total cost to get to get to this node from the root.
    cumulative_cost: N::Metric,
}

/// Implemented by a tree node, for which the tree search optimizes some
/// [`Metric`].
///
/// Each transition from parent to child has an associated relative cost. Each
/// path from the root node to each successful terminal node then has a total
/// cost. It is this total cost that the tree search will optimize over the
/// entire tree.
///
/// # Examples
/// For examples of the usage of this tree search method, see the
/// [2015 day 22
/// problem](../../advent_of_code/aoc_2015/day_22/solution/index.html) or the
/// [2021 day 23
/// problem](../../advent_of_code/aoc_2021/day_23/solution/index.html).
pub trait BestCostTreeNode: Sized + Clone + Eq + PartialEq + std::hash::Hash + Debug {
    /// The cost type, the default value should be initial or zero cost.
    type Metric: Metric + Clone + Default + Copy + std::ops::Add<Output = Self::Metric>;

    /// Determines the action to take by the algorithm from the current node.
    fn recurse_action(&mut self) -> ApplyNodeAction<BestCostChild<Self>>;

    /// Searches the tree to find the optimal [`Metric`] cost, which is returned
    /// if one was found.
    ///
    /// The algorithm includes the optimization of keeping a best cost table for
    /// each node, which stores the best cost if one were to start at that
    /// node, that is, the best cost of its sub-tree.
    /// This obviates the need to recurse past each node more than once, which
    /// can vastly reduce the algorithm execution time.
    /// Because of this, the nodes must implement [`Hash`](std::hash::Hash) and
    /// [`Eq`], for which two nodes should be equivalent if they would have
    /// the same optimum path past that point.
    ///
    /// Due to built-in optimizations, care must be taken when defining node
    /// equality, and/or when causing tree branches to end early. These can
    /// cause the optimizations to fail to work correctly, causing problems
    /// that are difficult to debug.
    ///
    /// # Panics
    /// This will panic if any node returns an empty array of children.
    fn traverse_tree(self) -> AocResult<Self::Metric> {
        /// A return value from the the recursive tree search function.
        struct BestCostReturn<N: BestCostTreeNode> {
            /// Whether to immediately terminate the search.
            complete: bool,
            /// The best cost of the sub-tree below the current node, if there
            /// is a valid path to a successful terminal node.
            best_cost: Option<N::Metric>,
        }

        /// This is an internal recursive function of
        /// [`BestCostTreeNode::traverse_tree`].
        ///
        /// Recursive performs the tree search.
        fn rec_traverse<N: BestCostTreeNode>(
            best_cost_state: &mut BestCostState<N>,
            mut current_node: BestCostNode<N>,
        ) -> BestCostReturn<N> {
            // If our cumulative cost is already worse than the best cost, we need not
            // proceed further
            if let Some(bc) = best_cost_state.best_cost
                && bc.is_better(&current_node.cumulative_cost)
            {
                return BestCostReturn {
                    complete: false,
                    best_cost: None,
                };
            }

            // If we already know the best cost to add for this node and its sub-tree, then
            // exit early
            if let Some(bc) = best_cost_state
                .node_best_costs
                .get(&current_node.node)
                .copied()
            {
                if let Some(best_cost) = bc {
                    best_cost_state.update_if_better(current_node.cumulative_cost + best_cost);
                }
                return BestCostReturn {
                    complete: false,
                    best_cost: bc,
                };
            }

            let bc_return = match current_node.node.recurse_action() {
                ApplyNodeAction::Stop(apply) => {
                    if apply {
                        best_cost_state.update_if_better(current_node.cumulative_cost);
                    }
                    BestCostReturn {
                        complete: false,
                        best_cost: apply.then_some(N::Metric::default()),
                    }
                }
                ApplyNodeAction::Complete(apply) => {
                    if apply {
                        best_cost_state.update_if_better(current_node.cumulative_cost);
                    }
                    BestCostReturn {
                        complete: true,
                        best_cost: apply.then_some(N::Metric::default()),
                    }
                }
                ApplyNodeAction::Continue(children) => {
                    if children.is_empty() {
                        panic!("node returned an empty child list");
                    }

                    let mut best_cost = None;

                    for child in children {
                        let mut bc_return = rec_traverse(
                            best_cost_state,
                            BestCostNode {
                                node: child.node,
                                cumulative_cost: current_node.cumulative_cost + child.cost,
                            },
                        );

                        bc_return.best_cost = bc_return.best_cost.map(|c| c + child.cost);

                        if bc_return.complete {
                            return bc_return;
                        }

                        best_cost.update_if_better(bc_return.best_cost);
                    }

                    BestCostReturn {
                        complete: false,
                        best_cost,
                    }
                }
            };

            // Update the best cost node optimization table
            best_cost_state
                .node_best_costs
                .insert(current_node.node.clone(), bc_return.best_cost);
            bc_return
        }

        let mut initial_state = BestCostState {
            best_cost: None,
            node_best_costs: HashMap::new(),
        };
        rec_traverse(
            &mut initial_state,
            BestCostNode {
                node: self,
                cumulative_cost: Self::Metric::default(),
            },
        );

        initial_state.best_cost.ok_or(AocError::NoSolution)
    }
}

/// A [`Metric`] that counts steps between node.
#[derive(Clone, Copy, Debug, Default, Add, From)]
struct Steps(usize);
impl Metric for Steps {
    fn is_better(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}

/// A tree node wrapper in a [`LeastStepsTreeNode`] search.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct LeastStepsNode<N: LeastStepsTreeNode>(N);
impl<N: LeastStepsTreeNode> BestCostTreeNode for LeastStepsNode<N> {
    type Metric = Steps;

    fn recurse_action(&mut self) -> ApplyNodeAction<BestCostChild<Self>> {
        match self.0.recurse_action() {
            ApplyNodeAction::Stop(a) => ApplyNodeAction::Stop(a),
            ApplyNodeAction::Complete(a) => ApplyNodeAction::Complete(a),
            ApplyNodeAction::Continue(v) => ApplyNodeAction::Continue(
                v.into_iter()
                    .map(|node| BestCostChild {
                        node: Self(node),
                        cost: 1.into(),
                    })
                    .collect(),
            ),
        }
    }
}

/// Implemented by a tree node, for which the tree search finds the least number
/// of steps to a successful terminal node.
///
/// # Examples
/// For examples of the usage of this tree search method, see the
/// [2015 day 19
/// problem](../../advent_of_code/aoc_2015/day_19/solution/index.html).
pub trait LeastStepsTreeNode: Sized + Clone + Eq + PartialEq + std::hash::Hash + Debug {
    /// Determines the action to take by the search algorithm from the current
    /// node.
    fn recurse_action(&mut self) -> ApplyNodeAction<Self>;

    /// Searches the tree until the whole tree is searched, or a node stops the
    /// search by returning [`ApplyNodeAction::Complete`].
    ///
    /// Returns the least number of steps to a successful terminal node, or
    /// [`AocError::NoSolution`] if no successful terminal nodes were
    /// encountered.
    ///
    /// The caveats that apply to [`BestCostTreeNode::traverse_tree`] apply here
    /// as well in terms of defining node equality and implementing
    /// premature tree branch trimming.
    fn traverse_tree(self) -> AocResult<usize> {
        LeastStepsNode(self).traverse_tree().map(|s| s.0)
    }
}
