use std::fmt::Display;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, Not};

use crate::square::Square;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct BitBoard {
    data: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate)enum Side {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    //this is a UCI-thing
    //pub const fn to_char(&self) -> char {
    //    match self {
    //        PieceType::Pawn => 'p',
    //        PieceType::Knight => 'n',
    //        PieceType::Bishop => 'b',
    //        PieceType::Rook => 'r',
    //        PieceType::Queen => 'q',
    //        PieceType::King => 'k',
    //    }
    //}

    pub(crate) fn iterator() -> std::slice::Iter<'static, PieceType> {
        const PIECETYPES: [PieceType; 6] =
            [PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King];
        PIECETYPES.iter()
    }
}



pub(crate) type ChessPiece = (Side, PieceType);

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for i in 0..8u64 {
            s.push_str(&format!(
                "{:08b}",
                (self.data & (0xFFu64 << 8 * (7 - i))) >> 8 * (7 - i)
            ));
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}

/* ==== bit ops ==== */

impl BitAnd for BitBoard {
    type Output = BitBoard;
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard {
            data: self.data & rhs.data,
        }
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
        BitBoard {
            data: self.data | rhs.data,
        }
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
        BitBoard {
            data: self.data ^ rhs.data,
        }
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

impl BitBoard {
    pub(crate) const fn new(data: u64) -> Self {
        Self { data }
    }

    pub(crate) const ZERO: BitBoard = BitBoard { data: 0u64 };
    pub(crate) const ONES: BitBoard = BitBoard { data: u64::MAX };

    //creates a bitboard with a a non-zero bit in the n-th place
    //pub(crate) const fn nth(n: usize) -> Self {
    //    Self { data: 1u64 << n }
    //}

    pub(crate) const fn nth(sq: Square) -> Self {
        Self { data: 1u64 << sq.to_index() }
    }


    pub const fn nth_is_zero(&self, sq: Square) -> bool {
        match self.data & (1u64 << sq.to_index()) {
            0 => true,
            _ => false,
        }
    }

    pub const fn nth_is_not_zero(&self, sq: Square) -> bool {
        match self.data & (1u64 << sq.to_index()) {
            0 => false,
            _ => true,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.data == 0u64
    }

    pub const fn is_not_zero(&self) -> bool {
        self.data != 0u64
    }

    pub(crate)  fn set_bit(&mut self, i: usize) {
        self.data = self.data | 1u64 << i;
    }

    pub(crate) const fn get_bit(&self, i: usize) -> BitBoard {
        BitBoard {
            data: self.data & (1u64 << i),
        }
    }

    pub(crate) const fn pop_bit(&self, square: Square) -> BitBoard {
        BitBoard {
            data: self.data & !(1u64 << square.to_index()),
        }
    }

    pub(crate) const fn get_bit_data(&self, i: usize) -> u64 {
        self.data & (1u64 << i)
    }

    pub(crate) const fn pop_bit_data(&self, i: usize) -> u64 {
        self.data & !(1u64 << i)
    }

    // index of least-significant-bit (lsb)
    pub(crate) const fn lsb_index(&self) -> Option<usize> {
        if self.data == 0u64 {
            return None;
        } else {
            return Some(self.data.trailing_zeros() as usize);
        }
    }

    // square of least-significant-bit (lsb)
    pub(crate) const fn lsb_square(&self) -> Option<Square> {
        if self.data == 0u64 {
            return None;
        } else {
            return Some(Square::new(self.data.trailing_zeros() as u8));
        }
    }

    pub(crate) const fn count_ones(&self) -> u32 {
        self.data.count_ones()
    }

    pub(crate) const fn bit_and(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data & other.data,
        }
    }

    pub(crate) const fn bit_or(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data | other.data,
        }
    }

    pub(crate) const fn bit_xor(&self, other: &BitBoard) -> BitBoard {
        BitBoard {
            data: self.data ^ other.data,
        }
    }

    pub(crate) const fn bit_not(&self) -> BitBoard {
        BitBoard { data: !self.data }
    }
}

