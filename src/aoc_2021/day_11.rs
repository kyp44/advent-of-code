use crate::aoc::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution_test;
    use Answer::Unsigned;

    solution_test! {
    vec![Unsigned(123)],
    "11111
19991
19191
19991
11111",
    vec![9u64].answer_vec(),
    "5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526",
    vec![1656u64].answer_vec()
    }
}

/*type Point = (usize, usize);

#[derive(CharGridDebug, Clone)]
struct Octopi {
    size: (usize, usize),
    energies: Box<[Box<[u8]>]>,
}
impl Grid for Octopi {
    type Element = u8;

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn element_at(&mut self, point: &GridPoint) -> &mut Self::Element {
        &mut self.energies[point.1][point.0]
    }
}
impl CharGrid for Octopi {
    /* TODO
    fn default_element() -> Self::Element {
        0
    }*/

    fn from_char(c: char) -> Self::Element {
        c.to_digit(10).unwrap().try_into().unwrap()
    }

    fn to_char(e: &Self::Element) -> char {
        char::from_digit((*e).into(), 10).unwrap()
    }
}
impl Octopi {
    fn evolve(&self) -> OctopiEvolver {
        OctopiEvolver {
            octopi: self.clone(),
            flashes: 0,
        }
    }

    fn energy(&mut self, point: &Point) -> &mut u8 {
        &mut self.energies[point.1][point.0]
    }

    fn energy_iter(&self) -> impl Iterator<Item = Point> {
        iproduct!(0..self.size.1, 0..self.size.0).map(|(y, x)| (x, y))
    }
}

struct OctopiEvolver {
    octopi: Octopi,
    flashes: u64,
}
impl Iterator for OctopiEvolver {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // Fist pass to increment all energies
        for point in self.octopi.energy_iter() {
            *self.octopi.energy(&point) += 1;
        }

        // Now repeated passes to look for flashes
        let mut flashes: HashSet<Point> = HashSet::new();
        loop {
            let mut had_flashes = false;

            for point in self.octopi.energy_iter() {
                let energy = self.octopi.energy(&point);
                if *energy > 8 && !flashes.contains(&point) {
                    // We have a new flash, increment neighbors

                    // Add flash
                    flashes.insert(point);
                    had_flashes = true;
                }
            }

            if !had_flashes {
                break;
            }
        }

        // Lastly, reset all energies that flashed
        for point in flashes {
            *self.octopi.energy(&point) = 0;
        }

        Some(self.flashes)
    }
}*/

pub const SOLUTION: Solution = Solution {
    day: 11,
    name: "Dumbo Octopus",
    solvers: &[
        // Part a)
        |input| {
            // Generation
            /*let octopi = Octopi::from_str(input)?;

            let mut evolver = octopi.evolve();
            for _ in 0..5 {
                println!("{:?}", evolver.octopi);
                evolver.next();
            }*/

            // Process
            Ok(0u64.into())
        },
    ],
};
