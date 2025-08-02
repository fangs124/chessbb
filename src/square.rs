use std::ops::Range;

use crate::bitboard::BitBoard;

// atlernative: enum all 64 squares, and to index do this
// make the enum `#[repr(u8)]`, then just cast it `as u8 as usize`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square {
    data: u8,
}

impl Default for Square {
    fn default() -> Self {
        Self { data: Default::default() }
    }
}

impl Square {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 63;
    pub const fn new(data: u8) -> Square {
        Square { data: data & Square::MAX }
    }

    pub const fn to_u8(&self) -> u8 {
        self.data
    }

    pub const fn to_index(&self) -> usize {
        self.data as usize
    }

    pub fn iterator() -> std::slice::Iter<'static, Square> {
        // english grammar is broken
        SQUARES.iter()
        //(Range { start: 0u8, end: 64u8 }).into_iter()
    }

    pub(crate) const W_KING_SQUARE: Square = Square { data: 03 };
    pub(crate) const W_KINGSIDE_CASTLE_SQUARE: Square = Square { data: 01 };
    pub(crate) const W_QUEENSIDE_CASTLE_SQUARE: Square = Square { data: 05 };
    pub(crate) const B_KING_SQUARE: Square = Square { data: 59 };
    pub(crate) const B_KINGSIDE_CASTLE_SQUARE: Square = Square { data: 57 };
    pub(crate) const B_QUEENSIDE_CASTLE_SQUARE: Square = Square { data: 61 };
}

const SQUARES_RAW: [u8; 64] = [
    00, 01, 02, 03, 04, 05, 06, 07, //
    08, 09, 10, 11, 12, 13, 14, 15, //
    16, 17, 18, 19, 20, 21, 22, 23, //
    24, 25, 26, 27, 28, 29, 30, 31, //
    32, 33, 34, 35, 36, 37, 38, 39, //
    40, 41, 42, 43, 44, 45, 46, 47, //
    48, 49, 50, 51, 52, 53, 54, 55, //
    56, 57, 58, 59, 60, 61, 62, 63, //
];

type Sq = Square;
#[rustfmt::skip]
const SQUARES: [Square; 64] = [
    Sq{data : 00}, Sq{data : 01}, Sq{data : 02}, Sq{data : 03}, Sq{data : 04}, Sq{data : 05}, Sq{data : 06}, Sq{data : 07}, //
    Sq{data : 08}, Sq{data : 09}, Sq{data : 10}, Sq{data : 11}, Sq{data : 12}, Sq{data : 13}, Sq{data : 14}, Sq{data : 15}, //
    Sq{data : 16}, Sq{data : 17}, Sq{data : 18}, Sq{data : 19}, Sq{data : 20}, Sq{data : 21}, Sq{data : 22}, Sq{data : 23}, //
    Sq{data : 24}, Sq{data : 25}, Sq{data : 26}, Sq{data : 27}, Sq{data : 28}, Sq{data : 29}, Sq{data : 30}, Sq{data : 31}, //
    Sq{data : 32}, Sq{data : 33}, Sq{data : 34}, Sq{data : 35}, Sq{data : 36}, Sq{data : 37}, Sq{data : 38}, Sq{data : 39}, //
    Sq{data : 40}, Sq{data : 41}, Sq{data : 42}, Sq{data : 43}, Sq{data : 44}, Sq{data : 45}, Sq{data : 46}, Sq{data : 47}, //
    Sq{data : 48}, Sq{data : 49}, Sq{data : 50}, Sq{data : 51}, Sq{data : 52}, Sq{data : 53}, Sq{data : 54}, Sq{data : 55}, //
    Sq{data : 56}, Sq{data : 57}, Sq{data : 58}, Sq{data : 59}, Sq{data : 60}, Sq{data : 61}, Sq{data : 62}, Sq{data : 63}, //
];
