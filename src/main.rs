use crate::board::Board;
use crate::utils::u8_v_to_s;
use std::io;

pub mod board;
pub mod fen;
pub mod movegen;
pub mod moves;
pub mod nnue;
pub mod perft;
pub mod utils;
pub mod zobrist;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut board = Board::default();

    let moves = board.gen_moves();

    for m in moves {
        board.make_move(m);

        println!(
            "MOVE {} = {}",
            u8_v_to_s(m.to_human()),
            nnue::nnue_eval_fen(&*board.fen())
        );

        board.undo_move();
    }
}

fn main() {
    // init nnue
    nnue::nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
