use std::pin;

use super::chessmove::*;
use super::*;

//not const?

const fn update_state(chessboard: &ChessBoard, chess_move: ChessMove) -> ChessBoard {
    todo!();
}

fn generate_moves(chessboard: &ChessBoard) -> Vec<ChessMove> {
    let mut moves: Vec<ChessMove> = Vec::new();
    //if three fold repetition, return empty set of moves

    //let blockers = chessboard.blockers();
    //let w_blockers = chessboard.white_blockers();
    //let b_blockers = chessboard.black_blockers();
    let side = chessboard.side_to_move;
    //let king_source: Square;
    //let king_castling_mask: BitBoard;
    //let king_castling_index: usize;
    //let queen_castling_mask: BitBoard;
    //let queen_castling_index: usize;
    //let friends: BitBoard;
    //let enemies: BitBoard;
    //
    //match side {
    //    Side::White => {
    //        king_source = match chessboard.piece_bbs[cpt_index!(K)].lsb_square() {
    //            Some(source) => source,
    //            None => panic!("generate moves: white king not found"),
    //        };
    //        king_castling_mask = W_KING_SIDE_CASTLE_MASK;
    //        king_castling_index = 0;
    //        queen_castling_mask = W_QUEEN_SIDE_CASTLE_MASK;
    //        queen_castling_index = 1;
    //        friends = w_blockers;
    //        enemies = b_blockers;
    //    }
    //    Side::Black => {
    //        king_source = match chessboard.piece_bbs[cpt_index!(k)].lsb_square() {
    //            Some(source) => source,
    //            None => panic!("generate moves: black king not found"),
    //        };
    //        king_castling_mask = B_KING_SIDE_CASTLE_MASK;
    //        king_castling_index = 2;
    //        queen_castling_mask = B_QUEEN_SIDE_CASTLE_MASK;
    //        queen_castling_index = 3;
    //        friends = b_blockers;
    //        enemies = w_blockers;
    //    }
    //}

    // consider if king is in check
    // calculates all the sources attacked by enemy's checking pieces
    let check_mask: BitBoard = chessboard.check_mask();
    let checkers_count = chessboard.check_bb.count_ones();

    for &piece_type in PieceType::iterator() {
        // if double check => king move (triple and higher checks impossible?)
        if checkers_count >= 2 && piece_type != PieceType::King {
            continue;
        }
        // if single check => king move, piece blocks, capture checker

        let mut sources = chessboard.piece_bb((side, piece_type));
        while sources.is_not_zero() {
            let source: Square = sources.lsb_square().unwrap();
            let (pinners, pin_mask) = chessboard.calculate_pin_data(source);
            match piece_type {
                PieceType::King => {
                    /* castling */
                    // cannot castle if in check
                    if chessboard.check_bb.is_zero() {
                        // king-side castle
                        if chessboard.is_able_kingside_castle(side) {
                            match side {
                                Side::White => moves.push(ChessMove::W_KINGSIDE_CASTLE),
                                Side::Black => moves.push(ChessMove::W_QUEENSIDE_CASTLE),
                            }
                        }
                        // queen-side castle
                        if chessboard.is_able_queenside_castle(side) {
                            match side {
                                Side::White => moves.push(ChessMove::B_KINGSIDE_CASTLE),
                                Side::Black => moves.push(ChessMove::B_QUEENSIDE_CASTLE),
                            }
                        }
                    }
                    /* moves and attacks */
                    moves.append(&mut calculate_attacks(chessboard, source, piece_type, pin_mask, check_mask));
                }
                PieceType::Knight => {
                    // pinned knights can not move
                    if pin_mask.is_not_zero() {
                        sources = sources.pop_bit(source);
                        continue;
                    }
                    //TODO this might be a source of error
                    moves.append(&mut calculate_attacks(chessboard, source, piece_type, pin_mask, check_mask));
                }
                PieceType::Pawn => {
                    moves.append(&mut calculate_pawn_moves(chessboard, source, pinners, pin_mask, check_mask))
                }
                _ => moves.append(&mut calculate_attacks(chessboard, source, piece_type, pin_mask, check_mask)),
            }
            sources = sources.pop_bit(source);
        }
    }
    //
    todo!();
}

