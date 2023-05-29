//! Potentially exhaustive search of a tree structure.
//!
//! Provides traits that can be implemented by tree nodes to enable recursive searches
//! of the tree. The different traits have different goals and provide different results when
//! searching the tree.
//!
//! Examples of problems amenable to tree structures including one or multiplayer game trees,
//! optimally solving a problem with a particular goal using a brute force search, etc.
//! Refer to AOC problem solutions that utilize this module for more examples.

use derive_new::new;

use std::{cell::RefCell, collections::HashMap, fmt, ops::Add, rc::Rc};

mod general {
    use super::*;

    #[derive(new)]
    pub struct Child<N: TreeNode> {
        pub node: N,
        pub state: N::DownwardState,
    }

    pub trait TreeUpwardState<N: TreeNode>: Sized {
        fn new(current: &Child<N>) -> Self;

        // Return whether or not to stop and return the current upward state.
        fn incorporate_child(&mut self, current: &Child<N>, child_upward_state: Self);

        fn finalize(&mut self, current: Child<N>);
    }

    pub enum TreeAction<N: TreeNode> {
        Stop(N::UpwardState),
        Continue(Vec<Child<N>>),
    }

    pub trait TreeNode: Sized {
        type DownwardState: Default;
        type UpwardState: TreeUpwardState<Self> + fmt::Debug;

        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self>;

        fn traverse_tree(self) -> Self::UpwardState {
            fn rec<N: TreeNode>(current: Child<N>) -> N::UpwardState {
                let mut upward_state = N::UpwardState::new(&current);

                match current.node.recurse_action(&current.state) {
                    TreeAction::Stop(child_upward_state) => {
                        upward_state.incorporate_child(&current, child_upward_state);
                    }
                    TreeAction::Continue(children) => {
                        // Recurse for each leaf
                        for child in children {
                            upward_state.incorporate_child(&current, rec(child));
                        }
                    }
                }

                upward_state.finalize(current);
                upward_state
            }

            rec(Child::new(self, Self::DownwardState::default()))
        }
    }
}

mod metric {
    use std::collections::hash_map::Entry;

    use super::general::*;
    use super::*;

    /// Global state among the whole recursive process.
    ///
    /// This is used mainly for optimization by terminating visiting sub-trees
    /// early whenever possible.
    #[derive(Clone)]
    struct MetricGlobalState<N: BestMetricTreeNode> {
        /// The global best cost to solve.
        best_metric: N::Metric,
        /// Map of node to the best cost to solve from this node position.
        seen: HashMap<N, N::Metric>,
    }
    impl<N: BestMetricTreeNode> Default for MetricGlobalState<N> {
        fn default() -> Self {
            Self {
                best_metric: N::Metric::INITIAL_BEST,
                seen: HashMap::new(),
            }
        }
    }
    impl<N: BestMetricTreeNode> MetricGlobalState<N> {
        fn solution_found(&self) -> Option<&N::Metric> {
            if self.best_metric.is_better(&N::Metric::INITIAL_BEST) {
                Some(&self.best_metric)
            } else {
                None
            }
        }
    }

    pub struct MetricDownwardState<N: BestMetricTreeNode> {
        /// The global state.
        global_state: Rc<RefCell<MetricGlobalState<N>>>,
        /// The total cost up to get to this node along the taken path.
        cumulative_cost: N::Metric,
        /// The cost of the previous move to get to the current node.
        node_cost: N::Metric,
        _level: usize,
    }
    impl<N: BestMetricTreeNode> Default for MetricDownwardState<N> {
        fn default() -> Self {
            Self {
                global_state: Default::default(),
                cumulative_cost: N::Metric::INITIAL_COST,
                node_cost: N::Metric::INITIAL_COST,
                _level: 0,
            }
        }
    }

    // Represent the best metric needed to solve from the associated position.
    #[derive(Debug)]
    pub struct BestMetric<N: BestMetricTreeNode>(pub N::Metric);
    impl<N: BestMetricTreeNode> TreeUpwardState<BestMetricNode<N>> for BestMetric<N> {
        fn new(_current: &Child<BestMetricNode<N>>) -> Self {
            Self(N::Metric::INITIAL_BEST)
        }

