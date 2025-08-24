use super::chessmove::*;
use super::*;

impl ChessBoardCore {
    pub fn update_state(&mut self, chess_move: ChessMove) {
        let mut enpassant_bb: BitBoard = BitBoard::ZERO;
        let source: Square = chess_move.source();
        let target: Square = chess_move.target();
        let source_piece = self.mailbox[source.to_usize()].expect("update_state error: source mailbox is None");
        let source_index = cp_index(source_piece);
        let target_piece = self.mailbox[target.to_usize()];

        let enemy_king_index: usize = match self.side() {
            Side::White => 6,
            Side::Black => 0,
        };
        assert!(self.piece_bbs[enemy_king_index].nth_is_zero(target), "position:{}\n\r\n\r\n\r\n\r", self);
        let mut current_hash = self.hash();
        current_hash ^= ZobristHash::compute_enpassant_hash(self.enpassant_bb);

        let mut is_counter_reset: bool = false; //fifty-move-rule counter

        /* special case bookkeeping */
        match source_piece {
            /* castling */
            (Side::White, PieceType::King) => {
                if self.castle_bools[0] {
                    current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::White));
                }
                self.castle_bools[0] = false;
                if self.castle_bools[1] {
                    current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::White));
                }
                self.castle_bools[1] = false;
            }

            (Side::Black, PieceType::King) => {
                if self.castle_bools[2] {
                    current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::Black));
                }
                self.castle_bools[2] = false;
                if self.castle_bools[3] {
                    current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::Black));
                }
                self.castle_bools[3] = false;
            }

            (Side::White, PieceType::Rook) => {
                if source == Square::new(0) {
                    if self.castle_bools[0] {
                        current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::White));
                    }
                    self.castle_bools[0] = false;
                } else if source == Square::new(7) {
                    if self.castle_bools[1] {
                        current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::White));
                    }
                    self.castle_bools[1] = false
                }
            }

            (Side::Black, PieceType::Rook) => {
                if source == Square::new(56) {
                    if self.castle_bools[2] {
                        current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::Black));
                    }
                    self.castle_bools[2] = false;
                } else if source == Square::new(63) {
                    if self.castle_bools[3] {
                        current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::Black));
                    }
                    self.castle_bools[3] = false
                }
            }

            /* en-passant and fifty-move-rule */
            (Side::White, PieceType::Pawn) => {
                //reset 50-move rule
                self.fifty_move_rule_counter = 0;
                is_counter_reset = true;
                //if move is a 2-square pawn move, update en-passant bitboard
                if self.is_pawn_move_enpassant_relevant(&source, &target) {
                    //FIXME should check if enpassant is even legan for enemy
                    enpassant_bb.set_bit(Square::new(target.to_u8() - 8));
                }
            }

            (Side::Black, PieceType::Pawn) => {
                //reset 50-move rule
                self.fifty_move_rule_counter = 0;
                is_counter_reset = true;
                //if move is a 2-square pawn move, update en-passant bitboard
                if self.is_pawn_move_enpassant_relevant(&source, &target) {
                    //FIXME should check if enpassant is even legan for enemy
                    enpassant_bb.set_bit(Square::new(target.to_u8() + 8));
                }
            }
            _ => (),
        }
        //TODO continue here
        //move the piece
        self.piece_bbs[source_index].pop_bit(source);
        self.piece_bbs[source_index].set_bit(target);
        current_hash ^= ZobristHash::piece_hash(source, source_piece);
        current_hash ^= ZobristHash::piece_hash(target, source_piece);
        self.mailbox[source.to_usize()] = None;
        self.mailbox[target.to_usize()] = Some(source_piece);

        //additional book keeping
        match chess_move.move_type() {
            MoveType::Normal => {
                //dealing with captures
                if let Some(target_piece) = target_piece {
                    let target_index = cp_index(target_piece);
                    self.piece_bbs[target_index].pop_bit(target);
                    current_hash ^= ZobristHash::piece_hash(target, target_piece);

                    //reset 50-move rule
                    self.fifty_move_rule_counter = 0;
                    is_counter_reset = true;

                    //if capturing enemy rook, update castling rights
                    match (target_piece, target.to_u8()) {
                        (cpt!(R), 00u8) => {
                            if self.castle_bools[0] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::White));
                            }
                            self.castle_bools[0] = false;
                        }
                        (cpt!(R), 07u8) => {
                            if self.castle_bools[1] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::White));
                            }
                            self.castle_bools[1] = false;
                        }
                        (cpt!(r), 56u8) => {
                            if self.castle_bools[2] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::Black));
                            }
                            self.castle_bools[2] = false;
                        }
                        (cpt!(r), 63u8) => {
                            if self.castle_bools[3] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::Black));
                            }
                            self.castle_bools[3] = false;
                        }
                        _ => (),
                    }
                }
            }

            MoveType::Castle(castling) => {
                let white_kingside_rook_sq_source: Square = Square::new(00);
                let white_kingside_rook_sq_target: Square = Square::new(02);
                let white_queenside_rook_sq_source: Square = Square::new(07);
                let white_queenside_rook_sq_target: Square = Square::new(04);
                let black_kingside_rook_sq_source: Square = Square::new(56);
                let black_kingside_rook_sq_target: Square = Square::new(58);
                let black_queenside_rook_sq_source: Square = Square::new(63);
                let black_queenside_rook_sq_target: Square = Square::new(60);
                match castling {
                    Castling::Kingside(Side::White) => {
                        // check if rook is present
                        assert!(self.piece_bbs[04].nth_is_not_zero(white_kingside_rook_sq_source));
                        self.piece_bbs[04].pop_bit(white_kingside_rook_sq_source);
                        self.piece_bbs[04].set_bit(white_kingside_rook_sq_target);
                        self.mailbox[white_kingside_rook_sq_source.to_usize()] = None;
                        self.mailbox[white_kingside_rook_sq_target.to_usize()] = opt_cpt!(R);

                        //update hash
                        current_hash ^= ZobristHash::piece_hash(white_kingside_rook_sq_source, cpt!(R));
                        current_hash ^= ZobristHash::piece_hash(white_kingside_rook_sq_target, cpt!(R));
                    }

                    Castling::Queenside(Side::White) => {
                        // check if rook is present
                        assert!(self.piece_bbs[04].nth_is_not_zero(white_queenside_rook_sq_source));
                        self.piece_bbs[04].pop_bit(white_queenside_rook_sq_source);
                        self.piece_bbs[04].set_bit(white_queenside_rook_sq_target);
                        self.mailbox[white_queenside_rook_sq_source.to_usize()] = None;
                        self.mailbox[white_queenside_rook_sq_target.to_usize()] = opt_cpt!(R);

                        //update hash
                        current_hash ^= ZobristHash::piece_hash(white_queenside_rook_sq_source, cpt!(R));
                        current_hash ^= ZobristHash::piece_hash(white_queenside_rook_sq_target, cpt!(R));
                    }

                    Castling::Kingside(Side::Black) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].nth_is_not_zero(black_kingside_rook_sq_source));
                        self.piece_bbs[10].pop_bit(black_kingside_rook_sq_source);
                        self.piece_bbs[10].set_bit(black_kingside_rook_sq_target);
                        self.mailbox[black_kingside_rook_sq_source.to_usize()] = None;
                        self.mailbox[black_kingside_rook_sq_target.to_usize()] = opt_cpt!(r);

                        //update hash
                        current_hash ^= ZobristHash::piece_hash(black_kingside_rook_sq_source, cpt!(r));
                        current_hash ^= ZobristHash::piece_hash(black_kingside_rook_sq_target, cpt!(r));
                    }

                    Castling::Queenside(Side::Black) => {
                        // check if rook is present
                        assert!(self.piece_bbs[10].nth_is_not_zero(black_queenside_rook_sq_source));
                        self.piece_bbs[10].pop_bit(black_queenside_rook_sq_source);
                        self.piece_bbs[10].set_bit(black_queenside_rook_sq_target);
                        self.mailbox[black_queenside_rook_sq_source.to_usize()] = None;
                        self.mailbox[black_queenside_rook_sq_target.to_usize()] = opt_cpt!(r);

                        //update hash
                        current_hash ^= ZobristHash::piece_hash(black_queenside_rook_sq_source, cpt!(r));
                        current_hash ^= ZobristHash::piece_hash(black_queenside_rook_sq_target, cpt!(r));
                    }
                }
            }

            MoveType::EnPassant => {
                let enemy_pawn_index: usize;
                let enemy_pawn_square: Square;
                let enemy_piece: ChessPiece;
                match self.side_to_move {
                    Side::White => {
                        enemy_pawn_index = 11;
                        enemy_pawn_square = Square::new((target.to_usize() - 8) as u8);
                        enemy_piece = (Side::Black, PieceType::Pawn);
                    }
                    Side::Black => {
                        enemy_pawn_index = 05;
                        enemy_pawn_square = Square::new((target.to_usize() + 8) as u8);
                        enemy_piece = (Side::White, PieceType::Pawn);
                    }
                }

                assert!(self.piece_bbs[enemy_pawn_index].nth_is_not_zero(enemy_pawn_square));
                assert!(
                    self.mailbox[enemy_pawn_square.to_usize()] == opt_cpt!(p)
                        || self.mailbox[enemy_pawn_square.to_usize()] == opt_cpt!(P)
                );

                self.piece_bbs[enemy_pawn_index].pop_bit(enemy_pawn_square);
                current_hash ^= ZobristHash::piece_hash(enemy_pawn_square, enemy_piece);
                self.mailbox[enemy_pawn_square.to_usize()] = None;
            }

            MoveType::Promotion(piece_type) => {
                let promoted_piece = (self.side_to_move, piece_type);
                let promoted_index = cp_index(promoted_piece);

                //dealing with captures
                if let Some(target_piece) = target_piece {
                    let target_index = cp_index(target_piece);
                    self.piece_bbs[target_index].pop_bit(target);
                    current_hash ^= ZobristHash::piece_hash(target, target_piece);

                    //reset 50-move rule
                    self.fifty_move_rule_counter = 0;
                    is_counter_reset = true;

                    //if capturing enemy rook, update castling rights
                    match (target_piece, target.to_u8()) {
                        (cpt!(R), 00u8) => {
                            if self.castle_bools[0] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::White));
                            }
                            self.castle_bools[0] = false;
                        }
                        (cpt!(R), 07u8) => {
                            if self.castle_bools[1] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::White));
                            }
                            self.castle_bools[1] = false;
                        }
                        (cpt!(r), 56u8) => {
                            if self.castle_bools[2] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Kingside(Side::Black));
                            }
                            self.castle_bools[2] = false;
                        }
                        (cpt!(r), 63u8) => {
                            if self.castle_bools[3] {
                                current_hash ^= ZobristHash::castle_hash(Castling::Queenside(Side::Black));
                            }
                            self.castle_bools[3] = false;
                        }
                        _ => (),
                    }
                }

                //remove the pawn piece
                self.piece_bbs[source_index].pop_bit(target);
                current_hash ^= ZobristHash::piece_hash(target, source_piece);

                //add the promoted piece
                self.piece_bbs[promoted_index].set_bit(target);
                current_hash ^= ZobristHash::piece_hash(target, promoted_piece);
                self.mailbox[target.to_usize()] = Some(promoted_piece);
            }
        }

        if self.side_to_move == Side::Black {
            self.full_move_counter += 1;
        }

        self.side_to_move = self.side_to_move.update();
        current_hash ^= ZobristHash::side_hash();
        if is_counter_reset == false {
            self.fifty_move_rule_counter += 1;
        }

        self.enpassant_bb = enpassant_bb;
        current_hash ^= ZobristHash::compute_enpassant_hash(enpassant_bb);

        self.zobrist_hash = current_hash;

        self.compute_check_bb();
        self.compute_check_mask();
        self.compute_pin_data();
    }

    pub fn generate_moves(&self) -> Vec<ChessMove> {
        let mut moves: Vec<ChessMove> = Vec::new();
        let mut king_moves: Vec<ChessMove> = Vec::new();
        let side = self.side_to_move;

        // consider if king is in check
        let checkers_count = self.check_bb.count_ones();

        for &piece_type in PieceType::iterator() {
            // if double check, king move (triple and higher checks impossible?)
            if checkers_count >= 2 && piece_type != PieceType::King {
                continue;
            }

            let mut sources = self.piece_bb((side, piece_type));
            while sources.is_not_zero() {
                let source: Square = sources.lsb_square().unwrap();
                let pinned_pieces = self.pinned_bb;

                match piece_type {
                    PieceType::King => {
                        /* castling */

                        // cannot castle if in check
                        if self.check_bb.is_zero() {
                            // king-side castle
                            if self.is_able_kingside_castle(side) {
                                match side {
                                    Side::White => moves.push(ChessMove::W_KINGSIDE_CASTLE),
                                    Side::Black => moves.push(ChessMove::B_KINGSIDE_CASTLE),
                                }
                            }
                            // queen-side castle
                            if self.is_able_queenside_castle(side) {
                                match side {
                                    Side::White => moves.push(ChessMove::W_QUEENSIDE_CASTLE),
                                    Side::Black => moves.push(ChessMove::B_QUEENSIDE_CASTLE),
                                }
                            }
                        }

                        /* moves and attacks */
                        king_moves.append(&mut self.calculate_moves(source, piece_type));
                        sources.pop_bit(source);
                    }

                    PieceType::Knight => {
                        // pinned knights can not move
                        if pinned_pieces.nth_is_not_zero(source) {
                            sources.pop_bit(source);
                            continue;
                        }
                        /* moves and attacks */
                        moves.append(&mut self.calculate_moves(source, piece_type));
                        sources.pop_bit(source);
                    }

                    _ => {
                        /* moves and attacks */
                        moves.append(&mut self.calculate_moves(source, piece_type));
                        sources.pop_bit(source);
                    }
                }

                // /* moves and attacks */
                // moves.append(&mut self.calculate_moves(source, piece_type));
                // sources.pop_bit(source);
            }
        }
        moves.append(&mut king_moves);
        return moves;
    }

    fn calculate_moves(&self, source: Square, piece_type: PieceType) -> Vec<ChessMove> {
        //pawn rules are complex, best handled separately, use calculate_pawn_moves()
        if matches!(piece_type, PieceType::Pawn) {
            return self.calculate_pawn_moves(source);
        }

        let check_mask = self.check_mask;
        let side = self.side_to_move;
        let blockers: BitBoard = self.blockers();
        let (friends, enemies) = match side {
            Side::White => (self.white_blockers(), self.black_blockers()),
            Side::Black => (self.black_blockers(), self.white_blockers()),
        };

        let mut moves: Vec<ChessMove> = Vec::new();
        let mut targets = match piece_type {
            PieceType::King => get_king_attack(source).bit_and(&friends.bit_not()),
            PieceType::Queen => get_queen_attack(source, blockers).bit_and(&friends.bit_not()),
            PieceType::Knight => get_knight_attack(source).bit_and(&friends.bit_not()),
            PieceType::Bishop => get_bishop_attack(source, blockers).bit_and(&friends.bit_not()),
            PieceType::Rook => get_rook_attack(source, blockers).bit_and(&friends.bit_not()),
            PieceType::Pawn => match side {
                Side::White => get_w_pawn_attack(source).bit_and(&enemies),
                Side::Black => get_b_pawn_attack(source).bit_and(&enemies),
            },
        };

        // only consider moves along pinning rays, if pinned
        let pin_mask: BitBoard = self.pin_mask(source);
        if pin_mask.is_not_zero() {
            targets = targets.bit_and(&pin_mask);
        }

        //only consider moves along checking ray if in check, unless piece is your king
        if self.check_bb.is_not_zero() && piece_type != PieceType::King {
            targets = targets.bit_and(&check_mask.bit_or(&self.check_bb));
        }

        while targets.is_not_zero() {
            let target = targets.lsb_square().unwrap();

            //king: cannot move to a square under attack
            let blockers = match self.side_to_move {
                Side::White => self.blockers_no_white_king().bit_and(&BitBoard::nth(target).bit_not()),
                Side::Black => self.blockers_no_black_king().bit_and(&BitBoard::nth(target).bit_not()),
            };

            if piece_type == PieceType::King && self.is_square_attacked(target, side.update(), blockers) {
                targets.pop_bit(target);
                continue;
            };

            //append moves
            moves.push(ChessMove::new(source, target, MoveType::Normal));
            targets.pop_bit(target);
        }
        return moves;
    }

    fn calculate_pawn_moves(&self, source: Square) -> Vec<ChessMove> {
        let pinners = self.pinner_bb;
        let pin_mask = self.pin_mask(source);
        let check_mask = self.check_mask;
        let king_square = self.king_square();
        let blockers = self.blockers();
        let side = self.side_to_move;

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
                let pinner = pinners.lsb_square().unwrap();
                let piece_type = self.mailbox[pinner.to_usize()].unwrap();

                is_pinned_diag |= is_same_diag_tri(source, pinner, king_square)
                    && matches!(piece_type, (_, PieceType::Bishop) | (_, PieceType::Queen));
                is_pinned_vert |= is_same_col_tri(source, pinner, king_square)
                    && matches!(piece_type, (_, PieceType::Rook) | (_, PieceType::Queen));
                is_pinned_horz |= is_same_row_tri(source, pinner, king_square)
                    && matches!(piece_type, (_, PieceType::Rook) | (_, PieceType::Queen));
                pinners.pop_bit(pinner);
            }
        }

        //pawn should not be in the first nor last row for either side
        debug_assert!(55 >= source.to_u8() && source.to_u8() >= 8);
        let next = match side {
            Side::White => Square::new(source.to_u8() + 8),
            Side::Black => Square::new(source.to_u8() - 8),
        };

        // this is equivalent to: !is_pinned_diag && !is_pinned_horz, due to ~p ^ ~q <=> ~(p v q)
        if !(is_pinned_diag || is_pinned_horz) {
            /* pawn move - one square */
            let target = next;
            // can only move one square if next square is empty
            if blockers.nth_is_zero(target) {
                debug_assert!(self.check_bb.count_ones() <= 1);
                // can only move one-square if not in check, or blocks check
                if check_mask.is_zero() || check_mask.nth_is_not_zero(target) {
                    match ROWS[target.to_usize()] == promotion_row {
                        true => moves.append(&mut ChessMove::promotions(source, target).to_vec()),
                        false => moves.push(ChessMove::new(source, target, MoveType::Normal)),
                    }
                }
            }

            /* pawn move - two squares */
            let starting_row = match self.side_to_move {
                Side::White => 1,
                Side::Black => 6,
            };

            if ROWS[source.to_usize()] == starting_row {
                let target = match side {
                    Side::White => Square::new(source.to_u8() + 16),
                    Side::Black => Square::new(source.to_u8() - 16),
                };

                //TODO maybe change this to make it less expensive?
                //can only move two-squares if pawn is in starting row, and next two squares are empty
                if blockers.bit_and(&BitBoard::nth(next).bit_or(&BitBoard::nth(target))).is_zero() {
                    // can only move two-squares if not in check, or blocks check
                    if check_mask.is_zero() || check_mask.nth_is_not_zero(target) {
                        moves.push(ChessMove::new(source, target, MoveType::Normal));
                    }
                }
            }
        }

        let attack_mask = match side {
            Side::White => get_w_pawn_attack(source).bit_and(&self.black_blockers()),
            Side::Black => get_b_pawn_attack(source).bit_and(&self.white_blockers()),
        };

        /* pawn attacks */
        // this is equivalent to: !is_pinned_horz && !is_pinned_vert, due to ~p ^ ~q <=> ~(p v q)
        if !(is_pinned_horz || is_pinned_vert) {
            let mut attacks = attack_mask;
            while attacks.is_not_zero() {
                let attack = attacks.lsb_square().unwrap();

                debug_assert!(self.check_bb.count_ones() <= 1);
                //can only attack a square if not in check or attack blocks check
                if check_mask.is_zero() || (check_mask.nth_is_not_zero(attack)) {
                    let is_attack_pinner =
                        pinners.nth_is_not_zero(attack) && is_same_diag_tri(source, attack, king_square);

                    //can only attack a square if not pinned or capturing piece pinning the pawn
                    if pin_mask.is_zero() || is_attack_pinner {
                        match ROWS[attack.to_usize()] == promotion_row {
                            true => moves.append(&mut ChessMove::promotions(source, attack).to_vec()),
                            false => moves.push(ChessMove::new(source, attack, MoveType::Normal)),
                        }
                    }
                }
                attacks.pop_bit(attack);
            }
        }

        /* pawn en-passant */
        if self.enpassant_bb.is_not_zero() && !is_pinned_diag && !is_pinned_horz && !is_pinned_vert {
            let mut attacks = match side {
                Side::White => self.enpassant_bb.bit_and(&get_w_pawn_attack(source)),
                Side::Black => self.enpassant_bb.bit_and(&get_b_pawn_attack(source)),
            };

            while attacks.is_not_zero() {
                let attack = attacks.lsb_square().unwrap();

                //special psuedo-pinned pawn case:
                // R . p P k
                // . . . ^ .
                // . . . | .
                // . . . x .

                //255u64 = 0b11111111u64 is an entire row
                let special_row_bb = BitBoard::new(
                    (255u64 << 8 * ROWS[source.to_usize()]) & (255u64 << 8 * (ROWS[king_square.to_usize()])),
                );
                let (enemy_rook_index, enemy_pawn_index, enemy_pawn_square) = match side {
                    Side::White => (cpt_index!(r), cpt_index!(p), Square::new(attack.to_u8() - 8u8)),
                    Side::Black => (cpt_index!(R), cpt_index!(P), Square::new(attack.to_u8() + 8u8)),
                };

                //if (chessboard.piece_bbs[enemy_rook_index].bit_or(&chessboard.piece_bbs[king_index])).bit_and(&king_row_bb).count_ones() >= 2
                //if enemy rook and friendly king is in the same row, check for special case
                if self.piece_bbs[enemy_rook_index].bit_and(&special_row_bb).is_not_zero() {
                    //NOTE: this is computationally costly
                    //check if en-passant leaves king in check
                    let mut test_cb: ChessBoardCore = self.duplicate();
                    let i = match side {
                        Side::White => cpt_index!(P),
                        Side::Black => cpt_index!(p),
                    };

                    test_cb.piece_bbs[i].pop_bit(source); //remove from source square
                    test_cb.piece_bbs[i].set_bit(attack); //add to attack square
                    test_cb.piece_bbs[enemy_pawn_index].pop_bit(enemy_pawn_square);
                    if test_cb.is_king_in_check(side) {
                        attacks.pop_bit(attack);
                        continue;
                    }

                    //if in check, can only en-passant to remove checking pawn
                    if self.check_bb.count_ones() == 1 {
                        let checker_square = self.check_bb.lsb_square().unwrap();
                        if checker_square == enemy_pawn_square {
                            moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                        }
                        attacks.pop_bit(attack);
                        continue;
                    }

                    //if there are no checks
                    moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                    attacks.pop_bit(attack);
                    continue;
                }

                //if in check, can only en-passant to remove checking pawn
                if self.check_bb.count_ones() == 1 {
                    let checker_square = self.check_bb.lsb_square().unwrap();
                    if checker_square == enemy_pawn_square {
                        moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                    }
                    attacks.pop_bit(attack);
                    continue;
                }

                //if there are no checks
                moves.push(ChessMove::new(source, attack, MoveType::EnPassant));
                attacks.pop_bit(attack);
            }
        }

        return moves;
    }

    pub(super) fn compute_check_bb(&mut self) {
        let blockers: BitBoard = self.blockers();
        match self.side_to_move {
            Side::White => {
                let king_square: Square = self.piece_bbs[0].lsb_square().unwrap();

                let queen_bb: BitBoard = self.piece_bbs[07].bit_and(&get_queen_attack(king_square, blockers));
                let knight_bb: BitBoard = self.piece_bbs[08].bit_and(&get_knight_attack(king_square));
                let bishop_bb: BitBoard = self.piece_bbs[09].bit_and(&get_bishop_attack(king_square, blockers));
                let rook_bb: BitBoard = self.piece_bbs[10].bit_and(&get_rook_attack(king_square, blockers));
                let pawn_bb: BitBoard = self.piece_bbs[11].bit_and(&get_w_pawn_attack(king_square));
                self.check_bb = queen_bb.bit_or(&knight_bb.bit_or(&bishop_bb.bit_or(&rook_bb.bit_or(&pawn_bb))));
            }

            Side::Black => {
                let king_square: Square = self.piece_bbs[6].lsb_square().unwrap();

                let queen_bb: BitBoard = self.piece_bbs[01].bit_and(&get_queen_attack(king_square, blockers));
                let knight_bb: BitBoard = self.piece_bbs[02].bit_and(&get_knight_attack(king_square));
                let bishop_bb: BitBoard = self.piece_bbs[03].bit_and(&get_bishop_attack(king_square, blockers));
                let rook_bb: BitBoard = self.piece_bbs[04].bit_and(&get_rook_attack(king_square, blockers));
                let pawn_bb: BitBoard = self.piece_bbs[05].bit_and(&get_b_pawn_attack(king_square));
                self.check_bb = queen_bb.bit_or(&knight_bb.bit_or(&bishop_bb.bit_or(&rook_bb.bit_or(&pawn_bb))));
            }
        }
    }

    // calculates all squares thats relevant to a check
    pub(super) const fn compute_check_mask(&mut self) {
        let mut check_bb: BitBoard = self.check_bb;
        let mut check_mask = check_bb;
        while check_bb.is_not_zero() {
            let checker_square = check_bb.lsb_square().unwrap();
            check_mask = match self.mailbox[checker_square.to_usize()].unwrap() {
                cpt!(K) | cpt!(k) => panic!("generate_moves: king is in check by another king!"),
                cpt!(N) | cpt!(n) => check_mask.bit_or(&BitBoard::nth(checker_square)),
                _ => check_mask.bit_or(&RAYS[checker_square.to_usize()][self.king_square().to_usize()]),
            };
            check_bb.pop_bit(checker_square);
        }
        self.check_mask = check_mask.bit_or(&check_bb);
    }

    pub(super) const fn compute_pin_data(&mut self) {
        let mut pinner_bb: BitBoard = BitBoard::ZERO;
        let mut pinned_bb: BitBoard = BitBoard::ZERO;

        let friends: BitBoard;
        let enemies: BitBoard;
        let diagonal_enemies: BitBoard;
        let lateral_enemies: BitBoard;
        let king_index: usize;

        match self.side_to_move {
            Side::White => {
                friends = self.white_blockers();
                enemies = self.black_blockers();
                diagonal_enemies = self.piece_bb(cpt!(q)).bit_or(&self.piece_bb(cpt!(b)));
                lateral_enemies = self.piece_bb(cpt!(q)).bit_or(&self.piece_bb(cpt!(r)));
                king_index = 0;
            }
            Side::Black => {
                friends = self.black_blockers();
                enemies = self.white_blockers();
                diagonal_enemies = self.piece_bb(cpt!(Q)).bit_or(&self.piece_bb(cpt!(B)));
                lateral_enemies = self.piece_bb(cpt!(Q)).bit_or(&self.piece_bb(cpt!(R)));
                king_index = 6;
            }
        }

        assert!(self.piece_bbs[king_index].count_ones() == 1);
        let king_square = self.piece_bbs[king_index].lsb_square().unwrap();
        let mut possible_pinners = possible_pinners(king_square, diagonal_enemies, lateral_enemies);
        while possible_pinners.is_not_zero() {
            let possible_pinner = possible_pinners.lsb_square().unwrap();
            let pinner_piece: (Side, PieceType) = self.mailbox[possible_pinner.to_usize()].unwrap();
            let attack_mask = match pinner_piece {
                (_, PieceType::Bishop) => get_bishop_attack(possible_pinner, enemies),
                (_, PieceType::Rook) => get_rook_attack(possible_pinner, enemies),
                (_, PieceType::Queen) => get_queen_attack(possible_pinner, enemies),
                _ => panic!(),
            };

            let relevant_mask: BitBoard =
                RAYS[king_square.to_usize()][possible_pinner.to_usize()].bit_and(&attack_mask);
            let enemy_blockers: BitBoard = relevant_mask.bit_and(&enemies);
            let possible_pinned: BitBoard = relevant_mask.bit_and(&friends);

            //NOTE: a piece is only pinned if and only if it is the only piece between the pinner and the king.
            //      enemy can also block the line of sight.
            if possible_pinned.count_ones() == 1 && enemy_blockers.count_ones() == 0 {
                pinner_bb = pinner_bb.bit_or(&BitBoard::nth(possible_pinner));
                pinned_bb = pinned_bb.bit_or(&possible_pinned);
            }

            possible_pinners.pop_bit(possible_pinner);
        }

        self.pinned_bb = pinned_bb;
        self.pinner_bb = pinner_bb;
    }

    #[inline(always)]
    const fn is_pawn_move_enpassant_relevant(&self, source: &Square, target: &Square) -> bool {
        match self.side() {
            Side::White => {
                (source.to_usize() + 16 == target.to_usize())
                    && ((matches!(self.mailbox[target.to_usize() + 1], opt_cpt!(p)) && (COLS[source.to_usize()] != 7))
                        || matches!(self.mailbox[target.to_usize() - 1], opt_cpt!(p)) && (COLS[source.to_usize()] != 0))
            }
            Side::Black => {
                (source.to_usize() == target.to_usize() + 16)
                    && (matches!(self.mailbox[target.to_usize() + 1], opt_cpt!(P)) && (COLS[source.to_usize()] != 7)
                        || matches!(self.mailbox[target.to_usize() - 1], opt_cpt!(P)) && (COLS[source.to_usize()] != 0))
            }
        }
    }
}

#[inline(always)]
fn is_same_diag_tri(s1: Square, s2: Square, s3: Square) -> bool {
    is_same_adiag(s1, s2) && is_same_adiag(s2, s3) || is_same_ddiag(s1, s2) && is_same_ddiag(s2, s3)
}

#[inline(always)]
fn is_same_col_tri(s1: Square, s2: Square, s3: Square) -> bool {
    is_same_col(s1, s2) && is_same_col(s2, s3)
}

#[inline(always)]
fn is_same_row_tri(s1: Square, s2: Square, s3: Square) -> bool {
    is_same_row(s1, s2) && is_same_row(s2, s3)
}

#[inline(always)]
const fn possible_pinners(k: Square, d: BitBoard, l: BitBoard) -> BitBoard {
    (get_bishop_attack(k, d).bit_and(&d)).bit_or(&get_rook_attack(k, l).bit_and(&l))
}
