use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use lazy_static::lazy_static;

use crate::chess::uci::Uci;
use crate::chess::{Chess, Move, Position, Setup};
use crate::nnue::nnue_eval_normal;
use crate::ordering::{MoveOrderer, OrderingHistory};
use crate::timeman::*;
use crate::tt::{TTEntry, TTFlag, TranspositionTable};
use crate::weight::{fast_eval, is_checkmate, INF_SCORE};

pub type Depth = i8;
pub type Ply = usize;

#[derive(Copy, Clone, Debug, Default)]
pub struct Statistics {
    pub leafs: u64,
    pub qleafs: u64,
    pub beta_cutoffs: u64,
    pub qbeta_cutoffs: u64,
    pub tt_hits: u64,
    pub nodes: u64,
    pub qnodes: u64,
}

impl Statistics {
    pub fn total_nodes(&self) -> u64 {
        self.nodes + self.qnodes
    }
}

pub struct Search<'a> {
    pub stop: bool,
    pub sel_depth: Ply,
    pub timer: Timer,
    pub tt: &'a mut TranspositionTable,
    pub stats: Statistics,
    pub ordering_history: OrderingHistory,
    pub move_table: Vec<u64>,
}

impl<'a> Search<'a> {
    pub fn new(timer: Timer, tt: &'a mut TranspositionTable) -> Self {
        Search {
            stop: false,
            sel_depth: 0,
            timer,
            tt,
            stats: Statistics::default(),
            ordering_history: OrderingHistory::default(),
            move_table: Vec::with_capacity(100),
        }
    }

    pub fn mtdf(&mut self, board: &Chess) -> (Move, i32) {
        let mut alpha = -INF_SCORE;
        let mut beta = INF_SCORE;
        let mut depth = 1;
        let mut final_move = None;
        let mut final_score = 0;
        let mut last_score = 0;

        let moves = board.legal_moves();
        if moves.len() == 1 {
            return (moves[0].clone(), 0);
        }

        while !self.stop && self.timer.start_check(depth) && !is_checkmate(final_score) {
            if board.is_game_over() {
                break;
            }

            let res = self.negamax_root(board, depth, alpha, beta);

            if self.stop {
                break;
            }

            let cur_move = res.0;
            final_score = res.1;

            if depth >= 4 {
                self.timer.update(final_score - last_score);
            }
            last_score = final_score;

            if final_score <= alpha {
                alpha = -INF_SCORE;
            } else if final_score >= beta {
                beta = INF_SCORE;
            } else {
                if let Some(res) = self.print_info(board, depth, cur_move, final_score) {
                    final_move = Some(res);
                }

                alpha = final_score - Self::ASPIRATION_WINDOW;
                beta = final_score + Self::ASPIRATION_WINDOW;
                depth += 1;
                self.stats = Statistics::default();
            }

            if is_checkmate(final_score) {
                break;
            }
        }
        (final_move.unwrap(), final_score)
    }

