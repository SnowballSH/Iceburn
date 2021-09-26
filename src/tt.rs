use std::collections::HashMap;
use std::intrinsics::size_of;

use crate::moves::Move;
use crate::search::search_constants;

#[derive(Copy, Clone, Debug)]
pub enum TTFlag {
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone, Debug)]
pub struct TTEntry {
    pub depth: i32,
    pub flag: TTFlag,
    pub score: i32,
    pub move_: Move,
}

macro_rules! entry_size {
    ($t: ty) => {
        size_of::<$t>()
    };
}

pub const TT_MAX_SIZE: usize = 16_000_000 / entry_size!(TTEntry); // 16 MB

#[derive(Clone, Debug)]
pub struct TranspositionTable {
    pub table: HashMap<u32, TTEntry>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        TranspositionTable {
            table: HashMap::with_capacity(TT_MAX_SIZE),
        }
    }
}

impl TranspositionTable {
    /// search for a table value
    pub fn get(
        &self,
        alpha: i32,
        beta: i32,
        move_: &mut Move,
        depth: i32,
        hash: &u32,
        ply: usize,
    ) -> Option<i32> {
        if let Some(entry) = self.table.get(hash) {
            if entry.depth >= depth {
                let mut score = entry.score;

                if score < -search_constants::MATE_SCORE {
                    score += ply as i32;
                } else if score > search_constants::MATE_SCORE {
                    score -= ply as i32;
                }

                match entry.flag {
                    TTFlag::Exact => return Some(score),
                    TTFlag::Alpha => {
                        if score <= alpha {
                            return Some(alpha);
                        }
                    }
                    TTFlag::Beta => {
                        if score >= beta {
                            return Some(beta);
                        }
                    }
                }
            }

            *move_ = entry.move_;

            None
        } else {
            None
        }
    }

    /// inserts an element to TT
    pub fn insert(&mut self, hash: u32, mut entry: TTEntry, ply: usize) {
        if entry.score < -search_constants::MATE_SCORE {
            entry.score -= ply as i32;
        } else if entry.score > search_constants::MATE_SCORE {
            entry.score += ply as i32;
        }
        self.table.insert(hash, entry);
    }
}
