use crate::chess::{Move, MoveList};
use crate::search::Depth;

#[rustfmt::skip]
pub const MMV_LVA: [u16; 36] = [
    105, 205, 305, 405, 505, 605,
    104, 204, 304, 404, 504, 604,
    103, 203, 303, 403, 503, 603,
    102, 202, 302, 402, 502, 602,
    101, 201, 301, 401, 501, 601,
    100, 200, 300, 400, 500, 600,
];

#[derive(Debug)]
pub struct OrderingHistory {
    pub history_moves: [[u16; 64]; 64],
}

impl Default for OrderingHistory {
    fn default() -> Self {
        OrderingHistory {
            history_moves: [[0; 64]; 64],
        }
    }
}

const HIS_CAP: u16 = u16::MAX / 2 - 1;

impl OrderingHistory {
    /// https://www.chessprogramming.org/History_Heuristic
    pub fn add_history(&mut self, m: &Move, depth: Depth) {
        self.history_moves[m.from().unwrap() as usize][m.to() as usize] +=
            depth as u16 * depth as u16;
        if self.history_moves[m.from().unwrap() as usize][m.to() as usize] > HIS_CAP {
            self.history_moves
                .iter_mut()
                .for_each(|x| x.iter_mut().for_each(|y| *y /= 2));
        }
    }
}

#[derive(Debug)]
pub struct MoveOrderer {
    pub ml: MoveList,
    pub score_list: Vec<Option<u16>>,
    pub index: usize,
}

impl MoveOrderer {
    pub fn new(ml: MoveList) -> Self {
        let l = ml.len();
        MoveOrderer {
            ml,
            score_list: vec![None; l],
            index: 0,
        }
    }

    pub fn score_of(&self, m: &Move, oh: &OrderingHistory, hash_move: &Option<Move>) -> u16 {
        let mut score = 0;

        if let Some(hash_move) = hash_move {
            if m == hash_move {
                score += 10000;
            }
        }

        score += if m.is_capture() {
            MMV_LVA[(m.role() as usize - 1) * 6 + (m.capture().unwrap() as usize - 1)] + 200
        } else if m.is_promotion() {
            5000
        } else if m.is_zeroing() {
            100
        } else {
            80.min(oh.history_moves[m.from().unwrap() as usize][m.to() as usize])
        };

        score
    }

    pub fn next_move(&mut self, oh: &OrderingHistory, hash_move: &Option<Move>) -> Option<Move> {
        if self.index >= self.ml.len() {
            return None;
        }

        let mut max = self.index;
        if self.score_list[max].is_none() {
            self.score_list[max] = Some(self.score_of(&self.ml[max], oh, hash_move));
        }
        for j in self.index + 1..self.ml.len() {
            if self.score_list[j].is_none() {
                self.score_list[j] = Some(self.score_of(&self.ml[j], oh, hash_move));
            }
            if self.score_list[j] > self.score_list[max] {
                max = j;
            }
        }

        if max != self.index {
            self.ml.swap(self.index, max);
            self.score_list.swap(self.index, max);
        }

        self.index += 1;

        Some(self.ml[self.index - 1].clone())
    }
}
