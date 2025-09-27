use std::{
    num::NonZero,
    ops::Neg,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use crate::{
    ChessBoard, ChessBoardSnapshot, ChessMove, GameResult, GameState, NodeType, PieceType, PositionData, Side, TranspositionTable,
    transposition::{AtomicTranspositionTable, SmallAtomicTranspositionTable},
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
    node_limit: Option<NonZero<usize>>,
    time_limit: Option<(Instant, Duration)>,
    is_aborted: bool,
    q_node_count: usize,
}

impl NegamaxData {
    pub fn new(node_limit: Option<NonZero<usize>>, time_limit: Option<(Instant, Duration)>) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_limit, time_limit, is_aborted: false, q_node_count: 0 }
    }
    #[inline(always)]
    pub fn new_no_limit() -> Self {
        NegamaxData { ply: 0, node_count: 0, node_limit: None, time_limit: None, is_aborted: false, q_node_count: 0 }
    }

    #[inline(always)]
    pub fn new_timed(start: Instant, limit: Duration) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_limit: None, time_limit: Some((start, limit)), is_aborted: false, q_node_count: 0 }
    }

    #[inline(always)]
    pub fn new_fixed_node(node_limit: NonZero<usize>) -> Self {
        NegamaxData { ply: 0, node_count: 0, node_limit: Some(node_limit), time_limit: None, is_aborted: false, q_node_count: 0 }
    }

    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    #[inline(always)]
    pub fn q_node_count(&self) -> usize {
        self.q_node_count
    }

    #[inline(always)]
    pub fn get_ply(&self) -> u16 {
        self.ply
    }

    #[inline(always)]
    pub fn set_ply(&mut self, ply: u16) {
        self.ply = ply;
    }

    #[inline(always)]
    pub fn is_aborted(&self) -> bool {
        self.is_aborted
    }
}

type AtomicTT = AtomicTranspositionTable;
type SmallAtomicTT = SmallAtomicTranspositionTable;
const NODE_CHECK_COUNT_LIMIT: usize = 1024;
pub const LOSE_SCORE: i16 = (i16::MIN + 2) / 2;
pub const WIN_SCORE: i16 = -LOSE_SCORE;
impl ChessBoard {
    pub fn negamax(&mut self, a: i16, b: i16, d: usize, ev: &mut impl Evaluator, data: &mut NegamaxData, tt: Arc<AtomicTT>, is_q: bool) -> i16 {
        data.ply += 1;
        let (moves, game_state) = self.try_check_state();
        if let GameState::Finished(state) = game_state {
            data.ply -= 1;
            match state {
                GameResult::WhiteWins | GameResult::BlackWins => {
                    return LOSE_SCORE + 1 + (data.ply as i16); //TODO determine if +d or -d or something else should be used here.
                }
                GameResult::Draw => return 0,
            }
        }

        let d = match self.is_king_in_check(self.side()) {
            true => d + 1,
            false => d,
        };

        if d == 0 {
            data.ply -= 1;
            //return ev.eval(self);
            return match is_q {
                true => self.quiescence_negamax(a, b, Self::QUIESCENE_FALLBACK_DEPTH, ev, data),
                false => ev.eval(self),
            };
        }

        let mut moves = moves.unwrap_or_else(|| self.core.generate_moves(None));
        let mut alpha: i16 = a;
        let mut best_value: i16 = i16::MIN + 1;
        let mut best_move: Option<ChessMove> = None;

        if let Some(position_data) = tt.load(self.hash(), Ordering::Relaxed) {
            if position_data.depth() as usize >= d {
                match position_data.ty() {
                    NodeType::Exact => {
                        {
                            data.ply -= 1;
                            return position_data.eval();
                        };
                    }
                    NodeType::Alpha if position_data.eval() >= b => {
                        data.ply -= 1;
                        return position_data.eval();
                    }
                    NodeType::Beta if position_data.eval() <= a => {
                        data.ply -= 1;
                        return position_data.eval();
                    }
                    _ => (),
                }

                if let Some(tt_chess_move) = position_data.best() {
                    if !self.core.is_move_illegal(&tt_chess_move) && position_data.ty() != NodeType::Beta {
                        let snapshot: ChessBoardSnapshot = self.explore_state(&tt_chess_move);
                        data.node_count += 1; //apparently this is the accepted way to count nps
                        let value: i16 = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone(), is_q);
                        self.restore_state(snapshot);

                        if value > best_value {
                            best_value = value;
                            best_move = Some(tt_chess_move);
                        }

                        if value > alpha {
                            alpha = value;
                        }

                        if alpha >= b {
                            if !data.is_aborted {
                                tt.update_tt(self.hash(), best_value, best_move, a, b, d as u16, Ordering::Relaxed);
                            }
                            data.ply -= 1;
                            return best_value;
                        }
                    }
                }
            }
        }

