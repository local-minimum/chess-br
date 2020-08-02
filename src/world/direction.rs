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

    pub fn rnd_next(&self) -> Direction {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 3) {
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
}
