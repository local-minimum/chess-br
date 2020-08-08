use std::collections::HashMap;

use crate::world::position::{Coord, Offset, Positional};
use crate::world::builders::{add_zones_rects, add_fog, add_fly_path};
use crate::world::board::Board;
use crate::world::pieces::{Piece, PieceType};
use crate::world::player::{Player, PlayerState, GamerNamer};
use crate::world::fog::Fog;
use crate::world::flyer::Flyer;

pub mod board;
pub mod builders;
pub mod display;
pub mod position;
pub mod pieces;
pub mod direction;
pub mod player;
pub mod fog;
pub mod flyer;

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
    pub flyer: Flyer,
    pub players: Vec<Player>,

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
            flyer: Flyer::new(settings.fly_start),
            settings,
            pieces: HashMap::new(),
            pieces_map: pieces,
            players: Vec::new(),
            req_air_action: Vec::new(),
            req_board_action: Vec::new(),
            tick: 0,
            history: Vec::new(),
        }
    }

    pub fn request_action(&mut self, action: Action) {
        match action {
            Action::Drop(user) => {
                if self.players.iter().any(|p| p.player_id == user && p.state.is_flying()) && self.flyer.can_drop() {
                    self.req_air_action.push(action);
                }
            }
            Action::Fly(user, _off) => {
                if self.players.iter().any(| p | p.player_id == user && p.state.can_fly()) {
                    self.req_air_action.push(action);
                }
            }
            Action::Move(user, _from, _to) => {
                if self.players.iter().any(| p | p.player_id == user && p.state.is_boarded()) {
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
                    for idx in 0..self.players.len() {
                        if self.players[idx].player_id != user || !self.players[idx].state.can_fly() { continue; }
                        match self.players[idx].state {
                            PlayerState::Falling(h, coord) => {
                                let next_coord = coord.translate(off);
                                if next_coord.is_inside(&shape) {
                                    self.players[idx].transition(PlayerState::Falling(h, next_coord));
                                    self.history.push(
                                        Record{
                                            player: user,
                                            tick: self.tick,
                                            piece: PieceType::King,
                                            event: format!("Fly -> {:?}", next_coord),
                                        }
                                    );
                                }
                            },
                            _ => (),
                        }
                    }
                },
                _ => (),
            }
        }
    }

    fn transition_player(&mut self, player: u16, state: PlayerState) {
        for idx in 0..self.players.len() {
            if self.players[idx].player_id != player { continue; }
            self.players[idx].transition(state);
            break
        }
    }

    fn do_drop(&mut self, mut drop_actions: Vec<Action>) {
        if self.flyer.must_drop() {
            // Force Drop
            match self.flyer.coord() {
                Some(flyer) => {
                    for idx in 0..self.players.len() {
                        if !self.players[idx].state.is_flying() { continue; }
                        self.players[idx].transition(PlayerState::Falling(self.settings.drop_height, flyer.clone()));
                        self.history.push(
                            Record{
                                player: self.players[idx].player_id,
                                tick: self.tick,
                                piece: PieceType::King,
                                event: format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
                            }
                        );
                    }
                }
                _ => (),
            }
        } else {
            // Drop everyone that wants to
            match self.flyer.coord() {
                Some(flyer) => {
                    while let Some(action) = drop_actions.pop() {
                        match action {
                            Action::Drop(user) => {
                                self.transition_player(user, PlayerState::Falling(self.settings.drop_height, flyer.clone()));
                                self.history.push(
                                    Record{
                                        player: user,
                                        tick: self.tick,
                                        piece: PieceType::King,
                                        event: format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
                                    }
                                );
                            }
                            _ => (),
                        }
                    }

                },
                _ => (),
            }
        }
    }

    fn do_lower_falling(&mut self) {
        for idx in 0..self.players.len() {
            match self.players[idx].state {
                PlayerState::Falling(h, coord) => {
                    let uid = self.players[idx].player_id;
                    if h > 1 {
                        self.players[idx].transition(PlayerState::Falling(h - 1, coord));
                        self.history.push(
                            Record{
                                player: uid,
                                piece: PieceType::King,
                                tick: self.tick,
                                event: format!("Fall -> {:?}:{}", coord, h - 1),
                            }
                        )
                    } else {
                        self.players[idx].transition(PlayerState::Boarded);
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
                    }
                },
                _ => (),
            }
        }
    }

    pub fn do_tick(&mut self) {
        // Modify world
        if self.flyer.flying() {
            if (self.tick % self.settings.flyer_every) == 0 { self.flyer.tick(); }
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
        self.players.iter().filter(| p | p.state.is_flying()).count()
    }

    pub fn airborne_count(&self) -> usize {
        self.players.iter().filter(| p | p.state.is_airborne()).count()
    }
}

pub fn spawn(shape: Coord, nzones: u16, players: &Vec<String>) -> World {
    let mut world = World::new(shape);
    world.fog.init(nzones, add_zones_rects, add_fog);
    world.flyer.init(world.fog.shape(), add_fly_path);
    let mut namer = GamerNamer::new();
    for (idx, player) in players.iter().enumerate() {
        world.players.push(Player::new(idx as u16 + 1, player.clone(), &mut namer));
    }
    world
}
