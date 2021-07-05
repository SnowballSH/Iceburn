use chess::ChessMove;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
// #[repr(u8)]
pub enum TTEntryFlag {
    Exact,
    Upper,
    Lower,
}

#[derive(Debug, Clone)]
pub struct TTEntry {
    pub score: i32,
    pub depth: i32,
    pub flag: TTEntryFlag,
    pub best_move: ChessMove,
}

pub type TT = HashMap<u64, TTEntry>;
