use crate::world::Direction; 

#[derive(Debug)]
pub struct Coord {pub x: usize, pub y: usize}
pub struct Offset {pub x: i16, pub y: i16}


pub trait Positional {
    fn translate(&self, offset: Offset) -> Self;
    fn translate_direction(&self, direction: Direction) -> Self;
    fn area(&self, other: &Self) -> i16;
    fn is_neighbour(&self, other: &Self) -> bool;
}

impl Positional for Coord {
    fn translate(&self, offset: Offset) -> Self {
        Coord{
            x: (self.x as i16 + offset.x) as usize,
            y: (self.y as i16 + offset.y) as usize,
        }
    }

    fn translate_direction(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.translate(Offset{x: 0, y: -1}),
            Direction::East => self.translate(Offset{x: 1, y: 0}),
            Direction::South => self.translate(Offset{x: 0, y: 1}),
            Direction::West => self.translate(Offset{x: -1, y: 0}),
        }
    }

    fn area(&self, other: &Coord) -> i16 {
        ((self.x as i16 - other.x as i16) * (self.y as i16 - other.y as i16)).abs()
    }

    fn is_neighbour(&self, other: &Self) -> bool {
        (self.x as i16 - other.x as i16).abs() <= 1 && (self.y as i16 - other.y as i16).abs() <= 1
    }
}
