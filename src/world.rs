use crate::world::position::{Coord, Offset, Positional};
use crate::world::builders::{add_zones_rects, add_fog};
use crate::world::board::Board;

pub mod board;
pub mod builders;
pub mod display;
pub mod position;

#[derive(Debug)]
pub enum FogState {
    Contracting,
    Zone,
    Done,
}

#[derive(Debug, Copy)]
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

impl Clone for Direction {
    fn clone(&self) -> Direction {
        *self
    }
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
}

#[derive(Debug)]
pub enum Action {
    None,
    Drop,
    Fly(Offset),
    Move(Coord, Coord),
}

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
        let off: Offset = to - from;
        match self {
            Pieces::Empty => None,
            Pieces::King => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            Pieces::Pawn => if off.chebyshev() == 1 { from.steps(&to) } else { None },
            Pieces::Knight => if off.chebyshev() == 2 && off.skew() == 1 { from.steps(&to) } else { None },
            Pieces::Bishop => if off.chebyshev() < 9 && off.skew() == 0 { from.steps(&to) } else { None },
            Pieces::Rook => {
                let c = off.chebyshev();
                if c < 9 && off.manhattan() == c { from.steps(&to) } else { None }
            },
            Pieces::Queen => {
                let c = off.chebyshev();
                if c < 9 && (off.skew() == 0 || off.manhattan() == c) {
                    from.steps(&to)
                } else { None }
            },
        }
    }
}

pub struct World {
    pub zones: Vec<Vec<u16>>,
    pub fog_curve: Vec<Vec<u16>>,
    pub fog: Vec<Vec<u16>>,
    pub pieces_types: Vec<Vec<Pieces>>,
    pub pieces_player: Vec<Vec<u16>>,
    fog_value: u16,
    active_zone: u16,
}

impl World {
    fn new(shape: Coord) -> Self {
        let zones = vec![vec![0; shape.x]; shape.y];
        let fog_curve = zones.new_with(0);
        let fog = zones.new_with(0);
        let player = zones.new_with(0);
        let pieces = zones.new_with(Pieces::Empty);
        World {
            zones,
            fog_curve,
            fog,
            fog_value: 0,
            active_zone: 0,
            pieces_types: pieces,
            pieces_player: player,
        }
    }

    pub fn contract_fog(&mut self) -> FogState {
        if self.fog_value == 0 {
            if self.active_zone == 0 {
                return FogState::Done;
            }
            self.active_zone -= 1;
            self.fog_value = self.fog_curve.max_when(&self.zones, self.active_zone);
        } else {
            self.fog_value -= 1;
            if self.fog_value == 0 {
                if self.active_zone == 1 {
                    return FogState::Done;
                }
                return FogState::Zone;
            }
        }
        let this_fog_curve = self.fog_curve
            .new_when(&self.zones, self.active_zone, 0);

        self.fog.apply_when(1, &this_fog_curve, self.fog_value);
        FogState::Contracting
    }

    pub fn status(&self) -> String {
        format!("Zone {} / step {}", self.active_zone, self.fog_value)
    }

    pub fn next_zone(&self, edge_only: bool) -> Vec<Vec<u16>> {
        let mut ret = self.zones.new_with(0 as u16);
        if self.active_zone < 2 {
            return ret;
        }

        let mut inner: Vec<Coord> = vec![];
        let coords = self.zones.coords_of_lambda(&(|val| val < self.active_zone));
        ret.apply(&coords, 1);
        if edge_only {
            for coord in coords.iter() {
                if !ret.neighbour_has_lambda(coord, true, &(|own, neigh| own == 1 && neigh == 0)) {
                    inner.push(Coord{x: coord.x, y: coord.y});
                }
            }
            ret.apply(&inner, 0);
        }

        ret
    }


    pub fn requst_action(&self, _action: Action, _user: u16) {
        // push to some list
    }

    fn has_piece_on_path(&self, steps: Vec<Coord>) -> bool {
        for step in steps {
            match self.pieces_types[step.y][step.x] {
                Pieces::Empty => { return true; },
                _ => (),
            }
        }
        false
    }

    pub fn do_move(&mut self, action: Action, user: u16) {
        match action {
            Action::None => (),
            Action::Drop => (),
            Action::Fly(_o) => (),
            Action::Move(from, to) => {
                if self.pieces_player[from.y][from.x] != user { return; }
                let piece = self.pieces_types[from.y][from.x];
                match piece.steps(from, to) {
                    None => (),
                    Some(steps) => {
                        if self.has_piece_on_path(steps) { return; }

                        // Moving
                        self.pieces_types[to.y][to.x] = self.pieces_types[from.y][from.x];
                        self.pieces_player[to.y][to.x] = self.pieces_player[from.y][from.x];
                        self.pieces_types[from.y][from.x] = Pieces::Empty;
                        self.pieces_player[from.y][from.x] = 0;
                    }
                }
            },
        }
    }
}

pub fn spawn(shape: Coord, nzones: u16) -> World {
    let mut world = World::new(shape);
    add_zones_rects(&mut world.zones, nzones);
    add_fog(&mut world.fog_curve, &world.zones);
    world.active_zone = world.zones.max_val() + 1;
    world
}
