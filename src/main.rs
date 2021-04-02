use std::io;
use std::time::Duration;

use shakmaty::{Chess, Color, Position, Setup};
use shakmaty::uci::Uci;
use rand::seq::SliceRandom;

fn main() {
    let mut stack: Vec<String> = vec![];
    let mut board = Chess::default();

    loop {
        let mut line = String::new();

        match stack.pop() {
            None => {
                io::stdin()
                    .read_line(&mut line)
                    .expect("Failed to read line");
            }
            Some(value) => {
                line = value;
            }
        };

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

                let mut nanos_for_move: i64 =
                    i64::from(time_difference + increment - 3_000) * 1_000_000;

                if nanos_for_move < (increment * 800_000).into() {
                    nanos_for_move = (increment * 800_000).into();
                }

                if nanos_for_move > 40_000_000 {
                    nanos_for_move = 40_000_000;
                }

                if nanos_for_move > 1_700_000_000 {
                    nanos_for_move -= 200_000_000 // Account for lag
                } else {
                    nanos_for_move = 500_000_000 // Minimum reasonable move time
                }

                let _ = Duration::new(
                    nanos_for_move as u64 / 1_000_000_000,
                    (nanos_for_move % 1_000_000_000) as u32,
                );

                let legal_moves = board.legal_moves();

                let m = &legal_moves.choose(&mut rand::thread_rng()).unwrap();

                println!("bestmove {}", Uci::from_standard(&m));
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}
