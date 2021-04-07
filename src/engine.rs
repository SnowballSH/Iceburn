use std::cmp::{max, min};

use futures::executor::block_on;
use futures::future::join_all;
use shakmaty::{Chess, Move, Position};
use shakmaty::uci::Uci;

use crate::eval::*;

const BASE_ALPHA: i16 = i16::MIN + 1;
const BASE_BETA: i16 = i16::MAX - 1;


pub fn best_move(board: Chess, depth: u8, is_black: bool) -> String {
    match block_on(minimax_root(depth, board, is_black)) {
        Some(x) => Uci::from_standard(&x).to_string(),
        None => "NULL".to_string(),
    }
}

async fn search(b: Chess, depth: u8, is_black: bool) -> i16 {
    minimax(depth - 1, b, !is_black, is_black, BASE_ALPHA, BASE_BETA)
}

async fn minimax_root(depth: u8, board: Chess, is_black: bool) -> Option<Move> {
    let moves = board.legal_moves();
    let mut best_m = None;
    let mut best_v = MATE_LOWER;
    let mut fut = Vec::with_capacity(moves.len());

    for m in &moves {
        let mut b = board.clone();
        b.play_unchecked(&m);
        let score = search(b, depth, is_black);
        fut.push(score);
    }

    let mut i = 0;
    let fs = join_all(fut).await;
    for m in moves {
        let score = fs[i];
        if score > best_v {
            best_v = score;
            best_m = Some(m);
        }
        i += 1;
    }

    best_m
}

fn minimax(depth: u8, board: Chess, is_black: bool, original: bool,
           mut alpha: i16, mut beta: i16) -> i16 {
    let maximizing = is_black == original;

    if depth == 0 || board.is_game_over() || board.legal_moves().len() == 0 {
        return eval_board(board) * if is_black { 1 } else { -1 };
    }

    let moves = board.legal_moves();

    if maximizing {
        let mut best = MATE_LOWER;
        for m in moves {
            let mut b = board.clone();
            b.play_unchecked(&m);
            let score = minimax(depth - 1, b, !is_black, original, alpha, beta);
            best = max(best, score);
            alpha = max(best, alpha);
            if beta <= alpha {
                return score;
            }
        }
        best
    } else {
        let mut best = MATE_UPPER;
        for m in moves {
            let mut b = board.clone();
            b.play_unchecked(&m);
            let score = minimax(depth - 1, b, !is_black, original, alpha, beta);
            best = min(best, score);
            beta = min(best, beta);
            if beta <= alpha {
                return score;
            }
        }
        best
    }
}