const W_PAWN_ATTACKS: [BitBoard; 64] = pawn_attack(Side::White);
const B_PAWN_ATTACKS: [BitBoard; 64] = pawn_attack(Side::Black);
const KNIGHT_ATTACKS: [BitBoard; 64] = knight_attack();
const KING_ATTACKS: [BitBoard; 64] = king_attack();
const BISHOP_MBB_MASK: [BitBoard; 64] = bishop_mbb_mask();
const ROOK_MBB_MASK: [BitBoard; 64] = rook_mbb_mask();
const BISHOP_ATTACKS_MBB: [[BitBoard; 1 << 9]; 64] = BISHOP;
const ROOK_ATTACKS_MBB: [[BitBoard; 1 << 12]; 64] = ROOK;
pub(crate) const RAYS: [[BitBoard; 64]; 64] = rays();

const fn pawn_attack(side: Side) -> [BitBoard; 64] {
    let mut i: usize = 0;
    let mut attack_array: [BitBoard; 64] = [BitBoard::ZERO; 64];
    while i < 64usize {
        let mut data: u64 = 0u64;
        match side {
            Side::White => {
                if i < 56 {
                    if i % 8 > 0 {
                        data |= (1u64 << i) << 7
                    }
                    if i % 8 < 7 {
                        data |= (1u64 << i) << 9
                    }
                }
            }
            Side::Black => {
                if i > 7 {
                    if i % 8 > 0 {
                        data |= (1u64 << i) >> 9
                    }
                    if i % 8 < 7 {
                        data |= (1u64 << i) >> 7
                    }
                }
            }
        }
        attack_array[i] = BitBoard { data };
        i += 1;
    }
    return attack_array;
}

const fn knight_attack() -> [BitBoard; 64] {
    let mut i: usize = 0;
    let mut attack_array: [BitBoard; 64] = [BitBoard::ZERO; 64];
    while i < 64usize {
        let mut data: u64 = 0u64;
        if i < 48 {
            if i % 8 < 7 {
                //up left is "<< 17"
                data |= (1u64 << i) << 17
            }
            if i % 8 > 0 {
                //up right is "<< 15"
                data |= (1u64 << i) << 15
            }
        }
        if i < 56 {
            if i % 8 < 6 {
                //left up is "<< 10"
                data |= (1u64 << i) << 10
            }
            if i % 8 > 1 {
                //right up is "<<  6"
                data |= (1u64 << i) << 6
            }
        }
        if i > 7 {
            if i % 8 < 6 {
                //left down is ">> 6"
                data |= (1u64 << i) >> 6
            }
            if i % 8 > 1 {
                //right down is ">> 10"
                data |= (1u64 << i) >> 10
            }
        }
        if i > 15 {
            if i % 8 < 7 {
                //down left is ">> 15"
                data |= (1u64 << i) >> 15
            }
            if i % 8 > 0 {
                //down right is ">> 17"
                data |= (1u64 << i) >> 17
            }
        }
        attack_array[i] = BitBoard { data };
        i += 1;
    }
    return attack_array;
}

const fn king_attack() -> [BitBoard; 64] {
    let mut i: usize = 0;
    let mut attack_array: [BitBoard; 64] = [BitBoard::ZERO; 64];
    while i < 64usize {
        let mut data: u64 = 0u64;
        if i < 56 {
            //up
            data |= (1u64 << i) << 8;
        }
        if i > 7 {
            //down
            data |= (1u64 << i) >> 8;
        }
        if i % 8 > 0 {
            //right
            data |= (1u64 << i) >> 1;
        }
        if i % 8 < 7 {
            //left
            data |= (1u64 << i) << 1;
        }
        if i < 56 && i % 8 > 0 {
            //up right
            data |= ((1u64 << i) << 8) >> 1;
        }
        if i < 56 && i % 8 < 7 {
            //up left
            data |= ((1u64 << i) << 8) << 1;
        }
        if i > 7 && i % 8 > 0 {
            //down right
            data |= ((1u64 << i) >> 8) >> 1;
        }
        if i > 7 && i % 8 < 7 {
            //down left
            data |= ((1u64 << i) >> 8) << 1;
        }
        attack_array[i] = BitBoard { data };
        i += 1;
    }
    return attack_array;
}

