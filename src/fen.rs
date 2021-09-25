use crate::board::{Board, Color, Piece, Square};
use crate::utils::*;

impl Board {
    pub fn fen(&self) -> String {
        let mut fen = String::new();

        for r in 0..8 {
            let mut empty_count = 0;
            for f in 0..16 {
                let sq = r * 16 + f;
                if sq & 0x88 == 0 {
                    let piece = self.board[sq];

                    if piece == Piece::EP {
                        empty_count += 1;
                    } else {
                        if empty_count != 0 {
                            fen.push(char::from_digit(empty_count, 10).unwrap());
                        }
                        fen.push(piece.symbol());
                        empty_count = 0;
                    }
                }
            }

            if empty_count != 0 {
                fen.push(char::from_digit(empty_count, 10).unwrap());
            }
            if r < 7 {
                fen.push('/');
            }
        }

        fen.push(' ');
        if self.turn == Color::White {
            fen.push('w');
        } else {
            fen.push('b');
        }

        fen.push(' ');

        let mut did = false;
        if self.castle & Board::CASTLING_BITS[0][0] != 0 {
            fen.push('K');
            did = true;
        }
        if self.castle & Board::CASTLING_BITS[0][1] != 0 {
            fen.push('Q');
            did = true;
        }
        if self.castle & Board::CASTLING_BITS[1][0] != 0 {
            fen.push('k');
            did = true;
        }
        if self.castle & Board::CASTLING_BITS[1][1] != 0 {
            fen.push('q');
            did = true;
        }

        if !did {
            fen.push('-');
        }

        fen.push(' ');

        if self.enpassant != Square::OFF_BOARD_ENPASSANT {
            let name = SQUARE_CHART[self.enpassant.usize()];
            fen.push(name[0] as char);
            fen.push(name[1] as char);
        } else {
            fen.push('-');
        }

        fen.push(' ');
        fen.push_str(&*self.fifty_move.to_string());

        fen
    }
}
