use std::i32;

use crate::{zobrist::ZobristHash, ChessBoard, ChessBoardCore, ChessMove, GameResult, GameState, PieceType, Side};

pub trait Evaluator {
    //i32 is used here as a fixed-precision evaluation out of 1000
    fn eval(&mut self, cb: &ChessBoard) -> i32;
}

pub struct MaterialEvaluator;

pub const MATERIAL_EVAL: MaterialEvaluator = MaterialEvaluator {};

impl Evaluator for MaterialEvaluator {
    fn eval(&mut self, cb: &ChessBoard) -> i32 {
        let mut total: i32 = 0;
        for piece in cb.mailbox_iterator() {
            if let Some(chesspiece) = piece {
                total += match chesspiece {
                    (Side::White, PieceType::King) => 100,
                    (Side::White, PieceType::Queen) => 009,
                    (Side::White, PieceType::Knight) => 003,
                    (Side::White, PieceType::Bishop) => 003,
                    (Side::White, PieceType::Rook) => 005,
                    (Side::White, PieceType::Pawn) => 001,
                    (Side::Black, PieceType::King) => -100,
                    (Side::Black, PieceType::Queen) => -009,
                    (Side::Black, PieceType::Knight) => -003,
                    (Side::Black, PieceType::Bishop) => -003,
                    (Side::Black, PieceType::Rook) => -005,
                    (Side::Black, PieceType::Pawn) => -001,
                }
            }
        }

        return match cb.side() {
            Side::White => total,
            Side::Black => -total,
        };
    }
}

impl ChessBoard {
    // a: min value for maximizing player
    //    worst possible outcome for you assuming enemy played best
    // b: max value for minimizing player
    //    best possible outcome for enemy assuming you played best
    // this implies the following bounds:
    // alpha <= eval <= beta
    #[rustfmt::skip]
    pub fn negamax(&mut self, a: i32, b: i32, d: usize, ply: usize, ev: &mut impl Evaluator) -> (i32, Option<ChessMove>) {
        let d = match self.is_king_in_check(self.side()) {
            true => d + 1,
            false => d,
        };

        if d == 0 {
            return (ev.eval(&self), None);
        }
        
        let (moves, game_state) = self.try_generate_moves();
        if let GameState::Finished(state) = game_state {
            match state {
                GameResult::WhiteWins | GameResult::BlackWins => {
                    return (((i32::MIN + 2) / 2) + (ply as i32), None); //TODO determine if +d or -d or something else should be used here.
                }
                GameResult::Draw => return (0, None),
            }
        }

        let mut alpha = a;
        let mut best_value = i32::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        for chessmove in  moves {
            let snapshot = self.explore_state(chessmove);
            let score = -self.negamax(-b, -alpha, d - 1,ply+1, ev).0;
            self.restore_state(snapshot);
            
            if score > best_value {
                best_value = score;
                best_move = Some(chessmove);
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= b {
                return (b, None);
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