const fn naive_bishop_attack(i: usize, blockers: BitBoard) -> BitBoard {
    let i_rank: isize = (i as isize) / 8isize;
    let i_file: isize = (i as isize) % 8isize;
    let mut j: isize = 1;
    let mut data: u64 = 0u64;
    let mut ul_is_blocked: bool = false;
    let mut dl_is_blocked: bool = false;
    let mut ur_is_blocked: bool = false;
    let mut dr_is_blocked: bool = false;
    while j <= 7 {
        //    up left direction: (+,+)
        if i_rank + j <= 7 && i_file + j <= 7 {
            if !ul_is_blocked {
                data |= 1u64 << (i_rank + j) * 8 + (i_file + j);
                if i_rank + j < 7 && i_file + j < 7 {
                    if 1u64 << (i_rank + j) * 8 + (i_file + j) & blockers.data
                        != BitBoard::ZERO.data
                    {
                        ul_is_blocked = true;
                    }
                }
            }
        }

        //  down left direction: (-,+)
        if 0 <= i_rank - j && i_file + j <= 7 {
            if !dl_is_blocked {
                data |= 1u64 << (i_rank - j) * 8 + (i_file + j);
                if 0 < i_rank - j && i_file + j < 7 {
                    if 1u64 << (i_rank - j) * 8 + (i_file + j) & blockers.data
                        != BitBoard::ZERO.data
                    {
                        dl_is_blocked = true;
                    }
                }
            }
        }

        //    up right direction: (+,-)
        if i_rank + j <= 7 && 0 <= i_file - j {
            if !ur_is_blocked {
                data |= 1u64 << (i_rank + j) * 8 + (i_file - j);
                if i_rank + j < 7 && 0 < i_file - j {
                    if 1u64 << (i_rank + j) * 8 + (i_file - j) & blockers.data
                        != BitBoard::ZERO.data
                    {
                        ur_is_blocked = true;
                    }
                }
            }
        }

        //  down right direction: (-,-)
        if 0 <= i_rank - j && 0 <= i_file - j {
            if !dr_is_blocked {
                data |= 1u64 << (i_rank - j) * 8 + (i_file - j);
                if 0 < i_rank - j && 0 < i_file - j {
                    if 1u64 << (i_rank - j) * 8 + (i_file - j) & blockers.data
                        != BitBoard::ZERO.data
                    {
                        dr_is_blocked = true;
                    }
                }
            }
        }
        j += 1
    }

    BitBoard { data }
}

const fn naive_rook_attack(i: usize, blockers: BitBoard) -> BitBoard {
    let i_rank: isize = (i as isize) / 8isize; // row
    let i_file: isize = (i as isize) % 8isize; // collumn
    let mut data: u64 = 0u64;

    let mut j: isize = 1;
    let mut r_is_blocked: bool = false;
    let mut l_is_blocked: bool = false;
    let mut u_is_blocked: bool = false;
    let mut d_is_blocked: bool = false;

    while j <= 7 {
        // right direction: (file - j, rank)
        if 0 <= i_file - j {
            if !r_is_blocked {
                data |= 1u64 << (i_rank * 8) + (i_file - j);
                if 0 < i_file - j {
                    if 1u64 << (i_rank * 8) + (i_file - j) & blockers.data != BitBoard::ZERO.data {
                        r_is_blocked = true;
                    }
                }
            }
        }
        // left direction: (file + j, rank)
        if i_file + j <= 7 {
            if !l_is_blocked {
                data |= 1u64 << (i_rank * 8) + (i_file + j);
                if i_file + j < 7 {
                    if 1u64 << (i_rank * 8) + (i_file + j) & blockers.data != BitBoard::ZERO.data {
                        l_is_blocked = true;
                    }
                }
            }
        }
        //   up direction: (file, rank + j)
        if i_rank + j <= 7 {
            if !u_is_blocked {
                data |= 1u64 << ((i_rank + j) * 8) + i_file;
                if i_rank + j < 7 {
                    if 1u64 << ((i_rank + j) * 8) + i_file & blockers.data != BitBoard::ZERO.data {
                        u_is_blocked = true;
                    }
                }
            }
        }
        // down direction: (file, rank - j)
        if 0 <= i_rank - j {
            if !d_is_blocked {
                data |= 1u64 << ((i_rank - j) * 8) + i_file;
                if 0 < i_rank - j {
                    if 1u64 << ((i_rank - j) * 8) + i_file & blockers.data != BitBoard::ZERO.data {
                        d_is_blocked = true;
                    }
                }
            }
        }
        j += 1
    }
    BitBoard { data }
}

