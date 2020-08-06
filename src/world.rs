use crate::world::position::{Coord, Offset, Positional};
use crate::world::builders::{add_zones_rects, add_fog, add_fly_path};
use crate::world::board::Board;
use crate::world::pieces::Pieces;
use crate::world::player::{Player, GameNamer};

pub mod board;
pub mod builders;
pub mod display;
pub mod position;
pub mod pieces;
pub mod direction;
pub mod player;

#[derive(Debug)]
pub enum FogState {
    Resting,
    Contracting,
    Zone,
    Done,
}

#[derive(Debug, Copy, Clone)]
pub enum Action {
    None(u16),
    Drop(u16),
    Fly(u16, Offset),
    Move(u16, Coord, Coord),
}

pub struct Record {
    pub player: u16,
    pub tick: usize,
    pub piece: Pieces,
    pub event: String,
}

struct WorldSettings {
    drop_height: u16,
    zone_every: usize,
    zone_rest: usize,
    flyer_every: usize,
    fly_start: i16,
}

impl WorldSettings {
    fn new() -> WorldSettings {
        WorldSettings{
            fly_start: -5,
            drop_height: 10,
            zone_every: 10,
            zone_rest: 42,
            flyer_every: 2,
        }
    }
}

pub struct World {
    settings: WorldSettings,
    pub zones: Vec<Vec<u16>>,
    pub fog_curve: Vec<Vec<u16>>,
    pub fog: Vec<Vec<u16>>,
    pub pieces_types: Vec<Vec<Pieces>>,
    pub pieces_player: Vec<Vec<u16>>,
    pub fly_path: Vec<Coord>,
    fly_path_idx: i16,
    players: Vec<Player>,
    zone_rest: usize,
    fog_value: u16,
    active_zone: u16,    
    flying: Vec<u16>,
    pub falling: Vec<(u16, u16, Coord)>,
    req_air_action: Vec<Action>,
    tick: usize,
    history: Vec<Record>,
}

impl World {
    fn new(shape: Coord) -> Self {
        let zones = vec![vec![0; shape.x]; shape.y];
        let fog_curve = zones.new_with(0);
        let fog = zones.new_with(0);
        let player = zones.new_with(0);
        let pieces = zones.new_with(Pieces::Empty);
        let settings = WorldSettings::new();
        World {
            fly_path_idx: settings.fly_start,
            zone_rest: settings.zone_rest,
            settings,
            zones,
            fog_curve,
            fog,
            fog_value: 0,
            active_zone: 0,
            pieces_types: pieces,
            pieces_player: player,
            fly_path: Vec::new(),
            players: Vec::new(),
            flying: Vec::new(),
            falling: Vec::new(),
            req_air_action: Vec::new(),
            tick: 0,
            history: Vec::new(),
        }
    }

