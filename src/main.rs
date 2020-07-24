use rand::Rng;
use std::convert::TryInto;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Coord {pub x: usize, pub y: usize}
pub struct Offset {pub x: i16, pub y: i16}

enum Direction {
    North,
    East,
    South,
    West,
}

trait Positional {
    fn translate(&self, offset: Offset) -> Self;
    fn translate_direction(&self, direction: Direction) -> Self;
    fn area(&self, other: &Self) -> i16;
    fn is_neighbour(&self, other: &Self) -> bool;
}

impl Positional for Coord {
    fn translate(&self, offset: Offset) -> Self {
        Coord{
            x: (self.x as i16 + offset.x) as usize,
            y: (self.y as i16 + offset.y) as usize,
        }
    }

    fn translate_direction(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.translate(Offset{x: 0, y: -1}),
            Direction::East => self.translate(Offset{x: 1, y: 0}),
            Direction::South => self.translate(Offset{x: 0, y: 1}),
            Direction::West => self.translate(Offset{x: -1, y: 0}),
        }
    }

    fn area(&self, other: &Coord) -> i16 {
        ((self.x as i16 - other.x as i16) * (self.y as i16 - other.y as i16)).abs()
    }

    fn is_neighbour(&self, other: &Self) -> bool {
        (self.x as i16 - other.x as i16).abs() <= 1 && (self.y as i16 - other.y as i16).abs() <= 1
    }
}

struct World {
    zones: Vec<Vec<u16>>,
    fog_curve: Vec<Vec<u16>>,
}

fn min_non_zero(a: u16, b: u16) -> u16 {
    if a == 0 { return  b;}
    if b == 0 { return  a;}
    a.min(b)
}

trait Board {
    fn shape(&self) -> Coord;
    fn new_with<T: Clone>(&self, value: T) -> Vec<Vec<T>>;
    fn fill(&mut self, c1: &Coord, c2: &Coord, when: u16, value: u16);
    fn mark_rnd_position(&mut self, value: u16) -> Coord;
    fn max_val(&self) -> u16;
    fn coords_of(&self, value: u16) -> Vec<Coord>;
    fn coords_not_of(&self, value: u16) -> Vec<Coord>;
    fn coords_of_filtered(&self, value: u16, other: &Self, other_value: u16) ->  Vec<Coord>;
    fn apply(&mut self, coords: &Vec<Coord>, value: u16);
    fn neighbour_min(&self, coord: &Coord, edge: &Coord) -> u16;
}

impl Board for Vec<Vec<u16>> {
    fn shape(&self) -> Coord {
        Coord {
            x: self[0].len(),
            y: self.len(),
        }
    }
    
    fn new_with<U: Clone>(&self, value: U) -> Vec<Vec<U>> {
       let s = self.shape();
       vec![vec![value; s.x]; s.y]
    }

    fn fill(&mut self, c1: &Coord, c2: &Coord, when: u16, value: u16) {
        for x in c1.x..c2.x {
            for y in c1.y..c2.y {
                if self[y][x] == when {
                    self[y][x] = value;
                }
            }
        }
    }

    fn mark_rnd_position(&mut self, value: u16) -> Coord {
        let s = self.shape();
        let mut rng = rand::thread_rng();
        let source_x = rng.gen_range(0, s.x);
        let source_y = rng.gen_range(0, s.y);
        self[source_y][source_x] = value;
        Coord{x: source_x, y: source_y}
    }

    fn max_val(&self) -> u16 {
        let mut m: u16 = 0;
        for row in self {
            let row_max = row.iter().max();
            match row_max {
                Some(i) => m = m.max(*i),
                _ => (),
            }
        }
        m
    }

