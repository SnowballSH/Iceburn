use std::mem::transmute;

use crate::board::Piece::{BK, EP, WP};
use crate::board::{Board, Piece, PieceType, Square};
use crate::moves::Move;
use crate::utils::get_pair;

impl Board {
    pub fn gen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(20);
        for piece in WP.usize()..=BK.usize() {
            for index in 0..self.piece_count[piece] {
                let from_sq = self.piece_list[piece * 10 + index];
                let piece_struct: Piece = unsafe { transmute(piece as u8) };
                if piece_struct.color() == Some(self.turn) {
                    let pt = piece_struct.piece_type();
                    match pt {
                        PieceType::Pawn => {
                            let dir = 16 * (2 * self.turn as i16 - 1);
                            let to = from_sq.0 as i16 + dir;

                            // basic movement: forward
                            // target square is empty and legal
                            if to & 0x88 == 0 && self.board[to as usize] == EP {
                                let to = to as u8;

                                // leftmost square is a promotion square
                                if to & 0xf0 == get_pair(self.promote_ranks, self.turn) {
                                    for p in
                                        ((PieceType::Knight as u8)..=(PieceType::Queen as u8)).rev()
                                    {
                                        moves.push(Move::construct(
                                            from_sq,
                                            Square(to),
                                            Piece::from_pt_u8(p, self.turn),
                                            false,
                                            false,
                                            false,
                                            false,
                                        ));
                                    }
                                } else {
                                    moves.push(Move::construct(
                                        from_sq,
                                        Square(to),
                                        Piece::EP,
                                        false,
                                        false,
                                        false,
                                        false,
                                    ));

                                    // double push
                                    let double = (to as i16 + dir) as u8;

                                    // double & 0x88 == 0 is not needed since pawns move vertically
                                    // also no need to check if the first square is blocked
                                    // if pawn hasn't moved and the target square is empty,
                                    if from_sq.0 & 0xf0 == get_pair(self.pawn_ranks, self.turn)
                                        && self.board[double as usize] == EP
                                    {
                                        moves.push(Move::construct(
                                            from_sq,
                                            Square(double),
                                            Piece::EP,
                                            false,
                                            true,
                                            false,
                                            false,
                                        ))
                                    }
                                }
                            } // basic movement end

                            // capture
                            for hdir in [1_i16, -1] {
                                let to = from_sq.0 as i16 + hdir + dir as i16;
                                if to & 0x88 != 0 {
                                    continue;
                                }

                                let to = to as u8;

                                let captured_piece = self.board[to as usize];
                                // if target is opponent's piece and is not empty
                                if captured_piece.color() == Some(self.turn.not()) {
                                    // if it is a promotion
                                    if to & 0xf0 == get_pair(self.promote_ranks, self.turn) {
                                        for p in ((PieceType::Knight as u8)
                                            ..=(PieceType::Queen as u8))
                                            .rev()
                                        {
                                            moves.push(Move::construct(
                                                from_sq,
                                                Square(to),
                                                Piece::from_pt_u8(p, self.turn),
                                                true,
                                                false,
                                                false,
                                                false,
                                            ));
                                        }
                                    } else {
                                        moves.push(Move::construct(
                                            from_sq,
                                            Square(to),
                                            Piece::EP,
                                            true,
                                            false,
                                            false,
                                            false,
                                        ))
                                    }
                                }
                            }
                        } // pawn end

                        _ => {
                            if pt == PieceType::King {
                                let king = get_pair(self.king_location, self.turn);

                                // king side
                                if self.castle & get_pair(self.castling_bits, self.turn)[0] != 0 {
                                    if self.board[king.0 as usize + 1] == EP
                                        && self.board[king.0 as usize + 2] == EP
                                    {
                                        // doesn't matter if king will be in check for now
                                        if !king.is_attacked(self.turn.not(), self)
                                            && !king.shift(1).is_attacked(self.turn.not(), self)
                                        {
                                            moves.push(Move::construct(
                                                king,
                                                king.shift(2),
                                                Piece::EP,
                                                false,
                                                false,
                                                false,
                                                true,
                                            ))
                                        }
                                    }
                                }

                                // queen side
                                if self.castle & get_pair(self.castling_bits, self.turn)[1] != 0 {
                                    if self.board[king.0 as usize - 1] == EP
                                        && self.board[king.0 as usize - 2] == EP
                                        && self.board[king.0 as usize - 3] == EP
                                    {
                                        // doesn't matter if king will be in check for now
                                        if !king.is_attacked(self.turn.not(), self)
                                            && !king.shift(-1).is_attacked(self.turn.not(), self)
                                        {
                                            moves.push(Move::construct(
                                                king,
                                                king.shift(-2),
                                                Piece::EP,
                                                false,
                                                false,
                                                false,
                                                true,
                                            ))
                                        }
                                    }
                                }
                            }

                            // bishop, rook, queen
                            let slider = pt as u8 >= 4;
                            let dirs = pt.offset();

                            for d in dirs {
                                let mut to = from_sq.0 as i16;

                                'l: loop {
                                    to += d as i16;
                                    if to & 0x88 != 0 {
                                        break 'l;
                                    }

                                    // capture
                                    if self.board[to as usize].color() == Some(self.turn.not()) {
                                        moves.push(Move::construct(
                                            from_sq,
                                            Square(to as u8),
                                            Piece::EP,
                                            true,
                                            false,
                                            false,
                                            false,
                                        ));

                                        break 'l;
                                    }
                                    // self-block
                                    else if self.board[to as usize].color() == Some(self.turn) {
                                        break 'l;
                                    }

                                    moves.push(Move::construct(
                                        from_sq,
                                        Square(to as u8),
                                        Piece::EP,
                                        false,
                                        false,
                                        false,
                                        false,
                                    ));

                                    if !slider {
                                        break 'l;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn gen_legal_moves(&self) -> Vec<Move> {
        todo!()
    }
}
