#![feature(core_intrinsics)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::chess::fen::Fen;
use crate::chess::uci::Uci;
use crate::chess::{uci, CastlingMode, Chess, Color, FromSetup, Position, Setup};
use crate::search::Search;
use crate::time::calc_time;
use crate::timeman::{TimeControl, Timer};
use crate::tt::TranspositionTable;

pub mod chess;
pub mod nnue;
pub mod ordering;
pub mod perft;
pub mod search;
pub mod time;
pub mod timeman;
pub mod tt;
pub mod utils;
pub mod weight;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

fn uci() {
    let mut board = Chess::default();
    let mut move_table = Vec::with_capacity(100);
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
                if args.starts_with("fen") {
                    let fenpart = &args[4..];
                    board = Chess::from_setup(
                        &Fen::from_ascii(fenpart.as_bytes()).unwrap(),
                        CastlingMode::Standard,
                    )
                    .unwrap();
                    move_table.clear();
                    continue;
                }

                if args.starts_with("startpos") {
                    move_table.clear();
                    board = Chess::default();
                    if args == "startpos" {
                        continue;
                    }
                }

                let idx = args.find("moves");

                let moves = if let Some(x) = idx {
                    let p = &args[x..];
                    p.split(" ").collect::<Vec<&str>>()[1..].to_vec()
                } else {
                    vec![]
                };

                for m in moves {
                    board.play_unchecked(&uci::Uci::from_str(m).unwrap().to_move(&board).unwrap());
                    let mut hasher = DefaultHasher::new();
                    board.board().hash(&mut hasher);
                    let hs = hasher.finish();
                    move_table.push(hs);
                }
            }
            "go" => {
                if args == "perft" {
                    let mut p = perft::Perft::new();
                    p.test(5);
                    continue;
                }

                let mut arg_slice: Vec<&str> = args.split(" ").collect();
                while arg_slice.len() < 10 {
                    arg_slice.push("");
                }

                let time_control: TimeControl;

                if arg_slice[0] == "movetime" {
                    time_control = TimeControl::FixedMillis(arg_slice[1].parse().unwrap());
                } else if arg_slice[0] == "wtime" && arg_slice[2] == "btime" {
                    let wtime: usize = arg_slice[1].parse().unwrap();
                    let btime: usize = arg_slice[3].parse().unwrap();
                    let mut winc: usize = 0;
                    let mut binc: usize = 0;
                    let length = board.fullmoves().get() as f64;
                    let expected_game_length: f64 = 85.0;
                    let mut moves_to_go: f64 = (expected_game_length - length).max(1.5);
                    if arg_slice[4] == "winc" && arg_slice[6] == "binc" {
                        winc = arg_slice[5].parse().unwrap();
                        binc = arg_slice[7].parse().unwrap();
                        if arg_slice[8] == "movestogo" {
                            moves_to_go = arg_slice[9].parse().unwrap();
                        }
                    } else {
                        if arg_slice[4] == "movestogo" {
                            moves_to_go = arg_slice[5].parse().unwrap();
                        }
                    }

                    let left = if board.turn() == Color::White {
                        wtime as f64 + winc as f64 * moves_to_go
                    } else {
                        btime as f64 + binc as f64 * moves_to_go
                    };

                    let mut our_time = left / moves_to_go;

                    our_time = calc_time(length, our_time);

                    time_control = TimeControl::FixedMillis(
                        our_time
                            .max(1.0)
                            // must be in time
                            .min(left - 1.0) as u64,
                    );
                } else {
                    time_control = TimeControl::FixedMillis(2000);
                }

                let mut searcher = Search::new(
                    Timer::new(&board, time_control, stop_search.clone()),
                    &mut tt,
                );
                searcher.move_table = move_table.clone();
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
