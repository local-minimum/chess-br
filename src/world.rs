use std::collections::HashMap;

use crate::world::position::{Coord, Offset, Positional};
use crate::world::builders::{add_zones_rects, add_fog, add_fly_path};
use crate::world::board::Board;
use crate::world::pieces::{Piece, PieceType};
use crate::world::player::{Player, PlayerState, GamerNamer};
use crate::world::fog::Fog;
use crate::world::flyer::Flyer;
use crate::world::historian::Historian;

pub mod board;
pub mod builders;
pub mod display;
pub mod position;
pub mod pieces;
pub mod direction;
pub mod player;
pub mod fog;
pub mod flyer;
pub mod historian;

#[derive(Debug, Copy, Clone)]
pub enum Action {
    None(u16),
    Drop(u16),
    Fly(u16, Offset),
    Move(u16, Coord, Coord),
}

struct WorldSettings {
    drop_height: u16,
    zone_every: usize,
    zone_rest: usize,
    flyer_every: usize,
    fly_start: i16,
    verbose: bool,
}

impl WorldSettings {
    fn new() -> WorldSettings {
        WorldSettings{
            fly_start: -5,
            drop_height: 10,
            zone_every: 10,
            zone_rest: 42,
            flyer_every: 1,
            verbose: true,
        }
    }
}

pub struct World {
    settings: WorldSettings,
    pub fog: Fog,
    pub pieces: HashMap<u16, Piece>,
    pub pieces_map: Vec<Vec<u16>>,
    pub flyer: Flyer,
    pub players: HashMap<u16, Player>,

    req_air_action: Vec<Action>,
    req_board_action: Vec<Action>,
    alive: bool,
    tick: usize,
    historian: Historian,
}

impl World {
    fn new(shape: Coord) -> Self {
        let fog = Fog::new(shape);
        let pieces = fog.zones.new_with(0);
        let settings = WorldSettings::new();
        World {
            fog,
            flyer: Flyer::new(settings.fly_start),
            historian: Historian::new(settings.verbose),
            settings,
            pieces: HashMap::new(),
            pieces_map: pieces,
            players: HashMap::new(),
            req_air_action: Vec::new(),
            req_board_action: Vec::new(),
            tick: 0,
            alive: true,
        }
    }

