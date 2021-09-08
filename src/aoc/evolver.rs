use std::marker::PhantomData;

/// Something that evolves, typically a Conway's Game of Life type cell array.
pub trait Evolver<C, T> {
    type Iter: Iterator<Item = C>;

    /// Create a new cell array in the default state.
    fn new(other: &Self) -> Self;

    /// Get the value at the specified coordinates.
    fn get(&self, pos: &C) -> T;

    /// Set the value at the specified coordinates.
    fn set(&mut self, pos: &C, val: T);

    /// Given the coordinates of a cell, return the value of the same cell in the next step.
    fn next_cell(&self, pos: &C) -> T;

    /// Get an iterator over the cells to be set in the next step.
    fn next_iter(&self) -> Self::Iter;

    /// Iterator over the steps in the evolution of the cell array.
    fn iter(&self) -> EvolverIter<Self, C, T>
    where
        Self: Sized + Clone,
    {
        EvolverIter {
            current: self.clone(),
            _phant1: PhantomData {},
            _phant2: PhantomData {},
        }
    }
}

/// Iterator to evolve a cell array.
pub struct EvolverIter<E, C, T> {
    current: E,
    _phant1: PhantomData<C>,
    _phant2: PhantomData<T>,
}
impl<E, C, T> Iterator for EvolverIter<E, C, T>
where
    E: Evolver<C, T> + Clone,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = E::new(&self.current);
        for coord in self.current.next_iter() {
            next.set(&coord, self.current.next_cell(&coord));
        }
        self.current = next.clone();
        Some(next)
    }
}
