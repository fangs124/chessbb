use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::bitboard::BitBoard;

/* last revised: 8/8/2025 */

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data & rhs.data }
    }
}

impl BitAndAssign for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.data &= rhs.data;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitor(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data | rhs.data }
    }
}

impl BitOrAssign for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.data |= rhs.data;
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data ^ rhs.data }
    }
}

impl BitXorAssign for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.data ^= rhs.data;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        BitBoard { data: !self.data }
    }
}
