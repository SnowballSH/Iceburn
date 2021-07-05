// Inspired by
// https://github.com/Recursing/sunfish_rs/blob/master/src/search.rs
// The sunfish rust port's search method

use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

use chess::{Board, BoardStatus, ChessMove, MoveGen, Piece};

use crate::eval::*;
use crate::search::entry::{TTEntry, TTEntryFlag, TT};

#[derive(Clone, Debug)]
pub struct Searcher {
    pub max_depth: i32,
    pub infinity: i32,
    pub null_min_depth: i32,
    pub lmr_min_depth: i32,
    pub lmr_moves_reduction: usize,
    pub window: i32,

    pub stop: bool,

    pub id_move: Option<ChessMove>,
    pub id_score: i32,
    pub sel_depth: i32,
    lmr_table: [[i32; 64]; 64],

    pub nodes: usize,
    now: Instant,
    duration: Duration,

    pub tt: TT,
}

impl Searcher {
    pub fn new(max_depth: i32) -> Self {
        let mut lmr = [[0; 64]; 64];

        for depth in 1..64 {
            for number in 1..64 {
                lmr[depth][number] =
                    (0.75 + (depth as f32).ln() * (number as f32).ln() / 2.25) as i32;
            }
        }

        let s = Self {
            max_depth,
            infinity: i32::MAX - 50,
            null_min_depth: 2,
            lmr_min_depth: 2,
            lmr_moves_reduction: 1,
            window: 25,
            stop: false,
            id_move: None,
            id_score: 0,
            sel_depth: 0,
            lmr_table: lmr,
            nodes: 0,
            now: Instant::now(),
            duration: Default::default(),
            tt: HashMap::with_capacity(3_000_000),
        };

        s
    }

    pub fn iterative_deepening(&mut self, board: Board, time: Duration) {
        self.now = Instant::now();
        self.duration = time;
        self.nodes = 0;
        self.stop = false;
        self.id_move = None;
        self.id_score = 0;
        self.sel_depth = 0;

        let mut alpha = -self.infinity;
        let mut beta = self.infinity;
        let mut depth = 1;

        while depth <= self.max_depth {
            self.nodes = 0;

            if self.now.elapsed() > self.duration || self.is_checkmate(self.id_score) {
                break;
            }

            self.negamax_root(board, depth, alpha, beta);

            if self.id_score <= alpha {
                alpha = -self.infinity;
            } else if self.id_score >= beta {
                beta = self.infinity;
            } else {
                println!(
                    "info currmove {} depth {} seldepth {} time {} score cp {} nodes {} pv",
                    self.id_move.unwrap(),
                    depth,
                    self.sel_depth,
                    self.now.elapsed().as_millis(),
                    self.id_score,
                    self.nodes
                );
                alpha = self.id_score - self.window;
                beta = self.id_score + self.window;
                depth += 1;
            }
        }
    }

    fn negamax_root(&mut self, board: Board, mut depth: i32, mut alpha: i32, beta: i32) {
        let mut value;
        let moves = MoveGen::new_legal(&board);

        let in_check = board.checkers().0 != 0;
        if in_check {
            depth += 1;
        }

        // return if there is only 1 legal move
        if moves.len() == 1 {
            self.id_move = moves.collect::<Vec<ChessMove>>().first().cloned();
            self.id_score = 0;
            return;
        }

        let mut best_move = None;

        // TODO MOVE ORDERING

        for m in moves {
            let nb = board.make_move_new(m);
            value = -self.negamax(nb, depth - 1, 1, -beta, -alpha, true);

            if self.stop || self.now.elapsed() > self.duration {
                self.stop = true;
                break;
            }

            if value > alpha {
                best_move = Some(m);
                if value >= beta {
                    self.tt.insert(
                        board.get_hash(),
                        TTEntry {
                            score: beta,
                            depth,
                            flag: TTEntryFlag::Lower,
                            best_move: best_move.unwrap(),
                        },
                    );
                    self.id_move = best_move;
                    self.id_score = beta;
                    return;
                }
                alpha = value;
                self.tt.insert(
                    board.get_hash(),
                    TTEntry {
                        score: alpha,
                        depth,
                        flag: TTEntryFlag::Upper,
                        best_move: best_move.unwrap(),
                    },
                );
            }
        }

        if best_move.is_none() {
            best_move = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>().first().cloned();
        }

        if !self.stop {
            self.tt.insert(
                board.get_hash(),
                TTEntry {
                    score: alpha,
                    depth,
                    flag: TTEntryFlag::Exact,
                    best_move: best_move.unwrap(),
                },
            );
            self.id_move = best_move;
            self.id_score = alpha;
        }
    }

