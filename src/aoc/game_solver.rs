//! Exhaustive solver for game states.
//!
//! TODO

use super::prelude::*;
use std::collections::HashSet;

trait GameSolver: Sized + Eq {
    type Metric: Copy;

    fn solved(&self) -> bool;

    fn next_states(&self) -> Vec<Self>;

    fn metric(&self) -> Self::Metric;

    fn compare_metric(better: &Self::Metric, than: &Self::Metric) -> bool;

    /* fn solve(&self) {
        fn rec<S: GameSolver>(
            state: S,
            seen: &mut HashSet<S>,
            global_opt_energy: &mut Option<S::Metric>,
            current_energy: u64,
        ) -> Option<S::Metric> {
            // Are we in a solved position?
            if state.solved() {
                return Some(state.metric());
            }

            // Have we seen this state before?
            if let Some(e) = seen.get(&position) {
                return *e;
            }

            // Are we already a larger energy than the global minimum?
            if let Some(e) = global_min_energy && current_energy >= e {
                return None;
            }

            //println!("Level {_level}:\n{}", position);

            // Recurse for each possible move
            let mut min_energy: Option<u64> = None;
            for mv in position.moves() {
                if let Some(e) = rec(
                    mv.new_position,
                    seen,
                    global_min_energy,
                    current_energy + mv.energy,
                    _level + 1,
                ) {
                    min_energy.update_min(e + mv.energy);
                    if let Some(me) = min_energy {
                        global_min_energy.update_min(me);
                    }
                }
            }

            // Mark this position as seen
            seen.insert(position, min_energy);

            min_energy
        }

        rec(self, &mut HashMap::new(), None, 0, 0).ok_or(AocError::NoSolution)
    } */
}

// TODO: Potential uses
// 2015 - 22 - RPG with spells
// 2020 - 10 - Part 2, Joltage adapters
// 2020 - 20 - Part 1, Lining up images
// 2021 - 20 - Part 2, Dirac die
// 2021 - 23 - Amphipods
