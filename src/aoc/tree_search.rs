//! Potentially exhaustive search of a tree.
//!
//! TODO

use derive_new::new;
use itertools::Itertools;

use std::{cell::RefCell, collections::HashMap, ops::Add, rc::Rc};

// TODO: Can we make the general stuff private if we can cover all use cases with more specific implementations?

#[derive(new)]
pub struct Child<N: TreeNode> {
    node: N,
    state: N::DownwardState,
}

trait TreeUpwardState<N: TreeNode>: Sized {
    fn new(current: &Child<N>) -> Self;

    fn stop(&self, current: &Child<N>) -> Option<Self>;

    fn incorporate_child(&mut self, current: &Child<N>, child_state: Self);

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

#[derive(Clone)]
struct MetricGlobalState<N: BestMetricTreeNode> {
    best_metric: N::Metric,
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
    global_state: Rc<RefCell<MetricGlobalState<N>>>,
    cumulative_cost: N::Metric,
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

fn level_spaces(level: usize) -> String {
    format!("{}{}:", (0..level).map(|_| "  ").join(""), level)
}

pub struct BestMetric<N: BestMetricTreeNode>(N::Metric);
impl<N: BestMetricTreeNode> TreeUpwardState<N> for BestMetric<N> {
    fn new(current: &Child<N>) -> Self {
        println!(
            "{} TODO About to process node with node cost {:?}",
            level_spaces(current.state._level),
            current.state.node_cost,
        );
        Self(N::Metric::INITIAL_BEST)
    }

    fn stop(&self, current: &Child<N>) -> Option<Self> {
        let mut global_state = current.state.global_state.as_ref().borrow_mut();

        // Are we at a terminal node?
        if current.node.end_state() {
            // Update global if better.
            global_state
                .best_metric
                .update_if_better(current.state.cumulative_cost);

            println!(
                "{} TODO Terminus global: {:?} current cum: {:?}",
                level_spaces(current.state._level),
                global_state.best_metric,
                current.state.cumulative_cost,
            );
            return Some(Self(N::Metric::INITIAL_COST));
        }

        // Is our cost already too high?
        if global_state
            .best_metric
            .is_better(&current.state.cumulative_cost)
        {
            println!(
                "{} TODO Cost too high global: {:?} current cum: {:?}",
                level_spaces(current.state._level),
                global_state.best_metric,
                current.state.cumulative_cost,
            );
            return Some(Self(N::Metric::INITIAL_BEST));
        }

        // Have we seen this node already?
        global_state.seen.get(&current.node).map(|bm| {
            println!(
                "{} TODO have seen node before global: {:?} current cum: {:?} seen: {:?} total: {:?}",
                level_spaces(current.state._level),
                global_state.best_metric,
                current.state.cumulative_cost,
                bm,
                current.state.cumulative_cost + *bm,
            );
            Self(bm.clone())
        })
    }

    fn incorporate_child(&mut self, current: &Child<N>, child: Self) {
        self.0.update_if_better(child.0 + current.state.node_cost);
        println!(
            "{} TODO Incorporated child: best for child + this {:?} updated best: {:?}",
            level_spaces(current.state._level),
            child.0 + current.state.node_cost,
            self.0
        )
    }

    fn finalize(&mut self, current: Child<N>) {
        let mut global_state = current.state.global_state.as_ref().borrow_mut();

        // Mark this as seen
        global_state.seen.insert(current.node, self.0.clone());
    }
}

pub trait BestMetricTreeNode: Sized + Eq + std::hash::Hash {
    type Metric: Metric + std::fmt::Debug;

    fn end_state(&self) -> bool;

    fn children(&self, cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>>;

    fn minimal_cost(self) -> Self::Metric {
        self.search_tree().0
    }
}
impl<N: BestMetricTreeNode> TreeNode for N {
    type DownwardState = MetricDownwardState<N>;
    type UpwardState = BestMetric<N>;

    fn node_children(&self, downward_state: &Self::DownwardState) -> Vec<Child<Self>> {
        let x: Vec<Child<Self>> = self
            .children(&downward_state.cumulative_cost)
            .into_iter()
            .map(|child| {
                Child::new(
                    child.node,
                    MetricDownwardState {
                        global_state: downward_state.global_state.clone(),
                        cumulative_cost: downward_state.cumulative_cost + child.cost,
                        node_cost: child.cost,
                        _level: downward_state._level + 1,
                    },
                )
            })
            .collect();

        println!(
            "{} TODO have {} children.",
            level_spaces(downward_state._level),
            x.len()
        );
        x
    }
}

// TODO: Potential uses
// 2015 - 22 - RPG with spells (min MP used)
// 2021 - 23 - Amphipods (Min energy)
//
// 2020 - 10 - Part 2, Joltage adapters (Count solutions)
// 2021 - 21 - Part 2, Dirac die (count of universes in which each player wins)
//
// 2020 - 20 - Part 1, Lining up images (Only care about first final solution)