        fn incorporate_child(
            &mut self,
            _current: &Child<BestMetricNode<N>>,
            child_upward_state: Self,
        ) {
            self.0.update_if_better(child_upward_state.0);
        }

        fn finalize(&mut self, current: Child<BestMetricNode<N>>) {
            let mut global_state = current.state.global_state.as_ref().borrow_mut();

            // Mark the seen metric as the best cost to solve from here.
            match global_state.seen.entry(current.node.0) {
                Entry::Occupied(mut entry) => entry.get_mut().update_if_better(self.0),
                Entry::Vacant(entry) => {
                    entry.insert(self.0.clone());
                }
            }

            // If selected and we already have a solution, just pass that up
            if N::STOP_AT_FIRST && let Some(bm) = global_state.solution_found() {
                self.0 = bm.clone();
            } else {
                // Update the global best cost if better for the total cost whose solution passes
                // through this node.
                global_state
                    .best_metric
                    .update_if_better(current.state.cumulative_cost + self.0);

                // Pass up the best cost to solve from the previous position
                self.0 = self.0 + current.state.node_cost;
            }
        }
    }

    pub struct BestMetricNode<N: BestMetricTreeNode>(pub N);
    impl<N: BestMetricTreeNode> TreeNode for BestMetricNode<N> {
        type DownwardState = MetricDownwardState<N>;
        type UpwardState = BestMetric<N>;

        // The action should contain the best metric to solve from this node.
        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self> {
            let global_state = downward_state.global_state.as_ref().borrow_mut();

            // If selected and we have already found a solution, then just stop
            if N::STOP_AT_FIRST && global_state.solution_found().is_some() {
                return TreeAction::Stop(BestMetric(N::Metric::INITIAL_BEST));
            }

            // Is our cost already too high?
            if global_state
                .best_metric
                .is_better(&downward_state.cumulative_cost)
            {
                return TreeAction::Stop(BestMetric(N::Metric::INITIAL_BEST));
            }

            // Have we seen this node already?
            if let Some(bm) = global_state.seen.get(&self.0) {
                // Pass the best to solve from this node
                return TreeAction::Stop(BestMetric(bm.clone()));
            }

            match self.0.recurse_action(&downward_state.cumulative_cost) {
                BestMetricAction::StopSuccess => {
                    TreeAction::Stop(BestMetric(N::Metric::INITIAL_COST))
                }
                BestMetricAction::StopFailure => {
                    TreeAction::Stop(BestMetric(N::Metric::INITIAL_BEST))
                }
                BestMetricAction::Continue(children) => TreeAction::Continue(
                    children
                        .into_iter()
                        .map(|child| {
                            Child::new(
                                Self(child.node),
                                MetricDownwardState {
                                    global_state: downward_state.global_state.clone(),
                                    cumulative_cost: downward_state.cumulative_cost + child.cost,
                                    node_cost: child.cost,
                                    _level: downward_state._level + 1,
                                },
                            )
                        })
                        .collect(),
                ),
            }
        }
    }
}

use general::TreeNode;

/// A metric for use with a [`BestMetricTreeNode`].
///
/// Typically metrics are numeric, often utilizing [`Infinitable`](infinitable::Infinitable) to have an
/// initial infinite value denoting that no metric has yet been set. This works
/// out based on the intuitive ordering of [`Infinitable`](infinitable::Infinitable) in that, for example,
/// any finite number is always less than (i.e. a better metric if minimizing) than the initial infinite value.
pub trait Metric: Add<Output = Self> + Copy {
    /// The initial metric to set as the best when the tree search starts.
    const INITIAL_BEST: Self;
    /// The initial cost when calculating cumulative costs of children at each step in a path.
    ///
    /// Typically this will be zero when the metric is numeric.
    const INITIAL_COST: Self;

    /// Returns whether this metric is better than some `other` metric.
    fn is_better(&self, other: &Self) -> bool;

