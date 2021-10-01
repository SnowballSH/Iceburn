use crate::board::Board;
use crate::moves::Move;
use crate::timeman::*;
use crate::tt::{TTEntry, TTFlag, TranspositionTable};
use crate::utils::u8_v_to_s;
use crate::weight::{is_checkmate, INF_SCORE};

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
}

impl<'a> Search<'a> {
    pub fn new(timer: Timer, tt: &'a mut TranspositionTable) -> Self {
        Search {
            stop: false,
            sel_depth: 0,
            timer,
            tt,
            stats: Statistics::default(),
        }
    }

    pub fn go(&mut self, board: &mut Board) -> (Move, i32) {
        let mut alpha = -INF_SCORE;
        let mut beta = INF_SCORE;
        let mut depth = 1;
        let mut final_move = Move(0);
        let mut final_score = 0;
        let mut last_score = 0;

        let moves = board.gen_moves();
        if moves.len() == 1 {
            return (moves[0], 0);
        }

        while !self.stop && self.timer.start_check(depth) && !is_checkmate(final_score) {
            let res = self.negamax_root(board, depth, alpha, beta);
            if self.stop {
                break;
            }

            final_move = res.0;
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
                self.print_info(board, depth, final_move, final_score);
                alpha = final_score - Self::ASPIRATION_WINDOW;
                beta = final_score + Self::ASPIRATION_WINDOW;
                depth += 1;
                self.stats = Statistics::default();
            }
        }
        (final_move, final_score)
    }

    pub fn negamax_root(
        &mut self,
        board: &mut Board,
        mut depth: Depth,
        mut alpha: i32,
        beta: i32,
    ) -> (Move, i32) {
        let moves = board.gen_moves();

        let in_check = board.is_checked(board.turn);
        if in_check {
            depth += 1;
        }

        let first_move = moves[0];

        let mut best_move = first_move;
        if moves.len() == 1 {
            return (best_move, 0);
        }

        let hs = board.gen_hash();

        let mut value;
        for m in moves {
            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }
            value = -self.negamax(board, depth - 1, 1, -beta, -alpha, true);
            board.undo_move();

            if self.stop || self.timer.stop_check() {
                self.stop = true;
                break;
            }

            if value > alpha {
                best_move = m;
                if value >= beta {
                    self.tt.insert(
                        hs,
                        TTEntry::construct(hs, beta, Some(best_move), depth, TTFlag::Lower),
                    );
                    return (best_move, beta);
                }
                alpha = value;
                self.tt.insert(
                    hs,
                    TTEntry::construct(hs, alpha, Some(best_move), depth, TTFlag::Upper),
                );
            }
        }

        if !self.stop {
            self.tt.insert(
                hs,
                TTEntry::construct(hs, alpha, Some(best_move), depth, TTFlag::Exact),
            );
        }
        (best_move, alpha)
    }

    pub fn negamax(
        &mut self,
        board: &mut Board,
        depth: Depth,
        ply: Ply,
        mut alpha: i32,
        mut beta: i32,
        can_apply_null: bool,
    ) -> i32 {
        if self.stop || self.timer.stop_check() {
            self.stop = true;
            return 0;
        }

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

        let in_check = board.is_checked(board.turn);
        if depth <= 0 && !in_check {
            return self.q_search(board, ply, alpha, beta);
        }
        self.stats.nodes += 1;

        if board.fifty_move >= 100 {
            self.stats.leafs += 1;
            return 0;
        }

        if let Some(tt_entry) = self.tt.get(board.gen_hash()) {
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
        }

        if Self::can_apply_null(board, depth, beta, in_check, can_apply_null) {
            let r = if depth > 6 { 3 } else { 2 };
            board.make_null_move();
            let value = -self.negamax(board, depth - r - 1, ply, -beta, -beta + 1, false);
            board.undo_null_move();
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
        let moves = board.gen_moves();
        let lmoves = moves.len();

        for m in moves {
            reduced_depth = depth;
            /*
            if Self::can_apply_lmr(&m, depth, idx) {
                reduced_depth -= Self::late_move_reduction(depth, idx);
            }
             */

            if in_check {
                reduced_depth += 1;
            }

            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }
            value = -self.negamax(board, reduced_depth - 1, ply + 1, -beta, -alpha, true);
            board.undo_move();

            if self.stop {
                return 0;
            }

            if value > alpha {
                best_move = Some(m);
                if value >= beta {
                    self.stats.beta_cutoffs += 1;
                    tt_flag = TTFlag::Lower;
                    alpha = beta;
                    break;
                }
                tt_flag = TTFlag::Exact;
                alpha = value;
            }
        }

        if lmoves == 0 {
            if in_check {
                alpha = -mate_value;
            } else {
                alpha = 0;
            }
        }

        if !self.stop {
            let hs = board.gen_hash();
            self.tt
                .insert(hs, TTEntry::construct(hs, alpha, best_move, depth, tt_flag));
        }
        alpha
    }

    pub fn q_search(&mut self, board: &mut Board, ply: Ply, mut alpha: i32, beta: i32) -> i32 {
        if self.stop || self.timer.stop_check() {
            self.stop = true;
            return 0;
        }

        self.sel_depth = self.sel_depth.max(ply);
        self.stats.qnodes += 1;

        let value = board.fast_fast_eval();

        if value >= beta {
            self.stats.qleafs += 1;
            return beta;
        }

        if alpha < value {
            alpha = value;
        }

        let mut value;

        let moves = board.gen_captures();
        for m in moves {
            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }
            value = -self.q_search(board, ply + 1, -beta, -alpha);
            board.undo_move();

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
    pub fn can_apply_null(
        board: &Board,
        depth: Depth,
        beta: i32,
        in_check: bool,
        can_apply_null: bool,
    ) -> bool {
        can_apply_null
            && !in_check
            && depth >= Self::NULL_MIN_DEPTH
            // && board.has_non_pawn_material
            && board.fast_fast_eval() >= beta
    }

    pub fn get_pv(&self, board: &mut Board, depth: Depth) -> String {
        if depth == 0 {
            return "".to_owned();
        }
        let hash_move;
        let tt_entry = self.tt.get(board.gen_hash());
        match tt_entry {
            Some(tt_entry) => {
                hash_move = tt_entry.bestmove;
                if hash_move == None {
                    return "".to_owned();
                }
            }
            None => {
                return "".to_owned();
            }
        }

        board.make_move(hash_move.unwrap());
        let pv = u8_v_to_s(hash_move.unwrap().to_uci()) + " " + &*self.get_pv(board, depth - 1);
        board.undo_move();

        pv
    }

    pub fn print_info(&self, board: &mut Board, depth: Depth, m: Move, score: i32) {
        println!(
            "info currmove {} depth {} seldepth {} time {} score cp {} nodes {} nps {} pv {}",
            u8_v_to_s(m.to_uci()),
            depth,
            self.sel_depth,
            self.timer.elapsed(),
            score,
            self.stats.total_nodes(),
            1000 * self.stats.total_nodes() / (self.timer.elapsed() + 1),
            self.get_pv(board, depth)
        );
    }

    // constants
    pub const NULL_MIN_DEPTH: Depth = 2;
    const ASPIRATION_WINDOW: i32 = 25;
}
