use std::{fmt::Display, slice::Iter};

use crate::square::Square;
pub use attack::*;

pub mod bit_ops;
pub mod attack;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard {
    data: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub(crate) const fn update(&self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
    const SIDES: [Side; 2] = [Side::White, Side::Black];

    pub fn iterator() -> Iter<'static, Side> {
        return Side::SIDES.iter();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    const PIECETYPES: [PieceType; 6] =
            [PieceType::King ,PieceType::Queen, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Pawn];
    pub fn iterator() -> std::slice::Iter<'static, PieceType> {
        
        PieceType::PIECETYPES.iter()
    }

    pub(crate) const fn to_uci_char(&self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }
}

pub type ChessPiece = (Side, PieceType);

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for i in 0..8u64 {
            s.push_str(&format!(
                "{:08b}",
                (self.data & (0xFFu64 << (8 * (7 - i)))) >> (8 * (7 - i))
            ));
            s.push('\n');
        }
        write!(f, "{s}")
    }
}

impl BitBoard {
    #[inline(always)]
    pub(crate) const fn new(data: u64) -> Self {
        Self { data }
    }

    pub(crate) const ZERO: BitBoard = BitBoard { data: 0u64 };
    pub(crate) const ONES: BitBoard = BitBoard { data: u64::MAX };

    //creates a bitboard with a a non-zero bit in the n-th place
    #[inline(always)]
    pub(crate) const fn nth(sq: Square) -> Self {
        Self { data: 1u64 << sq.to_usize() }
    }

    #[inline(always)]
    pub(crate) const fn nth_is_zero(&self, sq: Square) -> bool {
        match self.data & (1u64 << sq.to_usize()) {
            0 => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub(crate) const fn nth_is_not_zero(&self, sq: Square) -> bool {
        match self.data & (1u64 << sq.to_usize()) {
            0 => false,
            _ => true,
        }
    }

    #[inline(always)]
    pub(crate) const fn is_zero(&self) -> bool {
        self.data == 0u64
    }

    #[inline(always)]
    pub(crate) const fn is_not_zero(&self) -> bool {
        self.data != 0u64
    }

    #[inline(always)]
    pub(crate) const fn set_bit(&mut self, square: Square) { 
        self.data |= 1u64 << square.to_usize();
    }

    #[inline(always)]
    pub(crate) const fn get_bit(&self, i: usize) -> BitBoard {
        BitBoard {
            data: self.data & (1u64 << i),
        }
    }

    #[inline(always)]
    pub(crate) const fn pop_bit(&mut self, square: Square) {
        self.data &=!(1u64 << square.to_usize());
    }

    //pub(crate) const fn get_bit_data(&self, i: usize) -> u64 {
    //    self.data & (1u64 << i)
    //}
    //
    //pub(crate) const fn pop_bit_data(&self, i: usize) -> u64 {
    //    self.data & !(1u64 << i)
    //}

    // index of least-significant-bit (lsb)
    #[inline(always)]
    pub(crate) const fn lsb_index(&self) -> Option<usize> {
        if self.data == 0u64 {
            return None;
        } else {
            return Some(self.data.trailing_zeros() as usize);
        }
    }

    // square of least-significant-bit (lsb)
    #[inline(always)]
    pub(crate) const fn lsb_square(&self) -> Option<Square> {
        if self.data == 0u64 {
            return None;
        } else {
            return Some(Square::new(self.data.trailing_zeros() as u8));
        }
    }

    #[inline(always)]
    pub(crate) const fn count_ones(&self) -> u32 {
        self.data.count_ones()
    }

    #[inline(always)]
    pub(crate) const fn bit_and(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data & other.data,
        }
    }

    #[inline(always)]
    pub(crate) const fn bit_or(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data | other.data,
        }
    }

    #[inline(always)]
    pub(crate) const fn bit_xor(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data ^ other.data,
        }
    }

    #[inline(always)]
    pub(crate) const fn bit_not(&self) -> BitBoard {
        BitBoard { data: !self.data }
    }

    #[inline(always)]
    pub(crate) const fn flip(&self) -> Self {
        BitBoard { data: self.data.swap_bytes() }
    }
}


pub(crate) static RAYS: [[BitBoard; 64]; 64] = rays();