    pub fn negamax_root(
        &mut self,
        board: &Chess,
        mut depth: Depth,
        mut alpha: i32,
        beta: i32,
    ) -> (Move, i32) {
        let moves = board.legal_moves();

        let in_check = board.is_check();
        if in_check {
            depth += 1;
        }

        let first_move = moves[0].clone();

        let mut best_move = first_move;
        if moves.len() == 1 {
            return (best_move, 0);
        }

        let mut hasher = DefaultHasher::new();
        board.board().hash(&mut hasher);
        let hs = hasher.finish();

        let mut hash_move = None;
        if let Some(ttentry) = self.tt.get(hs) {
            hash_move = ttentry.bestmove.clone();
        }

        let mut value;
        let mut orderer = MoveOrderer::new(moves);
        while let Some(m) = orderer.next_move(&self.ordering_history, &hash_move, board, 0) {
            let mut nb = board.clone();
            nb.play_unchecked(&m);

            let mut hasher = DefaultHasher::new();
            nb.board().hash(&mut hasher);
            let nhs = hasher.finish();

            self.move_table.push(nhs);

            value = -self.negamax(&mut nb, depth - 1, 1, -beta, -alpha, true);

            self.move_table.pop();

            if self.stop || self.timer.stop_check() {
                self.stop = true;
                break;
            }

            if value > alpha {
                best_move = m;
                if value >= beta {
                    self.tt.insert(
                        hs,
                        TTEntry::construct(hs, beta, Some(best_move.clone()), depth, TTFlag::Lower),
                    );
                    return (best_move.clone(), beta);
                }
                alpha = value;
                self.tt.insert(
                    hs,
                    TTEntry::construct(hs, alpha, Some(best_move.clone()), depth, TTFlag::Upper),
                );
            }
        }

        if !self.stop {
            self.tt.insert(
                hs,
                TTEntry::construct(hs, alpha, Some(best_move.clone()), depth, TTFlag::Exact),
            );
        }
        (best_move.clone(), alpha)
    }

    /// core negamax search
    pub fn negamax(
        &mut self,
        board: &Chess,
        mut depth: Depth,
        ply: Ply,
        mut alpha: i32,
        mut beta: i32,
        can_apply_null: bool,
    ) -> i32 {
        if self.stop || self.timer.stop_check() {
            self.stop = true;
            return 0;
        }

        // Mate distance pruning
        // https://www.chessprogramming.org/Mate_Distance_Pruning
        let mate_value = INF_SCORE - (ply as i32);
        if alpha < -mate_value {
            alpha = -mate_value;
        }
        if beta > mate_value - 1 {
            beta = mate_value - 1;
        }
        if alpha >= beta {
            self.stats.leafs += 1;
            return alpha;
        }

        let in_check = board.is_check();

        // Extend search in check
        if in_check {
            depth += 1;
        }
        depth = depth.max(0);

        // Quiescence search
        // when depth is zero, we go for captures
        if depth <= 0 && !in_check {
            return self.q_search(board, ply, alpha, beta);
        }

        self.stats.nodes += 1;

        let mut hasher = DefaultHasher::new();
        board.board().hash(&mut hasher);
        let hs = hasher.finish();

        // Fifty-move rule and Repetitions
        if board.halfmoves() >= 100 || self.is_repetition(hs) {
            self.stats.leafs += 1;
            return 0;
        }

        // Transposition Table
        let mut hash_move = None;
        if let Some(tt_entry) = self.tt.get(hs) {
            if tt_entry.depth >= depth {
                self.stats.tt_hits += 1;
                match tt_entry.flag {
                    TTFlag::Exact => {
                        self.stats.leafs += 1;
                        return tt_entry.score;
                    }
                    TTFlag::Lower => {
                        alpha = alpha.max(tt_entry.score);
                    }
                    TTFlag::Upper => {
                        beta = beta.max(tt_entry.score);
                    }
                    _ => {}
                }
                if alpha >= beta {
                    self.stats.leafs += 1;
                    self.stats.beta_cutoffs += 1;
                    return tt_entry.score;
                }
            }
            hash_move = tt_entry.bestmove.clone();
        }

        // Null Move Pruning
        // Borrowed from https://github.com/Heiaha/Weiawaga/blob/92d1c9b7f78ec6a6f8762d8719fa7e8a871cf8df/src/search/search.rs#L224
        if Self::can_apply_null(board, depth, beta, in_check, can_apply_null) {
            let r = if depth > 6 { 3 } else { 2 };

            let nb = board.clone().swap_turn().unwrap();
            let value = -self.negamax(&nb, depth - r - 1, ply, -beta, -beta + 1, false);
            if self.stop {
                return 0;
            }
            if value >= beta {
                self.stats.beta_cutoffs += 1;
                return beta;
            }
        }

        let mut value: i32;
        let mut reduced_depth: Depth;
        let mut best_move: Option<Move> = None;
        let mut tt_flag = TTFlag::Upper;
        let moves = board.legal_moves();
        let lmoves = moves.len();
        let mut orderer = MoveOrderer::new(moves);

        let mut pidx = 0;

        while let Some(m) = orderer.next_move(&self.ordering_history, &hash_move, board, ply) {
            let mut nb = board.clone();
            nb.play_unchecked(&m);

            let mut hasher = DefaultHasher::new();
            nb.board().hash(&mut hasher);
            let nhs = hasher.finish();
            self.move_table.push(nhs);

            // PVS
            if pidx == 0 {
                value = -self.negamax(&nb, depth - 1, ply + 1, -beta, -alpha, true);
            } else {
                reduced_depth = depth;

                // LMR
                if Self::can_apply_lmr(&m, depth, pidx) {
                    reduced_depth -= Self::late_move_reduction(depth, pidx);
                }

                loop {
                    value =
                        -self.negamax(&nb, reduced_depth - 1, ply + 1, -alpha - 1, -alpha, true);
                    if value > alpha {
                        value = -self.negamax(&nb, reduced_depth - 1, ply + 1, -beta, -alpha, true);
                    }

                    if reduced_depth != depth && value > alpha {
                        reduced_depth = depth;
                    } else {
                        break;
                    }
                }
            }

            self.move_table.pop();

            if self.stop {
                return 0;
            }

            // Cutoff, TT storage
            if value > alpha {
                best_move = Some(m.clone());
                if value >= beta {
                    if !m.is_capture() && !m.is_promotion() {
                        self.ordering_history.add_killer(board, m.clone(), ply);
                        self.ordering_history.add_history(&m, depth);
                    }
                    self.stats.beta_cutoffs += 1;
                    tt_flag = TTFlag::Lower;
                    alpha = beta;
                    break;
                }
                tt_flag = TTFlag::Exact;
                alpha = value;
            }

            pidx += 1;
        }

        if lmoves == 0 {
            if in_check {
                alpha = -mate_value;
            } else {
                alpha = 0;
            }
        }

        if !self.stop {
            self.tt
                .insert(hs, TTEntry::construct(hs, alpha, best_move, depth, tt_flag));
        }
        alpha
    }

