use crate::world::position::{Coord, Offset, Positional};

#[derive(Debug, Copy)]
pub enum Pieces {
    Empty,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

impl Clone for Pieces {
    fn clone(&self) -> Self {
        *self
    }
}

impl Pieces {
    pub fn steps(&self, from: Coord, to: Coord) -> Option<Vec<Coord>> {
        const LIMIT: i16 = 9;
        let off: Offset = to - from;
        match self {
            Pieces::Empty => None,
            Pieces::King => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            Pieces::Pawn => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            Pieces::Knight => if off.chebyshev() == 2 && off.skew() == 1 { from.steps(&to) } else { None },
            Pieces::Bishop => if off.chebyshev() < LIMIT && off.skew() == 0 { from.steps(&to) } else { None },
            Pieces::Rook => {
                let c = off.chebyshev();
                if c < LIMIT && off.manhattan() == c { from.steps(&to) } else { None }
            },
            Pieces::Queen => {
                let c = off.chebyshev();
                if c < LIMIT && (off.skew() == 0 || off.manhattan() == c) {
                    from.steps(&to)
                } else { None }
            },
        }
    }
}
