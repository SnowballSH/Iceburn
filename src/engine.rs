use std::cmp::{max, min};

use shakmaty::{Chess, Color, File, Move, Outcome, Position, Rank, Role, Setup, Square};

const MATE_LOWER: i16 = i16::MIN + 2;
const MATE_UPPER: i16 = i16::MAX - 2;

const BASE_ALPHA: i16 = i16::MIN + 1;
const BASE_BETA: i16 = i16::MAX - 1;

const PAWN_MAP: [i16; 64] = [0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5, 5, 10, 25, 25, 10, 5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, -5, -10, 0, 0, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0
];

fn get(square: Square, is_black: bool) -> usize {
    let s = match square.file() {
        File::A => 0,
        File::B => 1,
        File::C => 2,
        File::D => 3,
        File::E => 4,
        File::F => 5,
        File::G => 6,
        File::H => 7,
    } + match square.rank() {
        Rank::First => 7,
        Rank::Second => 6,
        Rank::Third => 5,
        Rank::Fourth => 4,
        Rank::Fifth => 3,
        Rank::Sixth => 2,
        Rank::Seventh => 1,
        Rank::Eighth => 0,
    } * 8;
    if is_black { 64 - 8 * (s >> 3) + s % 8 - 8 } else { s }
}

pub fn best_move(board: Chess, depth: u8, is_black: bool) -> Option<Move> {
    minimax_root(depth, board, is_black)
}

fn eval_board(board: Chess) -> i16 {
    let mut score: i16 = 0;

    match board.outcome() {
        Some(o) => {
            score = match o {
                Outcome::Draw => 0,
                Outcome::Decisive {
                    winner: Color::White
                } => MATE_LOWER,
                Outcome::Decisive {
                    winner: Color::Black
                } => MATE_UPPER,
            };
        }
        None => {
            for y in board.board().pieces() {
                let piece = y.1;
                let multiplier = match piece.color {
                    Color::White => 1,
                    Color::Black => -1,
                };
                let pib = multiplier == -1;

                score += match piece.role {
                    Role::King => 20000,
                    Role::Queen => 900,
                    Role::Rook => 500,
                    Role::Bishop => 320,
                    Role::Knight => 320,
                    Role::Pawn => {
                        let s = 100 + PAWN_MAP[get(y.0, pib)];
                        s
                    }
                } * multiplier;
            }
        }
    }

    score
}

fn minimax_root(depth: u8, board: Chess, is_black: bool) -> Option<Move> {
    let moves = board.legal_moves();
    let mut best_m = None;
    let mut best_v = MATE_LOWER;

    for m in moves {
        let mut b = board.clone();
        b.play_unchecked(&m);
        let score = minimax(depth - 1, b, !is_black, is_black, BASE_ALPHA, BASE_BETA);
        //println!("{}: {}", m.to_string(), score);
        if score > best_v {
            best_v = score;
            best_m = Some(m);
        }
    }

    best_m
}

fn minimax(depth: u8, board: Chess, is_black: bool, original: bool,
           mut alpha: i16, mut beta: i16) -> i16 {
    let maximizing = is_black == original;

    if depth == 0 || board.is_game_over() {
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
