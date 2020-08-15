use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn iterator() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
            Direction::West,
            Direction::NorthWest,
        ].iter().copied()
    }

    pub fn cardinals() -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::East,
            Direction::West,
            Direction::South,
        ]
    }

    pub fn non_cardinals() -> Vec<Direction> {
        vec![
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::SouthWest,
            Direction::NorthWest,
        ]
    }

    pub fn rnd() -> Direction {
        let mut rng = rand::thread_rng();
        [
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
            Direction::West,
            Direction::NorthWest,
        ][rng.gen_range(0, 8)]

    }

    pub fn rnd_next(&self, ratio: u16) -> Direction {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, ratio + 2) {
            0 => {
                match self {
                    Direction::North => Direction::NorthWest,
                    Direction::NorthWest => Direction::West,
                    Direction::West => Direction::SouthWest,
                    Direction::SouthWest => Direction::South,
                    Direction::South => Direction::SouthEast,
                    Direction::SouthEast => Direction::East,
                    Direction::East => Direction::NorthEast,
                    Direction::NorthEast => Direction::North,
                }
            },
            1 => {
                match self {
                    Direction::North => Direction::NorthEast,
                    Direction::NorthWest => Direction::North,
                    Direction::West => Direction::NorthWest,
                    Direction::SouthWest => Direction::West,
                    Direction::South => Direction::SouthWest,
                    Direction::SouthEast => Direction::South,
                    Direction::East => Direction::SouthEast,
                    Direction::NorthEast => Direction::East,
                }
            },
            _ => self.clone(),
        }
    }

    fn as_rank(&self) -> i16 {
        let my_idxs: Vec<i16> = Direction::iterator()
            .enumerate()
            .filter_map(| (idx, d) | if d.is(self) { Some(idx as i16) } else { None })
            .collect();
        *my_idxs.first().unwrap()
    }

    pub fn neighbours(&self) -> Vec<Direction> {
        let rank = self.as_rank();
        let dirs: Vec<Direction> = Direction::iterator()
            .enumerate()
            .filter_map(|(idx, d)| if i16::abs(rank - (idx as i16)) % 8 == 1 { Some(d)} else { None  })
            .collect();
        dirs
    }

    pub fn rotation(&self, other: &Direction) -> i16 {
        self.as_rank() - other.as_rank()
    }

    pub fn is(&self, other: &Direction) -> bool {
        match self {
            Direction::North => {
                match other {
                    Direction::North => true,
                    _ => false,
                }
            }
            Direction::NorthEast => {
                match other {
                    Direction::NorthEast => true,
                    _ => false,
                }
            }
            Direction::NorthWest => {
                match other {
                    Direction::NorthWest => true,
                    _ => false,
                }
            }
            Direction::South => {
                match other {
                    Direction::South => true,
                    _ => false,
                }
            }
            Direction::SouthWest => {
                match other {
                    Direction::SouthWest => true,
                    _ => false,
                }
            }
            Direction::SouthEast => {
                match other {
                    Direction::SouthEast => true,
                    _ => false,
                }
            }
            Direction::West => {
                match other {
                    Direction::West => true,
                    _ => false,
                }
            }
            Direction::East => {
                match other {
                    Direction::East => true,
                    _ => false,
                }
            }
        }
    }

    pub fn closest_cardinals(&self) -> Vec<Direction> {
        match self {
            Direction::North => vec![Direction::North],
            Direction::NorthWest => vec![Direction::North, Direction::West],
            Direction::NorthEast => vec![Direction::North, Direction::East],
            Direction::West => vec![Direction::West],
            Direction::East => vec![Direction::East],
            Direction::South => vec![Direction::South],
            Direction::SouthWest => vec![Direction::South, Direction::West],
            Direction::SouthEast => vec![Direction::South, Direction::East],
        }
    }

    pub fn common_cardinals(&self, prev: Vec<Direction>) -> Vec<Direction> {
        let mut cross = self.closest_cardinals();
        cross.retain(| d | prev.iter().any(|o | d.is(o)));
        cross
    }
}
