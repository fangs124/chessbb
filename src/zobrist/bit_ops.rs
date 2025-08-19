use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::ZobristHash;

impl BitAnd for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitand(self, rhs: ZobristHash) -> Self::Output {
        ZobristHash { value: self.value & rhs.value }
    }
}

impl BitAndAssign for ZobristHash {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitor(self, rhs: ZobristHash) -> Self::Output {
        ZobristHash { value: self.value | rhs.value }
    }
}

impl BitOrAssign for ZobristHash {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitxor(self, rhs: ZobristHash) -> Self::Output {
        ZobristHash { value: self.value ^ rhs.value }
    }
}

impl BitXorAssign for ZobristHash {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}

impl Not for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn not(self) -> Self::Output {
        ZobristHash { value: !self.value }
    }
}
