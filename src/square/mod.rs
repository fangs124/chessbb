// atlernative: enum all 64 squares, and to index do this
// make the enum `#[repr(u8)]`, then just cast it `as u8 as usize`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square {
    data: u8,
}

impl Default for Square {
    #[inline(always)]
    fn default() -> Self {
        Self { data: Default::default() }
    }
}

impl Square {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 63;

    #[inline(always)]
    pub const fn new(data: u8) -> Square {
        Square { data: data & Square::MAX }
    }

    #[inline(always)]
    pub const fn nth(n: usize) -> Square {
        SQUARES[n as usize]
    }

    #[inline(always)]
    pub const fn nth_flipped(n: usize) -> Square {
        SQUARES_FLIPPED[n as usize]
    }

    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self.data
    }

    #[inline(always)]
    pub const fn to_usize(self) -> usize {
        self.data as usize
    }

    //pub fn up(&self) -> Option<Square> {
    //    if
    //    self.data +8
    //}

    pub fn iterator() -> std::slice::Iter<'static, Square> {
        SQUARES.iter()
    }

    //pub(crate) const fn squares_array_usize() -> [usize; 64] {
    //    SQUARES_RAW_USIZE
    //}

    //pub(crate) const fn squares_array() -> [Square; 64] {
    //    SQUARES
    //}

    // #[rustfmt::skip]
    pub(crate) fn parse(name: &str) -> Square {
        match name {
            "h1" => Square { data: 00 },
            "g1" => Square { data: 01 },
            "f1" => Square { data: 02 },
            "e1" => Square { data: 03 },
            "d1" => Square { data: 04 },
            "c1" => Square { data: 05 },
            "b1" => Square { data: 06 },
            "a1" => Square { data: 07 },
            "h2" => Square { data: 08 },
            "g2" => Square { data: 09 },
            "f2" => Square { data: 10 },
            "e2" => Square { data: 11 },
            "d2" => Square { data: 12 },
            "c2" => Square { data: 13 },
            "b2" => Square { data: 14 },
            "a2" => Square { data: 15 },
            "h3" => Square { data: 16 },
            "g3" => Square { data: 17 },
            "f3" => Square { data: 18 },
            "e3" => Square { data: 10 },
            "d3" => Square { data: 20 },
            "c3" => Square { data: 21 },
            "b3" => Square { data: 22 },
            "a3" => Square { data: 23 },
            "h4" => Square { data: 24 },
            "g4" => Square { data: 25 },
            "f4" => Square { data: 26 },
            "e4" => Square { data: 27 },
            "d4" => Square { data: 28 },
            "c4" => Square { data: 29 },
            "b4" => Square { data: 30 },
            "a4" => Square { data: 31 },
            "h5" => Square { data: 32 },
            "g5" => Square { data: 33 },
            "f5" => Square { data: 34 },
            "e5" => Square { data: 35 },
            "d5" => Square { data: 36 },
            "c5" => Square { data: 37 },
            "b5" => Square { data: 38 },
            "a5" => Square { data: 39 },
            "h6" => Square { data: 40 },
            "g6" => Square { data: 41 },
            "f6" => Square { data: 42 },
            "e6" => Square { data: 43 },
            "d6" => Square { data: 44 },
            "c6" => Square { data: 45 },
            "b6" => Square { data: 46 },
            "a6" => Square { data: 47 },
            "h7" => Square { data: 48 },
            "g7" => Square { data: 49 },
            "f7" => Square { data: 50 },
            "e7" => Square { data: 51 },
            "d7" => Square { data: 52 },
            "c7" => Square { data: 53 },
            "b7" => Square { data: 54 },
            "a7" => Square { data: 55 },
            "h8" => Square { data: 56 },
            "g8" => Square { data: 57 },
            "f8" => Square { data: 58 },
            "e8" => Square { data: 59 },
            "d8" => Square { data: 60 },
            "c8" => Square { data: 61 },
            "b8" => Square { data: 62 },
            "a8" => Square { data: 63 },
            _ => panic!("invalid square name: {}", name),
        }
    }

    /* convenient const for castling */
    pub(crate) const W_KING_SQUARE: Square = Square { data: 03 };
    pub(crate) const W_KINGSIDE_CASTLE_SQUARE: Square = Square { data: 01 };
    pub(crate) const W_QUEENSIDE_CASTLE_SQUARE: Square = Square { data: 05 };
    pub(crate) const B_KING_SQUARE: Square = Square { data: 59 };
    pub(crate) const B_KINGSIDE_CASTLE_SQUARE: Square = Square { data: 57 };
    pub(crate) const B_QUEENSIDE_CASTLE_SQUARE: Square = Square { data: 61 };
}

