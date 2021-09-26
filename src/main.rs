#![feature(core_intrinsics)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{io, thread};

use crate::board::Board;
use crate::moves::Move;
use crate::search::{Searcher, TimeMan};
use crate::tt::TranspositionTable;
use std::process::exit;

pub mod board;
pub mod fen;
pub mod movegen;
pub mod moves;
pub mod nnue;
pub mod perft;
pub mod search;
pub mod tt;
pub mod utils;
pub mod zobrist;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

fn uci() {
    let searcher = Arc::new(Mutex::new(Searcher::default()));
    let mut board = Board::default();
    let mut tt = Arc::new(Mutex::new(TranspositionTable::default()));

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
            "stop" => {
                *searcher.lock().unwrap().stop_search.lock().unwrap() = true;
            }
            "isready" => println!("readyok"),
            "ucinewgame" => {
                searcher.lock().unwrap().clear();
                board = Board::default();
                tt = Arc::new(Mutex::new(TranspositionTable::default()));
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

                board = Board::default();

                for m in moves {
                    board.make_move(Move::from_uci(&board, Vec::from(m)).unwrap());
                }
            }
            "go" => {
                let mut max_depth = 100;

                if args == "" {
                    searcher.lock().unwrap().timeman = None;
                } else if args.starts_with("movetime") {
                    let tms: u64 = args.split_whitespace().collect::<Vec<&str>>()[1]
                        .parse()
                        .unwrap();
                    searcher.lock().unwrap().timeman = Some(TimeMan {
                        start: Instant::now(),
                        amount: Duration::from_millis(tms - 50),
                    });
                } else {
                    max_depth = 6;
                    searcher.lock().unwrap().timeman = Some(TimeMan {
                        start: Instant::now(),
                        amount: Duration::from_millis(4000),
                    });
                }

                let mut bdc = board.clone();
                let ttc = tt.clone();
                let ssc = searcher.clone();
                thread::spawn(move || {
                    ssc.lock().unwrap().search(&mut bdc, ttc, max_depth);
                });
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
