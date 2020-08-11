use crate::world::position::{Coord, Offset, Positional};
use crate::world::direction::Direction;
use crate::world::World;
use crate::world::board::Board;

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

    pub fn is_rook(&self) -> bool {
        match self {
            PieceType::Rook => true,
            _ => false
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

    pub fn unmoved(&self) -> bool {
        self.history.len() == 1
    }

    pub fn pawn_direction(&self) -> Vec<Direction> {
        let l = self.history.len();
        if l < 2 { return Direction::cardinals() }
        // If previous move was straight then you have a direction
        let mut cross: Vec<Direction> = Vec::new();
        for idx in 1..l {
            let off = self.history[idx] - self.history[idx - 1];
            let dir: Direction = off.as_direction().unwrap();
            if cross.len() == 0 {
                cross = dir.closest_cardinals();
            } else {
                cross = dir.common_cardinals(cross);
            }
            if cross.len() == 1 { break; }
        }
        cross
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
                // Castling is only 2 along cardinal so no legal more than 2
                if off.manhattan() > 2 { return false; }
                if off.chebyshev() == 1 {
                    // TODO: Check so coord is not in check
                    return true;
                }
                // Castling
                if self.unmoved() { return false };
                // TODO: Check so pos, intermediate and coord is not in check
                // Project on world to first piece in direction
                match world.pieces_map.find_first(&coord, off.as_direction().unwrap()) {
                    Some(other_id) => {
                        let other = world.pieces.get(&other_id).unwrap();
                        // First piece find must be ours and umoved too
                        if other.player != self.player || !other.unmoved() { return false; }
                        // Must not be too far
                        let other_off: Offset = *other.position().unwrap() - *pos;
                        if other_off.manhattan() >= MOVE_RANGE_LIMIT { return false; }
                        // Check other is rook
                        other.kind.is_rook()
                    },
                    None => false
                }
            },
            PieceType::Pawn => {
                if off.manhattan() > 2 { return false; }

                let off_dir = off.as_direction().unwrap();
                let directions = self.pawn_direction();

                // Not taking
                if off.chebyshev() == off.manhattan() {
                    // Move length
                    if off.manhattan() == 2 || self.history.len() > 1 { return false; }
                    // Check not changing cardinal directions
                    if !directions.iter().any(| d | off_dir.is(d)) {
                        return false;
                    }
                    if world.no_piece_between(pos, coord) {
                        // To piece allowed at target position
                        return world.pieces_map[coord.y][coord.x] == 0;
                    }
                    return false;
                }

                // Taking is exactly one diag step 
                if off.chebyshev() > 1 || off.chebyshev() == 0 { return false }
                // Must be something to take / we don't support en passant
                if world.pieces_map[coord.y][coord.x] == 0 { return false };
                // Must neighbour any existing direction
                return !directions.iter().any(| d | off_dir.rotation(d).abs() < 2)
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
