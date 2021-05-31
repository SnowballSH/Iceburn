use std::io;
use std::str::FromStr;
use std::time::Duration;

use chess::{ChessMove, Color, Game};

use searcher::*;
use crate::eval::MATE_UPPER;

mod searcher;
mod eval;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut board = Game::new();

    let mut searcher = Searcher::default();

    let mut amount_moves: usize = 0;

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
                    let fenpart = if let Some(x) = idx {
                        &cmd[..x]
                    } else {
                        &cmd
                    };
                    amount_moves = 10;
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
                let m = searcher.search(board.current_position(), Duration::new(18, 0), amount_moves < 4);

                println!("info depth {} score cp {} time {:?} nodes {} pv",
                         m.0.2,
                         m.0.1 * if board.side_to_move() == Color::Black { -1 } else { 1 },
                         m.2,
                         m.1,
                );

                amount_moves += 1;

                if m.0.1 == -MATE_UPPER {
                    println!("resign");
                }

                println!("bestmove {}", m.0.0.to_string());
            }

            _ => println!("Unknown command: {}", cmd),
        };
    }
}

fn main() {
    uci();
}