    /// Sets this metric value to some other metric if the other is better.
    fn update_if_better(&mut self, other: Self) {
        if other.is_better(self) {
            *self = other;
        }
    }
}

/// A child of a [`BestMetricTreeNode`] for use with [`BestMetricAction::Continue`].
#[derive(new)]
pub struct MetricChild<N: BestMetricTreeNode> {
    /// The node of the child.
    node: N,
    /// The relative cost to go from the current node to the child node.
    cost: N::Metric,
}

/// An action to be returned by [`BestMetricTreeNode::recurse_action`].
pub enum BestMetricAction<N: BestMetricTreeNode> {
    /// This node is a successful terminal node in which the cumulative cost to get here is valid.
    StopSuccess,
    /// This node is a failure terminal node in which the cumulative cost to get here should be disregarded.
    StopFailure,
    /// This node has children into which the algorithm should recurse.
    Continue(Vec<MetricChild<N>>),
}

/// Implemented by a tree node, for which the tree search optimizes some [`Metric`].
///
/// Eeach transition from parent to child has an associated relative cost. Each path
/// from the root node to each successful terminal node then has a total cost. It
/// is this total cost that the tree search will optimize over the entire tree.
pub trait BestMetricTreeNode: Sized + Eq + std::hash::Hash + fmt::Debug {
    /// The [`Metric`] to use for costs and optimization.
    type Metric: Metric + fmt::Debug;
    /// Instead of searching the entire tree, this will stop the algorithm early, returning
    /// the total cost to the first success terminal node encountered.
    const STOP_AT_FIRST: bool = false;

    /// Determines the action to take by the algorithm from the current node.
    ///
    /// The `cumulative_cost` of the current path is available, which includes the cost to
    /// get to the current node.
    fn recurse_action(&self, cumulative_cost: &Self::Metric) -> BestMetricAction<Self>;

    /// Searches the tree to find the optimal [`Metric`].
    ///
    /// The algorithm includes optimizations such as keeping a global best metric and aborting
    /// a path if its cumulative cost becomes worse than the current best metric. The
    /// optimal/best metric is returned after the search is complete.
    fn best_metric(self) -> Self::Metric {
        metric::BestMetricNode(self).traverse_tree().0
    }
}

mod global {
    use super::general::*;
    use super::*;

    #[derive(Debug)]
    pub struct GlobalUpwardState<N: GlobalStateTreeNode>(pub Rc<RefCell<N::GlobalState>>);
    impl<N: GlobalStateTreeNode> TreeUpwardState<GlobalStateNode<N>> for GlobalUpwardState<N> {
        fn new(current: &Child<GlobalStateNode<N>>) -> Self {
            Self(current.state.clone())
        }

        fn incorporate_child(
            &mut self,
            _current: &Child<GlobalStateNode<N>>,
            _child_upward_state: Self,
        ) {
            // Do nothing
        }

        fn finalize(&mut self, _current: Child<GlobalStateNode<N>>) {
            // Do nothing
        }
    }

    pub struct GlobalStateNode<N: GlobalStateTreeNode>(pub N);
    impl<N: GlobalStateTreeNode> TreeNode for GlobalStateNode<N> {
        type DownwardState = Rc<RefCell<N::GlobalState>>;
        type UpwardState = GlobalUpwardState<N>;

        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self> {
            let mut global_state = downward_state.as_ref().borrow_mut();
            let done_action = TreeAction::Stop(GlobalUpwardState(downward_state.clone()));

            // Have we completed traversal?
            if global_state.complete() {
                return done_action;
            }

            match self.0.recurse_action(&global_state) {
                GlobalAction::Apply => {
                    global_state.update_with_node(&self.0);
                    done_action
                }
                GlobalAction::Stop => done_action,
                GlobalAction::Continue(children) => TreeAction::Continue(
                    children
                        .into_iter()
                        .map(|node| Child::new(Self(node), downward_state.clone()))
                        .collect(),
                ),
            }
        }
    }
}

pub trait GlobalState<N>: Default + fmt::Debug {
    fn update_with_node(&mut self, node: &N);

    fn complete(&self) -> bool;
}

