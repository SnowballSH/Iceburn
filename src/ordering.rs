use shakmaty::{Move, MoveList};

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

    pub fn score_of(&self, m: &Move) -> u16 {
        if m.is_capture() {
            MMV_LVA[(m.role() as usize - 1) * 6 + (m.capture().unwrap() as usize - 1)] + 10000
        } else if m.is_promotion() {
            7000
        } else if m.is_zeroing() {
            1000
        } else {
            0
        }
    }

    pub fn next_move(&mut self) -> Option<Move> {
        if self.index >= self.ml.len() {
            return None;
        }

        let mut max = self.index;
        if self.score_list[max].is_none() {
            self.score_list[max] = Some(self.score_of(&self.ml[max]));
        }
        for j in self.index + 1..self.ml.len() {
            if self.score_list[j].is_none() {
                self.score_list[j] = Some(self.score_of(&self.ml[j]));
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
