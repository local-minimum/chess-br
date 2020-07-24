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
}

struct World {
    zones: Vec<Vec<u16>>,
    fog_curve: Vec<Vec<u16>>,
}

trait Board {
    fn shape(&self) -> Coord;
    fn new_with<T: Clone>(&self, value: T) -> Vec<Vec<T>>;
    fn fill(&mut self, c1: &Coord, c2: &Coord, when: u16, value: u16);
    fn mark_rnd_position(&mut self, value: u16) -> Coord; 
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

fn main() {
    let shape = Coord{ x: 4 * 16, y: 16};
    let zones = vec![vec![0; shape.x]; shape.y];
    let fog_curve = zones.new_with(0);
    let mut world = World {
        zones,
        fog_curve,
    };
    
    add_zones_rects(&mut world.zones, 4);
    for (zone_row, fog_row) in world.zones.iter().zip(world.fog_curve.iter()) {
        let zone_out = zone_row.into_iter().map(|i| i.to_string()).collect::<String>();
        let fog_out = fog_row.into_iter().map(|i| i.to_string()).collect::<String>();
        println!("{} {}", zone_out, fog_out);
    }
    
}