// each bitboard flags relevant squares to a bishop in any given location on the chessboard
const fn bishop_mbb_mask() -> [BitBoard; 64] {
    let mut attack_array: [BitBoard; 64] = [BitBoard::ZERO; 64];

    let mut i: usize = 0;
    while i < 64usize {
        let i_rank: isize = (i as isize) / 8isize;
        let i_file: isize = (i as isize) % 8isize;
        let mut j: isize = 1;
        let mut data: u64 = 0u64;
        while j < 7 {
            //    up left direction: (+,+)
            if i_rank + j < 7 && i_file + j < 7 {
                data |= 1u64 << (i_rank + j) * 8 + (i_file + j);
            }
            //  down left direction: (-,+)
            if 0 < i_rank - j && i_file + j < 7 {
                data |= 1u64 << (i_rank - j) * 8 + (i_file + j);
            }
            //    up right direction: (+,-)
            if i_rank + j < 7 && 0 < i_file - j {
                data |= 1u64 << (i_rank + j) * 8 + (i_file - j);
            }
            //    up right direction: (-,-)
            if 0 < i_rank - j && 0 < i_file - j {
                data |= 1u64 << (i_rank - j) * 8 + (i_file - j);
            }
            j += 1
        }
        attack_array[i] = BitBoard { data };
        i += 1;
    }
    return attack_array;
}

// each bitboard flags relevant squares to a rook in any given location on the chessboard
const fn rook_mbb_mask() -> [BitBoard; 64] {
    let mut attack_array: [BitBoard; 64] = [BitBoard::ZERO; 64];

    let mut i: usize = 0;
    while i < 64usize {
        let i_rank: isize = (i as isize) / 8isize; // row
        let i_file: isize = (i as isize) % 8isize; // collumn
        let mut j: isize = 1;
        let mut data: u64 = 0u64;
        while j < 7 {
            // right direction: (file - j, rank)
            if 0 < i_file - j {
                data |= 1u64 << (i_rank * 8) + (i_file - j);
            }
            // left direction: (file + j, rank)
            if i_file + j < 7 {
                data |= 1u64 << (i_rank * 8) + (i_file + j);
            }
            //   up direction: (file, rank + j)
            if i_rank + j < 7 {
                data |= 1u64 << ((i_rank + j) * 8) + i_file;
            }
            // down direction: (file, rank - j)
            if 0 < i_rank - j {
                data |= 1u64 << ((i_rank - j) * 8) + i_file;
            }
            j += 1
        }
        attack_array[i] = BitBoard { data };
        i += 1;
    }
    return attack_array;
}

const fn compute_occ_bb(index: usize, mask_bitcount: usize, attack_mask: BitBoard) -> BitBoard {
    /* use pdep? */
    let mut attack_mask: BitBoard = attack_mask;
    let mut occupancy_bb: BitBoard = BitBoard::ZERO;
    let mut i: usize = 0;
    // while attack_mask is non-zero
    while i < mask_bitcount && attack_mask.data != 0 {
        // square_index is index of least_significant bit
        if let Some(square_index) = attack_mask.lsb_square() {
            attack_mask = attack_mask.pop_bit(square_index);
            // check that square is within range of index
            if index & (1 << i) != 0usize {
                occupancy_bb.data |= 1u64 << square_index.to_index()
            }
        }
        i += 1;
    }
    return occupancy_bb;
}

