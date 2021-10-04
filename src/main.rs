#![feature(core_intrinsics)]

use std::io;
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::chess::uci::Uci;
use crate::chess::{uci, Chess, Color, Position, Setup};
use crate::search::Search;
use crate::timeman::{TimeControl, Timer};
use crate::tt::TranspositionTable;

pub mod chess;
pub mod nnue;
pub mod ordering;
pub mod perft;
pub mod search;
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

                let arg_slice: Vec<&str> = args.split(" ").collect();

                let time_control: TimeControl;

                if arg_slice[0] == "movetime" {
                    time_control = TimeControl::FixedMillis(arg_slice[1].parse().unwrap());
                } else if arg_slice[0] == "wtime" && arg_slice[2] == "btime" {
                    let wtime: usize = arg_slice[1].parse().unwrap();
                    let btime: usize = arg_slice[3].parse().unwrap();
                    let mut winc: usize = 0;
                    let mut binc: usize = 0;
                    let length = board.fullmoves().get() as f64;
                    let expected_game_length: f64 = 60.0;
                    let mut moves_to_go: f64 = expected_game_length - length;
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

                    let our_time = if board.turn() == Color::White {
                        wtime as f64 + winc as f64 * moves_to_go
                    } else {
                        btime as f64 + binc as f64 * moves_to_go
                    } / moves_to_go
                        * 0.98;

                    time_control = TimeControl::FixedMillis(our_time as u64);
                } else {
                    time_control = TimeControl::FixedMillis(2000);
                }

                let mut searcher = Search::new(
                    Timer::new(&board, time_control, stop_search.clone()),
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
