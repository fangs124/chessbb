use std::{
    ops::Neg,
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

pub struct ScoredMove(i16, Option<ChessMove>);

impl Neg for ScoredMove {
    type Output = ScoredMove;

    fn neg(self) -> Self::Output {
        Self(-self.0, self.1)
    }
}

impl ScoredMove {
    pub const fn new(a: i16, b: Option<ChessMove>) -> Self {
        ScoredMove(a, b)
    }

    pub const fn unwrap(self) -> (i16, Option<ChessMove>) {
        (self.0, self.1)
    }
}

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
    time_data: Option<(Instant, Duration)>,
    pair: Option<(Vec<ChessMove>, GameState)>,
}

impl NegamaxData {
    #[inline(always)]
    pub fn new(pair: Option<(Vec<ChessMove>, GameState)>) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_check_count: 0, time_data: None, pair }
    }

    #[inline(always)]
    pub fn new_timed(pair: Option<(Vec<ChessMove>, GameState)>, start: Instant, limit: Duration) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_check_count: 0, time_data: Some((start, limit)), pair }
    }

    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    #[inline(always)]
    fn get_pair_or(&mut self, default: (Vec<ChessMove>, GameState)) -> (Vec<ChessMove>, GameState) {
        self.pair.take().unwrap_or(default)
    }
}

type AtomicTT = AtomicTranspositionTable;
const NODE_CHECK_COUNT_LIMIT: usize = 1 << 6;

impl ChessBoard {
    pub fn negamax(&mut self, a: i16, b: i16, d: u16, ev: &mut impl Evaluator, data: &mut NegamaxData, tt: Arc<AtomicTT>) -> ScoredMove {
        let mut alpha: i16 = a;
        let position_data: PositionData = tt.load(&self.hash(), Ordering::Relaxed);
        let mut tt_chess_move: Option<ChessMove> = None;
        if position_data.depth() >= d && position_data.is_valid(self.hash()) {
            tt_chess_move = position_data.pair().1;
            match position_data.ty() {
                NodeType::Exact => return position_data.pair(),
                NodeType::Alpha => {
                    if position_data.eval() >= b {
                        return position_data.pair();
                    }
                    alpha = alpha.max(position_data.eval());
                    tt_chess_move = position_data.pair().1;
                }
                NodeType::Beta => {
                    if position_data.eval() <= a && position_data.best().is_some() {
                        return position_data.pair();
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
            return ScoredMove(ev.eval(&self), None);
        }

        let (mut moves, game_state) = data.get_pair_or(self.try_generate_moves());
        if let GameState::Finished(state) = game_state {
            match state {
                GameResult::WhiteWins | GameResult::BlackWins => {
                    return ScoredMove(((i16::MIN + 2) / 2) + (data.ply as i16), None); //TODO determine if +d or -d or something else should be used here.
                }
                GameResult::Draw => return ScoredMove::new(0, None),
            }
        }

        let mut best_value: i16 = i16::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        //explore previous move in transposition table, if any
        if let Some(chess_move) = tt_chess_move {
            if let Some(time_data) = data.time_data {
                if time_data.0.elapsed() > time_data.1 {
                    return ScoredMove(best_value, best_move); //is the best_move usable here?
                }
            }

            if let Some(snapshot) = self.try_explore_state(chess_move) {
                data.node_count += 1; //apparently this is the accepted way to count nps
                data.node_check_count += 1;
                //let depth = match self.core.is_move_capture(&chess_move) {
                //    true => d,
                //    false => d - 1,
                //};
                let ScoredMove(value, next_move) = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone());
                self.restore_state(snapshot);
                if value > best_value {
                    best_value = value;
                    best_move = Some(chess_move);
                }

                if value > alpha {
                    alpha = value;
                }
            }
        }

        //TODO sort moves here
        self.core.sort_moves(&mut moves);
        for chess_move in moves {
            if let Some(time_data) = data.time_data {
                if data.node_check_count >= NODE_CHECK_COUNT_LIMIT {
                    if time_data.0.elapsed() > time_data.1 {
                        return ScoredMove(best_value, best_move); //is the best_move usable here?
                    }
                    data.node_check_count = 0;
                }
            }
            let snapshot: ChessBoardSnapshot = self.explore_state(chess_move);
            data.node_count += 1; //apparently this is the accepted way to count nps
            data.node_check_count += 1;
            //let depth = match self.core.is_move_capture(&chess_move) {
            //    true => d,
            //    false => d - 1,
            //};
            let ScoredMove(value, next_move) = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone());
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
        return ScoredMove(best_value, best_move);
    }
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
