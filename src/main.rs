use std::io;
use std::str::FromStr;
use std::time::Duration;

use chess::{ChessMove, Color, Game};

use searcher::*;

use crate::eval::MATE_UPPER;

mod eval;
mod searcher;

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

            "go" => {
                let params: Vec<&str> = cmd.split(' ').collect();

                let time: u64 = if params.len() < 9 {
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

                let nanos_for_move = (1_500.max(time / 50) + increment) * 1_000_000;

                let time_for_move = Duration::new(
                    nanos_for_move as u64 / 1_000_000_000,
                    (nanos_for_move % 1_000_000_000) as u32,
                )
                .max(Duration::new(2, 0))
                .min(Duration::new(22, 0));

                println!("{:?} {} {}", time_for_move, time, increment);

                let m = searcher.search(board.current_position(), time_for_move, 50);

                println!(
                    "info depth {} score cp {} time {} nodes {} pv",
                    m.0 .2,
                    (m.0 .1
                        * if board.side_to_move() == Color::Black {
                            -1
                        } else {
                            1
                        }) as f32
                        / 100.0,
                    m.2.as_millis(),
                    m.1,
                );

                if m.0 .1 == -MATE_UPPER {
                    println!("resign");
                }

                println!("bestmove {}", m.0 .0.to_string());
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}

fn main() {
    uci();
}
