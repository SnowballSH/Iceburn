use std::time::Instant;

use crate::board::Board;
use crate::utils::u8_v_to_s;

#[derive(Clone, Debug)]
pub struct Perft {
    pub nodes: usize,
}

impl Perft {
    pub fn new() -> Self {
        Perft { nodes: 0 }
    }

    pub fn undo_driver(&mut self, depth: usize, board: &mut Board) {
        let leaf = depth == 2;

        let moves = board.gen_moves();

        for m in moves {
            board.make_move(m);

            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }

            if leaf {
                self.nodes += board.gen_legal_moves().len();
            } else {
                self.undo_driver(depth - 1, board);
            };

            board.undo_move();
        }
    }

    pub fn clone_driver(&mut self, depth: usize, board: Board) {
        let leaf = depth == 2;

        let moves = board.gen_moves();

        for m in moves {
            let mut nb = board.clone();
            nb.make_move(m);

            if nb.is_checked(board.turn) {
                continue;
            }

            if leaf {
                self.nodes += nb.gen_legal_moves().len();
            } else {
                self.clone_driver(depth - 1, nb);
            };
        }
    }

    pub fn test(&mut self, depth: usize) {
        self.nodes = 0;
        println!("Perft");
        let start = Instant::now();

        let mut board = Board::default();

        let leaf = depth == 2;

        let moves = board.gen_moves();

        for m in moves {
            board.make_move(m);

            if board.is_checked(board.turn.not()) {
                board.undo_move();
                continue;
            }

            let prev = self.nodes;

            if leaf {
                self.nodes += board.gen_moves().len();
            } else {
                self.undo_driver(depth - 1, &mut board);
            }

            board.undo_move();

            let taken = self.nodes - prev;
            println!("move {} nodes {}", u8_v_to_s(m.to_uci()), taken);
        }

        println!(
            "\nFinished\nDepth {}\nNodes {}\nTime {:?}",
            depth,
            self.nodes,
            start.elapsed()
        );
    }
}
