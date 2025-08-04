use super::chessmove::*;
use super::*;

impl ChessBoard {
    const fn compute_check_bb(&mut self) {
        let blockers: BitBoard = self.blockers();
        let king_index: usize;
        let king_square: Square;
        match self.side_to_move {
            Side::White => {
                king_index = 0;
                assert!(self.piece_bbs[king_index].count_ones() == 1);
                king_square = self.piece_bbs[king_index].lsb_square().unwrap();

                let queen_bb = self.piece_bbs[07].bit_and(&get_queen_attack(king_square, blockers));
                let knight_bb = self.piece_bbs[08].bit_and(&get_knight_attack(king_square));
                let bishop_bb = self.piece_bbs[09].bit_and(&get_bishop_attack(king_square, blockers));
                let rook_bb = self.piece_bbs[10].bit_and(&get_rook_attack(king_square, blockers));
                let pawn_bb = self.piece_bbs[11].bit_and(&get_b_pawn_attack(king_square));
                self.check_bb = queen_bb.bit_or(&knight_bb.bit_or(&bishop_bb.bit_or(&rook_bb.bit_or(&pawn_bb))));
            }

            Side::Black => {
                king_index = 6;
                assert!(self.piece_bbs[king_index].count_ones() == 1);
                king_square = self.piece_bbs[king_index].lsb_square().unwrap();

                let queen_bb = self.piece_bbs[01].bit_and(&get_queen_attack(king_square, blockers));
                let knight_bb = self.piece_bbs[02].bit_and(&get_knight_attack(king_square));
                let bishop_bb = self.piece_bbs[03].bit_and(&get_bishop_attack(king_square, blockers));
                let rook_bb = self.piece_bbs[04].bit_and(&get_rook_attack(king_square, blockers));
                let pawn_bb = self.piece_bbs[05].bit_and(&get_w_pawn_attack(king_square));
                self.check_bb = queen_bb.bit_or(&knight_bb.bit_or(&bishop_bb.bit_or(&rook_bb.bit_or(&pawn_bb))));
            }
        }
    }

    pub(crate) const fn compute_pin_bb(&mut self) {
        let mut pinner_bb: BitBoard = BitBoard::ZERO;
        let mut pinned_bb: BitBoard = BitBoard::ZERO;

        let friends: BitBoard;
        let enemies: BitBoard;
        let blockers: BitBoard = self.blockers();
        let king_index: usize;
        let king_square: Square;

        match self.side_to_move {
            Side::White => {
                friends = self.white_blockers();
                enemies = self.black_blockers();
                king_index = 0;
            }
            Side::Black => {
                friends = self.black_blockers();
                enemies = self.white_blockers();
                king_index = 6;
            }
        }
        assert!(self.piece_bbs[king_index].count_ones() == 1);
        king_square = self.piece_bbs[king_index].lsb_square().unwrap();
        let mut possible_pinners = get_queen_attack(king_square, enemies);
        while possible_pinners.is_not_zero() {
            let possible_pinner = possible_pinners.lsb_square().unwrap();
            let possible_pinned = RAYS[king_square.to_index()][possible_pinner.to_index()].bit_and(&friends);
            if possible_pinned.count_ones() == 1 {
                pinner_bb = pinner_bb.bit_or(&BitBoard::nth(possible_pinner));
                pinned_bb = pinned_bb.bit_or(&possible_pinned);
            }
            possible_pinners = possible_pinners.pop_bit(possible_pinner);
        }
        self.pinned_bb = pinned_bb;
        self.pinner_bb = pinner_bb;
    }
}

