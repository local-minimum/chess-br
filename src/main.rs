use ::chess_br::world::spawn;
use ::chess_br::world::{World, Action};
use ::chess_br::world::direction::Direction;
use ::chess_br::world::position::{Coord, Positional, Offset};
use ::chess_br::world::display::{print_board_pair, print_air};

fn print_scores(world: &World) {
    for (idx, player) in world.players_by_score().iter().enumerate() {
        let (score, name) = player.in_game_info();
        println!("{}.\t({})\t{}", idx + 1, score, name);
    }
}

fn main() {
    let mut world = spawn(
        Coord{x: 42, y: 16},
        4,
        &vec![String::from("Player 1"), String::from("Player 2")]
    );
    println!("{} flyers", world.flyers_count());
    for _ in 0..8 {
        world.do_tick();
    }
    world.request_action(Action::Drop(1));
    world.do_tick();
    print_air(world.fog.shape(), &world.players, 10);
    world.request_action(Action::Fly(1, Offset{x: -1, y: 0}));
    world.request_action(Action::Drop(2));
    world.do_tick();

    print_air(world.fog.shape(), &world.players, 10);
    print_air(world.fog.shape(), &world.players, 9);
    while world.airborne_count() > 0 {
        world.do_tick()
    }
    let p2 = world.player_positions(2);
    let king = p2[0];
    let to = king.1.translate_direction(Direction::North);

    world.request_action(Action::Move(2, p2[0].1.clone(), to));
    world.do_tick();
    print_board_pair(&world.pieces_map, &world.fog.zones);

    print_scores(&world);    
}
