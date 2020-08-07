use std::collections::HashMap;

use crate::world::position::{Coord, Offset, Positional};
use crate::world::builders::{add_zones_rects, add_fog, add_fly_path};
use crate::world::board::Board;
use crate::world::pieces::{Piece, PieceType};
use crate::world::player::{Player, GamerNamer};
use crate::world::fog::Fog;

pub mod board;
pub mod builders;
pub mod display;
pub mod position;
pub mod pieces;
pub mod direction;
pub mod player;
pub mod fog;


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
    pub piece: PieceType,
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
            flyer_every: 1,
        }
    }
}

pub struct World {
    settings: WorldSettings,
    pub fog: Fog,
    pub pieces: HashMap<u16, Piece>,
    pub pieces_map: Vec<Vec<u16>>,
    pub fly_path: Vec<Coord>,
    fly_path_idx: i16,
    players: Vec<Player>,
    flying: Vec<u16>,
    boarded: Vec<u16>,
    pub falling: Vec<(u16, u16, Coord)>,
    req_air_action: Vec<Action>,
    req_board_action: Vec<Action>,
    tick: usize,
    history: Vec<Record>,
}

impl World {
    fn new(shape: Coord) -> Self {
        let fog = Fog::new(shape);
        let pieces = fog.zones.new_with(0);
        let settings = WorldSettings::new();
        World {
            fog,
            fly_path_idx: settings.fly_start,
            settings,
            pieces: HashMap::new(),
            pieces_map: pieces,
            fly_path: Vec::new(),
            players: Vec::new(),
            flying: Vec::new(),
            falling: Vec::new(),
            boarded: Vec::new(),
            req_air_action: Vec::new(),
            req_board_action: Vec::new(),
            tick: 0,
            history: Vec::new(),
        }
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
            Action::Move(user, _from, _to) => {
                if self.boarded.contains(&user) {
                    self.req_board_action.retain(|a| match a {
                        Action::Move(uid, _from, _to) => return *uid != user,
                        _ => return true
                    });
                }
                self.req_board_action.push(action);
            }
            Action::None(user) => {
                self.req_air_action.retain(|a | match a {
                    Action::Drop(uid) => return *uid == user,
                    Action::Fly(uid, _off) => return *uid == user,
                    _ => return true,
                });
                self.req_board_action.retain(|a | match a {
                    Action::Move(uid, _from, _to) => return *uid == user,
                    _ => return true,
                });
            }
        }
    }

    pub fn do_board_move(&mut self, action: Action) {
        match action {
            Action::Move(user, from, to) => {
                let piece_id = self.pieces_map[from.y][from.x];
                if piece_id == 0 { return; }
                match self.pieces.get_mut(&piece_id) {
                    Some(piece) => {
                        if piece.player != user { return; }
                        match piece.kind.intermediat_steps(from, to) {
                            None => (),
                            Some(steps) => {
                                for step in steps {
                                    if self.pieces_map[step.y][step.x] > 0 {
                                        return ;
                                    }
                                }
                                // Taking
                                // Moving
                                piece.place(&to);
                                self.pieces_map[to.y][to.x] = piece_id;
                                self.pieces_map[from.y][from.x] = 0;
                            }
                        }

                    }
                    _ => (),
                }
            },
            _ => ()
        }
    }

    fn do_move_falling(&mut self, mut fly_actions: Vec<Action>) {
        let shape = self.fog.shape();
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
                                    piece: PieceType::King,
                                    event: format!("Fly -> {:?}", next_coord),
                                }
                            );
                        }
                    }
                },
                _ => (),
            }
        }
    }

    fn do_drop(&mut self, mut drop_actions: Vec<Action>) {
        let last_fly_idx = self.fly_path.len() as i16 - 1;
        if self.fly_path_idx == last_fly_idx {
            // Force Drop
            let flyer = self.fly_path[self.fly_path_idx as usize];
            for uid in self.flying.iter() {
                self.falling.push((*uid, self.settings.drop_height, flyer.clone()));
                self.history.push(
                    Record{
                        player: *uid,
                        tick: self.tick,
                        piece: PieceType::King,
                        event: format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
                    }
                );
            }
            self.flying.clear();
        } else if self.fly_path_idx >= 0 && self.fly_path_idx < last_fly_idx {
            // Drop everyone that wants to
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
                                    piece: PieceType::King,
                                    event: format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
                                }
                            );
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn do_lower_falling(&mut self) {
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
                    piece: PieceType::King,
                    tick: self.tick,
                    event: format!("Fall -> {:?}:{}", *coord, *h),
                }
            )
        }

        for (uid, _h, coord) in landers {
            let mut piece = Piece::new(PieceType::King, uid);
            piece.place(&coord);
            let piece_id = self.pieces_map.max_val() + 1;
            self.pieces_map[coord.y][coord.x] = piece_id;
            self.pieces.insert(piece_id, piece);
            self.history.push(
                Record{
                    player: uid,
                    piece: PieceType::King,
                    tick: self.tick,
                    event: format!("Land -> {:?}", coord),
                }
            );
            self.boarded.push(uid);
        }

    }

    pub fn do_tick(&mut self) {
        // Modify world
        if self.fly_path_idx < self.fly_path.len() as i16 {
            if (self.tick % self.settings.flyer_every) == 0 { self.fly_path_idx += 1; }
        } else {
            if (self.tick % self.settings.zone_every) == 0 { self.fog.contract(self.settings.zone_rest); }
        }

        // Copy concurrent actions
        let drop_actions: Vec<Action> = self.req_air_action
            .iter()
            .filter(| e | match e { Action::Drop(_u) => true, _ => false })
            .map(|e | e.clone())
            .collect();
        let fly_actions: Vec<Action> = self.req_air_action
            .iter()
            .filter(| e | match e { Action::Fly(_u, _off) => true, _ => false })
            .map(|e | e.clone())
            .collect();
        // Reset concurrent action requests
        self.req_air_action.clear();

        self.do_move_falling(fly_actions);
        self.do_lower_falling();
        self.do_drop(drop_actions);

        let action = self.req_board_action.pop();
        match action {
            Some(action) => self.do_board_move(action),
            _ => ()
        }
        self.tick += 1;
    }

    pub fn flypath_map(&self) -> Vec<Vec<u16>> {
        let mut pathmap = self.fog.zones.new_with(0 as u16);
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

    pub fn players_by_score(&self) -> Vec<Player> {
        let mut players = self.players.clone();
        players.sort_by(|a, b| a.score.cmp(&b.score));
        players
    }

    pub fn player_positions(&self, user: u16) -> Vec<(PieceType, Coord)> {
        let mut pos = Vec::new();
        for piece in self.pieces.values() {
            if piece.player == user {
                match piece.position() {
                    Some(coord) => {
                        pos.push((piece.kind, coord.clone()))
                    }
                    _ => (),
                }
            }
        }
        return pos
    }

    pub fn flyers_count(&self) -> usize {
        self.flying.len()
    }

    pub fn falling_count(&self) -> usize {
        self.falling.len()
    }
}

pub fn spawn(shape: Coord, nzones: u16, players: &Vec<String>) -> World {
    let mut world = World::new(shape);
    world.fog.init(nzones, add_zones_rects, add_fog);
    add_fly_path(&mut world.fly_path, world.fog.shape());
    let mut namer = GamerNamer::new();
    for (idx, player) in players.iter().enumerate() {
        world.players.push(Player::new(idx as u16 + 1, player.clone(), &mut namer));
        world.flying.push(idx as u16 + 1);
    }
    world
}
