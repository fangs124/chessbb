use std::i32;

use crate::{
    ChessBoard, ChessBoardCore, ChessMove, GameResult, GameState, NodeData, PieceType, Side, TranspositionTable,
    chessmove, transposition::NodeType, zobrist::ZobristHash,
};

pub trait Evaluator {
    //i16 is used here as a fixed-precision evaluation out of 1000
    fn eval(&mut self, cb: &ChessBoard) -> i16;
}

pub struct MaterialEvaluator;

pub const MATERIAL_EVAL: MaterialEvaluator = MaterialEvaluator {};

impl Evaluator for MaterialEvaluator {
    fn eval(&mut self, cb: &ChessBoard) -> i16 {
        let mut total: i16 = 0;
        for piece in cb.mailbox_iterator() {
            if let Some(chesspiece) = piece {
                total += match chesspiece {
                    (Side::White, PieceType::King) => 1000,
                    (Side::White, PieceType::Queen) => 0900,
                    (Side::White, PieceType::Knight) => 0300,
                    (Side::White, PieceType::Bishop) => 0300,
                    (Side::White, PieceType::Rook) => 0500,
                    (Side::White, PieceType::Pawn) => 0100,
                    (Side::Black, PieceType::King) => -1000,
                    (Side::Black, PieceType::Queen) => -0900,
                    (Side::Black, PieceType::Knight) => -0300,
                    (Side::Black, PieceType::Bishop) => -0300,
                    (Side::Black, PieceType::Rook) => -0500,
                    (Side::Black, PieceType::Pawn) => -0100,
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
    //#[rustfmt::skip]
    //pub fn negamax(&mut self, a: i16, b: i16, d: usize, ply: usize, ev: &mut impl Evaluator) -> (i16, Option<ChessMove>) {
    pub fn negamax(
        &mut self,
        a: i16,
        b: i16,
        d: usize,
        ply: usize,
        ev: &mut impl Evaluator,
    ) -> (i16, Option<ChessMove>) {
        let data: NodeData = self.look_up_tt();
        if let Some(ty) = data.ty() {
            if data.depth() as usize >= d {
                match ty {
                    NodeType::Exact => return data.pair(),
                    NodeType::Alpha => {
                        if data.eval() >= b {
                            return data.pair();
                        }
                    }
                    NodeType::Beta => {
                        if data.eval() <= a {
                            return data.pair();
                        }
                    }
                }
            }
        }

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
                    return (((i16::MIN + 2) / 2) + (ply as i16), None); //TODO determine if +d or -d or something else should be used here.
                }
                GameResult::Draw => return (0, None),
            }
        }

        //TODO sort moves here
        let mut alpha: i16 = a;
        let mut best_value: i16 = i16::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        for chessmove in moves {
            let snapshot = self.explore_state(chessmove);
            let (score, chessmove) = self.negamax(-b, -alpha, d - 1, ply + 1, ev);
            self.restore_state(snapshot);

            if score > best_value {
                best_value = score;
                best_move = Some(chessmove.unwrap());
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= b {
                break;
            }
        }

        //tranposition table keep-up
        self.update_tt(best_value, best_move, a, b, d as u16);
        return (best_value, best_move);
    }

    //#[inline(always)]
    //pub fn negated_negamax(
    //    &mut self,
    //    a: i16,
    //    b: i16,
    //    d: usize,
    //    ply: usize,
    //    ev: &mut impl Evaluator,
    //) -> (i16, Option<ChessMove>) {
    //    self.negamax(a, b, d, ))
    //}

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