        //TODO sort moves here
        //self.core.sort_moves(&mut moves);
        //for chess_move in tt_chess_move.into_iter().chain(moves.into_iter()) {
        for chess_move in moves {
            //chef: only check every 1024 node
            if data.node_count % NODE_CHECK_COUNT_LIMIT == 0 {
                if let Some((start, limit)) = data.time_limit {
                    if start.elapsed() > limit {
                        data.is_aborted = true;
                        data.ply -= 1;
                        //when fail high, i.e. best_value >= beta, can return best_value
                        //return best_value; //is the best_move usable here?
                        return match best_value >= b {
                            true => best_value,
                            false => i16::MIN + 1,
                        };
                    }
                }

                if let Some(node_limit) = data.node_limit {
                    if data.node_count >= node_limit.get() {
                        data.is_aborted = true;
                        data.ply -= 1;
                        return match best_value >= b {
                            true => best_value,
                            false => i16::MIN + 1,
                        };
                    }
                }
            }

            let snapshot: ChessBoardSnapshot = self.explore_state(&chess_move);
            data.node_count += 1; //apparently this is the accepted way to count nps
            let value: i16 = -self.negamax(-b, -alpha, d - 1, ev, data, tt.clone(), is_q);
            self.restore_state(snapshot);

            if value > best_value {
                best_value = value;
                best_move = Some(chess_move);
            }

            if value > alpha {
                alpha = value;
            }

            if alpha >= b {
                break;
            }
        }

        //tranposition table keep-up
        if !data.is_aborted {
            tt.update_tt(self.hash(), best_value, best_move, a, b, d as u16, Ordering::Relaxed);
        }
        data.ply -= 1;
        return best_value;
    }

    const QUIESCENE_FALLBACK_DEPTH: u16 = 5;
    //pub fn negamax(&mut self, a: i16, b: i16, d: usize, ev: &mut impl Evaluator, data: &mut NegamaxData, tt: Arc<AtomicTT>, is_q: bool) -> i16
    fn quiescence_negamax(&mut self, a: i16, b: i16, d: u16, ev: &mut impl Evaluator, data: &mut NegamaxData) -> i16 {
        data.ply += 1;
        //let mut d = d;
        if self.repetition() >= 3 || self.is_fifty_move_rule() {
            return 0;
        }
        //let (moves, game_state) = self.try_check_state();
        //if let GameState::Finished(state) = game_state {
        //    data.ply -= 1;
        //    match state {
        //        GameResult::WhiteWins | GameResult::BlackWins => {
        //            return ((i16::MIN + 2) / 2) + (data.ply as i16); //TODO determine if +d or -d or something else should be used here.
        //        }
        //        GameResult::Draw => return 0,
        //    }
        //}

        let mut best_value: i16 = ev.eval(&self);
        let mut alpha = a;

        //assuming non-zugzwang?
        if best_value >= b || d == 0 || data.ply >= u8::MAX as u16 {
            data.ply -= 1;
            return best_value;
        }

        if best_value > alpha {
            alpha = best_value;
        }

        //let (mut moves, game_state) = self.try_generate_captures();
        //let mut moves = moves.unwrap_or_else(|| self.core.generate_captures(None));
        //Connor: In Seer, I just take the approach of generating all legal moves when in check in qsearch as it's rather unprincipled to eval a position where the stm is in check.
        let mut moves = match self.is_king_in_check(self.side()) {
            true => {
                let (moves, game_state) = self.try_generate_moves();
                if let GameState::Finished(state) = game_state {
                    data.ply -= 1;
                    match state {
                        GameResult::WhiteWins | GameResult::BlackWins => {
                            return LOSE_SCORE + 1 + (data.ply as i16); //TODO determine if +d or -d or something else should be used here.
                        }
                        GameResult::Draw => return 0,
                    }
                }
                moves
            }
            false => self.core.generate_captures(None),
        };

        //let mut moves = self.core.generate_captures(None);
        self.core.sort_moves(&mut moves);
        for chess_move in &moves {
            //if !self.core.is_move_capture(chess_move) {
            //    continue;
            //}

            if data.node_count % NODE_CHECK_COUNT_LIMIT == 0 {
                if let Some((start, limit)) = data.time_limit {
                    if start.elapsed() > limit {
                        data.is_aborted = true;
                        data.ply -= 1;
                        //when fail high, i.e. best_value >= beta, can return best_value
                        //return best_value; //is the best_move usable here?
                        return match best_value >= b {
                            true => best_value,
                            false => i16::MIN + 1,
                        };
                    }
                }

                if let Some(node_limit) = data.node_limit {
                    if data.node_count >= node_limit.get() {
                        data.is_aborted = true;
                        data.ply -= 1;
                        return match best_value >= b {
                            true => best_value,
                            false => i16::MIN + 1,
                        };
                    }
                }
            }

            //if let Some(node_limit) = data.node_limit {
            //    if data.node_count >= node_limit.get() {
            //        data.is_aborted = true;
            //        data.ply -= 1;
            //        return match best_value >= b {
            //            true => best_value,
            //            false => i16::MIN + 1,
            //        };
            //    }
            //}

            let snapshot: ChessBoardSnapshot = self.explore_state(&chess_move);
            data.q_node_count += 1; //apparently this is the accepted way to count nps
            let value: i16 = -self.quiescence_negamax(-b, -alpha, d - 1, ev, data);
            self.restore_state(snapshot);

            if value > best_value {
                best_value = value;
            }

            if value > alpha {
                alpha = value;
            }

            if alpha >= b {
                break;
            }
        }

        data.ply -= 1;
        return best_value;
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
