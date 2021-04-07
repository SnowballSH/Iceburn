use std::io;

use shakmaty::{Chess, Color, Position, Setup};
use shakmaty::uci::Uci;

use crate::engine::best_move;

mod engine;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn main() {
    let mut board = Chess::default();

    loop {
        let line = read_line();

        let cmd = line.trim();

        if cmd == "" {
            continue;
        }

        match cmd.split(' ').next().unwrap() {
            "quit" => break,
            "uci" => println!("uciok"),
            "isready" => println!("readyok"),
            "ucinewgame" => board = Chess::default(),

            "position" => {
                board = Chess::default();
                let moves: Vec<&str> = cmd.split(' ').collect();
                if moves.len() == 2 && moves[1] != "startpos" {
                    panic!("Invalid syntax");
                } else if moves.len() > 2
                    && (moves[0] != "position" || moves[1] != "startpos" || moves[2] != "moves")
                {
                    panic!("Invalid syntax");
                }
                for move_ in moves.iter().skip(3) {
                    let uci: Uci = move_.parse().unwrap();
                    let m = uci.to_move(&board).unwrap();
                    board.play_unchecked(&m);
                }
            }

            "go" => {
                // Command format is going to be:
                // go wtime 391360 btime 321390 winc 8000 binc 8000

                let infos: Vec<&str> = cmd.split(' ').collect();

                let is_black = board.turn() == Color::Black;

                let time_difference: i32 = if infos.len() < 9 {
                    3_000
                } else if is_black {
                    infos[4].parse::<i32>().expect("Failed to btime")
                        - infos[2].parse::<i32>().expect("Failed to parse wtime")
                } else {
                    infos[2].parse::<i32>().expect("Failed to parse wtime")
                        - infos[4].parse::<i32>().expect("Failed to parse btime")
                };

                let increment: i32 = if infos.len() < 9 {
                    0
                } else if is_black {
                    infos[8].parse::<i32>().expect("Failed to parse binc")
                } else {
                    infos[6].parse::<i32>().expect("Failed to parse winc")
                };

                let _: i64 =
                    i64::from(time_difference + increment - 2_000) * 1_000_000;

                let pair = best_move(board.clone(), 7, is_black);

                println!("bestmove {}", pair);
                //println!("score {}", pair.1)
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}