const fn rays() -> [[BitBoard; 64]; 64] {
    let mut rays: [[BitBoard; 64]; 64] = [[BitBoard::ZERO; 64]; 64];
    let mut i: usize = 0;
    while i < 64 {
        let i_square =  Square::new(i as u8);
        let mut j: usize = 0;
        while j < 64 {
            let j_square = Square::new(j as u8);
            let squares = BitBoard { data: (1u64 << i) | (1u64 << j) };
            if (ROWS[i] == ROWS[j]) || (COLS[i] == COLS[j]) {
                rays[i][j].data = get_rook_attack(i_square, squares).data & get_rook_attack(j_square, squares).data;
            } else if (DDIAG[i] == DDIAG[j]) || (ADIAG[i] == ADIAG[j]) {
                rays[i][j].data = get_bishop_attack(i_square, squares).data & get_bishop_attack(j_square, squares).data;
            }
            j += 1;
        }
        i += 1;
    }
    rays
}

/* ==== constants and supporting functions ==== */
pub(super) const ASCII_SYM: [char; 12] = ['K', 'Q', 'N', 'B', 'R', 'P', 'k', 'q', 'n', 'b', 'r', 'p'];
pub(super) const UNICODE_SYM: [char; 12] = ['♚', '♛', '♞', '♝', '♜', '♟', '♔', '♕', '♘', '♗', '♖', '♙'];

pub(crate) const W_KING_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000110);
pub(crate) const W_QUEEN_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00110000);
pub(crate) const B_KING_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub(crate) const B_QUEEN_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

#[inline(always)]
pub(crate) const fn is_same_diag(source: Square, target: Square) -> bool {
   (DDIAG[source.to_usize()] == DDIAG[target.to_usize()]) || (ADIAG[source.to_usize()] == ADIAG[target.to_usize()])
}

#[inline(always)]
pub(crate) const fn is_same_adiag(source: Square, target: Square) -> bool {
    ADIAG[source.to_usize()] == ADIAG[target.to_usize()]
}

#[inline(always)]
pub(crate) const fn is_same_ddiag(source: Square, target: Square) -> bool {
    DDIAG[source.to_usize()] == DDIAG[target.to_usize()]
}

#[inline(always)]
pub(crate) const fn is_same_col(source: Square, target: Square) -> bool {
   COLS[source.to_usize()] == COLS[target.to_usize()]
}

#[inline(always)]
pub(crate) const fn is_same_row(source: Square, target: Square) -> bool {
   ROWS[source.to_usize()] == ROWS[target.to_usize()]
}
/* ==== labels ==== */

/* indexing the 64-squares:
   -----------------------
8 |63 62 61 60 59 58 57 56|
7 |55 54 53 52 51 50 49 48|
6 |47 46 45 44 43 42 41 40|
5 |39 38 37 36 35 34 33 32|
4 |31 30 29 28 27 26 25 24|
3 |23 22 21 20 19 18 17 16|
2 |15 14 13 12 11 10  9  8|
1 | 7  6  5  4  3  2  1  0|
   -----------------------
    A  B  C  D  E  F  G  H */

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

pub(crate) const SQUARE_SYM_REV: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", //
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", //
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", //
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", //
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", //
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", //
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", //
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", //
];

pub(crate) const RANK_CHAR: [char; 64] = [
    '1', '1', '1', '1', '1', '1', '1', '1', //
    '2', '2', '2', '2', '2', '2', '2', '2', //
    '3', '3', '3', '3', '3', '3', '3', '3', //
    '4', '4', '4', '4', '4', '4', '4', '4', //
    '5', '5', '5', '5', '5', '5', '5', '5', //
    '6', '6', '6', '6', '6', '6', '6', '6', //
    '7', '7', '7', '7', '7', '7', '7', '7', //
    '8', '8', '8', '8', '8', '8', '8', '8', //
];

pub(crate) const FILE_CHAR: [char; 64] = [
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
    'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a', //
];

pub(crate) const ROWS: [usize; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    2, 2, 2, 2, 2, 2, 2, 2, //
    3, 3, 3, 3, 3, 3, 3, 3, //
    4, 4, 4, 4, 4, 4, 4, 4, //
    5, 5, 5, 5, 5, 5, 5, 5, //
    6, 6, 6, 6, 6, 6, 6, 6, //
    7, 7, 7, 7, 7, 7, 7, 7, //
];

