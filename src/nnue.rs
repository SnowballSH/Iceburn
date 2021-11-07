use lazy_static::lazy_static;
use libloading;

use crate::chess::{Chess, Color, Setup};

lazy_static! {
    static ref NNUE: libloading::Library = unsafe {
        libloading::Library::new("./nnue/libnnueprobe.dll").unwrap_or_else(|x| panic!("{}", x))
    };
    static ref NNUE_INIT: libloading::Symbol<'static, unsafe extern "C" fn(*const u8)> =
        unsafe { NNUE.get(b"nnue_init").unwrap() };
    static ref NNUE_EVAL_FEN: libloading::Symbol<'static, unsafe extern "C" fn(*const u8) -> i32> =
        unsafe { NNUE.get(b"nnue_evaluate_fen").unwrap() };
    static ref NNUE_EVAL_NORMAL: libloading::Symbol<'static, unsafe extern "C" fn(i32, *const i32, *const i32) -> i32> =
        unsafe { NNUE.get(b"nnue_evaluate").unwrap() };
}

pub fn nnue_init(eval_file: &str) {
    unsafe {
        NNUE_INIT(eval_file.as_ptr());
    }
}

pub fn nnue_eval_fen(fen: &str) -> i32 {
    unsafe { NNUE_EVAL_FEN(fen.as_ptr()) }
}

pub fn decode_board(board: &Chess) -> (i32, Vec<i32>, Vec<i32>) {
    let turn = if board.turn() == Color::White { 0 } else { 1 };
    let pcs = board.board().pieces();
    let mut pieces = Vec::with_capacity(pcs.len() + 1);
    pieces.push(1);
    pieces.push(7);
    let mut squares = Vec::with_capacity(pcs.len() + 1);
    squares.push(0);
    squares.push(0);
    for (sq, pc) in pcs {
        let side = if pc.color == Color::White { 0 } else { 1 };
        let pi = (7 - pc.role as i32) + 6 * side;
        let si = (7 - sq.rank() as i32) * 8 + (sq.file() as i32);
        if pi == 1 {
            squares[0] = si;
            continue;
        } else if pi == 7 {
            squares[1] = si;
            continue;
        }
        pieces.push(pi);
        squares.push(si);
    }
    pieces.push(0); // end of array
    squares.push(0);
    (turn, pieces, squares)
}

pub fn nnue_eval_normal(board: &Chess) -> i32 {
    let (turn, pieces, squares) = decode_board(board);
    unsafe { NNUE_EVAL_NORMAL(turn, pieces.as_ptr(), squares.as_ptr()) }
}
