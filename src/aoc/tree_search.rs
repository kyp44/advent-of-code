//! Potentially exhaustive search of a tree structure.
//!
//! Provides traits that can be implemented by tree nodes to enable recursive searches
//! of the tree. The different traits have different goals and provide different results when
//! searching the tree. Tree search capability is implemented by implementing one of the node
//! traits on tree node structures, which can then spawn child nodes into which the search
//! algorithm will recurse.
//!
//! Examples of problems amenable to tree structures including one or multiplayer game trees,
//! optimally solving a problem with a particular goal using a brute force search, etc.
//! Refer to AOC problem solutions that utilize this module for more examples.

use derive_new::new;

use std::{cell::RefCell, collections::HashMap, fmt, ops::Add, rc::Rc};

/// Private module for a general tree search, which is utilized by the various public
/// tree search methods.
mod general {
    use super::*;

    /// Represents a child node for a current node.
    #[derive(new)]
    pub struct Child<N: TreeNode> {
        /// The child node itself.
        pub node: N,
        /// Downward state to pass to this child.
        pub state: N::DownwardState,
    }

    /// The upward state for a general tree search, passed upward between nodes after all sub trees
    /// have been searched.
    pub trait TreeUpwardState<N: TreeNode>: Sized {
        /// Creates a new upward state based on the current node prior to any child recursion.
        fn new(current: &Child<N>) -> Self;

        /// Incorporates the upward state of a child whose recursion has completed.
        fn incorporate_child(&mut self, current: &Child<N>, child_upward_state: Self);

        /// Finalizes the upward state after incorporating all children, and possibly taking into
        /// account the current node.
        fn finalize(&mut self, current: Child<N>);
    }

    /// An action for a general tree node, to be returned from [`TreeNode::recurse_action`].
    pub enum TreeAction<N: TreeNode> {
        /// Stop and return the upward state.
        Stop(N::UpwardState),
        /// Continue recursing with children.
        Continue(Vec<Child<N>>),
    }

    /// Implemented by a general tree node.
    pub trait TreeNode: Sized {
        /// The downward state type.
        type DownwardState;
        /// The upward state type.
        type UpwardState: TreeUpwardState<Self> + fmt::Debug;

        /// Returns the tree action for this node.
        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self>;

        /// Performs the general tree search using the initial `downward_state`,
        /// returning the upward state of the root node.
        fn traverse_tree(self, downward_state: Self::DownwardState) -> Self::UpwardState {
            /// This is a recursive internal function of [`TreeNode::traverse_tree`].
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

            rec(Child::new(self, downward_state))
        }
    }
}

/// Private module that implements the [`BestMetricTreeNode`] search.
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
        /// Returns the global best metric if one has been found.
        fn solution_found(&self) -> Option<&N::Metric> {
            if self.best_metric.is_better(&N::Metric::INITIAL_BEST) {
                Some(&self.best_metric)
            } else {
                None
            }
        }
    }

    /// Downward state for the [`BestMetricTreeNode`] search.
    pub struct MetricDownwardState<N: BestMetricTreeNode> {
        /// The global state.
        global_state: Rc<RefCell<MetricGlobalState<N>>>,
        /// The total cost up to get to this node along the taken path.
        cumulative_cost: N::Metric,
        /// The cost of the previous move to get to the current node.
        node_cost: N::Metric,
    }
    impl<N: BestMetricTreeNode> Default for MetricDownwardState<N> {
        fn default() -> Self {
            Self {
                global_state: Default::default(),
                cumulative_cost: N::Metric::INITIAL_COST,
                node_cost: N::Metric::INITIAL_COST,
            }
        }
    }

    /// Wrapper for the [`Metric`].
    ///
    /// Represents the best metric needed to solve from the associated position.
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
                    entry.insert(self.0);
                }
            }

            // If selected and we already have a solution, just pass that up
            if N::STOP_AT_FIRST
                && let Some(bm) = global_state.solution_found()
            {
                self.0 = *bm;
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

    /// Wrapper tree node for the [`BestMetricTreeNode`].
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
                return TreeAction::Stop(BestMetric(*bm));
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

use self::metric::MetricDownwardState;

/// A metric for use with a [`BestMetricTreeNode`] tree search.
///
/// Typically metrics are numeric, often utilizing [`Infinitable`](infinitable::Infinitable) to have an
/// initial infinite value denoting that no metric has yet been set. This works
/// out based on the intuitive ordering of [`Infinitable`](infinitable::Infinitable) in that, for example,
/// any finite number is always less than (that is, a better metric if minimizing) than the initial infinite value.
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
/// Each transition from parent to child has an associated relative cost. Each path
/// from the root node to each successful terminal node then has a total cost. It
/// is this total cost that the tree search will optimize over the entire tree.
///
/// # Examples
/// For examples of the usage of this tree search method, see the
/// [2015 day 22 problem](../../advent_of_code/aoc_2015/day_22/solution/struct.Characters.html)
/// or the
/// [2021 day 23 problem](../../advent_of_code/aoc_2021/day_23/solution/struct.Position.html).
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
        metric::BestMetricNode(self)
            .traverse_tree(MetricDownwardState::default())
            .0
    }
}

