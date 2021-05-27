use chess::{ALL_PIECES, Color, Game, GameResult, Piece, Square, File, Rank, Board, CastleRights};

pub static MATE_UPPER: i32 = i32::MAX - 14;
pub static MATE_LOWER: i32 = i32::MIN + 14;

const PAWN_MAP: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    15, 10, 20, 30, 30, 20, 10, 15,
    10, 5, 10, 25, 25, 10, 5, 10,
    7, 0, 5, 20, 20, 3, 0, 7,
    5, -5, -10, -3, -3, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0
];

const KNIGHT_MAP: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 0, 0, 0, -20, -40,
    -30, 0, 10, 15, 15, 10, 0, -30,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 0, 15, 20, 20, 15, 0, -30,
    -30, 5, 8, 15, 15, 12, 5, -30,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50
];

const BISHOP_MAP: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10,
    -10, 5, 5, 10, 10, 5, 5, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_MAP: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, 10, 10, 10, 10, 5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    0, 0, 0, 5, 5, 0, 0, 0
];

const QUEEN_MAP: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    0, 0, 5, 5, 5, 5, 0, -5,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20
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
        Piece::Pawn => 100 + PAWN_MAP[k],
        Piece::Knight => 320 + KNIGHT_MAP[k],
        Piece::Bishop => 380 + BISHOP_MAP[k],
        Piece::Rook => 500 + ROOK_MAP[k],
        Piece::Queen => 900 + QUEEN_MAP[k],
        Piece::King => 20000,
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
    match board.result() {
        None => inner_eval(board),
        Some(x) => {
            match x {
                GameResult::WhiteCheckmates => MATE_UPPER,
                GameResult::WhiteResigns => MATE_LOWER,
                GameResult::BlackCheckmates => MATE_LOWER,
                GameResult::BlackResigns => MATE_UPPER,
                GameResult::Stalemate => 0,
                GameResult::DrawAccepted => 0,
                GameResult::DrawDeclared => inner_eval(board),
            }
        }
    }
}