use std::ops::{Index, IndexMut};

use crate::{ChessMove, search::ScoredMove, zobrist::ZobristHash};

//16MB for stc, 128MB for ltc
//128MB is 1073741824 bits
//16MB is  0134217728 bits
//NodeData: 96 bit

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeData {
    key: ZobristHash,
    depth: u16, //is u8 enough? u16 should be enough, its over 64 000+
    eval: i16,
    ty: Option<NodeType>,
    best: Option<ChessMove>,
}

impl Default for NodeData {
    fn default() -> Self {
        return Self { key: ZobristHash::ZERO, depth: 0, eval: 0, ty: None, best: None };
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        return Self::new();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    Alpha, //lower-bound
    Beta,  //upper-bound
}

//can make this generic if necessary. otherwise it adds complication to the code right now
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranspositionTable {
    data: Box<[NodeData; DEFAULT_SIZE]>,
}

//const foo: usize = size_of::<TranspositionTable>();
//const foo: usize = size_of::<NodeData>();

const DEFAULT_SIZE: usize = 1 << 22;

impl TranspositionTable {
    #[inline(always)]
    pub fn new() -> Self {
        //this is ugly
        return TranspositionTable { data: vec![NodeData::default(); DEFAULT_SIZE].try_into().unwrap() };
        //return TranspositionTable { data: [NodeData::default(); DEFAULT_SIZE] };
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn update_tt(&mut self, hash: ZobristHash, value: i16, chess_move: Option<ChessMove>, a: i16, b: i16, d: u16) {
        let node_type = NodeData::value_type(value, a, b);
        match node_type {
            NodeType::Beta => self.store(hash, d, value, Some(node_type), None),
            _ => self.store(hash, d, value, Some(node_type), chess_move),
        }
        ;
    }

    //TODO need to think of replacement policy. right now it uses the naive always replace
    #[rustfmt::skip]
    #[inline(always)]
    pub fn store(&mut self, hash: ZobristHash, depth: u16, eval: i16, ty: Option<NodeType>, best: Option<ChessMove>) {
        self.data[hash] = NodeData::new(hash, depth, eval, ty, best);
    }

    #[inline(always)]
    pub(crate) fn look_up(&self, hash: &ZobristHash) -> NodeData {
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

impl NodeData {
    #[inline(always)]
    fn new(key: ZobristHash, depth: u16, eval: i16, ty: Option<NodeType>, best: Option<ChessMove>) -> Self {
        return Self { key, depth, eval: eval, ty, best };
    }

    #[inline(always)]
    const fn const_default() -> Self {
        return Self { key: ZobristHash::ZERO, depth: 0, eval: 0, ty: None, best: None };
    }

    #[inline(always)]
    pub const fn is_valid(&self, hash: ZobristHash) -> bool {
        return !matches!(self.ty, None) && matches!(self.key, hash);
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
    pub const fn ty(&self) -> Option<NodeType> {
        return self.ty;
    }

    #[inline(always)]
    pub const fn best(&self) -> Option<ChessMove> {
        return self.best;
    }

    #[inline(always)]
    pub const fn pair(&self) -> ScoredMove {
        return ScoredMove::new(self.eval, self.best);
    }

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

impl Index<ZobristHash> for [NodeData] {
    type Output = NodeData;

    #[inline(always)]
    fn index(&self, index: ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl Index<&ZobristHash> for [NodeData] {
    type Output = NodeData;

    #[inline(always)]
    fn index(&self, index: &ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl IndexMut<ZobristHash> for [NodeData] {
    #[inline(always)]
    fn index_mut(&mut self, index: ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}

impl IndexMut<&ZobristHash> for [NodeData] {
    #[inline(always)]
    fn index_mut(&mut self, index: &ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}
