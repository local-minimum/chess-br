use std::ops::Sub;
use std::cmp::max;

use crate::world::direction::Direction;

#[derive(Debug, Copy, Clone)]
pub struct Coord {pub x: usize, pub y: usize}
#[derive(Debug, Copy, Clone)]
pub struct Offset {pub x: i16, pub y: i16}

pub trait Positional {
    fn translate(&self, offset: Offset) -> Self;
    fn translate_direction(&self, direction: Direction) -> Self;
    fn area(&self, other: &Self) -> i16;
    fn is_neighbour(&self, other: &Self) -> bool;
    fn is_legal_direction(&self, direction: Direction) -> bool;
    fn is_inside(&self, other: &Self) -> bool;
    fn steps(&self, other: &Self) -> Option<Vec<Coord>>;
}

impl Sub for Coord {
    type Output = Offset;
    fn sub(self, rhs: Coord) -> Self::Output {
        Offset{
            x: (self.x as i16) - (rhs.x as i16),
            y: (self.y as i16) - (rhs.y as i16),
        }
    }
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
            Direction::NorthEast => self.translate(Offset{x: 1, y: -1}),
            Direction::East => self.translate(Offset{x: 1, y: 0}),
            Direction::SouthEast=> self.translate(Offset{x: 1, y: 1}),
            Direction::South => self.translate(Offset{x: 0, y: 1}),
            Direction::SouthWest => self.translate(Offset{x: -1, y: 1}),
            Direction::West => self.translate(Offset{x: -1, y: 0}),
            Direction::NorthWest => self.translate(Offset{x: -1, y: -1}),
        }
    }

    fn is_legal_direction(&self, direction: Direction) -> bool {
        match direction {
            Direction::North => self.y > 0,
            Direction::NorthEast => self.y > 0,
            Direction::SouthWest => self.x > 0,
            Direction::West => self.x > 0,
            Direction::NorthWest => self.x > 0 && self.y > 0,
            _ => true,
        }

    }

    fn area(&self, other: &Coord) -> i16 {
        ((self.x as i16 - other.x as i16) * (self.y as i16 - other.y as i16)).abs()
    }

    fn is_neighbour(&self, other: &Self) -> bool {
        (self.x as i16 - other.x as i16).abs() <= 1 && (self.y as i16 - other.y as i16).abs() <= 1
    }

    fn is_inside(&self, other: &Self) -> bool {
        self.x < other.x && self.y < other.y
    }

    fn steps(&self, other: &Self) -> Option<Vec<Coord>> {
        let off: Offset = *self - *other;
        let s = off.skew();
        if s > 1 {
            None
        } else {
            let dir = off.direction();
            let mut cur = self.clone();
            let mut v = Vec::new();
            for _ in 0..(off.chebyshev() - 1) {
                cur = cur.translate(dir);
                v.push(cur.clone());
            }
            Some(v)
        }
    }
}

impl Offset {
    pub fn abs(&self) -> Self {
        Offset{x: self.x.abs(), y: self.y.abs()}
    }

    pub fn sq_len(&self) -> i16 {
        self.x * self.x + self.y * self.y
    }

    pub fn chebyshev(&self) -> i16 {
        max(self.x.abs(), self.y.abs())
    }

    pub fn manhattan(&self) -> i16 {
        self.x.abs() + self.y.abs()
    }

    pub fn skew(&self) -> i16 {
        let a = self.abs();
        return (a.x - a.y).abs()
    }

    pub fn direction(&self) -> Self {
        Offset{
            x: if self.x < 0 { -1 } else if self.x > 0 { 1 } else { 0 },
            y: if self.y < 0 { -1 } else if self.y > 0 { 1 } else { 0 },
        }
    }

    pub fn as_direction(&self) -> Option<Direction> {
        match self.direction() {
            Offset{x: 0, y: 1} => Some(Direction::South),
            Offset{x: 1, y: 1} => Some(Direction::SouthEast),
            Offset{x: -1, y: 1} => Some(Direction::SouthWest),
            Offset{x: 0, y: -1} => Some(Direction::North),
            Offset{x: 1, y: -1} => Some(Direction::NorthEast),
            Offset{x: -1, y: -1} => Some(Direction::NorthWest),
            Offset{x: 1, y: 0} => Some(Direction::East),
            Offset{x: -1, y: 0} => Some(Direction::West),
            _ => None
        }
    }
}
