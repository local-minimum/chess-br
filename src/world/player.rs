use rand::seq::SliceRandom;

use crate::world::position::Coord;

#[derive(Debug, Clone)]
pub enum PlayerState {
    Flying,
    Falling(u16, Coord), // height and position
    Boarded,
    Dead(u16), // Includes the rank
}

impl PlayerState {
    pub fn is_flying(&self) -> bool {
        match self {
            PlayerState::Flying => true,
            _ => false,
        }
    }

    pub fn can_fly(&self) -> bool {
        match self {
            PlayerState::Falling(h, _coord) => *h > 1,
            _ => false,
        }
    }

    pub fn is_landing(&self) -> bool {
        match self {
            PlayerState::Falling(h, _coord) => *h == 1,
            _ => false,
        }
    }

    pub fn is_boarded(&self) -> bool {
        match self {
            PlayerState::Boarded => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub game_id: u16,
    pub game_name: String,
    pub user_name: String,
    pub score: u16,
    pub state: PlayerState,
}

impl Player {
    pub fn new(game_id: u16, user_name: String, namer: &mut GamerNamer) -> Player {
        Player{
            game_id,
            game_name: namer.next(),
            user_name: user_name.clone(),
            score: 0,
            state: PlayerState::Flying,
        }
    }

    pub fn in_game_info(&self) -> (u16, String) {
        (self.score, self.game_name.clone())
    }

    pub fn transition(&mut self, state: PlayerState) {
        self.state = state;
    }
}

const GAME_NAME_ADJ: &'static [&'static str]  = &[
    "Quick", "Sneaky", "Lazy", "Clever", "Wise", "Lucky",
    "Nervous", "Happy", "Shrew", "Timid", "Advanced", "Bare",
    "Artificial", "Big", "Small", "Blocked", "Open", "Closed",
    "Connected", "Discovered", "Double", "Exposed", "Good",
    "Hanging", "Fianchettoed", "Active",
];

const GAME_NAME_NOUN: &'static [&'static str] = &[
    "Pawn", "Rook", "Bishop", "Knight", "Queen", "King", "Castling",
    "Gambit", "Check", "Stalemate", "Promotion", "Pin", "Sicilian",
    "Bind", "Blunder", "Move", "Break", "Piece", "Capture", "Center",
    "Attack", "Defence", "Exchange", "Fork", "File", "Rank", "Line",
    "Storm", "Skewer",
];

pub struct GamerNamer {
    adj_idxs: Vec<usize>,
    noun_idxs: Vec<usize>,
    adj_idx: usize,
    noun_idx: usize,
}

impl GamerNamer {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut adj_idxs: Vec<usize> = (0..GAME_NAME_ADJ.len()).collect();
        adj_idxs.shuffle(&mut rng);
        let mut noun_idxs: Vec<usize> = (0..GAME_NAME_NOUN.len()).collect();
        noun_idxs.shuffle(&mut rng);
        GamerNamer{
            adj_idxs,
            noun_idxs,
            adj_idx: 0,
            noun_idx: 0,
        }
    }

    pub fn next(&mut self) -> String {
        let ret = format!(
            "{} {}",
            GAME_NAME_ADJ[self.adj_idxs[self.adj_idx]],
            GAME_NAME_NOUN[self.noun_idxs[self.noun_idx]],
        );
        self.adj_idx += 1;
        if self.adj_idx >= self.adj_idxs.len() { self.adj_idx = 0; }
        self.noun_idx += 1;
        if self.noun_idx >= self.noun_idxs.len() { self.noun_idx = 0; }
        ret
    }
}
