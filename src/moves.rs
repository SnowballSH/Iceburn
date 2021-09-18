use std::mem::transmute;

use crate::board::{Board, Piece, PieceType, Square};
use crate::utils::SQUARE_CHART;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move(pub u32);

impl Move {
    #[inline]
    pub fn source(self) -> Square {
        Square((self.0 & 0x7f) as u8)
    }

    #[inline]
    pub fn target(self) -> Square {
        Square(((self.0 >> 7) & 0x7f) as u8)
    }

    #[inline]
    pub fn promote(self) -> Piece {
        unsafe { transmute(((self.0 >> 14) & 0xf) as u8) }
    }

    #[inline]
    pub fn is_capture(self) -> bool {
        (self.0 >> 18) & 0x1 == 1
    }

    #[inline]
    pub fn is_double_pawn_push(self) -> bool {
        (self.0 >> 19) & 0x1 == 1
    }

    #[inline]
    pub fn is_enpassant(self) -> bool {
        (self.0 >> 20) & 0x1 == 1
    }

    #[inline]
    pub fn is_castling(self) -> bool {
        (self.0 >> 21) & 0x1 == 1
    }

    pub fn construct(
        source: Square,
        target: Square,
        promotion_piece: Piece,
        is_capture: bool,
        is_double_pawn_push: bool,
        is_enpassant: bool,
        is_castling: bool,
    ) -> Self {
        Move(
            source.0 as u32
                | ((target.0 as u32) << 7)
                | ((promotion_piece.u8() as u32) << 14)
                | ((is_capture as u32) << 18)
                | ((is_double_pawn_push as u32) << 19)
                | ((is_enpassant as u32) << 20)
                | ((is_castling as u32) << 21),
        )
    }

    pub fn from_uci(board: &Board, s: Vec<u8>) -> Option<Move> {
        let from = s[0] - 'a' as u8 + 16 * (8 - (s[1] - '0' as u8));
        let to = s[2] - 'a' as u8 + 16 * (8 - (s[3] - '0' as u8));

        let moves = board.gen_moves();

        for m in moves {
            let promoted;
            if m.source().0 == from && m.target().0 == to {
                if s.len() >= 5 {
                    promoted = m.promote();
                    if promoted != Piece::EP {
                        if promoted.piece_type() == PieceType::Knight && s[4] == 'n' as u8 {
                            return Some(m);
                        } else if promoted.piece_type() == PieceType::Bishop && s[4] == 'b' as u8 {
                            return Some(m);
                        } else if promoted.piece_type() == PieceType::Rook && s[4] == 'r' as u8 {
                            return Some(m);
                        } else if promoted.piece_type() == PieceType::Queen && s[4] == 'q' as u8 {
                            return Some(m);
                        }
                        // you cannot move a pawn to the other side without promoting! i.e. g7g8
                        continue;
                    }
                }

                // legal
                return Some(m);
            }
        }

        None
    }

    pub fn to_uci(&self) -> Vec<u8> {
        let mut v = vec![];
        let f = SQUARE_CHART[self.source().usize()];
        let t = SQUARE_CHART[self.target().usize()];
        v.extend_from_slice(&f);
        v.extend_from_slice(&t);
        let p = self.promote();
        if p != Piece::EP {
            v.push(p.piece_type().symbol() as u8);
        }
        v
    }

    pub fn to_human(&self) -> Vec<u8> {
        let mut v = vec![];
        let f = SQUARE_CHART[self.source().usize()];
        let t = SQUARE_CHART[self.target().usize()];
        v.extend_from_slice(&f);
        if self.is_enpassant() {
            v.extend_from_slice(" enpassant".as_bytes())
        }
        if self.is_capture() {
            v.extend_from_slice(" captures".as_bytes());
        }
        if self.is_double_pawn_push() {
            v.extend_from_slice(" double push to".as_bytes());
        }
        if !self.is_enpassant() && !self.is_capture() && !self.is_double_pawn_push() {
            v.extend_from_slice(" to".as_bytes());
        };
        v.push(' ' as u8);
        v.extend_from_slice(&t);
        let p = self.promote();
        if p != Piece::EP {
            v.extend_from_slice(" promotes to ".as_bytes());
            v.push(p.piece_type().symbol() as u8);
        }
        v
    }
}
