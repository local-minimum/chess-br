use crate::world::position::{Coord, Offset, Positional};
use crate::world::direction::Direction;
use crate::world::World;

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

const MOVE_RANGE_LIMIT: i16 = 9;

impl PieceType {
    pub fn intermediat_steps(&self, from: Coord, to: Coord) -> Option<Vec<Coord>> {
        let off: Offset = to - from;
        match self {
            PieceType::Empty => None,
            PieceType::King => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            PieceType::Pawn => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            PieceType::Knight => if off.chebyshev() == 2 && off.skew() == 1 { from.steps(&to) } else { None },
            PieceType::Bishop => if off.chebyshev() < MOVE_RANGE_LIMIT && off.skew() == 0 { from.steps(&to) } else { None },
            PieceType::Rook => {
                let c = off.chebyshev();
                if c < MOVE_RANGE_LIMIT && off.manhattan() == c { from.steps(&to) } else { None }
            },
            PieceType::Queen => {
                let c = off.chebyshev();
                if c < MOVE_RANGE_LIMIT && (off.skew() == 0 || off.manhattan() == c) {
                    from.steps(&to)
                } else { None }
            },
        }
    }

    pub fn value(&self) -> u16 {
        match self {
            PieceType::Bishop => 3,
            PieceType::Empty => 0,
            PieceType::King => 20,
            PieceType::Knight => 3,
            PieceType::Pawn => 1,
            PieceType::Queen => 9,
            PieceType::Rook => 5,
        }
    }
}

pub struct Piece {
    pub kind: PieceType,
    pub player: u16,
    history: Vec<Coord>,
    pub alive: bool,
}

impl Piece {
    pub fn new(kind: PieceType, player: u16) -> Self {
        Piece{kind, player, history: Vec::new(), alive: true}
    }

    pub fn place(&mut self, coord: Coord) {
        if self.alive {
            self.history.push(coord);
        }
    }

    pub fn position(&self) -> Option<&Coord> {
        if self.alive {self.history.last()} else { None }
    }

    pub fn pawn_direction(&self) -> Option<Vec<Direction>> {
        let l = self.history.len();
        if l < 2 { return None }
        // If previous move was straight then you have a direction
        let mut cross: Vec<Direction> = Vec::new();
        for idx in 1..l {
            let off = self.history[idx] - self.history[idx - 1];
            let dir: Direction = off.as_direction().unwrap();
            if cross.len() == 0 {
                cross = dir.closest_cross();
            } else {
                cross = dir.common_cross(cross);
            }
            if cross.len() == 1 { break; }
        }
        Some(cross)
    }

    pub fn can_move_to(&self, world: &World, coord: &Coord) -> bool {
        let pos = self.position().unwrap();
        let off: Offset = *coord - *pos;
        // May not self take
        match world.pieces.get(&world.pieces_map[coord.y][coord.x]) {
            Some(target_piece) => {
                if target_piece.player == self.player {
                    return false
                }
            },
            _ => (),
        }
        match self.kind {
            PieceType::Empty => false,
            PieceType::King => {
                if off.chebyshev() > 1 {
                    // Castling
                    return false;
                }
                // Check if coord is checked
                true
            },
            PieceType::Pawn => {
                if self.history.len() == 1 {
                    // First move rules
                    // Move one or two steps
                    if off.chebyshev() == off.manhattan() && off.chebyshev() <= 2 {
                        return world.no_piece_between(pos, coord)
                    }
                    return false;
                } else {
                    if off.chebyshev() > 1 || off.chebyshev() == 0 { return false }
                    // All possible cross directions based on previous moves
                    let directions = self.pawn_direction().unwrap();
                    let off_dir = off.as_direction().unwrap();
                    if !directions.iter().any(| d | off_dir.rotation(d).abs() < 2) { return false }
                    // May not take straight
                    if off.manhattan() == 1 { return world.pieces_map[coord.y][coord.x] == 0;}
                    // Must take on diagonal (we already know it's not our piece)
                    return world.pieces_map[coord.y][coord.x] != 0;
                }
            },
            PieceType::Knight => off.chebyshev() == 2 && off.skew() == 1,
            PieceType::Bishop => {
                if off.chebyshev() < MOVE_RANGE_LIMIT && off.skew() == 0 {
                    return world.no_piece_between(pos, coord)
                }
                false
            },
            PieceType::Rook => {
                let c = off.chebyshev();
                if c < MOVE_RANGE_LIMIT && off.manhattan() == c {
                    return world.no_piece_between(pos, coord)
                }
                false
            },
            PieceType::Queen => {
                let c = off.chebyshev();
                if c < MOVE_RANGE_LIMIT && (off.skew() == 0 || off.manhattan() == c) {
                    return world.no_piece_between(pos, coord)
                }
                false
            },

        }
    }
}
