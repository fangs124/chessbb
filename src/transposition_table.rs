use std::ops::{Index, IndexMut};

use crate::zobrist::ZobristHash;

//16MB for stc, 128MB for ltc
//128MB is 1073741824 bits
//16MB is  0134217728 bits
//NodeData: 96 bit

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeData {
    key: ZobristHash,
    depth: u16, //is u8 enough? u16 should be enough, its over 64 000+
    eval: i16,
    ty: NodeType,
}

impl Default for NodeData {
    fn default() -> Self {
        return Self { key: ZobristHash::ZERO, depth: 0, eval: 0, ty: NodeType::Exact };
    }
}

impl<const N: usize> Default for TranspositionTable<N> {
    fn default() -> Self {
        return Self::new();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    Alpha,
    Beta,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TranspositionTable<const N: usize> {
    data: [NodeData; N],
}

const DEFAULT_S: usize = 1 << 24;
const DEFAULT_L: usize = 1 << 21;

pub type LargeTT = TranspositionTable<DEFAULT_L>;
pub type SmallTT = TranspositionTable<DEFAULT_S>;

impl<const N: usize> TranspositionTable<N> {
    #[inline(always)]
    pub fn new() -> Self {
        return TranspositionTable { data: [NodeData::default(); N] };
    }

    //TODO need to think of replacement policy. right now it uses the naive always replace
    #[inline(always)]
    pub fn store(&mut self, hash: ZobristHash, depth: u16, eval: f32, ty: NodeType) {
        self.data[hash] = NodeData::new(hash, depth, eval, ty);
    }
}

impl NodeData {
    #[inline(always)]
    pub fn new(key: ZobristHash, depth: u16, eval: f32, ty: NodeType) -> Self {
        return Self { key, depth, eval: (eval.clamp(-100.0, 100.0) / 100.0) as i16, ty };
    }
}

impl Index<ZobristHash> for [NodeData] {
    type Output = NodeData;

    #[inline(always)]
    fn index(&self, index: ZobristHash) -> &Self::Output {
        return &self[index.to_index() % self.len()];
    }
}

impl IndexMut<ZobristHash> for [NodeData] {
    #[inline(always)]
    fn index_mut(&mut self, index: ZobristHash) -> &mut Self::Output {
        return &mut self[index.to_index() % self.len()];
    }
}