fn calculate_attacks(cb: &ChessBoard, s: Square, p: PieceType, p_m: BitBoard, c_m: BitBoard) -> Vec<ChessMove> {
    let source = s;
    let piece_type = p;
    let side = cb.side_to_move;
    let friends: BitBoard;
    let enemies: BitBoard;
    let blockers: BitBoard = cb.blockers();
    let pin_mask = p_m;
    let check_mask = c_m;
    match side {
        Side::White => {
            friends = cb.white_blockers();
            enemies = cb.black_blockers();
        }
        Side::Black => {
            friends = cb.black_blockers();
            enemies = cb.white_blockers();
        }
    }

    let mut moves: Vec<ChessMove> = Vec::new();
    let mut targets = match piece_type {
        PieceType::King => get_king_attack(source).bit_and(&friends),
        PieceType::Queen => get_queen_attack(source, blockers).bit_and(&friends.bit_not()),
        PieceType::Knight => get_knight_attack(source).bit_and(&friends.bit_not()),
        PieceType::Bishop => get_bishop_attack(source, blockers).bit_and(&friends.bit_not()),
        PieceType::Rook => get_rook_attack(source, blockers).bit_and(&friends.bit_not()),
        PieceType::Pawn => match side {
            Side::White => get_w_pawn_attack(source).bit_and(&enemies),
            Side::Black => get_b_pawn_attack(source).bit_and(&enemies),
        },
    };

    //pawn rules are complex, best handled separately
    assert!(piece_type != PieceType::Pawn);

    while targets.is_not_zero() {
        let target = targets.lsb_square().unwrap();

        //just in case...
        assert!(piece_type != PieceType::Pawn);
        assert!(piece_type != PieceType::Knight || pin_mask.is_zero());

        //NOTE (special cases)
        //king: cannot move to a square under attack
        if piece_type == PieceType::King && cb.is_square_attacked_removed_piece(target, side, cb.king_square()) {
            targets = targets.pop_bit(target);
            continue;
        };

        //logic here
        //pinned logic
        //only consider moves along pinning ray if pinned
        if pin_mask.is_not_zero() && pin_mask.nth_is_zero(target) {
            targets = targets.pop_bit(target);
            //FIXME is this necessary?
            //assert!(piece_type != PieceType::Knight);
            continue;
        }

        //checked logic
        //only consider moves along checking ray if in check
        if check_mask.is_not_zero() && check_mask.nth_is_zero(target) {
            targets = targets.pop_bit(target);
            continue;
        }

        //append moves
        moves.push(ChessMove::new(source, target, MoveType::Normal));
        targets = targets.pop_bit(target);
    }
    return moves;
}

