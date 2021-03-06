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

    pub fn my_king_is_checked(&self, world: &World) -> bool {
        let king_id = world.players.get(&self.player).unwrap().king_id;
        let king: &Piece = world.pieces.get(&king_id).unwrap();
        king
            .others_threatening_me_at(world, king.position().unwrap())
            .len() > 0
    }

    pub fn can_move_to(&self, world: &World, coord: &Coord) -> bool {
        let pos = self.position().unwrap();
        let off: Offset = *coord - *pos;
        let checked = self.my_king_is_checked(world);
        
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
                if self.threatening(world).iter().any(| c: &Coord | c.x == coord.x && c.y == coord.y)
                    && self.others_threatening_me_at(world, coord).len() == 0
                {
                    return true;
                }

                // Else must be Castling
                if !self.unmoved() || checked { return false };
                // Castling always 2 steps towards rook
                if !(off.chebyshev() == 2) {
                    return false;
                }
                // May not castle through check
                let dir = off.as_direction().unwrap();
                let mut current = pos.clone();
                for _ in 0..2 {
                    current = current.translate_direction(dir);
                    if self.others_threatening_me_at(world, &current).len() > 0 {
                        return false;
                    }
                }

                // Project on world to first piece in direction
                match world.pieces_map.find_first(&coord, dir) {
                    Some(other_id) => {
                        let other = world.pieces.get(&other_id).unwrap();
                        // First piece find must be ours and umoved too
                        if other.player != self.player || !other.unmoved() { return false; }
                        // Must not be too far
                        let other_off: Offset = *other.position().unwrap() - *pos;
                        let dist = other_off.chebyshev();
                        if dist >= MOVE_RANGE_LIMIT { return false; }
                        // Other must be at least 3 steps away since we're moving 2
                        if dist < 3 { return false; }
                        // Check other is rook
                        other.kind.is_rook()
                    },
                    None => false
                }
            },
            PieceType::Pawn => {
                if checked || off.manhattan() > 2 { return false; }

                let off_dir = off.as_direction().unwrap();
                let directions = self.pawn_direction();

                // Not taking
                if off.chebyshev() == off.manhattan() {
                    // Move length, only 2 allowed first move
                    if off.manhattan() == 2 && !self.unmoved() { return false; }
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
                // Taking
                return self.threatening(world)
                    .iter()
                    .any(|c: &Coord | c.x == coord.x && c.y == coord.y);
            },
            PieceType::Knight => !checked && off.chebyshev() == 2 && off.skew() == 1,
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

    fn others_threatening_me_at(&self, world: &World, pos: &Coord) -> Vec<u16> {
        let shape = world.fog.shape();
        let lower_left = pos
            .translate_n_direction(Direction::NorthWest, MOVE_RANGE_LIMIT - 1)
            .clamp(&shape);
        let upper_right = pos
            .translate_n_direction(Direction::SouthEast, MOVE_RANGE_LIMIT - 1)
            .clamp(&shape);
        let mut ids = Vec::new();
        for idx in world.pieces_map.find_all(&lower_left, &upper_right) {
            let piece = world.pieces.get(&idx).unwrap();
            if piece.player == self.player { continue; }
            if piece.threatening(world).iter().any(|p| p.x == pos.x && p.y == pos.y) {
                ids.push(idx);
            }
        }
        ids
    }

    fn threatening(&self, world: &World) -> Vec<Coord> {
        let mut coords = Vec::new();
        let pos = self.position().unwrap();

        // Linear threats pawns are too special
        let directions = match self.kind {
            PieceType::Bishop => Direction::non_cardinals(),
            PieceType::Rook => Direction::cardinals(),
            PieceType::Queen | PieceType::King => Direction::iterator().collect(),
            PieceType::Pawn | PieceType::Knight | PieceType::Empty => Vec::new(),
        };
        let reach = match self.kind {
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => MOVE_RANGE_LIMIT - 1,
            PieceType::Knight => 1,
            _ => 0,
        };

        for dir in directions {
            let mut current: Coord = pos.clone();
            for _ in 0..reach{
                current = current.translate_direction(dir);
                let pid = world.pieces_map[current.y][current.x];
                if pid > 0 {
                    if world.pieces.get(&pid).unwrap().player != self.player {
                        coords.push(current);
                    }
                    break;
                }
                coords.push(current);
            }
        }

        // Special threats
        let shape = world.fog.shape();
        match self.kind {
            PieceType::Knight => {
                for coord in pos.knight_offsets() {
                    if !coord.is_inside(&shape) { continue; }
                    let pid = world.pieces_map[coord.y][coord.x];
                    if pid != 0 && world.pieces.get(&pid).unwrap().player == self.player {
                        continue;
                    }
                    coords.push(coord);
                }
            },
            PieceType::Pawn => {
                for cardinal in self.pawn_direction() {
                    for dir in cardinal.neighbours() {
                        let coord = pos.translate_direction(dir);
                        if coord.is_inside(&shape) {
                            let pid = world.pieces_map[coord.y][coord.x];
                            if pid != 0 && world.pieces.get(&pid).unwrap().player != self.player {
                                coords.push(coord);
                            }
                        }
                    }
                }

            },
            _ => (),
        }

        return coords
    }
}
