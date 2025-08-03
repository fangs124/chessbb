use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::BitBoard;

impl BitAnd for BitBoard {
    type Output = BitBoard;
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data & rhs.data }
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.data &= rhs.data;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;
    fn bitor(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data | rhs.data }
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.data |= rhs.data;
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;
    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        BitBoard { data: self.data ^ rhs.data }
    }
}

impl BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.data ^= rhs.data;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;
    fn not(self) -> Self::Output {
        BitBoard { data: !self.data }
    }
}