fn calculate_pawn_moves(
    chessboard: &ChessBoard,
    source: Square,
    pinners: BitBoard,
    pin_mask: BitBoard,
    check_mask: BitBoard,
) -> Vec<ChessMove> {
    let king_square = chessboard.king_square();
    let blockers = chessboard.blockers();
    let side = chessboard.side_to_move;
    let mut moves: Vec<ChessMove> = Vec::new();

    let mut is_pinned_diag: bool = false;
    let mut is_pinned_vert: bool = false;
    let mut is_pinned_horz: bool = false;

    let promotion_row = match side {
        Side::White => 7,
        Side::Black => 0,
    };

    if pin_mask.is_not_zero() {
        let mut pinners = pinners;
        while pinners.is_not_zero() {
            let square = pinners.lsb_square().unwrap();
            assert!(source != square);
            is_pinned_diag = is_pinned_diag || is_same_diag(source, square);
            is_pinned_vert = is_pinned_vert || is_same_col(source, square);
            is_pinned_horz = is_pinned_horz || is_same_row(source, square);
            pinners = pinners.pop_bit(square);
        }
    }

    let next = match side {
        //TODO safeguards(?)
        Side::White => Square::new(source.to_u8() + 8),
        Side::Black => Square::new(source.to_u8() - 8),
    };

    // ~p ^ ~q <=> ~(p v q)
    /* pawn move - one square */
    if (is_pinned_diag || is_pinned_horz) == false {
        // one-square pawn move
        let target = next;
        // can only move one square if next square is empty
        if blockers.nth_is_zero(target) {
            //FIXME assumption checkers_count == 1
            // can only move one-square if not in check, or blocks check
            if check_mask.is_zero() || check_mask.nth_is_not_zero(target) {
                match (ROWS[target.to_index()] == promotion_row) {
                    true => moves.append(&mut ChessMove::promotions(source, target).to_vec()),
                    false => moves.push(ChessMove::new(source, target, MoveType::Normal)),
                }
            }
        }
    }

    let attack_mask = match side {
        Side::White => get_w_pawn_attack(source).bit_and(&chessboard.black_blockers()),
        Side::Black => get_b_pawn_attack(source).bit_and(&chessboard.white_blockers()),
    };

    /* pawn move - two squares */
    let starting_row = match chessboard.side_to_move {
        Side::White => 1,
        Side::Black => 6,
    };
    if ROWS[source.to_index()] == starting_row {
        let target = match side {
            Side::White => Square::new(source.to_u8() + 16),
            Side::Black => Square::new(source.to_u8() - 16),
        };

        //can only move two-squares if pawn is in starting row, and next two squares are empty
        if blockers.bit_and(&BitBoard::nth(next).bit_or(&BitBoard::nth(target))).is_zero() {
            // can only move two-squares if not in check, or blocks check
            if check_mask.is_zero() || check_mask.nth_is_not_zero(target) {
                moves.push(ChessMove::new(source, target, MoveType::Normal));
            }
        }

        // ~p ^ ~q <=> ~(p v q)
        //if (is_pinned_horz == false) && (is_pinned_vert == false)
        /* pawn attacks */

        if (is_pinned_horz || is_pinned_vert) == false {
            let mut attacks = attack_mask;
            while attacks.is_not_zero() {
                let attack = attacks.lsb_square().unwrap();
                //FIXME assumption checkers count == 1
                //can only attack a square if not in check or attack blocks check
                if check_mask.is_zero() || (check_mask.nth_is_not_zero(attack)) {
                    //can only attack a square if not pinned or attack is along a pin-ray
                    if pin_mask.is_zero() || pin_mask.nth_is_not_zero(attack) {
                        match (ROWS[attack.to_index()] == promotion_row) {
                            true => moves.append(&mut ChessMove::promotions(source, attack).to_vec()),
                            false => moves.push(ChessMove::new(source, attack, MoveType::Normal)),
                        }
                    }
                }
                attacks = attacks.pop_bit(target);
            }
        }
    }
    //TODO can we use is_pinned_horz, is_pinned_vert, is_pinned_vert in place of is_piece_pinned()?
    /* pawn en-passant */
    if chessboard.enpassant_bb.is_not_zero() && (chessboard.is_piece_pinned(source) == false) {
        let mut attacks = match side {
            Side::White => chessboard.enpassant_bb.bit_and(&get_w_pawn_attack(source)),
            Side::Black => chessboard.enpassant_bb.bit_and(&get_b_pawn_attack(source)),
        };

        while attacks.is_not_zero() {
            let attack = attacks.lsb_square().unwrap();

            //special psuedo-pinned pawn case:
            // R . p P k
            // . . . ^ .
            // . . . | .
            // . . . x .

            let row_bb = BitBoard::new(0b11111111u64 << (8 * ROWS[source.to_index()]));

            let enemy_rook_index;
            let enemy_pawn_index;
            let enemy_pawn_square;

            match side {
                Side::White => {
                    enemy_rook_index = cpt_index!(r);
                    enemy_pawn_index = cpt_index!(p);
                    enemy_pawn_square = Square::new(attack.to_u8() - 8u8);
                }
                Side::Black => {
                    enemy_rook_index = cpt_index!(R);
                    enemy_pawn_index = cpt_index!(P);
                    enemy_pawn_square = Square::new(attack.to_u8() + 8u8);
                }
            }

            //if enemy rook and friendly king is in the same row, check for special case
            if (ROWS[king_square.to_index()] == ROWS[source.to_index()])
                && (chessboard.piece_bbs[enemy_rook_index].bit_and(&row_bb).is_not_zero())
            {
                //check if en-passant leaves king in check
                //FIXME this is computationally costly
                let mut test_cb = chessboard.duplicate();
                let i = match side {
                    Side::White => cpt_index!(P),
                    Side::Black => cpt_index!(p),
                };
                test_cb.piece_bbs[i] = test_cb.piece_bbs[i].bit_and(&BitBoard::nth(source).bit_not());
                test_cb.piece_bbs[i] = test_cb.piece_bbs[i].bit_and(&BitBoard::nth(attack));
                test_cb.piece_bbs[enemy_pawn_index] =
                    test_cb.piece_bbs[enemy_pawn_index].bit_and(&BitBoard::nth(enemy_pawn_square).bit_not());

                if test_cb.is_king_in_check(side) {
                    attacks = attacks.pop_bit(attack);
                    continue;
                }

                //if there are no checks
                if chessboard.check_bb.is_not_zero() {
                    moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                    attacks = attacks.pop_bit(attack);
                    continue;
                }

                //if in check, can only en-passant to remove checking pawn
                if chessboard.check_bb.count_ones() == 1 {
                    let checker_square = chessboard.check_bb.lsb_square().unwrap();
                    if checker_square == enemy_pawn_square {
                        moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                    }
                }
                attacks = attacks.pop_bit(attack);
            }
        }
    }
    return moves;
}

