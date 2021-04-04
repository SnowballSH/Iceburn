use std::cmp::max;

use shakmaty::{Chess, Color, File, Outcome, Position, Rank, Role, Setup, Square, uci::Uci};

pub static SQUARE_MAP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15,
    0, 1, 2, 3, 4, 5, 6, 7
];

fn rel_loc(square: Square, color: Color) -> usize {
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
        Rank::First => 0,
        Rank::Second => 1,
        Rank::Third => 2,
        Rank::Fourth => 3,
        Rank::Fifth => 4,
        Rank::Sixth => 5,
        Rank::Seventh => 6,
        Rank::Eighth => 7,
    } * 8;
    if color == Color::Black { s } else { SQUARE_MAP[s] }
}

pub static PAWN_SQUARE: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5, 5, 10, 25, 25, 10, 5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, -5, -10, 0, 0, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0,
];

pub static KNIGHT_SQUARE: [i8; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 0, 0, 0, -20, -40,
    -30, 0, 10, 15, 15, 10, 0, -30,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 0, 15, 20, 20, 15, 0, -30,
    -30, 5, 10, 15, 15, 10, 5, -30,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

pub static BISHOP_SQUARE: [i8; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10,
    -10, 5, 5, 10, 10, 5, 5, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

pub static ROOK_SQUARE: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, 10, 10, 10, 10, 5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    0, 0, 0, 5, 5, 0, 0, 0,
];

pub static QUEEN_SQUARE: [i8; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    0, 0, 5, 5, 5, 5, 0, -5,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

pub static KING_SQUARE: [i8; 64] = [
    -3, -4, -4, -5, -5, -4, -4, -3,
    -3, -4, -4, -5, -5, -4, -4, -3,
    -3, -4, -4, -5, -5, -4, -4, -3,
    -3, -4, -4, -5, -5, -4, -4, -3,
    -2, -3, -3, -4, -4, -3, -3, -2,
    -1, -2, -2, -2, -2, -2, -2, -1,
    2, 2, 0, 0, 0, 0, 2, 2,
    2, 3, 1, 0, 0, 1, 3, 2
];

pub fn eval_board(board: Chess) -> i16 {
    let mut score: i16 = 0;

    match board.outcome() {
        Some(o) => {
            score = match o {
                Outcome::Draw => 0,
                Outcome::Decisive {
                    winner: Color::White
                } => i16::MAX,
                Outcome::Decisive {
                    winner: Color::Black
                } => -i16::MAX,
            };
        }
        None => {
            for x in board.board().white() {
                let piece = board.board().piece_at(x).unwrap();
                let multiplier = match piece.color {
                    Color::White => 1,
                    Color::Black => -1,
                };
                score += match piece.role {
                    Role::King => 900 + i16::from(KING_SQUARE[rel_loc(x, piece.color)]) / 10,
                    Role::Queen => 90 + i16::from(QUEEN_SQUARE[rel_loc(x, piece.color)]) / 10,
                    Role::Rook => 50 + i16::from(ROOK_SQUARE[rel_loc(x, piece.color)]) / 10,
                    Role::Bishop => 30 + i16::from(BISHOP_SQUARE[rel_loc(x, piece.color)]) / 10,
                    Role::Knight => 30 + i16::from(KNIGHT_SQUARE[rel_loc(x, piece.color)]) / 10,
                    Role::Pawn => 10 + i16::from(PAWN_SQUARE[rel_loc(x, piece.color)]) / 10,
                } * multiplier;
            }
        }
    }

    score
}

pub fn eval_board_color(board: Chess, black: bool) -> i16 {
    let res = eval_board(board);
    if black { -res } else { res }
}

pub fn negamax(board: Chess, depth: u8, mut alpha: i16, beta: i16, black: bool) -> i16 {
    if depth == 0 || board.is_game_over() {
        return eval_board_color(board, black);
    };

    let mut best = -i16::MAX;

    for move_ in board.legal_moves() {
        let mut b = board.clone();
        b.play_unchecked(&move_);

        best = max(best, -negamax(b, depth - 1, -beta, -alpha, !black));

        alpha = max(alpha, best);
        if alpha >= beta {
            break;
        }
    }

    best
}

pub fn choose_move(board: Chess, depth: u8, black: bool) -> String {
    let mut best = -i16::MAX;
    let mut candidates = None;

    for move_ in board.legal_moves() {
        let mut b = board.clone();
        b.play_unchecked(&move_);

        let value = -negamax(b, depth, -i16::MAX, i16::MAX, !black);

        if value == best {
            candidates = Some(move_);
        } else if value > best {
            candidates = Some(move_);
            best = value;
        }
    }

    let res = candidates.unwrap();
    Uci::from_standard(&res).to_string()
}
