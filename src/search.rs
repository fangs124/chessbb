use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use crate::{
    ChessBoard, ChessBoardSnapshot, ChessMove, GameResult, GameState, NodeType, PieceType, PositionData, Side, TranspositionTable,
    transposition::AtomicTranspositionTable,
};

pub trait Evaluator {
    //i16 is used here as a fixed-precision evaluation out of 1000
    fn eval(&mut self, cb: &ChessBoard) -> i16;
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub struct NegamaxData {
    ply: u16,
    node_count: usize,
    node_check_count: usize,
    node_check_limit: usize,
    time_data: Option<(Instant, Duration)>,
}

impl NegamaxData {
    #[inline(always)]
    pub fn new() -> Self {
        NegamaxData { ply: 0, node_count: 0, node_check_count: 0, node_check_limit: DEFAULT_NODE_CHECK_COUNT_LIMIT, time_data: None }
    }

    #[inline(always)]
    pub fn new_timed(start: Instant, limit: Duration) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_check_count: 0, node_check_limit: DEFAULT_NODE_CHECK_COUNT_LIMIT, time_data: Some((start, limit)) }
    }

    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

type AtomicTT = AtomicTranspositionTable;
const DEFAULT_NODE_CHECK_COUNT_LIMIT: usize = 1 << 8;

impl ChessBoard {
    pub fn negamax(&mut self, a: i16, b: i16, d: usize, ev: &mut impl Evaluator, data: &mut NegamaxData, tt: Arc<AtomicTT>) -> i16 {
        if self.repetition() >= 2 {
            return 0;
        }

        let mut alpha: i16 = a;
        let position_data: PositionData = tt.load(&self.hash(), Ordering::Relaxed);
        let mut tt_chess_move: Option<ChessMove> = None;
        if position_data.depth() as usize >= d && position_data.is_valid(self.hash()) {
            match position_data.ty() {
                NodeType::Exact => return position_data.eval(),
                NodeType::Alpha => {
                    if position_data.eval() >= b {
                        return position_data.eval();
                    }
                    alpha = alpha.max(position_data.eval());
                    tt_chess_move = position_data.best();
                }
                NodeType::Beta => {
                    if position_data.eval() <= a && position_data.best().is_some() {
                        return position_data.eval();
                    }
                    //beta = beta.min(data.eval());
                }
                NodeType::None => unreachable!(),
            }
        }

        let d = match self.is_king_in_check(self.side()) {
            true => d + 1,
            false => d,
        };

        if d == 0 {
            return ev.eval(&self);
        }

        let (moves, game_state) = self.try_generate_moves();
        if let GameState::Finished(state) = game_state {
            match state {
                GameResult::WhiteWins | GameResult::BlackWins => {
                    return ((i16::MIN + 2) / 2) + (data.ply as i16); //TODO determine if +d or -d or something else should be used here.
                }
                GameResult::Draw => return 0,
            }
        }

        let mut best_value: i16 = i16::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        //TODO sort moves here
        for chess_move in moves {
            if let Some((start, limit)) = data.time_data {
                if data.node_check_count >= data.node_check_limit {
                    if start.elapsed() > limit {
                        //return best_value; //is the best_move usable here?
                        return i16::MIN + 1;
                    }
                    data.node_check_count = 0;
                }
            }

            let snapshot: ChessBoardSnapshot = self.explore_state(&chess_move);
            data.node_count += 1; //apparently this is the accepted way to count nps
            data.node_check_count += 1;
            let value: i16 = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone());
            self.restore_state(snapshot);

            if value > best_value {
                best_value = value;
                best_move = Some(chess_move);
            }

            if value > alpha {
                alpha = value;
                if alpha >= b {
                    break;
                }
            }
        }

