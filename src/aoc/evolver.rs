//! For cell arrays that can evolve in discrete steps.
//!
//! The [`Evolver`] trait can be implemented for cell arrays that can
//! evolve. For example a simulation of
//! [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life).
use std::{marker::PhantomData, rc::Rc};

/// Can be implemented for something that evolves in discrete steps and has
/// addressable cells.
///
/// For example, a [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) simulation.
/// Each cell has a value of type `T`.
pub trait Evolver<T> {
    /// Type that is used to address a single cell.
    type Point;

    /// Creates a new cell array in the default state based on the current
    /// cell array.
    fn next_default(other: &Self) -> Self;

    /// Set the cell value at the specified address.
    fn set_element(&mut self, point: &Self::Point, value: T);

    /// Given the address of a cell, should return the value of the that cell
    /// in the next step.
    fn next_cell(&self, point: &Self::Point) -> T;

    /// Should return an iterator over the cell addresses to be set in the
    /// next step.
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
    E: Evolver<T> + Clone,
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
