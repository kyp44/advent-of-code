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

        fn incorporate_child(&mut self, current: &Child<N>, child_upward_state: Self);

        fn finalize(&mut self, current: Child<N>);
    }

    pub enum TreeAction<N: TreeNode> {
        Stop(N::UpwardState),
        Continue(Vec<Child<N>>),
    }

    pub trait TreeNode: Sized {
        type DownwardState: Default;
        type UpwardState: TreeUpwardState<Self>;

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
                        upward_state.finalize(current);
                    }
                }

                upward_state
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

        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self> {
            let mut global_state = downward_state.global_state.as_ref().borrow_mut();

            // Is our cost already too high?
            if global_state
                .best_metric
                .is_better(&downward_state.cumulative_cost)
            {
                return TreeAction::Stop(BestMetric(N::Metric::INITIAL_BEST));
            }

            // Have we seen this node already?
            if let Some(bm) = global_state.seen.get(&self.0) {
                // Pass the best to solve from this node plus this node's cost, which is the best to solve
                // from the parent
                return TreeAction::Stop(BestMetric(bm.clone() + downward_state.node_cost));
            }

            match self.0.children(&downward_state.cumulative_cost) {
                BestMetricAction::Stop => {
                    // Update global if better.
                    global_state
                        .best_metric
                        .update_if_better(downward_state.cumulative_cost);

                    return TreeAction::Stop(BestMetric(downward_state.node_cost));
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

pub enum BestMetricAction<N: BestMetricTreeNode> {
    Stop,
    Continue(Vec<MetricChild<N>>),
}

pub trait BestMetricTreeNode: Sized + Eq + std::hash::Hash {
    type Metric: Metric;

    fn children(&self, cumulative_cost: &Self::Metric) -> BestMetricAction<Self>;

    fn best_metric(self) -> Self::Metric {
        metric::BestMetricNode(self).traverse_tree().0
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

        fn incorporate_child(
            &mut self,
            current: &Child<GlobalStateNode<N>>,
            child_upward_state: Self,
        ) {
            // Do nothing
        }

        fn finalize(&mut self, current: Child<GlobalStateNode<N>>) {
            // Do nothing
        }
    }

    pub struct GlobalStateNode<N: GlobalStateTreeNode>(pub N);
    impl<N: GlobalStateTreeNode> TreeNode for GlobalStateNode<N> {
        type DownwardState = Rc<RefCell<N::GlobalState>>;
        type UpwardState = GlobalUpwardState<N>;

        fn recurse_action(&self, downward_state: &Self::DownwardState) -> TreeAction<Self> {
            let mut global_state = downward_state.as_ref().borrow_mut();

            match self.0.node_children(&global_state) {
                GlobalAction::Apply => {
                    global_state.update_with_node(&self.0);
                    TreeAction::Stop(GlobalUpwardState(downward_state.clone()))
                }
                GlobalAction::Stop => TreeAction::Stop(GlobalUpwardState(downward_state.clone())),
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
}

pub enum GlobalAction<N: GlobalStateTreeNode> {
    Apply,
    Stop,
    Continue(Vec<N>),
}

pub trait GlobalStateTreeNode: Sized {
    type GlobalState: GlobalState<Self>;

    fn node_children(&self, state: &Self::GlobalState) -> GlobalAction<Self>;

    fn traverse_tree(self) -> Self::GlobalState {
        Rc::try_unwrap(global::GlobalStateNode(self).traverse_tree().0)
            .unwrap()
            .into_inner()
    }
}

// TODO: Potential uses
// X 2015 - 22 - RPG with spells (min MP used)
// X 2021 - 23 - Amphipods (Min energy)
//
// X 2020 - 10 - Part 2, Joltage adapters (Count solutions)
// X 2021 - 21 - Part 2, Dirac die (count of universes in which each player wins)
//
// 2020 - 20 - Part 1, Lining up images (Only care about first final solution)