const fn king_checkers(chessboard: &ChessBoard) -> (BitBoard, u32) {
    let mut check_mask: BitBoard;
    todo!();
}

const fn king_moves(chessboard: &ChessBoard) -> Vec<ChessMove> {
    if chessboard.check_bb.is_zero() {
        todo!()
    }
    todo!()
}

//queen, rook, bishops
fn ray_moves(source: Square, mut attacks: BitBoard) -> Vec<ChessMove> {
    //let mut attacks: BitBoard = get_queen_attack(source, blockers).bit_and(&friends.bit_not());
    let mut moves: Vec<ChessMove> = Vec::<ChessMove>::new();
    while attacks.is_not_zero() {
        let target = attacks.lsb_square().expect("attacks should not be empty");
        // only consider moves along pinning ray if pinned
        // only consider moves along checking ray if in check
        moves.push(ChessMove::new(source, target, MoveType::Normal));
        attacks = attacks.pop_bit(target);
    }
    return moves;
}

//pub const fn get_history(&self) -> [([BitBoard; 12], u16); 8] {
//    let mut new = self.history;
//    let mut i = 0;
//    while i < 7 {
//        new[i] = self.history[i + 1];
//        i += 1;
//    }
//    new[7] = (self.piece_bbs, self.rep_count());
//
//    return new;
//}

/* ================ additional ChessMove-specific implementations ================ */

// originally this was a separate implementation "ConstVec", but I decided to not be overly general
//struct Moves {
//    data: [Option<ChessMove>; 256],
//    count: usize,
//}

//impl Moves {
//    const MAX_CAPACITY: usize = 256;
//    const DEFAULT: Option<ChessMove> = None;
//
//    const fn new() -> Self {
//        Moves { data: [Self::DEFAULT; Self::MAX_CAPACITY], count: 0 }
//    }
//
//    const fn len(&self) -> usize {
//        return self.count;
//    }
//
//    const fn capacity(&self) -> usize {
//        return Self::MAX_CAPACITY - self.len();
//    }
//
//    const fn push(&mut self, t: Option<ChessMove>) {
//        assert!(self.len() < Self::MAX_CAPACITY);
//        self.data[self.len()] = t;
//        self.count += 1;
//    }
//
//    const fn nth(&self, index: usize) -> Option<ChessMove> {
//        assert!(index < Self::MAX_CAPACITY);
//        self.data[index]
//    }
//
//    ////indexing alternative
//    //pub fn set(&mut self, index: usize, data: T) {
//    //    self.data[index] = data;
//    //}
//    //
//    //pub const fn is_empty(&self) -> bool {
//    //    self.count == 0
//    //}
//    //
//    //pub const fn is_full(&self) -> bool {
//    //    self.count == Self::MAX_CAPACITY
//    //}
//    //
//    //pub const fn head(&self) -> T {
//    //    assert!(self.count > 0);
//    //    return self.data[0];
//    //}
//    //
//    //pub const fn tail(&self) -> T {
//    //    assert!(self.count > 0);
//    //    return self.data[self.count - 1];
//    //}
//    //
//    //pub fn pop(&mut self) -> Option<T> {
//    //    if self.count == 0 {
//    //        return None;
//    //    }
//    //    Some(self.data[self.count - 1])
//    //}
//    //
//    ////const pop alternative
//    //pub const fn unappend(self) -> (Self, Option<T>) {
//    //    if self.count == 0 {
//    //        return (self, None);
//    //    }
//    //    let mut new = ConstVec::<T, C> { data: self.data, count: self.count - 1 };
//    //    let x = self.data[self.count - 1];
//    //    new.data[self.count - 1] = Self::DEFAULT;
//    //    return (new, Some(x));
//    //}
//}
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
