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

fn level_spaces(level: usize) -> String {
    let mut id = ID_COUNTER.write().unwrap();
    let s = format!(
        "{}ID_{} {}:",
        (0..level).map(|_| "    ").join(""),
        id,
        level,
    );
    *id += 1;
    s
}

const TODO: bool = false;
static ID_COUNTER: std::sync::RwLock<u64> = std::sync::RwLock::new(1);

// Represent the best metric needed to solve from the associated position.
pub struct BestMetric<N: BestMetricTreeNode>(N::Metric);
impl<N: BestMetricTreeNode + std::fmt::Debug> TreeUpwardState<N> for BestMetric<N> {
    fn new(current: &Child<N>) -> Self {
        if TODO {
            println!(
                "{} TODO About to process node {:?} with node cost {:?} and cum cost {:?}",
                level_spaces(current.state._level),
                current.node,
                current.state.node_cost,
                current.state.cumulative_cost,
            );
        }
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

            if TODO {
                println!(
                    "{} TODO Terminus cum: {:?}, global: {:?}",
                    level_spaces(current.state._level),
                    current.state.cumulative_cost,
                    global_state.best_metric,
                );
            }
            return Some(Self(current.state.node_cost));
        }

        // Is our cost already too high?
        if global_state
            .best_metric
            .is_better(&current.state.cumulative_cost)
        {
            if TODO {
                println!(
                    "{} TODO Cost too high global: {:?} current cum: {:?}",
                    level_spaces(current.state._level),
                    global_state.best_metric,
                    current.state.cumulative_cost,
                );
            }
            return Some(Self(N::Metric::INITIAL_BEST));
        }

        // Have we seen this node already?
        global_state.seen.get(&current.node).map(|bm| {
            if TODO {
                println!(
                    "{} TODO have seen node before global: {:?} current cum: {:?} seen: {:?} total: {:?}",
                    level_spaces(current.state._level),
                    global_state.best_metric,
                    current.state.cumulative_cost,
                    bm,
                    current.state.cumulative_cost + *bm,
                );
            }
            // Pass the best to solve from this node plus this node's cost, which is the best to solve
            // from the parent
            Self(bm.clone() + current.state.node_cost)
        })
    }

    fn incorporate_child(&mut self, current: &Child<N>, child: Self) {
        self.0.update_if_better(child.0);
        if TODO {
            println!(
                "{} TODO Incorporated child: best for child + this {:?} updated best: {:?}",
                level_spaces(current.state._level),
                child.0,
                self.0
            )
        }
    }

    fn finalize(&mut self, current: Child<N>) {
        let mut global_state = current.state.global_state.as_ref().borrow_mut();

        if TODO {
            println!(
                "{} TODO Done with node about to pass {:?} upward",
                level_spaces(current.state._level),
                self.0
            );
        }

        // Mark the seen metric as the best cost to solve from here
        global_state.seen.insert(current.node, self.0.clone());

        // Pass the best to solve from the previous position up
        self.0 = self.0 + current.state.node_cost;

        // Update the global best cost if better for the total cost whose solution passes
        // through this node.
        global_state
            .best_metric
            .update_if_better(current.state.cumulative_cost + self.0)
    }
}

pub trait BestMetricTreeNode: Sized + Eq + std::hash::Hash + std::fmt::Debug {
    type Metric: Metric + std::fmt::Debug;

    fn end_state(&self) -> bool;

    fn children(&self, cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>>;

    fn minimal_cost(self) -> Self::Metric {
        self.search_tree().0
    }
}
impl<N: BestMetricTreeNode + std::fmt::Debug> TreeNode for N {
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

        if TODO {
            println!(
                "{} TODO have {} children.",
                level_spaces(downward_state._level),
                x.len()
            );
        }
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

#[cfg(test)]
mod tests {
    use infinitable::Infinitable;

    use super::*;
    use std::fmt;

    const NUM_HEAPS: usize = 5;
    const NUM_ELEMENTS: u8 = 2;

    type Heaps = [u8; NUM_HEAPS];

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    enum Player {
        One,
        Two,
    }
    impl Into<u8> for Player {
        fn into(self) -> u8 {
            match self {
                Player::One => 1,
                Player::Two => 2,
            }
        }
    }
    impl fmt::Debug for Player {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", Into::<u8>::into(*self))
        }
    }
    impl Player {
        pub fn next(&self) -> Self {
            match self {
                Player::One => Player::Two,
                Player::Two => Player::One,
            }
        }
    }

    struct Move {
        taken: Heaps,
        next_state: NimState,
    }

    /// A normally played game of Nim.
    #[derive(PartialEq, Eq, Hash, Clone)]
    struct NimState {
        turn: Player,
        heaps: Heaps,
        history: Vec<Heaps>,
    }
    impl Default for NimState {
        fn default() -> Self {
            Self {
                turn: Player::One,
                heaps: [NUM_ELEMENTS; NUM_HEAPS],
                history: vec![],
            }
        }
    }
    impl fmt::Debug for NimState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{:?}: {}",
                self.turn,
                self.history
                    .iter()
                    .chain([self.heaps].iter())
                    .map(|heaps| format!("{:?}", heaps))
                    .join(" -> ")
            )
        }
    }
    impl NimState {
        pub fn winner(&self) -> Option<Player> {
            if self.heaps == [0; NUM_HEAPS] {
                Some(self.turn.next())
            } else {
                None
            }
        }

        pub fn moves(&self) -> impl Iterator<Item = Move> + '_ {
            (0..NUM_HEAPS).flat_map(move |i| {
                (1..=self.heaps[i]).map(move |t| {
                    let mut taken = [0; NUM_HEAPS];
                    taken[i] = t;

                    let mut next_heaps = self.heaps.clone();
                    next_heaps[i] -= t;
                    next_heaps.sort();

                    let mut history = self.history.clone();
                    history.push(self.heaps);

                    Move {
                        taken,
                        next_state: NimState {
                            turn: self.turn.next(),
                            heaps: next_heaps,
                            history,
                        },
                    }
                })
            })
        }
    }

    impl Metric for Infinitable<u8> {
        const INITIAL_BEST: Self = Infinitable::Infinity;
        const INITIAL_COST: Self = Infinitable::Finite(0);

        fn is_better(&self, other: &Self) -> bool {
            self < other
        }
    }

    impl BestMetricTreeNode for NimState {
        type Metric = Infinitable<u8>;

        fn end_state(&self) -> bool {
            match self.winner() {
                Some(p) => p == Player::One,
                None => false,
            }
        }

        fn children(&self, cumulative_cost: &Self::Metric) -> Vec<MetricChild<Self>> {
            self.moves()
                .map(|mv| {
                    /* let cost = if mv.next_state.turn.next() == Player::One {
                        mv.taken.into_iter().sum::<u8>().into()
                    } else {
                        0.into()
                    }; */
                    let cost = mv.taken.into_iter().sum::<u8>().into();
                    MetricChild::new(mv.next_state, cost)
                })
                .collect()
        }
    }

    #[test]
    fn best_metric() {
        assert_eq!(NimState::default().minimal_cost(), 10.into());
    }
}