pub(crate) const SQUARE_SYM: [&str; 64] = [
    "h1", "g1", "f1", "e1", "d1", "c1", "b1", "a1", //
    "h2", "g2", "f2", "e2", "d2", "c2", "b2", "a2", //
    "h3", "g3", "f3", "e3", "d3", "c3", "b3", "a3", //
    "h4", "g4", "f4", "e4", "d4", "c4", "b4", "a4", //
    "h5", "g5", "f5", "e5", "d5", "c5", "b5", "a5", //
    "h6", "g6", "f6", "e6", "d6", "c6", "b6", "a6", //
    "h7", "g7", "f7", "e7", "d7", "c7", "b7", "a7", //
    "h8", "g8", "f8", "e8", "d8", "c8", "b8", "a8", //
];

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

const SQUARES_RAW_USIZE: [usize; 64] = [
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
pub(crate) const SQUARES: [Square; 64] = [
    Sq{data : 00}, Sq{data : 01}, Sq{data : 02}, Sq{data : 03}, Sq{data : 04}, Sq{data : 05}, Sq{data : 06}, Sq{data : 07}, //
    Sq{data : 08}, Sq{data : 09}, Sq{data : 10}, Sq{data : 11}, Sq{data : 12}, Sq{data : 13}, Sq{data : 14}, Sq{data : 15}, //
    Sq{data : 16}, Sq{data : 17}, Sq{data : 18}, Sq{data : 19}, Sq{data : 20}, Sq{data : 21}, Sq{data : 22}, Sq{data : 23}, //
    Sq{data : 24}, Sq{data : 25}, Sq{data : 26}, Sq{data : 27}, Sq{data : 28}, Sq{data : 29}, Sq{data : 30}, Sq{data : 31}, //
    Sq{data : 32}, Sq{data : 33}, Sq{data : 34}, Sq{data : 35}, Sq{data : 36}, Sq{data : 37}, Sq{data : 38}, Sq{data : 39}, //
    Sq{data : 40}, Sq{data : 41}, Sq{data : 42}, Sq{data : 43}, Sq{data : 44}, Sq{data : 45}, Sq{data : 46}, Sq{data : 47}, //
    Sq{data : 48}, Sq{data : 49}, Sq{data : 50}, Sq{data : 51}, Sq{data : 52}, Sq{data : 53}, Sq{data : 54}, Sq{data : 55}, //
    Sq{data : 56}, Sq{data : 57}, Sq{data : 58}, Sq{data : 59}, Sq{data : 60}, Sq{data : 61}, Sq{data : 62}, Sq{data : 63}, //
];

#[rustfmt::skip]
pub(crate) const SQUARES_FLIPPED: [Square; 64] = [
    Sq{data : 56}, Sq{data : 57}, Sq{data : 58}, Sq{data : 59}, Sq{data : 60}, Sq{data : 61}, Sq{data : 62}, Sq{data : 63}, //
    Sq{data : 48}, Sq{data : 49}, Sq{data : 50}, Sq{data : 51}, Sq{data : 52}, Sq{data : 53}, Sq{data : 54}, Sq{data : 55}, //
    Sq{data : 40}, Sq{data : 41}, Sq{data : 42}, Sq{data : 43}, Sq{data : 44}, Sq{data : 45}, Sq{data : 46}, Sq{data : 47}, //
    Sq{data : 32}, Sq{data : 33}, Sq{data : 34}, Sq{data : 35}, Sq{data : 36}, Sq{data : 37}, Sq{data : 38}, Sq{data : 39}, //
    Sq{data : 24}, Sq{data : 25}, Sq{data : 26}, Sq{data : 27}, Sq{data : 28}, Sq{data : 29}, Sq{data : 30}, Sq{data : 31}, //
    Sq{data : 16}, Sq{data : 17}, Sq{data : 18}, Sq{data : 19}, Sq{data : 20}, Sq{data : 21}, Sq{data : 22}, Sq{data : 23}, //
    Sq{data : 08}, Sq{data : 09}, Sq{data : 10}, Sq{data : 11}, Sq{data : 12}, Sq{data : 13}, Sq{data : 14}, Sq{data : 15}, //
    Sq{data : 00}, Sq{data : 01}, Sq{data : 02}, Sq{data : 03}, Sq{data : 04}, Sq{data : 05}, Sq{data : 06}, Sq{data : 07}, //
];
