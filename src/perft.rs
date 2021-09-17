use crate::board::Board;
use std::time::Instant;
use crate::utils::u8_v_to_s;

#[derive(Clone, Debug)]
pub struct Perft {
    pub nodes: usize,
    pub board: Board,
}

impl Perft {
    pub fn new() -> Self {
        Perft {
            nodes: 0,
            board: Board::default()
        }
    }

    pub fn driver(&mut self, depth: usize) {
        if depth == 0 {
            self.nodes += 1;
            return;
        }

        let moves = self.board.gen_moves();

        for m in moves {
            let res = self.board.make_move(m);
            if !res {
                continue;
            }

            self.driver(depth - 1);
            self.board.take_back();
        }
    }

    pub fn test(&mut self, depth: usize) {
        self.nodes = 0;
        println!("Perft");
        let start = Instant::now();

        let moves = self.board.gen_moves();

        for m in moves {
            let res = self.board.make_move(m);
            if !res {
                continue;
            }

            let prev = self.nodes;

            self.driver(depth - 1);
            self.board.take_back();

            let taken = self.nodes - prev;
            println!("move {} human {} nodes {}",
                     u8_v_to_s(m.to_uci()),
                     u8_v_to_s(m.to_human()), taken);
        }

        println!("\nFinished\nDepth {}\nNodes {}\nTime {:?}", depth, self.nodes, start.elapsed());
    }
}
