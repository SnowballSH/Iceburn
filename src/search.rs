use std::cmp::Reverse;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use search_constants::*;

use crate::board::*;
use crate::moves::Move;
use crate::tt::{TTEntry, TTFlag, TranspositionTable};
use crate::utils::u8_v_to_s;
use crate::weight::MATERIAL_OPENING;

pub mod search_constants {
    use crate::board::PieceType;
    use crate::weight::MATERIAL_OPENING;

    pub const MATE_SCORE: i32 = 48000;
    pub const MATE_VALUE: i32 = 49000;
    pub const INFINITY_VALUE: i32 = 50000;
    pub const MAX_PLY: usize = 100;

    #[rustfmt::skip]
    pub const MVV_LVA: [i32; 13 * 13] = [
        0,   0,   0,   0,   0,   0,   0,  0,   0,   0,   0,   0,   0,
        0, 105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605,
        0, 104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604,
        0, 103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603,
        0, 102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602,
        0, 101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601,
        0, 100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600,

        0, 105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605,
        0, 104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604,
        0, 103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603,
        0, 102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602,
        0, 101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601,
        0, 100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600
    ];

    pub const FUTILITY_MARGIN: [i32; 4] = [
        0,
        MATERIAL_OPENING[PieceType::Pawn as usize],
        MATERIAL_OPENING[PieceType::Knight as usize],
        MATERIAL_OPENING[PieceType::Rook as usize],
    ];
}

#[derive(Clone, Debug)]
pub struct TimeMan {
    pub start: Instant,
    pub amount: Duration,
}

impl TimeMan {
    pub fn is_time_up(&self) -> bool {
        self.start.elapsed() > self.amount
    }
}

#[derive(Debug)]
pub struct Searcher {
    pub timeman: Option<TimeMan>,
    pub stop_search: Mutex<bool>,
    pub nodes: usize,
    pub tthit: usize,

    pub search_ply: usize,

    pub killers: [Option<Move>; 2 * MAX_PLY],
    pub histories: [i32; 13 * 128],

    pub pv: [Option<Move>; (MAX_PLY * MAX_PLY) as usize],
    pub pv_indexing: [Option<usize>; MAX_PLY as usize],
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            timeman: None,
            stop_search: Mutex::new(false),
            nodes: 0,
            tthit: 0,
            search_ply: 0,
            killers: [None; 2 * MAX_PLY],
            histories: [0; 13 * 128],
            pv: [None; (MAX_PLY * MAX_PLY) as usize],
            pv_indexing: [None; MAX_PLY as usize],
        }
    }
}

impl Searcher {
    pub fn clear(&mut self) {
        if let Some(tm) = &mut self.timeman {
            tm.start = Instant::now();
        }
        self.nodes = 0;
        self.tthit = 0;
        self.search_ply = 0;
        self.stop_search = Mutex::new(false);
        self.killers = [None; 2 * MAX_PLY];
        self.histories = [0; 13 * 128];
        self.pv = [None; (MAX_PLY * MAX_PLY) as usize];
        self.pv_indexing = [None; MAX_PLY as usize];
    }

    pub fn check_time(&mut self) {
        if !*self.stop_search.lock().unwrap() {
            *self.stop_search.lock().unwrap() = if let Some(t) = &self.timeman {
                t.is_time_up()
            } else {
                false
            };
        }
    }

    pub fn move_importance(&self, m: Move, board: &Board) -> i32 {
        let importance;

        if m.is_capture() {
            importance = MVV_LVA[board.board[m.source().usize()].usize() * 13
                + board.board[m.target().usize()].usize()]
                + 10000;
        } else {
            if self.killers[self.search_ply] == Some(m) {
                importance = 9000;
            } else if self.killers[self.search_ply + MAX_PLY] == Some(m) {
                importance = 8000;
            } else {
                importance = self.histories[board.board[m.source().usize()].usize() * 128
                    + board.board[m.target().usize()].usize()];
            }
        }

        importance
    }

    pub fn sort_move(&mut self, board: &Board, list: &mut Vec<Move>) {
        list.sort_unstable_by_key(|x| Reverse(self.move_importance(*x, board)))
    }

    pub fn store_pv(&mut self, m: Move) {
        self.pv[self.search_ply * 64 + self.search_ply] = Some(m);
        for p in self.search_ply + 1..self.pv_indexing[self.search_ply + 1].unwrap_or(0) {
            self.pv[self.search_ply * 64 + p] = self.pv[self.search_ply * 64 + p + 64];
        }
        self.pv_indexing[self.search_ply] = self.pv_indexing[self.search_ply + 1];
    }

