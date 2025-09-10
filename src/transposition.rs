use std::{
    ops::{Index, IndexMut},
    rc::Rc,
    sync::atomic::Ordering,
};

use atomic::Atomic;
use bytemuck::{NoUninit, Pod};

use crate::{ChessMove, zobrist::ZobristHash};

//16MB for stc, 128MB for ltc
//128MB is 1073741824 bits
//16MB is  0134217728 bits
//NodeData: 96 bit

#[derive(Debug, Copy, Clone, PartialEq, Eq, NoUninit)]
#[repr(C)]
pub struct PositionData {
    key: ZobristHash,
    depth: u16, //is u8 enough? u16 should be enough, its over 64 000+
    eval: i16,
    ty: NodeType,
    best: MaybeChessMove,
}

impl Default for PositionData {
    fn default() -> Self {
        return Self { key: ZobristHash::ZERO, depth: 0, eval: 0, ty: NodeType::None, best: MaybeChessMove::NONE };
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        return Self::new();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, NoUninit)]
#[repr(u16)]
pub enum NodeType {
    Exact,
    Alpha, //lower-bound
    Beta,  //upper-bound
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, NoUninit)]
#[repr(transparent)]
struct MaybeChessMove(u16);

impl From<Option<ChessMove>> for MaybeChessMove {
    fn from(value: Option<ChessMove>) -> Self {
        match value {
            Some(chess_move) => MaybeChessMove(chess_move.data()), //assert! here?
            None => MaybeChessMove::NONE,
        }
    }
}

impl From<MaybeChessMove> for Option<ChessMove> {
    fn from(value: MaybeChessMove) -> Self {
        if value.0 == 0b10_11_0000_0000 {
            return None;
        } else {
            return Some(ChessMove::from_raw(value.0));
        }
    }
}

impl MaybeChessMove {
    const NONE: MaybeChessMove = MaybeChessMove(0b10_11_0000_0000);
}

impl PositionData {
    #[inline(always)]
    fn new(key: ZobristHash, depth: u16, eval: i16, ty: NodeType, best: Option<ChessMove>) -> Self {
        return Self { key, depth, eval: eval, ty, best: MaybeChessMove::from(best) };
    }

    #[inline(always)]
    const fn const_default() -> Self {
        return Self { key: ZobristHash::ZERO, depth: 0, eval: 0, ty: NodeType::None, best: MaybeChessMove::NONE };
    }

    #[inline(always)]
    pub fn is_valid(&self, hash: ZobristHash) -> bool {
        return (self.ty != NodeType::None) && (self.key == hash);
    }

    #[inline(always)]
    pub const fn key(&self) -> ZobristHash {
        return self.key;
    }

    #[inline(always)]
    pub const fn depth(&self) -> u16 {
        return self.depth;
    }

    #[inline(always)]
    pub const fn eval(&self) -> i16 {
        return self.eval;
    }

    #[inline(always)]
    pub const fn ty(&self) -> NodeType {
        return self.ty;
    }

    #[inline(always)]
    pub fn best(&self) -> Option<ChessMove> {
        return Option::<ChessMove>::from(self.best);
    }

    //#[inline(always)]
    //pub fn pair(&self) -> ScoredMove {
    //    return ScoredMove::new(self.eval, Option::<ChessMove>::from(self.best));
    //}

    #[inline(always)]
    pub const fn value_type(value: i16, a: i16, b: i16) -> NodeType {
        if value <= a {
            return NodeType::Beta;
        } else if b <= value {
            return NodeType::Alpha;
        } else {
            return NodeType::Exact;
        }
    }
}

impl Index<ZobristHash> for [PositionData] {
    type Output = PositionData;

