use ::chess_br::playingfield::spawn;
use ::chess_br::playingfield::Coord;
use ::chess_br::playingfield::encode_ch;
use ::chess_br::playingfield::print_board;
use ::chess_br::playingfield::FogState;

fn main() {
    let mut world = spawn(Coord{x: 42, y: 16}, 4);
    for (zone_row, fog_row) in world.zones.iter().zip(world.fog_curve.iter()) {
        let zone_out = zone_row
            .into_iter()
            .map(|i| encode_ch(*i))
            .collect::<String>();
        let fog_out = fog_row
            .into_iter()
            .map(|i| encode_ch(*i))
            .collect::<String>();
        println!("{} {}", zone_out, fog_out);
    }
    let mut fog_type = FogState::Contracting;
    loop {
        println!("\n** {} {:?}", world.status(), fog_type);
        print_board(&world.fog);
        fog_type = world.contract_fog();
        match fog_type {
            FogState::Done => break,
            _ => (),
        }
    }
    println!("\n** {} {:?}", world.status(), fog_type);
    print_board(&world.fog);
}
