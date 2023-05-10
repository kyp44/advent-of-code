//! Potentially exhaustive search of a tree.
//!
//! TODO

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

        fn stop(&self, current: &Child<N>) -> Option<Self>;

        fn incorporate_child(&mut self, current: &Child<N>, child_upward_state: Self);

        fn finalize(&mut self, current: Child<N>);
    }

    pub trait TreeNode: Sized {
        type DownwardState: Default;
        type UpwardState: TreeUpwardState<Self>;

        fn node_children(&self, downward_state: &Self::DownwardState) -> Vec<Child<Self>>;

        fn search_tree(self) -> Self::UpwardState {
            fn rec<N: TreeNode>(current: Child<N>) -> N::UpwardState {
                let mut upward_state = N::UpwardState::new(&current);
                match upward_state.stop(&current) {
                    Some(s) => s,
                    None => {
                        // Recurse for each leaf
                        for child in current.node.node_children(&current.state) {
                            upward_state.incorporate_child(&current, rec(child));
                        }
                        upward_state.finalize(current);
                        upward_state
                    }
                }
            }

            rec(Child::new(self, Self::DownwardState::default()))
        }
    }
}

mod metric {
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
    pub struct BestMetric<N: BestMetricTreeNode>(pub N::Metric);
    impl<N: BestMetricTreeNode> TreeUpwardState<BestMetricNode<N>> for BestMetric<N> {
        fn new(current: &Child<BestMetricNode<N>>) -> Self {
            Self(N::Metric::INITIAL_BEST)
        }

        fn stop(&self, current: &Child<BestMetricNode<N>>) -> Option<Self> {
            let mut global_state = current.state.global_state.as_ref().borrow_mut();

            // Are we at a terminal node?
            if current.node.0.end_state() {
                // Update global if better.
                global_state
                    .best_metric
                    .update_if_better(current.state.cumulative_cost);

                return Some(Self(current.state.node_cost));
            }

            // Is our cost already too high?
            if global_state
                .best_metric
                .is_better(&current.state.cumulative_cost)
            {
                return Some(Self(N::Metric::INITIAL_BEST));
            }

            // Have we seen this node already?
            global_state.seen.get(&current.node.0).map(|bm| {
                // Pass the best to solve from this node plus this node's cost, which is the best to solve
                // from the parent
                Self(bm.clone() + current.state.node_cost)
            })
        }

        fn incorporate_child(
            &mut self,
            current: &Child<BestMetricNode<N>>,
            child_upward_state: Self,
        ) {
            self.0.update_if_better(child_upward_state.0);
        }

        fn finalize(&mut self, current: Child<BestMetricNode<N>>) {
            let mut global_state = current.state.global_state.as_ref().borrow_mut();

            // Mark the seen metric as the best cost to solve from here
            global_state.seen.insert(current.node.0, self.0.clone());

            // Pass the best to solve from the previous position up
            self.0 = self.0 + current.state.node_cost;

            // Update the global best cost if better for the total cost whose solution passes
            // through this node.
            global_state
                .best_metric
                .update_if_better(current.state.cumulative_cost + self.0)
        }
    }

    pub struct BestMetricNode<N: BestMetricTreeNode>(pub N);
    impl<N: BestMetricTreeNode> TreeNode for BestMetricNode<N> {
        type DownwardState = MetricDownwardState<N>;
        type UpwardState = BestMetric<N>;

        fn node_children(&self, downward_state: &Self::DownwardState) -> Vec<Child<Self>> {
            self.0
                .children(&downward_state.cumulative_cost)
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
                .collect()
        }
    }
}

use general::TreeNode;

pub trait Metric: Add<Output = Self> + Copy {
    const INITIAL_BEST: Self;
    const INITIAL_COST: Self;

    fn is_better(&self, other: &Self) -> bool;

    fn update_if_better(&mut self, other: Self) {
        if other.is_better(self) {
            *self = other;
        }
    }
}

#[derive(new)]
pub struct MetricChild<N: BestMetricTreeNode> {
    node: N,
    cost: N::Metric,
}

pub trait BestMetricTreeNode: Sized + Eq + std::hash::Hash {
    type Metric: Metric;

    fn end_state(&self) -> bool;

    fn children(&self, cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>>;

    fn best_metric(self) -> Self::Metric {
        metric::BestMetricNode(self).search_tree().0
    }
}

mod global {
    use super::general::*;
    use super::*;

    pub struct GlobalUpwardState<N: GlobalStateTreeNode>(pub Rc<RefCell<N::GlobalState>>);
    impl<N: GlobalStateTreeNode> TreeUpwardState<GlobalStateNode<N>> for GlobalUpwardState<N> {
        fn new(current: &Child<GlobalStateNode<N>>) -> Self {
            Self(current.state.clone())
        }

        fn stop(&self, current: &Child<GlobalStateNode<N>>) -> Option<Self> {
            // Never return early
            None
        }

        fn incorporate_child(
            &mut self,
            current: &Child<GlobalStateNode<N>>,
            child_upward_state: Self,
        ) {
            // Do nothing
        }

        fn finalize(&mut self, current: Child<GlobalStateNode<N>>) {
            if current.node.0.apply_to_state() {
                self.0
                    .as_ref()
                    .borrow_mut()
                    .update_with_node(&current.node.0)
            }
        }
    }

    pub struct GlobalStateNode<N: GlobalStateTreeNode>(pub N);
    impl<N: GlobalStateTreeNode> TreeNode for GlobalStateNode<N> {
        type DownwardState = Rc<RefCell<N::GlobalState>>;
        type UpwardState = GlobalUpwardState<N>;

        fn node_children(&self, downward_state: &Self::DownwardState) -> Vec<Child<Self>> {
            self.0
                .node_children()
                .into_iter()
                .map(|node| Child::new(Self(node), downward_state.clone()))
                .collect()
        }
    }
}

pub trait GlobalState<N>: Default + fmt::Debug {
    fn update_with_node(&mut self, node: &N);
}

pub trait GlobalStateTreeNode: Sized {
    type GlobalState: GlobalState<Self>;

    fn node_children(&self) -> Vec<Self>;

    fn apply_to_state(&self) -> bool {
        true
    }

    fn traversal_state(self) -> Self::GlobalState {
        Rc::try_unwrap(global::GlobalStateNode(self).search_tree().0)
            .unwrap()
            .into_inner()
    }
}

#[derive(Debug, Default)]
pub struct CountLeaves {
    leaves: usize,
}
impl CountLeaves {
    pub fn count(&self) -> usize {
        self.leaves
    }
}
impl<N: GlobalStateTreeNode> GlobalState<N> for CountLeaves {
    fn update_with_node(&mut self, node: &N) {
        self.leaves += 1;
    }
}

// TODO: Potential uses
// X 2015 - 22 - RPG with spells (min MP used)
// X 2021 - 23 - Amphipods (Min energy)
//
// X 2020 - 10 - Part 2, Joltage adapters (Count solutions)
// 2021 - 21 - Part 2, Dirac die (count of universes in which each player wins)
//
// 2020 - 20 - Part 1, Lining up images (Only care about first final solution)
