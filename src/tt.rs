use crate::moves::Move;
use crate::search::Depth;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub enum TTFlag {
    INVALID = 0,
    Exact = 1,
    Upper = 2,
    Lower = 4,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TTEntry {
    pub key: u16,
    pub score: i32,
    pub bestmove: Option<Move>,
    pub depth: Depth,
    pub flag: TTFlag,
}

impl TTEntry {
    pub fn construct(
        hash: u64,
        score: i32,
        bestmove: Option<Move>,
        depth: Depth,
        flags: TTFlag,
    ) -> Self {
        TTEntry {
            key: (hash >> 48) as u16,
            score,
            bestmove,
            depth,
            flag: flags
        }
    }

    pub fn is_key_valid(&self, hash: u64) -> bool {
        self.key == (hash >> 48) as u16
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            score: 0,
            bestmove: None,
            depth: 0,
            flag: TTFlag::INVALID,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TranspositionTable {
    pub table: Vec<TTEntry>,
    pub size: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::with_size(16)
    }
}

impl TranspositionTable {
    pub fn with_size(size_mb: u64) -> Self {
        let count = size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>() as u64;
        let new_ttentry_count = count.next_power_of_two() / 2;
        TranspositionTable {
            table: vec![TTEntry::default(); new_ttentry_count as usize],
            size: new_ttentry_count as usize,
        }
    }

    pub fn get(&self, hash: u64) -> Option<&TTEntry> {
        unsafe{
            let res = self.table.get_unchecked(hash as usize % self.size);
            if res.is_key_valid(hash) {
                Some(res)
            } else {
                None
            }
        }
    }

    pub fn insert(&mut self, hash: u64, entry: TTEntry) {
        self.table[hash as usize % self.size] = entry;
    }

    pub fn clear(&mut self) {
        self.table = vec![TTEntry::default(); self.size];
    }
}
