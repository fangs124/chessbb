use std::fmt::{Debug, Display};
use std::ops::Not;

use crate::PieceType;
use crate::bitboard::*;
use crate::constvec::*;

/* indexing the 64-squares:
  |-----------------------| BLACK KING SIDE
8 |63 62 61 60 59 58 57 56|
7 |55 54 53 52 51 50 49 48|
6 |47 46 45 44 43 42 41 40|
5 |39 38 37 36 35 34 33 32|
4 |31 30 29 28 27 26 25 24| //30
3 |23 22 21 20 19 18 17 16| //20
2 |15 14 13 12 11 10  9  8|
1 | 7  6  5  4  3  2  1  0|
  |-----------------------| WHITE KING SIDE
    A  B  C  D  E  F  G  H                  */

/*  binary masks           description         hexidecimal masks
0000 0000 00XX XXXX    source square       0x3f
0000 XXXX XX00 0000    target square       0xfc0
00XX 0000 0000 0000    promoted piece data 0x3000
XX00 0000 0000 0000    move type           0xc000

note: move types are encoded as follows
00 - normal move
01 - castle move
10 - en passant
11 - promotion

note: promoted piece data are encoded as follows
00 - knight
01 - bishop
10 - rook
11 - queen                                                   */

//API traits: Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Default

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessMove {
    data: u16,
}

//impl Display for ChessMove {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        let s = self.print_move();
//        write!(f, "{}", s)
//    }
//}

//impl Debug for ChessMove {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        let mut s = self.print_move();
//        s.push_str(format!(" {:?}", self.move_type()).as_str());
//        write!(f, "{}", s)
//    }
//}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion(PieceType),
}

impl ChessMove {
    /* get functions */
    pub(crate) const fn source(&self) -> usize {
        ((self.data & 0b000000_111111u16) as usize) >> 0
    }

    pub(crate) const fn target(&self) -> usize {
        ((self.data & 0b111111_000000u16) as usize) >> 6
    }

    //pub(crate) const fn piece_data(&self) -> Option<PieceType> {
    //    if let MoveType::Promotion = self.move_type() {
    //        match ((self.data & 0b11_000000_000000u16) as usize) >> 12 {
    //            0b00 => Some(PieceType::Knight),
    //            0b01 => Some(PieceType::Bishop),
    //            0b10 => Some(PieceType::Rook),
    //            0b11 => Some(PieceType::Queen),
    //            _ => unreachable!(),
    //        }
    //    } else {
    //        None
    //    }
    //}

    pub(crate) const fn move_type(&self) -> MoveType {
        let piece: PieceType = match ((self.data & 0b11_000000_000000u16) as usize) >> 12 {
            0b00 => PieceType::Knight,
            0b01 => PieceType::Bishop,
            0b10 => PieceType::Rook,
            0b11 => PieceType::Queen,
            _ => unreachable!(),
        };

        match ((self.data & 0b11_00_000000_000000) as usize) >> 14 {
            0 => MoveType::Normal,
            1 => MoveType::Castle,
            2 => MoveType::EnPassant,
            3 => MoveType::Promotion(piece),
            _ => unreachable!(),
        }
    }

    /* set functions */
    pub(crate) const fn set_source(&mut self, index: usize) {
        self.data &= ((index << 0) & 0b111111) as u16;
    }

    pub(crate) const fn set_target(&mut self, index: usize) {
        self.data &= ((index << 6) & 0b111111_000000) as u16;
    }

    //pub(crate) const fn set_piece_data(&mut self, piece_data: Option<PieceType>) {
    //    //doesn't check: piece_data == None <-> move_type != Promotion
    //    if piece_data == None {
    //        return;
    //    } else {
    //        let piece_data: usize = match piece_data {
    //            Some(PieceType::Knight) => 0b00,
    //            Some(PieceType::Bishop) => 0b01,
    //            Some(PieceType::Rook) => 0b10,
    //            Some(PieceType::Queen) => 0b11,
    //            _ => panic!("set_piece_data error: invalid piece_data!"),
    //        };
    //        self.data &= ((piece_data << 12) & 0b11_00_000000_000000) as u16;
    //    }
    //}
    //
    //pub(crate) fn set_move_type(&mut self, move_type: MoveType) {
    //    let move_type_data = match move_type {
    //        MoveType::Normal => 0,
    //        MoveType::Castle => 1,
    //        MoveType::EnPassant => 2,
    //        MoveType::Promotion => 3,
    //    };
    //    self.data &= ((move_type_data << 14) & 0b11_00_000000_000000) as u16;
    //}

    /* helper functions */
    //fn set_data(&mut self, s: usize, t: usize, p: Option<PieceType>, m: MoveType) {
    //    assert!((p == None) == (m != MoveType::Promotion));
    //    self.set_source(s);
    //    self.set_target(t);
    //    self.set_piece_data(p);
    //    self.set_move_type(m);
    //}

