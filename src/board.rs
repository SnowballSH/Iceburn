use std::mem::transmute;

use Piece::*;

use crate::moves::Move;
use crate::utils::get_pair;

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

    pub fn symbol(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::None => '?',
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
    pub const PT_TO_PIECE: [Piece; 16] = [
        EP, WK, WP, WN, WB, WR, WQ, EP, EP, BK, BP, BN, BB, BR, BQ, EP,
    ];

    #[inline]
    pub fn u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn usize(self) -> usize {
        self as usize
    }

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

    pub fn color(self) -> Option<Color> {
        match self {
            BP | BB | BN | BR | BQ | BK => Some(Color::Black),
            WP | WB | WN | WR | WQ | WK => Some(Color::White),
            EP | OB => None,
        }
    }

    #[inline]
    pub fn from_pt_u8(pt: u8, color: Color) -> Self {
        Self::PT_TO_PIECE[pt as usize | ((color as usize) << 3)]
    }

    pub fn symbol(&self) -> char {
        let o = self.piece_type().symbol();
        if self.color() == Some(Color::White) {
            o.to_ascii_uppercase()
        } else {
            o
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Square(pub(crate) u8);

impl Square {
    const OFF_BOARD_ENPASSANT: Square = Square(120);

    #[inline]
    pub fn shift(&self, amount: i16) -> Self {
        Square((self.0 as i16 + amount) as u8)
    }

    pub fn is_attacked(&self, color: Color, board: &Board) -> bool {
        for pt in ((PieceType::King as u8)..=(PieceType::Queen as u8)).rev() {
            let piece = Piece::from_pt_u8(pt, color);

            if pt == PieceType::Pawn as u8 {
                let dir = 16 * (1 - 2 * color as i16);
                for hdir in [1_i16, -1] {
                    let to = self.0 as i16 + hdir + dir as i16;
                    if to & 0x88 == 0 && board.board[to as usize] == piece {
                        return true;
                    }
                }
            } else {
                // bishop, rook, queen
                let slider = pt >= 4;
                let dirs = unsafe { transmute::<u8, PieceType>(pt) }.offset();

                for d in dirs {
                    let mut to = self.0 as i16;

                    'l: loop {
                        to += d as i16;
                        if to & 0x88 != 0 {
                            break 'l;
                        }

                        if board.board[to as usize] != EP {
                            if board.board[to as usize] == piece {
                                return true;
                            }
                            break 'l;
                        }
                        if !slider {
                            break 'l;
                        }
                    }
                }
            }
        }

        false
    }
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

    pub pawn_ranks: (u8, u8),
    pub promote_ranks: (u8, u8),
    pub castling_rights: [u8; 128],
    pub castling_bits: ([u8; 2], [u8; 2]),
    pub king_location: (Square, Square),
}

impl Default for Board {
    fn default() -> Self {
        let mut b = Board {
            board: Self::STARTBOARD0X88,
            turn: Color::White,
            enpassant: Square::OFF_BOARD_ENPASSANT,
            castle: 0b1111,
            fifty_move: 0,
            piece_list: [Square(0); 13 * 10],
            piece_count: [0; 14],
            move_stack: Vec::with_capacity(64),

            pawn_ranks: (0x60, 0x10),
            promote_ranks: (0x00, 0x70),
            castling_rights: [
                7, 15, 15, 15, 3, 15, 15, 11, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15,
                15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13,
                13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 13, 13,
                13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15,
                15, 15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13,
                13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 12, 15, 15, 14, 13, 13, 13, 13, 13, 13,
                13, 13,
            ],
            castling_bits: ([1, 2], [4, 8]),
            king_location: (Square(0x74), Square(0x04)),
        };
        b.init_piece_list();
        b
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

    pub fn symbols(&self) -> Vec<Vec<char>> {
        let mut v = vec![vec![]];
        for sq in 0..128 {
            if sq & 0x88 == 0 {
                let pc = self.board[sq];
                if pc == EP {
                    v.last_mut().unwrap().push(' ');
                } else {
                    v.last_mut().unwrap().push(pc.symbol());
                }
            }
            if (sq + 1) % 16 == 0 {
                v.push(vec![]);
            }
        }
        v.pop();
        v
    }

    pub fn to_string(&self) -> String {
        let mut s: String = self
            .symbols()
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|y| y.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n");
        s.push_str(&format!("\n{} to move", if self.turn == Color::White {
            "White"
        } else {
            "Black"
        }));
        s
    }

    /// https://www.chessprogramming.org/Piece-Lists
    pub fn init_piece_list(&mut self) {
        self.piece_count = [0; 14];
        self.piece_list = [Square(0); 130];
        for sq in 0..128 {
            // if square is in-bound
            if sq & 0x88 == 0 {
                let pc = self.board[sq];
                if pc != EP {
                    self.piece_list[pc.usize() * 10 + self.piece_count[pc.usize()]] =
                        Square(sq as u8);
                    self.piece_count[pc.usize()] += 1;
                }
            }
        }
    }

    pub fn add_piece(&mut self, piece: Piece, square: Square) {
        self.board[square.0 as usize] = piece;
        self.piece_list[piece.usize() * 10 + self.piece_count[piece.usize()]] = square;
        self.piece_count[piece.usize()] += 1;
    }

    pub fn remove_piece(&mut self, piece: Piece, square: Square) {
        let mut captured_index = 0;
        for index in 0..self.piece_count[piece.usize()] {
            if self.piece_list[piece.usize() * 10 + index] == square {
                captured_index = index;
                break;
            }
        }

        self.piece_count[piece.usize()] -= 1;
        self.piece_list[piece.usize() * 10 + captured_index] =
            self.piece_list[piece.usize() * 10 + self.piece_count[piece.usize()]];
    }

    pub fn move_piece(&mut self, piece: Piece, source: Square, target: Square) {
        self.board[target.0 as usize] = piece;
        self.board[source.0 as usize] = Piece::EP;

        for index in 0..self.piece_count[piece.usize()] {
            if self.piece_list[piece.usize() * 10 + index] == source {
                self.piece_list[piece.usize() * 10 + index] = target;
                break;
            }
        }
    }

    pub fn make_move(&mut self, m: Move) -> bool {
        let source = m.source();
        let target = m.target();
        let captured = self.board[target.0 as usize];
        let promoted = m.promote();

        let piece = self.board[source.0 as usize];

        self.move_piece(piece, source, target);

        self.fifty_move += 1;

        if m.is_capture() {
            if captured != EP {
                self.remove_piece(captured, target);
            }
            self.fifty_move = 0;
        } else if self.board[target.0 as usize].piece_type() == PieceType::Pawn {
            self.fifty_move = 0;
        }

        // TODO enpassant

        if m.is_double_pawn_push() {
            if self.turn == Color::White {
                self.enpassant = target.shift(16);
            } else {
                self.enpassant = target.shift(-16);
            }
        } else if m.is_enpassant() {
            if self.turn == Color::White {
                self.board[target.0 as usize + 16] = EP;
                self.remove_piece(BP, target.shift(16));
            } else {
                self.board[target.0 as usize - 16] = EP;
                self.remove_piece(WP, target.shift(-16));
            }
        } else if m.is_castling() {
            match target.0 {
                // g1
                0x76 => self.move_piece(WR, Square(0x77), Square(0x75)),
                // c1
                0x72 => self.move_piece(WR, Square(0x70), Square(0x73)),
                // g8
                0x06 => self.move_piece(BR, Square(0x07), Square(0x05)),
                // c8
                0x02 => self.move_piece(BR, Square(0x00), Square(0x03)),
                _ => {}
            }
        }

        if promoted != EP {
            if self.turn == Color::White {
                self.remove_piece(WP, target);
            } else {
                self.remove_piece(BP, target);
            }
            self.add_piece(promoted, target);
        }

        if self.board[target.0 as usize].piece_type() == PieceType::King {
            if self.turn == Color::White {
                self.king_location.0 = target;
            } else {
                self.king_location.1 = target;
            }
        }

        self.castle &= self.castling_rights[source.0 as usize];
        self.castle &= self.castling_rights[target.0 as usize];

        self.turn = self.turn.not();

        /*
        if get_pair(self.king_location, self.turn.not()).is_attacked(self.turn, self) {
            self.take_back();
            return false;
        }
         */

        true
    }

    pub fn take_back(&mut self) {
        todo!("Implement it")
    }
}
