use crate::world::board::Board;
use crate::world::position::Coord;

#[derive(Debug)]
pub enum FogState {
    Resting,
    Contracting,
    Zone,
    Done,
}

pub struct Fog {
    pub zones: Vec<Vec<u16>>,
    pub fog_curve: Vec<Vec<u16>>,
    pub fog: Vec<Vec<u16>>,
    zone_rest: usize,
    fog_value: u16,
    active_zone: u16,
}

impl Fog {
    pub fn shape(&self) -> Coord {
        self.zones.shape()
    }

    pub fn new(shape: Coord) -> Self {
        let zones = vec![vec![0 as u16; shape.x]; shape.y];
        let fog_curve = zones.clone();
        let fog = zones.clone();
        Fog {
            zones,
            fog_curve,
            fog,
            zone_rest: 0,
            active_zone: 0,
            fog_value: 0,
        }
    }

    pub fn contract(&mut self, next_rest: usize) -> FogState {
        if self.zone_rest > 0 {
            self.zone_rest -= 1;
            return FogState::Resting;
        }
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
                self.zone_rest = next_rest;
                return FogState::Zone;
            }
        }
        let this_fog_curve = self.fog_curve
            .new_when(&self.zones, self.active_zone, 0);

        self.fog.apply_when(1, &this_fog_curve, self.fog_value);
        FogState::Contracting
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

    pub fn status(&self) -> String {
        format!("Zone {} / step {}", self.active_zone, self.fog_value)
    }

    pub fn init(
        &mut self,
        nzones: u16,
        init_zones: fn(&mut Vec<Vec<u16>>, u16),
        init_fog: fn(&mut Vec<Vec<u16>>, &Vec<Vec<u16>>),
    ) {
        init_zones(&mut self.zones, nzones);
        init_fog(&mut self.fog_curve, &self.zones);
        self.active_zone = self.zones.max_val() + 1;
    }
}
