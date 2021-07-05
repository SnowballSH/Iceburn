use std::io;
use std::str::FromStr;
use std::time::Duration;

use chess::{ChessMove, Color, Game};

use search::searcher::*;

use crate::eval::MATE_UPPER;
use crate::nnue::nnue_init;

mod eval;
mod nnue;
mod search;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut board = Game::new();

    let mut searcher = Searcher::new(99);

    loop {
        let line = read_line();

        let cmd = line.trim();

        if cmd == "" {
            continue;
        }

        match cmd.split(' ').next().unwrap() {
            "quit" => break,
            "uci" => {
                println!("id name Iceburn");
                println!("id author SnowballSH");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => board = Game::new(),

            "position" => {
                let params: Vec<&str> = cmd.split(' ').collect();
                let idx = cmd.find("moves");

                let moves = if let Some(x) = idx {
                    let p = &cmd[x..];
                    p.split(" ").collect::<Vec<&str>>()[1..].to_vec()
                } else {
                    vec![]
                };

                let fen = if params[1] == "fen" {
                    let fenpart = if let Some(x) = idx { &cmd[..x] } else { &cmd };
                    fenpart.split(" ").collect::<Vec<&str>>()[2..].join(" ")
                } else {
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
                };

                board = Game::from_str(&*fen).expect("Invalid board");

                for move_ in moves {
                    board.make_move(ChessMove::from_str(move_).unwrap());
                }
            }

            "eval" => {
                println!(
                    "\n----------\nClassical {}\nNNUE {}\nFinal {}\n----------\n",
                    eval::eval(board.current_position(), false),
                    eval::eval(board.current_position(), true),
                    (eval::eval(board.current_position(), true) as f32 * 0.8
                        + eval::eval(board.current_position(), false) as f32 * 0.2)
                        as i32
                )
            }

            "go" => {
                let params: Vec<&str> = cmd.split(' ').collect();

                let time_for_move = if params.len() == 3 {
                    let mil: u64 = params[2].parse().expect("Failed to parse movetime");
                    Duration::from_millis(mil)
                } else {
                    let time: u64 = if params.len() < 3 {
                        4_000
                    } else if board.side_to_move() == Color::Black {
                        params[4].parse().expect("Failed to parse btime")
                    } else {
                        params[2].parse().expect("Failed to parse wtime")
                    };

                    let increment: u64 = if params.len() < 9 {
                        0
                    } else if board.side_to_move() == Color::Black {
                        params[8].parse().expect("Failed to parse binc")
                    } else {
                        params[6].parse().expect("Failed to parse winc")
                    };

                    let nanos_for_move = (1_500.max(time / 60) + increment) * 1_000_000;

                    // println!("{} {} {:?} {}", time, increment, params, nanos_for_move);

                    Duration::new(
                        nanos_for_move as u64 / 1_000_000_000,
                        (nanos_for_move % 1_000_000_000) as u32,
                    )
                    .max(Duration::new(2, 0))
                    .min(Duration::new(50, 0))
                };

                searcher.iterative_deepening(board.current_position(), time_for_move);

                println!("bestmove {}", searcher.id_move.unwrap());
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}

fn main() {
    // init nnue
    nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
