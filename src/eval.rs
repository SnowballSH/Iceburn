use chess::{ALL_PIECES, Board, Color, File, Game, GameResult, Piece, Rank, Square};

pub const MATE_UPPER: i32 = 32_000 + 8 * QUEEN_VALUE;
pub const MATE_LOWER: i32 = 32_000 - 8 * QUEEN_VALUE;

pub const QUEEN_VALUE: i32 = 2529;

// @formatter:off
const PAWN_MAP: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10, -3, -3,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

// @formatter:off
const KNIGHT_MAP: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
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
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
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
    let s = match square.get_file() {
        File::A => 0,
        File::B => 1,
        File::C => 2,
        File::D => 3,
        File::E => 4,
        File::F => 5,
        File::G => 6,
        File::H => 7,
    } + match square.get_rank() {
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

fn to_val(p: Square, board: Board) -> i32 {
    let pib = board.color_on(p).unwrap() == Color::Black;
    let k = get(p, pib);
    match board.piece_on(p).unwrap() {
        Piece::Pawn => 136 + PAWN_MAP[k],
        Piece::Knight => 782 + KNIGHT_MAP[k],
        Piece::Bishop => 830 + BISHOP_MAP[k],
        Piece::Rook => 1289 + ROOK_MAP[k],
        Piece::Queen => QUEEN_VALUE + QUEEN_MAP[k],
        Piece::King => 32000 + (if board.combined().popcnt() < 9 { KING_MAP_END } else { KING_MAP_MIDDLE })[k],
    }
}

fn inner_eval(board: Game) -> i32 {
    let mut score = 0;
    let b = board.current_position();

    for piece_type in ALL_PIECES.iter() {
        let piece_bb = b.pieces(*piece_type);
        let white = b.color_combined(Color::White);
        let black = b.color_combined(Color::Black);

        let it = &mut (piece_bb & white).to_owned();
        for p in it {
            score += to_val(p, board.current_position());
        }

        let it = &mut (piece_bb & black).to_owned();
        for p in it {
            score -= to_val(p, board.current_position());
        }
    }

    score
}

pub fn eval(board: Game) -> i32 {
    (match board.clone().result() {
        None => inner_eval(board.clone()),
        Some(x) => {
            match x {
                GameResult::WhiteCheckmates => MATE_UPPER,
                GameResult::WhiteResigns => MATE_LOWER,
                GameResult::BlackCheckmates => MATE_LOWER,
                GameResult::BlackResigns => MATE_UPPER,
                GameResult::Stalemate => 0,
                GameResult::DrawAccepted => 0,
                GameResult::DrawDeclared => inner_eval(board.clone()),
            }
        }
    }) * if board.side_to_move() == Color::Black { -1 } else { 1 }
}