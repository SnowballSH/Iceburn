use std::mem::transmute;

use Piece::*;

use crate::moves::Move;
use crate::utils::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn not(self) -> Self {
        unsafe { transmute(self as u8 ^ 1) }
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
    EP, // 0
    WP, // 1
    WN, // 2
    WB, // 3
    WR, // 4
    WQ, // 5
    WK, // 6
    BP, // 7
    BN, // 8
    BB, // 9
    BR, // 10
    BQ, // 11
    BK, // 12
    OB, // 13
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
            EP | OB => None,
            _ => Some(unsafe { transmute(self.usize() > 6) }),
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
pub struct Square(pub u8);

impl Square {
    const OFF_BOARD_ENPASSANT: Square = Square(120);

    #[inline]
    pub fn shift(&self, amount: i16) -> Self {
        Square((self.0 as i16 + amount) as u8)
    }

    #[inline]
    pub fn usize(self) -> usize {
        self.0 as usize
    }

    pub fn is_attacked_by(&self, color: Color, board: &Board) -> bool {
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
#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub king_location: [Square; 2],
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

            king_location: [Square(0x74), Square(0x04)],
        };
        b.init_piece_list();
        b
    }
}

impl Board {
    pub const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const CASTLING_RIGHTS: [u8; 128] = [
        7, 15, 15, 15, 3, 15, 15, 11, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15,
        15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 13,
        13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15,
        15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13,
        13, 13, 13, 13, 15, 15, 15, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 13, 13, 13, 13, 15, 15,
        15, 12, 15, 15, 14, 13, 13, 13, 13, 13, 13, 13, 13,
    ];

    pub const PAWN_RANKS: [u8; 2] = [0x60, 0x10];
    pub const PROMOTE_RANKS: [u8; 2] = [0x00, 0x70];

    pub const CASTLING_BITS: [[u8; 2]; 2] = [[1, 2], [4, 8]];

    #[rustfmt::skip]
    pub const STARTBOARD0X88: [Piece; 128] = [
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
        self.to_string_padding(0)
    }

