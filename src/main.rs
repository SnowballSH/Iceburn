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
    let moves = board.gen_moves();
    for m in &moves {
        println!("{}", String::from_utf8(m.to_human()).unwrap());
    }
    let m = moves.last().unwrap();
    println!("made move {}", String::from_utf8(m.to_human()).unwrap());
    board.make_move(*m);
    let moves = board.gen_moves();
    for m in &moves {
        println!("{}", String::from_utf8(m.to_human()).unwrap());
    }
}

fn main() {
    // init nnue
    nnue::nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