    pub fn q_search(&mut self, board: &Chess, ply: Ply, mut alpha: i32, beta: i32) -> i32 {
        if self.stop || self.timer.stop_check() {
            self.stop = true;
            return 0;
        }

        self.sel_depth = self.sel_depth.max(ply);
        self.stats.qnodes += 1;

        let value = nnue_eval_normal(board);

        if value >= beta {
            self.stats.qleafs += 1;
            return beta;
        }

        if alpha < value {
            alpha = value;
        }

        let mut hasher = DefaultHasher::new();
        board.board().hash(&mut hasher);
        let hs = hasher.finish();

        let mut hash_move = None;
        if let Some(ttentry) = self.tt.get(hs) {
            hash_move = ttentry.bestmove.clone();
        }

        let mut value;

        let moves = board.capture_moves();
        let mut orderer = MoveOrderer::new(moves);

        while let Some(m) = orderer.next_move(&self.ordering_history, &hash_move, board, ply) {
            let mut nb = board.clone();
            nb.play_unchecked(&m);

            let mut hasher = DefaultHasher::new();
            nb.board().hash(&mut hasher);
            let nhs = hasher.finish();
            self.move_table.push(nhs);

            value = -self.q_search(&nb, ply + 1, -beta, -alpha);

            self.move_table.pop();

            if self.stop {
                return 0;
            }

            if value > alpha {
                if value >= beta {
                    self.stats.qbeta_cutoffs += 1;
                    return beta;
                }
                alpha = value;
            }
        }
        alpha
    }

    #[inline]
    fn is_repetition(&self, position: u64) -> bool {
        self.move_table.iter().rev().skip(1).any(|&x| x == position)
    }

