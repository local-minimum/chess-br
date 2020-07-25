use ::chess_br::world::spawn;
use ::chess_br::world::World;
use ::chess_br::world::FogState;
use ::chess_br::world::position::Coord;
use ::chess_br::world::display::{print_board, print_board_pair};

fn play_fog(mut world: World, print_fog: bool, print_next_zone: bool) {
    let mut fog_type = FogState::Contracting;
    loop {
        if print_fog {
            println!("\n** {} {:?}", world.status(), fog_type);
            print_board(&world.fog);
        }
        fog_type = world.contract_fog();
        match fog_type {
            FogState::Done => break,
            FogState::Zone => if print_next_zone {print_board(&world.next_zone(true));},
            _ => (),
        }
    }
    if print_fog {
        println!("\n** {} {:?}", world.status(), fog_type);
        print_board(&world.fog);
    }
}

fn main() {
    let world = spawn(Coord{x: 42, y: 16}, 4);
    print_board_pair(&world.zones, &world.fog_curve);  
    play_fog(world, false, true);
}
