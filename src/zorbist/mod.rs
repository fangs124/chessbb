use crate::bitboard::*;
use crate::chessmove::Castling;
use crate::{ChessBoard, square::Square};

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

include!("data/data.rs");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct ZorbistHash {
    value: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct ZorbistTable {
    data: [ZorbistHash; 1 << 14],
    index: usize,
}

impl ZorbistTable {
    pub(super) const fn new(hash: ZorbistHash) -> ZorbistTable {
        let mut data: [ZorbistHash; 1 << 14] = [ZorbistHash { value: 0 }; 1 << 14];
        data[0] = hash;
        return ZorbistTable { data, index: 0 };
    }

    #[inline(always)]
    pub(super) const fn initial_table() -> ZorbistTable {
        ZorbistTable::new(ZorbistHash::initial_hash())
    }

    #[inline(always)]
    pub(super) const fn add(&mut self, hash: ZorbistHash) {
        assert!(self.index < (1 << 14));
        self.index += 1;
        self.data[self.index] = hash;
    }

    #[inline(always)]
    pub(super) const fn last_hash(&self) -> ZorbistHash {
        self.data[self.index]
    }

    pub(super) const fn count_hash(&self, hash: ZorbistHash) -> usize {
        let mut i: usize = 0;
        let mut count: usize = 0;
        while i <= self.index {
            if self.data[i].value == hash.value {
                count += 1;
            }
            i += 1
        }
        return count;
    }
}

impl ZorbistHash {
    #[inline(always)]
    pub(super) const fn new(value: u64) -> ZorbistHash {
        ZorbistHash { value }
    }

    pub(super) const fn initial_hash() -> ZorbistHash {
        let mut value: u64 = 0;

        //starting side is white, no hash
        //no en-passant in starting position either

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = ChessBoard::INITIAL_MAILBOX[i] {
                value ^= PIECE_HASH[i][cp_index(piece_data)];
            }
            i += 1;
        }

        //castle hash
        i = 0;
        while i < 4 {
            value ^= CASTLE_HASH[i];
            i += 1;
        }

        return ZorbistHash { value };
    }

    pub(super) const fn compute_hash(chessboard: &ChessBoard) -> ZorbistHash {
        //side hash
        let mut value = match chessboard.side_to_move {
            crate::bitboard::Side::White => 0u64,
            crate::bitboard::Side::Black => SIDE_HASH[0],
        };

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = chessboard.mailbox[i] {
                value ^= PIECE_HASH[i][cp_index(piece_data)];
            }
            i += 1;
        }

        //castle hash
        i = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }

        //en-passant hash
        let mut enpassant_bb = chessboard.enpassant_bb;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_usize()]];
            enpassant_bb.pop_bit(square);
        }

        return ZorbistHash { value };
    }

    pub(super) const fn compute_castle_hash(chessboard: &ChessBoard) -> ZorbistHash {
        let mut value = 0u64;

        let mut i: usize = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }
        return ZorbistHash { value };
    }

    pub(super) const fn compute_enpassant_hash(enpassant_bb: BitBoard) -> ZorbistHash {
        let mut value: u64 = 0;
        let mut enpassant_bb = enpassant_bb;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_usize()]];
            enpassant_bb.pop_bit(square);
        }
        return ZorbistHash { value };
    }

    //const fn update_hash(&self, s: Square, t: Square, s_p: ChessPiece, t_p: Option<ChessPiece>) -> ZorbistHash {
    //    let mut value = self.value;
    //    value ^= PIECE_HASH[s.to_index()][cp_index(s_p)];
    //    value ^= PIECE_HASH[t.to_index()][cp_index(s_p)];
    //    value ^= match t_p {
    //        Some(t_p) => PIECE_HASH[t.to_index()][cp_index(t_p)],
    //        None => 0u64,
    //    };
    //    todo!()
    //}

    #[inline(always)]
    pub(crate) const fn piece_hash(square: Square, chesspiece: ChessPiece) -> ZorbistHash {
        ZorbistHash { value: PIECE_HASH[square.to_usize()][cp_index(chesspiece)] }
    }

    #[inline(always)]
    pub(crate) const fn castle_hash(castling: Castling) -> ZorbistHash {
        match castling {
            Castling::Kingside(Side::White) => ZorbistHash { value: CASTLE_HASH[0] },
            Castling::Queenside(Side::White) => ZorbistHash { value: CASTLE_HASH[1] },
            Castling::Kingside(Side::Black) => ZorbistHash { value: CASTLE_HASH[2] },
            Castling::Queenside(Side::Black) => ZorbistHash { value: CASTLE_HASH[3] },
        }
    }

    #[inline(always)]
    pub(crate) const fn side_hash() -> ZorbistHash {
        ZorbistHash { value: SIDE_HASH[0] }
    }
}

impl BitAnd for ZorbistHash {
    type Output = ZorbistHash;

    #[inline(always)]
    fn bitand(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value & rhs.value }
    }
}

impl BitAndAssign for ZorbistHash {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for ZorbistHash {
    type Output = ZorbistHash;

    #[inline(always)]
    fn bitor(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value | rhs.value }
    }
}

impl BitOrAssign for ZorbistHash {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for ZorbistHash {
    type Output = ZorbistHash;

    #[inline(always)]
    fn bitxor(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value ^ rhs.value }
    }
}

impl BitXorAssign for ZorbistHash {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}

impl Not for ZorbistHash {
    type Output = ZorbistHash;

    #[inline(always)]
    fn not(self) -> Self::Output {
        ZorbistHash { value: !self.value }
    }
}