    #[inline(always)]
    fn index(&self, index: ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl Index<&ZobristHash> for [PositionData] {
    type Output = PositionData;

    #[inline(always)]
    fn index(&self, index: &ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl IndexMut<ZobristHash> for [PositionData] {
    #[inline(always)]
    fn index_mut(&mut self, index: ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}

impl IndexMut<&ZobristHash> for [PositionData] {
    #[inline(always)]
    fn index_mut(&mut self, index: &ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}

//can make this generic if necessary. otherwise it adds complication to the code right now
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranspositionTable {
    data: Box<[PositionData; DEFAULT_SIZE]>,
}

//const foo: usize = size_of::<TranspositionTable>();
//const foo: usize = size_of::<NodeData>();

const DEFAULT_SIZE: usize = 1 << 22;

impl TranspositionTable {
    #[inline(always)]
    pub fn new() -> Self {
        //this is ugly
        return TranspositionTable { data: vec![PositionData::default(); DEFAULT_SIZE].try_into().unwrap() };
        //return TranspositionTable { data: [NodeData::default(); DEFAULT_SIZE] };
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn update_tt(&mut self, hash: ZobristHash, value: i16, chess_move: Option<ChessMove>, a: i16, b: i16, d: u16) {
        let node_type = PositionData::value_type(value, a, b);
        match node_type {
            NodeType::Beta => self.store(hash, d, value, node_type, None),
            _ => self.store(hash, d, value, node_type, chess_move),
        }
        ;
    }

    //TODO need to think of replacement policy. right now it uses the naive always replace
    #[rustfmt::skip]
    #[inline(always)]
    pub fn store(&mut self, hash: ZobristHash, depth: u16, eval: i16, ty: NodeType, best: Option<ChessMove>) {
        self.data[hash] = PositionData::new(hash, depth, eval, ty, best);
    }

    #[inline(always)]
    pub(crate) fn look_up(&self, hash: &ZobristHash) -> PositionData {
        self.data[hash]
    }
}

//#[derive(Debug, Copy, Clone, PartialEq, Eq)]
//pub struct TranspositionTable<const N: usize> {
//    data: [NodeData; N],
//}

//const DEFAULT_S: usize = 1 << 24;
//const DEFAULT_L: usize = 1 << 21;
//
//pub type TTLarge = TranspositionTable<DEFAULT_L>;
//pub type TTSmall = TranspositionTable<DEFAULT_S>;

//impl<const N: usize> TranspositionTable<N> {
//    #[inline(always)]
//    pub fn new() -> Self {
//        return TranspositionTable { data: [NodeData::default(); N] };
//    }
//
//    //TODO need to think of replacement policy. right now it uses the naive always replace
//    #[inline(always)]
//    pub fn store(&mut self, hash: ZobristHash, depth: u16, eval: f32, ty: NodeType, best: Option<ChessMove>) {
//        self.data[hash] = NodeData::new(hash, depth, eval, ty, best);
//    }
//}

#[derive(Debug)] //is this Sync?
pub struct AtomicTranspositionTable {
    data: Box<[AtomicPositionData; DEFAULT_SIZE]>,
}

//const fn check_sync<T: Sync>() {}
//const _: () = check_sync::<Rc<()>>();

#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicPositionData(Atomic<PositionData>);

impl Default for AtomicPositionData {
    fn default() -> Self {
        Self(Atomic::default())
    }
}

impl Clone for AtomicPositionData {
    fn clone(&self) -> Self {
        Self(Atomic::new(self.0.load(Ordering::SeqCst)))
    }
}

impl AtomicPositionData {
    #[inline(always)]
    fn new(key: ZobristHash, depth: u16, eval: i16, ty: NodeType, best: Option<ChessMove>) -> Self {
        return AtomicPositionData(Atomic::new(PositionData::new(key, depth, eval, ty, best)));
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut PositionData {
        self.0.get_mut()
    }

    #[inline(always)]
    pub fn into_inner(self) -> PositionData {
        self.0.into_inner()
    }

    #[inline(always)]
    pub fn load(&self, order: Ordering) -> PositionData {
        self.0.load(order)
    }

    #[inline(always)]
    pub fn store(&self, val: PositionData, order: Ordering) {
        self.0.store(val, order);
    }

    #[inline(always)]
    pub fn swap(&self, val: PositionData, order: Ordering) -> PositionData {
        self.0.swap(val, order)
    }

    #[inline(always)]
    const fn const_default() -> Self {
        return Self(Atomic::new(PositionData::const_default()));
    }

    //#[inline(always)]
    //pub fn is_valid(&self, hash: ZobristHash, order: Ordering) -> bool {
    //    let data = self.load(order);
    //    return !matches!(data.ty, NodeType::None) && matches!(data, hash);
    //}
    //
    //#[inline(always)]
    //pub fn key(&self, order: Ordering) -> ZobristHash {
    //    return self.load(order).key;
    //}
    //
    //#[inline(always)]
    //pub fn depth(&self, order: Ordering) -> u16 {
    //    return self.load(order).depth;
    //}
    //
    //#[inline(always)]
    //pub fn eval(&self, order: Ordering) -> i16 {
    //    return self.load(order).eval;
    //}
    //
    //#[inline(always)]
    //pub fn ty(&self, order: Ordering) -> NodeType {
    //    return self.load(order).ty;
    //}
    //
    //#[inline(always)]
    //pub fn best(&self, order: Ordering) -> Option<ChessMove> {
    //    return Option::<ChessMove>::from(self.load(order).best);
    //}
    //
    //#[inline(always)]
    //pub fn pair(&self, order: Ordering) -> ScoredMove {
    //    let data = self.load(order);
    //    return ScoredMove::new(data.eval, Option::<ChessMove>::from(data.best));
    //}
    //
    //#[inline(always)]
    //pub const fn value_type(value: i16, a: i16, b: i16) -> NodeType {
    //    if value <= a {
    //        return NodeType::Beta;
    //    } else if b <= value {
    //        return NodeType::Alpha;
    //    } else {
    //        return NodeType::Exact;
    //    }
    //}
}

impl Index<ZobristHash> for [AtomicPositionData] {
    type Output = AtomicPositionData;

    #[inline(always)]
    fn index(&self, index: ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl Index<&ZobristHash> for [AtomicPositionData] {
    type Output = AtomicPositionData;

    #[inline(always)]
    fn index(&self, index: &ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl IndexMut<ZobristHash> for [AtomicPositionData] {
    #[inline(always)]
    fn index_mut(&mut self, index: ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}

impl IndexMut<&ZobristHash> for [AtomicPositionData] {
    #[inline(always)]
    fn index_mut(&mut self, index: &ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}
impl AtomicTranspositionTable {
    #[inline(always)]
    pub fn new() -> Self {
        //this is ugly
        return AtomicTranspositionTable { data: vec![AtomicPositionData::default(); DEFAULT_SIZE].try_into().unwrap() };
        //return TranspositionTable { data: [NodeData::default(); DEFAULT_SIZE] };
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn update_tt(&self, hash: ZobristHash, value: i16, chess_move: Option<ChessMove>, a: i16, b: i16, d: u16, order: Ordering) {
        let node_type = PositionData::value_type(value, a, b);
       //NOTE: when node_type == NodeType::Beta, used to store None
        self.store(hash, d, value, node_type, chess_move, order);

    }

    //TODO need to think of replacement policy. right now it uses the naive always replace
    #[rustfmt::skip]
    #[inline(always)]
    pub fn store(&self, hash: ZobristHash, depth: u16, eval: i16, ty: NodeType, best: Option<ChessMove>, order: Ordering) {
        self.data[hash].store(  PositionData::new(hash, depth, eval, ty, best), order);
    }

    #[inline(always)]
    pub(crate) fn load(&self, hash: &ZobristHash, order: Ordering) -> PositionData {
        self.data[hash].load(order)
    }
}
