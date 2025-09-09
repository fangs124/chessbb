use std::fmt::Debug;

use crate::PieceType;
use crate::bitboard::*;
use crate::square::Square;

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
                       castling type
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
11 - queen

note: castling move are encoded as follows
00 - White Kingside
01 - White Queenside
02 - Black Kingside
03 - Black Queenside
//                                                           */

//API traits: Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Default

//FIXME need to change visibility here... its only pub for debug
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChessMove {
    data: u16,
}

pub trait LexiOrd {
    fn lexi_cmp(&self, other: &Self) -> std::cmp::Ordering;
}
//needed to sort chess moves
impl LexiOrd for ChessMove {
    fn lexi_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.print_move().cmp(&other.print_move())
    }
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
pub(crate) enum MoveType {
    Normal,
    Castle(Castling),
    EnPassant,
    Promotion(PieceType),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Castling {
    Kingside(Side),
    Queenside(Side),
}

impl ChessMove {
    pub fn print_move(&self) -> String {
        if let MoveType::Promotion(piece) = self.move_type() {
            return format!("{}{}{}", SQUARE_SYM[self.source().to_usize()], SQUARE_SYM[self.target().to_usize()], piece.to_uci_char());
        } else {
            return format!("{}{}", SQUARE_SYM[self.source().to_usize()], SQUARE_SYM[self.target().to_usize()]);
        }
    }

    #[inline(always)]
    pub(crate) const fn source(&self) -> Square {
        Square::new((self.data & 0b000000_111111u16) as u8)
    }

    #[inline(always)]
    pub(crate) const fn target(&self) -> Square {
        Square::new(((self.data & 0b111111_000000u16) >> 6) as u8)
    }

    pub(crate) const fn move_type(&self) -> MoveType {
        let piece: PieceType = match ((self.data & 0b11_000000_000000u16) as usize) >> 12 {
            0b00 => PieceType::Knight,
            0b01 => PieceType::Bishop,
            0b10 => PieceType::Rook,
            0b11 => PieceType::Queen,
            _ => unreachable!(),
        };

        let castling: Castling = match ((self.data & 0b11_000000_000000u16) as usize) >> 12 {
            0b00 => Castling::Kingside(Side::White),
            0b01 => Castling::Queenside(Side::White),
            0b10 => Castling::Kingside(Side::Black),
            0b11 => Castling::Queenside(Side::Black),
            _ => unreachable!(),
        };

        match ((self.data & 0b11_00_000000_000000) as usize) >> 14 {
            0 => MoveType::Normal,
            1 => MoveType::Castle(castling),
            2 => MoveType::EnPassant,
            3 => MoveType::Promotion(piece),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub(crate) const fn set_source(&mut self, index: usize) {
        self.data &= (index & 0b111111) as u16;
    }

    #[inline(always)]
    pub(crate) const fn set_target(&mut self, index: usize) {
        self.data &= ((index << 6) & 0b111111_000000) as u16;
    }

    #[inline(always)]
    pub const fn from_raw(data: u16) -> ChessMove {
        ChessMove { data }
    }

    #[inline(always)]
    pub const fn data(&self) -> u16 {
        self.data
    }

    pub const fn new(s: Square, t: Square, m: MoveType) -> Self {
        // can't promote to king/pawn
        // ps: !matches!(...) is ugly
        assert!(matches!(m, MoveType::Promotion(PieceType::King)) == false);
        assert!(matches!(m, MoveType::Promotion(PieceType::Pawn)) == false);
        let mut data: u16 = ((s.to_usize() & 0b111111) | ((t.to_usize() << 6) & 0b111111_000000)) as u16;

        let move_type_data: usize = match m {
            MoveType::Normal => 0b00_00,
            MoveType::Castle(Castling::Kingside(Side::White)) => 0b01_00,
            MoveType::Castle(Castling::Queenside(Side::White)) => 0b01_01,
            MoveType::Castle(Castling::Kingside(Side::Black)) => 0b01_10,
            MoveType::Castle(Castling::Queenside(Side::Black)) => 0b01_11,
            MoveType::EnPassant => 0b10_00,
            MoveType::Promotion(PieceType::Knight) => 0b11_00,
            MoveType::Promotion(PieceType::Bishop) => 0b11_01,
            MoveType::Promotion(PieceType::Rook) => 0b11_10,
            MoveType::Promotion(PieceType::Queen) => 0b11_11,
            MoveType::Promotion(_) => unreachable!(),
        };

        data |= ((move_type_data << 12) & 0b11_11_000000_000000) as u16;
        Self { data }
    }

    pub(crate) const fn promotions(source: Square, target: Square) -> [ChessMove; 4] {
        return [
            ChessMove::new(source, target, MoveType::Promotion(PieceType::Queen)),
            ChessMove::new(source, target, MoveType::Promotion(PieceType::Knight)),
            ChessMove::new(source, target, MoveType::Promotion(PieceType::Bishop)),
            ChessMove::new(source, target, MoveType::Promotion(PieceType::Rook)),
        ];
    }

    pub(crate) const W_KINGSIDE_CASTLE: ChessMove =
        ChessMove::new(Square::W_KING_SQUARE, Square::W_KINGSIDE_CASTLE_SQUARE, MoveType::Castle(Castling::Kingside(Side::White)));

    pub(crate) const W_QUEENSIDE_CASTLE: ChessMove =
        ChessMove::new(Square::W_KING_SQUARE, Square::W_QUEENSIDE_CASTLE_SQUARE, MoveType::Castle(Castling::Queenside(Side::White)));

    pub(crate) const B_KINGSIDE_CASTLE: ChessMove =
        ChessMove::new(Square::B_KING_SQUARE, Square::B_KINGSIDE_CASTLE_SQUARE, MoveType::Castle(Castling::Kingside(Side::Black)));

    pub(crate) const B_QUEENSIDE_CASTLE: ChessMove =
        ChessMove::new(Square::B_KING_SQUARE, Square::B_QUEENSIDE_CASTLE_SQUARE, MoveType::Castle(Castling::Queenside(Side::Black)));
}

// //trying to see if type-encoded chess move would be nice
// pub struct _ChessMove {
//     source: _Square,
//     target: _Square,
//     move_ty: MoveType,
// }
// pub enum _Square {} //suppose we enumerate all 64 squares