    pub const fn new(s: usize, t: usize, m: MoveType) -> Self {
        //assert!((p == None) == (m != MoveType::Promotion));
        //hack: PartialEq can't be used in const fn.
        //match p {
        //    Some(_) => match m {
        //        MoveType::Promotion => {}
        //        _ => {
        //            panic!("ChessMove::new() error!")
        //        }
        //    },
        //    None => match m {
        //        MoveType::Promotion => {
        //            panic!("ChessMove::new() error!")
        //        }
        //        _ => {}
        //    },
        //}
        //assert!(p.is_some() == matches!(m, MoveType::Promotion));

        // can't promote to king/pawn
        // ps: !matches!(...) is ugly
        assert!(matches!(m, MoveType::Promotion(PieceType::King)) == false);
        assert!(matches!(m, MoveType::Promotion(PieceType::Pawn)) == false);
        let mut data: u16 = (((s << 0) & 0b111111) | ((t << 6) & 0b111111_000000)) as u16;

        let move_type_data: usize = match m {
            MoveType::Normal => 0b00_00,
            MoveType::Castle => 0b01_00,
            MoveType::EnPassant => 0b10_00,
            MoveType::Promotion(PieceType::Knight) => 0b11_00,
            MoveType::Promotion(PieceType::Bishop) => 0b11_01,
            MoveType::Promotion(PieceType::Rook) => 0b11_10,
            MoveType::Promotion(PieceType::Queen) => 0b11_11,
            MoveType::Promotion(_) => unreachable!(),
        };

        //if p.is_some() {
        //    let piece_data: usize = match p {
        //        Some(PieceType::Knight) => 0b00_00,
        //        Some(PieceType::Bishop) => 0b00_01,
        //        Some(PieceType::Rook) => 0b10,
        //        Some(PieceType::Queen) => 0b11,
        //        _ => panic!("set_piece_data error: invalid piece_data!"),
        //    };
        //    data |= ((piece_data << 12) & 0b00_11_000000_000000) as u16;
        //}
        //let move_type_data: usize = match m {
        //    MoveType::Normal => 0b00,
        //    MoveType::Castle => 0b01,
        //    MoveType::EnPassant => 0b10,
        //    MoveType::Promotion(piece) => 0b11,
        //};
        //
        //data |= ((move_type_data << 14) & 0b11_00_000000_000000) as u16;

        data |= ((move_type_data << 12) & 0b11_11_000000_000000) as u16;
        Self { data }
    }

    //note: this is uci format
    //examples
    //e7e8q for queen promotion, e2e4, etc.
    //pub fn print_move(&self) -> String {
    //    if self.piece().is_some() {
    //        let piece = self.piece().unwrap();
    //        format!("{}{}{}", SQUARE_SYM[self.source()], SQUARE_SYM[self.target()], piece.to_char())
    //    } else {
    //        format!("{}{}", SQUARE_SYM[self.source()], SQUARE_SYM[self.target()])
    //    }
    //}
}

/* ================ additional ChessMove-specific implementations ================ */
//pub type MoveVec = ConstVec<Option<ChessMove>, 256>;
//impl ConstDefault for Option<ChessMove> {
//    const DEFAULT: Self = None;
//}
//
//impl MoveVec {
//    pub const fn nth_move(&self, index: usize) -> ChessMove {
//        return match self.nth(index) {
//            Some(x) => x,
//            None => panic!(),
//        };
//    }
//
//    pub const fn new_promotions(&self, source: usize, target: usize) -> Self {
//        assert!(self.len() + 4 <= Self::MAX_CAPACITY);
//        let data: [Option<ChessMove>; 4] = [
//            Some(ChessMove::new(source, target, Some(PieceType::Queen), MoveType::Promotion)),
//            Some(ChessMove::new(source, target, Some(PieceType::Rook), MoveType::Promotion)),
//            Some(ChessMove::new(source, target, Some(PieceType::Bishop), MoveType::Promotion)),
//            Some(ChessMove::new(source, target, Some(PieceType::Knight), MoveType::Promotion)),
//        ];
//
//        self.append(&data, 4)
//    }
//    //new_raw(03, 01, None, MT::Castle),
//    pub const fn append_one_move(&self, s: usize, t: usize, p: Option<PieceType>, m: MoveType) -> MoveVec {
//        self.append_one(Some(ChessMove::new(s, t, p, m)))
//    }
//
//    pub fn to_vec(&self) -> Vec<ChessMove> {
//        self.data()[0..self.len()].into_iter().map(|x| x.unwrap()).collect()
//    }
//
//    pub fn sort(&mut self) {
//        let moves_vec: Vec<ChessMove> = self.to_vec();
//        let str_vec = moves_vec.clone().into_iter().map(|x| format!("{}", x)).collect::<Vec<String>>();
//        let mut pair_vec: Vec<(String, ChessMove)> = str_vec.into_iter().zip(moves_vec.into_iter()).collect();
//        pair_vec.sort();
//        for i in 0..self.len() {
//            self.set(i, Some(pair_vec[i].1));
//        }
//    }
//}
//