    fn coords_of(&self, value: u16) -> Vec<Coord> {
        let mut coords: Vec<Coord> = vec![];
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if self[y][x] == value {
                    coords.push(Coord{x, y});
                }
            }
        }
        coords
    }

    fn coords_not_of(&self, value: u16) -> Vec<Coord> {
        let mut coords: Vec<Coord> = vec![];
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if self[y][x] != value {
                    coords.push(Coord{x, y});
                }
            }
        }
        coords
    }

    fn coords_of_filtered(&self, value: u16, other: &Self, other_value: u16) -> Vec<Coord> {
        let mut coords: Vec<Coord> = vec![];
        let shape = self.shape();
        for x in 0..shape.x {
            for y in 0..shape.y {
                if self[y][x] == value && other[y][x] == other_value {
                    coords.push(Coord{x, y});
                }
            }
        }
        coords
    }

    fn apply(&mut self, coords: &Vec<Coord>, value: u16) {
        for coord in coords.iter() {
            self[coord.y][coord.x] = value;
        }
    }

    fn neighbour_min(&self, coord: &Coord, edge: &Coord) -> u16 {
        let mut val = 0;
        if coord.x > 1 {
            if coord.y > 0 {
                val = min_non_zero(
                        min_non_zero(
                            min_non_zero(val, self[coord.y - 1][coord.x - 1]),
                            self[coord.y][coord.x - 1],
                        ),
                        self[coord.y - 1][coord.x],
                    );
            } else {
                val = min_non_zero(val, self[coord.y][coord.x - 1]);
            }
            if coord.y < edge.y {
                val = min_non_zero(val, self[coord.y + 1][coord.x - 1]);
            }
        } else if coord.y > 0 {
            val = min_non_zero(val, self[coord.y - 1][coord.x]);

        }
        if coord.x < edge.x {
            if coord.y < edge.y {
                val = min_non_zero(
                    min_non_zero(
                        min_non_zero(val, self[coord.y + 1][coord.x + 1]),
                        self[coord.y][coord.x + 1]),
                    self[coord.y + 1][coord.x],
                );

            } else {
                val = min_non_zero(val, self[coord.y][coord.x + 1]);
            }
            if coord.y > 0 {
                val = min_non_zero(val, self[coord.y - 1][coord.x + 1]);
            }
        } else if coord.y < edge.y {
            val = min_non_zero(val, self[coord.y + 1][coord.x]);
        }        
        val
    }
}

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

fn add_zones_rects(board: &mut Vec<Vec<u16>>, zones: u16) {
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

fn add_fog(fog: &mut Vec<Vec<u16>>, zones: &Vec<Vec<u16>>) {
    let mut prev_zone: Vec<Coord> = zones.coords_of(1);
    fog.apply(&prev_zone, 1);
    let max_zone = zones.max_val();
    let edge = zones.shape()
        .translate_direction(Direction::West)
        .translate_direction(Direction::North);

    for zone in 2..(max_zone + 1) {
        let this_zone = zones.coords_of(zone);

        // Set inner border distance as 1
        for coord in this_zone.iter() {
            if prev_zone.iter().any(|other| coord.is_neighbour(other)) {
                fog[coord.y][coord.x] = 1;
            }
        }
        prev_zone.extend(this_zone);

        /*
        // Set outer border distance as 2
        let this_zone = zones.coords_of_filtered(zone, &fog, 0);
        let not_this_zone = zones.coords_not_of(zone);
        for coord in this_zone.iter() {
            if not_this_zone.iter().any(|other| coord.is_neighbour(other)) {
                fog[coord.y][coord.x] = 2;
            }
        }

        // Set world edge as 2 too
        let this_zone = zones.coords_of_filtered(zone, &fog, 0);
        for coord in this_zone.iter() {
            if coord.x == 0 || coord.y == 0 || coord.x == edge.x || coord.y == edge.y {
                fog[coord.y][coord.x] = 2;
            }
        }
        */

        let mut cur_value = 1;
        loop {        
            let this_zone = zones.coords_of_filtered(zone, &fog, 0);
            if this_zone.len() == 0 {
                break;
            }
            if cur_value == 9 {
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

fn main() {
    let shape = Coord{ x: 4 * 16, y: 16};
    let zones = vec![vec![0; shape.x]; shape.y];
    let fog_curve = zones.new_with(0);
    let mut world = World {
        zones,
        fog_curve,
    };
    add_zones_rects(&mut world.zones, 4);
    add_fog(&mut world.fog_curve, &world.zones);
    for (zone_row, fog_row) in world.zones.iter().zip(world.fog_curve.iter()) {
        let zone_out = zone_row.into_iter().map(|i| i.to_string()).collect::<String>();
        let fog_out = fog_row.into_iter().map(|i| i.to_string()).collect::<String>();
        println!("{} {}", zone_out, fog_out);
    }
    
}
