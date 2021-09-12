use Piece::*;

use crate::moves::Move;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn not(self) -> Self {
        if self == Color::White {
            Color::Black
        } else {
            Color::White
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PieceType {
    None,
    King,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl PieceType {
    pub const KNIGHT_OFFSETS: [i8; 8] = [33, 31, 18, 14, -33, -31, -18, -14];
    pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];
    pub const ROOK_OFFSETS: [i8; 4] = [16, -16, 1, -1];
    pub const KING_OFFSETS: [i8; 8] = [16, -16, 1, -1, 15, 17, -15, -17];

    #[inline]
    pub fn offset(self) -> Vec<i8> {
        match self {
            PieceType::Pawn => vec![],
            PieceType::King | PieceType::Queen => PieceType::KING_OFFSETS.to_vec(),
            PieceType::Knight => PieceType::KNIGHT_OFFSETS.to_vec(),
            PieceType::Bishop => PieceType::BISHOP_OFFSETS.to_vec(),
            PieceType::Rook => PieceType::ROOK_OFFSETS.to_vec(),
            PieceType::None => vec![],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Piece {
    EP,
    WP,
    WN,
    WB,
    WR,
    WQ,
    WK,
    BP,
    BN,
    BB,
    BR,
    BQ,
    BK,
    OB,
}

impl Piece {
    #[inline]
    pub fn u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn usize(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn piece_type(self) -> PieceType {
        match self {
            BP | WP => PieceType::Pawn,
            BB | WB => PieceType::Bishop,
            BN | WN => PieceType::Knight,
            BR | WR => PieceType::Rook,
            BQ | WQ => PieceType::Queen,
            BK | WK => PieceType::King,
            EP | OB => PieceType::None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Square(pub(crate) u8);

impl Square {
    const OFF_BOARD_ENPASSANT: Square = Square(120);
}

/// Board
pub struct Board {
    /// https://www.chessprogramming.org/0x88
    pub board: [Piece; 128],
    pub turn: Color,
    pub enpassant: Square,
    pub castle: u8,
    pub fifty_move: u8,
    /// https://www.chessprogramming.org/Piece-Lists
    pub piece_list: [Square; 13 * 10],
    pub piece_count: [usize; 14],
    pub move_stack: Vec<Move>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            board: Self::STARTBOARD0X88,
            turn: Color::White,
            enpassant: Square::OFF_BOARD_ENPASSANT,
            castle: 0b1111,
            fifty_move: 0,
            piece_list: [Square(0); 13 * 10],
            piece_count: [0; 14],
            move_stack: Vec::with_capacity(64),
        }
    }
}

impl Board {
    const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    #[rustfmt::skip]
    const STARTBOARD0X88: [Piece; 128] = [
        BR, BN, BB, BQ, BK, BB, BN, BR,  OB, OB, OB, OB, OB, OB, OB, OB,
        BP, BP, BP, BP, BP, BP, BP, BP,  OB, OB, OB, OB, OB, OB, OB, OB,
        EP, EP, EP, EP, EP, EP, EP, EP,  OB, OB, OB, OB, OB, OB, OB, OB,
        EP, EP, EP, EP, EP, EP, EP, EP,  OB, OB, OB, OB, OB, OB, OB, OB,
        EP, EP, EP, EP, EP, EP, EP, EP,  OB, OB, OB, OB, OB, OB, OB, OB,
        EP, EP, EP, EP, EP, EP, EP, EP,  OB, OB, OB, OB, OB, OB, OB, OB,
        WP, WP, WP, WP, WP, WP, WP, WP,  OB, OB, OB, OB, OB, OB, OB, OB,
        WR, WN, WB, WQ, WK, WB, WN, WR,  OB, OB, OB, OB, OB, OB, OB, OB,
    ];

    /// https://www.chessprogramming.org/Piece-Lists
    pub fn init_piece_list(&mut self) {
        self.piece_count = [0; 14];
        self.piece_list = [Square(0); 130];
        for sq in 0..128 {
            // if square is in-bound
            if sq & 0x88 == 0 {
                let pc = self.board[sq];
                if pc != EP {
                    self.piece_list[pc.usize() * 10 + self.piece_list[pc.usize()].0 as usize] =
                        Square(sq as u8);
                    self.piece_list[pc.usize()].0 += 1;
                }
            }
        }
    }
}
