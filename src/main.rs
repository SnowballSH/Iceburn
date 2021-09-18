use std::io;

use crate::perft::Perft;

mod board;
mod movegen;
mod moves;
mod nnue;
mod perft;
mod utils;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn uci() {
    let mut perf = Perft::new();
    perf.test(5);
}

fn main() {
    // init nnue
    nnue::nnue_init("./nnue/nn-62ef826d1a6d.nnue");
    uci();
}