    pub fn to_string_padding(&self, padding: usize) -> String {
        let mut s: String = self
            .symbols()
            .into_iter()
            .map(|x| {
                " ".repeat(padding).to_string()
                    + &x.into_iter()
                        .map(|y| y.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n");
        s.push_str(&format!(
            "\n{} to move",
            if self.turn == Color::White {
                "White"
            } else {
                "Black"
            }
        ));
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

    /// is king checked?
    pub fn is_checked(&self, color: Color) -> bool {
        self.king_location[color as usize].is_attacked_by(color.not(), &self)
    }

    /// adds a piece to the board
    pub fn add_piece(&mut self, piece: Piece, square: Square) {
        self.board[square.usize()] = piece;
        if piece != EP {
            self.piece_list[piece.usize() * 10 + self.piece_count[piece.usize()]] = square;
            self.piece_count[piece.usize()] += 1;
        }
    }

    /// removes a piece from the board
    pub fn remove_piece(&mut self, piece: Piece, square: Square) {
        if piece != EP {
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
    }

    /// moves a piece from source to target
    pub fn move_piece(&mut self, piece: Piece, source: Square, target: Square) {
        self.board[target.usize()] = self.board[source.usize()];
        self.board[source.usize()] = Piece::EP;

        if piece != EP {
            for index in 0..self.piece_count[piece.usize()] {
                if self.piece_list[piece.usize() * 10 + index] == source {
                    self.piece_list[piece.usize() * 10 + index] = target;
                    break;
                }
            }
        }
    }

    /// performs a pseudo-legal move on the board. Returns false if king will be in check.
    pub fn make_move(&mut self, m: Move) {
        let source = m.source();
        let target = m.target();
        let captured = self.board[target.usize()];
        let promoted = m.promote();

        let piece = self.board[source.usize()];

        self.move_piece(piece, source, target);

        self.fifty_move += 1;

        if m.is_capture() {
            if captured != EP {
                self.remove_piece(captured, target);
            }
            self.fifty_move = 0;
        } else if self.board[target.usize()].piece_type() == PieceType::Pawn {
            self.fifty_move = 0;
        }

        if self.enpassant != Square::OFF_BOARD_ENPASSANT {
            // update hash key
        }
        // reset
        self.enpassant = Square::OFF_BOARD_ENPASSANT;

        if m.is_double_pawn_push() {
            if self.turn == Color::White {
                self.enpassant = target.shift(16);
            } else {
                self.enpassant = target.shift(-16);
            }
        } else if m.is_enpassant() {
            if self.turn == Color::White {
                self.board[target.usize() + 16] = EP;
                self.remove_piece(BP, target.shift(16));
            } else {
                self.board[target.usize() - 16] = EP;
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

        if self.board[target.usize()].piece_type() == PieceType::King {
            self.king_location[self.turn as usize] = target;
        }

        self.castle &= Self::CASTLING_RIGHTS[source.usize()];
        self.castle &= Self::CASTLING_RIGHTS[target.usize()];

        self.turn = self.turn.not();
    }

    /// undo the last move
    #[cfg(undo)]
    pub fn undo_move(&mut self) {
        let history = self.move_stack.pop().expect("No move to undo");
        let m = history.move_;
        let source = m.source();
        let target = m.target();

        // move piece back
        self.move_piece(self.board[target.usize()], target, source);

        // restore captured piece
        if m.is_capture() {
            self.add_piece(history.captured, target);
        }

        if m.is_enpassant() {
            if self.turn == Color::White {
                self.add_piece(WP, target.shift(-16));
            } else {
                self.add_piece(BP, target.shift(16));
            }
        } else if m.is_castling() {
            match target.0 {
                // g1
                0x76 => self.move_piece(WR, Square(0x75), Square(0x77)),
                // c1
                0x72 => self.move_piece(WR, Square(0x73), Square(0x70)),
                // g8
                0x06 => self.move_piece(BR, Square(0x05), Square(0x07)),
                // c8
                0x02 => self.move_piece(BR, Square(0x03), Square(0x00)),
                _ => {}
            }
        } else {
            let promote = m.promote();
            if promote != EP {
                if self.turn == Color::White {
                    self.add_piece(BP, source);
                } else {
                    self.add_piece(WP, source);
                }
                self.remove_piece(promote, source);
            }
        }

        if self.board[source.usize()].piece_type() == PieceType::King {
            *get_pair_mut(&mut self.king_location, self.turn.not()) = source;
        }

        self.turn = history.turn;

        self.enpassant = history.enpassant;
        self.fifty_move = history.fifty;
        self.castle = history.castle;
    }
}

#[cfg(test)]
mod tests {
    use crate::board::*;

    #[test]
    fn simple() {
        let mut board = Board::default();
        let moves = board.gen_moves();
        assert_eq!(moves.len(), 20);

        let moves_list = ["g1f3", "d7d5", "g2g3", "c8h3", "f1h3", "d8d6", "e1g1"];
        for mm in moves_list {
            let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
            board.make_move(m);
        }

        let mut board = Board::default();
        let moves_list = [
            "f2f4", "g7g5", "b2b4", "g5f4", "g2g3", "f4g3", "f1g2", "g3h2", "d2d3", "h2g1q", "h1g1",
        ];
        for mm in moves_list {
            let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
            board.make_move(m);
        }

        let mut board = Board::default();
        let moves_list = ["e2e4", "a7a6", "e4e5", "d7d5", "e5d6", "d8d6"];
        for mm in moves_list {
            let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
            board.make_move(m);
        }
    }

    /*
    #[test]
    fn undo() {
        let mut board = Board::default();
        let moves_list = [
            "a2a4", "b7b5", "a4b5", "c7c6", "b5c6", "c8b7", "c6b7", "d8c7", "b7a8r", "c7b7", "a8b8"
        ];
        let mut positions = vec![];
        for mm in moves_list {
            let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
            positions.push(board.clone());
            board.make_move(m);
        }
        for _i in 0..moves_list.len() {
            dbg!(moves_list[moves_list.len() - _i - 1]);
            println!("{}", board.to_string());
            board.undo_move();
            println!("{}", board.to_string());
            let o = positions.pop().unwrap();
            assert_eq!(board.board, o.board);
            assert_eq!(board.castle, o.castle);
            assert_eq!(board.fifty_move, o.fifty_move);
            assert_eq!(board.turn, o.turn);
            assert_eq!(board.enpassant, o.enpassant);
            assert_eq!(board.piece_count[1..], o.piece_count[1..]);
        }

        let mut board = Board::default();
        let moves_list = [
            "a2a4", "d7d6", "a4a5", "b7b5", "a5b6", "c7b6",
        ];
        let mut positions = vec![];
        for mm in moves_list {
            let m = Move::from_uci(&board, mm.as_bytes().to_vec()).unwrap();
            positions.push(board.clone());
            board.make_move(m);
        }
        for _i in 0..moves_list.len() {
            dbg!(moves_list[moves_list.len() - _i - 1]);
            println!("{}", board.to_string());
            board.undo_move();
            println!("{}", board.to_string());
            let o = positions.pop().unwrap();
            assert_eq!(board.board, o.board);
            assert_eq!(board.castle, o.castle);
            assert_eq!(board.fifty_move, o.fifty_move);
            assert_eq!(board.turn, o.turn);
            assert_eq!(board.enpassant, o.enpassant);
            assert_eq!(board.piece_count[1..], o.piece_count[1..]);
        }
    }
     */
}
