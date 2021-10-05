use crate::chess::{Move, MoveList, Setup};
use crate::search::Depth;
use shakmaty::Chess;

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
    pub killer_moves: [[Option<Move>; 512]; 2],
}

macro_rules! none_512 {
    () => {
        [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        ]
    };
}

const NONE512: [Option<Move>; 512] = none_512!();

impl Default for OrderingHistory {
    fn default() -> Self {
        OrderingHistory {
            history_moves: [[0; 64]; 64],
            killer_moves: [NONE512, NONE512],
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

    ///https://www.chessprogramming.org/Killer_Heuristic
    pub fn is_killer(&self, board: &Chess, m: Move, ply: usize) -> bool {
        let color = board.turn() as usize;
        if self.killer_moves[color][ply] == Some(m) {
            return true;
        }
        false
    }

    pub fn add_killer(&mut self, board: &Chess, m: Move, ply: usize) {
        let color = board.turn() as usize;
        self.killer_moves[color][ply] = Some(m);
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

    pub fn score_of(
        &self,
        m: &Move,
        oh: &OrderingHistory,
        hash_move: &Option<Move>,
        board: &Chess,
        ply: usize,
    ) -> u16 {
        let mut score = 0;

        if let Some(hash_move) = hash_move {
            if m == hash_move {
                score += 10000;
            }
        }

        if oh.is_killer(board, m.clone(), ply) {
            score += 90;
        }

        score += if m.is_capture() {
            MMV_LVA[(m.role() as usize - 1) * 6 + (m.capture().unwrap() as usize - 1)] + 200
        } else if m.is_promotion() {
            5000
        } else if m.is_zeroing() {
            1
        } else {
            90.min(oh.history_moves[m.from().unwrap() as usize][m.to() as usize])
        };

        score
    }

    pub fn next_move(
        &mut self,
        oh: &OrderingHistory,
        hash_move: &Option<Move>,
        board: &Chess,
        ply: usize,
    ) -> Option<Move> {
        if self.index >= self.ml.len() {
            return None;
        }

        let mut max = self.index;
        if self.score_list[max].is_none() {
            self.score_list[max] = Some(self.score_of(&self.ml[max], oh, hash_move, board, ply));
        }
        for j in self.index + 1..self.ml.len() {
            if self.score_list[j].is_none() {
                self.score_list[j] = Some(self.score_of(&self.ml[j], oh, hash_move, board, ply));
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
