use std::char;

fn encode_ch(val: u16) -> String {
    if val > 9 {
    let c = char::from_u32((val + 55) as u32);
    match c {
        None => ' '.to_string(),
        Some(s) => s.to_string()
    }
    } else {
        val.to_string()
    }
}

pub fn print_board(board: &Vec<Vec<u16>>) {
    println!("");
    for row in board.iter() {
        let out = row 
            .into_iter()
            .map(|i| encode_ch(*i))
            .collect::<String>();

        println!("{}", out);
    }
}

pub fn print_board_pair(first: &Vec<Vec<u16>>, second: &Vec<Vec<u16>>) {
    for (first_row, second_row) in first.iter().zip(second.iter()) {
        let first_out = first_row
            .into_iter()
            .map(|i| encode_ch(*i))
            .collect::<String>();
        let second_out = second_row
            .into_iter()
            .map(|i| encode_ch(*i))
            .collect::<String>();
        println!("{} {}", first_out, second_out);
    }
}