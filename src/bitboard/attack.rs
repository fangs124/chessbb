mod init;
mod magic;

use crate::Side;
use crate::bitboard::BitBoard;
use crate::bitboard::attack::init::*;
use crate::bitboard::attack::magic::*;
use crate::square::Square;

/* last revised: 8/8/2025 */

const W_PAWN_ATTACKS: [BitBoard; 64] = init_pawn_attack(Side::White);
const B_PAWN_ATTACKS: [BitBoard; 64] = init_pawn_attack(Side::Black);
const KNIGHT_ATTACKS: [BitBoard; 64] = init_knight_attack();
const KING_ATTACKS: [BitBoard; 64] = init_king_attack();

#[inline(always)]
pub const fn get_pawn_attack(square: Square, side: Side) -> BitBoard {
    match side {
        Side::White => W_PAWN_ATTACKS[square.to_index()],
        Side::Black => B_PAWN_ATTACKS[square.to_index()],
    }
}

#[inline(always)]
pub const fn get_w_pawn_attack(square: Square) -> BitBoard {
    W_PAWN_ATTACKS[square.to_index()]
}

#[inline(always)]
pub const fn get_b_pawn_attack(square: Square) -> BitBoard {
    B_PAWN_ATTACKS[square.to_index()]
}

#[inline(always)]
pub const fn get_knight_attack(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square.to_index()]
}

#[inline(always)]
pub const fn get_king_attack(square: Square) -> BitBoard {
    KING_ATTACKS[square.to_index()]
}

#[inline(always)]
pub const fn get_bishop_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let m = magic_index(
        BISHOP_MAGICS[square.to_index()],
        blockers.bit_and(&BISHOP_MBB_MASK[square.to_index()]),
        BISHOP_OCC_BITCOUNT[square.to_index()],
    );
    return BISHOP_ATTACKS_MBB[square.to_index()][m];
}

#[inline(always)]
pub const fn get_rook_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let m = magic_index(
        ROOK_MAGICS[square.to_index()],
        blockers.bit_and(&ROOK_MBB_MASK[square.to_index()]),
        ROOK_OCC_BITCOUNT[square.to_index()],
    );
    return ROOK_ATTACKS_MBB[square.to_index()][m];
}

#[inline(always)]
pub const fn get_queen_attack(square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard { data: get_bishop_attack(square, blockers).data | get_rook_attack(square, blockers).data }
}
