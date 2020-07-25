use crate::world::position::Coord;
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

pub struct World {
    pub zones: Vec<Vec<u16>>,
    pub fog_curve: Vec<Vec<u16>>,
    pub fog: Vec<Vec<u16>>,
    fog_value: u16,
    active_zone: u16,
}

impl World {
    fn new(shape: Coord) -> Self {
        let zones = vec![vec![0; shape.x]; shape.y];
        let fog_curve = zones.new_with(0);
        let fog = zones.new_with(0);
        World {
            zones,
            fog_curve,
            fog,
            fog_value: 0,
            active_zone: 0,
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
}

pub fn spawn(shape: Coord, nzones: u16) -> World {
    let mut world = World::new(shape);
    add_zones_rects(&mut world.zones, nzones);
    add_fog(&mut world.fog_curve, &world.zones);
    world.active_zone = world.zones.max_val() + 1;
    world
}