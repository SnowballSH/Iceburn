use crate::board::{Piece, Square};

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
    pub fn promote(self) -> u32 {
        (self.0 >> 14) & 0xf
    }

    #[inline]
    pub fn is_capture(self) -> bool {
        (self.0 >> 18) & 0x1 == 1
    }

    #[inline]
    pub fn is_pawn(self) -> bool {
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
        piece: Piece,
        is_capture: bool,
        is_pawn: bool,
        is_enpassant: bool,
        is_castling: bool,
    ) -> Self {
        Move(
            source.0 as u32
                | ((target.0 as u32) << 7)
                | ((piece.u8() as u32) << 14)
                | ((is_capture as u32) << 18)
                | ((is_pawn as u32) << 19)
                | ((is_enpassant as u32) << 20)
                | ((is_castling as u32) << 21),
        )
    }

    pub fn from_uci(s: Vec<u8>) -> Option<Move> {
        let from = s[0] - 'a' as u8 + 16 * (8 - (s[1] - '0' as u8));
        let to = s[2] - 'a' as u8 + 16 * (8 - (s[3] - '0' as u8));

        // TODO actual moves based on board
        Some(Move::construct(
            Square(from),
            Square(to),
            Piece::EP,
            false,
            false,
            false,
            false,
        ))
    }
}
