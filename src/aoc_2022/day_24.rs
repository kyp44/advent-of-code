use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";
            answers = unsigned![18, 54];
        }
        actual_answers = unsigned![292, 816];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::{
        grid::Digit,
        tree_search::{ApplyNodeAction, LeastStepsTreeNode},
    };
    use euclid::{Box2D, Point2D, Translation2D, Vector2D};
    use itertools::Itertools;
    use std::collections::{HashMap, hash_map::Entry};

    /// The point type used for blizzard space.
    type Point<U> = Point2D<isize, U>;
    /// The vector type used for blizzard space.
    type Vector<U> = Vector2D<isize, U>;

    /// The cardinal directions.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Direction {
        /// Up.
        Up,
        /// Down.
        Down,
        /// Left.
        Left,
        /// Right.
        Right,
    }
    impl Direction {
        /// Returns a displacement vector to move one space in this direction.
        pub fn as_vector<U>(&self) -> Vector<U> {
            match self {
                Self::Up => -Vector::unit_y(),
                Self::Down => Vector::unit_y(),
                Self::Left => -Vector::unit_x(),
                Self::Right => Vector::unit_x(),
            }
        }

        /// Returns the optimum move order for a given `goal`.
        ///
        /// For example, for [`Goal::Entrance`], we are generally tying to move
        /// up and left, so these are the first two moves in the order.
        pub fn move_order(goal: Goal) -> impl Iterator<Item = Self> {
            match goal {
                Goal::Entrance => [Self::Up, Self::Left, Self::Down, Self::Right].into_iter(),
                Goal::Exit => [Self::Down, Self::Right, Self::Up, Self::Left].into_iter(),
            }
        }
    }

    /// The translation from grid space to blizzard space.
    static GRID_TO_BLIZZARD: Translation2D<isize, GridSpace, Blizzard> = Translation2D::new(-1, -1);

    /// A space in the problem grid, which can be parsed from text input.
    #[derive(Clone, Default, PartialEq, Eq)]
    enum Space {
        /// An empty/free space.
        Empty,
        /// A wall into which we cannot move.
        #[default]
        Wall,
        /// The location of our expedition.
        _Expedition,
        /// A single blizzard with its direction.
        Blizzard(Direction),
        /// Multiple blizzards with the number of them.
        MultiBlizzard(u8),
    }
    impl TryFrom<char> for Space {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                '.' => Self::Empty,
                '#' => Self::Wall,
                '^' => Self::Blizzard(Direction::Up),
                'v' => Self::Blizzard(Direction::Down),
                '<' => Self::Blizzard(Direction::Left),
                '>' => Self::Blizzard(Direction::Right),
                _ => {
                    return Err(());
                }
            })
        }
    }
    impl std::fmt::Debug for Space {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Empty => write!(f, "."),
                Self::Wall => write!(f, "#"),
                Self::_Expedition => write!(f, "E"),
                Self::Blizzard(dir) => match dir {
                    Direction::Up => write!(f, "^"),
                    Direction::Down => write!(f, "v"),
                    Direction::Left => write!(f, "<"),
                    Direction::Right => write!(f, ">"),
                },
                Self::MultiBlizzard(n) => write!(f, "{:?}", Digit(*n)),
            }
        }
    }

    /// The outlet to which we are trying to work towards.
    #[derive(Clone, Copy)]
    pub enum Goal {
        /// The upper outlet and entrance.
        Entrance,
        /// The lower outlet and exit.
        Exit,
    }

    /// A single blizzard, which is used when tracking blizzards to construct
    /// the blizzard states.
    struct Blizzard {
        /// The direction in which the blizzard moves.
        direction: Direction,
        /// The current position of the blizzard (in blizzard space).
        position: Point<Self>,
    }
    /// A state of all the blizzards.
    ///
    /// The key is the location of one or more blizzards and the value is
    /// the grid space corresponding to the blizzard(s) there.
    type BlizzardState = HashMap<Point<Blizzard>, Space>;

    /// The overall valley that needs to be traversed, which can be parsed from
    /// text input.
    pub struct Valley {
        /// The location of the upper/entrance outlet in blizzard space.
        upper_outlet: Point<Blizzard>,
        /// The location of the lower/exit outlet in blizzard space.
        lower_outlet: Point<Blizzard>,
        /// The bounds of blizzard space in which the expedition can move.
        ///
        /// Note that the two outlet points are outside these bounds.
        bounds: Box2D<isize, Blizzard>,
        /// The size of the debug display grid.
        _grid_size: GridSize,
        /// The finite list of blizzard states, noting that these are
        /// periodic so are calculated ahead of time.
        blizzard_states: Box<[BlizzardState]>,
    }
    impl FromStr for Valley {
        type Err = AocError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let grid = Grid::<Space>::from_str(s)?;

            let bounds = GRID_TO_BLIZZARD
                .transform_size((grid.size() - GridSize::new(2, 2)).try_cast().unwrap());

            // Build the blizzard states
            let mut blizzards = grid
                .all_points()
                .filter_map(|p| {
                    if let Space::Blizzard(dir) = grid.get(&p) {
                        Some(Blizzard {
                            direction: *dir,
                            position: GRID_TO_BLIZZARD.transform_point(p.try_cast().unwrap()),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Box<[_]>>();

            let blizzard_states = (0..num::integer::lcm(bounds.width, bounds.height))
                .map(|_| {
                    // Convert blizzards to state while also advancing them
                    let mut state = BlizzardState::new();

                    for blizzard in blizzards.iter_mut() {
                        // Add/update state for this point
                        let entry = state.entry(blizzard.position);
                        match entry {
                            Entry::Occupied(occ) => {
                                let space = occ.into_mut();
                                *space = match space {
                                    Space::Blizzard(_) => Space::MultiBlizzard(2),
                                    Space::MultiBlizzard(n) => Space::MultiBlizzard(*n + 1),
                                    _ => panic!(),
                                };
                            }
                            Entry::Vacant(vac) => {
                                vac.insert(Space::Blizzard(blizzard.direction));
                            }
                        }

                        blizzard.position = (blizzard.position + blizzard.direction.as_vector())
                            .rem_euclid(&bounds);
                    }

                    state
                })
                .collect();

            let bounds = Box2D::from_size(bounds);
            Ok(Self {
                bounds,
                upper_outlet: Point::new(0, -1),
                lower_outlet: bounds.max + Vector::new(-1, 0),
                _grid_size: grid.size(),
                blizzard_states,
            })
        }
    }
    impl Valley {
        /// Returns the debug grid to be displayed, which does not include the
        /// location of the expedition.
        fn _debug_grid(
            &self,
            blizzard_state: &BlizzardState,
            expedition: Option<Point<Blizzard>>,
        ) -> Grid<Space> {
            let mut grid = Grid::default(self._grid_size);

            // Set entry and exit points
            grid.set(&GridPoint::new(1, 0), Space::Empty);
            grid.set(
                &(grid.size() - GridSize::<GridSpace>::new(2, 1).cast_unit())
                    .to_vector()
                    .to_point(),
                Space::Empty,
            );

            let blizzard_to_grid = GRID_TO_BLIZZARD.inverse();
            for blizzard_point in self.bounds.all_points() {
                grid.set(
                    &blizzard_to_grid
                        .transform_point(blizzard_point)
                        .try_cast()
                        .unwrap(),
                    match blizzard_state.get(&blizzard_point) {
                        Some(s) => s.clone(),
                        None => Space::Empty,
                    },
                );
            }

            if let Some(exp) = expedition {
                grid.set(
                    &blizzard_to_grid.transform_point(exp).try_cast().unwrap(),
                    Space::_Expedition,
                );
            }

            grid
        }

        /// Uses tree searches to calculate the minimal amount of time (in
        /// minutes) to traverse the valley multiple times given a list
        /// of subsequent `goals`.
        pub fn minimal_time(&self, goals: &[Goal]) -> AocResult<u64> {
            let mut total_time = 0;

            for goal in goals {
                total_time += SearchNode::new(self, *goal, total_time).traverse_tree()?;
            }

            Ok(total_time.try_into().unwrap())
        }
    }

    /// The actual data contained in a search tree node.
    #[derive(Clone)]
    struct NodeData<'a> {
        /// The valley we are traversing.
        valley: &'a Valley,
        /// The time that has elapsed to get to the current location of the
        /// expedition.
        time: usize,
        /// The current location of the expedition.
        expedition: Point<Blizzard>,
        /// The goal outlet we are working towards.
        goal: Goal,
    }
    impl<'a> NodeData<'a> {
        /// Initializes a new node data for a given `goal` and an initial
        /// `start_time` in minutes, which simply determines the initial
        /// state of the blizzards.
        pub fn new(valley: &'a Valley, goal: Goal, start_time: usize) -> Self {
            Self {
                time: start_time,
                valley,
                expedition: match goal {
                    Goal::Entrance => valley.lower_outlet,
                    Goal::Exit => valley.upper_outlet,
                },
                goal,
            }
        }

        /// Returns the point to which we are working in blizzard space.
        pub fn goal_point(&self) -> Point<Blizzard> {
            match self.goal {
                Goal::Entrance => self.valley.upper_outlet,
                Goal::Exit => self.valley.lower_outlet,
            }
        }

        /// Returns the blizzard state `n` steps ahead of the current blizzard
        /// state.
        pub fn relative_blizzard_state(&self, n: usize) -> &'a BlizzardState {
            &self.valley.blizzard_states[self.blizzard_idx(self.time + n)]
        }

        /// Returns the periodic index in [`Valley::blizzard_states`]
        /// corresponding to the elapsed `time` in minutes.
        fn blizzard_idx(&self, time: usize) -> usize {
            time % self.valley.blizzard_states.len()
        }

        /// Returns a new node data in which the expedition has moved one space
        /// in the specified `direction` over a minute, if moving in
        /// this direction is possible.
        pub fn move_expedition(&self, direction: Direction) -> Option<Self> {
            let new_point = self.expedition + direction.as_vector();

            (!self.relative_blizzard_state(1).contains_key(&new_point)
                && (self.valley.bounds.contains(new_point) || new_point == self.goal_point()))
            .then(|| Self {
                valley: self.valley,
                time: self.time + 1,
                expedition: new_point,
                goal: self.goal,
            })
        }

        /// Returns a new node data for which the expedition stays in its
        /// current location for a minute, if staying is possible, that
        /// is, staying will _not_ cause the expedition to be caught in
        /// a blizzard that has moved here.
        pub fn stay(&self) -> Option<Self> {
            (!self
                .relative_blizzard_state(1)
                .contains_key(&self.expedition))
            .then(|| Self {
                valley: self.valley,
                time: self.time + 1,
                expedition: self.expedition,
                goal: self.goal,
            })
        }
    }
    impl std::fmt::Debug for NodeData<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Minute: {}, Location: {:?}, Blizzard idx: {}",
                self.time,
                self.expedition,
                self.blizzard_idx(self.time),
            )
        }
    }
    impl std::hash::Hash for NodeData<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            std::ptr::hash(self.valley, state);
            self.time.hash(state);
            self.expedition.hash(state);
        }
    }
    impl PartialEq for NodeData<'_> {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self.valley, other.valley)
                && self.expedition == other.expedition
                && self.time == other.time
        }
    }
    impl Eq for NodeData<'_> {}

    /// The actual tree search node.
    #[derive(Clone)]
    struct SearchNode<'a> {
        /// The data for this node.
        data: NodeData<'a>,
        // Field useful for debugging
        //path: Vec<NodeData<'a>>,
    }
    impl<'a> SearchNode<'a> {
        /// Initializes a starting node for a tree search, given a `goal` and an
        /// initial `start_time` in minutes, which simply determines the
        /// initial state of the blizzards.
        pub fn new(valley: &'a Valley, goal: Goal, start_time: usize) -> Self {
            let data = NodeData::new(valley, goal, start_time);

            Self {
                data: data.clone(),
                //path: vec![data],
            }
        }

        /// Returns a new tree node with the new `data`.
        pub fn next(&self, data: NodeData<'a>) -> Self {
            //let mut path = self.path.clone();
            //path.push(data.clone());
            Self {
                data,
                //path,
            }
        }
    }
    impl std::fmt::Debug for SearchNode<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.data)
        }
    }
    impl std::hash::Hash for SearchNode<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.data.hash(state);
        }
    }
    impl PartialEq for SearchNode<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.data == other.data
        }
    }
    impl Eq for SearchNode<'_> {}

    impl LeastStepsTreeNode for SearchNode<'_> {
        fn recurse_action(&mut self) -> ApplyNodeAction<Self> {
            // Are we are we at the exit?
            if self.data.expedition == self.data.goal_point() {
                return ApplyNodeAction::Stop(true);
            }

            // Set children for directions in which we can move
            let mut children = Direction::move_order(self.data.goal)
                .filter_map(|dir| self.data.move_expedition(dir).map(|data| self.next(data)))
                .collect_vec();

            // Add a node where we stay if we can
            if let Some(data) = self.data.stay() {
                children.push(self.next(data));
            }

            if children.is_empty() {
                ApplyNodeAction::Stop(false)
            } else {
                ApplyNodeAction::Continue(children)
            }
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 24,
    name: "Blizzard Basin",
    preprocessor: Some(|input| Ok(Box::new(Valley::from_str(input)?).into())),
    solvers: &[
        // Part one
        |input| {
            // Process
            Ok(input
                .expect_data::<Valley>()?
                .minimal_time(&[Goal::Exit])?
                .into())
        },
        // Part two
        |input| {
            // Process
            Ok(input
                .expect_data::<Valley>()?
                .minimal_time(&[Goal::Exit, Goal::Entrance, Goal::Exit])?
                .into())
        },
    ],
};
