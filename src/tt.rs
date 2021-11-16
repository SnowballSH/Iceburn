use crate::chess::Move;
use crate::search::Depth;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub enum TTFlag {
    INVALID = 0,
    Exact = 1,
    Upper = 2,
    Lower = 4,
}

impl Default for TTFlag {
    fn default() -> Self {
        TTFlag::INVALID
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TTEntry {
    pub key: u64,
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
            key: hash,
            score,
            bestmove,
            depth,
            flag: flags,
        }
    }

    pub fn is_key_valid(&self, hash: u64) -> bool {
        self.key == hash
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
        let hash_size = 0x100000 * size_mb;
        let struct_size = std::mem::size_of::<TTEntry>() as u64;
        let hash_entries = hash_size / struct_size;
        let mut table = Vec::with_capacity(hash_entries as usize);
        table.resize(hash_entries as usize, TTEntry::default());
        TranspositionTable {
            table,
            size: hash_entries as usize,
        }
    }

    pub fn get(&self, hash: u64) -> Option<&TTEntry> {
        unsafe {
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
