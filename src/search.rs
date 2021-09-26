use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use search_constants::*;

use crate::board::{Board, Color};
use crate::moves::Move;
use crate::nnue::nnue_eval_fen;
use crate::tt::{TTEntry, TTFlag, TranspositionTable};
use crate::utils::u8_v_to_s;

pub mod search_constants {
    pub const MATE_SCORE: i32 = i32::MAX - 500;
    pub const MATE_VALUE: i32 = i32::MAX - 400;
    pub const INFINITY_VALUE: i32 = i32::MAX - 200;
    pub const MAX_PLY: usize = 100;
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
    pub search_ply: usize,

    pub pv: [Option<Move>; (MAX_PLY * MAX_PLY) as usize],
    pub pv_indexing: [Option<usize>; MAX_PLY as usize],
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            timeman: None,
            stop_search: Mutex::new(false),
            nodes: 0,
            search_ply: 0,
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
        self.search_ply = 0;
        self.stop_search = Mutex::new(false);
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

    pub fn store_pv(&mut self, m: Move) {
        self.pv[self.search_ply * 64 + self.search_ply] = Some(m);
        for p in self.search_ply + 1..self.pv_indexing[self.search_ply + 1].unwrap_or(0) {
            self.pv[self.search_ply * 64 + p] = self.pv[self.search_ply * 64 + p + 64];
        }
        self.pv_indexing[self.search_ply] = self.pv_indexing[self.search_ply + 1];
    }

    /// Negamax Alpha-Beta search.
    pub fn negamax(
        &mut self,
        board: &mut Board,
        tt: Arc<Mutex<TranspositionTable>>,
        mut alpha: i32,
        beta: i32,
        mut depth: i32,
    ) -> Option<i32> {
        self.pv_indexing[self.search_ply] = Some(self.search_ply);

        let mut tt_bestmove = Move(0);
        let mut tt_flag = TTFlag::Alpha;
        let mut score = None;
        let pv = beta - alpha > 1;

        if self.search_ply != 0 {
            let res = tt.lock().unwrap().get(
                alpha,
                beta,
                &mut tt_bestmove,
                depth,
                &board.gen_hash(),
                self.search_ply,
            );
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

        if depth == 0 {
            self.nodes += 1;
            return Some(
                nnue_eval_fen(&*board.fen()) * if board.turn == Color::White { 1 } else { -1 },
            );
        }

        let checked = board.is_checked(board.turn);

        if checked {
            depth += 1;
        }

        let moves = board.gen_moves();
        let mut legals: usize = 0;

        for m in moves {
            board.make_move(m);
            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }

            self.search_ply += 1;
            legals += 1;

            let res = self.negamax(board, tt.clone(), -beta, -alpha, depth - 1);
            board.undo_move();
            self.search_ply -= 1;

            if *self.stop_search.lock().unwrap() {
                return None;
            }

            if res.is_none() {
                panic!("None returned in negamax");
            }

            score = Some(-res.unwrap());

            let s = score.unwrap();
            if s > alpha {
                tt_flag = TTFlag::Exact;
                alpha = s;

                self.store_pv(m);

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

            let res = self.negamax(board, tt.clone(), -INFINITY_VALUE, INFINITY_VALUE, mtdf);

            self.check_time();

            if *self.stop_search.lock().unwrap() {
                break;
            }

            if res.is_none() {
                panic!("None returned in negamax");
            }

            let score = -res.unwrap();

            let info_str = format!(
                "info score cp {} depth {} nodes {} time {} pv {}",
                score,
                mtdf,
                self.nodes,
                start.elapsed().as_millis(),
                (0..self.pv_indexing[0].unwrap())
                    .map(|x| u8_v_to_s(self.pv[x].unwrap().to_uci()))
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            println!("{}", info_str);
        }

        let bestmove_overall = if *self.stop_search.lock().unwrap() {
            bestmove
        } else {
            self.pv[0]
        };

        println!("bestmove {}", u8_v_to_s(bestmove_overall.unwrap().to_uci()));
    }
}
