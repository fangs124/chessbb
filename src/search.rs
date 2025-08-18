use crate::{ChessBoard, ChessMove, GameState};

pub trait Evaluator {
    //i32 is used here as a fixed-precision evaluation out of 1000
    fn eval(&mut self, cb: &ChessBoard) -> i32;
}

impl ChessBoard {
    pub fn negamax(&mut self, a: i32, b: i32, d: usize, ev: &mut impl Evaluator) -> (i32, Option<ChessMove>) {
        if d == 0 {
            return (ev.eval(&self), None);
        }

        if let GameState::Finished(state) = self.state() {
            match state {
                crate::GameResult::WhiteWins | crate::GameResult::BlackWins => {
                    return (((i32::MIN + 1) / 2) - (d as i32), None);
                }
                crate::GameResult::Draw => return (0, None),
            }
        }

        let moves_vec = self.generate_moves();
        let mut alpha = a;
        let mut best_value = i32::MIN + 1;

        let mut best_move: Option<ChessMove> = None;
        for chessmove in moves_vec {
            let old_core = self.core.clone();

            self.update_state(chessmove);
            let current_hash = self.hash();

            let new_value = -self.negamax(-b, -alpha, d - 1, ev).0;

            self.zobrist_table.remove_last(current_hash);
            self.core = old_core;

            // value = max(value, new_value)
            if new_value > best_value {
                best_value = new_value;
                if new_value > alpha {
                    alpha = new_value;
                    best_move = Some(chessmove);
                }
            }

            if new_value >= b {
                return (best_value, best_move);
            }
        }
        return (best_value, best_move);
    }

    //pub fn negamax(&mut self, a: i32, b: i32, d: usize, ev: impl Fn(&Self) -> i32) -> (i32, Option<ChessMove>) {
    //    if d == 0 {
    //        return (ev(&self), None);
    //    }
    //
    //    let current_hash = self.hash();
    //    if self.zobrist_table.count_hash(current_hash) == 3 {
    //        return (0, None);
    //    }
    //
    //    let moves_vec = self.generate_moves();
    //    if moves_vec.len() == 0 {
    //        if self.is_king_in_check(self.side()) {
    //            return (((i32::MIN + 1) / 2) - (d as i32), None); //checkmate
    //        } else {
    //            return (0, None); //stalemate
    //        }
    //    }
    //
    //    let mut alpha = a;
    //    let mut value = i32::MIN + 1;
    //
    //    let mut best_move: Option<ChessMove> = None;
    //    for chessmove in moves_vec {
    //        let old_core = self.core.clone();
    //
    //        self.update_state(chessmove);
    //        let current_hash = self.hash();
    //        self.zobrist_table.add(current_hash);
    //        //&ev has type:
    //        //&...&{closure@src/chessnet.rs:135:9: 135:31}
    //        let new_value = -self.negamax(-b, -a, d - 1, &ev).0;
    //        // value = max(value, new_value)
    //        if new_value > value {
    //            value = new_value;
    //        }
    //        // alpha = max(alpha, value)
    //        if value > alpha {
    //            alpha = value;
    //            best_move = Some(chessmove);
    //        }
    //        //this is not necessary!
    //        self.zobrist_table.remove_last(current_hash);
    //        self.core = old_core;
    //    }
    //    return (value, best_move);
    //}
}