const SIZE_BISHOP: usize = 1 << 9; //size of the index for bishop magic bitboard index in bits
const SIZE_ROOK: usize = 1 << 12; //size of the index for rook magic bitboard index in bits

const fn bishop_attack_mbb() -> [[BitBoard; SIZE_BISHOP]; 64] {
    let mut i: usize = 0;
    let mut attacks: [[BitBoard; 1 << 9]; 64] = [[BitBoard::ZERO; 1 << 9]; 64];
    let bishop_mbb_mask = BISHOP_MBB_MASK;
    let bishop_occ_bitcount = BISHOP_OCC_BITCOUNT;
    while i < 64 {
        let mask = bishop_mbb_mask[i];
        let bitcount = bishop_occ_bitcount[i];
        let max_index: usize = 1 << bitcount;

        let mut j: usize = 0;
        while j < max_index {
            let blockers = compute_occ_bb(j, bitcount, mask);
            let m = magic_index(BISHOP_MAGICS[i], blockers, bitcount);

            if attacks[i][m].data == BitBoard::ZERO.data {
                attacks[i][m] = naive_bishop_attack(i, blockers);
            } else if attacks[i][m].data != naive_bishop_attack(i, blockers).data {
                panic!("bishop_attack_mbb error: invalid colision!");
            }
            j += 1;
        }
        i += 1;
    }
    return attacks;
}

const fn rook_attack_mbb() -> [[BitBoard; SIZE_ROOK]; 64] {
    let mut i: usize = 0;
    let mut attacks: [[BitBoard; 1 << 12]; 64] = [[BitBoard::ZERO; 1 << 12]; 64];
    let rook_mbb_mask = ROOK_MBB_MASK;
    let rook_occ_bitcount = ROOK_OCC_BITCOUNT;
    while i < 64 {
        let mask = rook_mbb_mask[i];
        let bitcount = rook_occ_bitcount[i];
        let max_index: usize = 1 << bitcount;

        let mut j: usize = 0;
        while j < max_index {
            let blockers = compute_occ_bb(j, bitcount, mask);
            let m = magic_index(ROOK_MAGICS[i], blockers, bitcount);

            if attacks[i][m].data == BitBoard::ZERO.data {
                attacks[i][m] = naive_rook_attack(i, blockers);
            } else if attacks[i][m].data != naive_rook_attack(i, blockers).data {
                panic!("rook_attack_mbb error: invalid colision!");
            }
            j += 1;
        }
        i += 1;
    }
    return attacks;
}

const fn rays() -> [[BitBoard; 64]; 64] {
    let mut rays: [[BitBoard; 64]; 64] = [[BitBoard::ZERO; 64]; 64];
    let mut i: usize = 0;
    while i < 64 {
        let i_square =  Square::new(i as u8);
        let mut j: usize = 0;
        while j < 64 {
            let j_square = Square::new(j as u8);
            let data = (1u64 << i) | (1u64 << j);
            let squares = BitBoard { data };
            if (ROWS[i] == ROWS[j]) || (COLS[i] == COLS[j]) {
                let data: u64 = const_get_rook_attack(i_square, squares).data & const_get_rook_attack(j_square, squares).data;
                rays[i][j].data = data;
            } else if (DDIAG[i] == DDIAG[j]) || (ADIAG[i] == ADIAG[j]) {
                let data = const_get_bishop_attack(i_square, squares).data & const_get_bishop_attack(j_square, squares).data;
                rays[i][j].data = data;
            }
            j += 1;
        }
        i += 1;
    }
    rays
}

/* ==== magic ==== */
include!("data/bishop.rs");
include!("data/rook.rs");

