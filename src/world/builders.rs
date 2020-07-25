use rand::Rng;
use std::convert::TryInto;

use crate::world::position::{Coord, Positional};
use crate::world::Direction;
use crate::world::board::Board;

fn get_zone_sizes(zones: u16, shape: &Coord, portion: f32) -> Vec<u16> {
    let mut area: u16 = (shape.x * shape.y).try_into().unwrap();
    let mut areas = vec![1 as u16; zones.into()];    
    for idx in 1..zones.into() {
        let a = ((area as f32) * portion).floor() as u16;
        areas[zones as usize - idx] = a;
        area -= a;
    }
    return areas 
}

pub fn add_zones_rects(board: &mut Vec<Vec<u16>>, zones: u16) {
    if zones < 2 {
        return;
    }
    let shape = board.shape();
    let areas = get_zone_sizes(zones, &shape, 0.75);
    let mut c1 = board.mark_rnd_position(1);
    let mut c2 = c1
        .translate_direction(Direction::East)
        .translate_direction(Direction::South);

    let mut rng = rand::thread_rng();
    for zone in 1..zones {
        while areas[zone as usize] > c1.area(&c2) as u16 {
            let grow = rng.gen_range(0, 4);
            match grow {            
                0 => if c1.x > 1 {c1 = c1.translate_direction(Direction::West)},
                1 => if c2.x + 1 < shape.x {c2 = c2.translate_direction(Direction::East)},
                2 => if c1.y > 1 {c1 = c1.translate_direction(Direction::North)},
                3 => if c2.y + 1 < shape.y {c2 = c2.translate_direction(Direction::South)},
                _ => (),
            };
            board.fill(&c1, &c2, 0, zone + 1);
        }
    }
    board.fill(&Coord{x: 0, y: 0}, &shape, 0, zones + 1);
}

pub fn add_fog(fog: &mut Vec<Vec<u16>>, zones: &Vec<Vec<u16>>) {
    let mut prev_zone: Vec<Coord> = zones.coords_of(1);
    fog.apply(&prev_zone, 1);
    let max_zone = zones.max_val();
    let edge = zones.shape()
        .translate_direction(Direction::NorthWest);

    for zone in 2..(max_zone + 1) {
        let this_zone = zones.coords_of(zone);

        // Set inner border distance as 1
        for coord in this_zone.iter() {
            if prev_zone.iter().any(|other| coord.is_neighbour(other)) {
                fog[coord.y][coord.x] = 1;
            }
        }
        prev_zone.extend(this_zone);

        let mut cur_value = 1;
        loop {        
            let this_zone = zones.coords_when(zone, &fog, 0);
            if this_zone.len() == 0 {
                break;
            }
            for coord in this_zone.iter() {
                let nmin = fog.neighbour_min(coord, &edge);
                if nmin == cur_value {
                    fog[coord.y][coord.x] = cur_value + 1;
                }
            }
            cur_value += 1;
        }
    }
}

