use rand::Rng;
use std::convert::TryInto;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Point {pub x: usize, pub y: usize}

struct World {
    zones: Vec<Vec<u16>>,
    fog_curve: Vec<Vec<u16>>,
}

trait Board {
    fn shape(&self) -> Point;
    fn new_with<T: Clone>(&self, value: T) -> Vec<Vec<T>>;
    fn fill(&mut self, x0: usize, x1: usize, y0: usize, y1: usize, when: u16, value: u16);
    fn mark_rnd_position(&mut self, value: u16) -> Point; 
}

impl Board for Vec<Vec<u16>> {
    fn shape(&self) -> Point {
        Point {
            x: self[0].len(),
            y: self.len(),
        }
    }
    
    fn new_with<U: Clone>(&self, value: U) -> Vec<Vec<U>> {
       let s = self.shape();
       vec![vec![value; s.x]; s.y]
    }

    fn fill(&mut self, x0: usize, x1: usize, y0: usize, y1: usize, when: u16, value: u16) {
        for x in x0..x1 {
            for y in y0..y1 {
                if self[y][x] == when {
                    self[y][x] = value;
                }
            }
        }
    }

    fn mark_rnd_position(&mut self, value: u16) -> Point {
        let s = self.shape();
        let mut rng = rand::thread_rng();
        let source_x = rng.gen_range(0, s.x);
        let source_y = rng.gen_range(0, s.y);
        self[source_y][source_x] = value;
        Point{x: source_x, y: source_y}
    }
}

fn get_zone_sizes(zones: u16, size_x: usize, size_y: usize, portion: f32) -> Vec<u16> {
    let mut area: u16 = (size_x * size_y).try_into().unwrap();
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
    let areas = get_zone_sizes(zones, shape.x, shape.y, 0.75);
    let source = board.mark_rnd_position(1);
    let mut x0 = source.x;
    let mut x1 = source.x + 1;
    let mut y0 = source.y;
    let mut y1: usize = source.y + 1;

    let mut rng = rand::thread_rng();
    for zone in 1..zones {
        while areas[zone as usize] > (((x1 - x0) * (y1 - y0)) as u16) {
            let grow = rng.gen_range(0, 4);
            match grow {            
                0 => if x0 > 1 {x0 -= 1},
                1 => x1 = (x1 + 1).min(shape.x),
                2 => if y0 > 1 {y0 -= 1},
                3 => y1 = (y1 + 1).min(shape.y),
                _ => println!("{}", grow),
            };
            board.fill(x0, x1, y0, y1, 0, zone + 1);
        }
    }
    board.fill(0, shape.x, 0, shape.y, 0, zones + 1);
}

fn main() {
    let shape = Point{ x: 4 * 16, y: 16};
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