    fn negamax(
        &mut self,
        board: Board,
        depth: i32,
        ply: i32,
        mut alpha: i32,
        mut beta: i32,
        can_null: bool,
    ) -> i32 {
        let mate = self.infinity - ply;
        let in_check;
        let mut flag = TTEntryFlag::Upper;
        let mut reduced_depth;

        if self.stop || self.now.elapsed() > self.duration {
            self.stop = true;
            return 0;
        }

        if alpha < -mate {
            alpha = -mate;
        }
        if beta > mate - 1 {
            beta = mate - 1;
        }
        if alpha >= beta {
            return alpha;
        }

        in_check = board.checkers().0 != 0;

        if depth <= 0 && !in_check {
            return self.q(board, depth, ply, alpha, beta);
        }

        self.nodes += 1;

        // TODO repetition

        let entry = self.tt.get(&board.get_hash());

        if let Some(entry) = entry {
            if entry.depth >= depth {
                match entry.flag {
                    TTEntryFlag::Exact => {
                        return entry.score;
                    }
                    TTEntryFlag::Upper => {
                        alpha = alpha.max(entry.score);
                    }
                    TTEntryFlag::Lower => {
                        beta = beta.min(entry.score);
                    }
                }
            }
        }

        if can_null
            && !in_check
            && depth >= self.null_min_depth
            && (board.color_combined(board.side_to_move())
                & (board.pieces(Piece::Bishop)
                    | board.pieces(Piece::Knight)
                    | board.pieces(Piece::Rook)
                    | board.pieces(Piece::Queen)))
            .0 != 0
            && eval(board, false) >= beta
        {
            let r = if depth > 6 { 3 } else { 2 };
            if let Some(nb) = board.null_move() {
                let value = -self.negamax(nb, depth - r - 1, ply, -beta, -beta + 1, false);
                if self.stop {
                    return 0;
                }
                if value >= beta {
                    return beta;
                }
            }
        }

        let moves = MoveGen::new_legal(&board);
        let mut value = 0;
        let mut best_move = None;

        // TODO ORDER MOVES

        for (i, m) in moves.enumerate() {
            reduced_depth = depth;

            if depth > self.lmr_min_depth
                && i > self.lmr_moves_reduction
                && board.piece_on(m.get_dest()).is_none()
            {
                reduced_depth -= self.lmr_table[depth.min(63) as usize][i.min(63) as usize];
            }

            if in_check {
                reduced_depth += 1;
            }

            let nb = board.make_move_new(m);
            value = -self.negamax(nb, reduced_depth - 1, ply + 1, -beta, -alpha, true);

            if self.stop {
                return 0;
            }

            if value > alpha {
                best_move = Some(m);
                if value >= beta {
                    flag = TTEntryFlag::Lower;
                    alpha = beta;
                    break;
                }

                flag = TTEntryFlag::Exact;
                alpha = value;
            }
        }

        if board.status() == BoardStatus::Stalemate {
            alpha = 0;
        } else if board.status() == BoardStatus::Checkmate {
            alpha = -mate;
        }

        if let Some(x) = best_move {
            if !self.stop {
                self.tt.insert(
                    board.get_hash(),
                    TTEntry {
                        score: alpha,
                        depth,
                        flag,
                        best_move: x,
                    },
                );
            }
        }

        alpha
    }

    fn q(&mut self, board: Board, depth: i32, ply: i32, mut alpha: i32, beta: i32) -> i32 {
        if self.stop || self.now.elapsed() > self.duration {
            self.stop = true;
            return 0;
        }

        self.sel_depth = ply.max(self.sel_depth);

        let mut value = eval(board, true);

        if value >= beta {
            return beta;
        }

        if alpha < value {
            alpha = value
        }

        let mut moves = MoveGen::new_legal(&board);
        let targets = board.color_combined(!board.side_to_move());
        moves.set_iterator_mask(*targets);

        for m in moves {
            let nb = board.make_move_new(m);
            value = -self.q(nb, depth - 1, ply + 1, -beta, -alpha);

            if self.stop {
                return 0;
            }

            if value > alpha {
                if value >= beta {
                    return beta;
                }
                alpha = value;
            }
        }

        alpha
    }

    #[inline]
    fn is_checkmate(&self, score: i32) -> bool {
        score.abs() >= self.infinity / 2
    }
}
