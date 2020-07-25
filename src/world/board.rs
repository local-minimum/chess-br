use rand::Rng;

use crate::world::position::{Coord, Positional};
use crate::world::Direction;

pub trait Board {
    fn shape(&self) -> Coord;
    fn new_with<T: Clone>(&self, value: T) -> Vec<Vec<T>>;
    fn new_when(&self, other: &Self, other_value: u16, fill: u16) -> Self;
    fn fill(&mut self, c1: &Coord, c2: &Coord, when: u16, value: u16);
    fn mark_rnd_position(&mut self, value: u16) -> Coord;
    fn max_val(&self) -> u16;
    fn max_when(&self, other: &Self, other_value: u16) -> u16;
    fn coords_of(&self, value: u16) -> Vec<Coord>;
    fn coords_of_lambda(&self, test: &dyn Fn(u16) -> bool) -> Vec<Coord>;
    fn coords_not_of(&self, value: u16) -> Vec<Coord>;
    fn coords_when(&self, value: u16, other: &Self, other_value: u16) ->  Vec<Coord>;
    fn apply(&mut self, coords: &Vec<Coord>, value: u16);
    fn apply_when(&mut self, value: u16, other: &Self, other_value: u16);
    fn neighbour_min(&self, coord: &Coord, edge: &Coord) -> u16;
    fn neighbour_has_lambda(&self, coord: &Coord, out_of_bound_true: bool, test: &dyn Fn(u16, u16) -> bool) -> bool;
}

fn min_non_zero(a: u16, b: u16) -> u16 {
    if a == 0 { return  b;}
    if b == 0 { return  a;}
    a.min(b)
}

impl Board for Vec<Vec<u16>> {
    fn shape(&self) -> Coord {
        Coord {
            x: self[0].len(),
            y: self.len(),
        }
    }
    
    fn new_with<U: Clone>(&self, value: U) -> Vec<Vec<U>> {
        let s = self.shape();
        vec![vec![value; s.x]; s.y]
    }

    fn new_when(&self, other: &Self, other_value: u16, fill: u16) -> Self {
        let s = self.shape();
        let mut b = vec![vec![fill; s.x]; s.y];
        for x in 0..s.x {
            for y in 0..s.y {
                if other[y][x] == other_value {
                    b[y][x] = self[y][x];
                }
            }
        }
        b
    }

    fn fill(&mut self, c1: &Coord, c2: &Coord, when: u16, value: u16) {
        for x in c1.x..c2.x {
            for y in c1.y..c2.y {
                if self[y][x] == when {
                    self[y][x] = value;
                }
            }
        }
    }

    fn mark_rnd_position(&mut self, value: u16) -> Coord {
        let s = self.shape();
        let mut rng = rand::thread_rng();
        let source_x = rng.gen_range(0, s.x);
        let source_y = rng.gen_range(0, s.y);
        self[source_y][source_x] = value;
        Coord{x: source_x, y: source_y}
    }

    fn max_val(&self) -> u16 {
        let mut m: u16 = 0;
        for row in self {
            let row_max = row.iter().max();
            match row_max {
                Some(i) => m = m.max(*i),
                _ => (),
            }
        }
        m
    }

    fn max_when(&self, other: &Self, other_value: u16) -> u16 {
        let mut m: u16 = 0;
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if other[y][x] == other_value {
                    m = m.max(self[y][x]);
                }
            }
        }
        m
    }

    fn coords_of_lambda(&self, test: & dyn Fn(u16) -> bool) -> Vec<Coord> {
        let mut coords: Vec<Coord> = vec![];
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if test(self[y][x]) {
                    coords.push(Coord{x, y});
                }
            }
        }
        coords
    }

    fn coords_of(&self, value: u16) -> Vec<Coord> {
        self.coords_of_lambda(&(|val| val == value))
    }

    fn coords_not_of(&self, value: u16) -> Vec<Coord> {
        self.coords_of_lambda(&(|val| val != value))
    }

    fn coords_when(&self, value: u16, other: &Self, other_value: u16) -> Vec<Coord> {
        let mut coords: Vec<Coord> = vec![];
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if self[y][x] == value && other[y][x] == other_value {
                    coords.push(Coord{x, y});
                }
            }
        }
        coords
    }

    fn apply(&mut self, coords: &Vec<Coord>, value: u16) {
        for coord in coords.iter() {
            self[coord.y][coord.x] = value;
        }
    }

    fn apply_when(&mut self, value: u16, other: &Self, other_value: u16) {
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if other[y][x] == other_value {
                    self[y][x] = value; 
                }
            }
        }
        
    }

    fn neighbour_min(&self, coord: &Coord, edge: &Coord) -> u16 {
        let mut val = 0;
        if coord.x > 1 {
            if coord.y > 0 {
                val = min_non_zero(
                        min_non_zero(
                            min_non_zero(val, self[coord.y - 1][coord.x - 1]),
                            self[coord.y][coord.x - 1],
                        ),
                        self[coord.y - 1][coord.x],
                    );
            } else {
                val = min_non_zero(val, self[coord.y][coord.x - 1]);
            }
            if coord.y < edge.y {
                val = min_non_zero(val, self[coord.y + 1][coord.x - 1]);
            }
        } else if coord.y > 0 {
            val = min_non_zero(val, self[coord.y - 1][coord.x]);

        }
        if coord.x < edge.x {
            if coord.y < edge.y {
                val = min_non_zero(
                    min_non_zero(
                        min_non_zero(val, self[coord.y + 1][coord.x + 1]),
                        self[coord.y][coord.x + 1]),
                    self[coord.y + 1][coord.x],
                );

            } else {
                val = min_non_zero(val, self[coord.y][coord.x + 1]);
            }
            if coord.y > 0 {
                val = min_non_zero(val, self[coord.y - 1][coord.x + 1]);
            }
        } else if coord.y < edge.y {
            val = min_non_zero(val, self[coord.y + 1][coord.x]);
        }        
        val
    }

    fn neighbour_has_lambda(&self, coord: &Coord, out_of_bound_true: bool, test: &dyn Fn(u16, u16) -> bool) -> bool {
        let shape = self.shape();
        let own = self[coord.y][coord.x];        
        for direction in Direction::iterator() {
            if coord.is_legal_direction(direction) {
                let other = coord.translate_direction(direction);
                if other.is_inside(&shape) {
                    if test(own, self[other.y][other.x]) {
                        return true;
                    }
                } else if out_of_bound_true {
                    return true;
                }
            } else if out_of_bound_true {
                return true;
            }
        }
        false
    }
}