use std::io;

use chess::{Game, ChessMove, Color};

mod searcher;
mod eval;

use searcher::*;
use std::str::FromStr;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut board = Game::new();

    let mut searcher = Searcher::default();

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
            "ucinewgame" => board = Game::new(),

            "position" => {
                board = Game::new();
                let moves: Vec<&str> = cmd.split(' ').collect();
                if moves.len() == 2 && moves[1] != "startpos" {
                    panic!("Invalid syntax");
                } else if moves.len() > 2
                    && (moves[0] != "position" || moves[1] != "startpos" || moves[2] != "moves")
                {
                    panic!("Invalid syntax");
                }
                for move_ in moves.iter().skip(3) {
                    board.make_move(ChessMove::from_str(move_).unwrap());
                }
            }

            "go" => {
                // Command format is going to be:
                // go wtime 391360 btime 321390 winc 8000 binc 8000

                /*
                let infos: Vec<&str> = cmd.split(' ').collect();

                let time_difference: i32 = if infos.len() < 9 {
                    4_000
                } else if board.turn() == Color::Black {
                    infos[4].parse::<i32>().expect("Failed to btime")
                        - infos[2].parse::<i32>().expect("Failed to parse wtime")
                } else {
                    infos[2].parse::<i32>().expect("Failed to parse wtime")
                        - infos[4].parse::<i32>().expect("Failed to parse btime")
                };

                let increment: i32 = if infos.len() < 9 {
                    0
                } else if board.turn() == Color::Black {
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

                let time_for_move = Duration::new(
                    15,
                    (nanos_for_move % 1_000_000_000) as u32,
                );
                 */

                let m = searcher.best_move(board.clone(), 4, board.side_to_move());

                println!("bestmove {}", m.unwrap().to_string());
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}

fn test_() {
    let mut game = Game::new();
    let mut searcher1 = Searcher::default();
    let mut searcher2 = Searcher::default();
    for _ in 0..12 {
        if game.clone().result().is_some() {
            break;
        }
        let best_move_ = searcher1.best_move(game.clone(), 5, Color::White);
        if let Some(m) = best_move_ {
            game.make_move(m);
            println!("{}", m.to_string());
        } else {
            break;
        }

        if game.clone().result().is_some() {
            break;
        }
        let best_move_ = searcher2.best_move(game.clone(), 5, Color::Black);
        if let Some(m) = best_move_ {
            game.make_move(m);
            println!("{}", m.to_string());
        } else {
            break;
        }
    }
}

fn main() {
    uci();
}
