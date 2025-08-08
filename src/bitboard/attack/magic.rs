use crate::bitboard::BitBoard;
use crate::bitboard::attack::{naive_bishop_attack, naive_rook_attack};
use crate::square::Square;

pub(super) const SIZE_BISHOP: usize = 1 << 9; //size of the index for bishop magic bitboard index in bits
pub(super) const SIZE_ROOK: usize = 1 << 12; //size of the index for rook magic bitboard index in bits

pub(super) const BISHOP_MBB_MASK: [BitBoard; 64] = bishop_mbb_mask();
pub(super) const ROOK_MBB_MASK: [BitBoard; 64] = rook_mbb_mask();
pub(super) const BISHOP_ATTACKS_MBB: [[BitBoard; SIZE_BISHOP]; 64] = BISHOP;
pub(super) const ROOK_ATTACKS_MBB: [[BitBoard; SIZE_ROOK]; 64] = ROOK;

include!("data/bishop.rs");
include!("data/rook.rs");

pub(super) const BISHOP_OCC_BITCOUNT: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, //
    5, 5, 5, 5, 5, 5, 5, 5, //
    5, 5, 7, 7, 7, 7, 5, 5, //
    5, 5, 7, 9, 9, 7, 5, 5, //
    5, 5, 7, 9, 9, 7, 5, 5, //
    5, 5, 7, 7, 7, 7, 5, 5, //
    5, 5, 5, 5, 5, 5, 5, 5, //
    6, 5, 5, 5, 5, 5, 5, 6, //
];

pub(super) const ROOK_OCC_BITCOUNT: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    11, 10, 10, 10, 10, 10, 10, 11, //
    12, 11, 11, 11, 11, 11, 11, 12, //
];

#[rustfmt::skip]
pub(super) const BISHOP_MAGICS: [u64; 64] = [
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
pub(super) const ROOK_MAGICS: [u64; 64] =  [
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

const fn compute_occ_bb(index: usize, mask_bitcount: usize, attack_mask: BitBoard) -> BitBoard {
    /* use pdep? */
    let mut attack_mask: BitBoard = attack_mask;
    let mut occupancy_bb: BitBoard = BitBoard::ZERO;
    let mut i: usize = 0;
    // while attack_mask is non-zero
    while i < mask_bitcount && attack_mask.data != 0 {
        // square_index is index of least_significant bit
        if let Some(square_index) = attack_mask.lsb_square() {
            attack_mask.pop_bit(square_index);
            // check that square is within range of index
            if index & (1 << i) != 0usize {
                occupancy_bb.data |= 1u64 << square_index.to_index()
            }
        }
        i += 1;
    }
    return occupancy_bb;
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

#[inline(always)]
pub(super) const fn magic_index(magic_num: u64, blockers: BitBoard, bitcount: usize) -> usize {
    ((blockers.data.wrapping_mul(magic_num)) >> (64 - bitcount)) as usize
}

//TODO generate magic numbers instead of using a fixed precalculated values and tables
