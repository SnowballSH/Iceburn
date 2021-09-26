use crate::board::{Board, Color};

pub const MATERIAL_OPENING: [i32; 7] = [0, 110, 280, 330, 550, 1080, 0];

impl Board {
    pub fn fast_fast_eval(&self) -> i32 {
        let mut score = 0;
        for p in self.board {
            let pt = p.piece_type();
            if 1 <= pt as usize && pt as usize <= 5 {
                score +=
                    MATERIAL_OPENING[pt as usize] * if p.color() == Some(Color::White) { 1 } else { -1 };
            }
        }
        score
    }
}
