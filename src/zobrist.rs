use lazy_static::lazy_static;

use crate::board::{Board, Color, Piece, Square};
use crate::utils::pseudo_random;

fn gen_pk() -> [u64; 13 * 128] {
    let mut temp = [0; 13 * 128];
    for i in 0..13 * 128 {
        temp[i] = pseudo_random();
    }
    temp
}

fn gen_castle() -> [u64; 16] {
    let mut temp = [0; 16];
    for i in 0..16 {
        temp[i] = pseudo_random();
    }
    temp
}

lazy_static! {
    pub static ref PIECE_KEYS: [u64; 13 * 128] = gen_pk();
    pub static ref CASTLE_KEYS: [u64; 16] = gen_castle();
    pub static ref SIDE_KEY: u64 = pseudo_random();
}

impl Board {
    pub fn gen_hash(&self) -> u64 {
        let mut res = 0u64;

        for sq in 0..128 {
            if sq & 0x88 == 0 {
                let p = self.board[sq];
                if p != Piece::EP {
                    res ^= PIECE_KEYS[(p.usize() * 128) + sq]
                }
            }
        }

        if self.turn == Color::White {
            res ^= *SIDE_KEY;
        }

        if self.enpassant != Square::OFF_BOARD_ENPASSANT {
            res ^= PIECE_KEYS[self.enpassant.usize()];
        }

        res ^= CASTLE_KEYS[self.castle as usize];

        res
    }
}