#[rustfmt::skip]
const BISHOP_MAGICS: [u64; 64] = [
    0x0140C80810488022, 0x0020021C01142000, 0x00308C2080200102, 0x0004040880000A09,
    0x0824042080000001, 0x00C1010840807080, 0x810C010403200000, 0x49CE404044202081,
    0x4405048410020200, 0x0000042104440080, 0x0801C12112008003, 0x0100080A43014001,
    0x0000020210010000, 0x0110020110080990, 0x0800004804042000, 0x0000002434020800,
    0x00C108E014890204, 0x0004040210440100, 0x4808001000801012, 0x0008004620801080,
    0x0481000290400A01, 0x0001000180A00921, 0x1204010900A80492, 0x0A88400024041C00,
    0x1002100088501014, 0x005045040818008C, 0x0002080081004408, 0x0208280005820002,
    0x0509010040104008, 0x8010004000241000, 0x8908108440540400, 0x0142060800404240,
    0x0231101010402410, 0x0002011140241020, 0x100A002A00101180, 0x2001010800110041,
    0x8118022401224100, 0x4420092A40020800, 0x22D000C880031400, 0x000102108002A420,
    0x4008044404102020, 0x8000842402002000, 0x000200242400080E, 0x0030004202208802,
    0x0000011214000601, 0x10C0008099011081, 0x10080104608A0C00, 0x0002285D00202700,
    0x009A182414050000, 0x020100A210223022, 0x0000002C02080102, 0x0000000020884010,
    0x0280029002022040, 0x8250102490342010, 0x0040020464048080, 0x4120040102042200,
    0x280A010401018800, 0x8010008084104200, 0x009009002484501A, 0x1A08830080420208,
    0x2000064022604100, 0x0012400420044101, 0x0040042818810C00, 0x1024211464008200,
];

#[rustfmt::skip]
const ROOK_MAGICS: [u64; 64] =  [
    0x818001C000802018, 0xA240100020004000, 0x0100081041002000, 0x1080048010000800,
    0x8600020020040810, 0x0580018002004400, 0x1080020000800100, 0x020000204A088401,
    0x4000800080204000, 0x0040804000200080, 0x0000801000200080, 0x0222000C10204200,
    0x0042000600081020, 0x00A2001004080200, 0x1000800100800200, 0x0082000092010044,
    0x0800848000400420, 0x0030044040002001, 0x8000110041002004, 0x00004200200A0010,
    0x0810808004000800, 0xC028808002000400, 0x0280040090080201, 0x0804020000508104,
    0x0080400480088024, 0x0400200440100241, 0x0401001100200040, 0x0000100080800800,
    0x0008010100041008, 0x8000020080800400, 0x1000012400024830, 0x0004008200210054,
    0x08084A0082002100, 0x4080201000404000, 0xC000102001004100, 0x0004082101001002,
    0x0009820800800400, 0x900C800400800200, 0x9040080204008150, 0x80B0140446000493,
    0x6040244000828000, 0x0210002000504000, 0x0015002002110040, 0x0041001000210008,
    0x0001004800050010, 0x0002000804010100, 0x5008081002040081, 0x00220040A1020004,
    0x0101400120800180, 0x2040002000C08180, 0x1120001000480040, 0x18001020400A0200,
    0x0004050010080100, 0x1023020080040080, 0x0001080102100400, 0x0001000282004300,
    0x0190401100800021, 0x0805854001021021, 0x600010400C200101, 0x0010210009100005,
    0x1001001002080005, 0x9801000C00080A29, 0x2006080A45029014, 0x0008804581022C02,
];

const BISHOP_OCC_BITCOUNT: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, //
    5, 5, 5, 5, 5, 5, 5, 5, //
    5, 5, 7, 7, 7, 7, 5, 5, //
    5, 5, 7, 9, 9, 7, 5, 5, //
    5, 5, 7, 9, 9, 7, 5, 5, //
    5, 5, 7, 7, 7, 7, 5, 5, //
    5, 5, 5, 5, 5, 5, 5, 5, //
    6, 5, 5, 5, 5, 5, 5, 6, //
];

const ROOK_OCC_BITCOUNT: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    12, 11, 11, 11, 11, 11, 11, 12, //
];