    pub fn request_action(&mut self, action: Action) {
        match action {
            Action::Drop(user) => {

                if self.players.contains_key(&user) && self.players[&user].state.is_flying() && self.flyer.can_drop() {
                    self.req_air_action.push(action);
                }
            }
            Action::Fly(user, _off) => {
                if self.players.contains_key(&user) && self.players[&user].state.can_fly() {
                    self.req_air_action.push(action);
                }
            }
            Action::Move(user, _from, _to) => {
                if self.players.contains_key(&user) && self.players[&user].state.is_boarded() {
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
                let mut other_piece_id = 0 as u16;
                let mut kind = PieceType::Empty;
                match self.pieces.get_mut(&piece_id) {
                    Some(piece) => {
                        if piece.player != user || !piece.alive { return; }
                        match piece.kind.intermediat_steps(from, to) {
                            None => (),
                            Some(steps) => {
                                for step in steps {
                                    if self.pieces_map[step.y][step.x] > 0 {
                                        return ;
                                    }
                                }

                                // Taking (preparing, actual happens below)
                                other_piece_id = self.pieces_map[to.y][to.x];
                                kind = piece.kind.clone();

                                // Moving
                                piece.place(to.clone());
                                self.pieces_map[to.y][to.x] = piece_id;
                                self.pieces_map[from.y][from.x] = 0;
                                self.historian.record_player(
                                    user,
                                    self.tick,
                                    piece.kind,
                                    format!("Move {:?} -> {:?}", from, to),
                                )
                            }
                        }
                    }
                    _ => (),
                }
                self.do_take(other_piece_id, user, kind);
            },
            _ => ()
        }
    }

    fn do_take(&mut self, other_piece_id: u16, taker_id: u16, taken_by: PieceType) {
        match self.pieces.get_mut(&other_piece_id) {
            Some(other) => {
                if !other.alive { return; }
                let pos = other.position().unwrap().clone();
                other.alive = false;
                self.players.get_mut(&taker_id).unwrap().score += other.kind.value();
                match other.kind {
                    PieceType::Empty => (),
                    PieceType::King => {
                        if self.players.contains_key(&other.player) {
                            let rank = self.players.iter()
                                .filter(| (_, p) | p.state.is_alive()).count();
                            if rank == 2 {
                                // Give rank 1 to self
                                self.players.get_mut(&taker_id).unwrap()
                                    .transition(PlayerState::Dead(1));
                                self.alive = false;
                            }
                            self.players.get_mut(&other.player).unwrap()
                                .transition(PlayerState::Dead(rank as u16));
                        }
                        self.historian.record_player(
                            taker_id,
                            self.tick,
                            taken_by,
                            format!("Capture {:?} @ {:?}", other.kind, pos),
                        );
                    },
                    _ => {
                        self.historian.record_player(
                            taker_id,
                            self.tick,
                            taken_by,
                            format!("Capture {:?} @ {:?}", other.kind, pos),
                        )
                    }
                }
            },
            _ => (),
        }

    }

    fn do_move_falling(&mut self, mut fly_actions: Vec<Action>) {
        let shape = self.fog.shape();
        while let Some(action) = fly_actions.pop() {
            match action {
                Action::Fly(user, off) => {
                    if !self.players.contains_key(&user) || !self.players[&user].state.can_fly() { continue; }
                    match self.players[&user].state {
                        PlayerState::Falling(h, coord) => {
                            let next_coord = coord.translate(off);
                            if next_coord.is_inside(&shape) {
                                self.players.get_mut(&user).unwrap().transition(PlayerState::Falling(h, next_coord));
                                self.historian.record_player(
                                    user,
                                    self.tick,
                                    PieceType::King,
                                    format!("Fly -> {:?}", next_coord),
                                );
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }

    fn do_drop(&mut self, mut drop_actions: Vec<Action>) {
        if self.flyer.must_drop() {
            // Force Drop
            match self.flyer.coord() {
                Some(flyer) => {
                    let uids: Vec<u16> = self.players.iter().map(|(k, _)| k.clone()).collect();
                    for uid in uids {
                        if !self.players.contains_key(&uid) || !self.players[&uid].state.is_flying() { continue; }
                        self.players.get_mut(&uid).unwrap().transition(PlayerState::Falling(self.settings.drop_height, flyer.clone()));
                        self.historian.record_player(
                            uid,
                            self.tick,
                            PieceType::King,
                            format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
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
                                if !self.players.contains_key(&user) || !self.players[&user].state.is_flying() { continue; }
                                self.players.get_mut(&user).unwrap().transition(PlayerState::Falling(self.settings.drop_height, flyer.clone()));
                                self.historian.record_player(
                                    user,
                                    self.tick,
                                    PieceType::King,
                                    format!("Drop -> {:?}:{}", flyer, self.settings.drop_height),
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
        let uids: Vec<u16> = self.players.iter().map(|(k, _)| k.clone()).collect();
        for uid in uids.iter() {
            match self.players[uid].state {
                PlayerState::Falling(h, coord) => {
                    if h > 1 {
                        self.players.get_mut(uid).unwrap().transition(PlayerState::Falling(h - 1, coord));
                        self.historian.record_player(
                            *uid,
                            self.tick,
                            PieceType::King,
                            format!("Fall -> {:?}:{}", coord, h - 1),
                        )
                    } else {
                        self.players.get_mut(uid).unwrap().transition(PlayerState::Boarded);
                        let mut piece = Piece::new(PieceType::King, *uid);
                        piece.place(coord.clone());
                        let piece_id = self.pieces.len() as u16 + 1;
                        self.pieces.insert(piece_id, piece);
                        if self.pieces_map[coord.y][coord.x] > 0 {
                            self.do_take(
                                self.pieces_map[coord.y][coord.x],
                                *uid,
                                PieceType::King,
                            )
                        }
                        self.pieces_map[coord.y][coord.x] = piece_id;
                        self.historian.record_player(
                            *uid,
                            self.tick,
                            PieceType::King,
                            format!("Land -> {:?}", coord),
                        );
                    }
                },
                _ => (),
            }
        }
    }

    pub fn do_tick(&mut self) {
        if !self.alive { return; }
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
        let mut players: Vec<Player> = self.players
            .iter()
            .map(|(_, p)| p.clone())
            .collect();
        players.sort_by(|a, b| b.score.cmp(&a.score));
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
        self.players.iter().filter(| (_, p) | p.state.is_flying()).count()
    }

    pub fn airborne_count(&self) -> usize {
        self.players.iter().filter(| (_, p) | p.state.is_airborne()).count()
    }

    pub fn add_player(&mut self, user_name: String, gamer_namer: &mut GamerNamer) {
        let uid = self.players.len() as u16 + 1;
        self.players.insert(uid, Player::new(uid, user_name, gamer_namer));
    }
}

pub fn spawn(shape: Coord, nzones: u16, players: &Vec<String>) -> World {
    let mut world = World::new(shape);
    world.fog.init(nzones, add_zones_rects, add_fog);
    world.flyer.init(world.fog.shape(), add_fly_path);
    let mut namer = GamerNamer::new();
    for player in players.iter() {
        world.add_player( player.clone(), &mut namer);
    }
    world
}
