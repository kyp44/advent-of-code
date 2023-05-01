//! Potentially exhaustive search of a tree.
//!
//! TODO

use derive_new::new;
use num::PrimInt;

use std::{collections::HashMap, hash::Hash, ops::Add};

// TODO: Ideas for versatility (Metric, Count, First solution, etc.):
// Some kind of general state trait that we implement for each
// Different overall TreeSearch traits for each of these
// How do we handle special case, e.g. counting wins for each player?

#[derive(new)]
pub struct Child<N: TreeNode> {
    node: N,
    state: N::DownwardState,
}

pub trait TreeNode: Sized {
    type DownwardState: Default;
    type GlobalState: Default;

    fn node_children(
        &self,
        downward_state: &Self::DownwardState,
        global_state: &mut Self::GlobalState,
    ) -> Vec<Child<Self>>;

    fn search_tree(self) -> Self::GlobalState {
        fn rec<N: TreeNode>(current: Child<N>, global_state: &mut N::GlobalState) {
            // Recurse for each leaf
            for child in current.node.node_children(&current.state, global_state) {
                rec(child, global_state);
            }
        }

        let mut global_state = Self::GlobalState::default();
        rec(
            Child::new(self, Self::DownwardState::default()),
            &mut global_state,
        );
        global_state
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
pub struct MetricCost<M: Metric>(M);
impl<M: Metric> Default for MetricCost<M> {
    fn default() -> Self {
        Self(M::INITIAL_COST)
    }
}
pub struct MetricBest<M: Metric>(M);
impl<M: Metric> Default for MetricBest<M> {
    fn default() -> Self {
        Self(M::INITIAL_BEST)
    }
}

pub trait BestMetricTreeNode: Sized {
    type Metric: Metric + std::fmt::Debug;

    fn end_state(&self) -> bool;

    fn children(&self, cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>>;

    fn minimal_cost(self) -> Self::Metric {
        self.search_tree().0
    }
}
impl<N: BestMetricTreeNode> TreeNode for N {
    type DownwardState = MetricCost<N::Metric>;
    type GlobalState = MetricBest<N::Metric>;

    fn node_children(
        &self,
        downward_state: &Self::DownwardState,
        global_state: &mut Self::GlobalState,
    ) -> Vec<Child<Self>> {
        // Are we at an end node?
        // TODO: What about if we've seen this state before?
        if self.end_state() {
            // We are at an end node so update global best if we beat it
            global_state.0.update_if_better(downward_state.0);
            //println!("TODO End node! {:?}", global_state.0);
            vec![]
        } else if global_state.0.is_better(&downward_state.0) {
            // Our cost is already too high so just stop
            /* println!(
                "TODO Stopped early {:?} {:?}!",
                global_state.0, downward_state.0,
            ); */
            vec![]
        } else {
            self.children(&downward_state.0)
                .into_iter()
                .map(|child| Child::new(child.node, MetricCost(downward_state.0 + child.cost)))
                .collect()
        }
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
