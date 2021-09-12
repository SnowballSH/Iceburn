use std::io;

use crate::board::Board;
use crate::moves::Move;

mod board;
mod movegen;
mod moves;
mod nnue;
mod utils;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut board = Board::default();
    println!("\n{}\n", board.to_string());
    let moves = board.gen_moves();
    for m in moves {
        println!("{}", String::from_utf8(m.to_human()).unwrap());
    }

    let moves_list = ["g1f3", "d7d5", "g2g3", "c8h3", "f1h3", "d8d6", "e1g1"];
    for mm in moves_list {
        let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
        println!("made move {}", String::from_utf8(m.to_human()).unwrap());
        board.make_move(m);
        println!("\n{}\n", board.to_string());
    }

    let mut board = Board::default();
    println!("\n{}\n", board.to_string());

    let moves_list = [
        "f2f4", "g7g5", "b2b4", "g5f4", "g2g3", "f4g3", "f1g2", "g3h2", "d2d3", "h2g1q", "h1g1",
    ];
    for mm in moves_list {
        let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
        println!("made move {}", String::from_utf8(m.to_human()).unwrap());
        board.make_move(m);
        println!("\n{}\n", board.to_string());
    }
}

fn main() {
    // init nnue
    nnue::nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
