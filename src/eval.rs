use crate::nnue::nnue_eval_fen;
use chess::{get_bishop_moves, get_file, Board, BoardStatus, Color, Piece, Square, ALL_PIECES};

pub const MATE_UPPER: i32 = 32_000 + 8 * QUEEN_VALUE;
pub const MATE_LOWER: i32 = 32_000 - 8 * QUEEN_VALUE;

pub const QUEEN_VALUE: i32 = 980;

// @formatter:off
const PAWN_MAP: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

// @formatter:off
const KNIGHT_MAP: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

// @formatter:off
const BISHOP_MAP: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

// @formatter:off
const ROOK_MAP: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

// @formatter:off
const QUEEN_MAP: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

// @formatter:off
const KING_MAP_MIDDLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, -5, 0, 0, -5, 20, 20, 20, 35, 15, 0,
    0, 10, 60, 20,
];

// @formatter:off
const KING_MAP_END: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

fn get(square: Square, is_black: bool) -> usize {
    let s = square.get_file().to_index() + 56 - square.get_rank().to_index() * 8;
    if is_black {
        s ^ 56
    } else {
        s
    }
}

fn to_val(p: Square, board: Board) -> i32 {
    let pib = board.color_on(p).unwrap() == Color::Black;
    let k = get(p, pib);
    match board.piece_on(p).unwrap() {
        Piece::Pawn => 100 + PAWN_MAP[k],
        Piece::Knight => 320 + KNIGHT_MAP[k],
        Piece::Bishop => 340 + BISHOP_MAP[k],
        Piece::Rook => 550 + ROOK_MAP[k],
        Piece::Queen => QUEEN_VALUE + QUEEN_MAP[k],
        Piece::King => {
            32000
                + (if board.combined().popcnt() < 9 {
                    KING_MAP_END
                } else {
                    KING_MAP_MIDDLE
                })[k]
        }
    }
}

fn inner_eval(board: Board) -> i32 {
    let mut score = 0;
    let b = board;

    for piece_type in ALL_PIECES.iter() {
        let piece_bb = b.pieces(*piece_type);
        let white = b.color_combined(Color::White);
        let black = b.color_combined(Color::Black);

        let it = piece_bb & white;
        for p in it {
            score += to_val(p, b);
        }

        let it = piece_bb & black;
        for p in it {
            score -= to_val(p, b);
        }
    }

    for x in *board.pinned() {
        let p = board.piece_on(x).unwrap();
        score -= match p {
            Piece::Pawn => 6,
            Piece::Knight => 60,
            Piece::Bishop => 60,
            Piece::Rook => 80,
            Piece::Queen => 150,
            Piece::King => 0,
        }
    }
    score -= board.checkers().popcnt() as i32 * 14;

    score += passed_pawns(board);
    score += supported_pawns(board);
    score += bishop_activity(board);

    score
}

#[inline]
fn passed_pawns(board: Board) -> i32 {
    let mut score = 0;
    let pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
    for p in pawns {
        if (board.pieces(Piece::Pawn) & board.color_combined(Color::Black) & get_file(p.get_file()))
            .popcnt()
            == 0
        {
            score += 30
        }
    }

    let pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);
    for p in pawns {
        if (board.pieces(Piece::Pawn) & board.color_combined(Color::White) & get_file(p.get_file()))
            .popcnt()
            == 0
        {
            score -= 30
        }
    }
    score
}

#[inline]
fn supported_pawns(board: Board) -> i32 {
    let mut score = 0;
    let pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
    for p in pawns {
        if let Some(a) = p.left() {
            if let Some(b) = a.down() {
                if let Some(x) = board.piece_on(b) {
                    if x == Piece::Pawn && board.color_on(b).unwrap() == Color::White {
                        score += 10;
                    }
                }
            }
        }
        if let Some(a) = p.right() {
            if let Some(b) = a.down() {
                if let Some(x) = board.piece_on(b) {
                    if x == Piece::Pawn && board.color_on(b).unwrap() == Color::White {
                        score += 10;
                    }
                }
            }
        }
    }

    let pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);
    for p in pawns {
        if let Some(a) = p.left() {
            if let Some(b) = a.up() {
                if let Some(x) = board.piece_on(b) {
                    if x == Piece::Pawn && board.color_on(b).unwrap() == Color::Black {
                        score -= 10;
                    }
                }
            }
        }
        if let Some(a) = p.right() {
            if let Some(b) = a.up() {
                if let Some(x) = board.piece_on(b) {
                    if x == Piece::Pawn && board.color_on(b).unwrap() == Color::Black {
                        score -= 10;
                    }
                }
            }
        }
    }
    score
}

#[inline]
fn bishop_activity(board: Board) -> i32 {
    let mut score = 0;
    let bishops = board.pieces(Piece::Bishop) & board.color_combined(Color::White);
    for b in bishops {
        let b_moves = get_bishop_moves(b, *board.combined());
        score += b_moves.popcnt() as i32 * 2;
    }
    let bishops = board.pieces(Piece::Bishop) & board.color_combined(Color::Black);
    for b in bishops {
        let b_moves = get_bishop_moves(b, *board.combined());
        score -= b_moves.popcnt() as i32 * 2;
    }

    score
}

pub fn eval(board: Board, nnue: bool) -> i32 {
    (match board.clone().status() {
        BoardStatus::Stalemate => 0,
        BoardStatus::Ongoing => {
            if nnue {
                (nnue_eval_fen(board.to_string().as_str()) as f32 * 0.7).round() as i32
                    * if board.side_to_move() == Color::Black {
                        -1
                    } else {
                        1
                    }
            } else {
                inner_eval(board)
            }
        }
        BoardStatus::Checkmate => {
            if board.side_to_move() == Color::Black {
                MATE_UPPER
            } else {
                MATE_LOWER
            }
        }
    }) * if board.side_to_move() == Color::Black {
        -1
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chess::Board;

    use crate::eval::{bishop_activity, passed_pawns, supported_pawns};

    #[test]
    fn pawns_test() {
        let board = Board::from_str("q4k2/8/p1p5/3p4/5P1P/P3P3/2Q5/5K2 b - - 0 1").unwrap();
        assert_eq!(passed_pawns(board), 30);

        let board = Board::from_str("1k6/1pp1p3/p2p4/2p5/4PP2/1P4P1/P1P5/1K6 w - - 0 1").unwrap();
        assert_eq!(supported_pawns(board), -10);

        let board = Board::from_str("1k6/1pp5/p3b3/5b2/4N3/1P6/P1P3B1/1K6 w - - 0 1").unwrap();
        assert_eq!(bishop_activity(board), -18);
    }
}
