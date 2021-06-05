use chess::{ALL_PIECES, Board, BoardStatus, Color, Piece, Square};

pub const MATE_UPPER: i32 = 32_000 + 8 * QUEEN_VALUE;
pub const MATE_LOWER: i32 = 32_000 - 8 * QUEEN_VALUE;

pub const QUEEN_VALUE: i32 = 1800;

// @formatter:off
const PAWN_MAP: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    15, 10, 20, 36, 36, 20, 10, 15,
    11, 5, 10, 28, 28, 10,  5,  8,
    9,  0, 15, 24, 21,  6,  0,  5,
    5, -5,-10, -3, -3,-10, -5,  5,
    5, 10, 10,-23,-23, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

// @formatter:off
const KNIGHT_MAP: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5,  7, 15, 15,  9,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

// @formatter:off
const BISHOP_MAP: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

// @formatter:off
const ROOK_MAP: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0
];

// @formatter:off
const QUEEN_MAP: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

// @formatter:off
const KING_MAP_MIDDLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20, -5,  0,  0, -5, 20, 20,
     20, 35, 15,  0,  0, 10, 60, 20
];

// @formatter:off
const KING_MAP_END: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];

fn get(square: Square, is_black: bool) -> usize {
    let s = square.get_file().to_index() + 56 - square.get_rank().to_index() * 8;
    if is_black { s ^ 56 } else { s }
}

fn to_val(p: Square, board: Board) -> i32 {
    let pib = board.color_on(p).unwrap() == Color::Black;
    let k = get(p, pib);
    match board.piece_on(p).unwrap() {
        Piece::Pawn => 140 + PAWN_MAP[k],
        Piece::Knight => 586 + KNIGHT_MAP[k],
        Piece::Bishop => 630 + BISHOP_MAP[k],
        Piece::Rook => 929 + ROOK_MAP[k],
        Piece::Queen => QUEEN_VALUE + QUEEN_MAP[k],
        Piece::King => 32000 + (if board.combined().popcnt() < 9 { KING_MAP_END } else { KING_MAP_MIDDLE })[k],
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
    score -= board.checkers().popcnt() as i32 * 40;

    score
}

pub fn eval(board: Board) -> i32 {
    (match board.clone().status() {
        BoardStatus::Stalemate => 0,
        BoardStatus::Ongoing => inner_eval(board),
        BoardStatus::Checkmate => if board.side_to_move() == Color::Black {MATE_UPPER} else {MATE_LOWER}
    }) * if board.side_to_move() == Color::Black { -1 } else { 1 }
}