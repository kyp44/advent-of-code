//! For cell arrays that can evolve in discrete steps.
//!
//! The [`Evolver`] trait can be implemented for cell arrays that can
//! evolve.
use std::{marker::PhantomData, rc::Rc};

/// Can be implemented for something that evolves in discrete steps and has
/// addressable cells.
///
/// For example, a [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) simulation.
/// Each cell has a value of type `T`. An basic example of this using the Toad oscillator is provided below.
///
/// # Examples
/// Basic usage:
/// ```
/// use aoc::prelude::*;
/// use aoc::grid::StdBool;
/// use std::str::FromStr;
///
/// #[derive(Clone, Debug, PartialEq, Eq)]
/// struct ConwayCells {
///     grid: Grid<StdBool>,
/// }
/// impl From<Grid<StdBool>> for ConwayCells {
///     fn from(value: Grid<StdBool>) -> Self {
///         Self { grid: value }
///     }
/// }
/// impl Evolver<bool> for ConwayCells {
///     type Point = GridPoint;
///
///     fn next_default(other: &Self) -> Self {
///         Self {
///             grid: Grid::default(*other.grid.size()),
///         }
///     }
///
///     fn set_element(&mut self, point: &Self::Point, value: bool) {
///         self.grid.set(point, value.into())
///     }
///
///     fn next_cell(&self, point: &Self::Point) -> bool {
///         let live_neighbors: usize = self
///             .grid
///             .neighbor_points(point, true, false)
///             .filter_count(|p| **self.grid.get(p));
///         if **self.grid.get(point) {
///             // The current cell is live
///             live_neighbors == 2 || live_neighbors == 3
///         } else {
///             // The current cell is dead
///             live_neighbors == 3
///         }
///     }
///
///     fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>> {
///         self.grid.size().all_points()
///     }
/// }
///
/// // The first and initial frame of the Toad oscillator
/// let step_a: ConwayCells = Grid::from_str(
///     "....
/// .###
/// ####.
/// ....",
/// )
/// .unwrap()
/// .into();
/// // The second frame of the Toad oscillator
/// let step_b: ConwayCells = Grid::from_str(
///     "..#.
/// ##..#
/// ##..#
/// .#..",
/// )
/// .unwrap()
/// .into();
///
/// // The two frames of the Toad oscillator alternate forever.
/// let mut evolutions = step_a.evolutions();
/// for _ in 0..10 {
///     assert_eq!(evolutions.next().unwrap().as_ref(), &step_b);
///     assert_eq!(evolutions.next().unwrap().as_ref(), &step_a);
/// }
/// ```
pub trait Evolver<T> {
    /// Type that is used to address a single cell.
    type Point;

    /// Creates a new cell array in the default state based on the current
    /// cell array.
    ///
    /// This is used to create the initial cell array for the next step before
    /// actually setting each cell to the new value using [`Evolver::next_cell`].
    fn next_default(other: &Self) -> Self;

    /// Sets the cell value at the specified address.
    fn set_element(&mut self, point: &Self::Point, value: T);

    /// Returns the value of the addressed cell for the next step.
    fn next_cell(&self, point: &Self::Point) -> T;

    /// Returns an iterator over the cell addresses to be set in the
    /// next step using the [`Evolver::next_cell`] method.
    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>>;

    /// Returns an [`Iterator`] over the steps in the evolution of the cell array.
    ///
    /// The first element will be the next evolution, not the current array.
    fn evolutions(&self) -> EvolverIter<Self, T>
    where
        Self: Sized + Clone,
    {
        EvolverIter {
            current: Rc::new(self.clone()),
            _phant: PhantomData {},
        }
    }
}

/// [`Iterator`] to evolve an [`Evolver`] cell array.
///
/// `E` is the type that implements the [`Evolver`] trait, while `T` is
/// type of the cell values.
pub struct EvolverIter<E, T> {
    /// The current cell array.
    current: Rc<E>,
    /// Phantom data for the cell value type `T`.
    _phant: PhantomData<T>,
}
impl<E, T> Iterator for EvolverIter<E, T>
where
    E: Evolver<T>,
{
    type Item = Rc<E>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = E::next_default(&self.current);
        for point in self.current.next_iter() {
            next.set_element(&point, self.current.next_cell(&point));
        }
        self.current = Rc::new(next);
        Some(self.current.clone())
    }
}