pub enum GlobalAction<N: GlobalStateTreeNode> {
    Apply,
    Stop,
    Continue(Vec<N>),
}

pub trait GlobalStateTreeNode: Sized + fmt::Debug {
    type GlobalState: GlobalState<Self>;

    fn recurse_action(&self, state: &Self::GlobalState) -> GlobalAction<Self>;

    fn traverse_tree(self) -> Self::GlobalState {
        Rc::try_unwrap(global::GlobalStateNode(self).traverse_tree().0)
            .unwrap()
            .into_inner()
    }
}

#[derive(Debug)]
pub struct FirstSolutionGlobalState<N> {
    solution: Option<N>,
}
impl<N> Default for FirstSolutionGlobalState<N> {
    fn default() -> Self {
        Self { solution: None }
    }
}
impl<N> FirstSolutionGlobalState<N> {
    pub fn solution(self) -> Option<N> {
        self.solution
    }
}
impl<N: GlobalStateTreeNode + Clone + fmt::Debug> GlobalState<N> for FirstSolutionGlobalState<N> {
    fn update_with_node(&mut self, node: &N) {
        self.solution = Some(node.clone());
    }

    fn complete(&self) -> bool {
        self.solution.is_some()
    }
}

mod least_steps {
    use super::*;
    use infinitable::Infinitable;

    type LeastStepsMetric = Infinitable<usize>;
    impl Metric for LeastStepsMetric {
        const INITIAL_BEST: Self = Infinitable::Infinity;
        const INITIAL_COST: Self = Infinitable::Finite(0);

        fn is_better(&self, other: &Self) -> bool {
            *self < *other
        }
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    pub struct LeastStepsNode<N: LeastStepsTreeNode>(pub N);
    impl<N: LeastStepsTreeNode> BestMetricTreeNode for LeastStepsNode<N> {
        type Metric = LeastStepsMetric;
        const STOP_AT_FIRST: bool = N::STOP_AT_FIRST;

        fn recurse_action(&self, _cumulative_cost: &Self::Metric) -> BestMetricAction<Self> {
            match self.0.recurse_action() {
                LeastStepsAction::StopSuccess => BestMetricAction::StopSuccess,
                LeastStepsAction::StopFailure => BestMetricAction::StopFailure,
                LeastStepsAction::Continue(children) => BestMetricAction::Continue(
                    children
                        .into_iter()
                        .map(|child| MetricChild::new(Self(child), 1.into()))
                        .collect(),
                ),
            }
        }
    }
}

pub enum LeastStepsAction<N> {
    StopSuccess,
    StopFailure,
    Continue(Vec<N>),
}

pub trait LeastStepsTreeNode: Sized + fmt::Debug + Eq + std::hash::Hash {
    const STOP_AT_FIRST: bool = false;

    fn recurse_action(&self) -> LeastStepsAction<Self>;

    fn least_steps(self) -> Option<usize> {
        match least_steps::LeastStepsNode(self).best_metric() {
            infinitable::Infinitable::Finite(steps) => Some(steps),
            _ => None,
        }
    }
}

// TODO: Potential uses
// X 2015 - 19 - Part 2, making a medicine (New LeastStepsTreeNode)
// X 2015 - 22 - RPG with spells (min MP used)
// X 2015 - 24 - Sleight compartment packages (cannot use because recursive but not a tree)
//
// X 2020 - 07 - Recursive bags (recursive but not a tree)
// X 2020 - 10 - Part 2, Joltage adapters (Cannot use due to optimization)
// X 2020 - 19 - Recursive rule validation (Recursive but not easily a tree)
// X 2020 - 20 - Part 1, Lining up images (Only care about first final solution)
//
// X 2021 - 09 - Regions sizes in caves (recursive but not a tree)
// X 2021 - 12 - Paths through a cave system (uses GlobalStateTree)
// X 2021 - 19 - Correlating scanners (recursive, but not easy to solve using tree, was able to optimize though)
// X 2021 - 21 - Part 2, Dirac die (count of universes in which each player wins)
// X 2021 - 23 - Amphipods (Min energy)
