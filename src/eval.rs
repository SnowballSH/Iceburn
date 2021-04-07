use shakmaty::*;

pub const MATE_LOWER: i16 = i16::MIN + 2;
pub const MATE_UPPER: i16 = i16::MAX - 2;

const PAWN_MAP: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5, 5, 10, 25, 25, 10, 5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, -5, -10, 0, 0, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0
];

const KNIGHT_MAP: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 0, 0, 0, -20, -40,
    -30, 0, 10, 15, 15, 10, 0, -30,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 0, 15, 20, 20, 15, 0, -30,
    -30, 5, 10, 15, 15, 10, 5, -30,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50
];

const BISHOP_MAP: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10,
    -10, 5, 5, 10, 10, 5, 5, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_MAP: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, 10, 10, 10, 10, 5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    0, 0, 0, 5, 5, 0, 0, 0
];

const QUEEN_MAP: [i16; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    0, 0, 5, 5, 5, 5, 0, -5,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20
];

const KING_MAP: [i16; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    20, 20, 0, 0, 0, 0, 20, 20,
    20, 30, 10, 0, 0, 10, 30, 20
];

const KING_ENDGAME: [i16; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10, 0, 0, -10, -20, -30,
    -30, -10, 20, 30, 30, 20, -10, -30,
    -30, -10, 30, 40, 40, 30, -10, -30,
    -30, -10, 30, 40, 40, 30, -10, -30,
    -30, -10, 20, 30, 30, 20, -10, -30,
    -30, -30, 0, 0, 0, 0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50
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

pub fn eval_board(board: Chess) -> i16 {
    let mut score: i16 = 0;

    match board.outcome() {
        Some(o) => {
            score = match o {
                Outcome::Draw => 0,
                Outcome::Decisive {
                    winner: Color::White
                } => MATE_UPPER,
                Outcome::Decisive {
                    winner: Color::Black
                } => MATE_LOWER,
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
                    Role::King => {
                        let s = 20000 + (
                            if board.board().pieces().len() < 6 { KING_ENDGAME } else { KING_MAP }
                        )[get(y.0, pib)];
                        s
                    }
                    Role::Queen => {
                        let s = 900 + QUEEN_MAP[get(y.0, pib)] / 3;
                        s
                    }
                    Role::Rook => {
                        let s = 500 + ROOK_MAP[get(y.0, pib)];
                        s
                    }
                    Role::Bishop => {
                        let s = 320 + BISHOP_MAP[get(y.0, pib)];
                        s
                    }
                    Role::Knight => {
                        let s = 320 + KNIGHT_MAP[get(y.0, pib)];
                        s
                    }
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
