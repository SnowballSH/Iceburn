use chess::{get_bishop_moves, get_file, Board, BoardStatus, Color, Piece, Square, ALL_PIECES};

pub const MATE_UPPER: i32 = 32_000 + 8 * QUEEN_VALUE;
pub const MATE_LOWER: i32 = 32_000 - 8 * QUEEN_VALUE;

pub const QUEEN_VALUE: i32 = 980;

// @formatter:off
const PAWN_MAP: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 15, 31, 20, 14, 23, 11, 37, 24, -1, -3, 15, 26, 1, 10, -7, -9, 8, -1,
    -5, 13, 24, 11, -10, 3, -9, -18, 8, 32, 43, 25, -4, -16, -9, -13, -40, 22, 26, -40, 1, -22, 2,
    0, 15, 3, 11, 22, 11, -1, 0, 0, 0, 0, 0, 0, 0, 0,
];

// @formatter:off
const KNIGHT_MAP: [i32; 64] = [
    -90, -40, -30, -30, -30, -30, -40, -90, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 11, 16, 16, 12,
    5, -30, -40, -20, 0, 4, 5, 0, -20, -40, -90, -40, -30, -30, -30, -30, -40, -90,
];

// @formatter:off
const BISHOP_MAP: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

// @formatter:off
// https://github.com/Recursing/sunfish_rs/blob/master/src/pieces.rs#L173 because I don't know how to make rooks good
const ROOK_MAP: [i32; 64] = [
    -22, -24, -6, 4, 4, -6, -24, -22, -8, 6, 10, 12, 12, 10, 6, -8, -24, -4, 4, 10, 10, 4, -4, -24,
    -24, -12, -1, 6, 6, -1, -12, -24, -13, -5, -4, -6, -6, -4, -5, -13, -21, -7, 3, -1, -1, 3, -7,
    -21, -18, -10, -5, 9, 9, -5, -10, -18, -24, -13, -7, 2, 2, -7, -13, -24,
];

// @formatter:off
const QUEEN_MAP: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
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
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
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

    score -= board.pinned().popcnt() as i32 * 40;
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

pub fn eval(board: Board) -> i32 {
    (match board.clone().status() {
        BoardStatus::Stalemate => 0,
        BoardStatus::Ongoing => inner_eval(board),
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
    use crate::eval::{bishop_activity, passed_pawns, supported_pawns};
    use chess::Board;
    use std::str::FromStr;

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