    #[inline]
    fn can_apply_null(
        board: &Chess,
        depth: Depth,
        beta: i32,
        in_check: bool,
        can_apply_null: bool,
    ) -> bool {
        let mt = board.board().material_side(board.turn());
        can_apply_null
            && !in_check
            && depth >= Self::NULL_MIN_DEPTH
            && (mt.count() - mt.pawns as usize) > 0
            && fast_eval(board) >= beta
    }

    #[inline]
    fn can_apply_lmr(m: &Move, depth: Depth, move_index: usize) -> bool {
        depth >= Self::LMR_MIN_DEPTH
            && move_index >= Self::LMR_MOVE_WO_REDUCTION
            && !m.is_promotion()
            && !m.is_capture()
    }

    #[inline]
    fn late_move_reduction(depth: Depth, move_index: usize) -> Depth {
        LMR_TABLE[(depth as usize).min(63)][move_index.min(63)]
    }

    pub fn get_pv(&self, board: &mut Chess, depth: Depth) -> String {
        if depth == 0 {
            return "".to_owned();
        }

        let mut hasher = DefaultHasher::new();
        board.board().hash(&mut hasher);
        let hs = hasher.finish();

        let hash_move;
        let tt_entry = self.tt.get(hs);
        match tt_entry {
            Some(tt_entry) => {
                hash_move = tt_entry.bestmove.clone();
                if hash_move == None {
                    return "".to_owned();
                }
            }
            None => {
                return "".to_owned();
            }
        }

        board.play_unchecked(&hash_move.clone().unwrap());

        let pv = crate::chess::uci::Uci::from_standard(&hash_move.unwrap()).to_string()
            + " "
            + &*self.get_pv(board, depth - 1);

        pv
    }

    pub fn print_info(&self, board: &Chess, depth: Depth, m: Move, score: i32) -> Option<Move> {
        let pv = self.get_pv(&mut board.clone(), depth);
        println!(
            "info currmove {} depth {} seldepth {} time {} score cp {} nodes {} nps {} pv {}",
            crate::chess::uci::Uci::from_standard(&m).to_string(),
            depth,
            self.sel_depth,
            self.timer.elapsed(),
            score,
            self.stats.total_nodes(),
            1000 * self.stats.total_nodes() / (self.timer.elapsed() + 1),
            pv
        );
        if pv.len() > 0 {
            Some(
                Uci::from_str(pv.split(" ").collect::<Vec<&str>>()[0])
                    .unwrap()
                    .to_move(board)
                    .unwrap(),
            )
        } else {
            None
        }
    }

    // constants
    pub const NULL_MIN_DEPTH: Depth = 2;
    const ASPIRATION_WINDOW: i32 = 25;

    const LMR_MOVE_WO_REDUCTION: usize = 2;
    const LMR_MIN_DEPTH: Depth = 3;
    const LMR_BASE_REDUCTION: f32 = 0.75;
    const LMR_MOVE_DIVIDER: f32 = 2.25;
}

lazy_static! {
    pub static ref LMR_TABLE: [[Depth; 64]; 64] = init_lmr_table();
}

fn init_lmr_table() -> [[Depth; 64]; 64] {
    let mut lmr_table = [[0; 64]; 64];
    for depth in 1..64 {
        for move_number in 1..64 {
            lmr_table[depth][move_number] = (Search::LMR_BASE_REDUCTION
                + f32::ln(depth as f32) * f32::ln(move_number as f32) / Search::LMR_MOVE_DIVIDER)
                as Depth;
        }
    }
    lmr_table
}

#[cfg(test)]
mod tests {
    use crate::weight::{is_checkmate, INF_SCORE};

    #[test]
    fn is_mate() {
        let score = 66666;
        assert!(!is_checkmate(score));

        let score = INF_SCORE - 20;
        assert!(is_checkmate(score));
    }
}
