// Inspired by
// https://github.com/Recursing/sunfish_rs/blob/master/src/search.rs
// The sunfish rust port's search method

use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

use chess::{Board, BoardStatus, ChessMove, MoveGen, Piece};
use serde_json;

use crate::eval::*;

const TRANSPOSITION_TABLE_SIZE: usize = 15_000_000;
const QUIESCENCE_SEARCH_LIMIT: i32 = 130;
const EVAL_ROUGHNESS: i32 = 10;
const STOP_SEARCH: i32 = MATE_UPPER * 101;

#[derive(Clone, Copy)]
pub struct Entry {
    lower: i32,
    upper: i32,
}

const DEFAULT_ENTRY: Entry = Entry {
    lower: -MATE_UPPER,
    upper: MATE_UPPER,
};

pub struct Searcher {
    pub scores: HashMap<(u64, i32, bool), Entry>,
    pub moves: HashMap<u64, ChessMove>,
    pub nodes: u32,
    now: Instant,
    duration: Duration,
}

impl Default for Searcher {
    fn default() -> Self {
        let mut s = Searcher {
            scores: HashMap::with_capacity(TRANSPOSITION_TABLE_SIZE),
            moves: HashMap::with_capacity(TRANSPOSITION_TABLE_SIZE),
            nodes: 0,
            now: Instant::now(),
            duration: Duration::new(4, 0),
        };

        let json = std::fs::read_to_string("./openings.json");

        if let Ok(val) = json {
            if let serde_json::Value::Object(mp) = serde_json::from_str(&*val).unwrap() {
                mp.iter().for_each(|x| {
                    s.scores.insert((Board::from_str(x.0.as_str()).unwrap().get_hash(), 40, true), Entry {
                        lower: x.1.as_i64().unwrap() as i32,
                        upper: x.1.as_i64().unwrap() as i32,
                    });
                });
            } else {
                eprintln!("File not in right format")
            }
        } else {
            eprintln!("Unable to load opening file: openings.json");
        }

        s
    }
}

impl Searcher {
    fn q(&mut self, board_state: Board, gamma: i32, depth: i32, root: bool) -> i32 {
        self.nodes += 1;

        let ps = board_state;
        let hs = ps.get_hash();

        let entry = *self
            .scores
            .get(&(hs, depth.max(0), root))
            .unwrap_or(&DEFAULT_ENTRY);

        if entry.lower >= gamma && (
            !root || self.moves.get(&hs).is_some()
        ) {
            return entry.lower;
        } else if entry.upper < gamma {
            return entry.upper;
        }

        if self.now.elapsed() > self.duration {
            return STOP_SEARCH;
        }

        let mut best = -MATE_UPPER;

        // Null Move
        if depth > 0
            && !root
            && (
            ps.color_combined(board_state.side_to_move()) & (
                ps.pieces(Piece::Bishop) |
                    ps.pieces(Piece::Knight) |
                    ps.pieces(Piece::Rook) |
                    ps.pieces(Piece::Queen)
            )).0 != 0 {
            let nb = ps.null_move();
            if let Some(x) = nb {
                let score = -self.q(
                    x, 1 - gamma,
                    depth - 3, false);
                if score == -STOP_SEARCH {
                    return STOP_SEARCH;
                }
                best = best.max(score);
            }
        } else if depth <= 0 {
            let score = eval(board_state);
            best = best.max(score);
        }

        // Killer move
        if best <= gamma {
            if let Some(killer_move) = self.moves.get(&hs).copied() {
                let nb = board_state.make_move_new(killer_move);
                if depth > 0 || eval(nb.clone()) >= QUIESCENCE_SEARCH_LIMIT {
                    let score = -self.q(
                        nb,
                        1 - gamma,
                        depth - 1,
                        false,
                    );
                    if score == -STOP_SEARCH {
                        return STOP_SEARCH;
                    }
                    best = std::cmp::max(best, score);
                    // self.move_transposition_table.insert(*board_state, killer_move);
                }
            }
        }

        if best < gamma {
            let moves = MoveGen::new_legal(&ps);
            // move ordering
            let mut move_vals: Vec<_> = moves
                .map(|m| {
                    let nb = board_state.make_move_new(m);
                    (-eval(nb), m)
                })
                .collect();
            move_vals.sort_unstable();

            for (val, m) in move_vals {
                if depth > 0
                    || (
                    -val >= QUIESCENCE_SEARCH_LIMIT
                        &&
                        eval(board_state.clone()) - val > best) {
                    let nb = board_state.make_move_new(m);
                    let score =
                        -self.q(nb, 1 - gamma, depth - 1, false);
                    if score == -STOP_SEARCH {
                        return STOP_SEARCH;
                    }
                    best = best.max(score);
                    if best >= gamma {
                        // Save the move for pv construction and killer heuristic
                        if self.moves.len() >= TRANSPOSITION_TABLE_SIZE {
                            self.moves.clear();
                        }
                        self.moves.insert(hs, m);
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        if best < gamma && best < 0 && depth > 0 {
            if board_state.status() == BoardStatus::Stalemate {
                best = 0
            }
        }

        if best >= gamma {
            self.scores.insert(
                (hs, depth, root),
                Entry {
                    lower: best,
                    upper: entry.upper,
                },
            );
        } else if best < gamma {
            self.scores.insert(
                (hs, depth, root),
                Entry {
                    lower: entry.lower,
                    upper: best,
                },
            );
        }

        best
    }

    pub fn search(
        &mut self,
        board_state: Board,
        duration: Duration,
        max_depth: i32,
    ) -> ((ChessMove, i32, i32), u32, Duration) {
        self.nodes = 0;
        let mut reached_depth;
        self.now = Instant::now();
        self.duration = duration;
        let mut last_move = (ChessMove::from_str("e2e4").unwrap(), 0, 0);

        for depth in 1..max_depth {
            let mut lower = -MATE_UPPER;
            let mut upper = MATE_UPPER;
            while lower < upper - EVAL_ROUGHNESS {
                let gamma = (lower + upper + 1) / 2;
                let score = self.q(board_state.clone(), gamma, depth, true);
                if score == STOP_SEARCH {
                    lower = STOP_SEARCH;
                    break;
                }
                if score >= gamma {
                    lower = score;
                } else {
                    upper = score;
                }
            }
            if lower == STOP_SEARCH {
                break;
            }
            let score = self.q(board_state.clone(), lower, depth, true);
            if score == STOP_SEARCH {
                break;
            }
            reached_depth = depth;
            println!(
                "Reached depth {: <2} score {: <5} nodes {: <7} time {:?}",
                depth,
                score,
                self.nodes,
                self.now.elapsed()
            );

            last_move = (
                *self.moves
                    .get(&board_state.get_hash())
                    .expect("move not in table"),
                self.scores
                    .get(&(board_state.get_hash(), reached_depth, true))
                    .expect("score not in table")
                    .lower,
                reached_depth,
            );

            if self.now.elapsed() > self.duration || score <= -MATE_LOWER || score == MATE_UPPER {
                break;
            }
        }

        (last_move, self.nodes, self.now.elapsed())
    }
}