const fn magic_index(magic_num: u64, blockers: BitBoard, bitcount: usize) -> usize {
    ((blockers.data.wrapping_mul(magic_num)) >> (64 - bitcount)) as usize
}

const fn const_get_bishop_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let mask = BISHOP_MBB_MASK[square.to_index()];
    let data = blockers.data & mask.data;
    let blockers = BitBoard { data };
    let m = magic_index(BISHOP_MAGICS[square.to_index()], blockers, BISHOP_OCC_BITCOUNT[square.to_index()]);
    return BISHOP_ATTACKS_MBB[square.to_index()][m];
}

const fn const_get_rook_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let mask = ROOK_MBB_MASK[square.to_index()];
    let data = blockers.data & mask.data;
    let blockers = BitBoard { data };
    let m = magic_index(ROOK_MAGICS[square.to_index()], blockers, ROOK_OCC_BITCOUNT[square.to_index()]);
    return ROOK_ATTACKS_MBB[square.to_index()][m];
}

/* ==== constants and supporting functions ==== */
const ASCII_SYM: [char; 12] = ['K', 'Q', 'N', 'B', 'R', 'P', 'k', 'q', 'n', 'b', 'r', 'p'];
const UNICODE_SYM: [char; 12] = ['♚', '♛', '♞', '♝', '♜', '♟', '♔', '♕', '♘', '♗', '♖', '♙'];

pub(crate) const W_KING_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000110);
pub(crate) const W_QUEEN_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01110000);
pub(crate) const B_KING_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000);
pub(crate) const B_QUEEN_SIDE_CASTLE_MASK: BitBoard =
    BitBoard::new(0b01110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

pub fn get_pawn_attack(square: Square, side: Side) -> BitBoard {
    match side {
        Side::White => W_PAWN_ATTACKS[square.to_index()],
        Side::Black => B_PAWN_ATTACKS[square.to_index()],
    }
}

pub const fn get_w_pawn_attack(square: Square) -> BitBoard {
    W_PAWN_ATTACKS[square.to_index()]
}

pub const fn get_b_pawn_attack(square: Square) -> BitBoard {
    B_PAWN_ATTACKS[square.to_index()]
}

pub const fn get_knight_attack(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square.to_index()]
}

pub const fn get_king_attack(square: Square) -> BitBoard {
    KING_ATTACKS[square.to_index()]
}

pub const fn get_bishop_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let data = blockers.data & BISHOP_MBB_MASK[square.to_index()].data;
    let m = magic_index(
        BISHOP_MAGICS[square.to_index()],
        BitBoard { data },
        BISHOP_OCC_BITCOUNT[square.to_index()],
    );
    return BISHOP_ATTACKS_MBB[square.to_index()][m];
}

pub const fn get_rook_attack(square: Square, blockers: BitBoard) -> BitBoard {
    let data = blockers.data & ROOK_MBB_MASK[square.to_index()].data;
    let m = magic_index(
        ROOK_MAGICS[square.to_index()],
        BitBoard { data },
        ROOK_OCC_BITCOUNT[square.to_index()],
    );
    return ROOK_ATTACKS_MBB[square.to_index()][m];
}

pub const fn get_queen_attack(square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard {
        data: get_bishop_attack(square, blockers).data | get_rook_attack(square, blockers).data,
    }
}
pub const fn is_same_diag(source: Square, target: Square) -> bool {
   (DDIAG[source.to_index()] == DDIAG[target.to_index()]) || (ADIAG[source.to_index()] == ADIAG[target.to_index()])
}
pub const fn is_same_adiag(source: Square, target: Square) -> bool {
    ADIAG[source.to_index()] == ADIAG[target.to_index()]
}
pub const fn is_same_ddiag(source: Square, target: Square) -> bool {
    DDIAG[source.to_index()] == DDIAG[target.to_index()]
}
pub const fn is_same_col(source: Square, target: Square) -> bool {
   COLS[source.to_index()] == COLS[target.to_index()]
}
pub const fn is_same_row(source: Square, target: Square) -> bool {
   ROWS[source.to_index()] == ROWS[target.to_index()]
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