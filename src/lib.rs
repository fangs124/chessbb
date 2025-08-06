mod bitboard;
pub mod chessmove;
mod movegen;
mod square;
mod zorbist;
mod perft;
use std::fmt::Display;

use crate::{bitboard::*, square::Square, zorbist::{ZorbistHash, ZorbistTable}};
use crate::perft::*;
/* chessboard specific bitboard functions and definitions*/

/* ChessBoard encodes the board-state of the game */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    pub piece_bbs: [BitBoard; 12],
    pub mailbox: [Option<ChessPiece>; 64],
    pub castle_bools: [bool; 4],
    pub enpassant_bb: BitBoard, //pieces triggering en-passant rule
    //attacked_bb: BitBoard, //a mask showing all attacked squares (do I need this?)
    pub check_bb: BitBoard, //pieces triggering check condition
    pub check_mask: BitBoard, //all the squares attacked by checking pieces;
    pub pinned_bb: BitBoard, //pieces that are pinned
    pub pinner_bb: BitBoard, //pieces doing the pin
    pub side_to_move: Side,
    pub full_move_counter: u16,
    pub fifty_move_rule_counter: u16,
    pub zorbist_table: ZorbistTable,
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard::start_pos()
    }
}

//TODO rewrite this
impl Display for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        // get empty_squares
        let mut empty_squares = BitBoard::ZERO;
        for piece_bb in self.piece_bbs {
            empty_squares = piece_bb | empty_squares;
        }
        empty_squares = !empty_squares;

        // append characters according to piece
        for i in 1..=64usize {
            if !empty_squares.nth_is_zero(Square::new((64 - i) as u8)) {
                s.push('.');
            } else {
                let mut j = 0usize;
                while j < self.piece_bbs.len() {
                    let piece_bb: BitBoard = self.piece_bbs[j];
                    if !piece_bb.nth_is_zero(Square::new((64 - i) as u8 )) {
                        s.push(UNICODE_SYM[j]);
                    }
                    j += 1;
                }
            }

            if i % 8 == 0 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

impl ChessBoard {
    pub const fn start_pos() -> Self {
        Self {
            piece_bbs: ChessBoard::INITIAL_CHESS_POS,
            mailbox: ChessBoard::INITIAL_MAILBOX,
            castle_bools: [true; 4],
            enpassant_bb: BitBoard::ZERO,
            check_bb: BitBoard::ZERO,
            check_mask: BitBoard::ZERO,
            pinned_bb: BitBoard::ZERO,
            pinner_bb: BitBoard::ZERO,
            side_to_move: Side::White,
            full_move_counter: 0,
            fifty_move_rule_counter: 0,
            zorbist_table: ZorbistTable::initial_table(),
        }
    }
    
    //TODO rewrite this
    pub fn from_fen(input: &str) -> ChessBoard {
        //ChessBoard {
        //    check_mask: self.check_mask,
        //    check_bb: self.check_bb,
        //    pinned_bb: self.pinned_bb,
        //    pinner_bb: self.pinned_bb,
        //}
        assert!(input.is_ascii());
        let input_vec: Vec<&str> = input.split_ascii_whitespace().collect();
        assert!(input_vec.len() == 6);
        let mut chessboard = ChessBoard::start_pos();
        chessboard.piece_bbs = [BitBoard::ZERO; 12];
        chessboard.mailbox = [None; 64];
        chessboard.castle_bools = [false, false, false, false];


        // parse piece placement data
        let mut square: usize = 0;
        for c in input_vec[0].chars().rev() {
            //println!("c:{}", c);
            match c {
                'K' |'Q' |'N' |'B' |'R' |'P' |'k' |'q' |'n' |'b' |'r' |'p' => {
                    chessboard.piece_bbs[sym_index(c)] = chessboard.piece_bbs[sym_index(c)].set_bit(Square::new(square as u8));
                    chessboard.mailbox[square] = match c {
                        'K' => Some((Side::White, PieceType::King)),
                        'Q' => Some((Side::White, PieceType::Queen)),
                        'N' => Some((Side::White, PieceType::Knight)),
                        'B' => Some((Side::White, PieceType::Bishop)),
                        'R' => Some((Side::White, PieceType::Rook)),
                        'P' => Some((Side::White, PieceType::Pawn)),
                        'k' => Some((Side::Black, PieceType::King)),
                        'q' => Some((Side::Black, PieceType::Queen)),
                        'n' => Some((Side::Black, PieceType::Knight)),
                        'b' => Some((Side::Black, PieceType::Bishop)),
                        'r' => Some((Side::Black, PieceType::Rook)),
                        'p' => Some((Side::Black, PieceType::Pawn)),
                        _ => unreachable!()
                    }
                }

                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    square += (c.to_digit(10).unwrap() as usize) - 1;
                }

                '/' => continue,
                _ => panic!("from_fen error: invalid char in piece placement portion. char: {}", c)
            }
            square += 1;
        }

        // parse active colour
        chessboard.side_to_move = match input_vec[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => panic!("from_fen error: invalid active side!"),
        };

        let mut i: usize = 0;
        // parse castling information
        while i < input_vec[2].len() {
            let s = match input_vec[2].chars().nth(i) {
                Some(x) => x,
                None => unreachable!(),
            };

            match s {
                '-' => (),
                'K' => chessboard.castle_bools[0] = true,
                'Q' => chessboard.castle_bools[1] = true,
                'k' => chessboard.castle_bools[2] = true,
                'q' => chessboard.castle_bools[3] = true,
                _ => panic!("from_fen error: invalid castling information!"),
            }
            i += 1;
        }

        // parse en passant information
        if input_vec[3] != "-" {
            chessboard.enpassant_bb.set_bit(Square::new(ChessBoard::square_index(input_vec[3]) as u8));
        }

        //parse fifty-move-rule counter
        chessboard.fifty_move_rule_counter = input_vec[4].parse::<u16>().unwrap();
        
        //parse fullmove number
        chessboard.full_move_counter = input_vec[5].parse::<u16>().unwrap();

        //calculate king_is_in_check information.
        assert!(chessboard.piece_bbs[0].count_ones() == 1);
        assert!(chessboard.piece_bbs[6].count_ones() == 1);
        let side = chessboard.side_to_move;
        if chessboard.is_king_in_check(side) {
            match side {
                Side::White => {
                    let blockers = chessboard.blockers();
                    let king_pos= chessboard.piece_bbs[0].lsb_square().unwrap();
                    let mut check_bitboard = BitBoard::ZERO;
                    //q
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[07] & get_queen_attack(king_pos, blockers));
                    //n
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[08] & get_knight_attack(king_pos));
                    //b
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[09] & get_bishop_attack(king_pos, blockers));
                    //r
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[10] & get_rook_attack(king_pos, blockers));
                    //p
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[11] & get_w_pawn_attack(king_pos));
                    chessboard.check_bb = check_bitboard;
                }

                Side::Black => {
                    let blockers = chessboard.blockers();
                    let king_pos = chessboard.piece_bbs[6].lsb_square().unwrap();
                    let mut check_bitboard = BitBoard::ZERO;
                    //Q
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[01] & get_queen_attack(king_pos, blockers));
                    //N
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[02] & get_knight_attack(king_pos));
                    //B
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[03] & get_bishop_attack(king_pos, blockers));
                    //R
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[04] & get_rook_attack(king_pos, blockers));
                    //P
                    check_bitboard = check_bitboard | (chessboard.piece_bbs[05] & get_b_pawn_attack(king_pos));
                    chessboard.check_bb = check_bitboard;
                }
            }
        }
        chessboard.compute_pin_data();
        chessboard.compute_check_bb();
        chessboard.compute_check_mask();
        chessboard.zorbist_table = ZorbistTable::new(ZorbistHash::compute_hash(&chessboard));
        return chessboard;
    }
    
    //TODO rewrite this
    pub const fn square_index(square_name: &str) -> usize {
    let mut i: usize = 0;
    while i < 64 {
        let mut j: usize = 0;
        let mut is_match = SQUARE_SYM[i].as_bytes().len() == square_name.as_bytes().len();
        while j < SQUARE_SYM[i].as_bytes().len() {
            if SQUARE_SYM[i].as_bytes()[j] != square_name.as_bytes()[j] {
                is_match = false;
            }
            j += 1;
        }
        if is_match {
            return i;
        }
        i += 1
    }
    panic!("square_index error: invalid square!");
}


    pub const fn duplicate(&self) -> ChessBoard {
        ChessBoard {
            piece_bbs: self.piece_bbs,
            mailbox: self.mailbox,
            castle_bools: self.castle_bools,
            enpassant_bb: self.enpassant_bb, //enpassant_bb are pawn-attackable square via en-passant
            check_mask: self.check_mask,
            check_bb: self.check_bb,
            pinned_bb: self.pinned_bb,
            pinner_bb: self.pinned_bb,
            side_to_move: self.side_to_move,
            full_move_counter: self.full_move_counter,
            fifty_move_rule_counter: self.fifty_move_rule_counter,
            zorbist_table: self.zorbist_table,
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

    pub const fn blockers_no_white_king(&self) -> BitBoard {
        let mut i = 0;
        let mut bitboard: BitBoard = BitBoard::ZERO;
        while i < 12 {
            bitboard = bitboard.bit_or(&self.piece_bbs[i]);
            i += 1;
        }
        return bitboard.bit_and(&self.piece_bbs[0].bit_not());
    }

    pub const fn blockers_no_black_king(&self) -> BitBoard {
        let mut i = 0;
        let mut bitboard: BitBoard = BitBoard::ZERO;
        while i < 12 {
            bitboard = bitboard.bit_or(&self.piece_bbs[i]);
            i += 1;
        }
        return bitboard.bit_and(&self.piece_bbs[6].bit_not());
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

    pub(crate) const fn current_hash(&self) -> ZorbistHash {
        self.zorbist_table.last_hash()
    }

    pub(crate) const fn count_hash(&self, hash: ZorbistHash) -> usize {
        self.zorbist_table.count_hash(hash)
    }

    //TODO maybe is_square_attacked should have parameterized blockers?
    pub const fn is_square_attacked(&self, square: Square, attacker_side: Side, blockers: BitBoard) -> bool {
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
                return (get_w_pawn_attack(square).bit_and(&self.piece_bbs[11])).is_not_zero()
                    || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[10])).is_not_zero()
                    || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[9])).is_not_zero()
                    || (get_knight_attack(square).bit_and(&self.piece_bbs[8])).is_not_zero()
                    || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[7])).is_not_zero()
                    || (get_king_attack(square).bit_and(&self.piece_bbs[6])).is_not_zero();
            }
        }
    }

    //TODO do something about htis.. currently used to see if a square behind a king is attacked
    //pub(crate) const fn is_square_attacked_removed_piece(&self, square: Square, side: Side, removed_square: Square) -> bool {
    //    let blockers = self.blockers().pop_bit(removed_square);
    //    match side {//FIXME maybe can do bit_and, so only one is_not_zero() call?
    //        Side::White => {
    //            return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[11])).is_not_zero()
    //                || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[10])).is_not_zero()
    //                || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[9])).is_not_zero()
    //                || (get_knight_attack(square).bit_and(&self.piece_bbs[8])).is_not_zero()
    //                || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[7])).is_not_zero()
    //                || (get_king_attack(square).bit_and(&self.piece_bbs[6])).is_not_zero();
    //        }
    //        Side::Black => {
    //            return (get_w_pawn_attack(square).bit_and(&self.piece_bbs[5])).is_not_zero()
    //                || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[4])).is_not_zero()
    //                || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[3])).is_not_zero()
    //                || (get_knight_attack(square).bit_and(&self.piece_bbs[2])).is_not_zero()
    //                || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[1])).is_not_zero()
    //                || (get_king_attack(square).bit_and(&self.piece_bbs[0])).is_not_zero();
    //        }
    //    }
    //}

    pub const fn is_king_in_check(&self, king_side: Side) -> bool {
        let i = match king_side {
            Side::White => 0,
            Side::Black => 6,
        };

        let square = match self.piece_bbs[i].lsb_square() {
            Some(x) => x,
            None => panic!("king_is_in_check error: king not found!"),
        };

        self.is_square_attacked(square, self.side_to_move.update(), self.blockers())
    }

    // castling kingside
    pub fn is_able_kingside_castle(&self, side: Side) -> bool {
        let king_square: Square;
        let rook_square: Square;
        let castling_mask: BitBoard;
        let castling_index: usize;
        let blockers = self.blockers();

        match side {
            Side::White => {
                king_square = match self.piece_bbs[cpt_index!(K)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: white king not found"),
                };
                rook_square = Square::new(0);
                castling_mask= W_KING_SIDE_CASTLE_MASK;
                castling_index = 0;
                //queen_castling_mask = W_QUEEN_SIDE_CASTLE_MASK;
                //queen_castling_index = 1;
                //friends = w_blockers;
                //enemies = b_blockers;
            }
            Side::Black => {
                king_square = match self.piece_bbs[cpt_index!(k)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: black king not found"),
            };
            rook_square = Square::new(56);
            castling_mask= B_KING_SIDE_CASTLE_MASK;
            castling_index = 2;
            //queen_castling_mask = B_QUEEN_SIDE_CASTLE_MASK;
            //queen_castling_index = 3;
            //friends = b_blockers;
            //enemies = w_blockers;
            }
        }

        // check if friendly side can still castle, and if there are blockers in relevant squares
        if (self.castle_bools[castling_index] == false) ||  (blockers.bit_and(&castling_mask).is_zero() == false) {
            return false;
        }

        // check if squares between rook and king are empty
        if RAYS[king_square.to_index()][rook_square.to_index()].bit_and(&blockers).is_not_zero() {
            return false;
        }

        let mut squares = castling_mask;
        while squares.is_not_zero() {
            let square = squares.lsb_square().unwrap();
            if self.is_square_attacked(square, side.update(), self.blockers()) {
                return false;
            }
            squares = squares.pop_bit(square);
        }
        return true;
    }
    
    // castling queenside
    pub fn is_able_queenside_castle(&self, side: Side) -> bool {
        let king_square: Square;
        let rook_square: Square;
        let castling_mask: BitBoard;
        let castling_index: usize;
        let blockers = self.blockers();

        match side {
            Side::White => {
                king_square = match self.piece_bbs[cpt_index!(K)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: white king not found"),
                };
                rook_square = Square::new(7);
                //king_castling_mask= W_KING_SIDE_CASTLE_MASK;
                //king_castling_index = 0;
                castling_mask = W_QUEEN_SIDE_CASTLE_MASK;
                castling_index = 1;
                //friends = w_blockers;
                //enemies = b_blockers;
            }
            Side::Black => {
                king_square = match self.piece_bbs[cpt_index!(k)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: black king not found"),
                };
                rook_square = Square::new(63);
                //king_castling_mask= B_KING_SIDE_CASTLE_MASK;
                //king_castling_index = 2;
                castling_mask = B_QUEEN_SIDE_CASTLE_MASK;
                castling_index = 3;
                //friends = b_blockers;
                //enemies = w_blockers;
            }
        }

        // check if friendly side can still castle, and if there are blockers in relevant squares
        if (self.castle_bools[castling_index] == false) ||  (blockers.bit_and(&castling_mask).is_zero() == false) {
            return false;
        }

        // check if squares between rook and king are empty
        if RAYS[king_square.to_index()][rook_square.to_index()].bit_and(&blockers).is_not_zero() {
            return false;
        }

        let mut squares = castling_mask;
        while squares.is_not_zero() {
            let square = squares.lsb_square().unwrap();
            if self.is_square_attacked(square, side.update(), self.blockers()) {
                return false;
            }
            squares = squares.pop_bit(square);
        }
        return true;
    }
    
    pub(crate) fn is_piece_pinned(&self, square: Square) -> bool {
        self.pinned_bb.nth_is_not_zero(square)
    }

    //calculates all squares attacked by pinning pieces, that passes through a square
    pub(crate) const fn pin_mask(&self, square:Square) -> BitBoard {
        let mut pin_mask: BitBoard = BitBoard::ZERO;
        let mut pinners = self.pinner_bb;
        while pinners.is_not_zero() {
            let pinner = pinners.lsb_square().unwrap();
            // check if square is between king and potential_pinner
            if RAYS[self.king_square().to_index()][pinner.to_index()].nth_is_not_zero(square) {
                pin_mask = pin_mask.bit_or(&RAYS[self.king_square().to_index()][pinner.to_index()].bit_or(&BitBoard::nth(pinner)))
            }
            pinners = pinners.pop_bit(pinner);
        }
        return pin_mask;
    } 
    
    // calculates number of enemy checking piece
    pub(crate) fn count_checking_pieces(&self) -> u32 {
        self.check_bb.count_ones()
    }

    pub(crate) const fn king_square(&self) -> Square {
        match self.side_to_move {
            Side::White => self.piece_bbs[cpt_index!(K)].lsb_square().expect("king_square: king must be present"),
            Side::Black => self.piece_bbs[cpt_index!(k)].lsb_square().expect("king_square: king must be present"),
        }
    }

    pub(crate) const fn piece_bb(&self, piece_type: ChessPiece) -> BitBoard {
        match piece_type {
            (Side::White, PieceType::King  ) => self.piece_bbs[00],
            (Side::White, PieceType::Queen ) => self.piece_bbs[01],
            (Side::White, PieceType::Knight) => self.piece_bbs[02],
            (Side::White, PieceType::Bishop) => self.piece_bbs[03],
            (Side::White, PieceType::Rook )  => self.piece_bbs[04],
            (Side::White, PieceType::Pawn  ) => self.piece_bbs[05],
            (Side::Black, PieceType::King  ) => self.piece_bbs[06],
            (Side::Black, PieceType::Queen ) => self.piece_bbs[07],
            (Side::Black, PieceType::Knight) => self.piece_bbs[08],
            (Side::Black, PieceType::Bishop) => self.piece_bbs[09],
            (Side::Black, PieceType::Rook  ) => self.piece_bbs[10],
            (Side::Black, PieceType::Pawn  ) => self.piece_bbs[11],
        }
    }

    #[rustfmt::skip]
    const INITIAL_CHESS_POS: [BitBoard; 12] = [
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000), // ♔
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000), // ♕
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010), // ♘
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100), // ♗
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001), // ♖
        BitBoard::new(0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000), // ♙
        BitBoard::new(0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000), // ♚
        BitBoard::new(0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000), // ♛
        BitBoard::new(0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000), // ♞
        BitBoard::new(0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000), // ♝
        BitBoard::new(0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000), // ♜
        BitBoard::new(0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000), // ♟
    ];
    
    #[rustfmt::skip]
    const INITIAL_MAILBOX: [Option<ChessPiece>; 64] = [
        opt_cpt!(R), opt_cpt!(N), opt_cpt!(B), opt_cpt!(K), opt_cpt!(Q), opt_cpt!(B), opt_cpt!(N), opt_cpt!(R),
        opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P), opt_cpt!(P),
        opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
        opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
        opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
        opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_), opt_cpt!(_),
        opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p), opt_cpt!(p),
        opt_cpt!(r), opt_cpt!(n), opt_cpt!(b), opt_cpt!(k), opt_cpt!(q), opt_cpt!(b), opt_cpt!(n), opt_cpt!(r),
    ];

    const INITIAL_ATTACKED_BB: BitBoard = BitBoard::new(0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000);
}

