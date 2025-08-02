mod bitboard;
mod chessmove;
mod movegen;
mod square;
use crate::{bitboard::*, square::Square};

/* chessboard specific bitboard functions and definitions*/

/* ChessBoard encodes the board-state of the game */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    piece_bbs: [BitBoard; 12],
    mailbox: [Option<ChessPiece>; 64],
    castle_bools: [bool; 4],
    enpassant_bb: BitBoard,
    check_bb: BitBoard, //piece locations causing the check
    side_to_move: Side,
    half_move_clock: u16,
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard::start_pos()
    }
}

impl ChessBoard {
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

    pub const fn start_pos() -> Self {
        Self {
            piece_bbs: ChessBoard::INITIAL_CHESS_POS,
            mailbox: ChessBoard::INITIAL_MAILBOX,
            castle_bools: [true; 4],
            enpassant_bb: BitBoard::ZERO,
            side_to_move: Side::White,
            check_bb: BitBoard::ZERO,
            half_move_clock: 0,
        }
    }
    pub const fn duplicate(&self) -> ChessBoard {
        ChessBoard {
            piece_bbs: self.piece_bbs,
            mailbox: self.mailbox,
            castle_bools: self.castle_bools,
            enpassant_bb: self.enpassant_bb,
            side_to_move: self.side_to_move,
            half_move_clock: self.half_move_clock,
            check_bb: self.check_bb,
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
    //TODO maybe is_square_attacked should have parameterized blockers?
    pub const fn is_square_attacked(&self, square: Square, attacker_side: Side) -> bool {
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

    //pub(crate) const fn is_square_behind_king_attacked(&self, square: Square, side: Side) -> bool {
    //    let blockers = self.blockers().pop_bit(self.king_square());
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
    //            return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[5])).is_not_zero()
    //                || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[4])).is_not_zero()
    //                || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[3])).is_not_zero()
    //                || (get_knight_attack(square).bit_and(&self.piece_bbs[2])).is_not_zero()
    //                || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[1])).is_not_zero()
    //                || (get_king_attack(square).bit_and(&self.piece_bbs[0])).is_not_zero();
    //        }
    //    }
    //}

    pub(crate) const fn is_square_attacked_removed_piece(&self, square: Square, side: Side, removed_square: Square) -> bool {
        let blockers = self.blockers().pop_bit(removed_square);
        match side {//FIXME maybe can do bit_and, so only one is_not_zero() call?
            Side::White => {
                return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[11])).is_not_zero()
                    || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[10])).is_not_zero()
                    || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[9])).is_not_zero()
                    || (get_knight_attack(square).bit_and(&self.piece_bbs[8])).is_not_zero()
                    || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[7])).is_not_zero()
                    || (get_king_attack(square).bit_and(&self.piece_bbs[6])).is_not_zero();
            }
            Side::Black => {
                return (get_b_pawn_attack(square).bit_and(&self.piece_bbs[5])).is_not_zero()
                    || (get_rook_attack(square, blockers).bit_and(&self.piece_bbs[4])).is_not_zero()
                    || (get_bishop_attack(square, blockers).bit_and(&self.piece_bbs[3])).is_not_zero()
                    || (get_knight_attack(square).bit_and(&self.piece_bbs[2])).is_not_zero()
                    || (get_queen_attack(square, blockers).bit_and(&self.piece_bbs[1])).is_not_zero()
                    || (get_king_attack(square).bit_and(&self.piece_bbs[0])).is_not_zero();
            }
        }
    }

    pub const fn is_king_in_check(&self, king_side: Side) -> bool {
        let i = match king_side {
            Side::White => 0,
            Side::Black => 6,
        };

        let square = match self.piece_bbs[i].lsb_square() {
            Some(x) => x,
            None => panic!("king_is_in_check error: king not found!"),
        };

        self.is_square_attacked(square, self.side_to_move.update())
    }

    // castling kingside
    pub(crate) fn is_able_kingside_castle(&self, side: Side) -> bool {
        let king_square: Square;
        let castling_mask: BitBoard;
        let castling_index: usize;
        let blockers = self.blockers();

        match side {
            Side::White => {
                king_square = match self.piece_bbs[cpt_index!(K)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: white king not found"),
                };
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
        let mut squares = castling_mask;
        while squares.is_not_zero() {
            let square = squares.lsb_square().unwrap();
            if self.is_square_attacked(square, side.update()) {
                return false;
            }
            squares = squares.pop_bit(square);
        }
        return true;
    }
    // castling queenside
    pub(crate) fn is_able_queenside_castle(&self, side: Side) -> bool {
        let king_square: Square;
        let castling_mask: BitBoard;
        let castling_index: usize;
        let blockers = self.blockers();

        match side {
            Side::White => {
                king_square = match self.piece_bbs[cpt_index!(K)].lsb_square() {
                    Some(square) => square,
                    None => panic!("generate moves: white king not found"),
                };
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
        let mut squares = castling_mask;
        while squares.is_not_zero() {
            let square = squares.lsb_square().unwrap();
            if self.is_square_attacked(square, side.update()) {
                return false;
            }
            squares = squares.pop_bit(square);
        }
        return true;
    }

    pub(crate) const fn is_piece_pinned(&self, square: Square) -> bool {
        //FIXME ?? duplicating self, mutating it, is it necessary?
        let mut chessboard: ChessBoard = self.duplicate();
        let piece = self.mailbox[square.to_index()].expect("is_piece_pinned error: mailbox square is empty!");
        let mut piece_bb = self.piece_bbs[cp_index(piece)].bit_and(&BitBoard::nth(square).bit_not());
        //let mut mailbox = self.mailbox;
        chessboard.mailbox[cp_index(piece)] = None;
        let side = self.side_to_move;
        assert!(matches!(piece, cpt!(K) | cpt!(k)) == false, "is_piece_pinned error: checking if king is pinned");
        if self.is_king_in_check(side) == false {
            // if king is not in check, test if removing piece causes king to be in check
            return chessboard.is_king_in_check(side); //FIXME
        } else {
            //check if king are present for both sides
            assert!(self.piece_bbs[0].count_ones() == 1 && self.piece_bbs[6].count_ones() == 1);
            let king_square: Square;
            let diagonals: BitBoard;
            let laterals: BitBoard;
            let enemies: BitBoard;
            match side {
                Side::White => {
                    king_square = self.piece_bbs[cpt_index!(K)].lsb_square().unwrap();
                    diagonals = self.piece_bbs[cpt_index!(q)].bit_or(&self.piece_bbs[cpt_index!(b)]);
                    laterals = self.piece_bbs[cpt_index!(q)].bit_or(&self.piece_bbs[cpt_index!(r)]);
                    enemies = self.black_blockers();
                }
                Side::Black => {
                    king_square = self.piece_bbs[cpt_index!(k)].lsb_square().unwrap();
                    diagonals = self.piece_bbs[cpt_index!(Q)].bit_or(&self.piece_bbs[cpt_index!(B)]);
                    laterals = self.piece_bbs[cpt_index!(Q)].bit_or(&self.piece_bbs[cpt_index!(R)]);
                    enemies = self.white_blockers();
                }
            }
            // blockers without friendly king
            let removed_blockers = self.blockers().bit_and(&BitBoard::nth(square).bit_not());
            // position of diagonal attackers
            let diag_attackers = get_bishop_attack(square, removed_blockers).bit_and(&diagonals);
            // position of lateral attackers
            let latr_attackers = get_rook_attack(square, removed_blockers).bit_and(&laterals);

            // if king is in check, see if square is in ray of a checking piece
            // note: why are we testing for all rays from the king here?
            let mut potential_pinners = enemies.bit_and(&diag_attackers.bit_or(&latr_attackers));
            while potential_pinners.is_not_zero() {
                let potential_pinner = potential_pinners.lsb_square().unwrap();
                // check if piece is between king and potential_pinner
                if RAYS[king_square.to_index()][potential_pinner.to_index()].nth_is_not_zero(square) {
                    return true;
                }
                potential_pinners = potential_pinners.pop_bit(potential_pinner);
            }
        }
        return false;
    }

    pub(crate) const fn calculate_pin_data(&self, square: Square) -> (BitBoard,BitBoard) {
        //FIXME ?? duplicating self, mutating it, is it necessary?
        //let is_pinned: bool;
        let mut pinners: BitBoard = BitBoard::ZERO;
        let mut pin_mask: BitBoard = BitBoard::ZERO;
        //let mut chessboard: ChessBoard = self.duplicate();
        let piece = self.mailbox[square.to_index()].expect("is_piece_pinned error: mailbox square is empty!");
        //let mut piece_bb = self.piece_bbs[cp_index(piece)].bit_and(&BitBoard::nth(square).bit_not());
        let mut mailbox = self.mailbox;
        mailbox[cp_index(piece)] = None;
        let side = self.side_to_move;
        //assert!(matches!(piece, cpt!(K) | cpt!(k)) == false, "is_piece_pinned error: checking if king is pinned");
        if matches!(piece, cpt!(K) | cpt!(k)) {
            return (BitBoard::ZERO, BitBoard::ZERO);
        }
        if self.is_king_in_check(side) == false {
            // if king is not in check, test if removing piece causes king to be in check
            //is_pinned = chessboard.is_king_in_check(side); //FIXME
        } else {
            //check if king are present for both sides
            assert!(self.piece_bbs[cpt_index!(K)].count_ones() == 1 && self.piece_bbs[cpt_index!(k)].count_ones() == 1);
            let king_square: Square;
            let diagonals: BitBoard;
            let laterals: BitBoard;
            let enemies: BitBoard;
            match side {
                Side::White => {
                    king_square = self.piece_bbs[cpt_index!(K)].lsb_square().unwrap();
                    diagonals = self.piece_bbs[cpt_index!(q)].bit_or(&self.piece_bbs[cpt_index!(b)]);
                    laterals = self.piece_bbs[cpt_index!(q)].bit_or(&self.piece_bbs[cpt_index!(r)]);
                    enemies = self.black_blockers();
                }
                Side::Black => {
                    king_square = self.piece_bbs[cpt_index!(k)].lsb_square().unwrap();
                    diagonals = self.piece_bbs[cpt_index!(Q)].bit_or(&self.piece_bbs[cpt_index!(B)]);
                    laterals = self.piece_bbs[cpt_index!(Q)].bit_or(&self.piece_bbs[cpt_index!(R)]);
                    enemies = self.white_blockers();
                }
            }
            // blockers without friendly king
            let removed_blockers = self.blockers().bit_and(&BitBoard::nth(square).bit_not());
            // position of diagonal attackers
            let diag_attackers = get_bishop_attack(square, removed_blockers).bit_and(&diagonals);
            // position of lateral attackers
            let latr_attackers = get_rook_attack(square, removed_blockers).bit_and(&laterals);

            // if king is in check, see if square is in ray of a checking piece
            // note: why are we testing for all rays from the king here?
            let mut potential_pinners = enemies.bit_and(&diag_attackers.bit_or(&latr_attackers));
            while potential_pinners.is_not_zero() {
                let potential_pinner = potential_pinners.lsb_square().unwrap();
                // check if piece is between king and potential_pinner
                if RAYS[king_square.to_index()][potential_pinner.to_index()].nth_is_not_zero(square) {
                    //is_pinned = true;
                    pinners = pinners.bit_or(&BitBoard::nth(potential_pinner));
                    pin_mask = pin_mask.bit_or(&RAYS[king_square.to_index()][potential_pinner.to_index()].bit_or(&BitBoard::nth(potential_pinner)))
                }
                potential_pinners = potential_pinners.pop_bit(potential_pinner);
            }
        }
        //is_pinned = false;
        return (pinners, pin_mask);
    }

    // calculates all the squares attacked by enemy's checking pieces
    pub(crate) fn check_mask(&self) -> BitBoard {
        let mut check_bb: BitBoard = self.check_bb;
        let mut check_mask = check_bb;
        while check_bb.is_not_zero() {
            let checker_square = check_bb.lsb_square().unwrap();
            match self.mailbox[checker_square.to_index()].expect("generate_moves: checker mailbox is empty") {
                cpt!(K) | cpt!(k) => panic!("generate_moves: king is in check by another king!"),
                cpt!(N) | cpt!(n) => continue,
                _ => {
                    check_mask = check_mask.bit_or(&RAYS[checker_square.to_index()][self.king_square().to_index()]);
                }
            }
            check_bb = check_bb.pop_bit(checker_square);
        }
        return check_mask;
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

    pub(crate) fn piece_bb(&self, piece_type: ChessPiece) -> BitBoard {
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