    pub fn contract_fog(&mut self) -> FogState {
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
                self.zone_rest = self.settings.zone_rest;
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

    fn has_piece_on_path(&self, steps: Vec<Coord>) -> bool {
        for step in steps {
            match self.pieces_types[step.y][step.x] {
                Pieces::Empty => { return true; },
                _ => (),
            }
        }
        false
    }

    pub fn request_action(&mut self, action: Action) {
        match action {
            Action::Drop(user) => {
                if self.flying.contains(&user) && self.fly_path_idx >= 0 {
                    self.req_air_action.push(action);
                }
            }
            Action::Fly(user, _off) => {
                if self.falling.iter().any(|(uid, h, _c)| *uid == user && *h > 1) {
                    self.req_air_action.push(action);
                }
            }
            _ => ()
        }
    }

    pub fn do_move(&mut self, action: Action) {
        match action {
            Action::Move(user, from, to) => {
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
            _ => ()
        }
    }

    pub fn flypath_map(&self) -> Vec<Vec<u16>> {
        let mut pathmap = self.zones.new_with(0 as u16);
        for idx in 0..self.fly_path.len() {
            let coord = self.fly_path[idx];
            if self.fly_path_idx == idx as i16 {
                pathmap[coord.y][coord.x] = 33; // X
            } else {
                pathmap[coord.y][coord.x] = 1;
            }
        }
        pathmap
    }

    pub fn do_tick(&mut self) {
        // Modify world
        if self.fly_path_idx < self.fly_path.len() as i16 {
            if (self.tick % self.settings.flyer_every) == 0 { self.fly_path_idx += 1; }
        } else {
            if (self.tick % self.settings.zone_every) == 0 { self.contract_fog(); }
        }

        let mut drop_actions: Vec<Action> = self.req_air_action
            .iter()
            .filter(| e | match e { Action::Drop(_u) => true, _ => false })
            .map(|e | e.clone())
            .collect();
        let mut fly_actions: Vec<Action> = self.req_air_action
            .iter()
            .filter(| e | match e { Action::Fly(_u, _off) => true, _ => false })
            .map(|e | e.clone())
            .collect();

        // Reset requests
        self.req_air_action.clear();

        // Move those that falling
        let shape = self.zones.shape();
        while let Some(action) = fly_actions.pop() {
            match action {
                Action::Fly(user, off) => {
                    let mut itemidx = -1;
                    for (idx, item) in self.falling.iter().enumerate() {
                        if user != item.0 { continue; }
                        itemidx = idx as i32;
                        break;
                    }
                    if itemidx > -1 {
                        let (uid, height, coord) = self.falling[itemidx as usize];
                        let next_coord = coord.translate(off);
                        if next_coord.is_inside(&shape) {
                            self.falling[itemidx as usize] = (uid, height, next_coord);
                            self.history.push(
                                Record{
                                    player: user,
                                    tick: self.tick,
                                    piece: Pieces::King,
                                    event: format!("Fly -> {:?}", next_coord),
                                }
                            );
                        }
                    }
                },
                _ => (),
            }
        }

        // Drop everyone that wants to
        if self.fly_path_idx >= 0 {
            let flyer = self.fly_path[self.fly_path_idx as usize];
            while let Some(action) = drop_actions.pop() {
                match action {
                    Action::Drop(user) => {
                        if self.flying.contains(&user) {
                            self.flying.retain(|a| *a != user);
                            self.falling.push((user, self.settings.drop_height, flyer.clone()));
                            self.history.push(
                                Record{
                                    player: user,
                                    tick: self.tick,
                                    piece: Pieces::King,
                                    event: format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
                                }
                            );
                        }
                    }
                    _ => (),
                }
            }
        }

        // Lower fallings
        let landers: Vec<(u16, u16, Coord)> = self.falling
            .iter()
            .filter(| (_uid, height, _coord) | *height == 1)
            .map(| (uid, height, coord) | (*uid, *height, coord.clone()))
            .collect();

        self.falling = self.falling
            .iter()
            .filter(| (_uid, height, _coord) | *height > 1)
            .map(| (uid, height, coord) | (*uid, *height - 1, coord.clone()))
            .collect();

        for (uid, h, coord) in self.falling.iter() {
            self.history.push(
                Record{
                    player: *uid,
                    piece: Pieces::King,
                    tick: self.tick,
                    event: format!("Fall -> {:?}:{}", *coord, *h), 
                }
            )
        }

        for (uid, _h, coord) in landers {
            self.pieces_player[coord.y][coord.x] = uid;
            self.pieces_types[coord.y][coord.x] = Pieces::King;
            self.history.push(
                Record{
                    player: uid,
                    piece: Pieces::King,
                    tick: self.tick,
                    event: format!("Land -> {:?}", coord),
                }
            )
        }

        self.tick += 1;
    }

    pub fn players_by_score(&self) -> Vec<Player> {
        let mut players = self.players.clone();
        players.sort_by(|a, b| a.score.cmp(&b.score));
        players
    }

    pub fn flyers_count(&self) -> usize {
        self.flying.len()
    }
}

pub fn spawn(shape: Coord, nzones: u16, players: &Vec<String>) -> World {
    let mut world = World::new(shape);
    add_zones_rects(&mut world.zones, nzones);
    add_fog(&mut world.fog_curve, &world.zones);
    add_fly_path(&mut world.fly_path, world.zones.shape());
    world.active_zone = world.zones.max_val() + 1;
    let mut namer = GameNamer::new();
    for (idx, player) in players.iter().enumerate() {
        world.players.push(Player::new(idx as u16 + 1, player.clone(), &mut namer));
        world.flying.push(idx as u16 + 1);
    }
    world
}
