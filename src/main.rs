use rand::Rng;
use std::convert::TryInto;
use std::fmt::Debug;

#[derive(Debug)]
struct Shape {pub size_x: usize, pub size_y: usize}

struct Board<T>(Vec<Vec<T>>);

trait World<T> {
    fn new(size_x: usize, size_y: usize, value: T) -> Self;
    fn shape(&self) -> Shape;
}

impl World<u16> for Board<u16> {
    fn new(size_x: usize, size_y: usize, value: u16) -> Board<u16> {
        Board{0: vec![vec![value; size_x]; size_y]}
    }

    fn shape(&self) -> Shape {
        Shape {
            size_x: self.0[0].len(),
            size_y: self.0.len(),
        }
    }    
}

fn board_from_board<T: Clone, U: Clone>(board: Vec<Vec<U>>, value: T) -> Vec<Vec<T>> {
    let size_y = board.len();
    let size_x = board[0].len();
    let board = vec![vec![value; size_x]; size_y];
    board
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

fn add_zones_rects(mut board: Vec<Vec<u16>>, zones: u16) -> Vec<Vec<u16>> {
    if zones < 2 {
        return board;
    }
    let size_y = board.len();
    let size_x = board[0].len();
    let mut rng = rand::thread_rng();
    let source_x = rng.gen_range(0, size_x);
    let source_y = rng.gen_range(0, size_y);
    board[source_y][source_x] = 1;
    let areas = get_zone_sizes(zones, size_x, size_y, 0.75);
        
    let mut x0 = source_x;
    let mut x1 = source_x + 1;
    let mut y0 = source_y;
    let mut y1: usize = source_y + 1;

    for zone in 1..zones {
        while areas[zone as usize] > (((x1 - x0) * (y1 - y0)) as u16) {
            let grow = rng.gen_range(0, 4);
            match grow {            
                0 => if x0 > 1 {x0 -= 1},
                1 => x1 = (x1 + 1).min(size_x),
                2 => if y0 > 1 {y0 -= 1},
                3 => y1 = (y1 + 1).min(size_y),
                _ => println!("{}", grow),
            };
        }
        for x in x0..x1 {
            for y in y0..y1 {
                if board[y][x] == 0 {
                    board[y][x] = zone + 1;
                }
            }
        }
    }

    for x in 0..size_x{
        for y in 0..size_y{
            if board[y][x] == 0 {
                board[y][x] = zones + 1;
            }
        }
    }

    board
}

fn main() {
    let world_size = 16;
    let board_zones = Board::new(world_size * 3, world_size, 0);
    println!("{:?}", board_zones.shape());
    /*
    board_zones = add_zones_rects(board_zones, 4);
    let board_fog = board_from_board(board_zones, 0 as f32);
    for row in board_zones {
        let out = row.into_iter().map(|i| i.to_string()).collect::<String>();
        println!("{}", out);
    }
    */
}
