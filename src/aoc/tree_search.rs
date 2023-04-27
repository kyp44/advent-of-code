//! Potentially exhaustive search of a tree.
//!
//! TODO

use derive_new::new;

use super::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::Add,
};

// TODO: Do we need this or can we accomplish this with clever metrics?
pub enum SearchMethod<M> {
    FirstSolution,
    BestMetric {
        initial: M,
        initial_best: M,
        solved: M,
    },
}
// TODO: Ideas for versatility (Metric, Count, First solution, etc.):
// Some kind of general state trait that we implement for each
// Different overall TreeSearch traits for each of these
// How do we handle special case, e.g. counting wins for each player?

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

trait DownwardTreeState: Sized + Default {
    type UpwardState: UpwardTreeState<Self>;

    // Whether the state is such that we do not need to expand the associated node, and instead
    // just return the returned state.
    fn stop(&self) -> Option<Self::UpwardState>;
    fn new_upward_state(&self) -> Self::UpwardState;
}

trait UpwardTreeState<T> {
    fn incorporate_child(&mut self, child_state: &T);
}

/* struct MetricTreeState<M: Metric> {
    // The global best metric to get to a terminating leaf so far.
    global_best_cost: M,
    // The best metric to get to a terminating leaf when returned from a node recursion.
    best_cost: M,
    // The metric cost so far to get to this node.
    current_cost: M,
}
impl<M: Metric> Default for MetricTreeState<M> {
    fn default() -> Self {
        Self {
            global_best_cost: M::initial_best,
            best_cost: M::initial_best,
            current_cost: M::initial_cost,
        }
    }
}
impl<M: Metric> TreeState for MetricTreeState<M> {
    fn stop(&self) -> Option<Self> {
        if self.best_cost.is_better(&self.global_best_cost) {
            Some(Self {
                global_best_cost: self.best_cost,
                best_cost: todo!(),
                current_cost: todo!(),
            })
        } else {
            None
        }
    }

    fn new_for_children(&self) -> Self {
        Self {
            global_best_cost: self.global_best_cost,
            best_cost: M::initial_best,
            current_cost: M::initial_cost,
        }
    }

    fn incorporate_child(&mut self, child_state: &Self) {
        todo!()
    }
} */

#[derive(new)]
pub struct Child<N: TreeSearch> {
    node: N,
    state: N::DownwardState,
}

pub trait TreeSearch: Sized + Eq + Hash {
    type DownwardState: DownwardTreeState;

    fn stop(&self) -> Option<Self::DownwardState::UpwardState>;

    fn children(&self) -> Vec<Child<Self>>;

    fn search_tree(self) -> Self::State {
        fn rec<N: TreeSearch>(child: Child<N>, seen: &mut HashMap<N, N::State>) -> S::Metric {
            // Are we in a finished state?
            if let Some(state) = child.node.stop() {
                return state;
            }

            // Have we seen this state before?
            if let Some(state) = seen.get(&child.node) {
                return *state;
            }

            // Check if we are in a state that should stop us
            if let Some(state) = child.state.stop() {
                return state;
            }

            // Recurse for each leaf
            let mut state = child.state.new_for_children();
            for child in child.node.children() {
                state.incorporate_child(rec(child, seen));
            }

            // Mark this position as seen
            seen.insert(child.node, state);

            state
        }

        rec(
            Child::new(self, Self::State::default()),
            &mut HashMap::new(),
        )
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
