use crate::world::position::{Coord, Offset, Positional};

#[derive(Debug, Copy, Clone)]
pub enum PieceType {
    Empty,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

impl PieceType {
    pub fn intermediat_steps(&self, from: Coord, to: Coord) -> Option<Vec<Coord>> {
        const LIMIT: i16 = 9;
        let off: Offset = to - from;
        match self {
            PieceType::Empty => None,
            PieceType::King => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            PieceType::Pawn => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            PieceType::Knight => if off.chebyshev() == 2 && off.skew() == 1 { from.steps(&to) } else { None },
            PieceType::Bishop => if off.chebyshev() < LIMIT && off.skew() == 0 { from.steps(&to) } else { None },
            PieceType::Rook => {
                let c = off.chebyshev();
                if c < LIMIT && off.manhattan() == c { from.steps(&to) } else { None }
            },
            PieceType::Queen => {
                let c = off.chebyshev();
                if c < LIMIT && (off.skew() == 0 || off.manhattan() == c) {
                    from.steps(&to)
                } else { None }
            },
        }
    }
}

pub struct Piece {
    pub kind: PieceType,
    pub player: u16,
    history: Vec<Coord>
}

impl Piece {
    pub fn new(kind: PieceType, player: u16) -> Self {
        Piece{kind, player, history: Vec::new()}
    }

    pub fn place(&mut self, coord: &Coord) {
        self.history.push(coord.clone());
    }

    pub fn position(&self) -> Option<&Coord> {
        self.history.last()
    }
}