pub(crate) const COLS: [usize; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
    0, 1, 2, 3, 4, 5, 6, 7, //
];

pub(crate) const DDIAG: [usize; 64] = [
    07, 08, 09, 10, 11, 12, 13, 14, //
    06, 07, 08, 09, 10, 11, 12, 13, //
    05, 06, 07, 08, 09, 10, 11, 12, //
    04, 05, 06, 07, 08, 09, 10, 11, //
    03, 04, 05, 06, 07, 08, 09, 10, //
    02, 03, 04, 05, 06, 07, 08, 09, //
    01, 02, 03, 04, 05, 06, 07, 08, //
    00, 01, 02, 03, 04, 05, 06, 07, //
];

pub(crate) const ADIAG: [usize; 64] = [
    00, 01, 02, 03, 04, 05, 06, 07, //
    01, 02, 03, 04, 05, 06, 07, 08, //
    02, 03, 04, 05, 06, 07, 08, 09, //
    03, 04, 05, 06, 07, 08, 09, 10, //
    04, 05, 06, 07, 08, 09, 10, 11, //
    05, 06, 07, 08, 09, 10, 11, 12, //
    06, 07, 08, 09, 10, 11, 12, 13, //
    07, 08, 09, 10, 11, 12, 13, 14, //
];


/* ==== macros ==== */

#[rustfmt::skip]
#[macro_export] 
macro_rules! opt_cpt {
    (K) => {Some((Side::White, PieceType::King  ))};
    (Q) => {Some((Side::White, PieceType::Queen ))};
    (N) => {Some((Side::White, PieceType::Knight))};
    (B) => {Some((Side::White, PieceType::Bishop))};
    (R) => {Some((Side::White, PieceType::Rook  ))};
    (P) => {Some((Side::White, PieceType::Pawn  ))};
    (k) => {Some((Side::Black, PieceType::King  ))};
    (q) => {Some((Side::Black, PieceType::Queen ))};
    (n) => {Some((Side::Black, PieceType::Knight))};
    (b) => {Some((Side::Black, PieceType::Bishop))};
    (r) => {Some((Side::Black, PieceType::Rook  ))};
    (p) => {Some((Side::Black, PieceType::Pawn  ))};
    (_) => {None};
}


#[rustfmt::skip]
#[macro_export]
macro_rules! cpt {
    (K) => {(Side::White, PieceType::King  )};
    (Q) => {(Side::White, PieceType::Queen )};
    (N) => {(Side::White, PieceType::Knight)};
    (B) => {(Side::White, PieceType::Bishop)};
    (R) => {(Side::White, PieceType::Rook  )};
    (P) => {(Side::White, PieceType::Pawn  )};
    (k) => {(Side::Black, PieceType::King  )};
    (q) => {(Side::Black, PieceType::Queen )};
    (n) => {(Side::Black, PieceType::Knight)};
    (b) => {(Side::Black, PieceType::Bishop)};
    (r) => {(Side::Black, PieceType::Rook  )};
    (p) => {(Side::Black, PieceType::Pawn  )};
}


#[rustfmt::skip]
#[macro_export]
macro_rules! cpt_index {
    (K) => {(00)};
    (Q) => {(01)};
    (N) => {(02)};
    (B) => {(03)};
    (R) => {(04)};
    (P) => {(05)};
    (k) => {(06)};
    (q) => {(07)};
    (n) => {(08)};
    (b) => {(09)};
    (r) => {(10)};
    (p) => {(11)};
}

pub const fn cp_index(data: ChessPiece) -> usize {
    match data {
        cpt!(K) => 00,
        cpt!(Q) => 01,
        cpt!(N) => 02,
        cpt!(B) => 03,
        cpt!(R) => 04,
        cpt!(P) => 05,
        cpt!(k) => 06,
        cpt!(q) => 07,
        cpt!(n) => 08,
        cpt!(b) => 09,
        cpt!(r) => 10,
        cpt!(p) => 11,
    }
}

pub const fn sym_index(c: char) -> usize {
    match c {
        'K' => 0,
        'Q' => 1,
        'N' => 2,
        'B' => 3,
        'R' => 4,
        'P' => 5,
        'k' => 6,
        'q' => 7,
        'n' => 8,
        'b' => 9,
        'r' => 10,
        'p' => 11,
        _ => panic!("sym_index error: invalid char!"),
    }
}