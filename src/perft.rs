use std::time::Instant;

use shakmaty::uci::Uci;
use shakmaty::{Chess, Position};

#[derive(Clone, Debug)]
pub struct Perft {
    pub nodes: usize,
}

impl Perft {
    pub fn new() -> Self {
        Perft { nodes: 0 }
    }

    pub fn clone_driver(&mut self, depth: usize, board: Chess) {
        let leaf = depth == 2;

        let moves = board.legal_moves();

        for m in moves {
            let mut nb = board.clone();
            nb.play_unchecked(&m);

            if leaf {
                self.nodes += nb.legal_moves().len();
            } else {
                self.clone_driver(depth - 1, nb);
            };
        }
    }

    pub fn test(&mut self, depth: usize) {
        self.nodes = 0;
        println!("Perft");
        let start = Instant::now();

        let board = Chess::default();

        let leaf = depth == 2;

        let moves = board.legal_moves();

        for m in moves {
            let mut nb = board.clone();
            nb.play_unchecked(&m);

            let prev = self.nodes;

            if leaf {
                self.nodes += board.legal_moves().len();
            } else {
                self.clone_driver(depth - 1, nb);
            }

            let taken = self.nodes - prev;
            println!("move {} nodes {}", Uci::from_standard(&m), taken);
        }

        println!(
            "\nFinished\nDepth {}\nNodes {}\nTime {:?}",
            depth,
            self.nodes,
            start.elapsed()
        );
    }
}