fn update_state(cb: &mut ChessBoard, chess_move: ChessMove) {
    let mut enpassant_bb: BitBoard = BitBoard::ZERO;
    let source: Square = chess_move.source();
    let target: Square = chess_move.target();
    let source_piece = cb.mailbox[source.to_index()].expect("update_state error: source mailbox is None");
    let source_index = cp_index(source_piece);
    let target_piece = cb.mailbox[target.to_index()];
    let target_index = match target_piece {
        Some(target_piece) => Some(cp_index(target_piece)),
        None => None,
    };

    let mut current_hash = cb.hash();
    current_hash ^= ZorbistHash::compute_enpassant_hash(cb.enpassant_bb);

    let mut is_counter_reset: bool = false; //fifty-move-rule counter

    /* special case book keeping */
    match source_piece {
        /* castling */
        (Side::White, PieceType::King) => {
            if cb.castle_bools[0] {
                current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::White));
            }
            cb.castle_bools[0] = false;
            if cb.castle_bools[1] {
                current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::White));
            }
            cb.castle_bools[1] = false;
        }
        (Side::Black, PieceType::King) => {
            if cb.castle_bools[2] {
                current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::Black));
            }
            cb.castle_bools[2] = false;
            if cb.castle_bools[3] {
                current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::Black));
            }
            cb.castle_bools[3] = false;
        }
        (Side::White, PieceType::Rook) => {
            //FIXME
            if source == Square::new(0) {
                if cb.castle_bools[0] {
                    current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::White));
                }
                cb.castle_bools[0] = false;
            } else if source == Square::new(7) {
                if cb.castle_bools[1] {
                    current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::White));
                }
                cb.castle_bools[1] = false
            }
        }
        (Side::Black, PieceType::Rook) => {
            //FIXME
            if source == Square::new(56) {
                if cb.castle_bools[2] {
                    current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::Black));
                }
                cb.castle_bools[2] = false;
            } else if source == Square::new(63) {
                if cb.castle_bools[3] {
                    current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::Black));
                }
                cb.castle_bools[3] = false
            }
        }

        /* en-passant and fifty-move-rule */
        (Side::White, PieceType::Pawn) => {
            //reset 50-move rule
            cb.fifty_move_rule_counter = 0;
            is_counter_reset = true;
            //if move is a 2-square pawn move, update en-passant bitboard
            if source.to_index() + 16 == target.to_index() {
                //FIXME Square::new(_) ugliness, logical ugliness
                if (cb.mailbox[target.to_index() + 1] == opt_cpt!(p)
                    && (cb.is_piece_pinned(Square::new(target.to_u8() + 1)) == false)
                    && (COLS[source.to_index()] != 7))
                    || cb.mailbox[target.to_index() - 1] == opt_cpt!(p)
                        && (cb.is_piece_pinned(Square::new(target.to_u8() - 1)) == false
                            && (COLS[source.to_index()] != 0))
                {
                    enpassant_bb = enpassant_bb.bit_and(&BitBoard::nth(Square::new(target.to_u8() - 8)));
                }
            }
        }
        (Side::Black, PieceType::Pawn) => {
            //reset 50-move rule
            cb.fifty_move_rule_counter = 0;
            is_counter_reset = true;
            //if move is a 2-square pawn move, update en-passant bitboard
            if source.to_index() == target.to_index() + 16 {
                //FIXME Square::new(_) ugliness, logical ugliness
                if (cb.mailbox[target.to_index() + 1] == opt_cpt!(p)
                    && (cb.is_piece_pinned(Square::new(target.to_u8() + 1)) == false)
                    && (COLS[source.to_index()] != 7))
                    || cb.mailbox[target.to_index() - 1] == opt_cpt!(p)
                        && (cb.is_piece_pinned(Square::new(target.to_u8() - 1)) == false
                            && (COLS[source.to_index()] != 0))
                {
                    enpassant_bb = enpassant_bb.bit_and(&BitBoard::nth(Square::new(target.to_u8() - 8)));
                }
            }
        }
        _ => (),
    }

    //move the piece
    cb.piece_bbs[source_index] = cb.piece_bbs[source_index].pop_bit(source);
    cb.piece_bbs[source_index] = cb.piece_bbs[source_index].bit_or(&BitBoard::nth(target));
    current_hash ^= ZorbistHash::piece_hash(source, source_piece);
    current_hash ^= ZorbistHash::piece_hash(target, source_piece);
    cb.mailbox[source.to_index()] = None;
    cb.mailbox[target.to_index()] = Some(source_piece);

    //additional book keeping
    match chess_move.move_type() {
        MoveType::Normal => {
            //dealing with captures
            if let Some(target_piece) = target_piece {
                is_counter_reset = true;
                current_hash ^= ZorbistHash::piece_hash(target, target_piece);

                //reset 50-move rule
                cb.fifty_move_rule_counter = 0;
                is_counter_reset = true;

                //if capturing enemy rook, update castling rights
                match (target_piece, target.to_u8()) {
                    (cpt!(R), 00u8) => {
                        if cb.castle_bools[0] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::White));
                        }
                        cb.castle_bools[0] = false;
                    }
                    (cpt!(R), 07u8) => {
                        if cb.castle_bools[1] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::White));
                        }
                        cb.castle_bools[1] = false;
                    }
                    (cpt!(r), 56u8) => {
                        if cb.castle_bools[2] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::Black));
                        }
                        cb.castle_bools[2] = false;
                    }
                    (cpt!(r), 63u8) => {
                        if cb.castle_bools[3] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::Black));
                        }
                        cb.castle_bools[3] = false;
                    }
                    _ => (),
                }
            }
        }

        MoveType::Castle(castling) => {
            //let white_kingside_rook_sq_source: Square = Square::new(00);
            //let white_kingside_rook_sq_target: Square = Square::new(02);
            //let white_queenside_rook_sq_source: Square = Square::new(07);
            //let white_queenside_rook_sq_target: Square = Square::new(04);
            //let black_kingside_rook_sq_source: Square = Square::new(56);
            //let black_kingside_rook_sq_target: Square = Square::new(58);
            //let black_queenside_rook_sq_source: Square = Square::new(63);
            //let black_queenside_rook_sq_target: Square = Square::new(60);
            let wkr_s: Square = Square::new(00);
            let wkr_t: Square = Square::new(02);
            let wqr_s: Square = Square::new(07);
            let wqr_t: Square = Square::new(04);
            let bkr_s: Square = Square::new(56);
            let bkr_t: Square = Square::new(58);
            let bqr_s: Square = Square::new(63);
            let bqr_t: Square = Square::new(60);
            match castling {
                Castling::KINGSIDE(Side::White) => {
                    // check if rook is present
                    assert!(cb.piece_bbs[04].nth_is_not_zero(wkr_s));
                    cb.piece_bbs[04] = cb.piece_bbs[04].pop_bit(wkr_s);
                    cb.piece_bbs[04] = cb.piece_bbs[04].bit_or(&BitBoard::nth(wkr_t));
                    cb.mailbox[wkr_s.to_index()] = None;
                    cb.mailbox[wkr_t.to_index()] = opt_cpt!(R);

                    //update hash
                    current_hash ^= ZorbistHash::piece_hash(wkr_s, cpt!(R));
                    current_hash ^= ZorbistHash::piece_hash(wkr_t, cpt!(R));
                }
                Castling::QUEENSIDE(Side::White) => {
                    // check if rook is present
                    assert!(cb.piece_bbs[04].nth_is_not_zero(wqr_s));
                    cb.piece_bbs[04] = cb.piece_bbs[04].pop_bit(wqr_s);
                    cb.piece_bbs[04] = cb.piece_bbs[04].bit_or(&BitBoard::nth(wqr_t));
                    cb.mailbox[wqr_s.to_index()] = None;
                    cb.mailbox[wqr_t.to_index()] = opt_cpt!(R);

                    //update hash
                    current_hash ^= ZorbistHash::piece_hash(wqr_s, cpt!(R));
                    current_hash ^= ZorbistHash::piece_hash(wqr_t, cpt!(R));
                }
                Castling::KINGSIDE(Side::Black) => {
                    // check if rook is present
                    assert!(cb.piece_bbs[10].nth_is_not_zero(bkr_s));
                    cb.piece_bbs[10] = cb.piece_bbs[10].pop_bit(bkr_s);
                    cb.piece_bbs[10] = cb.piece_bbs[10].bit_or(&BitBoard::nth(bkr_t));
                    cb.mailbox[bkr_s.to_index()] = None;
                    cb.mailbox[bkr_t.to_index()] = opt_cpt!(R);

                    //update hash
                    current_hash ^= ZorbistHash::piece_hash(bkr_s, cpt!(r));
                    current_hash ^= ZorbistHash::piece_hash(bkr_t, cpt!(r));
                }
                Castling::QUEENSIDE(Side::Black) => {
                    // check if rook is present
                    assert!(cb.piece_bbs[10].nth_is_not_zero(bqr_s));
                    cb.piece_bbs[10] = cb.piece_bbs[10].pop_bit(bqr_s);
                    cb.piece_bbs[10] = cb.piece_bbs[10].bit_or(&BitBoard::nth(bqr_t));
                    cb.mailbox[bqr_s.to_index()] = None;
                    cb.mailbox[bqr_t.to_index()] = opt_cpt!(R);

                    //update hash
                    current_hash ^= ZorbistHash::piece_hash(bqr_s, cpt!(r));
                    current_hash ^= ZorbistHash::piece_hash(bqr_t, cpt!(r));
                }
            }
        }

        MoveType::EnPassant => {
            let enemy_pawn_index: usize;
            let enemy_pawn_square: Square;
            let enemy_piece: ChessPiece;
            match cb.side_to_move {
                Side::White => {
                    enemy_pawn_index = 11;
                    enemy_pawn_square = Square::new((target.to_index() - 8) as u8);
                    enemy_piece = (Side::Black, PieceType::Pawn);
                }
                Side::Black => {
                    enemy_pawn_index = 05;
                    enemy_pawn_square = Square::new((target.to_index() + 8) as u8);
                    enemy_piece = (Side::White, PieceType::Pawn);
                }
            }

            assert!(cb.piece_bbs[enemy_pawn_index].nth_is_not_zero(enemy_pawn_square));
            assert!(
                cb.mailbox[enemy_pawn_square.to_index()] == opt_cpt!(p)
                    || cb.mailbox[enemy_pawn_square.to_index()] == opt_cpt!(P)
            );

            cb.piece_bbs[enemy_pawn_index] = cb.piece_bbs[enemy_pawn_index].pop_bit(enemy_pawn_square);
            current_hash ^= ZorbistHash::piece_hash(enemy_pawn_square, enemy_piece);
            cb.mailbox[enemy_pawn_square.to_index()] = None;
        }

        MoveType::Promotion(piece_type) => {
            let promoted_piece = (cb.side_to_move, piece_type);
            let promoted_index = cp_index(promoted_piece);
            //dealing with captures
            if let Some(target_piece) = target_piece {
                cb.fifty_move_rule_counter = 0;
                is_counter_reset = true;
                current_hash ^= ZorbistHash::piece_hash(target, target_piece);

                //if capturing enemy rook, update castling rights
                match (target_piece, target.to_u8()) {
                    (cpt!(R), 00u8) => {
                        if cb.castle_bools[0] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::White));
                        }
                        cb.castle_bools[0] = false;
                    }
                    (cpt!(R), 07u8) => {
                        if cb.castle_bools[1] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::White));
                        }
                        cb.castle_bools[1] = false;
                    }
                    (cpt!(r), 56u8) => {
                        if cb.castle_bools[2] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::KINGSIDE(Side::Black));
                        }
                        cb.castle_bools[2] = false;
                    }
                    (cpt!(r), 63u8) => {
                        if cb.castle_bools[3] {
                            current_hash ^= ZorbistHash::castle_hash(Castling::QUEENSIDE(Side::Black));
                        }
                        cb.castle_bools[3] = false;
                    }
                    _ => (),
                }
            }
            //remove the pawn piece
            cb.piece_bbs[source_index] = cb.piece_bbs[source_index].pop_bit(target);
            current_hash ^= ZorbistHash::piece_hash(target, source_piece);
            //add the promoted piece
            cb.piece_bbs[promoted_index] = cb.piece_bbs[promoted_index].bit_or(&BitBoard::nth(target));
            current_hash ^= ZorbistHash::piece_hash(target, promoted_piece);
            cb.mailbox[target.to_index()] = Some(promoted_piece);
        }
    }

    cb.side_to_move = cb.side_to_move.update();
    cb.half_move_counter += 1;
    cb.full_move_counter = cb.full_move_counter / 2;
    current_hash ^= ZorbistHash::side_hash();

    cb.enpassant_bb = enpassant_bb;
    current_hash ^= ZorbistHash::compute_enpassant_hash(enpassant_bb);
    cb.zorbist_table.add(current_hash);
    cb.compute_check_bb();
    cb.compute_pin_bb();
    //ChessBoard:
    //done:      piece_bbs: [BitBoard; 12],
    //done:      mailbox: [Option<ChessPiece>; 64],
    //done:      castle_bools: [bool; 4],
    //done:      enpassant_bb: BitBoard, //pieces triggering en-passant rule
    //????:      attacked_bb: BitBoard, //a mask showing all attacked squares
    //done:      check_bb: BitBoard, //pieces triggering check condition
    //done:      pinned_bb: BitBoard, //pieces that are pinned
    //done:      pinner_bb: BitBoard, //pieces doing the pin
    //done:      side_to_move: Side,
    //done:      half_move_counter: u16,
    //done:      full_move_counter: u16,
    //done:      fifty_move_rule_counter: u16,
    //done:      zorbist_table: ZorbistTable,
}

//not const?
const fn const_update_state(chessboard: &ChessBoard, chess_move: ChessMove) -> ChessBoard {
    let mut chessboard = chessboard.duplicate();
    let mut enpassant_bb: BitBoard = BitBoard::ZERO;
    let source: Square = chess_move.source();
    let target: Square = chess_move.target();
    let source_data = chessboard.mailbox[source.to_index()].expect("update_state error: source mailbox is None");
    let source_index = cp_index(source_data);
    let mut current_hash = chessboard.hash();
    todo!();
}

fn generate_moves(chessboard: &ChessBoard) -> Vec<ChessMove> {
    let mut moves: Vec<ChessMove> = Vec::new();
    //if three fold repetition, return empty set of moves
    //TODO

    let side = chessboard.side_to_move;

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
