use crate::{ChessBoard, ChessMove, GameResult, GameState, NodeData, NodeType, PieceType, Side, TranspositionTable};

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
        tt: &mut TranspositionTable,
    ) -> (i16, Option<ChessMove>) {
        let mut alpha: i16 = a;
        //let mut beta: i16 = b;
        let data: NodeData = tt.look_up(&self.hash());
        if let Some(ty) = data.ty() {
            if data.depth() as usize >= d && data.key() == self.hash() {
                match ty {
                    NodeType::Exact => return data.pair(),
                    NodeType::Alpha => {
                        if data.eval() >= b {
                            return data.pair();
                        }
                        alpha = alpha.max(data.eval());
                    }
                    NodeType::Beta => {
                        if data.eval() <= a && data.best().is_some() {
                            return data.pair();
                        }
                        //beta = beta.min(data.eval());
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
        let mut best_value: i16 = i16::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        for chessmove in moves {
            let snapshot: crate::ChessBoardSnapshot = self.explore_state(chessmove);
            let (score, chessmove) = self.negated_negamax(-b, -alpha, d - 1, ply + 1, ev, tt);
            self.restore_state(snapshot);

            if score > best_value {
                best_value = score;
                best_move = chessmove;
            }

            if score > alpha {
                alpha = score;
            }

            if score >= b {
                break;
            }
        }

        //tranposition table keep-up
        tt.update_tt(self.hash(), best_value, best_move, a, b, d as u16);
        return (best_value, best_move);
    }

    #[inline(always)]
    pub fn negated_negamax(
        &mut self,
        a: i16,
        b: i16,
        d: usize,
        ply: usize,
        ev: &mut impl Evaluator,
        tt: &mut TranspositionTable,
    ) -> (i16, Option<ChessMove>) {
        negated_pair(self.negamax(a, b, d, ply, ev, tt))
    }
}
#[inline(always)]
fn negated_pair(pair: (i16, Option<ChessMove>)) -> (i16, Option<ChessMove>) {
    (-pair.0, pair.1)
}
