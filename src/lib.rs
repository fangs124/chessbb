mod bitboard;
mod chessmove;
mod constvec;

use crate::bitboard::*;

type ChessPiece = (Side, PieceType);

/* chessboard specific bitboard functions and definitions*/
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
    pub const fn to_char(&self) -> char {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

/* ChessBoard encodes the board-state of the game */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    pub piece_bbs: [BitBoard; 12],
    pub castle_bools: [bool; 4],
    pub enpassant_bb: BitBoard,
    pub check_bb: BitBoard, //piece locations causing the check
    pub side_to_move: Side,
    pub half_move_clock: u16,
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard::start_pos()
    }
}

impl ChessBoard {
    pub const fn start_pos() -> Self {
        Self {
            piece_bbs: bitboard::INITIAL_CHESS_POS,
            castle_bools: [true; 4],
            enpassant_bb: BitBoard::ZERO,
            side_to_move: Side::White,
            check_bb: BitBoard::ZERO,
            half_move_clock: 0,
        }
    }

    pub const fn blockers(&self) -> BitBoard {
        let mut i = 0;
        let mut bitboard: BitBoard = BitBoard::ZERO;
        while i < 12 {
            bitboard = bitboard.bit_or(&self.piece_bbs[i]);
            i += 1;
        }
        return bitboard;
    }

    pub const fn white_blockers(&self) -> BitBoard {
        let mut i = 0;
        let mut bitboard: BitBoard = BitBoard::ZERO;
        while i < 6 {
            bitboard = bitboard.bit_or(&self.piece_bbs[i]);
            i += 1;
        }
        return bitboard;
    }

    pub const fn black_blockers(&self) -> BitBoard {
        let mut i = 6;
        let mut bitboard: BitBoard = BitBoard::ZERO;
        while i < self.piece_bbs.len() {
            bitboard = bitboard.bit_or(&self.piece_bbs[i]);
            i += 1;
        }
        return bitboard;
    }

    pub const fn is_square_attacked(&self, square: usize, attacker_side: Side) -> bool {
        assert!(square < 64);
        let blockers = self.blockers();
        match attacker_side {
            Side::White => {
                return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[5])).is_not_zero()
                    || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[4])).is_not_zero()
                    || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[3])).is_not_zero()
                    || (get_knight_attack(square).bit_and(&self.piece_bbs[2])).is_not_zero()
                    || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[1])).is_not_zero()
                    || (get_king_attack(square).bit_and(&self.piece_bbs[0])).is_not_zero();
            }
            Side::Black => {
                return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[11])).is_not_zero()
                    || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[10])).is_not_zero()
                    || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[9])).is_not_zero()
                    || (get_knight_attack(square).bit_and(&self.piece_bbs[8])).is_not_zero()
                    || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[7])).is_not_zero()
                    || (get_king_attack(square).bit_and(&self.piece_bbs[6])).is_not_zero();
            }
        }
    }
}

/*
// generate all moves
static inline void generate_moves()
{
    // define source & target squares
    int source_square, int target_square;

    // define current piece's bitboard copy & it's attacks
    U64 bitboard, attacks;

    // loop over all the bitboards
    for (int piece = P; piece <= k; piece++)
    {
        // init piece bitboard copy
        bitboard = bitboards[piece];

        // generate white pawns & white king castling moves
        if (side == white)
        {

        }

        // generate black pawns & black king castling moves
        else
        {

        }

        // genarate knight moves

        // generate bishop moves

        // generate rook moves

        // generate queen moves

        // generate king moves
    }
}
*/
