use crate::bitboard::*;
use crate::chessmove::Castling;
use crate::{ChessBoardCore, square::Square};

pub mod bit_ops;

include!("data/data.rs");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ZobristHash {
    value: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ZobristTable {
    data: [ZobristHash; 1 << 14],
    index: usize,
}

impl ZobristTable {
    pub const fn new(hash: ZobristHash) -> ZobristTable {
        let mut data: [ZobristHash; 1 << 14] = [ZobristHash { value: 0 }; 1 << 14];
        data[0] = hash;
        return ZobristTable { data, index: 0 };
    }

    #[inline(always)]
    pub const fn initial_table() -> ZobristTable {
        ZobristTable::new(ZobristHash::initial_hash())
    }

    #[inline(always)]
    pub const fn add(&mut self, hash: ZobristHash) {
        assert!(self.index < (1 << 14));
        self.index += 1;
        self.data[self.index] = hash;
    }

    #[inline(always)]
    pub const fn remove_last(&mut self, hash: ZobristHash) {
        assert!(self.data[self.index].value == hash.value);
        self.index -= 1;
    }

    #[inline(always)]
    pub const fn last_hash(&self) -> ZobristHash {
        self.data[self.index]
    }

    pub const fn count_hash(&self, hash: ZobristHash) -> usize {
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

impl ZobristHash {
    pub const ZERO: ZobristHash = ZobristHash { value: 0 };

    #[inline(always)]
    pub(super) const fn new(value: u64) -> ZobristHash {
        ZobristHash { value }
    }

    #[inline(always)]
    pub const fn to_index(&self) -> usize {
        return self.value as usize;
    }
    pub(super) const fn initial_hash() -> ZobristHash {
        let mut value: u64 = 0;

        //starting side is white, no hash
        //no en-passant in starting position either

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = ChessBoardCore::INITIAL_MAILBOX[i] {
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

        return ZobristHash { value };
    }

    #[rustfmt::skip]
    pub(super) const fn compute_hash(side: Side,mb: [Option<ChessPiece>; 64],castle: [bool; 4],enpassant: BitBoard) -> ZobristHash {
        //side hash
        let mut value = match side {
            crate::bitboard::Side::White => 0u64,
            crate::bitboard::Side::Black => SIDE_HASH[0],
        };

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = mb[i] {
                value ^= PIECE_HASH[i][cp_index(piece_data)];
            }
            i += 1;
        }

        //castle hash
        i = 0;
        while i < 4 {
            if castle[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }

        //en-passant hash
        let mut enpassant_bb = enpassant;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_usize()]];
            enpassant_bb.pop_bit(square);
        }

        return ZobristHash { value };
    }

    pub(super) const fn recompute_hash(chessboard: &ChessBoardCore) -> ZobristHash {
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

        return ZobristHash { value };
    }

    pub(super) const fn compute_castle_hash(chessboard: &ChessBoardCore) -> ZobristHash {
        let mut value = 0u64;

        let mut i: usize = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }
        return ZobristHash { value };
    }

    pub(super) const fn compute_enpassant_hash(enpassant_bb: BitBoard) -> ZobristHash {
        let mut value: u64 = 0;
        let mut enpassant_bb = enpassant_bb;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_usize()]];
            enpassant_bb.pop_bit(square);
        }
        return ZobristHash { value };
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
    pub(crate) const fn piece_hash(square: Square, chesspiece: ChessPiece) -> ZobristHash {
        ZobristHash { value: PIECE_HASH[square.to_usize()][cp_index(chesspiece)] }
    }

    #[inline(always)]
    pub(crate) const fn castle_hash(castling: Castling) -> ZobristHash {
        match castling {
            Castling::Kingside(Side::White) => ZobristHash { value: CASTLE_HASH[0] },
            Castling::Queenside(Side::White) => ZobristHash { value: CASTLE_HASH[1] },
            Castling::Kingside(Side::Black) => ZobristHash { value: CASTLE_HASH[2] },
            Castling::Queenside(Side::Black) => ZobristHash { value: CASTLE_HASH[3] },
        }
    }

    #[inline(always)]
    pub(crate) const fn side_hash() -> ZobristHash {
        ZobristHash { value: SIDE_HASH[0] }
    }
}
