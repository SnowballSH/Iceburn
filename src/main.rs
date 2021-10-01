#![feature(core_intrinsics)]

use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc};
use std::{io};

use crate::board::Board;
use crate::moves::Move;
use crate::search::Search;
use crate::timeman::{TimeControl, Timer};
use crate::tt::TranspositionTable;
use crate::utils::u8_v_to_s;

pub mod board;
pub mod fen;
pub mod movegen;
pub mod moves;
pub mod nnue;
pub mod perft;
pub mod search;
pub mod timeman;
pub mod tt;
pub mod utils;
pub mod weight;
pub mod zobrist;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

fn uci() {
    let mut board = Board::default();
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
                board = Board::default();
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

                board = Board::default();

                for m in moves {
                    board.make_move(Move::from_uci(&board, Vec::from(m)).unwrap());
                }
            }
            "go" => {
                let mut searcher = Search::new(
                    Timer::new(&board, TimeControl::FixedMillis(10000), stop_search.clone()),
                    &mut tt,
                );
                let best_move;
                let best_score;
                let res = searcher.go(&mut board);
                best_move = res.0;
                best_score = res.1;
                println!("info score cp {}", best_score);
                println!("bestmove {}", u8_v_to_s(best_move.to_uci()));
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
