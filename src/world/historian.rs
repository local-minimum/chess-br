use crate::world::pieces::PieceType;

#[derive(Debug)]
pub struct Record {    
    pub player: u16,
    pub tick: usize,
    pub piece: PieceType,
    pub event: String,
}

impl Record {
    pub fn summarize(&self) -> String {
        format!("Tick {}, player {}: {:?} {}", self.tick, self.player, self.piece, self.event)
    }
}

#[derive(Debug)]
pub struct Historian {
    player_record: Vec<Record>,
    print_events: bool
}

impl Historian {
    pub fn new(print_events: bool) -> Self {
        Historian{
            player_record: Vec::new(),
            print_events,
        }
    }

    pub fn record_player(
        &mut self,
        player: u16,
        tick: usize,
        piece: PieceType,
        event: String,
    ) {
        let record = Record{player, tick, piece, event};
        if self.print_events { println!("{}", record.summarize()); }
        self.player_record.push(record)
    }
}