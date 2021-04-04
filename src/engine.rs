use shakmaty::{Chess, Color, Outcome, Position, Role, Setup};
use shakmaty::uci::Uci;

pub fn eval_board(board: Chess) -> i16 {
    let mut score: i16 = 0;

    match board.outcome() {
        Some(o) => {
            score = match o {
                Outcome::Draw => 0,
                Outcome::Decisive {
                    winner: Color::White
                } => i16::MAX,
                Outcome::Decisive {
                    winner: Color::Black
                } => i16::MIN,
            };
        }
        None => {
            for y in board.board().pieces() {
                let piece = y.1;
                let multiplier = match piece.color {
                    Color::White => 1,
                    Color::Black => -1,
                };

                score += match piece.role {
                    Role::King => 900,
                    Role::Queen => 90,
                    Role::Rook => 50,
                    Role::Bishop => 30,
                    Role::Knight => 30,
                    Role::Pawn => 10,
                } * multiplier;
            }
        }
    }

    score
}

fn eval_board_color(board: Chess, black: bool) -> i16 {
    let res = eval_board(board);
    if black { -res } else { res }
}

fn pvs(board: Chess, depth: u8, mut alpha: i16, beta: i16, black: bool) -> (String, i16) {
    if depth == 0 || board.is_game_over() || board.legal_moves().len() == 0 {
        return (String::new(), eval_board_color(board, black));
    }

    let mut first = true;
    let mut result = (String::new(), i16::MIN);
    let mut alpha_move = String::new();

    for m in board.legal_moves() {
        if first {
            first = false;
            let mut b = board.clone();
            b.play_unchecked(&m);
            let res = pvs(b, depth-1, -beta, -alpha, !black);
            result.1 = -res.1;
            result.0 = res.0;
        } else {
            let mut b = board.clone();
            b.play_unchecked(&m);
            let res = pvs(b, depth-1, -alpha-1, -alpha, !black);
            result.1 = -res.1;
            result.0 = res.0;

            if alpha < result.1 && result.1 < beta {
                let mut b = board.clone();
                b.play_unchecked(&m);
                let res = pvs(b, depth-1, -beta, -result.1, !black);
                result.1 = -res.1;
                result.0 = res.0;
            }
        }

        if result.1 > alpha {
            alpha = result.1;
            alpha_move = Uci::from_standard(&m).to_string();
        }
        if alpha >= beta {
            break; // beta cutoff
        }
    };

    (alpha_move, alpha)
}

fn minimax(board: Chess, depth: u8, mut alpha: i16, mut beta: i16, black: bool, original: bool) -> (String, i16) {
    if depth == 0 || board.is_game_over() || board.legal_moves().len() == 0 {
        return (String::new(), eval_board_color(board, black));
    }

    let maximizing = black == original;

    return if maximizing {
        let mut best = (String::new(), i16::MIN);

        for m in board.legal_moves() {
            let mut b = board.clone();
            b.play_unchecked(&m);
            let res = minimax(b, depth-1, alpha, beta, !black, original);
            if res.1 > best.1 {
                best = res;
                best.0 = Uci::from_standard(&m).to_string();
            }
            if best.1 > alpha {
                alpha = best.1;
            }
            if alpha >= beta {
                break;
            }
        }
        best
    } else {
        let mut best = (String::new(), i16::MAX);

        for m in board.legal_moves() {
            let mut b = board.clone();
            b.play_unchecked(&m);
            let res = minimax(b, depth-1, alpha, beta, !black, original);
            if res.1 < best.1 {
                best = res;
                best.0 = Uci::from_standard(&m).to_string();
            }
            if best.1 < beta {
                beta = best.1;
            }
            if beta <= alpha {
                break;
            }
        }
        best
    };
}

pub fn choose(board: Chess, depth: u8, black: bool) -> (String, i16) {
    pvs(board, depth, i16::MIN, i16::MAX, black)
}

pub fn choose_minimax(board: Chess, depth: u8, black: bool) -> (String, i16) {
    minimax(board, depth, i16::MIN, i16::MAX, black, black)
}
