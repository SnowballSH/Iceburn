#![feature(core_intrinsics)]

use std::io;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::search::Search;
use crate::timeman::{TimeControl, Timer};
use crate::tt::TranspositionTable;
use shakmaty::uci::Uci;
use shakmaty::{uci, Chess, Position};
use std::str::FromStr;

pub mod nnue;
pub mod perft;
pub mod search;
pub mod timeman;
pub mod tt;
pub mod utils;
pub mod weight;
pub mod ordering;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

fn uci() {
    let mut board = Chess::default();
    let mut tt = TranspositionTable::default();
    let stop_search = Arc::new(AtomicBool::new(false));

    loop {
        let line = read_line();
        let cmd_slice = line.trim();
        let (token, args) = if let Some(idx) = cmd_slice.find(char::is_whitespace) {
            cmd_slice.split_at(idx)
        } else {
            (cmd_slice, "")
        };

        let args = args.trim();

        match token {
            "quit" => {
                exit(0);
            }
            "stop" => {}
            "isready" => println!("readyok"),
            "ucinewgame" => {
                board = Chess::default();
                tt = TranspositionTable::default();
            }
            "uci" => {
                println!("id name Iceburn");
                println!("id author SnowballSH");
                println!("uciok");
            }
            "position" => {
                let idx = args.find("moves");

                let moves = if let Some(x) = idx {
                    let p = &args[x..];
                    p.split(" ").collect::<Vec<&str>>()[1..].to_vec()
                } else {
                    vec![]
                };

                board = Chess::default();

                for m in moves {
                    board.play_unchecked(&uci::Uci::from_str(m).unwrap().to_move(&board).unwrap());
                }
            }
            "go" => {
                if args == "perft" {
                    let mut p = perft::Perft::new();
                    p.test(5);
                    continue;
                }

                let mut searcher = Search::new(
                    Timer::new(&board, TimeControl::FixedMillis(5000), stop_search.clone()),
                    &mut tt,
                );
                let res = searcher.mtdf(&mut board);
                let best_move = res.0;
                let best_score = res.1;
                println!("info score cp {}", best_score);
                println!("bestmove {}", Uci::from_standard(&best_move));
                // tt.clear();
            }
            _ => {
                println!("No such command")
            }
        }
    }
}

fn main() {
    // init nnue
    nnue::nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
