use std::collections::HashMap;

use chess::{ChessMove, Color, Game, MoveGen, Board};
use rand::Rng;

use std::str::FromStr;

use crate::eval::{eval, MATE_LOWER, MATE_UPPER};

#[derive(Copy, Clone)]
struct NodeResult {
    depth: usize,
    value: i32,
}

#[derive(Clone)]
struct Table {
    table: HashMap<u64, NodeResult>,
}

static OF: i32 = 5;

impl Table {
    fn new() -> Self {
        let mut t = Table {
            table: HashMap::with_capacity(1000000),
        };

        // Queen's pawn
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            47, 15 * OF);

        // King's pawn
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            49, 15 * OF);

        // English
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/2P5/8/PP1PPPPP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            47, 15 * OF);

        // Van't Kruijs Opening
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            48, 0 * OF);

        // Mieses Opening
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            42, -10 * OF);

        // Reti Opening
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1")
                .unwrap().get_hash(),
            47, 15 * OF);

        // Bird Opening
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/5P2/8/PPPPP1PP/RNBQKBNR b KQkq - 0 1")
                .unwrap().get_hash(),
            42, -40 * OF);

        // Van Geet Opening
        t.set(
            Board::from_str("rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b KQkq - 1 1")
                .unwrap().get_hash(),
            43, -20 * OF);

        // Queen's pawn, d5
        t.set(
            Board::from_str("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2")
                .unwrap().get_hash(),
            46, 30 * OF);

        // Queen's pawn, Horwitz Defense
        t.set(
            Board::from_str("rnbqkbnr/pppp1ppp/4p3/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2")
                .unwrap().get_hash(),
            45, 25 * OF);

        // Queen's pawn, Nc6
        t.set(
            Board::from_str("r1bqkbnr/pppppppp/2n5/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 1 2")
                .unwrap().get_hash(),
            37, 60 * OF);

        // Queen's pawn, Nf6
        t.set(
            Board::from_str("rnbqkb1r/pppppppp/5n2/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 1 2")
                .unwrap().get_hash(),
            45, 20 * OF);

        // King's pawn, Sicilian Defense
        t.set(
            Board::from_str("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .unwrap().get_hash(),
            45, -5 * OF);

        // King's pawn, e5
        t.set(
            Board::from_str("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .unwrap().get_hash(),
            51, 10 * OF);

        // King's pawn, Caro-Kann Defense
        t.set(
            Board::from_str("rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .unwrap().get_hash(),
            47, 30 * OF);

        t
    }

    fn get(&self, hash: u64, depth: usize) -> Option<i32> {
        if depth <= 0 {
            return None;
        }
        self.table.get(&hash).filter(|val| val.depth >= depth).map(|val| val.value)
    }

    fn set(&mut self, hash: u64, depth: usize, value: i32) {
        let better_evaluation = self
            .table
            .get(&hash)
            .filter(|evaluation| evaluation.depth >= depth)
            .is_some();
        if !better_evaluation {
            let board_evaluation = NodeResult {
                depth,
                value,
            };
            self.table.insert(hash, board_evaluation);
        }
    }
}

pub struct Searcher {
    table: Table,
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            table: Table::new()
        }
    }
}

impl Searcher {
    fn alpha_beta_with_memory(&mut self, board: Game, depth: usize,
                              mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
        let store = self.table.get(board.current_position().get_hash(), depth);
        if let Some(store) = store {
            return store;
        }

        if depth == 0 || board.result().is_some() {
            return eval(board);
        }

        let move_gen = MoveGen::new_legal(&board.current_position());
        let mut best: i32 = if maximizing {
            MATE_LOWER
        } else {
            MATE_UPPER
        };

        for m in move_gen {
            let mut nb = board.clone();
            nb.make_move(m);

            let evaluation =
                self.alpha_beta_with_memory(
                    nb, depth - 1, alpha, beta, !maximizing,
                );
            if maximizing {
                best = best.max(evaluation);
                alpha = alpha.max(best);
            } else {
                best = best.min(evaluation);
                beta = beta.min(best);
            }
            if beta <= alpha {
                break;
            }
        }

        self.table.set(board.current_position().get_hash(), depth, best);

        best
    }

    fn mtdf(&mut self, board: Game, f: i32, depth: usize, maxi: bool) -> i32 {
        let mut g = f;
        let mut upper = BETA;
        let mut lower = ALPHA;
        while lower < upper {
            let beta = if g == lower {
                g + 1
            } else {
                g
            };
            g = self.alpha_beta_with_memory(board.clone(), depth, beta-1, beta, maxi);
            if g < beta {
                upper = g;
            } else {
                lower = g;
            }
        }

        g
    }

    pub fn best_move(&mut self, board: Game, depth: usize, for_color: Color) -> Option<ChessMove> {
        let moves = MoveGen::new_legal(&board.current_position());

        if depth == 0 || board.result().is_some() {
            return None;
        }

        let maxi = for_color == Color::White;

        let mut pair: Option<(ChessMove, i32)> = None;
        for m in moves {
            let mut nb = board.clone();
            nb.make_move(m);

            let score = self.alpha_beta_with_memory(nb.clone(), depth, ALPHA, BETA, !maxi);

            // println!("Move {} Score {}", m.to_string(), score);
            if maxi {
                if pair.is_none() {
                    pair = Some((m, score));
                } else if score > pair.unwrap().1 {
                    pair = Some((m, score));
                } else if (score - pair.unwrap().1).abs() < 20 {
                    let mut rng = rand::thread_rng();
                    if rng.gen_range(0..2) == 0 {
                        pair = Some((m, score));
                    }
                }
            } else {
                if pair.is_none() {
                    pair = Some((m, score));
                } else if score < pair.unwrap().1 {
                    pair = Some((m, score));
                } else if (score - pair.unwrap().1).abs() < 20 {
                    let mut rng = rand::thread_rng();
                    if rng.gen_range(0..2) == 0 {
                        pair = Some((m, score));
                    }
                }
            }
        }

        if pair.is_none() {
            None
        } else {
            Some(pair.unwrap().0)
        }
    }
}

static ALPHA: i32 = i32::MIN + 8;
static BETA: i32 = i32::MAX - 8;
