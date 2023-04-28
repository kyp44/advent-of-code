//! Potentially exhaustive search of a tree.
//!
//! TODO

use derive_new::new;

use std::{collections::HashMap, hash::Hash, ops::Add};

// TODO: Ideas for versatility (Metric, Count, First solution, etc.):
// Some kind of general state trait that we implement for each
// Different overall TreeSearch traits for each of these
// How do we handle special case, e.g. counting wins for each player?

#[derive(new)]
pub struct Child<N: TreeSearch> {
    node: N,
    state: N::DownwardState,
}

pub trait TreeSearch: Sized + Eq + Hash {
    type DownwardState: DownwardTreeState<Self::UpwardState>;
    type UpwardState: UpwardTreeState;

    fn stop(&self) -> Option<Self::UpwardState>;

    fn children(&self) -> Vec<Child<Self>>;

    fn search_tree(self) -> Self::UpwardState {
        fn rec<N: TreeSearch>(
            child: Child<N>,
            seen: &mut HashMap<N, N::UpwardState>,
        ) -> N::UpwardState {
            // Are we in a finished state?
            if let Some(state) = child.node.stop() {
                return state;
            }

            // Have we seen this state before?
            if let Some(state) = seen.get(&child.node) {
                return state.clone();
            }

            // Check if we are in a state that should stop us
            if let Some(state) = child.state.stop() {
                return state;
            }

            // Recurse for each leaf
            let mut state = child.state.new_upward_state();
            for child in child.node.children() {
                state.incorporate_child(&rec(child, seen));
            }

            // Mark this position as seen
            seen.insert(child.node, state.clone());

            state
        }

        rec(
            Child::new(self, Self::DownwardState::default()),
            &mut HashMap::new(),
        )
    }
}

trait Metric: Add<Output = Self> + Copy {
    const initial_best: Self;
    const initial_cost: Self;

    fn is_better(&self, other: &Self) -> bool;

    fn update_if_better(&mut self, other: Self) {
        if other.is_better(self) {
            *self = other;
        }
    }
}

pub trait DownwardTreeState<U>: Sized + Default {
    // Whether the state is such that we do not need to expand the associated node, and instead
    // just return the returned state.
    fn stop(&self) -> Option<U>;
    fn new_upward_state(&self) -> U;
}

pub trait UpwardTreeState: Clone {
    fn incorporate_child(&mut self, child_upward_state: &Self);
}

struct MetricDownwardState<M: Metric> {
    global_best_cost: M,
    cumulative_cost: M,
    cost: M,
}
impl<M: Metric> Default for MetricDownwardState<M> {
    fn default() -> Self {
        Self {
            global_best_cost: M::initial_best,
            cumulative_cost: M::initial_cost,
            cost: M::initial_cost,
        }
    }
}
impl<M: Metric> DownwardTreeState<MetricUpwardState<M>> for MetricDownwardState<M> {
    fn stop(&self) -> Option<MetricUpwardState<M>> {
        if self.global_best_cost.is_better(&self.cumulative_cost) {
            Some(MetricUpwardState::new(
                self.global_best_cost,
                M::initial_best,
            ))
        } else {
            None
        }
    }

    fn new_upward_state(&self) -> MetricUpwardState<M> {
        MetricUpwardState::new(self.global_best_cost, M::initial_best)
    }
}

#[derive(Clone, new)]
struct MetricUpwardState<M: Metric> {
    global_best_cost: M,
    best_cost: M,
}
impl<M: Metric> UpwardTreeState for MetricUpwardState<M> {
    fn incorporate_child(&mut self, child_upward_state: &Self) {
        let x = self
            .self
            .best_cost
            .update_if_better(child_upward_state.best_cost);
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
