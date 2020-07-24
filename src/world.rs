use crate::world::position::{Coord, Positional};
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

pub enum Direction {
    North,
    East,
    South,
    West,
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
    
    pub fn next_zone(&self) -> Vec<Vec<u16>> {
        let edge = self.zones.shape()
            .translate_direction(Direction::West)
            .translate_direction(Direction::North);
        if self.active_zone < 2 {
            return self.zones.new_with(0 as u16);
        }
        let coords = self.zones.coords_of(self.active_zone - 1);
        let mut edges: Vec<Coord> = vec![];
        for coord in coords.iter() {
            //TODO: Should be neighbour_max
            let val = self.zones.neighbour_min(coord, &edge);
            if val == self.active_zone {
                edges.push(Coord{x: coord.x, y: coord.y});
            }
        }
        let mut ret = self.zones.new_with(0 as u16);
        ret.apply(&edges, 1);
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