/// Private module that performs the [`GlobalStateTreeNode`] search.
mod global {
    use super::general::*;
    use super::*;

    /// Upward state for the [`GlobalStateTreeNode`] search.
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

    /// Wrapper tree node for the [`GlobalStateTreeNode`].
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

/// A global state structure for use with a [`GlobalStateTreeNode`] tree search.
///
/// The state can contain any data needed for the specific solution.
pub trait GlobalState<N>: fmt::Debug {
    /// Updates the global state with information from a [`GlobalStateTreeNode`].
    fn update_with_node(&mut self, node: &N);

    /// Returns whether or not the search is complete based on the current global state.
    ///
    /// The will cause the tree search to stop and return the current global state.
    fn complete(&self) -> bool;
}

/// An action to be returned from [`GlobalStateTreeNode::recurse_action`].
pub enum GlobalAction<N: GlobalStateTreeNode> {
    /// Apply the current node to the global state by calling [`GlobalState::update_with_node`].
    Apply,
    /// Stop at this node without applying it to the global state.
    Stop,
    /// Continue recursing to child nodes.
    Continue(Vec<N>),
}

/// Implemented by a tree node, for which the tree search updates some global data that
/// implements [`GlobalState`].
///
/// The [`GlobalState`] can also prematurely terminate the search algorithm if problem
/// specific conditions have already been met.
///
/// # Examples
/// For examples of the usage of this tree search method, see [`LeastStepsTreeNode`]
/// or the
/// [2021 day 21 problem](../../advent_of_code/aoc_2021/day_21/solution/struct.GameNode.html).
pub trait GlobalStateTreeNode: Sized + fmt::Debug {
    /// The global state to use.
    type GlobalState: GlobalState<Self>;

    /// Returns the action for the algorithm to take for this node.
    fn recurse_action(&self, state: &Self::GlobalState) -> GlobalAction<Self>;

    /// Performs the search tree with this node as the root, initializing the algorithm
    /// with the initial [`GlobalState`], and returning the current [`GlobalState`] after the
    /// search is complete.
    fn traverse_tree(self, initial_state: Self::GlobalState) -> Self::GlobalState {
        let initial_state = Rc::new(RefCell::new(initial_state));

        Rc::try_unwrap(global::GlobalStateNode(self).traverse_tree(initial_state).0)
            .unwrap()
            .into_inner()
    }
}

/// A [`GlobalState`] that will terminate the tree search once the first solution
/// node is encountered.
///
/// This should be the type set in [`GlobalStateTreeNode::GlobalState`] for a node
/// structure implementing [`GlobalStateTreeNode`].
#[derive(Debug)]
pub struct FirstSolutionGlobalState<N> {
    /// The first solution encountered, if one has been found.
    solution: Option<N>,
}
impl<N> Default for FirstSolutionGlobalState<N> {
    fn default() -> Self {
        Self { solution: None }
    }
}
impl<N> FirstSolutionGlobalState<N> {
    /// Consumes the state, returning the first solution that was encountered, if one was found.
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

/// Private module that performs the actual [`LeastStepsTreeNode`] search.
mod least_steps {
    use super::*;
    use infinitable::Infinitable;

    /// [`Metric`] for the private least steps tree search.
    type LeastStepsMetric = Infinitable<usize>;
    impl Metric for LeastStepsMetric {
        const INITIAL_BEST: Self = Infinitable::Infinity;
        const INITIAL_COST: Self = Infinitable::Finite(0);

        fn is_better(&self, other: &Self) -> bool {
            *self < *other
        }
    }

    /// Wrapper tree node for the [`LeastStepsTreeNode`].
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

/// An action to be returned from [`LeastStepsTreeNode::recurse_action`].
pub enum LeastStepsAction<N> {
    /// This is a successful terminal node.
    StopSuccess,
    /// This is an unsuccessful terminal node.
    StopFailure,
    /// Recurse to some child nodes.
    Continue(Vec<N>),
}

/// Implemented by a tree node, for which the tree search determines the shortest path
/// to a successful leaf node.
///
/// That is, the minimal depth over all the successful leaf nodes.
///
/// # Examples
/// For examples of the usage of this tree search method, see the
/// [2015 day 19 problem](../../advent_of_code/aoc_2015/day_19/solution/struct.Molecule.html).
pub trait LeastStepsTreeNode: Sized + fmt::Debug + Eq + std::hash::Hash {
    /// Setting this to `true` will terminate the search once the first successful node
    /// is encountered.
    const STOP_AT_FIRST: bool = false;

    /// Returns the action for the algorithm to take at this node.
    fn recurse_action(&self) -> LeastStepsAction<Self>;

    /// Performs the tree search, returning the shortest path to reach a successful
    /// leaf node.
    fn least_steps(self) -> Option<usize> {
        match least_steps::LeastStepsNode(self).best_metric() {
            infinitable::Infinitable::Finite(steps) => Some(steps),
            _ => None,
        }
    }
}
