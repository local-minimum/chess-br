use crate::world::position::Coord;

pub struct Flyer {
    path: Vec<Coord>,
    idx: i16,
    map_shape: Coord,
}

impl Flyer {
    pub fn new(start_idx: i16) -> Self {
        Flyer{
            idx: start_idx,
            path: Vec::new(),
            map_shape: Coord{x: 0, y: 0},
        }
    }

    pub fn init(&mut self, shape: Coord, pather: fn(&mut Vec<Coord>, Coord)) {
        self.map_shape = shape.clone();
        pather(&mut self.path, shape);
    }

    pub fn can_drop(&self) -> bool {
        self.idx >= 0
    }

    pub fn must_drop(&self) -> bool {
        self.path.len() as i16 - 1 == self.idx
    }

    pub fn flying(&self) -> bool {
        self.idx < self.path.len() as i16
    }

    pub fn coord(&self) -> Option<Coord> {
        if self.idx < 0 { return None }
        if self.idx >= self.path.len() as i16 { return None }
        Some(self.path[self.idx as usize])
    }

    pub fn tick(&mut self) {
        self.idx += 1;
    }

    pub fn as_map(&self) -> Vec<Vec<u16>> {
        let mut pathmap = vec![vec![0 as u16; self.map_shape.x]; self.map_shape.y];
        for idx in 0..self.path.len() {
            let coord = self.path[idx];
            if self.idx == idx as i16 {
                pathmap[coord.y][coord.x] = 33; // X
            } else {
                pathmap[coord.y][coord.x] = 1;
            }
        }
        pathmap
    }

}