    /// Capture-only search
    pub fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> Option<i32> {
        self.pv_indexing[self.search_ply] = Some(self.search_ply);
        self.nodes += 1;

        if self.nodes & 2047 == 0 {
            self.check_time();
            if *self.stop_search.lock().unwrap() {
                return None;
            }
        }

        if self.search_ply > MAX_PLY - 1 {
            return Some(board.eval());
        }

        let evaluation = board.eval();

        if evaluation >= beta {
            return Some(beta);
        }
        if evaluation > alpha {
            alpha = evaluation;
        }

        let mut capture_moves = board.gen_captures();
        self.sort_move(board, &mut capture_moves);

        for m in capture_moves {
            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }

            self.search_ply += 1;

            let res = self.quiescence(board, -beta, -alpha);
            board.undo_move();
            self.search_ply -= 1;

            if *self.stop_search.lock().unwrap() {
                return None;
            }

            if res.is_none() {
                panic!("None returned in negamax");
            }

            let score = Some(-res.unwrap());

            let s = score.unwrap();
            if s > alpha {
                self.store_pv(m);
                alpha = s;

                if s >= beta {
                    return Some(beta);
                }
            }
        }

        Some(alpha)
    }

    /// Negamax Alpha-Beta search.
    pub fn negamax(
        &mut self,
        board: &mut Board,
        tt: Arc<Mutex<TranspositionTable>>,
        mut alpha: i32,
        mut beta: i32,
        mut depth: i32,
        null_move: bool,
    ) -> Option<i32> {
        self.pv_indexing[self.search_ply] = Some(self.search_ply);

        let mut tt_bestmove = Move(0);
        let mut tt_flag = TTFlag::Alpha;
        let mut score = None;
        let pv = beta - alpha > 1;
        let mut futility = false;

        if self.search_ply != 0 {
            let res = tt.lock().unwrap().get(
                alpha,
                beta,
                &mut tt_bestmove,
                depth,
                &board.gen_hash(),
                self.search_ply,
            );
            if res.is_some() {
                self.tthit += 1;
            }
            score = res;
            if score.is_some() && pv == false {
                return score;
            }
        }

        if self.nodes & 2047 == 0 {
            self.check_time();
            if *self.stop_search.lock().unwrap() {
                return None;
            }
        }

        if board.fifty_move >= 100 {
            return Some(0);
        }

        if depth == 0 {
            self.nodes += 1;
            return self.quiescence(board, alpha, beta);
        }

        if alpha < -MATE_VALUE {
            alpha = -MATE_VALUE;
        }
        if beta > MATE_VALUE - 1 {
            beta = MATE_VALUE - 1;
        }
        if alpha > beta {
            return Some(alpha);
        }

        let checked = board.is_checked(board.turn);

        if checked {
            depth += 1;
        }

        if !checked && !pv {
            let evaluation = board.eval();

            if depth < 3 && (beta - 1).abs() > -MATE_VALUE + 100 {
                let eval_margin = 120 * depth;
                if evaluation - eval_margin >= beta {
                    return Some(evaluation - eval_margin);
                }
            }

            if null_move {
                // null move pruning
                if self.search_ply != 0 && depth > 2 && evaluation >= beta {
                    board.make_null_move();
                    let res = self.negamax(board, tt.clone(), -beta, -beta + 1, depth - 3, false);
                    board.undo_null_move();
                    if *self.stop_search.lock().unwrap() {
                        return None;
                    }

                    if res.is_none() {
                        panic!("None returned in negamax");
                    }

                    score = Some(-res.unwrap());

                    if score.unwrap() >= beta {
                        return Some(beta);
                    }
                }

                // razoring
                if depth < 4 {
                    score = Some(evaluation + MATERIAL_OPENING[PieceType::Pawn as usize]);
                    let new_score;

                    if score.unwrap_or(0) < beta && depth == 1 {
                        new_score = self.quiescence(board, alpha, beta);
                        return if new_score.unwrap_or(0) > score.unwrap_or(0) {
                            new_score
                        } else {
                            score
                        };
                    }

                    score = Some(score.unwrap_or(0) + MATERIAL_OPENING[PieceType::Pawn as usize]);

                    if score.unwrap_or(0) < beta && depth < 4 {
                        new_score = self.quiescence(board, alpha, beta);
                        return if new_score.unwrap_or(0) > score.unwrap_or(0) {
                            new_score
                        } else {
                            score
                        };
                    }
                }
            }

            // futility pruning
            if depth < 4
                && alpha.abs() < MATE_SCORE
                && evaluation + FUTILITY_MARGIN[depth as usize] <= alpha
            {
                futility = true;
            }
        }

        let mut moves = board.gen_moves();
        let mut legals: usize = 0;
        let mut searched: usize = 0;

        self.sort_move(board, &mut moves);

        for m in moves {
            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }

            self.search_ply += 1;
            legals += 1;

            // futility pruning
            if futility
                && searched != 0
                && !m.is_capture()
                && m.promote() == Piece::EP
                && !board.is_checked(board.turn)
            {
                board.undo_move();
                self.search_ply -= 1;
                continue;
            }

            let res;

            if searched == 0 {
                res = self.negamax(board, tt.clone(), -beta, -alpha, depth - 1, true)
            } else {
                // LMR pruning
                if pv == false
                    && searched > 3
                    && depth > 2
                    && !checked
                    && (m.source() != self.killers[self.search_ply].unwrap_or(Move(0)).source()
                        || m.target() != self.killers[self.search_ply].unwrap_or(Move(0)).target())
                    && (m.source()
                        != self.killers[self.search_ply + MAX_PLY]
                            .unwrap_or(Move(0))
                            .source()
                        || m.target()
                            != self.killers[self.search_ply + MAX_PLY]
                                .unwrap_or(Move(0))
                                .target())
                    && !m.is_capture()
                    && m.promote() == Piece::EP
                {
                    res = self.negamax(board, tt.clone(), -alpha - 1, -alpha, depth - 2, true)
                } else {
                    res = Some(-(alpha + 1))
                }
            };

            if *self.stop_search.lock().unwrap() {
                return None;
            }

            if res.is_none() {
                panic!("None returned in negamax");
            }

            score = Some(-res.unwrap());

            // Principle-variation search
            if searched != 0 {
                if score.unwrap() > alpha {
                    score = Some(
                        -self
                            .negamax(board, tt.clone(), -alpha - 1, -alpha, depth - 1, true)
                            .unwrap_or(0),
                    );
                    if score.unwrap() > alpha && score.unwrap() < beta {
                        score = Some(
                            -self
                                .negamax(board, tt.clone(), -beta, -alpha, depth - 1, true)
                                .unwrap_or(0),
                        );
                    }
                }
            }

            let s = score.unwrap();

            board.undo_move();
            searched += 1;
            self.search_ply -= 1;

            if s > alpha {
                tt_flag = TTFlag::Exact;
                alpha = s;

                self.store_pv(m);

                if m.is_capture() {
                    self.histories[board.board[m.source().usize()].usize() * 128
                        + board.board[m.target().usize()].usize()] += depth;
                }

                if s >= beta {
                    tt.lock().unwrap().insert(
                        board.gen_hash(),
                        TTEntry {
                            depth,
                            flag: TTFlag::Beta,
                            score: beta,
                            move_: tt_bestmove,
                        },
                        self.search_ply,
                    );

                    if m.is_capture() {
                        self.killers[MAX_PLY + self.search_ply] = self.killers[self.search_ply];
                        self.killers[self.search_ply] = Some(m);
                    }

                    return Some(beta);
                }
            }
        } // move iteration

        if legals == 0 {
            return if checked {
                Some(-MATE_VALUE + self.search_ply as i32)
            } else {
                Some(0)
            };
        }

        tt.lock().unwrap().insert(
            board.gen_hash(),
            TTEntry {
                depth,
                flag: tt_flag,
                score: alpha,
                move_: tt_bestmove,
            },
            self.search_ply,
        );

        score
    }

    /// helper function for searching
    pub fn search(&mut self, board: &mut Board, tt: Arc<Mutex<TranspositionTable>>, depth: i32) {
        let start = Instant::now();
        let mut bestmove = None;

        self.clear();

        // MTD(f)
        for mtdf in 1..=depth {
            bestmove = self.pv[0];

            let res = self.negamax(
                board,
                tt.clone(),
                -INFINITY_VALUE,
                INFINITY_VALUE,
                mtdf,
                false,
            );

            self.check_time();

            if *self.stop_search.lock().unwrap() {
                break;
            }

            if res.is_none() {
                panic!("None returned in negamax");
            }

            let score = -res.unwrap();

            let info_str = format!(
                "info score cp {} depth {} nodes {} time {} tthit {} pv {}",
                score,
                mtdf,
                self.nodes,
                start.elapsed().as_millis(),
                self.tthit,
                (0..self.pv_indexing[0].unwrap())
                    .map(|x| u8_v_to_s(self.pv[x].unwrap().to_uci()))
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            println!("{}", info_str);

            if score >= MATE_SCORE && score <= MATE_VALUE {
                break;
            }
        }

        let bestmove_overall = if *self.stop_search.lock().unwrap() {
            bestmove
        } else {
            self.pv[0]
        };

        println!("bestmove {}", u8_v_to_s(bestmove_overall.unwrap().to_uci()));
    }
}