        //tranposition table keep-up
        tt.update_tt(self.hash(), best_value, best_move, a, b, d as u16, Ordering::Relaxed);
        return best_value;
    }

    //fn quiescence_negamax(&mut self, a: i16, b: i16, d: usize, ev: &mut impl Evaluator, data: &mut NegamaxData, tt: Arc<AtomicTT>) -> i16 {
    //    let mut best_value: i16 = ev.eval(&self);
    //    let mut alpha = a;
    //
    //    //assuming non-zugzwang?
    //    if best_value >= b {
    //        return best_value;
    //    }
    //
    //    if best_value > alpha {
    //        alpha = best_value;
    //    }
    //
    //    let (mut moves, game_state) = self.try_generate_moves();
    //    if let GameState::Finished(state) = game_state {
    //        match state {
    //            GameResult::WhiteWins | GameResult::BlackWins => {
    //                return ((i16::MIN + 2) / 2) + (data.ply as i16); //TODO determine if +d or -d or something else should be used here.
    //            }
    //            GameResult::Draw => return 0,
    //        }
    //    }
    //    moves = moves.into_iter().filter(|x| self.core.is_move_capture(x)).collect();
    //    self.core.sort_moves(&mut moves.into_iter().filter(|x| self.core.is_move_capture(x)).collect());
    //
    //    for chess_move in &moves {
    //        if let Some((start, limit)) = data.time_data {
    //            if data.node_check_count >= data.node_check_limit {
    //                if start.elapsed() > limit {
    //                    //return best_value; //is the best_move usable here?
    //                    return i16::MIN + 1;
    //                }
    //                data.node_check_count = 0;
    //            }
    //        }
    //
    //        let snapshot: ChessBoardSnapshot = self.explore_state(&chess_move);
    //        data.node_count += 1; //apparently this is the accepted way to count nps
    //        data.node_check_count += 1;
    //        let value: i16 = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone());
    //        self.restore_state(snapshot);
    //
    //        if value > best_value {
    //            best_value = value;
    //        }
    //
    //        if value > alpha {
    //            alpha = value;
    //            if alpha >= b {
    //                break;
    //            }
    //        }
    //    }
    //    return best_value;
    //}
}

/*
#[allow(non_camel_case_types)]
pub struct MCTS_Tree {
    root: MCTS_Node,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct MCTS_Node {
    is_visited: bool,
    is_expanded: bool,
    total_reward: i32,
    total_visits: u32,
    children: Vec<(MCTS_Node, f32)>,
}

pub trait Policy {
    //i16 is used here as a fixed-precision evaluation out of 1000
    fn policy(&mut self, cb: &ChessBoard) -> Vec<(ChessMove, f32)>;
}

impl ChessBoard {
    pub fn rollout(&self) {}
}

//const is blocked here mainly by f32::sqrt() fn not being const
impl MCTS_Node {
    const CONSTANT: f32 = 0.5;
    pub fn traverse(&self) -> &MCTS_Node {
        let mut node = self;
        while node.is_expanded {
            node = node.best_uct();
            //node = best_uct(node);
        }
        for child in &node.children {
            if !child.0.is_visited {
                return &child.0;
            }
        }
        return node;
    }

    pub fn best_uct(&self) -> &MCTS_Node {
        let mut best: f32 = 0.0;
        let mut best_node: &MCTS_Node = self;
        let mut i: usize = 0;
        while i < self.children.len() {
            let value = self.uct(i);
            if value > best {
                best = value;
                best_node = &self.children[i].0;
            }
            i += 1;
        }
        return best_node;
    }

    #[inline(always)]
    pub fn uct(&self, i: usize) -> f32 {
        assert!(i < self.children.len());
        let (node, prior) = &self.children[i];
        return (node.total_reward as f32) / (node.total_visits as f32)
            + MCTS_Node::CONSTANT * prior * ((self.total_reward as f32).sqrt() / (1.0 + (node.total_reward as f32)));
    }

    //pub fn rollout(&self) {
    //    let node = self;
    //    while
    //}
}*/
