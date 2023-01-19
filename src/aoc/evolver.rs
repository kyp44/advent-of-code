use std::{marker::PhantomData, rc::Rc};

// Something that evolves and has cells. For example, a Conway's Game of Life simulation.
pub trait Evolver<T> {
    // Point to address elements
    type Point;

    // Create a new cell array for the next iteration in the default state.
    fn next_default(other: &Self) -> Self;

    // Get the value at the specified coordinates.
    fn get_element(&self, point: &Self::Point) -> T;

    // Set the value at the specified coordinates.
    fn set_element(&mut self, point: &Self::Point, value: T);

    // Given the coordinates of a cell, return the value of the same cell in the next step.
    fn next_cell(&self, point: &Self::Point) -> T;

    // Get an iterator over the cells to be set in the next step.
    fn next_iter(&self) -> Box<dyn Iterator<Item = Self::Point>>;

    // Iterate over the steps in the evolution of the cell array.
    // The first element will be the next evolution, not the current array.
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

// Iterator to evolve a cell array.
pub struct EvolverIter<E, T> {
    current: Rc<E>,
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
